// Orchestrator MCP access in AI clients (Claude Code, Codex, ...): a stable launcher plus a journaled, approval-bound registration.
// The Orchestrator itself edits the client configs (its `integrations.*` ops: ownership, foreign entries preserved, entry-level
// rollback). The Hub only supplies the launcher, shows the plan, binds the apply to the approved plan digest and journals it.
// Nothing here keeps the Hub running: the client starts the launcher (the packaged Orchestrator) on its own.
use crate::lifecycle::{self, LifecycleError, Roots};
use crate::{pump, rpp, Settings};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, process::{Command, Stdio}, time::Duration};

const PRODUCT: &str = "orchestrator";
const ACTION: &str = "integrations";

fn err(code: &'static str, message: impl Into<String>, hint: impl Into<String>) -> LifecycleError { LifecycleError { code, message: message.into(), hint: hint.into() } }

pub fn launcher_path(roots: &Roots) -> PathBuf {
    roots.managed.join(PRODUCT).join("mcp").join(if cfg!(windows) { "respected-orchestrator-mcp.cmd" } else { "respected-orchestrator-mcp.sh" })
}

/// What clients run: a path that never changes across Orchestrator updates (the launcher is rewritten instead).
pub fn launcher_json(roots: &Roots) -> Value {
    let path = launcher_path(roots).to_string_lossy().to_string();
    if cfg!(windows) { json!({ "command": "cmd", "args": ["/c", path] }) } else { json!({ "command": path, "args": [] }) }
}

/// The program and leading arguments of the launcher: the packaged executable, or (development builds) the source on Python.
fn engine_program(settings: &Settings) -> Option<(String, Vec<String>, Option<PathBuf>)> {
    if let Some(exe) = settings.orchestrator_exe.as_ref().filter(|exe| exe.is_file()) { return Some((exe.to_string_lossy().to_string(), Vec::new(), None)); }
    let source = settings.orchestrator_root.join("src");
    if !source.join("respected_orchestrator").is_dir() { return None; }
    Some((rpp::resolve_program(&settings.python), vec!["-m".into(), "respected_orchestrator".into()], Some(source)))
}

pub fn write_launcher(roots: &Roots, settings: &Settings) -> Result<PathBuf, LifecycleError> {
    let path = launcher_path(roots);
    let Some((program, lead, source)) = engine_program(settings) else { return Err(err("VERIFY_FAILED", "Orchestrator kurulu değil.", "Önce Orchestrator'ı Hub'dan kurun.")) };
    let lead = lead.join(" ");
    // the working folder is the launcher's own (never the program folder, which an update replaces); a source run must not
    // import from it either (PYTHONSAFEPATH), so only the named source tree is on the path
    let text = if cfg!(windows) {
        let python = source.map(|s| format!("set \"PYTHONSAFEPATH=1\"\r\nset \"PYTHONPATH={}\"\r\nset \"PYTHONDONTWRITEBYTECODE=1\"\r\n", s.display())).unwrap_or_default();
        format!("@echo off\r\nrem Written by Respected Hub; rewritten when Orchestrator is installed, updated or repaired.\r\ncd /d \"%~dp0\"\r\n{python}\"{program}\" {lead} --data-root \"{}\" mcp\r\n",
            settings.data_root.display())
    } else {
        let python = source.map(|s| format!("PYTHONSAFEPATH=1 PYTHONPATH='{}' PYTHONDONTWRITEBYTECODE=1 ", s.display())).unwrap_or_default();
        format!("#!/bin/sh\n# Written by Respected Hub; rewritten when Orchestrator is installed, updated or repaired.\ncd \"$(dirname \"$0\")\" || exit 1\n{python}exec '{program}' {lead} --data-root '{}' mcp\n",
            settings.data_root.display())
    };
    let io = |e: std::io::Error| err("INTERNAL", format!("Başlatıcı yazılamadı: {e}"), "Hub veri klasörüne yazma iznini denetleyin.");
    fs::create_dir_all(path.parent().unwrap()).map_err(io)?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, text).map_err(io)?;
    fs::rename(&temporary, &path).map_err(io)?;
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o700)); }
    Ok(path)
}

