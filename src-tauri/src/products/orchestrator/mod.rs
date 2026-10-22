//! The Orchestrator as the Hub runs it: engine operations and campaign listing (installing it is its own installer's work).
pub mod mcp;
use serde_json::Value;
use std::time::Duration;
use crate::{rpp, Settings};
use crate::process::pump;

const MAX_CAMPAIGN_SPEC: usize = 64 * 1024;

/// Campaign identities the engine accepts: `camp-` followed by 1..60 characters of [a-z0-9-].
pub fn valid_campaign_id(id: &str) -> bool {
    id.strip_prefix("camp-").map_or(false, |rest| {
        (1..=60).contains(&rest.len())
            && rest.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    })
}

pub fn call_engine(settings: Settings, action: &str, run_id: Option<String>, request: Option<Value>) -> Result<Value, String> {
    if !["status", "create", "start", "list", "inspect", "cancel", "check", "retry", "campaign-start", "campaign-status", "campaign-list"].contains(&action) {
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
    if !output.success { return Err(value["error"]["message"].as_str().unwrap_or("Engine operation failed").into()); }
    Ok(value)
}

/// Newest-first identities of the campaigns in the data root, as the engine itself reports them.
pub fn list_campaigns(settings: &Settings) -> Result<Vec<String>, String> {
    let reply = call_engine(settings.clone(), "campaign-list", None, None)?;
    let names = reply["campaigns"].as_array().ok_or("Engine campaign list is not an array")?;
    Ok(names.iter().filter_map(Value::as_str).filter(|name| valid_campaign_id(name)).map(str::to_string).collect())
}
