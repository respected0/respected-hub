use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, io::{Read, Write}, path::{Path, PathBuf}, process::Command, time::{Duration, Instant}};
use tauri::Manager;

pub mod catalog;
pub mod registration;
pub mod products;
// product modules keep their short paths (crate::brain, respected_hub_lib::mcp_access, ...)
pub use products::{brain, mcp_access, memory_access, memory_index, memory_semantic};
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
    /// Packaged Orchestrator (the Hub-managed copy's respected-orchestrator.exe); absent means the source checkout on Python.
    #[serde(default)]
    pub orchestrator_exe: Option<PathBuf>,
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
        brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None })
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
    let stdout = child.stdout.take().ok_or(PumpError::Missing("Missing stdout"))?;
    let stderr = child.stderr.take().ok_or(PumpError::Missing("Missing stderr"))?;
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
    let mut command = rpp::orchestrator_command(&settings, &[action]).ok_or("Orchestrator kurulu değil; Hub'dan kurun.")?;
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
    let orchestrator = settings.orchestrator_exe.as_ref().map_or(false, |exe| exe.is_file()) || settings.orchestrator_root.join("src/respected_orchestrator/__main__.py").is_file();
    let engine = tauri::async_runtime::spawn_blocking(move || call_engine(settings, "status", None, None)).await.map_err(|e| e.to_string())?;
    Ok(json!({ "development": trust::dev_enabled(), "brain_available": brain, "orchestrator_available": orchestrator,
        "hub_version": env!("CARGO_PKG_VERSION"), "engine": engine.unwrap_or_else(|error| json!({"error": error})) }))
}

fn lifecycle_roots(_app: &tauri::AppHandle) -> Result<lifecycle::Roots, String> { Ok(standard_roots()) }

/// `<Respected>\RespectedHub\managed` and its catalog cache (bundles downloaded from GitHub Releases). Only a development build
/// may point elsewhere or fall back to `<ecosystem>/catalog` built by release/tools.
pub fn standard_roots() -> lifecycle::Roots {
    let managed = trust::dev_abs("RESPECTED_HUB_MANAGED_ROOT").unwrap_or_else(|| brain::respected_root().join("RespectedHub").join("managed"));
    let sibling = if trust::dev_enabled() { Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).map(|p| p.join("catalog")).filter(|p| p.is_dir()) } else { None };
    let catalog = trust::dev_abs("RESPECTED_HUB_CATALOG_ROOT").or(sibling).unwrap_or_else(|| managed.join("catalog"));
    lifecycle::Roots { managed, catalog }
}

/// The packaged Orchestrator's launcher inside its Hub-managed copy.
pub const ORCHESTRATOR_EXE: &str = "respected-orchestrator.exe";

/// Point settings at the Hub-managed copies: the packaged Orchestrator and SecondBrain's installed launcher.
pub fn overlay(roots: &lifecycle::Roots, dirs: &brain::BrainDirs, settings: &mut Settings) {
    if let Some(exe) = lifecycle::installed_launcher(roots, "orchestrator", ORCHESTRATOR_EXE) { settings.orchestrator_exe = Some(exe); }
    brain::overlay(roots, dirs, settings);
}

