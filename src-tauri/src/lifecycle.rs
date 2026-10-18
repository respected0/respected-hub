// Hub-managed product lifecycle: install / update / repair / uninstall / rollback / recover.
//
// A product is a verified copy of a catalog bundle (`bundle.json` + `payload/src/...`) placed in
// `<managed>/<product>/versions/<version>`. `current.json` is the single commit point: it is
// written atomically only after every file hash and the product's own RPP answer verified.
// Every operation is journaled under `<managed>/journal`, so an interrupted run is recovered
// (rolled back or confirmed) at the next start. User data roots are never touched here.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PRODUCTS: [&str; 2] = ["orchestrator", "secondbrain"];
/// Everything the lifecycle can manage: the two products plus the Hub-managed Python runtime.
pub const COMPONENTS: [&str; 3] = ["orchestrator", "secondbrain", "runtime"];
pub const ACTIONS: [&str; 4] = ["install", "update", "repair", "uninstall"];
const MAX_FILES: usize = 5000;
const MAX_TOTAL_BYTES: u64 = 200 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Roots { pub managed: PathBuf, pub catalog: PathBuf }

/// Typed failure: `code` follows RPP v1.1 names where one exists; message/hint are Turkish.
#[derive(Debug, Clone)]
pub struct LifecycleError { pub code: &'static str, pub message: String, pub hint: String }

fn fail<T>(code: &'static str, message: impl Into<String>, hint: impl Into<String>) -> Result<T, LifecycleError> {
    Err(LifecycleError { code, message: message.into(), hint: hint.into() })
}

impl LifecycleError {
    pub fn to_json(&self) -> Value { json!({ "code": self.code, "message": self.message, "hint": self.hint }) }
}

#[derive(Clone, Debug)]
pub struct Bundle { pub dir: PathBuf, pub product: String, pub version: String, pub files: Vec<(String, u64, String)>, pub digest: String, pub signed: bool, pub trust: String, pub kind: String }

// ---------- small helpers ----------

/// Epoch milliseconds as fixed-width text, so string order is time order.
fn now_iso() -> String {
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    format!("{ms:013}")
}

pub(crate) fn stamp() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0) }

pub(crate) fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }

pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 { break; }
        hasher.update(&buffer[..read]);
    }
    Ok(hex(&hasher.finalize()))
}

fn valid_version(version: &str) -> bool {
    (1..=32).contains(&version.len()) && version.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+'))
        && !version.starts_with('.')
}

pub fn valid_operation_id(id: &str) -> bool {
    (8..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// A bundle file path must be a clean relative path inside the kind's payload folder (no drive, no
/// traversal, no backslash). Distribution bundles are whole product folders, so any clean path is fine.
fn valid_bundle_path(path: &str, kind: &str) -> bool {
    if path.is_empty() || path.chars().any(|c| c.is_control() || matches!(c, '<' | '>' | '"' | '|' | '?' | '*')) || path.contains('\\') || path.contains(':') || path == "bundle.json" || !match kind { "source" => path.starts_with("src/"), "runtime" => path.starts_with("runtime/"), _ => true } { return false; }
    path.split('/').all(|part| !part.is_empty() && part != "." && part != ".." && Path::new(part).components().all(|c| matches!(c, Component::Normal(_))))
}

/// Windows briefly holds directories (indexer, antivirus, a just-exited process): retry before giving up.
fn rename_retry(from: impl AsRef<Path>, to: impl AsRef<Path>) -> std::io::Result<()> {
    let mut last = None;
    for attempt in 0..8 {
        match fs::rename(from.as_ref(), to.as_ref()) { Ok(()) => return Ok(()), Err(e) => last = Some(e) }
        std::thread::sleep(std::time::Duration::from_millis(150 * (attempt + 1)));
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("rename failed")))
}

fn is_link(path: &Path) -> bool { fs::symlink_metadata(path).map(|m| m.file_type().is_symlink()).unwrap_or(false) }

pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), LifecycleError> {
    let io = |e: std::io::Error| LifecycleError { code: "INTERNAL", message: format!("Dosya yazılamadı: {e}"), hint: "Disk alanını ve izinleri kontrol edin.".into() };
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(io)?; }
    let temporary = path.with_extension(format!("tmp-{}", stamp()));
    fs::write(&temporary, bytes).map_err(io)?;
    rename_retry(&temporary, path).map_err(|e| { let _ = fs::remove_file(&temporary); io(e) })
}

fn product_dir(roots: &Roots, product: &str) -> PathBuf { roots.managed.join(product) }
fn versions_dir(roots: &Roots, product: &str) -> PathBuf { product_dir(roots, product).join("versions") }
pub fn version_dir(roots: &Roots, product: &str, version: &str) -> PathBuf { versions_dir(roots, product).join(version) }
pub(crate) fn removed_dir(roots: &Roots, product: &str) -> PathBuf { product_dir(roots, product).join("removed") }
fn current_path(roots: &Roots, product: &str) -> PathBuf { product_dir(roots, product).join("current.json") }
fn journal_dir(roots: &Roots) -> PathBuf { roots.managed.join("journal") }
fn journal_path(roots: &Roots, id: &str) -> PathBuf { journal_dir(roots).join(format!("{id}.json")) }

fn check_product(product: &str) -> Result<(), LifecycleError> {
    if COMPONENTS.contains(&product) { Ok(()) } else { fail("BAD_REQUEST", format!("Bilinmeyen ürün: {product}."), "orchestrator, secondbrain veya runtime kullanın.") }
}

// ---------- bundles ----------

