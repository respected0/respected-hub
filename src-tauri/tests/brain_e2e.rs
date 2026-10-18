//! Real end-to-end proof of SecondBrain's engine-driven lifecycle (frozen launcher, no Python): verified catalog
//! package -> read-only SIL plan -> product `setup` with AI-client integrations -> RPP checks -> repair ->
//! rollback/uninstall. Everything runs on a fake HOME and temporary roots; the user's real configs are never read or written.
use respected_hub_lib::brain::{self, BrainDirs};
use respected_hub_lib::lifecycle::{self, Roots};
use respected_hub_lib::{hub_apply, hub_plan, hub_rollback, hub_status, Settings};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Temp(PathBuf);
impl Temp {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let path = std::env::temp_dir().join(format!("hub-brain-e2e-{label}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Temp(path)
    }
}
impl Drop for Temp { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

fn ecosystem() -> PathBuf {
    std::env::var_os("RESPECTED_ECOSYSTEM_ROOT").map(PathBuf::from).unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(".."))
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() { stack.push(path); } else { found.push(path); }
        }
    }
    found
}

/// Does the file hold the secret in clear text, or inside any base64 value (journals and manifests store images that way)?
fn holds(bytes: &[u8], secret: &str) -> bool {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let text = String::from_utf8_lossy(bytes);
    if text.contains(secret) { return true; }
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='))
        .filter(|token| token.len() >= 24)
        .any(|token| STANDARD.decode(token).map_or(false, |decoded| String::from_utf8_lossy(&decoded).contains(secret)))
}

fn dist(eco: &Path) -> PathBuf {
    let built = Command::new("python").arg("-B").arg(eco.join("tools").join("build_brain_dist.py")).output().expect("python");
    assert!(built.status.success(), "frozen SecondBrain build failed: {}", String::from_utf8_lossy(&built.stderr));
    PathBuf::from(String::from_utf8_lossy(&built.stdout).lines().last().unwrap().trim())
}

