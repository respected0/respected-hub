// Respected Product Protocol (RPP) 1 client for the Hub's native backend.
// One JSON request on stdin, one JSON response line on stdout. Never panics: every
// failure becomes a protocol envelope so callers always get a typed answer.
use crate::lifecycle::valid_operation_id;
use crate::{pump, PumpError, Settings};
use serde_json::{json, Value};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const RPP_VERSION: &str = "1";
const PRODUCTS: [&str; 2] = ["orchestrator", "secondbrain"];
const OPS: [&str; 24] = ["manifest", "health", "settings.schema", "settings.validate", "settings.apply", "settings.readback",
    "integrations.plan", "integrations.apply", "integrations.rollback",
    "memory.connect", "memory.disconnect", "memory.status", "memory.index.status", "memory.index.refresh",
    "memory.semantic.status", "memory.semantic.configure", "memory.semantic.pull", "memory.capture.run",
    "memory.search", "memory.note.read", "memory.capture.note", "memory.briefing.run", "memory.recall", "memory.write"];
const OPERATION_ID_REQUIRED_OPS: [&str; 3] = ["settings.apply", "integrations.apply", "integrations.rollback"];
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

/// What the webview's generic bridge may not do: memory grants/links and the Orchestrator's memory provider, and writing AI
/// client configurations, all go through the Hub's own commands (which show the plan or ask first).
fn forbidden_call(op: &str, payload: Option<&Value>) -> Option<(&'static str, &'static str)> {
    const MEMORY_HINT: &str = "Bellek erişimi yalnızca Hub'ın kendi bellek komutlarıyla açılır veya kapatılır.";
    match op {
        _ if op.starts_with("memory.") => Some(("Bellek işlemleri köprü üzerinden yapılamaz.", MEMORY_HINT)),
        "settings.apply" | "settings.validate" if payload.and_then(|p| p.get("settings")).and_then(Value::as_object).map_or(false, |s| s.keys().any(|k| k.starts_with("memory_"))) =>
            Some(("Bellek ayarları köprü üzerinden değiştirilemez.", MEMORY_HINT)),
        "integrations.apply" | "integrations.rollback" => Some(("İstemci yapılandırması köprü üzerinden değiştirilemez.", "AI istemci kayıtları yalnızca onay ekranı olan Hub komutlarıyla değiştirilir.")),
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
        // Windows derives the per-user folders from USERPROFILE (the installers' {localappdata} follows it), so all of them move together
        command.env("USERPROFILE", &home).env("HOME", &home)
            .env("LOCALAPPDATA", home.join("AppData").join("Local")).env("APPDATA", home.join("AppData").join("Roaming"));
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
/// folder and make the product's installer fail while a health poll is running.
pub fn neutral_cwd() -> std::path::PathBuf { std::env::temp_dir() }

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
    command.current_dir(neutral_cwd()).env("PYTHONDONTWRITEBYTECODE", "1").arg("rpp")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    Some(command)
}

/// A bare program name such as "python" resolved through PATH (development source runs); an unresolvable name is returned unchanged.
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

/// The Orchestrator with `args`: its installed program, or (development builds only reach this) the source checkout on Python.
/// `None` when neither exists. The product chooses its own data folder.
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
    command.args(args).current_dir(neutral_cwd())
        .env("PYTHONDONTWRITEBYTECODE", "1").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    Some(command)
}

/// How the person starts the Orchestrator from a PowerShell prompt (manual approval and apply are never run by the Hub).
/// `None` when no Orchestrator is installed.
pub fn orchestrator_cli_line(settings: &Settings) -> Option<String> {
    let exe = settings.orchestrator_exe.as_ref().filter(|exe| exe.is_file())?;
    Some(format!("& \"{}\"", exe.display()))
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
    command.current_dir(neutral_cwd()).env("PYTHONPATH", source).env("PYTHONDONTWRITEBYTECODE", "1")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).args(["-m", "respectedbrain", "rpp"]);
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
    dispatch(settings, product, op, payload, operation_id, false, ENGINE_TIMEOUT)
}