pub fn read_bundle(dir: &Path) -> Result<Bundle, LifecycleError> {
    let bad = |m: String| -> Result<Bundle, LifecycleError> { fail("VERIFY_FAILED", format!("Paket geçersiz: {m}"), "Paketi tools/build_bundle.py ile yeniden oluşturun.") };
    let bytes = match fs::read(dir.join("bundle.json")) { Ok(b) => b, Err(e) => return bad(format!("bundle.json okunamadı ({e})")) };
    let doc: Value = match serde_json::from_slice(&bytes) { Ok(v) => v, Err(e) => return bad(format!("bundle.json JSON değil ({e})")) };
    if doc["schema"] != 1 { return bad("desteklenmeyen şema".into()); }
    let product = doc["product"].as_str().unwrap_or("").to_string();
    if !COMPONENTS.contains(&product.as_str()) { return bad("bilinmeyen ürün".into()); }
    let kind = doc["kind"].as_str().unwrap_or(if product == "runtime" { "runtime" } else { "source" }).to_string();
    if !["source", "runtime", "distribution"].contains(&kind.as_str()) || (product == "runtime") != (kind == "runtime") { return bad("geçersiz paket türü".into()); }
    let version = doc["version"].as_str().unwrap_or("").to_string();
    if !valid_version(&version) { return bad("geçersiz sürüm".into()); }
    let list = match doc["files"].as_array() { Some(l) if !l.is_empty() && l.len() <= MAX_FILES => l, _ => return bad("dosya listesi boş veya çok büyük".into()) };
    let mut files = Vec::new();
    let mut seen = BTreeSet::new();
    let mut total = 0u64;
    for item in list {
        let path = item["path"].as_str().unwrap_or("");
        let size = item["size"].as_u64();
        let sha = item["sha256"].as_str().unwrap_or("");
        if !valid_bundle_path(path, &kind) || size.is_none() || sha.len() != 64 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) { return bad(format!("geçersiz dosya girdisi '{path}'")); }
        if !seen.insert(path.to_string()) { return bad(format!("yinelenen yol '{path}'")); }
        total += size.unwrap_or(0);
        files.push((path.to_string(), size.unwrap_or(0), sha.to_ascii_lowercase()));
    }
    if total > MAX_TOTAL_BYTES { return bad("paket çok büyük".into()); }
    // The digest binds the exact file list as the builder serialised it (sorted keys, compact).
    let canonical = serde_json::to_string(&Value::Array(list.iter().map(|i| {
        let mut m = serde_json::Map::new(); // BTreeMap-ordered by serde_json default => sorted keys
        m.insert("path".into(), i["path"].clone()); m.insert("sha256".into(), i["sha256"].clone()); m.insert("size".into(), i["size"].clone());
        Value::Object(m)
    }).collect())).unwrap_or_default();
    let digest = hex(&Sha256::digest(canonical.as_bytes()));
    if doc["bundle_digest"].as_str() != Some(digest.as_str()) { return bad("paket özeti dosya listesiyle uyuşmuyor".into()); }
    // fail closed: only a signature from a key this build trusts (or the explicit development opt-in) is accepted
    let trust = match crate::trust::verify(&doc, &product, &kind, &version, &digest) { Ok(t) => t, Err(reason) => return bad(reason) };
    let signed = trust != crate::trust::Trust::UnsignedAllowed;
    Ok(Bundle { dir: dir.to_path_buf(), product, version, files, digest, signed, trust: trust.as_str().to_string(), kind })
}

pub fn version_key(version: &str) -> Vec<u64> { version.split(|c: char| !c.is_ascii_digit()).filter_map(|p| p.parse().ok()).collect() }

/// Valid catalog bundles for `product` (source/runtime copies), newest first, plus the reasons others were skipped.
pub fn catalog(roots: &Roots, product: &str) -> (Vec<Bundle>, Vec<String>) { catalog_kind(roots, product, false) }

/// Same listing for whole-distribution bundles (`distribution == true`) or for the managed-copy kinds.
pub fn catalog_kind(roots: &Roots, product: &str, distribution: bool) -> (Vec<Bundle>, Vec<String>) {
    let (mut found, mut skipped) = (Vec::new(), Vec::new());
    if let Ok(entries) = fs::read_dir(&roots.catalog) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() || is_link(&path) || !path.join("bundle.json").is_file() { continue; }
            match read_bundle(&path) {
                Ok(b) if b.product == product && (b.kind == "distribution") == distribution => found.push(b),
                Ok(_) => {}
                Err(e) => skipped.push(format!("{}: {}", entry.file_name().to_string_lossy(), e.message)),
            }
        }
    }
    found.sort_by(|a, b| version_key(&b.version).cmp(&version_key(&a.version)));
    (found, skipped)
}

/// Compare `root` (a directory holding `src/...`) with the bundle manifest: returns problems.
pub(crate) fn tree_problems(root: &Path, bundle: &Bundle) -> Vec<String> {
    let mut problems = Vec::new();
    let mut expected = BTreeSet::new();
    for (path, size, sha) in &bundle.files {
        expected.insert(path.clone());
        let file = root.join(path);
        if is_link(&file) || !file.is_file() { problems.push(format!("eksik: {path}")); continue; }
        if fs::metadata(&file).map(|m| m.len()).ok() != Some(*size) { problems.push(format!("boyut farklı: {path}")); continue; }
        match sha256_file(&file) { Ok(actual) if &actual == sha => {}, _ => problems.push(format!("içerik farklı: {path}")) }
    }
    let mut stack = vec![if bundle.kind == "distribution" { root.to_path_buf() } else { root.join(if bundle.kind == "runtime" { "runtime" } else { "src" }) }];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if is_link(&path) { problems.push(format!("bağlantı yasak: {name}")); continue; }
            if path.is_dir() { if name != "__pycache__" { stack.push(path); } continue; }
            if name.ends_with(".pyc") || (name == "bundle.json" && path.parent() == Some(root)) { continue; }
            let relative = path.strip_prefix(root).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
            if !expected.contains(&relative) { problems.push(format!("fazladan dosya: {relative}")); }
        }
    }
    problems.truncate(20);
    problems
}

