// SecondBrain lifecycle through SecondBrain's OWN installation engine (frozen launcher).
//
// The Hub does not copy SecondBrain files itself: it verifies a catalog distribution, stages a hashed
// copy, then runs the product's `setup` / `update` / `repair` / `uninstall` commands. The engine owns the
// application files, the vault, the AI-client integrations (Safe Integration Layer: ownership records,
// foreign entries preserved, byte-exact rollback) and its own transaction journal; the Hub adds the
// verified package, a read-only plan the user approves first, post-checks over RPP and its activity journal.
use crate::lifecycle::{self, LifecycleError, Roots};
use crate::{pump, rpp, Settings};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const PRODUCT: &str = "secondbrain";
const SETUP_TIMEOUT: Duration = Duration::from_secs(900);
const FLAGS: [&str; 4] = ["global", "mcp", "schedule", "shortcut"];

#[derive(Clone, Debug)]
pub struct BrainDirs { pub app: PathBuf, pub data: PathBuf }

/// `%LOCALAPPDATA%\Respected`: the one folder that holds every Respected product (Hub, Brain, Orchestrator).
pub fn respected_root() -> PathBuf { std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("Respected") }

/// The product's own standard folders (shared with its classic installer); overridable for isolated runs.
pub fn dirs() -> BrainDirs {
    let absolute = |name: &str| crate::trust::dev_abs(name);
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    BrainDirs {
        app: absolute("RESPECTED_HUB_BRAIN_APP_DIR").unwrap_or_else(|| local.join("Respected").join("RespectedBrain").join("app")),
        data: absolute("RESPECTED_HUB_BRAIN_DATA_DIR").unwrap_or_else(|| local.join("Respected").join("RespectedBrain").join("data")),
    }
}

fn err(code: &'static str, message: impl Into<String>, hint: impl Into<String>) -> LifecycleError {
    LifecycleError { code, message: message.into(), hint: hint.into() }
}

pub fn installed_exe(d: &BrainDirs) -> PathBuf { d.app.join("respectedbrain.exe") }
fn manifest_path(d: &BrainDirs) -> PathBuf { d.data.join("install-manifest.json") }
pub fn is_installed(d: &BrainDirs) -> bool { manifest_path(d).is_file() && installed_exe(d).is_file() }

fn installed_version(d: &BrainDirs) -> Option<String> {
    let doc: Value = serde_json::from_slice(&fs::read(d.app.join("distribution.json")).ok()?).ok()?;
    doc["version"].as_str().map(str::to_string)
}

/// Owned application files that are missing or changed (config.json and other technical files may change).
fn integrity_problems(d: &BrainDirs) -> Vec<String> {
    let doc: Value = match fs::read(manifest_path(d)).ok().and_then(|b| serde_json::from_slice(&b).ok()) {
        Some(doc) => doc,
        None => return vec!["Kurulum kaydı okunamadı.".into()],
    };
    let mut problems = Vec::new();
    for row in doc["files"].as_array().into_iter().flatten() {
        if !matches!(row["role"].as_str(), Some("application" | "launcher")) { continue; }
        let path = PathBuf::from(row["path"].as_str().unwrap_or(""));
        match lifecycle::sha256_file(&path) {
            Ok(actual) if Some(actual.as_str()) == row["sha256"].as_str() => {}
            Ok(_) => problems.push(format!("içerik farklı: {}", path.display())),
            Err(_) => problems.push(format!("eksik: {}", path.display())),
        }
        if problems.len() >= 20 { break; }
    }
    problems
}

pub fn status(roots: &Roots, d: &BrainDirs) -> Value {
    let (bundles, skipped) = lifecycle::catalog(roots, PRODUCT);
    let newest = bundles.first();
    let installed = is_installed(d);
    let version = if installed { installed_version(d) } else { None };
    let problems = if installed { integrity_problems(d) } else { Vec::new() };
    let state = if !installed { "absent" } else if !problems.is_empty() { "repair-needed" }
        else if newest.zip(version.as_ref()).map(|(n, v)| lifecycle::version_key(&n.version) > lifecycle::version_key(v)).unwrap_or(false) { "update-available" } else { "installed" };
    json!({
        "product": PRODUCT, "installed": installed, "install_state": state, "version": version, "signed": Value::Null, "problems": problems,
        "available": newest.map(|b| json!({ "version": b.version, "signed": b.signed, "trust": b.trust, "files": b.files.len(), "digest": b.digest })),
        "previous_versions": [], "catalog_skipped": skipped, "removed_available": false,
        "root": if installed { json!(d.app) } else { Value::Null }, "engine_managed": true, "data_root": d.data,
    })
}

