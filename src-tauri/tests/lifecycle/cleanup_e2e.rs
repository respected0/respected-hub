//! The Hub's own uninstall path: `respected-hub.exe --uninstall-products` (what the uninstaller runs when the user chose to remove the
//! products too) must undo every AI-client registration, remove the products and the Hub's managed folder, and leave the user's
//! notes, vault and foreign client configuration alone. Everything runs on a fake HOME and temporary roots.
use respected_hub_lib::secondbrain::BrainDirs;
use respected_hub_lib::lifecycle::Roots;
use respected_hub_lib::{hub_apply, orchestrator::mcp as orchestrator_mcp, Settings};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
#[path = "../common/mod.rs"]
mod common;
use common::{ecosystem, Temp};

// the tests point process-wide HOME variables at their own fake homes
static ENV: Mutex<()> = Mutex::new(());

fn json_of(path: &Path) -> Value { serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap() }

#[test]
fn uninstalling_the_hub_can_remove_the_products_and_every_registration_without_touching_user_data() {
    let _env = ENV.lock().unwrap_or_else(|e| e.into_inner());
    let eco = std::env::var_os("RESPECTED_ECOSYSTEM_ROOT").map(PathBuf::from).unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(".."));
    let (managed, catalog, work) = (Temp::new("managed"), Temp::new("catalog"), Temp::new("work"));
    let managed_root = managed.0.join("hubdata").join("managed");
    let home = work.0.join("home");
    std::fs::create_dir_all(home.join(".codex")).unwrap();
    let foreign_json = json!({ "theme": "dark", "mcpServers": { "someone-elses": { "command": "x", "args": [] } } });
    std::fs::write(home.join(".claude.json"), serde_json::to_vec_pretty(&foreign_json).unwrap()).unwrap();
    let foreign_toml = "# mine\nmodel = \"m\"\n\n[mcp_servers.other]\ncommand = \"y\"\n";
    std::fs::write(home.join(".codex/config.toml"), foreign_toml).unwrap();
    for name in ["USERPROFILE", "HOME"] { std::env::set_var(name, &home); }
    std::env::set_var("APPDATA", home.join("AppData"));
    let dirs = BrainDirs { app: work.0.join("app"), data: work.0.join("data") };
    for (name, value) in [("RESPECTED_HUB_BRAIN_APP_DIR", &dirs.app), ("RESPECTED_HUB_BRAIN_DATA_DIR", &dirs.data)] { std::env::set_var(name, value); }

    let built = |args: &[&str], out: &Path| {
        let done = Command::new("python").arg("-B").arg(eco.join("respected-hub").join("release").join("tools").join("build").join("build_bundle.py")).args(args).arg("--out").arg(out).status().unwrap();
        assert!(done.success());
    };
    built(&["secondbrain"], &catalog.0);
    built(&["orchestrator"], &catalog.0);
    let roots = Roots { managed: managed_root.clone(), catalog: catalog.0.clone() };
    let settings = Settings { python: "python".into(), orchestrator_root: eco.join("respected-orchestrator"), brain_root: work.0.join("none"),
        data_root: work.0.join("orch"), brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None };

    let vault = work.0.join("Vault");
    let params = json!({ "vault_path": vault, "profile": { "USER_NAME": "Test" }, "desired": { "global": true, "mcp": true } });
    hub_apply(&roots, &dirs, settings.clone(), "secondbrain", "install", &params, "e2e-clean-brain-1").unwrap_or_else(|e| panic!("brain install: {} {}", e.message, e.hint));
    hub_apply(&roots, &dirs, settings.clone(), "orchestrator", "install", &json!({}), "e2e-clean-orch-1").unwrap_or_else(|e| panic!("orchestrator install: {} {}", e.message, e.hint));
    let desired = json!({ "claude-code": true, "codex": true });
    let planned = orchestrator_mcp::plan(&roots, &settings, &desired).unwrap();
    orchestrator_mcp::apply(&roots, &settings, &desired, planned["plan_digest"].as_str().unwrap(), "e2e-clean-mcp-01").unwrap();
    let registered = json_of(&home.join(".claude.json"));
    assert!(registered["mcpServers"]["respected-vault"].is_object() && registered["mcpServers"]["respected-orchestrator"].is_object());
    assert!(std::fs::read_to_string(home.join(".codex/config.toml")).unwrap().contains("respected-orchestrator"));
    std::fs::write(vault.join("my-own-note.md"), "keep me").unwrap();

    // the clients keep rewriting their files in the meantime
    let mut rewritten = registered.clone();
    rewritten["numStartups"] = json!(7);
    rewritten["mcpServers"]["figma"] = json!({ "command": "npx", "env": { "TOKEN": "sk-live-9f8e7d6c5b4a39281706" } });
    std::fs::write(home.join(".claude.json"), serde_json::to_vec_pretty(&rewritten).unwrap()).unwrap();
    let mut toml = std::fs::read_to_string(home.join(".codex/config.toml")).unwrap();
    toml.push_str("\n[projects.\"C:/new\"]\ntrust_level = \"trusted\"\n");
    std::fs::write(home.join(".codex/config.toml"), &toml).unwrap();

    // the uninstaller's command, run exactly like that: the real executable, development overrides confining it to the temp roots
    let exe = env!("CARGO_BIN_EXE_respected-hub");
    let ran = Command::new(exe).arg("--uninstall-products")
        .env("LOCALAPPDATA", work.0.join("localappdata")).env("USERPROFILE", &home).env("HOME", &home).env("APPDATA", home.join("AppData"))
        .env("RESPECTED_HUB_MANAGED_ROOT", &managed_root).env("RESPECTED_HUB_CATALOG_ROOT", &catalog.0)
        .env("RESPECTED_HUB_BRAIN_APP_DIR", &dirs.app).env("RESPECTED_HUB_BRAIN_DATA_DIR", &dirs.data)
        .env("RESPECTED_ORCHESTRATOR_DATA_ROOT", work.0.join("orch")).output().unwrap();
    let log = std::fs::read_to_string(managed.0.join("hubdata").join("uninstall-products.log")).unwrap_or_default();
    assert!(ran.status.success(), "headless uninstall failed: {log} {}", String::from_utf8_lossy(&ran.stderr));

    let after = json_of(&home.join(".claude.json"));
    assert!(after["mcpServers"]["respected-vault"].is_null() && after["mcpServers"]["respected-orchestrator"].is_null(), "our entries are gone: {after}");
    assert_eq!(after["mcpServers"]["someone-elses"], foreign_json["mcpServers"]["someone-elses"]);
    assert_eq!(after["mcpServers"]["figma"], rewritten["mcpServers"]["figma"], "the user's later additions survive");
    assert_eq!(after["numStartups"], 7);
    let toml_after = std::fs::read_to_string(home.join(".codex/config.toml")).unwrap();
    assert!(!toml_after.contains("respected-orchestrator") && toml_after.contains("[mcp_servers.other]") && toml_after.contains("# mine"), "{toml_after}");
    assert!(toml_after.contains("trust_level = \"trusted\""), "Codex's own later change survives");
    assert!(!dirs.app.join("respectedbrain.exe").exists(), "SecondBrain removed");
    assert!(!managed_root.exists(), "the Hub's managed copies are gone");
    assert_eq!(std::fs::read_to_string(vault.join("my-own-note.md")).unwrap(), "keep me", "the user's notes are untouched");
    assert!(vault.is_dir());
    assert!(!home.join(".claude/settings.json").exists() || !std::fs::read_to_string(home.join(".claude/settings.json")).unwrap().contains("respectedbrain"),
        "no hook of ours is left in the client settings");
}

