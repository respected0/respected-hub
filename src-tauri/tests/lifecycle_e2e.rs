//! Real end-to-end proof of the Hub-managed lifecycle: the real packaged Orchestrator (frozen from its source, signed with
//! the development key), verified by its own RPP answers, on temporary managed/catalog/data roots only.
use respected_hub_lib::lifecycle::{self, Roots};
use respected_hub_lib::rpp::rpp_call;
use respected_hub_lib::{engine_verifier, Settings, ORCHESTRATOR_EXE};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;
mod common;
use common::{ecosystem, Temp};

fn tools(eco: &Path) -> PathBuf { eco.join("respected-hub").join("release").join("tools") }

fn build(eco: &Path, out: &Path) -> PathBuf {
    let status = Command::new("python").arg("-B").arg(tools(eco).join("build_bundle.py")).args(["orchestrator", "--out"]).arg(out).status().expect("python");
    assert!(status.success(), "orchestrator bundle build failed");
    out.read_dir().unwrap().flatten().map(|e| e.path()).find(|p| p.join("bundle.json").is_file()).unwrap()
}

/// No source checkout and no usable Python: only the packaged executable of the managed copy can answer.
fn settings_without_python(data: &Path) -> Settings {
    Settings { python: "definitely-not-a-python-executable".into(), orchestrator_root: data.join("no-source-checkout"), brain_root: data.join("no-brain"),
        data_root: data.to_path_buf(), brain_data_root: Some(data.to_path_buf()), brain_exe: None, brain_app_dir: None, orchestrator_exe: None }
}

#[test]
fn the_packaged_orchestrator_installs_verifies_repairs_and_uninstalls_without_python() {
    let eco = ecosystem();
    let (managed, catalog, data) = (Temp::new("managed"), Temp::new("catalog"), Temp::new("data"));
    let roots = Roots { managed: managed.0.clone(), catalog: catalog.0.clone() };
    let settings = settings_without_python(&data.0);
    let verifier = engine_verifier(&settings);
    build(&eco, &catalog.0);
    let first = lifecycle::status(&roots, "orchestrator").unwrap();
    assert_eq!(first["install_state"], "absent");
    assert_eq!(first["catalog_skipped"].as_array().unwrap().len(), 0, "{first}");
    assert_eq!(rpp_call(&settings, "orchestrator", "manifest", None, None)["error"]["code"], "ENGINE_NOT_FOUND", "nothing runs before the install");

    let done = lifecycle::apply(&roots, "orchestrator", "install", "e2e-install-orch", &verifier).unwrap_or_else(|e| panic!("install: {}", e.message));
    assert_eq!(done["status"], "applied");
    let st = lifecycle::status(&roots, "orchestrator").unwrap();
    assert_eq!(st["install_state"], "installed", "{st}");
    assert_eq!(st["signed"], true, "local bundles carry a verified DEVELOPMENT signature");
    assert_eq!(st["available"]["trust"], "development");

    // The installed copy really answers over RPP, and agrees with the Hub that it is installed.
    let mut live = settings.clone();
    live.orchestrator_exe = lifecycle::installed_launcher(&roots, "orchestrator", ORCHESTRATOR_EXE);
    let exe = live.orchestrator_exe.clone().expect("the managed copy carries its executable");
    assert!(exe.starts_with(&managed.0), "{}", exe.display());
    let manifest = rpp_call(&live, "orchestrator", "manifest", None, None);
    assert_eq!(manifest["ok"], true, "{manifest}");
    assert_eq!(manifest["result"]["install_state"], "installed", "product and Hub agree: {manifest}");
    assert_eq!(rpp_call(&live, "orchestrator", "health", None, None)["ok"], true);
    // The stable MCP launcher a client registers starts the packaged executable on its own (no Hub, no Python).
    respected_hub_lib::orchestrator_mcp::write_launcher(&roots, &live).unwrap_or_else(|e| panic!("launcher: {}", e.message));
    let tools = respected_hub_lib::orchestrator_mcp::probe(&roots).unwrap_or_else(|e| panic!("MCP launcher: {e}"));
    assert!(tools > 0);

    // Damage a runtime file: both the Hub and the product notice; repair restores it from the catalog.
    let version = st["version"].as_str().unwrap().to_string();
    let victim = lifecycle::version_dir(&roots, "orchestrator", &version).join("app").join("base_library.zip");
    let mut bytes = std::fs::read(&victim).unwrap(); bytes.push(0); std::fs::write(&victim, bytes).unwrap();
    assert_eq!(lifecycle::status(&roots, "orchestrator").unwrap()["install_state"], "repair-needed");
    let after = rpp_call(&live, "orchestrator", "manifest", None, None);
    assert_eq!(after["result"]["install_state"], "repair-needed", "{after}");
    let repaired = lifecycle::apply(&roots, "orchestrator", "repair", "e2e-repair-orch", &verifier).unwrap_or_else(|e| panic!("repair: {}", e.message));
    assert_eq!(repaired["status"], "applied");
    assert_eq!(lifecycle::status(&roots, "orchestrator").unwrap()["install_state"], "installed");

    // At this (transaction) layer an uninstall keeps a backup, so the operation itself can be undone; the Hub's final removal
    // (data deleted, no undo) is proven through hub_apply in cleanup_e2e.
    lifecycle::apply(&roots, "orchestrator", "uninstall", "e2e-remove-orch", &verifier).unwrap();
    assert!(lifecycle::read_current(&roots, "orchestrator").is_none());
    assert!(data.0.is_dir(), "user data root untouched");
    lifecycle::rollback(&roots, "e2e-remove-orch").unwrap();
    assert_eq!(lifecycle::status(&roots, "orchestrator").unwrap()["install_state"], "installed");
}