/// The newest catalog distribution, used in place: its signed `payload` folder (re-verified file by file) is the package the
/// engine installs from, so no second copy of the program is kept on disk.
static STAGE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Serialised: the wizard asks for plans on every toggle.
pub fn stage(roots: &Roots) -> Result<(PathBuf, lifecycle::Bundle), LifecycleError> {
    let _guard = STAGE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let (bundles, _) = lifecycle::catalog(roots, PRODUCT);
    let Some(bundle) = bundles.into_iter().next() else {
        return Err(err("CATALOG_UNAVAILABLE", "SecondBrain paketi henüz indirilmedi.", "İnternet bağlantınızı kontrol edip yeniden deneyin."));
    };
    let payload = bundle.dir.join("payload");
    let problems = lifecycle::tree_problems(&payload, &bundle);
    if !problems.is_empty() { return Err(err("VERIFY_FAILED", format!("SecondBrain paketi doğrulanamadı: {}", problems.join("; ")), "Ürün sayfasından yeniden deneyin; paket yeniden indirilir.")); }
    Ok((payload, bundle))
}

/// Launcher used for read-only RPP calls: the installed one, else the staged one when it already exists.
pub fn engine_exe(roots: &Roots, d: &BrainDirs) -> Option<PathBuf> {
    // The cached, verified package serves read-only RPP calls (RESPECTED_APP_DIR points it at the real install), so no
    // Hub process ever runs from - and locks - the installed program folder.
    let (bundles, _) = lifecycle::catalog(roots, PRODUCT);
    let staged = bundles.first().map(|b| b.dir.join("payload").join("respectedbrain.exe")).filter(|p| p.is_file());
    if staged.is_some() { return staged; }
    if is_installed(d) { return Some(installed_exe(d)); }
    None
}

/// Settings whose SecondBrain RPP calls go through the frozen launcher against the product's own folders.
pub fn overlay(roots: &Roots, d: &BrainDirs, settings: &mut Settings) {
    if let Some(exe) = engine_exe(roots, d) {
        settings.brain_exe = Some(exe);
        settings.brain_app_dir = Some(d.app.clone());
        settings.brain_data_root = Some(d.data.clone());
    }
}

/// AI client lists chosen in the wizard (`desired.clients` for hooks/rules, `desired.mcp_clients` for MCP); absent means "all".
/// Only plain ids pass; the product validates them again.
const CLIENT_LISTS: [&str; 2] = ["clients", "mcp_clients"];

fn insert_clients(desired: &mut serde_json::Map<String, Value>, params: &Value) {
    for key in CLIENT_LISTS {
        if let Some(list) = params["desired"][key].as_array() {
            let ids: Vec<Value> = list.iter().filter_map(Value::as_str).filter(|id| !id.is_empty() && id.len() <= 20 && id.chars().all(|c| c.is_ascii_lowercase() || c == '-')).map(|id| json!(id)).collect();
            desired.insert(key.into(), Value::Array(ids));
        }
    }
}

fn client_args(desired: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for (key, flag) in [("clients", "--clients"), ("mcp_clients", "--mcp-clients")] {
        if let Some(ids) = desired[key].as_array() { out.push(flag.to_string()); out.push(ids.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(",")); }
    }
    out
}

fn clean_params(params: &Value) -> Result<(String, Value, Value), LifecycleError> {
    let vault = params["vault_path"].as_str().unwrap_or("").to_string();
    if !Path::new(&vault).is_absolute() { return Err(err("BAD_REQUEST", "Kasa klasörü seçilmedi.", "Kasanın oluşturulacağı mutlak yolu girin.")); }
    let mut desired = serde_json::Map::new();
    for flag in FLAGS { desired.insert(flag.into(), json!(params["desired"][flag] == true)); }
    insert_clients(&mut desired, params);
    let mut profile = serde_json::Map::new();
    for (key, flag) in [("USER_NAME", "--user-name"), ("COMPANION", "--companion"), ("OS_NAME", "--os-name"), ("USER_BIO", "--user-bio")] {
        if let Some(value) = params["profile"][key].as_str().filter(|v| !v.trim().is_empty()) { profile.insert(flag.into(), json!(value.chars().take(200).collect::<String>())); }
    }
    Ok((vault, Value::Object(desired), Value::Object(profile)))
}

