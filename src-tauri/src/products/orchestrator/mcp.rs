// Orchestrator MCP access in AI clients (Claude Code, Codex, ...). The Orchestrator itself edits the client configs (its
// `integrations.*` ops: ownership, foreign entries preserved, entry-level rollback) and the registration names the installed
// program (`<Respected>\RespectedOrchestrator\app\respected-orchestrator.exe mcp`), a path its installer keeps stable across
// updates. The Hub only shows the plan and binds the apply to the approved plan digest; nothing of it stays in the clients.
use crate::lifecycle::LifecycleError;
use crate::{rpp, Settings};
use serde_json::{json, Value};

const PRODUCT: &str = "orchestrator";

fn err(code: &'static str, message: impl Into<String>, hint: impl Into<String>) -> LifecycleError { LifecycleError { code, message: message.into(), hint: hint.into() } }

/// What clients run: the installed program itself.
fn launcher(settings: &Settings) -> Result<Value, LifecycleError> {
    let exe = settings.orchestrator_exe.as_ref().filter(|exe| exe.is_file())
        .ok_or_else(|| err("VERIFY_FAILED", "Orchestrator kurulu değil.", "Önce Orchestrator'ı kurun."))?;
    Ok(json!({ "command": exe.to_string_lossy(), "args": ["mcp"] }))
}

fn engine_error(reply: &Value) -> LifecycleError {
    let code = match reply["error"]["code"].as_str().unwrap_or("") {
        "PLAN_CHANGED" => "PLAN_CHANGED", "LIFECYCLE_CONFLICT" => "LIFECYCLE_CONFLICT", "BAD_REQUEST" => "BAD_REQUEST", _ => "INTERNAL",
    };
    err(code, reply["error"]["message"].as_str().unwrap_or("Orchestrator yanıt vermedi."), reply["error"]["hint"].as_str().unwrap_or(""))
}

/// Read-only: which clients exist, what Respected owns in them, and what a change would do. Empty `desired` = status.
pub fn plan(settings: &Settings, desired: &Value) -> Result<Value, LifecycleError> {
    let reply = rpp::rpp_call_trusted(settings, PRODUCT, "integrations.plan", Some(json!({ "desired": desired, "launcher": launcher(settings)? })), None);
    if reply["ok"] == true { Ok(reply["result"].clone()) } else { Err(engine_error(&reply)) }
}

pub fn apply(settings: &Settings, desired: &Value, digest: &str, id: &str) -> Result<Value, LifecycleError> {
    let reply = rpp::rpp_call_trusted(settings, PRODUCT, "integrations.apply",
        Some(json!({ "desired": desired, "launcher": launcher(settings)?, "plan_digest": digest })), Some(id.to_string()));
    if reply["ok"] == true { Ok(reply["result"].clone()) } else { Err(engine_error(&reply)) }
}
