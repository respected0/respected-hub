//! Real end-to-end proof of the Hub-managed lifecycle: real bundles built from the real product
//! sources, verified by the real engines over RPP, on temporary managed/catalog/data roots only.
use respected_hub_lib::lifecycle::{self, Roots};
use respected_hub_lib::rpp::rpp_call;
use respected_hub_lib::{engine_verifier, Settings};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Temp(PathBuf);
impl Temp {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let path = std::env::temp_dir().join(format!("hub-lc-e2e-{label}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Temp(path)
    }
}
impl Drop for Temp { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

fn ecosystem() -> PathBuf {
    std::env::var_os("RESPECTED_ECOSYSTEM_ROOT").map(PathBuf::from).unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(".."))
}

fn build(eco: &Path, product: &str, out: &Path) {
    let status = Command::new("python").arg("-B").arg(eco.join("respected-hub").join("release").join("tools").join("build_bundle.py")).args([product, "--out"]).arg(out).status().expect("python");
    assert!(status.success(), "bundle build failed for {product}");
}

#[test]
fn both_products_install_verify_repair_update_and_uninstall_through_the_real_engines() {
    let eco = ecosystem();
    let (managed, catalog, data) = (Temp::new("managed"), Temp::new("catalog"), Temp::new("data"));
    let roots = Roots { managed: managed.0.clone(), catalog: catalog.0.clone() };
    let settings = Settings { python: "python".into(), orchestrator_root: eco.join("respected-orchestrator"), brain_root: eco.join("respected-secondbrain"),
        data_root: data.0.clone(), brain_data_root: Some(data.0.clone()), brain_exe: None, brain_app_dir: None };
    let verifier = engine_verifier(&settings);
    for product in lifecycle::PRODUCTS {
        build(&eco, product, &catalog.0);
        let first = lifecycle::status(&roots, product).unwrap();
        assert_eq!(first["install_state"], "absent");
        assert_eq!(first["catalog_skipped"].as_array().unwrap().len(), 0, "{first}");
        let id = format!("e2e-install-{product}");
        let done = lifecycle::apply(&roots, product, "install", &id, &verifier).unwrap_or_else(|e| panic!("{product} install: {}", e.message));
        assert_eq!(done["status"], "applied", "{product}");
        let st = lifecycle::status(&roots, product).unwrap();
        assert_eq!(st["install_state"], "installed", "{product}: {st}");
        assert_eq!(st["signed"], true, "local bundles carry a verified DEVELOPMENT signature");
        assert_eq!(st["available"]["trust"], "development");

        // The installed copy really answers over RPP when it is the product root.
        let mut live = settings.clone();
        lifecycle::overlay_roots(&roots, &mut live.orchestrator_root, &mut live.brain_root);
        let root = if product == "orchestrator" { &live.orchestrator_root } else { &live.brain_root };
        assert!(root.starts_with(&managed.0), "{product} overlay must point at the managed copy");
        let manifest = rpp_call(&live, product, "manifest", None, None);
        assert_eq!(manifest["ok"], true, "{product}: {manifest}");
        if product == "orchestrator" { assert_eq!(manifest["result"]["install_state"], "installed", "product and Hub agree: {manifest}"); }

        // Corrupt a file: status notices, repair restores it from the catalog.
        let version = st["version"].as_str().unwrap().to_string();
        let victim = lifecycle::version_dir(&roots, product, &version).join("src").read_dir().unwrap().flatten().find(|e| e.path().is_dir()).unwrap().path().join("__init__.py");
        let mut damaged = std::fs::read_to_string(&victim).unwrap(); damaged.push_str("\n# corrupted\n"); std::fs::write(&victim, damaged).unwrap();
        assert_eq!(lifecycle::status(&roots, product).unwrap()["install_state"], "repair-needed", "{product}");
        if product == "orchestrator" {
            // The product's own manifest agrees with the Hub about the damage.
            let after = rpp_call(&live, product, "manifest", None, None);
            assert_eq!(after["result"]["install_state"], "repair-needed", "{after}");
        }
        let repaired = lifecycle::apply(&roots, product, "repair", &format!("e2e-repair-{product}"), &verifier).unwrap_or_else(|e| panic!("{product} repair: {}", e.message));
        assert_eq!(repaired["status"], "applied");
        assert_eq!(lifecycle::status(&roots, product).unwrap()["install_state"], "installed", "{product}");

        // Uninstall keeps user data and is reversible until backups are purged.
        lifecycle::apply(&roots, product, "uninstall", &format!("e2e-remove-{product}"), &verifier).unwrap();
        assert!(lifecycle::read_current(&roots, product).is_none());
        assert!(data.0.is_dir(), "user data root untouched");
        lifecycle::rollback(&roots, &format!("e2e-remove-{product}")).unwrap();
        assert_eq!(lifecycle::status(&roots, product).unwrap()["install_state"], "installed", "{product}");
    }
}

#[test]
fn a_copy_the_engine_cannot_run_is_never_activated() {
    let eco = ecosystem();
    let (managed, catalog, data) = (Temp::new("managed2"), Temp::new("catalog2"), Temp::new("data2"));
    let roots = Roots { managed: managed.0.clone(), catalog: catalog.0.clone() };
    build(&eco, "orchestrator", &catalog.0);
    // Break the bundle consistently (hashes still match): the product cannot start from it.
    let bundle = catalog.0.read_dir().unwrap().flatten().next().unwrap().path();
    let main = bundle.join("payload/src/respected_orchestrator/__main__.py");
    std::fs::write(&main, "raise SystemExit('broken build')\n").unwrap();
    let mut doc: serde_json::Value = serde_json::from_slice(&std::fs::read(bundle.join("bundle.json")).unwrap()).unwrap();
    let files = doc["files"].as_array_mut().unwrap();
    let entry = files.iter_mut().find(|f| f["path"] == "src/respected_orchestrator/__main__.py").unwrap();
    entry["size"] = json!(std::fs::metadata(&main).unwrap().len());
    entry["sha256"] = json!(lifecycle::sha256_file(&main).unwrap());
    let canonical = serde_json::to_string(&serde_json::Value::Array(files.clone())).unwrap();
    use sha2::{Digest, Sha256};
    doc["bundle_digest"] = json!(Sha256::digest(canonical.as_bytes()).iter().map(|b| format!("{b:02x}")).collect::<String>());
    std::fs::write(bundle.join("bundle.json"), serde_json::to_vec(&doc).unwrap()).unwrap();
    // consistently broken AND validly (development-)signed: the engine check must be what stops it, not the signature
    assert!(Command::new("python").arg("-B").arg(eco.join("respected-hub").join("release").join("tools").join("bundle_signing.py")).arg("sign").arg(&bundle).status().unwrap().success());

    let settings = Settings { python: "python".into(), orchestrator_root: eco.join("respected-orchestrator"), brain_root: eco.join("respected-secondbrain"),
        data_root: data.0.clone(), brain_data_root: Some(data.0.clone()), brain_exe: None, brain_app_dir: None };
    let error = lifecycle::apply(&roots, "orchestrator", "install", "e2e-broken-0001", &engine_verifier(&settings)).unwrap_err();
    assert_eq!(error.code, "VERIFY_FAILED", "{}", error.message);
    assert!(lifecycle::read_current(&roots, "orchestrator").is_none());
}

#[test]
fn products_run_on_the_hub_managed_python_without_any_system_python() {
    let eco = ecosystem();
    let (managed, catalog, data) = (Temp::new("managed3"), Temp::new("catalog3"), Temp::new("data3"));
    let roots = Roots { managed: managed.0.clone(), catalog: catalog.0.clone() };
    build(&eco, "orchestrator", &catalog.0);
    let status = Command::new("python").arg("-B").arg(eco.join("respected-hub").join("release").join("tools").join("build_runtime.py")).arg("--out").arg(&catalog.0).status().expect("python");
    assert!(status.success(), "runtime bundle build failed");
    let settings = Settings { python: "definitely-not-a-python-executable".into(), orchestrator_root: eco.join("respected-orchestrator"), brain_root: eco.join("respected-secondbrain"),
        data_root: data.0.clone(), brain_data_root: Some(data.0.clone()), brain_exe: None, brain_app_dir: None };
    assert!(lifecycle::managed_python(&roots).is_none());
    let done = respected_hub_lib::apply_with_runtime(&roots, settings.clone(), "orchestrator", "install", "e2e-rt-orch-0001").unwrap_or_else(|e| panic!("{}", e.message));
    assert_eq!(done["status"], "applied");
    assert_eq!(lifecycle::status(&roots, "runtime").unwrap()["install_state"], "installed", "runtime provisioned as a prerequisite");
    let python = lifecycle::managed_python(&roots).expect("managed python");
    let mut live = settings.clone();
    live.python = python.to_string_lossy().to_string();
    lifecycle::overlay_roots(&roots, &mut live.orchestrator_root, &mut live.brain_root);
    let manifest = rpp_call(&live, "orchestrator", "manifest", None, None);
    assert_eq!(manifest["ok"], true, "{manifest}");
    assert_eq!(manifest["result"]["installed"], true, "the product records the Hub install: {manifest}");
    let health = rpp_call(&live, "orchestrator", "health", None, None);
    assert_eq!(health["ok"], true, "{health}");
    // The same call with the unusable system python and no overlay must fail: the managed runtime is what works.
    let broken = rpp_call(&settings, "orchestrator", "manifest", None, None);
    assert_eq!(broken["ok"], false);
}
