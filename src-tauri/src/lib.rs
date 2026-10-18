use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, io::{Read, Write}, path::{Path, PathBuf}, process::{Command, Stdio}, time::{Duration, Instant}};
use tauri::Manager;

pub mod brain;
pub mod mcp_access;
pub mod memory_access;
pub mod memory_index;
pub mod memory_semantic;
pub mod lifecycle;
pub mod trust;
pub mod cleanup;
pub mod rpp;

#[derive(Clone, Deserialize, Serialize)]
pub struct Settings {
    pub python: String,
    pub orchestrator_root: PathBuf,
    pub brain_root: PathBuf,
    pub data_root: PathBuf,
    #[serde(default)]
    pub brain_data_root: Option<PathBuf>,
    /// Frozen SecondBrain launcher (installed or staged) that serves RPP; absent means run the source with Python.
    #[serde(default)]
    pub brain_exe: Option<PathBuf>,
    /// Application folder the launcher should treat as its install location (RESPECTED_APP_DIR).
    #[serde(default)]
    pub brain_app_dir: Option<PathBuf>,
}

fn settings_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(|e| e.to_string())?.join("connections.json"))
}

/// Default data_root for a fresh install. An explicit absolute path from
/// RESPECTED_ORCHESTRATOR_DATA_ROOT wins; a saved connections.json still wins
/// over this because get_settings returns it before reaching here. Pure, unit-tested.
fn default_data_root(env_value: Option<&str>, fallback: PathBuf) -> PathBuf {
    match env_value {
        Some(value) if !value.trim().is_empty() && Path::new(value).is_absolute() => PathBuf::from(value),
        _ => fallback,
    }
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let file = settings_file(&app)?;
    if file.exists() {
        return serde_json::from_slice(&fs::read(file).map_err(|e| e.to_string())?).map_err(|e| e.to_string());
    }
    let data_base = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    // Production: no source checkout and no development data folder - products come from the managed, verified copies only.
    let (siblings, data_root) = if trust::dev_enabled() {
        let hub = Path::new(env!("CARGO_MANIFEST_DIR")).parent().ok_or("Missing Hub root")?;
        (hub.parent().ok_or("Missing product root")?.to_path_buf(),
         default_data_root(trust::dev_env("RESPECTED_ORCHESTRATOR_DATA_ROOT").and_then(|v| v.into_string().ok()).as_deref(), data_base.with_file_name("RespectedOrchestratorDevelopment")))
    } else {
        (data_base.join("no-source-checkout"), brain::respected_root().join("RespectedOrchestrator").join("data"))
    };
    Ok(Settings { python: "python".into(), orchestrator_root: siblings.join("respected-orchestrator"),
        brain_root: siblings.join("respected-secondbrain"), data_root,
        brain_data_root: None, brain_exe: None, brain_app_dir: None })
}

fn validate(settings: &Settings) -> Result<(), String> {
    if settings.python.trim().is_empty() || !settings.orchestrator_root.is_absolute() || !settings.data_root.is_absolute() {
        return Err("Explicit product/data paths and Python executable required".into());
    }
    if !settings.orchestrator_root.join("src/respected_orchestrator/__main__.py").is_file() {
        return Err("Independent Orchestrator source not found".into());
    }
    Ok(())
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    // These fields choose which programs the Hub starts (Python, source checkouts, SecondBrain launcher): only a development build
    // lets them be edited, never a webview of an installed product.
    if !trust::dev_enabled() { return Err("Ürün kaynak bağlantıları yalnızca geliştirme derlemelerinde değiştirilir.".into()); }
    validate(&settings)?;
    let file = settings_file(&app)?;
    fs::create_dir_all(file.parent().ok_or("Invalid settings parent")?).map_err(|e| e.to_string())?;
    let temporary = file.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(temporary, file).map_err(|e| e.to_string())
}

pub(crate) struct PumpOutput { pub stdout: Vec<u8>, pub stderr: Vec<u8>, pub success: bool }

pub(crate) enum PumpError { Spawn(String), Missing(&'static str), Io(String), Timeout, Reader(&'static str), TooLarge }