/// Copy the bundle payload into `dest`, verifying each written file's hash.
pub(crate) fn stage_bundle(bundle: &Bundle, dest: &Path) -> Result<(), LifecycleError> {
    let io = |e: std::io::Error| LifecycleError { code: "INTERNAL", message: format!("Dosya kopyalanamadı: {e}"), hint: "Disk alanını ve izinleri kontrol edin.".into() };
    if dest.exists() { fs::remove_dir_all(dest).map_err(io)?; }
    for (path, _, sha) in &bundle.files {
        let source = bundle.dir.join("payload").join(path);
        if is_link(&source) || !source.is_file() { let _ = fs::remove_dir_all(dest); return fail("VERIFY_FAILED", format!("Paket dosyası eksik: {path}"), "Paketi yeniden oluşturun."); }
        let target = dest.join(path);
        fs::create_dir_all(target.parent().unwrap_or(dest)).map_err(io)?;
        fs::copy(&source, &target).map_err(io)?;
        if &sha256_file(&target).map_err(io)? != sha { let _ = fs::remove_dir_all(dest); return fail("VERIFY_FAILED", format!("Kopyalanan dosyanın özeti uyuşmuyor: {path}"), "Paket bozuk olabilir; yeniden oluşturun."); }
    }
    fs::copy(bundle.dir.join("bundle.json"), dest.join("bundle.json")).map_err(io)?;
    Ok(())
}

// ---------- state ----------

pub fn read_current(roots: &Roots, product: &str) -> Option<(String, String)> {
    let doc: Value = serde_json::from_slice(&fs::read(current_path(roots, product)).ok()?).ok()?;
    let version = doc["version"].as_str()?.to_string();
    if !valid_version(&version) { return None; }
    Some((version, doc["bundle_digest"].as_str().unwrap_or("").to_string()))
}

fn write_current(roots: &Roots, product: &str, version: &str, digest: &str) -> Result<(), LifecycleError> {
    write_atomic(&current_path(roots, product), &serde_json::to_vec_pretty(&json!({ "version": version, "bundle_digest": digest, "switched_at": now_iso() })).unwrap_or_default())
}

fn installed_bundle(roots: &Roots, product: &str) -> Option<Bundle> {
    let (version, _) = read_current(roots, product)?;
    read_bundle_installed(&version_dir(roots, product, &version))
}

// The installed copy keeps bundle.json beside src/, but its payload lives directly under the dir.
fn read_bundle_installed(dir: &Path) -> Option<Bundle> { read_bundle(dir).ok() }

pub fn status(roots: &Roots, product: &str) -> Result<Value, LifecycleError> {
    check_product(product)?;
    let (bundles, skipped) = catalog(roots, product);
    let newest = bundles.first();
    let current = read_current(roots, product);
    let (mut state, mut problems, mut signed) = ("absent", Vec::new(), Value::Null);
    if let Some((version, _)) = &current {
        match installed_bundle(roots, product) {
            None => { state = "repair-needed"; problems.push("Kurulu paket bilgisi okunamadı.".to_string()); }
            Some(bundle) => {
                signed = json!(bundle.signed);
                problems = tree_problems(&version_dir(roots, product, version), &bundle);
                state = if !problems.is_empty() { "repair-needed" }
                    else if newest.map(|n| version_key(&n.version) > version_key(version)).unwrap_or(false) { "update-available" } else { "installed" };
            }
        }
    }
    let previous: Vec<String> = fs::read_dir(versions_dir(roots, product)).map(|rd| rd.flatten().filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| valid_version(n) && Some(n) != current.as_ref().map(|c| &c.0)).collect()).unwrap_or_default();
    Ok(json!({
        "product": product, "installed": current.is_some(), "install_state": state,
        "version": current.as_ref().map(|c| c.0.clone()), "signed": signed, "problems": problems,
        "available": newest.map(|b| json!({ "version": b.version, "signed": b.signed, "trust": b.trust, "files": b.files.len(), "digest": b.digest })),
        "previous_versions": previous, "catalog_skipped": skipped,
        "removed_available": removed_dir(roots, product).is_dir() && fs::read_dir(removed_dir(roots, product)).map(|mut r| r.next().is_some()).unwrap_or(false),
        "root": current.as_ref().map(|c| version_dir(roots, product, &c.0).to_string_lossy().to_string()),
    }))
}

// ---------- plan ----------

fn step(id: &str, summary: &str, reversible: bool, reason: Option<&str>) -> Value {
    json!({ "id": id, "summary": summary, "reversible": reversible, "irreversible_reason": reason })
}