#[test]
fn secondbrain_lifecycle_runs_through_its_own_engine_with_safe_integrations() {
    let eco = ecosystem();
    let (managed, catalog, work) = (Temp::new("managed"), Temp::new("catalog"), Temp::new("work"));
    let home = work.0.join("home");
    std::fs::create_dir_all(home.join(".gemini")).unwrap();
    let foreign = r#"{"mcpServers":{"someone-elses":{"command":"x","args":[]}},"theme":"dark"}"#;
    std::fs::write(home.join(".claude.json"), foreign).unwrap();
    // Only this test touches these variables; they confine the product to the fake HOME and temporary roots.
    for name in ["USERPROFILE", "HOME"] { std::env::set_var(name, &home); }
    let dirs = BrainDirs { app: work.0.join("app"), data: work.0.join("data") };
    std::env::set_var("RESPECTED_HUB_BRAIN_APP_DIR", &dirs.app);
    std::env::set_var("RESPECTED_HUB_BRAIN_DATA_DIR", &dirs.data);

    let source = dist(&eco);
    let built = Command::new("python").arg("-B").arg(eco.join("tools").join("build_bundle.py")).args(["secondbrain-dist", "--dist"]).arg(&source).arg("--out").arg(&catalog.0).status().unwrap();
    assert!(built.success());
    let roots = Roots { managed: managed.0.clone(), catalog: catalog.0.clone() };
    let settings = Settings { python: "no-python-needed".into(), orchestrator_root: eco.join("respected-orchestrator"), brain_root: work.0.join("none"),
        data_root: work.0.join("orch"), brain_data_root: None, brain_exe: None, brain_app_dir: None };
    let vault = work.0.join("Vault");
    let params = json!({ "vault_path": vault, "profile": { "USER_NAME": "Test" }, "desired": { "global": true, "mcp": true } });

    let st = hub_status(&roots, &dirs);
    let brain_state = |st: &Value| st["result"]["products"].as_array().unwrap().iter().find(|p| p["product"] == "secondbrain").unwrap().clone();
    assert_eq!(brain_state(&st)["install_state"], "absent");
    assert!(brain_state(&st)["available"]["version"].is_string());

    // Plan (Discover/Inspect/Plan/Explain): read-only, names exactly what would change.
    let plan = hub_plan(&roots, &dirs, &settings, "secondbrain", "install", &params).unwrap_or_else(|e| panic!("plan: {} {}", e.message, e.hint));
    let changes = plan["integrations"]["changes"].as_array().unwrap();
    assert!(changes.iter().any(|c| c["target"].as_str().unwrap().ends_with(".claude.json") && c["action"] == "modify"), "{plan}");
    assert_eq!(std::fs::read_to_string(home.join(".claude.json")).unwrap(), foreign, "planning wrote nothing");
    assert!(!dirs.app.exists() && !vault.exists());

    // Apply: the product's installer does the work.
    let done = hub_apply(&roots, &dirs, settings.clone(), "secondbrain", "install", &params, "e2e-brain-install-1").unwrap_or_else(|e| panic!("apply: {} {}", e.message, e.hint));
    assert_eq!(done["status"], "applied");
    assert_eq!(brain_state(&hub_status(&roots, &dirs))["install_state"], "installed");
    assert!(vault.is_dir() && dirs.app.join("respectedbrain.exe").is_file());
    let claude: Value = serde_json::from_str(&std::fs::read_to_string(home.join(".claude.json")).unwrap()).unwrap();
    assert_eq!(claude["theme"], "dark");
    assert!(claude["mcpServers"]["someone-elses"].is_object(), "foreign MCP entry preserved");
    assert!(claude["mcpServers"]["respected-vault"]["command"].as_str().unwrap().ends_with("respectedbrain.exe"), "owned entry added");
    assert_eq!(hub_apply(&roots, &dirs, settings.clone(), "secondbrain", "install", &params, "e2e-brain-install-1").unwrap()["status"], "applied", "idempotent replay");

    // The product answers over RPP through its own launcher (no Python anywhere).
    let mut live = settings.clone();
    brain::overlay(&roots, &dirs, &mut live);
    let manifest = respected_hub_lib::rpp::rpp_call(&live, "secondbrain", "manifest", None, None);
    assert_eq!(manifest["ok"], true, "{manifest}");
    assert_eq!(manifest["result"]["install_state"], "installed");
    assert_eq!(manifest["result"]["roots"]["vault"].as_str().map(PathBuf::from), Some(vault.clone()));

    // Tamper with an owned file: both the Hub and the product report repair-needed; repair fixes it.
    let victim = dirs.app.join("distribution.json");
    let original = std::fs::read(&victim).unwrap();
    let launcher = dirs.app.join("respectedbrain.exe");
    let launcher_bytes = std::fs::read(&launcher).unwrap();
    std::fs::write(&launcher, b"not the launcher").unwrap();
    assert_eq!(brain_state(&hub_status(&roots, &dirs))["install_state"], "repair-needed");
    // The tampered launcher no longer runs, so repair is driven by the staged (verified) copy.
    let fixed = hub_apply(&roots, &dirs, settings.clone(), "secondbrain", "repair", &json!({}), "e2e-brain-repair-1").unwrap_or_else(|e| panic!("repair: {} {}", e.message, e.hint));
    assert_eq!(fixed["status"], "applied");
    assert_eq!(std::fs::read(&launcher).unwrap(), launcher_bytes);
    assert_eq!(std::fs::read(&victim).unwrap(), original);
    assert_eq!(brain_state(&hub_status(&roots, &dirs))["install_state"], "installed");

    // Integrations can be switched off and on again later; only Respected's own entries move.
    let off = hub_plan(&roots, &dirs, &settings, "secondbrain", "integrations", &json!({ "desired": {} })).unwrap_or_else(|e| panic!("int plan: {} {}", e.message, e.hint));
    assert!(off["integrations"]["changes"].as_array().is_some());
    hub_apply(&roots, &dirs, settings.clone(), "secondbrain", "integrations", &json!({ "desired": {} }), "e2e-brain-int-off").unwrap_or_else(|e| panic!("int off: {} {}", e.message, e.hint));
    let after_off: Value = serde_json::from_str(&std::fs::read_to_string(home.join(".claude.json")).unwrap()).unwrap();
    assert!(after_off["mcpServers"]["respected-vault"].is_null() && after_off["mcpServers"]["someone-elses"].is_object(), "{after_off}");
    hub_apply(&roots, &dirs, settings.clone(), "secondbrain", "integrations", &json!({ "desired": { "mcp": true } }), "e2e-brain-int-on").unwrap_or_else(|e| panic!("int on: {} {}", e.message, e.hint));
    let after_on: Value = serde_json::from_str(&std::fs::read_to_string(home.join(".claude.json")).unwrap()).unwrap();
    assert!(after_on["mcpServers"]["respected-vault"].is_object() && after_on["mcpServers"]["someone-elses"].is_object());

    // Roll the install back: the product's uninstall keeps the vault and restores the foreign config byte for byte.
    assert_eq!(hub_rollback(&roots, &dirs, "e2e-brain-install-1").unwrap_err().code, "ROLLBACK_UNAVAILABLE", "a newer operation (repair) exists");
    lifecycle::rollback(&roots, "e2e-brain-repair-1").ok();
    // The AI client keeps rewriting its own config while SecondBrain is installed: it adds a project, bumps a counter and a
    // user adds another MCP server that carries a secret. Uninstall must still remove only Respected's entries.
    let secret = "sk-live-9f8e7d6c5b4a39281706";
    let mut rewritten: Value = serde_json::from_str(&std::fs::read_to_string(home.join(".claude.json")).unwrap()).unwrap();
    rewritten["numStartups"] = json!(42);
    rewritten["projects"] = json!({ "C:/work": { "allowedTools": [] } });
    rewritten["mcpServers"]["figma"] = json!({ "command": "npx", "env": { "FIGMA_TOKEN": secret } });
    std::fs::write(home.join(".claude.json"), serde_json::to_vec_pretty(&rewritten).unwrap()).unwrap();
    let removed = hub_apply(&roots, &dirs, settings.clone(), "secondbrain", "uninstall", &json!({}), "e2e-brain-remove-1").unwrap_or_else(|e| panic!("uninstall: {} {}", e.message, e.hint));
    assert_eq!(removed["status"], "applied");
    assert!(!dirs.app.join("respectedbrain.exe").exists());
    assert!(vault.is_dir(), "vault kept");
    let mut expected: Value = serde_json::from_str(foreign).unwrap();
    expected["numStartups"] = json!(42);
    expected["projects"] = json!({ "C:/work": { "allowedTools": [] } });
    expected["mcpServers"]["figma"] = json!({ "command": "npx", "env": { "FIGMA_TOKEN": secret } });
    let final_config: Value = serde_json::from_str(&std::fs::read_to_string(home.join(".claude.json")).unwrap()).unwrap();
    assert_eq!(final_config, expected, "only Respected's entries were removed; the client's and the user's changes survive");
    // minimal retention: neither the ownership manifest nor any transaction journal kept a copy of the user's secret
    let mut leaked = Vec::new();
    for entry in walk(&dirs.data) {
        if holds(&std::fs::read(&entry).unwrap_or_default(), secret) { leaked.push(entry); }
    }
    assert!(leaked.is_empty(), "secret retained in: {leaked:?}");

    // A foreign 'respected-vault' entry is a conflict: the plan refuses and nothing is changed.
    let hostile = r#"{"mcpServers":{"respected-vault":{"command":"not-ours","args":[]}}}"#;
    std::fs::write(home.join(".claude.json"), hostile).unwrap();
    let vault2 = work.0.join("Vault2");
    let conflict = hub_plan(&roots, &dirs, &settings, "secondbrain", "install", &json!({ "vault_path": vault2, "desired": { "mcp": true } })).unwrap_err();
    assert_eq!(conflict.code, "LIFECYCLE_CONFLICT", "{}", conflict.message);
    assert_eq!(std::fs::read_to_string(home.join(".claude.json")).unwrap(), hostile);
    assert!(!vault2.exists());
}