impl PumpError {
    pub(crate) fn message(&self) -> String {
        match self {
            PumpError::Spawn(e) => format!("Engine could not start: {e}"),
            PumpError::Missing(what) => (*what).to_string(),
            PumpError::Io(e) => e.clone(),
            PumpError::Timeout => "Engine request timed out; inspect task state before retrying".into(),
            PumpError::Reader(which) => format!("Engine {which} reader failed"),
            PumpError::TooLarge => "Engine output exceeded the allowed size".into(),
        }
    }
}

/// Kill the engine and everything it started (a plain `kill` leaves grandchildren running and writing files).
fn kill_tree(child: &mut std::process::Child) {
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        let _ = Command::new("taskkill").args(["/T", "/F", "/PID", &child.id().to_string()]).creation_flags(0x08000000).output();
    }
    let _ = child.kill();
    let _ = child.wait();
}

pub(crate) fn pump(mut command: Command, request: Option<Vec<u8>>, timeout: Duration, max_output: usize) -> Result<PumpOutput, PumpError> {
    let mut child = command.spawn().map_err(|e| PumpError::Spawn(e.to_string()))?;
    if let Some(mut input) = child.stdin.take() {
        if let Some(payload) = request {
            if let Err(error) = input.write_all(&payload) { let _ = child.kill(); let _ = child.wait(); return Err(PumpError::Io(error.to_string())); }
        }
    }
    let mut stdout = child.stdout.take().ok_or(PumpError::Missing("Missing stdout"))?;
    let mut stderr = child.stderr.take().ok_or(PumpError::Missing("Missing stderr"))?;
    let limit = max_output.saturating_add(1) as u64;
    let out = std::thread::spawn(move || { let mut bytes = Vec::new(); stdout.take(limit).read_to_end(&mut bytes).map(|_| bytes) });
    let err = std::thread::spawn(move || { let mut bytes = Vec::new(); stderr.take(1 << 20).read_to_end(&mut bytes).map(|_| bytes) });
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| PumpError::Io(e.to_string()))? { break status; }
        if Instant::now() >= deadline { kill_tree(&mut child); return Err(PumpError::Timeout); }
        std::thread::sleep(Duration::from_millis(25));
    };
    // a grandchild that inherited the pipes must not hold the caller (and the engine gate) forever
    let grace = Instant::now() + Duration::from_secs(10);
    while !(out.is_finished() && err.is_finished()) {
        if Instant::now() >= grace { return Err(PumpError::Io("Engine output pipe is still held by a child process".into())); }
        std::thread::sleep(Duration::from_millis(25));
    }
    let bytes = out.join().map_err(|_| PumpError::Reader("output"))?.map_err(|e| PumpError::Io(e.to_string()))?;
    let error_bytes = err.join().map_err(|_| PumpError::Reader("error"))?.map_err(|e| PumpError::Io(e.to_string()))?;
    if bytes.len() > max_output { return Err(PumpError::TooLarge); }
    Ok(PumpOutput { stdout: bytes, stderr: error_bytes, success: status.success() })
}

const MAX_CAMPAIGN_SPEC: usize = 64 * 1024;

/// Campaign identities the engine accepts: `camp-` followed by 1..60 characters of [a-z0-9-].
pub fn valid_campaign_id(id: &str) -> bool {
    id.strip_prefix("camp-").map_or(false, |rest| {
        (1..=60).contains(&rest.len())
            && rest.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    })
}