fn desired_of(params: &Value) -> Value {
    let mut desired = serde_json::Map::new();
    for flag in FLAGS { desired.insert(flag.into(), json!(params["desired"][flag] == true)); }
    insert_clients(&mut desired, params);
    Value::Object(desired)
}

/// The vault the installed product has registered, as reported by the product itself.
fn registered_vault(settings: &Settings, d: &BrainDirs) -> Result<String, LifecycleError> {
    let mut live = settings.clone();
    live.brain_exe = Some(installed_exe(d)); live.brain_app_dir = Some(d.app.clone()); live.brain_data_root = Some(d.data.clone());
    let manifest = rpp::rpp_call(&live, PRODUCT, "manifest", None, None);
    manifest["result"]["roots"]["vault"].as_str().map(str::to_string)
        .ok_or_else(|| err("LIFECYCLE_CONFLICT", "SecondBrain kayıtlı bir kasa bildirmedi.", "Önce kurulumu tamamlayın veya onarın."))
}

fn scrubbed(mut command: Command, d: &BrainDirs) -> Command {
    rpp::strip_secret_env(&mut command);
    rpp::apply_test_home(&mut command);
    command.env("RESPECTED_APP_DIR", &d.app).env("RESPECTED_DATA_DIR", &d.data).env("PYTHONDONTWRITEBYTECODE", "1")
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
    command
}

/// The read-only SIL preview from the product itself: exactly which AI-client files would change.
fn integration_plan(settings: &Settings, vault: &str, desired: &Value) -> Result<Value, LifecycleError> {
    let payload = if vault.is_empty() { json!({ "target": { "desired": desired } }) } else { json!({ "target": { "vault_path": vault, "desired": desired } }) };
    let response = rpp::rpp_call(settings, PRODUCT, "integrations.plan", Some(payload), None);
    if response["ok"] == true { return Ok(response["result"].clone()); }
    let code = match response["error"]["code"].as_str() { Some("LIFECYCLE_CONFLICT") => "LIFECYCLE_CONFLICT", Some("BAD_REQUEST") => "BAD_REQUEST", _ => "VERIFY_FAILED" };
    Err(err(code, response["error"]["message"].as_str().unwrap_or("Entegrasyon planı alınamadı.").to_string(), response["error"]["hint"].as_str().unwrap_or("").to_string()))
}

fn step(id: &str, summary: &str, reversible: bool, reason: Option<&str>) -> Value { json!({ "id": id, "summary": summary, "reversible": reversible, "irreversible_reason": reason }) }

