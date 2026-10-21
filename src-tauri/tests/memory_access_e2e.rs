//! Real end-to-end proof of enabling/disabling SecondBrain memory access for the Orchestrator through the
//! real engines. Temporary data roots and a fake home only; the grant token never reaches this test's API.
use respected_hub_lib::memory_access;
use respected_hub_lib::rpp::{brain_rpp_command, rpp_call, rpp_call_trusted};
use respected_hub_lib::Settings;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
mod common;
use common::{ecosystem, Temp};

fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() { if let Some(found) = find_file(&path, name) { return Some(found); } }
        else if path.file_name().map_or(false, |n| n == name) { return Some(path); }
    }
    None
}

fn find_grant<'a>(value: &'a Value, id: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            if map.get("grant_id").or_else(|| map.get("id")).and_then(Value::as_str) == Some(id) { return Some(value); }
            if let Some(entry) = map.get(id) { if entry.is_object() { return Some(entry); } }
            map.values().find_map(|v| find_grant(v, id))
        }
        Value::Array(items) => items.iter().find_map(|v| find_grant(v, id)),
        _ => None,
    }
}

#[test]
fn memory_access_is_enabled_and_disabled_through_the_real_engines() {
    let eco = ecosystem();
    let (orch_data, brain_data, home, vault) = (Temp::new("orch"), Temp::new("brain"), Temp::new("home"), Temp::new("vault"));
    std::env::set_var("RESPECTED_HUB_TEST_HOME", &home.0);
    let settings = Settings { python: "python".into(), orchestrator_root: eco.join("respected-orchestrator"), brain_root: eco.join("respected-secondbrain"),
        data_root: orch_data.0.clone(), brain_data_root: Some(brain_data.0.clone()), brain_exe: None, brain_app_dir: None, orchestrator_exe: None };
    let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp_call_trusted(&settings, p, o, pl, id);

    // SecondBrain needs a registered vault before it can grant anything.
    let install = call("secondbrain", "lifecycle.apply", Some(json!({ "action": "install", "target": { "vault_path": vault.0.to_string_lossy() } })), Some("mem-e2e-vault-1".into()));
    assert_eq!(install["ok"], true, "{install}");

    let status = memory_access::status(&call);
    assert_eq!(status["supported"], true, "{status}");
    assert_eq!(status["enabled"], false, "{status}");

    let command = brain_rpp_command(&settings).expect("brain command");
    let enabled = memory_access::enable(&call, command, false).unwrap_or_else(|e| panic!("enable: {e}"));
    assert_eq!(enabled["enabled"], true);
    let grant_id = enabled["grant_id"].as_str().expect("grant id").to_string();

    let readback = call("orchestrator", "settings.readback", None, None);
    let shown = &readback["result"]["settings"];
    assert_eq!(shown["memory_enabled"], true, "{readback}");
    assert_eq!(shown["memory_grant_id"], grant_id.as_str(), "{readback}");
    assert_eq!(shown["memory_token"], "***set***", "{readback}");
    let token_file = find_file(&orch_data.0, "memory-token").expect("memory-token file in the Orchestrator data root");
    let token = std::fs::read_to_string(&token_file).unwrap().trim().to_string();
    assert!(!token.is_empty());
    let settings_json = std::fs::read_to_string(find_file(&orch_data.0, "settings.json").expect("settings.json")).unwrap();
    assert!(!settings_json.contains("rbm_") && !settings_json.contains(&token), "settings.json must not hold the token");

    let grants_path = find_file(&brain_data.0, "memory-grants.json").or_else(|| find_file(&vault.0, "memory-grants.json")).or_else(|| find_file(&home.0, "memory-grants.json"))
        .expect("SecondBrain grants file");
    let grants_text = std::fs::read_to_string(&grants_path).unwrap();
    assert!(!grants_text.contains(&token), "the grants file must never contain the token");
    let grants: Value = serde_json::from_str(&grants_text).unwrap();
    let grant = find_grant(&grants, &grant_id).unwrap_or_else(|| panic!("grant {grant_id} listed"));
    assert_ne!(grant["revoked"], true, "{grant}");
    assert_eq!(memory_access::status(&call)["enabled"], true);

    let disabled = memory_access::disable(&call).unwrap_or_else(|e| panic!("disable: {e}"));
    assert_eq!(disabled, json!({ "enabled": false, "revoked": true, "pending_revoke": false }));
    let after = call("orchestrator", "settings.readback", None, None);
    assert_eq!(after["result"]["settings"]["memory_enabled"], false, "{after}");
    assert_ne!(after["result"]["settings"]["memory_token"], "***set***", "{after}");
    assert!(!token_file.exists(), "token file must be gone");
    let grants: Value = serde_json::from_str(&std::fs::read_to_string(&grants_path).unwrap()).unwrap();
    assert_eq!(find_grant(&grants, &grant_id).expect("grant still recorded")["revoked"], true);

    // The webview bridge (untrusted) must not reach memory.* ops.
    let denied = rpp_call(&settings, "secondbrain", "memory.grant", Some(json!({ "label": "webview" })), None);
    assert_eq!(denied["ok"], false, "{denied}");
    assert_eq!(denied["error"]["code"], "PERMISSION_DENIED", "{denied}");
}