pub fn plan(roots: &Roots, product: &str, action: &str) -> Result<Value, LifecycleError> {
    check_product(product)?;
    if !ACTIONS.contains(&action) { return fail("BAD_REQUEST", format!("Bilinmeyen eylem: {action}."), "install, update, repair veya uninstall kullanın."); }
    let st = status(roots, product)?;
    let installed = st["installed"] == true;
    let available = st["available"].clone();
    let mut steps = Vec::new();
    match action {
        "install" | "update" => {
            if action == "install" && installed { return fail("LIFECYCLE_CONFLICT", "Ürün zaten kurulu.", "Güncelleme veya onarım kullanın."); }
            if action == "update" && !installed { return fail("LIFECYCLE_CONFLICT", "Ürün kurulu değil.", "Önce kurun."); }
            if available.is_null() { return fail("BAD_REQUEST", "Katalogda bu ürün için geçerli paket yok.", "tools/build_bundle.py ile paket oluşturup kataloğa koyun."); }
            if action == "update" && st["install_state"] == "repair-needed" { return fail("LIFECYCLE_CONFLICT", "Kurulu kopya bozuk.", "Önce onarın, sonra güncelleyin."); }
            if action == "update" && st["install_state"] != "update-available" { return fail("LIFECYCLE_CONFLICT", "Güncelleme yok; kurulu sürüm güncel.", "Sorun varsa onarımı kullanın."); }
            steps.push(step("stage", "Paket dosyalarını güvenli bir ara klasöre kopyala ve her dosyanın özetini doğrula", true, None));
            steps.push(step("verify", "Ürünün kendi RPP yanıtıyla (manifest + sağlık) yeni kopyayı doğrula", true, None));
            steps.push(step("switch", if action == "install" { "Kurulumu etkinleştir" } else { "Yeni sürüme geç; önceki sürüm geri alma için saklanır" }, true, None));
        }
        "repair" => {
            if !installed { return fail("LIFECYCLE_CONFLICT", "Ürün kurulu değil.", "Önce kurun."); }
            if st["install_state"] != "repair-needed" { return fail("LIFECYCLE_CONFLICT", "Onarım gerekmiyor; dosyalar doğrulandı.", "Sorun yaşıyorsanız ürün sağlığına bakın."); }
            if available.is_null() { return fail("BAD_REQUEST", "Onarım için katalogda paket yok.", "Kurulu sürümün paketini kataloğa koyun."); }
            steps.push(step("stage", "Bozuk/eksik dosyaları katalog paketinden yeniden oluştur", true, None));
            steps.push(step("verify", "Ürünün kendi RPP yanıtıyla doğrula", true, None));
            steps.push(step("swap", "Bozuk kopyayı yedekle ve onarılmışı yerine koy", true, None));
        }
        _ => {
            if !installed { return fail("LIFECYCLE_CONFLICT", "Ürün kurulu değil.", "Kaldırılacak bir şey yok."); }
            if product == "runtime" && read_current(roots, "orchestrator").is_some() { return fail("LIFECYCLE_CONFLICT", "Orchestrator bu Python çalışma ortamını kullanıyor.", "Önce Orchestrator'ı kaldırın."); }
            steps.push(step("remove", "Hub'ın yönettiği program kopyasını ve ürüne ait tüm dosyaları kaldır (geri alma kopyası tutulmaz)", false, Some("Kaldırma geri alınamaz; yeniden kurmak gerekir")));
            steps.push(step("keep-data", "Verileriniz, notlarınız ve çalışma kayıtlarınız silinmez", true, None));
        }
    }
    Ok(json!({ "product": product, "action": action, "dry_run": true, "from_version": st["version"], "to_version": available.get("version").cloned().unwrap_or(Value::Null), "plan": { "steps": steps } }))
}

// ---------- journal ----------

pub fn read_journal(roots: &Roots, id: &str) -> Option<Value> { if !valid_operation_id(id) { return None; } serde_json::from_slice(&fs::read(journal_path(roots, id)).ok()?).ok() }

pub fn save_journal(roots: &Roots, entry: &Value) -> Result<(), LifecycleError> {
    let id = entry["operation_id"].as_str().unwrap_or("");
    if !valid_operation_id(id) { return fail("BAD_REQUEST", "Geçersiz işlem kimliği.", "İşlemi yeniden başlatın."); }
    write_atomic(&journal_path(roots, id), &serde_json::to_vec_pretty(entry).unwrap_or_default())
}

fn is_terminal(status: &str) -> bool { matches!(status, "applied" | "failed" | "rolled_back") }

/// Operation receipts and the user-facing activity feed, newest first.
pub fn activity(roots: &Roots, limit: usize) -> Vec<Value> {
    let mut items: Vec<Value> = fs::read_dir(journal_dir(roots)).map(|rd| rd.flatten().filter_map(|e| serde_json::from_slice(&fs::read(e.path()).ok()?).ok()).collect()).unwrap_or_default();
    items.sort_by(|a, b| b["started_at"].as_str().cmp(&a["started_at"].as_str()).then(b["operation_id"].as_str().cmp(&a["operation_id"].as_str())));
    items.truncate(limit);
    items
}

pub fn begin(roots: &Roots, id: &str, product: &str, action: &str, from: Value, to: Value) -> Result<Value, LifecycleError> {
    let entry = json!({ "operation_id": id, "product": product, "action": action, "status": "staging", "from_version": from, "to_version": to,
        "started_at": now_iso(), "finished_at": Value::Null, "error": Value::Null, "undo": Value::Null });
    save_journal(roots, &entry)?;
    Ok(entry)
}

pub fn finish(roots: &Roots, mut entry: Value, status: &str, error: Option<&LifecycleError>) -> Result<Value, LifecycleError> {
    entry["status"] = json!(status);
    entry["finished_at"] = json!(now_iso());
    entry["error"] = error.map(|e| e.to_json()).unwrap_or(Value::Null);
    save_journal(roots, &entry)?;
    Ok(entry)
}

pub fn set_status(roots: &Roots, entry: &mut Value, status: &str) -> Result<(), LifecycleError> { entry["status"] = json!(status); save_journal(roots, entry) }

// ---------- apply ----------

/// `verify` runs the product's own RPP answer against the staged copy (injected so tests can use fakes).
pub type Verifier<'a> = &'a dyn Fn(&str, &Path) -> Result<(), String>;