/// For Hub code only (never reachable from the webview): skips the bridge filter for the Hub's own memory and
/// integration commands.
pub fn rpp_call_trusted(settings: &Settings, product: &str, op: &str, payload: Option<Value>, operation_id: Option<String>) -> Value {
    rpp_call_trusted_timeout(settings, product, op, payload, operation_id, ENGINE_TIMEOUT)
}

/// Like `rpp_call_trusted` with a caller-chosen timeout, for long operations such as a search index refresh.
pub fn rpp_call_trusted_timeout(settings: &Settings, product: &str, op: &str, payload: Option<Value>, operation_id: Option<String>, timeout: Duration) -> Value {
    dispatch(settings, product, op, payload, operation_id, true, timeout)
}

fn dispatch(settings: &Settings, product: &str, op: &str, payload: Option<Value>, operation_id: Option<String>, trusted: bool, timeout: Duration) -> Value {
    let rpp_version = RPP_VERSION;
    if !PRODUCTS.contains(&product) {
        return err_envelope(rpp_version, operation_id, "INVALID_PRODUCT", &format!("Unknown product '{product}'."),
            "Allowed products: orchestrator, secondbrain.");
    }
    if !OPS.contains(&op) {
        return err_envelope(rpp_version, operation_id, "INVALID_OP", &format!("Unknown operation '{op}'."),
            "Allowed operations: manifest, health, settings.*, integrations.*, memory.*.");
    }
    if let Some((message, hint)) = forbidden_call(op, payload.as_ref()).filter(|_| !trusted) {
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
        Settings { python: python.into(), orchestrator_root: PathBuf::from(root), brain_root: PathBuf::from(brain), brain_exe: None, orchestrator_exe: None }
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
    fn memory_and_client_configuration_ops_are_denied_on_the_bridge_but_known_to_trusted_code() {
        let settings = settings("python", "/missing-hub-orchestrator", "/missing-hub-brain");
        for op in ["memory.connect", "memory.disconnect", "memory.status", "memory.index.refresh", "memory.semantic.pull", "memory.search", "integrations.apply", "integrations.rollback"] {
            assert!(forbidden_call(op, None).is_some(), "{op}");
            assert_eq!(rpp_call(&settings, "orchestrator", op, Some(json!({})), None)["error"]["code"], "PERMISSION_DENIED", "{op}");
            let trusted = rpp_call_trusted(&settings, "orchestrator", op, Some(json!({})), Some("trusted_id_1".into()));
            assert_eq!((trusted["error"]["code"].as_str(), trusted["rpp"].as_str()), (Some("ENGINE_NOT_FOUND"), Some("1")), "{op}");
        }
        for op in ["manifest", "health", "settings.schema", "settings.readback", "integrations.plan"] {
            assert!(forbidden_call(op, None).is_none(), "{op}");
        }
        for op in ["lifecycle.apply", "lifecycle.plan", "operation.receipt", "memory.grant"] {
            assert_eq!(rpp_call_trusted(&settings, "secondbrain", op, None, None)["error"]["code"], "INVALID_OP", "{op}: no protocol op installs or grants");
        }
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
                assert!(forbidden_call(op, Some(&payload)).is_some(), "{op} {key}");
            }
            let ordinary = json!({ "settings": { "auto_assign": true, "mcp_acceptance_allow": "[]" } });
            assert!(forbidden_call(op, Some(&ordinary)).is_none(), "{op}");
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
        let local = home.join("AppData").join("Local");
        assert!(command.get_envs().any(|(k, v)| k == "LOCALAPPDATA" && v == Some(local.as_os_str())), "per-user folders move with the home");
    }

    #[test]
    fn programs_are_found_on_path_and_products_get_no_data_root_from_the_hub() {
        assert!(std::path::Path::new(&resolve_program("cargo")).is_file() || !cfg!(windows) || resolve_program("cargo") == "cargo");
        let mut frozen = settings("py", "/x", "/y");
        frozen.orchestrator_exe = Some(std::env::current_exe().unwrap());
        let command = orchestrator_command(&frozen, &["rpp"]).unwrap();
        assert_eq!(command.get_args().collect::<Vec<_>>(), ["rpp"], "the product chooses its own data folder");
        assert!(command.get_envs().all(|(k, _)| k != "RESPECTED_DATA_DIR" && k != "RESPECTED_APP_DIR"));
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