pub fn call_engine(settings: Settings, action: &str, run_id: Option<String>, request: Option<Value>) -> Result<Value, String> {
    validate(&settings)?;
    if !["status", "create", "start", "list", "inspect", "cancel", "check", "retry", "campaign-start", "campaign-status"].contains(&action) {
        return Err("Unsupported engine operation".into());
    }
    match action {
        "campaign-start" => {
            if run_id.is_some() { return Err("campaign-start does not accept a run identity".into()); }
            let body = request.as_ref().ok_or("campaign-start requires a JSON body")?;
            if serde_json::to_vec(body).map_err(|e| e.to_string())?.len() > MAX_CAMPAIGN_SPEC {
                return Err("Campaign spec exceeds the allowed size".into());
            }
        }
        "campaign-status" => {
            let id = run_id.as_deref().ok_or("campaign-status requires a campaign identity")?;
            if !valid_campaign_id(id) { return Err("Invalid campaign identity".into()); }
        }
        _ => {}
    }
    let mut command = Command::new(&settings.python);
    command.args(["-m", "respected_orchestrator", "--data-root"])
        .arg(&settings.data_root).arg(action).current_dir(rpp::neutral_cwd(&settings))
        .env("PYTHONPATH", settings.orchestrator_root.join("src")).env("PYTHONNOUSERSITE", "1").env_remove("PYTHONHOME").env_remove("PYTHONSTARTUP").env_remove("PYTHONUSERBASE")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(id) = run_id { command.arg(id); }
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let payload = match request { Some(value) => Some(serde_json::to_vec(&value).map_err(|e| e.to_string())?), None => None };
    let output = pump(command, payload, Duration::from_secs(30), usize::MAX).map_err(|e| e.message())?;
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| format!("Invalid engine JSON: {e}; {}", String::from_utf8_lossy(&output.stderr)))?;
    if !output.success { return Err(value.get("error").and_then(Value::as_str).unwrap_or("Engine operation failed").into()); }
    Ok(value)
}

/// Newest-first names of campaign directories under `<data_root>/campaigns`, read-only.
pub fn list_campaigns(settings: &Settings) -> Result<Vec<String>, String> {
    let root = settings.data_root.join("campaigns");
    let entries = match fs::read_dir(&root) { Ok(entries) => entries, Err(_) => return Ok(Vec::new()) };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name = match entry.file_name().into_string() { Ok(name) => name, Err(_) => continue };
        if !valid_campaign_id(&name) { continue; }
        // A symlink or junction is a reparse point: never follow it out of the campaigns root.
        if entry.file_type().map(|kind| kind.is_dir() && !kind.is_symlink()).unwrap_or(false) { names.push(name); }
    }
    names.sort_unstable_by(|a, b| b.cmp(a));
    names.truncate(30);
    Ok(names)
}

#[tauri::command]
async fn engine_request(app: tauri::AppHandle, action: String, run_id: Option<String>, request: Option<Value>) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || call_engine(settings, &action, run_id, request)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn campaign_list(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || list_campaigns(&settings)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn rpp_request(app: tauri::AppHandle, product: String, op: String, payload: Option<Value>, operation_id: Option<String>) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || rpp::rpp_call(&settings, &product, &op, payload, operation_id)).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn product_status(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    let brain = settings.brain_root.join("src/respectedbrain/__init__.py").is_file();
    let orchestrator = settings.orchestrator_root.join("src/respected_orchestrator/__main__.py").is_file();
    let engine = tauri::async_runtime::spawn_blocking(move || call_engine(settings, "status", None, None)).await.map_err(|e| e.to_string())?;
    Ok(json!({ "development": trust::dev_enabled(), "brain_available": brain, "orchestrator_available": orchestrator,
        "hub_version": env!("CARGO_PKG_VERSION"), "engine": engine.unwrap_or_else(|error| json!({"error": error})) }))
}

fn lifecycle_roots(app: &tauri::AppHandle) -> Result<lifecycle::Roots, String> {
    let base = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let absolute = |name: &str| trust::dev_abs(name);
    // Packaged builds ship their bundles beside the app; only a development build may fall back to <ecosystem>/catalog.
    let shipped = app.path().resource_dir().ok().map(|p| p.join("catalog")).filter(|p| p.is_dir());
    let sibling = shipped.or_else(|| if trust::dev_enabled() { Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).map(|p| p.join("catalog")).filter(|p| p.is_dir()) } else { None });
    Ok(lifecycle::Roots {
        managed: absolute("RESPECTED_HUB_MANAGED_ROOT").unwrap_or_else(|| brain::respected_root().join("RespectedHub").join("managed")),
        catalog: absolute("RESPECTED_HUB_CATALOG_ROOT").or(sibling).unwrap_or_else(|| base.join("catalog")),
    })
}

