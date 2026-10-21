// Respected Product Protocol (RPP) v1.0/v1.1 client for the Hub's native backend.
// One JSON request on stdin, one JSON response line on stdout. Never panics: every
// failure becomes a protocol envelope so callers always get a typed answer.
use crate::lifecycle::valid_operation_id;
use crate::{pump, PumpError, Settings};
use serde_json::{json, Value};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const RPP_VERSION: &str = "1.0";
const RPP_VERSION_V11: &str = "1.1";
const PRODUCTS: [&str; 2] = ["orchestrator", "secondbrain"];
const OPS: [&str; 26] = ["manifest", "health", "settings.schema", "settings.validate", "settings.apply", "settings.readback",
    "lifecycle.plan", "lifecycle.apply", "lifecycle.status", "lifecycle.rollback", "operation.receipt", "integrations.plan",
    "integrations.apply", "integrations.rollback",
    "memory.grant", "memory.revoke", "memory.index.status", "memory.index.refresh",
    "memory.semantic.status", "memory.semantic.configure", "memory.semantic.pull", "memory.capture.run",
    "memory.search", "memory.note.read", "memory.capture.note", "memory.briefing.run"];
// Ops introduced in v1.1; they are sent under rpp "1.1" and everything else stays on "1.0" so v1.0-only products keep working.
const V11_OPS: [&str; 20] = ["lifecycle.plan", "lifecycle.apply", "lifecycle.status", "lifecycle.rollback", "operation.receipt", "integrations.plan",
    "integrations.apply", "integrations.rollback",
    "memory.grant", "memory.revoke", "memory.index.status", "memory.index.refresh",
    "memory.semantic.status", "memory.semantic.configure", "memory.semantic.pull", "memory.capture.run",
    "memory.search", "memory.note.read", "memory.capture.note", "memory.briefing.run"];
const OPERATION_ID_REQUIRED_OPS: [&str; 5] = ["settings.apply", "lifecycle.apply", "lifecycle.rollback", "integrations.apply", "integrations.rollback"];
const SECRET_ENV_MARKERS: [&str; 8] = ["key", "token", "secret", "passw", "credential", "auth", "cookie", "session"];
const ENGINE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_OUTPUT: usize = 1024 * 1024;

fn ok_envelope(version: &str, operation_id: Option<String>, result: Value) -> Value {
    json!({ "rpp": version, "ok": true, "operation_id": operation_id, "result": result, "error": Value::Null })
}

fn err_envelope(version: &str, operation_id: Option<String>, code: &str, message: &str, hint: &str) -> Value {
    json!({ "rpp": version, "ok": false, "operation_id": operation_id, "result": Value::Null,
        "error": { "code": code, "message": message, "hint": hint } })
}

/// The generic IPC bridge may only reach the product lifecycle surface the Hub itself needs:
/// read-only plan/status/receipt, and SecondBrain's first-vault creation. Anything that could
/// write product files at a caller-chosen path (Orchestrator `lifecycle.apply`, any rollback) goes
/// through the Hub's own journaled `lifecycle_*` commands instead.
fn forbidden_lifecycle_call(product: &str, op: &str, payload: Option<&Value>) -> Option<(&'static str, &'static str)> {
    const HINT: &str = "Kurulum işlemleri Hub'ın kendi yaşam döngüsü komutlarıyla yapılır.";
    match op {
        _ if op.starts_with("memory.") => Some(("Bellek izinleri köprü üzerinden yönetilemez.", "Bellek erişimi yalnızca Hub'ın kendi bellek komutlarıyla açılır veya kapatılır.")),
        // memory grants and the Orchestrator's memory provider are set by the Hub's own native commands only
        "settings.apply" | "settings.validate" if payload.and_then(|p| p.get("settings")).and_then(Value::as_object).map_or(false, |s| s.keys().any(|k| k.starts_with("memory_"))) =>
            Some(("Bellek ayarları köprü üzerinden değiştirilemez.", "Bellek erişimi yalnızca Hub'ın kendi bellek komutlarıyla açılır veya kapatılır.")),
        "lifecycle.rollback" => Some(("Bu işlem köprü üzerinden yapılamaz.", HINT)),
        "integrations.apply" | "integrations.rollback" => Some(("İstemci yapılandırması köprü üzerinden değiştirilemez.", "AI istemci kayıtları yalnızca onay ekranı olan Hub komutlarıyla değiştirilir.")),
        "lifecycle.apply" => {
            let target = payload.and_then(|p| p.get("target")).and_then(Value::as_object);
            let allowed = product == "secondbrain"
                && payload.and_then(|p| p.get("action")).and_then(Value::as_str) == Some("install")
                && target.map(|t| t.get("vault_path").map_or(false, Value::is_string) && t.keys().all(|k| k == "vault_path" || k == "profile")).unwrap_or(false);
            if allowed { None } else { Some(("Bu işlem köprü üzerinden yapılamaz.", HINT)) }
        }
        _ => None,
    }
}