#[test]
fn the_cleanup_command_is_a_no_op_on_a_machine_with_nothing_installed() {
    let work = Temp::new("empty");
    let exe = env!("CARGO_BIN_EXE_respected-hub");
    let ran = Command::new(exe).arg("--uninstall-products")
        .env("LOCALAPPDATA", work.0.join("localappdata")).env("USERPROFILE", work.0.join("home")).env("HOME", work.0.join("home"))
        .env("RESPECTED_HUB_MANAGED_ROOT", work.0.join("hubdata").join("managed")).env("RESPECTED_HUB_CATALOG_ROOT", work.0.join("none"))
        .env("RESPECTED_HUB_BRAIN_APP_DIR", work.0.join("app")).env("RESPECTED_HUB_BRAIN_DATA_DIR", work.0.join("data"))
        .env("RESPECTED_ORCHESTRATOR_DATA_ROOT", work.0.join("orch")).output().unwrap();
    assert!(ran.status.success(), "{}", String::from_utf8_lossy(&ran.stderr));
    assert!(!work.0.join("home").exists(), "nothing was created in the (fake) home");
}

#[test]
fn a_removal_through_the_hub_is_final_in_the_result_and_in_the_journal() {
    let _env = ENV.lock().unwrap_or_else(|e| e.into_inner());
    let eco = ecosystem();
    let (managed, catalog, work) = (Temp::new("managed2"), Temp::new("catalog2"), Temp::new("work2"));
    let home = work.0.join("home");
    std::fs::create_dir_all(&home).unwrap();
    for name in ["USERPROFILE", "HOME"] { std::env::set_var(name, &home); }
    std::env::set_var("APPDATA", home.join("AppData"));
    let dirs = BrainDirs { app: work.0.join("app"), data: work.0.join("data") };
    assert!(Command::new("python").arg("-B").arg(eco.join("respected-hub").join("release").join("tools").join("build").join("build_bundle.py"))
        .args(["orchestrator", "--out"]).arg(&catalog.0).status().unwrap().success());
    let roots = Roots { managed: managed.0.clone(), catalog: catalog.0.clone() };
    let settings = Settings { python: "python".into(), orchestrator_root: work.0.join("none"), brain_root: work.0.join("none"),
        data_root: work.0.join("orch"), brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None };
    hub_apply(&roots, &dirs, settings.clone(), "orchestrator", "install", &json!({}), "e2e-final-inst-1").unwrap_or_else(|e| panic!("install: {}", e.message));
    let removed = hub_apply(&roots, &dirs, settings, "orchestrator", "uninstall", &json!({}), "e2e-final-rem-01").unwrap_or_else(|e| panic!("uninstall: {}", e.message));
    assert!(removed["undo"].is_null(), "the result offers no undo: {removed}");
    let journal = respected_hub_lib::lifecycle::read_journal(&roots, "e2e-final-rem-01").unwrap();
    assert!(journal["undo"].is_null(), "the journal offers no undo either");
    let refused = respected_hub_lib::hub_rollback(&roots, &dirs, "e2e-final-rem-01").unwrap_err();
    assert_eq!(refused.code, "ROLLBACK_UNAVAILABLE");
}