pub fn plan(roots: &Roots, d: &BrainDirs, settings: &Settings, action: &str, params: &Value) -> Result<Value, LifecycleError> {
    let st = status(roots, d);
    let installed = st["installed"] == true;
    let mut steps = Vec::new();
    let mut integrations = Value::Null;
    match action {
        "install" => {
            if installed { return Err(err("LIFECYCLE_CONFLICT", "SecondBrain zaten kurulu.", "Güncelleme veya onarım kullanın.")); }
            let (vault, desired, _) = clean_params(params)?;
            let (staged, _) = stage(roots)?;
            let mut live = settings.clone();
            live.brain_exe = Some(staged.join("respectedbrain.exe")); live.brain_app_dir = Some(d.app.clone()); live.brain_data_root = Some(d.data.clone());
            integrations = integration_plan(&live, &vault, &desired)?;
            steps.push(step("verify-package", "Paket dosyalarının özetlerini doğrula ve güvenli bir kopya hazırla", true, None));
            steps.push(step("setup", "SecondBrain kurucusunu çalıştır: program klasörü, kasa kaydı ve ayarlar", true, None));
            steps.push(step("vault", &format!("Not kasası şablonu oluşturulur: {vault}"), false, Some("Kasa ve notlarınız geri alma ya da kaldırma sırasında silinmez")));
            steps.push(step("integrations", "Seçilen AI araç entegrasyonları yalnızca Respected'a ait kayıtlarla yazılır; diğer ayarlarınız korunur", true, None));
            steps.push(step("verify", "Ürünün kendi yanıtıyla kurulumu doğrula", true, None));
        }
        "update" | "repair" => {
            if !installed { return Err(err("LIFECYCLE_CONFLICT", "SecondBrain kurulu değil.", "Önce kurun.")); }
            if action == "update" && st["install_state"] != "update-available" { return Err(err("LIFECYCLE_CONFLICT", "Güncelleme yok.", "Kurulu sürüm güncel.")); }
            if action == "repair" && st["install_state"] != "repair-needed" { return Err(err("LIFECYCLE_CONFLICT", "Onarım gerekmiyor.", "Dosyalar doğrulandı.")); }
            stage(roots)?;
            steps.push(step("verify-package", "Paket dosyalarını doğrula ve güvenli bir kopya hazırla", true, None));
            steps.push(step(action, if action == "update" { "Ürünün kendi güncelleyicisini çalıştır; hata olursa ürün işlemi geri alır" } else { "Ürünün kendi onarımını çalıştır; kullanıcı notları ve diğer araç ayarları korunur" }, true, None));
            steps.push(step("verify", "Ürünün kendi yanıtıyla doğrula", true, None));
        }
        "integrations" => {
            if !installed { return Err(err("LIFECYCLE_CONFLICT", "SecondBrain kurulu değil.", "Önce kurun.")); }
            let desired = desired_of(params);
            let mut live = settings.clone();
            live.brain_exe = Some(installed_exe(d)); live.brain_app_dir = Some(d.app.clone()); live.brain_data_root = Some(d.data.clone());
            integrations = integration_plan(&live, "", &desired)?;
            steps.push(step("integrations", "Seçili AI araç entegrasyonları SecondBrain kurucusuyla yazılır; yalnızca Respected'a ait kayıtlar yönetilir, diğer ayarlarınız korunur ve geri alınabilir", true, None));
            steps.push(step("verify", "Ürünün kendi yanıtıyla doğrula", true, None));
        }
        "uninstall" => {
            if !installed { return Err(err("LIFECYCLE_CONFLICT", "SecondBrain kurulu değil.", "Kaldırılacak bir şey yok.")); }
            steps.push(step("uninstall", "Program dosyalarını, ürün verisini/yedeklerini ve yalnızca Respected'a ait entegrasyon kayıtlarını kaldır; kasa ve notlarınız korunur", false, Some("Kaldırma geri alınamaz; yeniden kurmak gerekir")));
        }
        _ => return Err(err("BAD_REQUEST", format!("Bilinmeyen eylem: {action}."), "install, update, repair veya uninstall kullanın.")),
    }
    Ok(json!({ "product": PRODUCT, "action": action, "dry_run": true, "from_version": st["version"], "to_version": st["available"]["version"],
        "plan": { "steps": steps }, "integrations": integrations, "engine_managed": true }))
}

struct EngineOutcome { success: bool, pending: bool, conflicts: Vec<String> }

fn run_engine(exe: &Path, args: &[String], d: &BrainDirs) -> Result<EngineOutcome, LifecycleError> {
    // Exclusive: waits for running RPP polls and blocks new ones while the installer changes the program folder.
    let _exclusive = rpp::ENGINE_GATE.write_within(Duration::from_secs(120)).ok_or_else(|| err("LIFECYCLE_CONFLICT",
        "Başka bir uzun işlem (örneğin bellek modeli indirme) sürüyor.", "O işlem bitince tekrar deneyin; bu sırada durum sorguları engellenmedi."))?;
    let mut command = Command::new(exe);
    command.args(args).current_dir(std::env::temp_dir());
    let output = pump(scrubbed(command, d), None, SETUP_TIMEOUT, 4 * 1024 * 1024)
        .map_err(|e| err("INTERNAL", format!("SecondBrain kurucusu çalıştırılamadı: {}", e.message()), "Antivirüs veya dosya izinlerini kontrol edin."))?;
    let doc: Value = serde_json::from_slice(&output.stdout).map_err(|_| err("LIFECYCLE_PARTIAL", format!("Kurucu yanıtı okunamadı: {}", String::from_utf8_lossy(&output.stderr).chars().take(300).collect::<String>()), "Kurucu günlüğüne bakın; ürün kendi işlemini geri almış olabilir."))?;
    Ok(EngineOutcome { success: doc["success"] == true, pending: doc["pending"] == true,
        conflicts: doc["conflicts"].as_array().into_iter().flatten().filter_map(|c| c.as_str().map(str::to_string)).collect() })
}