pub fn generate_operation_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    let candidate = format!("op{:x}{:x}{:x}", std::process::id(), nanos, sequence);
    if valid_operation_id(&candidate) { candidate } else { format!("op{nanos:016x}") }
}

fn is_secret_env(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    SECRET_ENV_MARKERS.iter().any(|marker| lower.contains(marker))
}

/// Isolated runs (tests) give product processes a fake user home through RESPECTED_HUB_TEST_HOME while the Hub
/// itself keeps its real profile (WebView2 needs it).
pub fn apply_test_home(command: &mut Command) {
    if let Some(home) = crate::trust::dev_abs("RESPECTED_HUB_TEST_HOME") {
        command.env("USERPROFILE", &home).env("HOME", &home);
    }
}

/// A development build runs product sources on a Python: it must not pick up the user's Python environment (Anaconda
/// PYTHONHOME, startup scripts, user site-packages).
pub fn isolate_python(command: &mut Command) {
    for name in ["PYTHONHOME", "PYTHONSTARTUP", "PYTHONUSERBASE", "PYTHONINSPECT", "PYTHONBREAKPOINT"] { command.env_remove(name); }
    command.env("PYTHONNOUSERSITE", "1");
}

pub fn strip_secret_env(command: &mut Command) {
    for (name, _) in std::env::vars_os() {
        if is_secret_env(&name.to_string_lossy()) { command.env_remove(name); }
    }
}

fn build_request(op: &str, payload: Option<Value>, operation_id: Option<String>, rpp_version: &str) -> Value {
    json!({ "rpp": rpp_version, "op": op, "operation_id": operation_id,
        "payload": payload.unwrap_or_else(|| json!({})) })
}

/// Engines never run with their program folder as working directory: on Windows that would lock the
/// folder and make update/repair/uninstall of the managed copy fail while a health poll is running.
pub fn neutral_cwd(settings: &Settings) -> std::path::PathBuf {
    let _ = std::fs::create_dir_all(&settings.data_root);
    if settings.data_root.is_dir() { settings.data_root.clone() } else { std::env::temp_dir() }
}

/// Writer-priority gate (std's RwLock lets steady polling starve a writer): many `read()` guards for status polls and
/// wizard calls, one `write()` guard while a product installer changes its program folder.
pub struct Gate { state: std::sync::Mutex<(usize, bool)>, changed: std::sync::Condvar }
pub struct GateGuard<'a> { gate: &'a Gate, writer: bool }

impl Gate {
    pub const fn new() -> Self { Gate { state: std::sync::Mutex::new((0, false)), changed: std::sync::Condvar::new() } }
    pub fn read(&self) -> GateGuard<'_> {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        while state.1 { state = self.changed.wait(state).unwrap_or_else(|p| p.into_inner()); }
        state.0 += 1;
        GateGuard { gate: self, writer: false }
    }
    /// Like `write`, but gives up (and lets new readers in again) when running readers do not finish within `limit`:
    /// a long operation such as the semantic model download must not freeze every status poll behind a queued installer.
    pub fn write_within(&self, limit: std::time::Duration) -> Option<GateGuard<'_>> {
        let deadline = std::time::Instant::now() + limit;
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        while state.1 {
            let left = deadline.checked_duration_since(std::time::Instant::now())?;
            state = self.changed.wait_timeout(state, left).unwrap_or_else(|p| p.into_inner()).0;
        }
        state.1 = true;
        while state.0 > 0 {
            match deadline.checked_duration_since(std::time::Instant::now()) {
                Some(left) => state = self.changed.wait_timeout(state, left).unwrap_or_else(|p| p.into_inner()).0,
                None => { state.1 = false; self.changed.notify_all(); return None; }
            }
        }
        Some(GateGuard { gate: self, writer: true })
    }
}

