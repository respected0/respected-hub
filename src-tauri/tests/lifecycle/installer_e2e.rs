//! The Hub installs and removes both products through their REAL signed installers (Inno, silent), exactly as a person could.
//! The installers write to the real per-user locations (%LOCALAPPDATA%\Respected\<Product> and the Installed-apps key), which no
//! environment variable redirects, so this test is ignored by default and refuses to run over an existing install:
//!     cargo test --test installer_e2e -- --ignored --nocapture
//! It builds (or reuses) each product's installer with release/tools/build/build_bundle.py.
use respected_hub_lib::hub::installed;
use respected_hub_lib::lifecycle::Roots;
use respected_hub_lib::{hub_apply, hub_plan, hub_status, overlay, Settings};
use serde_json::json;
use std::process::Command;
#[path = "../common/mod.rs"]
mod common;
use common::{ecosystem, Temp};

fn settings() -> Settings {
    let mut settings = Settings { python: "definitely-not-a-python-executable".into(), orchestrator_root: "C:\\no-source-checkout".into(),
        brain_root: "C:\\no-source-checkout".into(), brain_exe: None, orchestrator_exe: None };
    overlay(&mut settings);
    settings
}

#[test]
#[ignore = "installs the real product installers into %LOCALAPPDATA%"]
fn the_hub_installs_and_removes_each_product_through_its_own_installer() {
    if installed::PRODUCTS.iter().any(|p| p.folder().exists()) { eprintln!("a Respected product folder already exists on this machine: not touching it"); return; }
    let eco = ecosystem();
    let (managed, catalog, vault) = (Temp::new("managed"), Temp::new("catalog"), Temp::new("vault"));
    let roots = Roots { managed: managed.0.clone(), catalog: catalog.0.clone() };
    for product in ["orchestrator", "secondbrain"] {
        let made = Command::new("python").arg("-B").arg(eco.join("respected-hub").join("release").join("tools").join("build").join("build_bundle.py"))
            .arg(product).arg("--out").arg(&catalog.0).status().expect("python");
        assert!(made.success(), "{product} installer bundle build failed");
    }

    let outcome = std::panic::catch_unwind(|| {
        let notes = vault.0.join("RespectedOS");
        for (product, params) in [("orchestrator", json!({ "settings": { "default_timeout_seconds": 1200 } })),
                                  ("secondbrain", json!({ "vault_path": notes.to_string_lossy(), "settings": { "assistant_name": "Jarvis", "user_name": "Ada" } }))] {
            let planned = hub_plan(&roots, product, "install", &params).unwrap_or_else(|e| panic!("{product} plan: {}", e.message));
            assert_eq!(planned["plan"]["steps"].as_array().unwrap().len(), 3, "{planned}");
            let done = hub_apply(&roots, settings(), product, "install", &params, &format!("e2e-install-{product}")).unwrap_or_else(|e| panic!("{product} install: {} {}", e.message, e.hint));
            assert_eq!(done["status"], "applied", "{done}");
            let found = installed::product(product).unwrap().installed().expect("the installer put the product in its own place");
            assert!(found.exe.is_file() && found.app_dir.join("unins000.exe").is_file());
            assert_eq!(hub_plan(&roots, product, "install", &params).unwrap_err().code, "LIFECYCLE_CONFLICT", "already installed");
        }
        // what the person chose arrived as the products' own settings
        let readback = respected_hub_lib::rpp::rpp_call(&settings(), "secondbrain", "settings.readback", None, None);
        assert_eq!((readback["result"]["settings"]["assistant_name"].as_str(), readback["result"]["settings"]["user_name"].as_str()), (Some("Jarvis"), Some("Ada")), "{readback}");
        assert!(notes.join(".respected.json").is_file(), "the vault was created where the person chose");
        let status = hub_status(&roots, &settings());
        assert!(status["result"]["products"].as_array().unwrap().iter().all(|p| p["installed"] == true), "{status}");

        for product in ["orchestrator", "secondbrain"] {
            let removed = hub_apply(&roots, settings(), product, "uninstall", &json!({ "purge": true }), &format!("e2e-remove-{product}")).unwrap_or_else(|e| panic!("{product} uninstall: {} {}", e.message, e.hint));
            assert_eq!(removed["status"], "applied", "{removed}");
            assert!(installed::product(product).unwrap().installed().is_none(), "the product's own uninstaller removed it");
        }
        assert!(notes.join(".respected.json").is_file(), "the vault is never removed");
    });

    // never leave the machine changed, whatever happened above
    for product in installed::PRODUCTS.iter() {
        if product.uninstaller().is_file() { let _ = Command::new(product.uninstaller()).args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/PURGE=1"]).status(); }
    }
    if let Err(panic) = outcome { std::panic::resume_unwind(panic); }
}