pub fn apply(roots: &Roots, product: &str, action: &str, id: &str, verify: Verifier) -> Result<Value, LifecycleError> {
    check_product(product)?;
    if !valid_operation_id(id) { return fail("BAD_REQUEST", "İşlem kimliği geçersiz.", "8-64 karakter; harf, rakam, _ veya -."); }
    if let Some(stored) = read_journal(roots, id) {
        if stored["product"] == product && stored["action"] == action { return Ok(stored); }
        return fail("LIFECYCLE_CONFLICT", "Bu işlem kimliği başka bir eylem için kullanılmış.", "Yeni bir işlem kimliği kullanın.");
    }
    if !is_idle(roots, product) { return fail("LIFECYCLE_CONFLICT", "Bu ürün için başka bir işlem sürüyor.", "Bitmesini bekleyin."); }
    let planned = plan(roots, product, action)?;
    let to = planned["to_version"].clone();
    let mut entry = begin(roots, id, product, action, planned["from_version"].clone(), to.clone())?;
    let result = match action {
        "install" | "update" => do_install_or_update(roots, product, action, &mut entry, verify),
        "repair" => do_repair(roots, product, &mut entry, verify),
        _ => do_uninstall(roots, product, id, &mut entry),
    };
    match result {
        Ok(()) => finish(roots, entry, "applied", None),
        Err(error) => { let _ = finish(roots, entry, "failed", Some(&error)); Err(error) }
    }
}

pub fn is_idle(roots: &Roots, product: &str) -> bool { activity(roots, 200).iter().all(|e| e["product"] != product || is_terminal(e["status"].as_str().unwrap_or(""))) }

fn do_install_or_update(roots: &Roots, product: &str, action: &str, entry: &mut Value, verify: Verifier) -> Result<(), LifecycleError> {
    let (bundles, _) = catalog(roots, product);
    let Some(bundle) = bundles.into_iter().next() else { return fail("BAD_REQUEST", "Katalogda geçerli paket yok.", "Paketi oluşturun.") };
    let target = version_dir(roots, product, &bundle.version);
    let staging = versions_dir(roots, product).join(format!(".staging-{}", stamp()));
    stage_bundle(&bundle, &staging)?;
    let problems = tree_problems(&staging, &bundle);
    if !problems.is_empty() { let _ = fs::remove_dir_all(&staging); return fail("VERIFY_FAILED", format!("Kopya doğrulanamadı: {}", problems.join("; ")), "Yeniden deneyin."); }
    set_status(roots, entry, "verifying")?;
    if let Err(message) = verify(product, &staging) {
        let _ = fs::remove_dir_all(&staging);
        return fail("VERIFY_FAILED", format!("Ürün yeni kopyada çalışmadı: {message}"), "Hiçbir değişiklik uygulanmadı; önceki sürüm etkin kalır.");
    }
    set_status(roots, entry, "switching")?;
    if target.exists() { // same version dir left behind (e.g. earlier rollback): replace it with the verified copy
        let aside = removed_dir(roots, product).join(format!("{}-{}", bundle.version, stamp()));
        fs::create_dir_all(removed_dir(roots, product)).and_then(|_| rename_retry(&target, &aside)).map_err(|e| LifecycleError { code: "INTERNAL", message: format!("Eski kopya taşınamadı: {e}"), hint: "Dosyaların açık olmadığından emin olun.".into() })?;
    }
    rename_retry(&staging, &target).map_err(|e| { let _ = fs::remove_dir_all(&staging); LifecycleError { code: "INTERNAL", message: format!("Kopya etkinleştirilemedi: {e}"), hint: "Dosyaların açık olmadığından emin olun.".into() } })?;
    let previous = read_current(roots, product).map(|c| c.0);
    entry["undo"] = json!({ "kind": action, "previous_version": previous });
    entry["to_version"] = json!(bundle.version);
    write_current(roots, product, &bundle.version, &bundle.digest)?;
    prune_old_versions(roots, product, &bundle.version, previous.as_deref());
    Ok(())
}

/// Keep the current and the previous version; older ones are no longer needed for rollback.
fn prune_old_versions(roots: &Roots, product: &str, current: &str, previous: Option<&str>) {
    if let Ok(rd) = fs::read_dir(versions_dir(roots, product)) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name != current && Some(name.as_str()) != previous && !name.starts_with(".staging-") { let _ = fs::remove_dir_all(e.path()); }
        }
    }
}

fn do_repair(roots: &Roots, product: &str, entry: &mut Value, verify: Verifier) -> Result<(), LifecycleError> {
    let Some((version, _)) = read_current(roots, product) else { return fail("LIFECYCLE_CONFLICT", "Ürün kurulu değil.", "Önce kurun.") };
    let (bundles, _) = catalog(roots, product);
    let Some(bundle) = bundles.into_iter().find(|b| b.version == version) else { return fail("BAD_REQUEST", format!("Kataloğda {version} sürümünün paketi yok."), "Kurulu sürümün paketini kataloğa koyun.") };
    let target = version_dir(roots, product, &version);
    let staging = versions_dir(roots, product).join(format!(".staging-{}", stamp()));
    stage_bundle(&bundle, &staging)?;
    set_status(roots, entry, "verifying")?;
    if let Err(message) = verify(product, &staging) { let _ = fs::remove_dir_all(&staging); return fail("VERIFY_FAILED", format!("Onarılan kopya çalışmadı: {message}"), "Mevcut kurulum olduğu gibi bırakıldı."); }
    set_status(roots, entry, "switching")?;
    let aside = removed_dir(roots, product).join(format!("{version}-broken-{}", stamp()));
    fs::create_dir_all(removed_dir(roots, product)).and_then(|_| rename_retry(&target, &aside)).map_err(|e| LifecycleError { code: "INTERNAL", message: format!("Bozuk kopya taşınamadı: {e}"), hint: "Dosyaların açık olmadığından emin olun.".into() })?;
    if let Err(e) = rename_retry(&staging, &target) { let _ = rename_retry(&aside, &target); let _ = fs::remove_dir_all(&staging); return fail("INTERNAL", format!("Onarım yerleştirilemedi: {e}"), "Eski kopya geri konuldu."); }
    entry["undo"] = json!({ "kind": "repair", "aside": aside.to_string_lossy() });
    Ok(())
}

