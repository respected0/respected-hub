//! The Hub installs, replaces and removes the Orchestrator through its REAL signed installer (Inno, silent).
//! The installer writes to the real per-user locations (%LOCALAPPDATA%\Respected\RespectedOrchestrator\app and the Installed-apps
//! key), which no environment variable redirects, so this test is ignored by default and refuses to run over an existing install:
//!     cargo test --test installer_e2e -- --ignored --nocapture
//! It needs respected-orchestrator/dist/RespectedOrchestrator-Windows-Setup.exe (packaging/build_installer.py).
use respected_hub_lib::hub::standalone;
use respected_hub_lib::lifecycle::Roots;
use respected_hub_lib::secondbrain::BrainDirs;
use respected_hub_lib::{hub_apply, hub_plan, hub_status, Settings};
use serde_json::json;
use std::path::Path;
use std::process::Command;
#[path = "../common/mod.rs"]
mod common;
use common::{ecosystem, Temp};

fn settings(data: &Path) -> Settings {
    Settings { python: "definitely-not-a-python-executable".into(), orchestrator_root: data.join("no-source-checkout"), brain_root: data.join("no-brain"),
        data_root: data.to_path_buf(), brain_data_root: Some(data.to_path_buf()), brain_exe: None, brain_app_dir: None, orchestrator_exe: None }
}

#[test]
#[ignore = "installs the real Orchestrator installer into %LOCALAPPDATA%"]
fn the_hub_installs_replaces_and_removes_the_orchestrator_through_its_own_installer() {
    if standalone::orchestrator().is_some() { eprintln!("an Orchestrator is already installed on this machine: not touching it"); return; }
    let eco = ecosystem();
    let repo = eco.join("respected-orchestrator").join("dist");
    let installer = repo.join("RespectedOrchestrator-Windows-Setup.exe");
    assert!(installer.is_file(), "build it first: python respected-orchestrator/packaging/build_installer.py");
    let (managed, catalog, data) = (Temp::new("managed"), Temp::new("catalog"), Temp::new("data"));
    let roots = Roots { managed: managed.0.clone(), catalog: catalog.0.clone() };
    let dirs = BrainDirs { app: data.0.join("brain").join("app"), data: data.0.join("brain").join("data") };
    let made = Command::new("python").arg("-B").arg(eco.join("respected-hub").join("release").join("tools").join("build").join("build_bundle.py"))
        .args(["orchestrator", "--installer"]).arg(&installer).arg("--dist").arg(repo.join("RespectedOrchestrator")).arg("--out").arg(&catalog.0).status().expect("python");
    assert!(made.success(), "installer bundle build failed");
    let receipts = common_data_dir().join("receipts");
    let had_receipts = receipts.exists();

    let outcome = std::panic::catch_unwind(|| {
        let planned = hub_plan(&roots, &dirs, &settings(&data.0), "orchestrator", "install", &json!({})).unwrap_or_else(|e| panic!("plan: {}", e.message));
        assert_eq!(planned["plan"]["steps"].as_array().unwrap().len(), 3, "{planned}");
        let done = hub_apply(&roots, &dirs, settings(&data.0), "orchestrator", "install", &json!({}), "e2e-installer-install").unwrap_or_else(|e| panic!("install: {} {}", e.message, e.hint));
        assert_eq!(done["status"], "applied", "{done}");
        let found = standalone::orchestrator().expect("the installer put the product in its own place");
        assert!(found.exe.is_file() && found.app_dir.join("unins000.exe").is_file());
        assert!(!roots.managed.join("orchestrator").join("current.json").exists(), "the Hub keeps no copy of its own");
        let status = hub_status(&roots, &dirs);
        let row = status["result"]["products"].as_array().unwrap().iter().find(|p| p["product"] == "orchestrator").unwrap().clone();
        assert_eq!((row["installed"].clone(), row["install_state"].clone(), row["source"].clone()), (json!(true), json!("installed"), json!("standalone")), "{row}");
        let again = hub_plan(&roots, &dirs, &settings(&data.0), "orchestrator", "install", &json!({}));
        assert_eq!(again.unwrap_err().code, "LIFECYCLE_CONFLICT", "already installed");
        let removed = hub_apply(&roots, &dirs, settings(&data.0), "orchestrator", "uninstall", &json!({}), "e2e-installer-remove").unwrap_or_else(|e| panic!("uninstall: {} {}", e.message, e.hint));
        assert_eq!(removed["status"], "applied", "{removed}");
        assert!(standalone::orchestrator().is_none(), "the product's own uninstaller removed it");
    });

    // never leave the machine changed, whatever happened above
    if let Some(left) = standalone::orchestrator() {
        let _ = Command::new(left.app_dir.join("unins000.exe")).args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"]).status();
    }
    if !had_receipts { let _ = std::fs::remove_dir_all(&receipts); let _ = std::fs::remove_dir(common_data_dir()); }
    if let Err(panic) = outcome { std::panic::resume_unwind(panic); }
}

fn common_data_dir() -> std::path::PathBuf {
    respected_hub_lib::secondbrain::respected_root().join("RespectedOrchestrator").join("data")
}
