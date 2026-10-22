// The Hub's side of product lifecycle: verified installer bundles from the catalog and the Hub's own activity journal.
//
// A product is installed, updated, repaired and removed by its own installer (`<Product>-Windows-Setup.exe`, a catalog bundle of
// kind `installer`: `bundle.json` + `payload/<setup.exe>`, Ed25519 signed). The Hub verifies and runs it (`hub::installer`);
// what is installed is read from the product itself. The journal under `<managed>/journal` is the Hub's own record of what it ran.
pub mod catalog;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PRODUCTS: [&str; 2] = ["orchestrator", "secondbrain"];
pub const ACTIONS: [&str; 4] = ["install", "update", "repair", "uninstall"];
const MAX_FILES: usize = 1;
const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Roots { pub managed: PathBuf, pub catalog: PathBuf }

/// Typed failure: `code` follows RPP names where one exists; message/hint are Turkish.
#[derive(Debug, Clone)]
pub struct LifecycleError { pub code: &'static str, pub message: String, pub hint: String }

fn fail<T>(code: &'static str, message: impl Into<String>, hint: impl Into<String>) -> Result<T, LifecycleError> {
    Err(LifecycleError { code, message: message.into(), hint: hint.into() })
}

impl LifecycleError {
    pub fn to_json(&self) -> Value { json!({ "code": self.code, "message": self.message, "hint": self.hint }) }
}

#[derive(Clone, Debug)]
pub struct Bundle { pub dir: PathBuf, pub product: String, pub kind: String, pub version: String, pub files: Vec<(String, u64, String)>, pub digest: String, pub signed: bool, pub trust: String }

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

pub(crate) fn valid_version(version: &str) -> bool {
    (1..=32).contains(&version.len()) && version.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+'))
        && !version.starts_with('.')
}

pub fn valid_operation_id(id: &str) -> bool {
    (8..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// A bundle file path must be a clean relative path inside the product folder (no drive, no traversal, no backslash).
fn valid_bundle_path(path: &str) -> bool {
    if path.is_empty() || path.chars().any(|c| c.is_control() || matches!(c, '<' | '>' | '"' | '|' | '?' | '*')) || path.contains('\\') || path.contains(':') || path == "bundle.json" || path == ".respected" || path.starts_with(".respected/") { return false; }
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

fn journal_dir(roots: &Roots) -> PathBuf { roots.managed.join("journal") }
fn journal_path(roots: &Roots, id: &str) -> PathBuf { journal_dir(roots).join(format!("{id}.json")) }

// ---------- bundles ----------

pub fn read_bundle(dir: &Path) -> Result<Bundle, LifecycleError> {
    let bad = |m: String| -> Result<Bundle, LifecycleError> { fail("VERIFY_FAILED", format!("Paket geçersiz: {m}"), "Ürün kataloğunu yenileyin; sorun sürerse paketi yeniden indirmek için ürünü onarın.") };
    let bytes = match fs::read(dir.join("bundle.json")) { Ok(b) => b, Err(e) => return bad(format!("bundle.json okunamadı ({e})")) };
    let doc: Value = match serde_json::from_slice(&bytes) { Ok(v) => v, Err(e) => return bad(format!("bundle.json JSON değil ({e})")) };
    if doc["schema"] != 1 { return bad("desteklenmeyen şema".into()); }
    let product = doc["product"].as_str().unwrap_or("").to_string();
    if !PRODUCTS.contains(&product.as_str()) { return bad("bilinmeyen ürün".into()); }
    // a product ships as its own signed installer (one file)
    let kind = doc["kind"].as_str().unwrap_or("").to_string();
    if kind != "installer" { return bad("geçersiz paket türü".into()); }
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
        if !valid_bundle_path(path) || size.is_none() || sha.len() != 64 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) { return bad(format!("geçersiz dosya girdisi '{path}'")); }
        if !seen.insert(path.to_string()) { return bad(format!("yinelenen yol '{path}'")); }
        total += size.unwrap_or(0);
        files.push((path.to_string(), size.unwrap_or(0), sha.to_ascii_lowercase()));
    }
    if total > MAX_TOTAL_BYTES { return bad("paket çok büyük".into()); }
    if files.len() != 1 || !files[0].0.ends_with("-Windows-Setup.exe") { return bad("kurucu paketi tek bir ...-Windows-Setup.exe içermeli".into()); }
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
    Ok(Bundle { dir: dir.to_path_buf(), product, kind, version, files, digest, signed, trust: trust.as_str().to_string() })
}

/// Release order key: SemVer 2 (`0.1.0-beta.2 < 0.1.0-rc.1 < 0.1.0 < 0.1.1-beta.1`) plus the PEP 440 spellings Python
/// packages report (`0.1.0b2` = `0.1.0-beta.2`). Every Hub ever shipped compares versions with this, so it must stay stable.
pub fn version_key(version: &str) -> (Vec<u64>, u8, Vec<(u8, u64, String)>) {
    let text = version.trim().trim_start_matches(['v', 'V']);
    let text = text.split('+').next().unwrap_or("");
    let (core, mut pre) = match text.split_once('-') { Some((c, p)) => (c.to_string(), p.to_string()), None => (text.to_string(), String::new()) };
    let mut core = core;
    if pre.is_empty() {
        // PEP 440: digits then a/b/rc/c and a number, e.g. 0.1.0b1
        if let Some(at) = core.find(|c: char| c.is_ascii_alphabetic()) {
            let (number, suffix) = core.split_at(at);
            let tag = suffix.trim_end_matches(|c: char| c.is_ascii_digit());
            let name = match tag.to_ascii_lowercase().as_str() { "a" | "alpha" => "alpha", "b" | "beta" => "beta", "c" | "rc" => "rc", other => other }.to_string();
            pre = format!("{name}.{}", &suffix[tag.len()..]).trim_end_matches('.').to_string();
            core = number.trim_end_matches('.').to_string();
        }
    }
    let mut numbers: Vec<u64> = core.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    numbers.resize(numbers.len().max(3), 0);
    let ids = pre.split('.').filter(|p| !p.is_empty()).map(|p| match p.parse::<u64>() { Ok(n) => (0, n, String::new()), Err(_) => (1, 0, p.to_ascii_lowercase()) }).collect::<Vec<_>>();
    (numbers, if ids.is_empty() { 1 } else { 0 }, ids)
}

/// Valid installer bundles for `product`, newest first, plus the reasons others were skipped.
pub fn installers(roots: &Roots, product: &str) -> (Vec<Bundle>, Vec<String>) {
    let (mut found, mut skipped) = (Vec::new(), Vec::new());
    if let Ok(entries) = fs::read_dir(&roots.catalog) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() || is_link(&path) || !path.join("bundle.json").is_file() { continue; }
            match read_bundle(&path) {
                Ok(b) if b.product == product => found.push(b),
                Ok(_) => {}
                Err(e) => skipped.push(format!("{}: {}", entry.file_name().to_string_lossy(), e.message)),
            }
        }
    }
    found.sort_by(|a, b| version_key(&b.version).cmp(&version_key(&a.version)));
    (found, skipped)
}