/// Saved connections with each Hub-managed product pointing at its active verified copy.
fn resolved_settings(app: &tauri::AppHandle) -> Result<Settings, String> {
    let mut settings = get_settings(app.clone())?;
    overlay(&lifecycle_roots(app)?, &brain::dirs(), &mut settings);
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
        if product != "orchestrator" { return Err(format!("{product} is installed by its own engine, not verified here")); }
        fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
        let mut settings = base.clone();
        settings.orchestrator_exe = Some(dir.join(ORCHESTRATOR_EXE));
        settings.data_root = scratch.clone();
        settings.brain_data_root = Some(scratch.clone());
        let outcome = (|| {
            for op in ["manifest", "health"] {
                let response = rpp::rpp_call(&settings, product, op, None, None);
                if response["ok"] != true { return Err(format!("{op}: {}", response["error"]["message"].as_str().unwrap_or("yanıt yok"))); }
            }
            {
                // The product records the install in its own manifest from its distribution.json (every file hash checked
                // again on its side), so the product and the Hub agree on what is installed and when it needs repair.
                let target = json!({ "action": "install", "target": { "app_root": dir, "distribution": true } });
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

/// Install, update, repair or uninstall the Hub-managed packaged Orchestrator; its MCP registrations follow the copy.
pub fn apply_orchestrator(roots: &lifecycle::Roots, settings: Settings, action: &str, operation_id: &str) -> Result<Value, lifecycle::LifecycleError> {
    if action == "uninstall" { mcp_access::disable_all(roots, &settings, operation_id); }
    let applied = lifecycle::apply(roots, "orchestrator", action, operation_id, &engine_verifier(&settings));
    // the stable MCP launcher names the versioned executable: rewrite it whenever the active copy changes
    if applied.is_ok() && matches!(action, "install" | "update" | "repair") { mcp_access::refresh_launcher(roots, &settings); }
    applied
}

/// Component status: SecondBrain comes from its own engine, the rest from the Hub-managed copies.
pub fn hub_status(roots: &lifecycle::Roots, d: &brain::BrainDirs) -> Value {
    let products: Vec<Value> = lifecycle::PRODUCTS.iter().map(|p| {
        let mut status = if *p == "secondbrain" { brain::status(roots, d) } else { lifecycle::status(roots, p).unwrap_or(Value::Null) };
        catalog::overlay_status(roots, &mut status);
        status
    }).collect();
    json!({ "ok": true, "result": { "products": products, "managed_root": roots.managed, "catalog_root": roots.catalog, "brain_app_dir": d.app, "brain_data_dir": d.data }, "error": Value::Null })
}

/// Download (once) the bundles an action needs: the newest release for install/update, the installed version for repair.
fn ensure_bundles(roots: &lifecycle::Roots, d: &brain::BrainDirs, product: &str, action: &str) -> Result<(), lifecycle::LifecycleError> {
    match action {
        "install" | "update" => {
            catalog::ensure_bundle(roots, product, None)?;
        }
        "repair" => {
            let installed = if product == "secondbrain" { brain::status(roots, d)["version"].as_str().map(str::to_string) } else { lifecycle::read_current(roots, product).map(|c| c.0) };
            catalog::ensure_bundle(roots, product, installed.as_deref())?;
        }
        _ => {}
    }
    Ok(())
}

pub fn hub_plan(roots: &lifecycle::Roots, d: &brain::BrainDirs, settings: &Settings, product: &str, action: &str, params: &Value) -> Result<Value, lifecycle::LifecycleError> {
    ensure_bundles(roots, d, product, action)?;
    if product == "secondbrain" { return brain::plan(roots, d, settings, action, params); }
    lifecycle::plan(roots, product, action)
}

pub fn hub_apply(roots: &lifecycle::Roots, d: &brain::BrainDirs, settings: Settings, product: &str, action: &str, params: &Value, id: &str) -> Result<Value, lifecycle::LifecycleError> {
    ensure_bundles(roots, d, product, action)?;
    let applied = hub_apply_inner(roots, d, settings.clone(), product, action, params, id);
    if applied.is_ok() {
        if action == "uninstall" { cleanup::after_uninstall(roots, d, &settings, product, id); } else { sweep(roots, d); }
        registration::sync(roots, d);
    }
    applied
}

/// No leftovers: only the installed version of each component stays, in the managed folder and in the catalog cache.
pub fn sweep(roots: &lifecycle::Roots, d: &brain::BrainDirs) {
    for product in lifecycle::PRODUCTS {
        lifecycle::prune_old_versions(roots, product);
        let installed = if product == "secondbrain" { brain::status(roots, d)["version"].as_str().map(str::to_string) } else { lifecycle::read_current(roots, product).map(|c| c.0) };
        if let Some(version) = installed { catalog::prune(roots, product, &version); }
    }
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
    apply_orchestrator(roots, settings, action, id)
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
    let reply = tauri::async_runtime::spawn_blocking({ let roots = roots.clone(); move || lifecycle_reply(hub_apply(&roots, &brain::dirs(), settings, &product, &action, &params.unwrap_or(Value::Null), &operation_id)) }).await.map_err(|e| e.to_string())?;
    // left behind by a kept-products Hub uninstall: once nothing is installed it has no job, so it removes itself
    if registration::running_from_maintenance(&roots) && registration::sync(&roots, &brain::dirs()) == 0 {
        registration::retire_maintenance(&roots);
        std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(2500)); app.exit(0); });
    }
    Ok(reply)
}

#[tauri::command]
async fn catalog_refresh(app: tauri::AppHandle) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(catalog::refresh(&roots).map(|_| catalog::hub_update(&roots)))).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn catalog_set_channel(app: tauri::AppHandle, channel: String) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(catalog::set_channel(&roots, &channel).map(|_| catalog::hub_update(&roots)))).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn hub_update_status(app: tauri::AppHandle) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || json!({ "ok": true, "result": catalog::hub_update(&roots), "error": Value::Null })).await.map_err(|e| e.to_string())
}

/// The person confirmed "install the Hub update": start the verified installer and close so it can replace this executable.
#[tauri::command]
async fn hub_update_install(app: tauri::AppHandle) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    let reply = tauri::async_runtime::spawn_blocking(move || lifecycle_reply(catalog::install_hub_update(&roots).map(|_| json!({ "started": true })))).await.map_err(|e| e.to_string())?;
    if reply["ok"] == true { std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(800)); app.exit(0); }); }
    Ok(reply)
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

/// What the person came for: `--install=<product>` (product installer), `--uninstall=<product>` / `--manage=<product>`
/// (Windows Installed apps). Unknown products and actions are ignored.
fn intent(args: impl Iterator<Item = String>) -> Option<Value> {
    args.filter_map(|arg| ["install", "uninstall", "manage"].into_iter().find_map(|action| arg.strip_prefix(&format!("--{action}=")).map(|p| (action, p.to_string()))))
        .find(|(_, product)| lifecycle::PRODUCTS.contains(&product.as_str()))
        .map(|(action, product)| json!({ "action": action, "product": product }))
}