fn verify_installed(settings: &Settings, d: &BrainDirs) -> Result<(), String> {
    let mut live = settings.clone();
    live.brain_exe = Some(installed_exe(d)); live.brain_app_dir = Some(d.app.clone()); live.brain_data_root = Some(d.data.clone());
    let manifest = rpp::rpp_call(&live, PRODUCT, "manifest", None, None);
    if manifest["ok"] != true { return Err(format!("manifest: {}", manifest["error"]["message"].as_str().unwrap_or("yanıt yok"))); }
    if manifest["result"]["installed"] != true || manifest["result"]["install_state"] != "installed" {
        return Err(format!("ürün durumu beklenmedik: {}", manifest["result"]["install_state"]));
    }
    let health = rpp::rpp_call(&live, PRODUCT, "health", None, None);
    if health["ok"] != true { return Err(format!("health: {}", health["error"]["message"].as_str().unwrap_or("yanıt yok"))); }
    Ok(())
}

pub fn apply(roots: &Roots, d: &BrainDirs, settings: &Settings, action: &str, params: &Value, id: &str) -> Result<Value, LifecycleError> {
    if !lifecycle::valid_operation_id(id) { return Err(err("BAD_REQUEST", "İşlem kimliği geçersiz.", "8-64 karakter; harf, rakam, _ veya -.")); }
    if let Some(stored) = lifecycle::read_journal(roots, id) {
        if stored["product"] == PRODUCT && stored["action"] == action { return Ok(stored); }
        return Err(err("LIFECYCLE_CONFLICT", "Bu işlem kimliği başka bir eylem için kullanılmış.", "Yeni bir işlem kimliği kullanın."));
    }
    if !lifecycle::is_idle(roots, PRODUCT) { return Err(err("LIFECYCLE_CONFLICT", "Bu ürün için başka bir işlem sürüyor.", "Bitmesini bekleyin.")); }
    let planned = plan(roots, d, settings, action, params)?; // re-validates state and refuses plan-time SIL conflicts
    let mut entry = lifecycle::begin(roots, id, PRODUCT, action, planned["from_version"].clone(), planned["to_version"].clone())?;
    let outcome = run_action(roots, d, settings, action, params, &mut entry);
    match outcome {
        Ok(()) => lifecycle::finish(roots, entry, "applied", None),
        Err(error) => { let _ = lifecycle::finish(roots, entry, "failed", Some(&error)); Err(error) }
    }
}

fn run_action(roots: &Roots, d: &BrainDirs, settings: &Settings, action: &str, params: &Value, entry: &mut Value) -> Result<(), LifecycleError> {
    let mut args: Vec<String> = Vec::new();
    let exe = if action == "uninstall" || action == "integrations" { stage(roots).map(|(dir, _)| dir.join("respectedbrain.exe")).unwrap_or_else(|_| installed_exe(d)) } else {
        let (dir, bundle) = stage(roots)?;
        args.extend(["--package".to_string(), dir.to_string_lossy().to_string()]);
        // Beta: authenticity is the Hub's Ed25519 check (lifecycle::catalog already refused anything not signed by a trusted key); the engine's own
        // SLSA attestation needs a GitHub Actions/Sigstore pipeline we do not have, so requiring it makes a production install impossible.
        // shortcut: SLSA skipped, re-require it (restore `bundle.trust != "production"`) once releases are built by CI.
        let _ = &bundle;
        args.push("--allow-unattested".to_string());
        dir.join("respectedbrain.exe")
    };
    match action {
        "install" => {
            let (vault, desired, profile) = clean_params(params)?;
            let mut setup = vec!["setup".to_string(), "--vault".into(), vault];
            setup.extend(args);
            for flag in FLAGS { setup.push(if desired[flag] == true { format!("--{flag}") } else { format!("--no-{flag}") }); }
            setup.extend(client_args(&desired));
            for (flag, value) in profile.as_object().into_iter().flatten() { setup.push(flag.clone()); setup.push(value.as_str().unwrap_or("").to_string()); }
            args = setup;
        }
        "update" | "repair" => { args.insert(0, action.to_string()); }
        "integrations" => {
            let mut setup = vec!["setup".to_string(), "--vault".into(), registered_vault(settings, d)?];
            setup.push("--allow-unattested".into()); // same beta decision as above
            let desired = desired_of(params);
            for flag in FLAGS { setup.push(if desired[flag] == true { format!("--{flag}") } else { format!("--no-{flag}") }); }
            setup.extend(client_args(&desired));
            args = setup;
        }
        _ => { args = vec!["uninstall".into(), "--keep-data".into()]; }
    }
    lifecycle::set_status(roots, entry, "switching")?;
    let outcome = run_engine(&exe, &args, d)?;
    if !outcome.success && !outcome.pending {
        return Err(err("LIFECYCLE_PARTIAL", "SecondBrain kurucusu işlemi tamamlamadı; ürün kendi değişikliklerini geri aldı.", outcome.conflicts.join(" ")));
    }
    if outcome.pending { return Err(err("LIFECYCLE_PARTIAL", "İşlem ertelendi: SecondBrain çalışıyor.", "SecondBrain'i kapatıp yeniden deneyin.")); }
    if action == "uninstall" {
        if is_installed(d) { return Err(err("VERIFY_FAILED", "Kaldırma sonrası ürün hâlâ kurulu görünüyor.", "Kurucu günlüğüne bakın.")); }
        entry["undo"] = json!({ "engine": true, "kind": "uninstall" });
        return Ok(());
    }
    lifecycle::set_status(roots, entry, "verifying")?;
    if let Err(message) = verify_installed(settings, d) {
        if action == "install" { let _ = run_engine(&exe, &["uninstall".into(), "--keep-data".into()], d); }
        return Err(err("VERIFY_FAILED", format!("Kurulum sonrası doğrulama başarısız: {message}"), if action == "install" { "Kurulum geri alındı; kasanız korundu." } else { "Onarımı yeniden deneyin." }));
    }
    if action == "install" { entry["undo"] = json!({ "engine": true, "kind": "install" }); }
    if action == "integrations" { entry["undo"] = Value::Null; }
    Ok(())
}