/// Saved connections with each Hub-managed product pointing at its active verified copy.
fn resolved_settings(app: &tauri::AppHandle) -> Result<Settings, String> {
    let mut settings = get_settings(app.clone())?;
    let roots = lifecycle_roots(app)?;
    lifecycle::overlay_roots(&roots, &mut settings.orchestrator_root, &mut settings.brain_root);
    if let Some(python) = lifecycle::managed_python(&roots) { settings.python = python.to_string_lossy().to_string(); }
    brain::overlay(&roots, &brain::dirs(), &mut settings);
    Ok(settings)
}

fn lifecycle_reply(result: Result<Value, lifecycle::LifecycleError>) -> Value {
    match result { Ok(value) => json!({ "ok": true, "result": value, "error": Value::Null }), Err(error) => json!({ "ok": false, "result": Value::Null, "error": error.to_json() }) }
}

/// Verify a staged copy with the product's own RPP answers; scratch data roots keep user data out of it.
pub fn engine_verifier(settings: &Settings) -> impl Fn(&str, &Path) -> Result<(), String> {
    let base = settings.clone();
    move |product, dir| {
        let scratch = std::env::temp_dir().join(format!("hub-verify-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)));
        fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
        if product == "runtime" {
            let python = dir.join("runtime").join("python.exe");
            let mut command = Command::new(&python);
            command.args(["-I", "-c", "import agent_framework, ssl, sqlite3, asyncio"]).current_dir(&scratch).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
            #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
            let outcome = pump(command, None, Duration::from_secs(60), 64 * 1024).map_err(|e| e.message()).and_then(|out| if out.success { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).chars().take(300).collect()) });
            let _ = fs::remove_dir_all(&scratch);
            return outcome;
        }
        let mut settings = base.clone();
        if product == "orchestrator" { settings.orchestrator_root = dir.to_path_buf(); } else { settings.brain_root = dir.to_path_buf(); }
        settings.data_root = scratch.clone();
        settings.brain_data_root = Some(scratch.clone());
        let outcome = (|| {
            for op in ["manifest", "health"] {
                let response = rpp::rpp_call(&settings, product, op, None, None);
                if response["ok"] != true { return Err(format!("{op}: {}", response["error"]["message"].as_str().unwrap_or("yanıt yok"))); }
            }
            if product == "orchestrator" {
                // Record the install in the product's own manifest (position independent) so the product and the Hub agree.
                // Hand the product the verified file contents so its own integrity check (verify) covers them too.
                let mut files = serde_json::Map::new();
                if let Ok(bundle) = lifecycle::read_bundle(dir) {
                    for (path, _, _) in &bundle.files { if let Ok(text) = fs::read_to_string(dir.join(path)) { files.insert(path.clone(), Value::String(text)); } }
                }
                let target = json!({ "action": "install", "target": { "app_root": dir, "files": files } });
                let recorded = rpp::rpp_call_trusted(&settings, product, "lifecycle.apply", Some(target), Some(format!("hub-record-{}", std::process::id())));
                if recorded["ok"] != true { return Err(format!("install record: {}", recorded["error"]["message"].as_str().unwrap_or("yanıt yok"))); }
                let manifest = rpp::rpp_call(&settings, product, "manifest", None, None);
                if manifest["result"]["installed"] != true || manifest["result"]["install_state"] != "installed" { return Err(format!("product reports {}", manifest["result"]["install_state"])); }
            }
            Ok(())
        })();
        let _ = fs::remove_dir_all(&scratch);
        outcome
    }
}