impl Drop for GateGuard<'_> {
    fn drop(&mut self) {
        let mut state = self.gate.state.lock().unwrap_or_else(|p| p.into_inner());
        if self.writer { state.1 = false; } else { state.0 -= 1; }
        self.gate.changed.notify_all();
    }
}

pub static ENGINE_GATE: Gate = Gate::new();

/// SecondBrain served by its own frozen launcher: no Python and no source tree involved.
fn frozen_brain_command(settings: &Settings) -> Option<Command> {
    let exe = settings.brain_exe.as_ref().filter(|path| path.is_file())?;
    let mut command = Command::new(exe);
    apply_test_home(&mut command);
    command.current_dir(neutral_cwd(settings)).env("PYTHONDONTWRITEBYTECODE", "1").arg("rpp")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(app) = &settings.brain_app_dir { command.env("RESPECTED_APP_DIR", app); }
    if let Some(data) = &settings.brain_data_root { command.env("RESPECTED_DATA_DIR", data).arg("--data-root").arg(data); }
    Some(command)
}

/// argv and env a process needs to speak RPP to SecondBrain, from the same facts as `frozen_brain_command`/`engine_command`.
/// Never carries secret-looking environment.
/// The Orchestrator starts SecondBrain from a command line it validates (argv[0] must be an existing file), so a bare
/// program name such as "python" is resolved through PATH here; an unresolvable name is returned unchanged.
pub fn resolve_program(name: &str) -> String {
    let path = std::path::Path::new(name);
    if path.is_absolute() || name.contains('/') || name.contains('\\') { return name.to_string(); }
    let extensions: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT".into()).split(';').map(|e| e.to_string()).chain(std::iter::once(String::new())).collect()
    } else { vec![String::new()] };
    for dir in std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect::<Vec<_>>()).unwrap_or_default() {
        for extension in &extensions {
            let candidate = dir.join(format!("{name}{extension}"));
            if candidate.is_file() { return candidate.to_string_lossy().to_string(); }
        }
    }
    name.to_string()
}

pub fn brain_rpp_command(settings: &Settings) -> Option<(Vec<String>, serde_json::Map<String, Value>)> {
    let text = |p: &std::path::Path| p.to_string_lossy().to_string();
    let mut env = serde_json::Map::new();
    env.insert("PYTHONDONTWRITEBYTECODE".into(), json!("1"));
    let argv = if let Some(exe) = settings.brain_exe.as_ref().filter(|path| path.is_file()) {
        let mut argv = vec![text(exe), "rpp".to_string()];
        if let Some(app) = &settings.brain_app_dir { env.insert("RESPECTED_APP_DIR".into(), json!(text(app))); }
        if let Some(data) = &settings.brain_data_root {
            env.insert("RESPECTED_DATA_DIR".into(), json!(text(data)));
            argv.extend(["--data-root".to_string(), text(data)]);
        }
        argv
    } else {
        if !crate::trust::dev_enabled() { return None; }
        let source = settings.brain_root.join("src");
        let package = source.join("respectedbrain");
        if !(package.join("__main__.py").is_file() || package.join("__init__.py").is_file() || source.join("respectedbrain.py").is_file()) { return None; }
        env.insert("PYTHONPATH".into(), json!(text(&source)));
        // The Orchestrator starts SecondBrain with a minimal environment: pass on the dev Hub's own data roots so grant and recall agree.
        for name in ["RESPECTED_APP_DIR", "RESPECTED_DATA_DIR"] {
            if let Some(dir) = crate::trust::dev_abs(name) { env.insert(name.into(), json!(text(&dir))); }
        }
        let mut argv = vec![resolve_program(&settings.python), "-m".to_string(), "respectedbrain".to_string(), "rpp".to_string()];
        if let Some(data) = &settings.brain_data_root { argv.extend(["--data-root".to_string(), text(data)]); }
        argv
    };
    if let Some(home) = crate::trust::dev_abs("RESPECTED_HUB_TEST_HOME") {
        env.insert("USERPROFILE".into(), json!(text(&home)));
        env.insert("HOME".into(), json!(text(&home)));
    }
    Some((argv, env))
}