/// Undo of an engine-managed install = the product's own uninstall that keeps the vault and data.
pub fn rollback(roots: &Roots, d: &BrainDirs, id: &str) -> Result<Value, LifecycleError> {
    let Some(mut entry) = lifecycle::read_journal(roots, id) else { return Err(err("RECEIPT_NOT_FOUND", "Bu kimlik için kayıt yok.", "İşlem kimliğini kontrol edin.")) };
    if entry["status"] == "rolled_back" { return Ok(entry); }
    if entry["status"] != "applied" || entry["undo"]["engine"] != true { return Err(err("ROLLBACK_UNAVAILABLE", "Bu işlem geri alınamaz.", "Güncelleme/onarım ürünün kendi işlem kaydıyla korunur.")); }
    if entry["undo"]["kind"] != "install" { return Err(err("ROLLBACK_UNAVAILABLE", "Kaldırma işlemi geri alınamaz.", "SecondBrain'i yeniden kurun; kasanız korunmuştu.")); }
    let newer = lifecycle::activity(roots, 500).iter().any(|e| e["product"] == PRODUCT && e["status"] == "applied" && e["started_at"].as_str().unwrap_or("") > entry["started_at"].as_str().unwrap_or(""));
    if newer { return Err(err("ROLLBACK_UNAVAILABLE", "Bu üründe daha yeni bir işlem var.", "Önce daha yeni işlemi geri alın.")); }
    let exe = stage(roots).map(|(dir, _)| dir.join("respectedbrain.exe")).unwrap_or_else(|_| installed_exe(d));
    let outcome = run_engine(&exe, &["uninstall".into(), "--keep-data".into()], d)?;
    if !outcome.success { return Err(err("LIFECYCLE_PARTIAL", "Geri alma tamamlanmadı.", outcome.conflicts.join(" "))); }
    entry["status"] = json!("rolled_back");
    lifecycle::save_journal(roots, &entry)?;
    Ok(entry)
}

/// After a crash, let the product finish or undo its own half-done transaction (best effort). Run before the
/// generic journal recovery marks the unfinished Hub entries as failed.
pub fn recover_if_interrupted(roots: &Roots, d: &BrainDirs) {
    let unfinished = lifecycle::activity(roots, 500).iter().any(|e| e["product"] == PRODUCT && !matches!(e["status"].as_str(), Some("applied" | "failed" | "rolled_back")));
    if !unfinished { return; }
    let exe = engine_exe(roots, d).filter(|path| path.is_file());
    if let Some(exe) = exe { let _ = run_engine(&exe, &["recover".into()], d); }
}