fn do_uninstall(roots: &Roots, product: &str, id: &str, entry: &mut Value) -> Result<(), LifecycleError> {
    let Some((version, digest)) = read_current(roots, product) else { return fail("LIFECYCLE_CONFLICT", "Ürün kurulu değil.", "Kaldırılacak bir şey yok.") };
    let backup = removed_dir(roots, product).join(id);
    fs::create_dir_all(&backup).and_then(|_| rename_retry(versions_dir(roots, product), backup.join("versions"))).map_err(|e| LifecycleError { code: "INTERNAL", message: format!("Program klasörü taşınamadı: {e}"), hint: "Ürünün çalışmadığından emin olun.".into() })?;
    entry["undo"] = json!({ "kind": "uninstall", "backup": backup.to_string_lossy(), "version": version, "bundle_digest": digest });
    set_status(roots, entry, "switching")?;
    fs::remove_file(current_path(roots, product)).map_err(|e| LifecycleError { code: "INTERNAL", message: format!("Kurulum kaydı silinemedi: {e}"), hint: "Yeniden deneyin; geri alma kullanılabilir.".into() })
}

// ---------- rollback ----------

pub fn rollback(roots: &Roots, id: &str) -> Result<Value, LifecycleError> {
    let Some(mut entry) = read_journal(roots, id) else { return fail("RECEIPT_NOT_FOUND", "Bu kimlik için kayıt yok.", "İşlem kimliğini kontrol edin.") };
    if entry["status"] == "rolled_back" { return Ok(entry); }
    if entry["status"] != "applied" || entry["undo"].is_null() { return fail("ROLLBACK_UNAVAILABLE", "Yalnızca uygulanmış bir işlem geri alınabilir.", "İşlem kaydının durumuna bakın."); }
    let product = entry["product"].as_str().unwrap_or("").to_string();
    check_product(&product)?;
    let started = entry["started_at"].as_str().unwrap_or("").to_string();
    if activity(roots, 500).iter().any(|e| e["product"] == product.as_str() && e["status"] == "applied" && e["started_at"].as_str().unwrap_or("") > started.as_str()) {
        return fail("ROLLBACK_UNAVAILABLE", "Bu üründe daha yeni bir işlem var.", "Önce daha yeni işlemi geri alın.");
    }
    let undo = entry["undo"].clone();
    let io = |e: std::io::Error| LifecycleError { code: "INTERNAL", message: format!("Geri alma başarısız: {e}"), hint: "Dosyaların açık olmadığından emin olun.".into() };
    match undo["kind"].as_str().unwrap_or("") {
        "install" | "update" => {
            match undo["previous_version"].as_str() {
                Some(previous) if version_dir(roots, &product, previous).is_dir() => {
                    let bundle = read_bundle(&version_dir(roots, &product, previous)).map_err(|_| LifecycleError { code: "ROLLBACK_UNAVAILABLE", message: "Önceki sürüm doğrulanamadı.".into(), hint: "Onarım veya yeniden kurulum kullanın.".into() })?;
                    write_current(roots, &product, previous, &bundle.digest)?;
                }
                Some(_) => return fail("ROLLBACK_UNAVAILABLE", "Önceki sürümün kopyası artık yok.", "Güncelleme yapın."),
                None => {
                    let backup = removed_dir(roots, &product).join(format!("rolled-back-{id}"));
                    fs::create_dir_all(&backup).and_then(|_| rename_retry(versions_dir(roots, &product), backup.join("versions"))).map_err(io)?;
                    let _ = fs::remove_file(current_path(roots, &product));
                }
            }
        }
        "repair" => {
            let (version, _) = read_current(roots, &product).ok_or(LifecycleError { code: "ROLLBACK_UNAVAILABLE", message: "Kurulum kaydı yok.".into(), hint: "Yeniden kurun.".into() })?;
            let aside = PathBuf::from(undo["aside"].as_str().unwrap_or(""));
            if !aside.is_dir() { return fail("ROLLBACK_UNAVAILABLE", "Onarım öncesi yedek bulunamadı.", "Gerekirse yeniden onarın."); }
            let target = version_dir(roots, &product, &version);
            let fixed = removed_dir(roots, &product).join(format!("repaired-{id}"));
            rename_retry(&target, &fixed).map_err(io)?;
            rename_retry(&aside, &target).map_err(io)?;
        }
        "uninstall" => {
            let backup = PathBuf::from(undo["backup"].as_str().unwrap_or("")).join("versions");
            if !backup.is_dir() || versions_dir(roots, &product).exists() { return fail("ROLLBACK_UNAVAILABLE", "Kaldırma yedeği bulunamadı veya ürün yeniden kurulmuş.", "Yeniden kurun."); }
            rename_retry(&backup, versions_dir(roots, &product)).map_err(io)?;
            write_current(roots, &product, undo["version"].as_str().unwrap_or(""), undo["bundle_digest"].as_str().unwrap_or(""))?;
        }
        _ => return fail("ROLLBACK_UNAVAILABLE", "Bu işlem türü geri alınamaz.", "Kaydı inceleyin."),
    }
    entry["status"] = json!("rolled_back");
    entry["rolled_back_at"] = json!(now_iso());
    save_journal(roots, &entry)?;
    Ok(entry)
}

/// Permanently delete the retained backups (removed/ and stale staging); the only irreversible step.
pub fn purge_backups(roots: &Roots, product: &str) -> Result<Value, LifecycleError> {
    check_product(product)?;
    let removed = removed_dir(roots, product);
    let count = fs::read_dir(&removed).map(|rd| rd.flatten().count()).unwrap_or(0);
    if removed.is_dir() { fs::remove_dir_all(&removed).map_err(|e| LifecycleError { code: "INTERNAL", message: format!("Yedekler silinemedi: {e}"), hint: "Dosyaların açık olmadığından emin olun.".into() })?; }
    Ok(json!({ "product": product, "removed_entries": count }))
}