/// The Orchestrator engine with `args` after its data root: the packaged executable the Hub installed, or (development
/// builds only reach this) the source checkout on Python. `None` when neither exists.
pub fn orchestrator_command(settings: &Settings, args: &[&str]) -> Option<Command> {
    let mut command = match settings.orchestrator_exe.as_ref().filter(|exe| exe.is_file()) {
        Some(exe) => Command::new(exe),
        None => {
            if !crate::trust::dev_enabled() || !settings.orchestrator_root.join("src/respected_orchestrator/__main__.py").is_file() { return None; }
            let mut command = Command::new(&settings.python);
            isolate_python(&mut command);
            command.env("PYTHONPATH", settings.orchestrator_root.join("src")).args(["-m", "respected_orchestrator"]);
            command
        }
    };
    apply_test_home(&mut command);
    command.arg("--data-root").arg(&settings.data_root).args(args).current_dir(neutral_cwd(settings))
        .env("PYTHONDONTWRITEBYTECODE", "1").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    Some(command)
}

/// How the person starts the Orchestrator from a PowerShell prompt (manual approval and apply are never run by the Hub):
/// the installed program with the data folder this Hub uses. `None` when no packaged Orchestrator is installed.
pub fn orchestrator_cli_line(settings: &Settings) -> Option<String> {
    let exe = settings.orchestrator_exe.as_ref().filter(|exe| exe.is_file())?;
    Some(format!("& \"{}\" --data-root \"{}\"", exe.display(), settings.data_root.display()))
}

fn engine_command(settings: &Settings, product: &str) -> Option<Command> {
    if product == "orchestrator" { return orchestrator_command(settings, &["rpp"]); }
    if let Some(command) = frozen_brain_command(settings) { return Some(command); }
    if !crate::trust::dev_enabled() { return None; }
    let source = settings.brain_root.join("src");
    let package = source.join("respectedbrain");
    if !(package.join("__main__.py").is_file() || package.join("__init__.py").is_file() || source.join("respectedbrain.py").is_file()) { return None; }
    let mut command = Command::new(&settings.python);
    apply_test_home(&mut command);
    isolate_python(&mut command);
    command.current_dir(neutral_cwd(settings)).env("PYTHONPATH", source).env("PYTHONDONTWRITEBYTECODE", "1")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).args(["-m", "respectedbrain", "rpp"]);
    if let Some(data_root) = &settings.brain_data_root { command.arg("--data-root").arg(data_root); }
    Some(command)
}

fn normalize(version: &str, response: Value) -> Result<Value, String> {
    let object = response.as_object().ok_or("not a JSON object")?;
    let ok = object.get("ok").and_then(Value::as_bool).ok_or("field 'ok' is not a boolean")?;
    let operation_id = match object.get("operation_id") {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) => Some(value.clone()),
        Some(_) => return Err("field 'operation_id' is not a string or null".into()),
    };
    if ok {
        if object.get("error") != Some(&Value::Null) { return Err("successful response must carry a null error".into()); }
        let result = object.get("result").ok_or("field 'result' is missing")?.clone();
        Ok(ok_envelope(version, operation_id, result))
    } else {
        if object.get("result") != Some(&Value::Null) { return Err("failed response must carry a null result".into()); }
        let error = object.get("error").and_then(Value::as_object).ok_or("field 'error' is not an object")?;
        let code = error.get("code").and_then(Value::as_str).filter(|c| !c.is_empty()).ok_or("error code is missing")?;
        let message = error.get("message").and_then(Value::as_str).filter(|m| !m.is_empty()).ok_or("error message is missing")?;
        let mut rebuilt = serde_json::Map::new();
        rebuilt.insert("code".into(), Value::String(code.to_string()));
        rebuilt.insert("message".into(), Value::String(message.to_string()));
        if let Some(hint) = error.get("hint").and_then(Value::as_str) { rebuilt.insert("hint".into(), Value::String(hint.to_string())); }
        Ok(json!({ "rpp": version, "ok": false, "operation_id": operation_id, "result": Value::Null, "error": Value::Object(rebuilt) }))
    }
}

pub fn rpp_call(settings: &Settings, product: &str, op: &str, payload: Option<Value>, operation_id: Option<String>) -> Value {
    
    rpp_call_version(settings, product, op, payload, operation_id, if V11_OPS.contains(&op) { RPP_VERSION_V11 } else { RPP_VERSION })
}

pub fn rpp_call_version(settings: &Settings, product: &str, op: &str, payload: Option<Value>, operation_id: Option<String>, rpp_version: &str) -> Value {
    dispatch(settings, product, op, payload, operation_id, rpp_version, false, ENGINE_TIMEOUT)
}