/// Products run on the Hub-managed Python runtime, so a product install first provisions it (own journal
/// entry) when it is missing and a runtime package exists. Without a package the system Python is used.
pub fn apply_with_runtime(roots: &lifecycle::Roots, mut settings: Settings, product: &str, action: &str, operation_id: &str) -> Result<Value, lifecycle::LifecycleError> {
    if matches!(action, "install" | "update") && product != "runtime" && lifecycle::managed_python(roots).is_none() && !lifecycle::catalog(roots, "runtime").0.is_empty() {
        let id: String = format!("rt-{operation_id}").chars().take(64).collect();
        let verifier = engine_verifier(&settings);
        lifecycle::apply(roots, "runtime", "install", &id, &verifier)?;
    }
    if let Some(python) = lifecycle::managed_python(roots) { settings.python = python.to_string_lossy().to_string(); }
    if product == "orchestrator" && action == "uninstall" { mcp_access::disable_all(roots, &settings, operation_id); }
    let verifier = engine_verifier(&settings);
    let applied = lifecycle::apply(roots, product, action, operation_id, &verifier);
    if product == "orchestrator" && applied.is_ok() && matches!(action, "install" | "update" | "repair") { mcp_access::refresh_launcher(roots, &settings); }
    // the MCP launcher stores the versioned Python path: keep it pointing at the live runtime (audit P1 #11)
    if product == "runtime" && applied.is_ok() && matches!(action, "install" | "update" | "repair") && lifecycle::read_current(roots, "orchestrator").is_some() {
        if let Some(python) = lifecycle::managed_python(roots) { settings.python = python.to_string_lossy().to_string(); }
        mcp_access::refresh_launcher(roots, &settings);
    }
    applied
}

/// Component status: SecondBrain comes from its own engine, the rest from the Hub-managed copies.
pub fn hub_status(roots: &lifecycle::Roots, d: &brain::BrainDirs) -> Value {
    let products: Vec<Value> = lifecycle::COMPONENTS.iter().map(|p| if *p == "secondbrain" { brain::status(roots, d) } else { lifecycle::status(roots, p).unwrap_or(Value::Null) }).collect();
    json!({ "ok": true, "result": { "products": products, "managed_root": roots.managed, "catalog_root": roots.catalog, "brain_app_dir": d.app, "brain_data_dir": d.data }, "error": Value::Null })
}

pub fn hub_plan(roots: &lifecycle::Roots, d: &brain::BrainDirs, settings: &Settings, product: &str, action: &str, params: &Value) -> Result<Value, lifecycle::LifecycleError> {
    if product == "secondbrain" { return brain::plan(roots, d, settings, action, params); }
    let mut plan = lifecycle::plan(roots, product, action)?;
    if matches!(action, "install" | "update") && product != "runtime" && lifecycle::managed_python(roots).is_none() && !lifecycle::catalog(roots, "runtime").0.is_empty() {
        if let Some(steps) = plan["plan"]["steps"].as_array_mut() {
            steps.insert(0, json!({ "id": "runtime", "summary": "Önce Hub'ın Python çalışma ortamını kur (sistemde Python gerekmez)", "reversible": true, "irreversible_reason": Value::Null }));
        }
    }
    Ok(plan)
}

pub fn hub_apply(roots: &lifecycle::Roots, d: &brain::BrainDirs, settings: Settings, product: &str, action: &str, params: &Value, id: &str) -> Result<Value, lifecycle::LifecycleError> {
    let applied = hub_apply_inner(roots, d, settings.clone(), product, action, params, id);
    if applied.is_ok() && action == "uninstall" { cleanup::after_uninstall(roots, d, &settings, product, id); }
    applied
}

fn hub_apply_inner(roots: &lifecycle::Roots, d: &brain::BrainDirs, settings: Settings, product: &str, action: &str, params: &Value, id: &str) -> Result<Value, lifecycle::LifecycleError> {
    if product == "secondbrain" {
        let applied = brain::apply(roots, d, &settings, action, params, id);
        if applied.is_ok() && matches!(action, "install" | "update" | "repair") {
            let mut fresh = settings.clone();
            brain::overlay(roots, d, &mut fresh);
            if let Some(command) = rpp::brain_rpp_command(&fresh) {
                memory_access::refresh_command(&|p: &str, o: &str, pl: Option<Value>, i: Option<String>| rpp::rpp_call_trusted(&fresh, p, o, pl, i), command);
            }
        }
        return applied;
    }
    apply_with_runtime(roots, settings, product, action, id)
}

pub fn hub_rollback(roots: &lifecycle::Roots, d: &brain::BrainDirs, id: &str) -> Result<Value, lifecycle::LifecycleError> {
    match lifecycle::read_journal(roots, id) {
        Some(entry) if entry["undo"]["engine"] == true => brain::rollback(roots, d, id),
        _ => lifecycle::rollback(roots, id),
    }
}