#[test]
fn a_copy_the_engine_cannot_run_is_never_activated() {
    let eco = ecosystem();
    let (managed, catalog, data) = (Temp::new("managed2"), Temp::new("catalog2"), Temp::new("data2"));
    let roots = Roots { managed: managed.0.clone(), catalog: catalog.0.clone() };
    let bundle = build(&eco, &catalog.0);
    // Break the build consistently (bundle hashes, distribution.json and the signature all match): the executable cannot
    // start from it, so the engine's own answer is what must stop the install.
    let library = bundle.join("payload/app/base_library.zip");
    std::fs::write(&library, b"not a zip archive").unwrap();
    let digest = lifecycle::sha256_file(&library).unwrap();
    let distribution = bundle.join("payload/distribution.json");
    let mut dist: serde_json::Value = serde_json::from_slice(&std::fs::read(&distribution).unwrap()).unwrap();
    dist["files"]["app/base_library.zip"] = json!(digest);
    std::fs::write(&distribution, serde_json::to_vec_pretty(&dist).unwrap()).unwrap();
    let mut doc: serde_json::Value = serde_json::from_slice(&std::fs::read(bundle.join("bundle.json")).unwrap()).unwrap();
    let files = doc["files"].as_array_mut().unwrap();
    for (path, file) in [("app/base_library.zip", &library), ("distribution.json", &distribution)] {
        let entry = files.iter_mut().find(|f| f["path"] == path).unwrap();
        entry["size"] = json!(std::fs::metadata(file).unwrap().len());
        entry["sha256"] = json!(lifecycle::sha256_file(file).unwrap());
    }
    let canonical = serde_json::to_string(&serde_json::Value::Array(files.clone())).unwrap();
    use sha2::{Digest, Sha256};
    doc["bundle_digest"] = json!(Sha256::digest(canonical.as_bytes()).iter().map(|b| format!("{b:02x}")).collect::<String>());
    std::fs::write(bundle.join("bundle.json"), serde_json::to_vec(&doc).unwrap()).unwrap();
    assert!(Command::new("python").arg("-B").arg(tools(&eco).join("bundle_signing.py")).arg("sign").arg(&bundle).status().unwrap().success());

    let error = lifecycle::apply(&roots, "orchestrator", "install", "e2e-broken-0001", &engine_verifier(&settings_without_python(&data.0))).unwrap_err();
    assert_eq!(error.code, "VERIFY_FAILED", "{}", error.message);
    assert!(lifecycle::read_current(&roots, "orchestrator").is_none());
    assert!(!lifecycle::version_dir(&roots, "orchestrator", &doc["version"].as_str().unwrap().to_string()).exists(), "nothing staged is left");
}