// ---------- recovery ----------

/// Resolve operations an interrupted Hub left unfinished. Returns one line per decision.
pub fn recover(roots: &Roots) -> Vec<Value> {
    let mut report = Vec::new();
    for mut entry in activity(roots, 500) {
        let status = entry["status"].as_str().unwrap_or("").to_string();
        if is_terminal(&status) { continue; }
        let product = entry["product"].as_str().unwrap_or("").to_string();
        if !COMPONENTS.contains(&product.as_str()) { continue; }
        let id = entry["operation_id"].as_str().unwrap_or("").to_string();
        let current = read_current(roots, &product);
        let target = entry["to_version"].as_str().map(|v| v.to_string());
        let committed = matches!(entry["action"].as_str(), Some("install" | "update")) && status == "switching" && current.as_ref().map(|c| Some(&c.0) == target.as_ref()).unwrap_or(false)
            && target.as_deref().map(|v| installed_bundle(roots, &product).map(|b| tree_problems(&version_dir(roots, &product, v), &b).is_empty()).unwrap_or(false)).unwrap_or(false);
        let decision = if committed {
            entry["status"] = json!("applied"); "confirmed"
        } else {
            if let Ok(rd) = fs::read_dir(versions_dir(roots, &product)) { for e in rd.flatten() { if e.file_name().to_string_lossy().starts_with(".staging-") { let _ = fs::remove_dir_all(e.path()); } } }
            entry["status"] = json!("failed");
            entry["error"] = json!({ "code": "LIFECYCLE_PARTIAL", "message": "İşlem yarıda kesildi; kurulu sürüm değiştirilmedi.", "hint": "İşlemi yeniden başlatabilirsiniz." });
            "abandoned"
        };
        entry["finished_at"] = json!(now_iso());
        let _ = save_journal(roots, &entry);
        report.push(json!({ "operation_id": id, "decision": decision }));
    }
    report
}

/// Settings with each managed product's root replaced by its active verified copy.
pub fn overlay_roots(roots: &Roots, orchestrator: &mut PathBuf, brain: &mut PathBuf) {
    if let Some((v, _)) = read_current(roots, "orchestrator") { let dir = version_dir(roots, "orchestrator", &v); if dir.join("src").is_dir() { *orchestrator = dir; } }
    if let Some((v, _)) = read_current(roots, "secondbrain") { let dir = version_dir(roots, "secondbrain", &v); if dir.join("src").is_dir() { *brain = dir; } }
}