#[tauri::command]
async fn lifecycle_status(app: tauri::AppHandle) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || hub_status(&roots, &brain::dirs())).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn lifecycle_plan(app: tauri::AppHandle, product: String, action: String, params: Option<Value>) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(hub_plan(&roots, &brain::dirs(), &settings, &product, &action, &params.unwrap_or(Value::Null)))).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn lifecycle_apply(app: tauri::AppHandle, product: String, action: String, operation_id: String, params: Option<Value>) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(hub_apply(&roots, &brain::dirs(), settings, &product, &action, &params.unwrap_or(Value::Null), &operation_id))).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn lifecycle_rollback(app: tauri::AppHandle, operation_id: String) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let mcp = lifecycle::read_journal(&roots, &operation_id).map_or(false, |e| e["undo"]["mcp"] == true);
        lifecycle_reply(if mcp { mcp_access::rollback(&roots, &settings, &operation_id) } else { hub_rollback(&roots, &brain::dirs(), &operation_id) })
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn orchestrator_mcp_plan(app: tauri::AppHandle, desired: Value) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(mcp_access::plan(&roots, &settings, &desired))).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn orchestrator_mcp_apply(app: tauri::AppHandle, desired: Value, plan_digest: String, operation_id: String) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(mcp_access::apply(&roots, &settings, &desired, &plan_digest, &operation_id))).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn lifecycle_purge_backups(app: tauri::AppHandle, product: String) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(lifecycle::purge_backups(&roots, &product))).await.map_err(|e| e.to_string())
}

fn memory_reply(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => json!({ "ok": true, "result": value, "error": Value::Null }),
        Err(message) => json!({ "ok": false, "result": Value::Null,
            "error": { "code": "MEMORY_ACCESS", "message": message, "hint": "SecondBrain ve Orchestrator'ın kurulu ve güncel olduğunu denetleyin." } }),
    }
}

#[tauri::command]
async fn memory_access_status(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted(&settings, p, o, pl, id);
        memory_reply(Ok(memory_access::status(&call)))
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn memory_access_enable(app: tauri::AppHandle, write: Option<bool>) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted(&settings, p, o, pl, id);
        memory_reply(match rpp::brain_rpp_command(&settings) {
            Some(command) => memory_access::enable(&call, command, write.unwrap_or(false)),
            None => Err("SecondBrain başlatma komutu bulunamadı; SecondBrain'i kurun.".to_string()),
        })
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn memory_access_disable(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted(&settings, p, o, pl, id);
        memory_reply(memory_access::disable(&call))
    }).await.map_err(|e| e.to_string())
}

fn index_reply(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => json!({ "ok": true, "result": value, "error": Value::Null }),
        Err(message) => json!({ "ok": false, "result": Value::Null,
            "error": { "code": "MEMORY_INDEX", "message": message, "hint": "SecondBrain'in kurulu ve güncel olduğunu denetleyin." } }),
    }
}

#[tauri::command]
async fn memory_index_status(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted(&settings, p, o, pl, id);
        index_reply(Ok(memory_index::status(&call)))
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn memory_index_refresh(app: tauri::AppHandle, force: Option<bool>) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        // A refresh can take minutes on a big vault; only that op gets the long timeout.
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted_timeout(&settings, p, o, pl, id,
            if o == "memory.index.refresh" { Duration::from_secs(15 * 60) } else { Duration::from_secs(30) });
        index_reply(memory_index::refresh(&call, force.unwrap_or(false)))
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn memory_semantic_status(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted(&settings, p, o, pl, id);
        index_reply(Ok(memory_semantic::status(&call)))
    }).await.map_err(|e| e.to_string())
}

fn semantic_call<'a>(settings: &'a Settings) -> impl Fn(&str, &str, Option<Value>, Option<String>) -> Value + 'a {
    // The model download and the first vector build are slow; only those ops get long timeouts.
    move |p, o, pl, id| rpp::rpp_call_trusted_timeout(settings, p, o, pl, id, match o {
        "memory.semantic.pull" => Duration::from_secs(65 * 60),
        "memory.index.refresh" => Duration::from_secs(30 * 60),
        _ => Duration::from_secs(30),
    })
}