/// Compare `root` (a staged copy of the bundle payload) with the bundle manifest: returns problems.
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
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if is_link(&path) { problems.push(format!("bağlantı yasak: {name}")); continue; }
            if path.is_dir() { stack.push(path); continue; }
            if name == "bundle.json" && path.parent() == Some(root) { continue; }
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

// ---------- journal ----------

pub fn read_journal(roots: &Roots, id: &str) -> Option<Value> { if !valid_operation_id(id) { return None; } serde_json::from_slice(&fs::read(journal_path(roots, id)).ok()?).ok() }

pub fn save_journal(roots: &Roots, entry: &Value) -> Result<(), LifecycleError> {
    let id = entry["operation_id"].as_str().unwrap_or("");
    if !valid_operation_id(id) { return fail("BAD_REQUEST", "Geçersiz işlem kimliği.", "İşlemi yeniden başlatın."); }
    write_atomic(&journal_path(roots, id), &serde_json::to_vec_pretty(entry).unwrap_or_default())
}

fn is_terminal(status: &str) -> bool { matches!(status, "applied" | "failed") }

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

pub fn is_idle(roots: &Roots, product: &str) -> bool { activity(roots, 200).iter().all(|e| e["product"] != product || is_terminal(e["status"].as_str().unwrap_or(""))) }

/// A run the Hub was killed in the middle of (closed, crashed) never finished its journal entry: it is closed as failed at
/// the next start, so the product is not reported as busy forever. The product's own installer keeps the product consistent.
pub fn close_interrupted(roots: &Roots) {
    for mut entry in activity(roots, 200) {
        if is_terminal(entry["status"].as_str().unwrap_or("")) { continue; }
        entry["status"] = json!("failed");
        entry["finished_at"] = json!(now_iso());
        entry["error"] = json!({ "code": "INTERRUPTED", "message": "Hub işlem sürerken kapandı.", "hint": "Ürünün durumunu kontrol edin; gerekirse onarın." });
        let _ = save_journal(roots, &entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_order_follows_semver_and_pep440() {
        let ordered = ["0.1.0-beta.1", "0.1.0b2", "0.1.0-rc.1", "0.1.0", "0.1.1-beta.1", "0.2.0"];
        for pair in ordered.windows(2) { assert!(version_key(pair[0]) < version_key(pair[1]), "{} < {}", pair[0], pair[1]); }
        assert_eq!(version_key("0.1.0b1"), version_key("0.1.0-beta.1"));
    }

    #[test]
    fn only_a_single_installer_file_is_a_bundle() {
        let dir = std::env::temp_dir().join(format!("hub-bundle-{}-{}", std::process::id(), stamp()));
        fs::create_dir_all(dir.join("payload")).unwrap();
        for (kind, path) in [("distribution", "Product-Windows-Setup.exe"), ("installer", "app/tool.exe")] {
            let file = json!({ "path": path, "size": 1, "sha256": "0".repeat(64) });
            let digest = hex(&Sha256::digest(serde_json::to_string(&json!([file.clone()])).unwrap().as_bytes()));
            fs::write(dir.join("bundle.json"), json!({ "schema": 1, "product": "orchestrator", "kind": kind, "version": "0.1.0", "files": [file], "bundle_digest": digest }).to_string()).unwrap();
            assert_eq!(read_bundle(&dir).unwrap_err().code, "VERIFY_FAILED", "{kind} {path}");
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn an_interrupted_entry_is_closed_and_the_product_is_idle_again() {
        let roots = Roots { managed: std::env::temp_dir().join(format!("hub-journal-{}-{}", std::process::id(), stamp())), catalog: PathBuf::new() };
        begin(&roots, "op-journal-1", "secondbrain", "install", Value::Null, Value::Null).unwrap();
        assert!(!is_idle(&roots, "secondbrain") && is_idle(&roots, "orchestrator"));
        close_interrupted(&roots);
        assert_eq!(read_journal(&roots, "op-journal-1").unwrap()["status"], "failed");
        assert!(is_idle(&roots, "secondbrain"));
        let _ = fs::remove_dir_all(&roots.managed);
    }
}