#[tauri::command]
fn startup_intent() -> Option<Value> { intent(std::env::args()) }

/// `respected-hub.exe --uninstall-products` (no window): called by the Hub's uninstaller when the user chose to remove the
/// products too. Returns the process exit code and leaves a log beside the Hub's data.
pub fn headless_uninstall_products() -> i32 {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let base = local.join("com.respected.hub");
    let roots = standard_roots();
    let dirs = brain::dirs();
    let data_root = if trust::dev_enabled() {
        default_data_root(trust::dev_env("RESPECTED_ORCHESTRATOR_DATA_ROOT").and_then(|v| v.into_string().ok()).as_deref(), base.with_file_name("RespectedOrchestratorDevelopment"))
    } else { brain::respected_root().join("RespectedOrchestrator").join("data") };
    let mut settings = Settings { python: "python".into(), orchestrator_root: base.join("no-source-checkout").join("respected-orchestrator"),
        brain_root: base.join("no-source-checkout").join("respected-secondbrain"), data_root, brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None };
    overlay(&roots, &dirs, &mut settings);
    let report = cleanup::remove_products(&roots, &dirs, &settings);
    registration::sync(&roots, &dirs);
    let log_dir = roots.managed.parent().map(Path::to_path_buf).unwrap_or(base); // beside the managed folder: the Hub's own data folder
    let _ = fs::create_dir_all(&log_dir);
    let _ = fs::write(log_dir.join("uninstall-products.log"), serde_json::to_vec_pretty(&report).unwrap_or_default());
    if report["ok"] == true { 0 } else { 1 }
}

/// `respected-hub.exe --sync-registrations` (no window): the Hub uninstaller runs this from the maintenance copy when the
/// person keeps the products, so their Installed-apps entries point at an executable that still exists.
pub fn headless_sync_registrations() -> i32 { registration::sync(&standard_roots(), &brain::dirs()); 0 }

pub fn run() {
    tauri::Builder::default()
        // a second window would run startup recovery against the first one's live install (audit P1 #9): focus the existing one instead
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            use tauri::{Emitter, Manager};
            if let Some(window) = app.webview_windows().values().next() { let _ = window.unminimize(); let _ = window.set_focus(); }
            // a product installer started meanwhile: tell the open window which product the user came to install
            if let Some(wanted) = intent(args.into_iter()) { let _ = app.emit("startup-intent", wanted); }
        }))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![startup_intent, get_settings, save_settings, engine_request, rpp_request, product_status, campaign_list,
            lifecycle_status, lifecycle_plan, lifecycle_apply, lifecycle_rollback, lifecycle_purge_backups, activity_list,
            catalog_refresh, catalog_set_channel, hub_update_status, hub_update_install,
            orchestrator_mcp_plan, orchestrator_mcp_apply,
            memory_access_status, memory_access_enable, memory_access_disable,
            memory_index_status, memory_index_refresh,
            memory_semantic_status, memory_semantic_enable, memory_semantic_disable, memory_capture_run,
            memory_search, memory_note_read, memory_capture_note, memory_briefing_run])
        // recovery can take minutes (engine call): never hold the window back for it
        .setup(|app| { if let Ok(roots) = lifecycle_roots(app.handle()) {
            // the catalog first and on its own: recovery below can take minutes, and the window must learn about products at once
            let (handle, catalog_roots) = (app.handle().clone(), roots.clone());
            std::thread::spawn(move || { use tauri::Emitter; let _ = catalog::refresh(&catalog_roots); let _ = handle.emit("catalog-updated", ()); });
            std::thread::spawn(move || {
                let dirs = brain::dirs();
                brain::recover_if_interrupted(&roots, &dirs);
                let _ = lifecycle::recover(&roots);
                sweep(&roots, &dirs);                 // a folder an MCP server still held at update time goes now
                registration::sync(&roots, &dirs);   // Installed-apps entries follow what is really installed
                registration::drop_maintenance(&roots);
            });
        } Ok(()) })
        .run(tauri::generate_context!()).expect("Respected Hub startup failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intents_accept_only_known_products_and_actions() {
        let args = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>().into_iter();
        assert_eq!(intent(args(&["hub.exe", "--install=secondbrain"])), Some(json!({ "action": "install", "product": "secondbrain" })));
        assert_eq!(intent(args(&["hub.exe", "--uninstall=orchestrator"])), Some(json!({ "action": "uninstall", "product": "orchestrator" })));
        assert_eq!(intent(args(&["hub.exe", "--manage=secondbrain"])), Some(json!({ "action": "manage", "product": "secondbrain" })));
        assert_eq!(intent(args(&["hub.exe", "--install=../evil"])), None);
        assert_eq!(intent(args(&["hub.exe", "--uninstall-products"])), None);
        assert_eq!(intent(args(&["hub.exe", "--uninstall=runtime"])), None, "only the two products can be named");
        assert_eq!(intent(args(&["hub.exe"])), None);
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