#[tauri::command]
async fn memory_semantic_enable(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || index_reply(memory_semantic::enable(&semantic_call(&settings)))).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn memory_semantic_disable(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || index_reply(memory_semantic::disable(&semantic_call(&settings)))).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn memory_capture_run(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        // Summarising sessions runs the user's AI CLI; it can take many minutes.
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted_timeout(&settings, p, o, pl, id, Duration::from_secs(30 * 60));
        let reply = call("secondbrain", "memory.capture.run", Some(json!({ "confirm": true })), Some(rpp::generate_operation_id()));
        if reply["ok"] == true { return json!({ "ok": true, "result": reply["result"], "error": Value::Null }); }
        index_reply(Err(format!("{}: {}", reply["error"]["code"].as_str().unwrap_or("UNKNOWN"), reply["error"]["message"].as_str().unwrap_or("yanıt yok"))))
    }).await.map_err(|e| e.to_string())
}

/// The product's own reply (ok/result/error with code, message, hint) for the memory workspace commands.
async fn workspace_call(app: tauri::AppHandle, op: &'static str, payload: Value, with_id: bool, timeout: Duration) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let id = with_id.then(rpp::generate_operation_id);
        let reply = rpp::rpp_call_trusted_timeout(&settings, "secondbrain", op, Some(payload), id, timeout);
        json!({ "ok": reply["ok"] == true, "result": reply["result"], "error": reply["error"] })
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn memory_search(app: tauri::AppHandle, query: String, limit: Option<u32>, mode: Option<String>) -> Result<Value, String> {
    workspace_call(app, "memory.search", json!({ "query": query, "limit": limit.unwrap_or(15), "mode": mode.unwrap_or_else(|| "auto".into()) }), false, Duration::from_secs(60)).await
}

#[tauri::command]
async fn memory_note_read(app: tauri::AppHandle, path: String) -> Result<Value, String> {
    workspace_call(app, "memory.note.read", json!({ "path": path }), false, Duration::from_secs(30)).await
}

#[tauri::command]
async fn memory_capture_note(app: tauri::AppHandle, title: Option<String>, content: String) -> Result<Value, String> {
    workspace_call(app, "memory.capture.note", json!({ "title": title.unwrap_or_default(), "content": content }), true, Duration::from_secs(60)).await
}

#[tauri::command]
async fn memory_briefing_run(app: tauri::AppHandle) -> Result<Value, String> {
    workspace_call(app, "memory.briefing.run", json!({ "confirm": true }), false, Duration::from_secs(30 * 60)).await
}

#[tauri::command]
async fn activity_list(app: tauri::AppHandle, limit: Option<usize>) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || json!({ "ok": true, "result": lifecycle::activity(&roots, limit.unwrap_or(50).min(200)), "error": Value::Null })).await.map_err(|e| e.to_string())
}

/// Product the user downloaded, passed by its installer as `--install=<product>`; anything else is ignored.
fn install_intent(args: impl Iterator<Item = String>) -> Option<String> {
    args.filter_map(|arg| arg.strip_prefix("--install=").map(str::to_string)).find(|p| p == "orchestrator" || p == "secondbrain")
}

#[tauri::command]
fn startup_intent() -> Option<String> { install_intent(std::env::args()) }