/// Start the launcher exactly as a client would and ask for its tools: proves the registration works with the Hub closed.
pub fn probe(roots: &Roots) -> Result<usize, String> {
    let launch = launcher_json(roots);
    let mut command = Command::new(launch["command"].as_str().unwrap_or(""));
    command.args(launch["args"].as_array().into_iter().flatten().filter_map(Value::as_str))
        .current_dir(std::env::temp_dir()).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    rpp::strip_secret_env(&mut command);
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
    let lines = [json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"respected-hub","version":"1"}}}),
                 json!({"jsonrpc":"2.0","method":"notifications/initialized"}), json!({"jsonrpc":"2.0","id":2,"method":"tools/list"})];
    let input = lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join("\n") + "\n";
    let out = pump(command, Some(input.into_bytes()), Duration::from_secs(60), 256 * 1024).map_err(|e| e.message())?;
    let text = String::from_utf8_lossy(&out.stdout);
    let tools = text.lines().filter_map(|l| serde_json::from_str::<Value>(l).ok()).find(|v| v["id"] == 2)
        .and_then(|v| v["result"]["tools"].as_array().map(|t| t.len())).unwrap_or(0);
    if tools == 0 { Err(format!("MCP sunucusu araç listesi vermedi. {}", String::from_utf8_lossy(&out.stderr).chars().take(200).collect::<String>())) } else { Ok(tools) }
}

fn engine_error(reply: &Value) -> LifecycleError {
    let code = match reply["error"]["code"].as_str().unwrap_or("") {
        "PLAN_CHANGED" => "PLAN_CHANGED", "LIFECYCLE_CONFLICT" => "LIFECYCLE_CONFLICT", "BAD_REQUEST" => "BAD_REQUEST",
        "ROLLBACK_UNAVAILABLE" => "ROLLBACK_UNAVAILABLE",
        "UNKNOWN_OP" | "UNSUPPORTED_OP" => "LIFECYCLE_CONFLICT", _ => "INTERNAL",
    };
    let stale = matches!(reply["error"]["code"].as_str(), Some("UNKNOWN_OP" | "UNSUPPORTED_OP"));
    err(code, reply["error"]["message"].as_str().unwrap_or("Orchestrator yanıt vermedi."),
        if stale { "Kurulu Orchestrator bu özelliği desteklemiyor; Hub'dan güncelleyin.".to_string() } else { reply["error"]["hint"].as_str().unwrap_or("").to_string() })
}

/// Read-only: which clients exist, what Respected owns in them, and what a change would do. Empty `desired` = status.
pub fn plan(roots: &Roots, settings: &Settings, desired: &Value) -> Result<Value, LifecycleError> {
    let reply = rpp::rpp_call_trusted(settings, PRODUCT, "integrations.plan", Some(json!({ "desired": desired, "launcher": launcher_json(roots) })), None);
    if reply["ok"] == true { Ok(reply["result"].clone()) } else { Err(engine_error(&reply)) }
}

fn run(roots: &Roots, settings: &Settings, desired: &Value, digest: &str, id: &str, entry: &mut Value) -> Result<(), LifecycleError> {
    if desired.as_object().map_or(false, |m| m.values().any(|v| *v == true)) {
        write_launcher(roots, settings)?;
        lifecycle::set_status(roots, entry, "verifying")?;
        probe(roots).map_err(|m| err("VERIFY_FAILED", format!("MCP sunucusu başlatılamadı, hiçbir istemci değiştirilmedi: {m}"), "Orchestrator'ı onarın."))?;
    }
    lifecycle::set_status(roots, entry, "switching")?;
    let reply = rpp::rpp_call_trusted(settings, PRODUCT, "integrations.apply",
        Some(json!({ "desired": desired, "launcher": launcher_json(roots), "plan_digest": digest })), Some(id.to_string()));
    if reply["ok"] != true { return Err(engine_error(&reply)); }
    entry["undo"] = json!({ "mcp": true });
    Ok(())
}