/// For Hub code only (never reachable from the webview): skips the bridge filter so the Hub can record an
/// install in a product's own manifest as part of its journaled lifecycle.
pub fn rpp_call_trusted(settings: &Settings, product: &str, op: &str, payload: Option<Value>, operation_id: Option<String>) -> Value {
    rpp_call_trusted_timeout(settings, product, op, payload, operation_id, ENGINE_TIMEOUT)
}

/// Like `rpp_call_trusted` with a caller-chosen timeout, for long operations such as a search index refresh.
pub fn rpp_call_trusted_timeout(settings: &Settings, product: &str, op: &str, payload: Option<Value>, operation_id: Option<String>, timeout: Duration) -> Value {
    dispatch(settings, product, op, payload, operation_id, if V11_OPS.contains(&op) { RPP_VERSION_V11 } else { RPP_VERSION }, true, timeout)
}

fn dispatch(settings: &Settings, product: &str, op: &str, payload: Option<Value>, operation_id: Option<String>, rpp_version: &str, trusted: bool, timeout: Duration) -> Value {
    if !PRODUCTS.contains(&product) {
        return err_envelope(rpp_version, operation_id, "INVALID_PRODUCT", &format!("Unknown product '{product}'."),
            "Allowed products: orchestrator, secondbrain.");
    }
    if !OPS.contains(&op) {
        return err_envelope(rpp_version, operation_id, "INVALID_OP", &format!("Unknown operation '{op}'."),
            "Allowed operations: manifest, health, settings.*, lifecycle.plan/apply/status/rollback, operation.receipt.");
    }
    if let Some((message, hint)) = forbidden_lifecycle_call(product, op, payload.as_ref()).filter(|_| !trusted) {
        return err_envelope(rpp_version, operation_id, "PERMISSION_DENIED", message, hint);
    }
    let operation_id = if OPERATION_ID_REQUIRED_OPS.contains(&op) {
        Some(match operation_id { Some(id) if valid_operation_id(&id) => id, _ => generate_operation_id() })
    } else { operation_id };
    let mut command = match engine_command(settings, product) {
        Some(command) => command,
        None => return err_envelope(rpp_version, operation_id, "ENGINE_NOT_FOUND", &format!("The {product} engine is not installed."),
            "Install the product from the Hub."),
    };
    strip_secret_env(&mut command);
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let request = build_request(op, payload, operation_id.clone(), rpp_version);
    let payload_bytes = serde_json::to_vec(&request).unwrap_or_else(|_| b"{}".to_vec());
    // Many readers (status polls, wizard calls) or one writer (a product install/update/repair/uninstall):
    // the writer waits for running polls and blocks new ones, so the Hub never holds the files it changes.
    let _gate = ENGINE_GATE.read();
    let output = match pump(command, Some(payload_bytes), timeout, MAX_OUTPUT) {
        Ok(output) => output,
        Err(PumpError::Spawn(error)) => return err_envelope(rpp_version, operation_id, "ENGINE_UNAVAILABLE",
            &format!("The {product} engine could not be started ({error})."),
            "Repair the product from the Hub."),
        Err(PumpError::Timeout) => return err_envelope(rpp_version, operation_id, "ENGINE_TIMEOUT",
            &format!("The {product} engine did not answer within {} seconds.", timeout.as_secs()), "Retry the request."),
        Err(error) => return err_envelope(rpp_version, operation_id, "BAD_RESPONSE",
            &format!("The {product} engine response could not be read: {}", error.message()),
            "Check that the engine implements RPP 1.0/1.1."),
    };
    let response: Value = match serde_json::from_slice(&output.stdout) {
        Ok(value) => value,
        Err(_) => return err_envelope(rpp_version, operation_id, "BAD_RESPONSE",
            &format!("The {product} engine did not return valid JSON."), "Check that the engine implements RPP 1.0/1.1."),
    };
    match normalize(rpp_version, response) {
        Ok(envelope) => envelope,
        Err(message) => err_envelope(rpp_version, operation_id, "BAD_RESPONSE",
            &format!("The {product} engine returned an unexpected response: {message}"), "Check that the engine implements RPP 1.0/1.1."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn settings(python: &str, root: &str, brain: &str) -> Settings {
        Settings { python: python.into(), orchestrator_root: PathBuf::from(root), brain_root: PathBuf::from(brain),
            data_root: PathBuf::from("/tmp/hub-rpp-unit"), brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None }
    }

    #[test]
    fn unknown_product_and_op_are_rejected_without_spawning() {
        let settings = settings("definitely-not-a-real-python", "/missing-hub-orchestrator", "/missing-hub-brain");
        let product = rpp_call(&settings, "hub", "manifest", None, None);
        assert_eq!(product["ok"], false);
        assert_eq!(product["error"]["code"], "INVALID_PRODUCT");
        assert_eq!(product["result"], Value::Null);
        let op = rpp_call(&settings, "orchestrator", "tasks.run", None, None);
        assert_eq!(op["ok"], false);
        assert_eq!(op["error"]["code"], "INVALID_OP");
    }

    #[test]
    fn unknown_op_is_checked_before_the_engine_is_resolved() {
        let settings = settings("python", "/missing-hub-orchestrator", "/missing-hub-brain");
        let response = rpp_call(&settings, "orchestrator", "not-an-op", None, Some("fixed_id_1".into()));
        assert_eq!(response["error"]["code"], "INVALID_OP");
        assert_eq!(response["operation_id"], "fixed_id_1");
    }

    #[test]
    fn missing_engine_source_reports_engine_not_found() {
        let settings = settings("python", "/missing-hub-orchestrator", "/missing-hub-brain");
        for product in ["orchestrator", "secondbrain"] {
            let response = rpp_call(&settings, product, "manifest", None, None);
            assert_eq!(response["ok"], false, "{product}");
            assert_eq!(response["error"]["code"], "ENGINE_NOT_FOUND", "{product}");
            assert_eq!(response["result"], Value::Null, "{product}");
        }
    }

    #[test]
    fn operation_ids_are_validated_and_generated() {
        assert!(valid_operation_id("abcd1234"));
        assert!(valid_operation_id("op-2024_10-A"));
        assert!(!valid_operation_id("short"));
        assert!(!valid_operation_id("has space"));
        assert!(!valid_operation_id(&"x".repeat(65)));
        let generated = generate_operation_id();
        assert!(valid_operation_id(&generated), "{generated}");
    }

    #[test]
    fn request_uses_the_contract_shape() {
        let request = build_request("settings.apply", Some(json!({"settings": {}})), Some("apply_12345".into()), RPP_VERSION);
        assert_eq!(request["rpp"], RPP_VERSION);
        assert_eq!(request["op"], "settings.apply");
        assert_eq!(request["operation_id"], "apply_12345");
        assert_eq!(request["payload"], json!({"settings": {}}));
        let without_payload = build_request("health", None, None, RPP_VERSION);
        assert_eq!(without_payload["operation_id"], Value::Null);
        assert_eq!(without_payload["payload"], json!({}));
    }

    #[test]
    fn the_gate_gives_a_waiting_writer_priority_over_a_steady_stream_of_readers() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        static GATE: Gate = Gate::new();
        let first = GATE.read();
        let wrote = Arc::new(AtomicBool::new(false));
        let flag = wrote.clone();
        let writer = std::thread::spawn(move || { let _w = GATE.write_within(Duration::from_secs(10)).expect("the reader finishes in time"); flag.store(true, Ordering::SeqCst); });
        std::thread::sleep(Duration::from_millis(100));
        assert!(!wrote.load(Ordering::SeqCst), "the writer waits for the running reader");
        let late = std::thread::spawn(|| { let _r = GATE.read(); });
        std::thread::sleep(Duration::from_millis(100));
        assert!(!late.is_finished(), "a reader arriving after the writer queued must wait");
        drop(first);
        writer.join().unwrap();
        late.join().unwrap();
        assert!(wrote.load(Ordering::SeqCst));
    }

    #[test]
    fn the_bridge_only_allows_read_only_lifecycle_and_first_vault_creation() {
        let vault = json!({ "action": "install", "target": { "vault_path": "C:/v", "profile": {} } });
        assert!(forbidden_lifecycle_call("secondbrain", "lifecycle.apply", Some(&vault)).is_none());
        assert!(forbidden_lifecycle_call("orchestrator", "lifecycle.apply", Some(&vault)).is_some());
        let files = json!({ "action": "install", "target": { "app_root": "C:/x", "files": [] } });
        assert!(forbidden_lifecycle_call("orchestrator", "lifecycle.apply", Some(&files)).is_some());
        assert!(forbidden_lifecycle_call("secondbrain", "lifecycle.apply", Some(&json!({ "action": "uninstall", "target": { "vault_path": "C:/v" } }))).is_some());
        assert!(forbidden_lifecycle_call("secondbrain", "lifecycle.apply", Some(&json!({ "action": "install", "target": { "vault_path": "C:/v", "extra": 1 } }))).is_some());
        assert!(forbidden_lifecycle_call("secondbrain", "lifecycle.apply", None).is_some());
        assert!(forbidden_lifecycle_call("secondbrain", "lifecycle.rollback", None).is_some());
        for op in ["lifecycle.plan", "lifecycle.status", "operation.receipt", "manifest", "settings.apply"] {
            assert!(forbidden_lifecycle_call("orchestrator", op, None).is_none(), "{op}");
        }
    }

    #[test]
    fn memory_ops_are_denied_on_the_bridge_but_known_to_trusted_code() {
        let settings = settings("python", "/missing-hub-orchestrator", "/missing-hub-brain");
        for op in ["memory.grant", "memory.revoke", "memory.anything"] {
            assert!(forbidden_lifecycle_call("secondbrain", op, None).is_some(), "{op}");
        }
        let denied = rpp_call(&settings, "secondbrain", "memory.grant", Some(json!({ "label": "x" })), None);
        assert_eq!(denied["error"]["code"], "PERMISSION_DENIED");
        let trusted = rpp_call_trusted(&settings, "secondbrain", "memory.grant", Some(json!({ "label": "x" })), Some("trusted_id_1".into()));
        assert_eq!(trusted["error"]["code"], "ENGINE_NOT_FOUND");
        assert_eq!(trusted["rpp"], RPP_VERSION_V11);
    }

    #[test]
    fn a_queued_installer_gives_up_instead_of_freezing_every_status_poll_behind_a_long_operation() {
        use std::time::{Duration, Instant};
        let gate = Gate::new();
        let long_reader = gate.read();                       // e.g. the semantic model download
        let started = Instant::now();
        assert!(gate.write_within(Duration::from_millis(150)).is_none());
        assert!(started.elapsed() < Duration::from_secs(5));
        let again = gate.read();                             // the failed writer left no stale "writer waiting" flag behind
        drop(again);
        drop(long_reader);
        let writer = gate.write_within(Duration::from_millis(150));
        assert!(writer.is_some(), "with no readers the writer gets in");
        drop(writer);
        assert!(gate.write_within(Duration::from_millis(50)).is_some());
    }

    #[test]
    fn the_webview_cannot_set_the_memory_provider_through_orchestrator_settings() {
        for op in ["settings.apply", "settings.validate"] {
            for key in ["memory_command", "memory_token", "memory_enabled", "memory_grant_id", "memory_write"] {
                let payload = json!({ "settings": { key: "x" } });
                assert!(forbidden_lifecycle_call("orchestrator", op, Some(&payload)).is_some(), "{op} {key}");
            }
            let ordinary = json!({ "settings": { "auto_assign": true, "mcp_acceptance_allow": "[]" } });
            assert!(forbidden_lifecycle_call("orchestrator", op, Some(&ordinary)).is_none(), "{op}");
        }
    }

    // Debug build (the default for `cargo test`): the isolated-test home is honoured. The production profile is proven in trust.rs.
    #[cfg(any(debug_assertions, feature = "dev-trust"))]
    #[test]
    fn a_development_build_honours_the_isolated_test_home() {
        let home = std::env::temp_dir().join("hub-dev-home");
        std::env::set_var("RESPECTED_HUB_TEST_HOME", &home);
        let mut command = Command::new("x");
        apply_test_home(&mut command);
        std::env::remove_var("RESPECTED_HUB_TEST_HOME");
        assert!(command.get_envs().any(|(k, v)| k == "USERPROFILE" && v == Some(home.as_os_str())));
    }

    #[test]
    fn brain_rpp_command_mirrors_the_engine_command_facts() {
        let dir = std::env::temp_dir().join(format!("hub-brain-cmd-{}", std::process::id()));
        let package = dir.join("src").join("respectedbrain");
        std::fs::create_dir_all(&package).unwrap();
        std::fs::write(package.join("__init__.py"), "").unwrap();
        let exe = std::env::current_exe().unwrap();

        let mut source = settings("definitely-not-a-python", "/missing-hub-orchestrator", &dir.to_string_lossy());
        source.brain_data_root = Some(PathBuf::from("/data/brain"));
        let (argv, env) = brain_rpp_command(&source).unwrap();
        assert_eq!(argv, ["definitely-not-a-python", "-m", "respectedbrain", "rpp", "--data-root", "/data/brain"]);
        assert!(std::path::Path::new(&resolve_program("cargo")).is_file() || !cfg!(windows) || resolve_program("cargo") == "cargo");
        assert_eq!(env["PYTHONDONTWRITEBYTECODE"], "1");
        assert_eq!(env["PYTHONPATH"], json!(dir.join("src").to_string_lossy().to_string()));

        let mut frozen = source.clone();
        frozen.brain_exe = Some(exe.clone());
        frozen.brain_app_dir = Some(PathBuf::from("/app/brain"));
        let (argv, env) = brain_rpp_command(&frozen).unwrap();
        assert_eq!(argv, vec![exe.to_string_lossy().to_string(), "rpp".to_string(), "--data-root".to_string(), "/data/brain".to_string()]);
        assert_eq!(env["RESPECTED_APP_DIR"], "/app/brain");
        assert_eq!(env["RESPECTED_DATA_DIR"], "/data/brain");
        assert!(!env.contains_key("PYTHONPATH"));
        assert!(env.keys().all(|name| !is_secret_env(name)), "{env:?}");

        assert!(brain_rpp_command(&settings("py", "/x", "/missing-hub-brain")).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn index_ops_are_denied_on_the_bridge_but_known_to_trusted_code() {
        let settings = settings("python", "/missing-hub-orchestrator", "/missing-hub-brain");
        for op in ["memory.index.status", "memory.index.refresh", "memory.semantic.status", "memory.semantic.pull"] {
            assert!(forbidden_lifecycle_call("secondbrain", op, None).is_some(), "{op}");
            let denied = rpp_call(&settings, "secondbrain", op, Some(json!({})), None);
            assert_eq!(denied["error"]["code"], "PERMISSION_DENIED", "{op}");
            let trusted = rpp_call_trusted(&settings, "secondbrain", op, Some(json!({})), Some("trusted_id_1".into()));
            assert_eq!(trusted["error"]["code"], "ENGINE_NOT_FOUND", "{op}");
            assert_eq!(trusted["rpp"], RPP_VERSION_V11, "{op}");
            let timed = rpp_call_trusted_timeout(&settings, "secondbrain", op, Some(json!({})), Some("trusted_id_2".into()), Duration::from_secs(900));
            assert_eq!(timed["error"]["code"], "ENGINE_NOT_FOUND", "{op}");
        }
    }

    #[test]
    fn source_mode_passes_on_the_hubs_own_absolute_data_roots_only() {
        let dir = std::env::temp_dir().join(format!("hub-brain-env-{}", std::process::id()));
        let package = dir.join("src").join("respectedbrain");
        std::fs::create_dir_all(&package).unwrap();
        std::fs::write(package.join("__init__.py"), "").unwrap();
        let source = settings("py", "/x", &dir.to_string_lossy());
        let absolute = std::env::temp_dir().join("hub-env-data");

        std::env::remove_var("RESPECTED_APP_DIR");
        std::env::remove_var("RESPECTED_DATA_DIR");
        let (_, env) = brain_rpp_command(&source).unwrap();
        assert!(!env.contains_key("RESPECTED_APP_DIR") && !env.contains_key("RESPECTED_DATA_DIR"), "{env:?}");

        std::env::set_var("RESPECTED_APP_DIR", &absolute);
        std::env::set_var("RESPECTED_DATA_DIR", "relative/data");
        let (_, env) = brain_rpp_command(&source).unwrap();
        assert_eq!(env["RESPECTED_APP_DIR"], json!(absolute.to_string_lossy().to_string()));
        assert!(!env.contains_key("RESPECTED_DATA_DIR"), "relative paths are ignored");
        assert!(env.keys().all(|name| !is_secret_env(name)), "{env:?}");

        std::env::remove_var("RESPECTED_APP_DIR");
        std::env::remove_var("RESPECTED_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn secret_looking_environment_names_are_filtered() {
        for name in ["OPENAI_API_KEY", "GITHUB_TOKEN", "DB_PASSWORD", "MY_SECRET", "AWS_CREDENTIAL", "AUTH_HEADER", "SESSION_ID", "HTTP_COOKIE"] {
            assert!(is_secret_env(name), "{name}");
        }
        for name in ["PATH", "SYSTEMROOT", "PYTHONPATH", "TEMP", "LANG"] {
            assert!(!is_secret_env(name), "{name}");
        }
    }
}