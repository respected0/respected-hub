//! The Orchestrator as the Hub runs it: engine operations, campaign listing, verification of a staged copy and its lifecycle.
use serde_json::{json, Value};
use std::{fs, path::Path, time::Duration};
use crate::{lifecycle, orchestrator_mcp, rpp, Settings};
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
    if !output.success { return Err(value.get("error").and_then(Value::as_str).unwrap_or("Engine operation failed").into()); }
    Ok(value)
}

/// Newest-first identities of the campaigns in the data root, as the engine itself reports them.
pub fn list_campaigns(settings: &Settings) -> Result<Vec<String>, String> {
    let reply = call_engine(settings.clone(), "campaign-list", None, None)?;
    let names = reply["campaigns"].as_array().ok_or("Engine campaign list is not an array")?;
    Ok(names.iter().filter_map(Value::as_str).filter(|name| valid_campaign_id(name)).map(str::to_string).collect())
}

/// The packaged Orchestrator's launcher inside its Hub-managed copy.
pub const ORCHESTRATOR_EXE: &str = "respected-orchestrator.exe";

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
    if action == "uninstall" { orchestrator_mcp::disable_all(roots, &settings, operation_id); }
    let applied = lifecycle::apply(roots, "orchestrator", action, operation_id, &engine_verifier(&settings));
    // the stable MCP launcher names the versioned executable: rewrite it whenever the active copy changes
    if applied.is_ok() && matches!(action, "install" | "update" | "repair") { orchestrator_mcp::refresh_launcher(roots, &settings); }
    applied
}