pub fn apply(roots: &Roots, settings: &Settings, desired: &Value, digest: &str, id: &str) -> Result<Value, LifecycleError> {
    if !lifecycle::valid_operation_id(id) { return Err(err("BAD_REQUEST", "İşlem kimliği geçersiz.", "8-64 karakter; harf, rakam, _ veya -.")); }
    if let Some(stored) = lifecycle::read_journal(roots, id) {
        if stored["product"] == PRODUCT && stored["action"] == ACTION { return Ok(stored); }
        return Err(err("LIFECYCLE_CONFLICT", "Bu işlem kimliği başka bir eylem için kullanılmış.", "Yeni bir işlem kimliği kullanın."));
    }
    if !lifecycle::is_idle(roots, PRODUCT) { return Err(err("LIFECYCLE_CONFLICT", "Bu ürün için başka bir işlem sürüyor.", "Bitmesini bekleyin.")); }
    let mut entry = lifecycle::begin(roots, id, PRODUCT, ACTION, Value::Null, Value::Null)?;
    match run(roots, settings, desired, digest, id, &mut entry) {
        Ok(()) => lifecycle::finish(roots, entry, "applied", None),
        Err(error) => { let _ = lifecycle::finish(roots, entry, "failed", Some(&error)); Err(error) }
    }
}

pub fn rollback(roots: &Roots, settings: &Settings, id: &str) -> Result<Value, LifecycleError> {
    let Some(mut entry) = lifecycle::read_journal(roots, id) else { return Err(err("RECEIPT_NOT_FOUND", "Bu kimlik için kayıt yok.", "İşlem kimliğini kontrol edin.")) };
    if entry["status"] == "rolled_back" { return Ok(entry); }
    if entry["status"] != "applied" || entry["undo"]["mcp"] != true { return Err(err("ROLLBACK_UNAVAILABLE", "Bu işlem geri alınamaz.", "İşlem kaydının durumuna bakın.")); }
    let reply = rpp::rpp_call_trusted(settings, PRODUCT, "integrations.rollback", Some(json!({ "operation_id": id })), Some(id.to_string()));
    if reply["ok"] != true { return Err(engine_error(&reply)); }
    entry["status"] = json!("rolled_back");
    lifecycle::save_journal(roots, &entry)?;
    Ok(entry)
}

/// After Orchestrator install/update/repair: keep the stable launcher pointing at the active copy (only if registered once).
pub fn refresh_launcher(roots: &Roots, settings: &Settings) {
    if !launcher_path(roots).is_file() { return; }
    let mut fresh = settings.clone();
    if let Some(exe) = lifecycle::installed_launcher(roots, PRODUCT, crate::ORCHESTRATOR_EXE) { fresh.orchestrator_exe = Some(exe); }
    let _ = write_launcher(roots, &fresh);
}

/// Before Orchestrator is uninstalled: remove the registrations Respected owns so no client keeps a dead server (best effort).
pub fn disable_all(roots: &Roots, settings: &Settings, id: &str) {
    let Ok(status) = plan(roots, settings, &json!({})) else { return };
    let owned: serde_json::Map<String, Value> = status["clients"].as_array().into_iter().flatten()
        .filter(|c| c["state"] == "owned").filter_map(|c| c["client"].as_str().map(|n| (n.to_string(), json!(false)))).collect();
    if owned.is_empty() { return; }
    let desired = Value::Object(owned);
    if let Ok(planned) = plan(roots, settings, &desired) {
        let _ = apply(roots, settings, &desired, planned["plan_digest"].as_str().unwrap_or(""), &format!("mcpoff-{id}").chars().take(64).collect::<String>());
    }
}