/// `respected-hub.exe --uninstall-products` (no window): called by the Hub's uninstaller when the user chose to remove the
/// products too. Returns the process exit code and leaves a log beside the Hub's data.
pub fn headless_uninstall_products() -> i32 {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let base = local.join("com.respected.hub");
    let shipped = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("catalog"))).filter(|p| p.is_dir());
    let roots = lifecycle::Roots {
        managed: trust::dev_abs("RESPECTED_HUB_MANAGED_ROOT").unwrap_or_else(|| brain::respected_root().join("RespectedHub").join("managed")),
        catalog: trust::dev_abs("RESPECTED_HUB_CATALOG_ROOT").or(shipped).unwrap_or_else(|| base.join("catalog")),
    };
    let dirs = brain::dirs();
    let data_root = if trust::dev_enabled() {
        default_data_root(trust::dev_env("RESPECTED_ORCHESTRATOR_DATA_ROOT").and_then(|v| v.into_string().ok()).as_deref(), base.with_file_name("RespectedOrchestratorDevelopment"))
    } else { brain::respected_root().join("RespectedOrchestrator").join("data") };
    let mut settings = Settings { python: "python".into(), orchestrator_root: base.join("no-source-checkout").join("respected-orchestrator"),
        brain_root: base.join("no-source-checkout").join("respected-secondbrain"), data_root, brain_data_root: None, brain_exe: None, brain_app_dir: None };
    lifecycle::overlay_roots(&roots, &mut settings.orchestrator_root, &mut settings.brain_root);
    if let Some(python) = lifecycle::managed_python(&roots) { settings.python = python.to_string_lossy().to_string(); }
    brain::overlay(&roots, &dirs, &mut settings);
    let report = cleanup::remove_products(&roots, &dirs, &settings);
    let log_dir = roots.managed.parent().map(Path::to_path_buf).unwrap_or(base); // beside the managed folder: the Hub's own data folder
    let _ = fs::create_dir_all(&log_dir);
    let _ = fs::write(log_dir.join("uninstall-products.log"), serde_json::to_vec_pretty(&report).unwrap_or_default());
    if report["ok"] == true { 0 } else { 1 }
}

pub fn run() {
    tauri::Builder::default()
        // a second window would run startup recovery against the first one's live install (audit P1 #9): focus the existing one instead
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            use tauri::{Emitter, Manager};
            if let Some(window) = app.webview_windows().values().next() { let _ = window.unminimize(); let _ = window.set_focus(); }
            // a product installer started meanwhile: tell the open window which product the user came to install
            if let Some(product) = install_intent(args.into_iter()) { let _ = app.emit("startup-intent", product); }
        }))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![startup_intent, get_settings, save_settings, engine_request, rpp_request, product_status, campaign_list,
            lifecycle_status, lifecycle_plan, lifecycle_apply, lifecycle_rollback, lifecycle_purge_backups, activity_list,
            orchestrator_mcp_plan, orchestrator_mcp_apply,
            memory_access_status, memory_access_enable, memory_access_disable,
            memory_index_status, memory_index_refresh,
            memory_semantic_status, memory_semantic_enable, memory_semantic_disable, memory_capture_run,
            memory_search, memory_note_read, memory_capture_note, memory_briefing_run])
        // recovery can take minutes (engine call): never hold the window back for it
        .setup(|app| { if let Ok(roots) = lifecycle_roots(app.handle()) { std::thread::spawn(move || { brain::recover_if_interrupted(&roots, &brain::dirs()); let _ = lifecycle::recover(&roots); }); } Ok(()) })
        .run(tauri::generate_context!()).expect("Respected Hub startup failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_intent_accepts_only_known_products() {
        let args = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>().into_iter();
        assert_eq!(install_intent(args(&["hub.exe", "--install=secondbrain"])), Some("secondbrain".into()));
        assert_eq!(install_intent(args(&["hub.exe", "--install=../evil"])), None);
        assert_eq!(install_intent(args(&["hub.exe"])), None);
    }

    #[test]
    fn default_data_root_prefers_absolute_env() {
        let fallback = std::env::temp_dir().join("hub-fallback");
        let custom = std::env::temp_dir().join("hub-custom");
        assert_eq!(default_data_root(Some(custom.to_str().unwrap()), fallback.clone()), custom);
    }

    #[test]
    fn default_data_root_ignores_relative_or_empty_or_missing() {
        let fallback = std::env::temp_dir().join("hub-fallback");
        assert_eq!(default_data_root(Some("relative/data"), fallback.clone()), fallback);
        assert_eq!(default_data_root(Some("   "), fallback.clone()), fallback);
        assert_eq!(default_data_root(None, fallback.clone()), fallback);
    }
}