/// The Hub-managed Python interpreter, when the runtime component is installed and intact on disk.
pub fn managed_python(roots: &Roots) -> Option<PathBuf> {
    let (version, _) = read_current(roots, "runtime")?;
    let python = version_dir(roots, "runtime", &version).join("runtime").join("python.exe");
    python.is_file().then_some(python)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(label: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("hub-lc-{label}-{}-{}", std::process::id(), stamp()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    /// Make a catalog bundle for `product` with the given file contents.
    pub fn make_bundle(catalog: &Path, product: &str, version: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = catalog.join(format!("{product}-{version}"));
        let mut list = Vec::new();
        for (path, text) in files {
            let target = dir.join("payload").join(path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(&target, text).unwrap();
            list.push(json!({ "path": path, "size": text.len(), "sha256": sha256_file(&target).unwrap() }));
        }
        let canonical = serde_json::to_string(&Value::Array(list.clone())).unwrap();
        let digest = hex(&Sha256::digest(canonical.as_bytes()));
        std::env::set_var("RESPECTED_HUB_ALLOW_UNSIGNED", "1"); // these fixtures are hand-made unsigned bundles: the explicit development opt-in
        fs::write(dir.join("bundle.json"), serde_json::to_vec(&json!({ "schema": 1, "product": product, "version": version, "files": list, "bundle_digest": digest, "signature": null })).unwrap()).unwrap();
        dir
    }

    fn roots(label: &str) -> Roots { Roots { managed: temp(&format!("{label}-m")), catalog: temp(&format!("{label}-c")) } }
    fn ok_verify(_: &str, _: &Path) -> Result<(), String> { Ok(()) }
    fn bad_verify(_: &str, _: &Path) -> Result<(), String> { Err("kırık".into()) }
    const FILES: [(&str, &str); 2] = [("src/respected_orchestrator/__main__.py", "print('a')"), ("src/respected_orchestrator/__init__.py", "")];

    #[test]
    fn path_rules_reject_traversal_and_foreign_roots() {
        for good in ["src/a.py", "src/pkg/b.py"] { assert!(valid_bundle_path(good, "source"), "{good}"); }
        for bad in ["", "a.py", "src/../x", "/src/a", "src\\a.py", "C:/x", "src//a", "src/./a", "../src/a"] { assert!(!valid_bundle_path(bad, "source"), "{bad}"); }
    }

    #[test]
    fn install_update_rollback_uninstall_cycle_on_temp_roots() {
        let r = roots("cycle");
        make_bundle(&r.catalog, "orchestrator", "0.1.0", &FILES);
        assert_eq!(status(&r, "orchestrator").unwrap()["install_state"], "absent");
        let done = apply(&r, "orchestrator", "install", "install-0001", &ok_verify).unwrap();
        assert_eq!(done["status"], "applied");
        assert_eq!(status(&r, "orchestrator").unwrap()["install_state"], "installed");
        assert_eq!(apply(&r, "orchestrator", "install", "install-0001", &ok_verify).unwrap()["status"], "applied", "idempotent replay");
        assert_eq!(apply(&r, "orchestrator", "uninstall", "install-0001", &ok_verify).unwrap_err().code, "LIFECYCLE_CONFLICT");

        make_bundle(&r.catalog, "orchestrator", "0.2.0", &[("src/respected_orchestrator/__main__.py", "print('b')")]);
        assert_eq!(status(&r, "orchestrator").unwrap()["install_state"], "update-available");
        apply(&r, "orchestrator", "update", "update-0001", &ok_verify).unwrap();
        assert_eq!(read_current(&r, "orchestrator").unwrap().0, "0.2.0");
        assert!(version_dir(&r, "orchestrator", "0.1.0").is_dir(), "previous version kept for rollback");
        apply(&r, "orchestrator", "uninstall", "remove-0000", &ok_verify).unwrap();
        assert_eq!(rollback(&r, "update-0001").unwrap_err().code, "ROLLBACK_UNAVAILABLE", "only the newest applied operation can be undone");
        rollback(&r, "remove-0000").unwrap();
        assert_eq!(rollback(&r, "update-0001").unwrap()["status"], "rolled_back");
        assert_eq!(read_current(&r, "orchestrator").unwrap().0, "0.1.0");
        assert_eq!(rollback(&r, "update-0001").unwrap()["status"], "rolled_back", "rollback is idempotent");

        apply(&r, "orchestrator", "uninstall", "remove-0001", &ok_verify).unwrap();
        assert!(read_current(&r, "orchestrator").is_none());
        rollback(&r, "remove-0001").unwrap();
        assert_eq!(status(&r, "orchestrator").unwrap()["install_state"], "update-available");
        assert!(purge_backups(&r, "orchestrator").unwrap()["removed_entries"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn failed_verification_changes_nothing() {
        let r = roots("verify");
        make_bundle(&r.catalog, "orchestrator", "0.1.0", &FILES);
        let error = apply(&r, "orchestrator", "install", "install-0002", &bad_verify).unwrap_err();
        assert_eq!(error.code, "VERIFY_FAILED");
        assert!(read_current(&r, "orchestrator").is_none());
        assert!(!version_dir(&r, "orchestrator", "0.1.0").exists());
        assert_eq!(read_journal(&r, "install-0002").unwrap()["status"], "failed");
    }

    #[test]
    fn tampered_bundles_and_trees_are_detected_and_repaired() {
        let r = roots("tamper");
        let dir = make_bundle(&r.catalog, "orchestrator", "0.1.0", &FILES);
        apply(&r, "orchestrator", "install", "install-0003", &ok_verify).unwrap();
        assert_eq!(apply(&r, "orchestrator", "repair", "repair-0001", &ok_verify).unwrap_err().code, "LIFECYCLE_CONFLICT", "healthy tree needs no repair");
        let file = version_dir(&r, "orchestrator", "0.1.0").join("src/respected_orchestrator/__main__.py");
        fs::write(&file, "evil").unwrap();
        fs::write(file.with_file_name("extra.py"), "x").unwrap();
        let st = status(&r, "orchestrator").unwrap();
        assert_eq!(st["install_state"], "repair-needed");
        assert!(st["problems"].as_array().unwrap().len() >= 2);
        apply(&r, "orchestrator", "repair", "repair-0002", &ok_verify).unwrap();
        assert_eq!(status(&r, "orchestrator").unwrap()["install_state"], "installed");
        assert_eq!(rollback(&r, "repair-0002").unwrap()["status"], "rolled_back");
        assert_eq!(status(&r, "orchestrator").unwrap()["install_state"], "repair-needed", "rollback restores the broken copy");

        fs::write(dir.join("payload/src/respected_orchestrator/__main__.py"), "tampered after build").unwrap();
        assert_eq!(apply(&r, "orchestrator", "repair", "repair-0003", &ok_verify).unwrap_err().code, "VERIFY_FAILED");
    }

    #[test]
    fn corrupt_catalog_entries_are_skipped_with_a_reason() {
        let r = roots("catalog");
        let dir = make_bundle(&r.catalog, "orchestrator", "0.1.0", &FILES);
        let mut doc: Value = serde_json::from_slice(&fs::read(dir.join("bundle.json")).unwrap()).unwrap();
        doc["files"][0]["path"] = json!("src/../../escape.py");
        fs::write(dir.join("bundle.json"), serde_json::to_vec(&doc).unwrap()).unwrap();
        let (found, skipped) = catalog(&r, "orchestrator");
        assert!(found.is_empty());
        assert_eq!(skipped.len(), 1);
        assert_eq!(plan(&r, "orchestrator", "install").unwrap_err().code, "BAD_REQUEST");
    }

    #[test]
    fn interrupted_operations_are_recovered_without_changing_the_install() {
        let r = roots("recover");
        make_bundle(&r.catalog, "orchestrator", "0.1.0", &FILES);
        apply(&r, "orchestrator", "install", "install-0004", &ok_verify).unwrap();
        make_bundle(&r.catalog, "orchestrator", "0.2.0", &[("src/respected_orchestrator/__main__.py", "print('b')")]);
        fs::create_dir_all(versions_dir(&r, "orchestrator").join(".staging-1")).unwrap();
        save_journal(&r, &json!({ "operation_id": "update-0002", "product": "orchestrator", "action": "update", "status": "verifying",
            "from_version": "0.1.0", "to_version": "0.2.0", "started_at": "9999999999", "finished_at": null, "error": null, "undo": null })).unwrap();
        let report = recover(&r);
        assert_eq!(report[0]["decision"], "abandoned");
        assert_eq!(read_current(&r, "orchestrator").unwrap().0, "0.1.0");
        assert!(!versions_dir(&r, "orchestrator").join(".staging-1").exists());
        assert_eq!(read_journal(&r, "update-0002").unwrap()["status"], "failed");
        assert!(recover(&r).is_empty());
    }
}
