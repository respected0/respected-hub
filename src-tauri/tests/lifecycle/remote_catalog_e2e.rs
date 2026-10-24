//! The download path end to end: a catalog made by the real release tool (development key, file:// URLs) is fetched with
//! curl, verified, and its archive unpacked into the catalog cache the Hub runs installers from. Tampered archives, tampered
//! indexes and older indexes are refused. Temporary roots only.
use respected_hub_lib::lifecycle::{self, catalog, Roots};
use std::path::{Path, PathBuf};
use std::process::Command;
#[path = "../common/mod.rs"]
mod common;
use common::Temp;

fn tools() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("common").join("release").join("tools") }

fn python(args: &[&str]) {
    let done = Command::new("python").arg("-B").args(args).status().expect("python");
    assert!(done.success(), "python {args:?}");
}

/// A signed orchestrator bundle and a published-style catalog for it in `served`.
fn publish(work: &Path, served: &Path) -> String {
    let stage = work.join("stage");
    python(&[tools().join("build").join("build_bundle.py").to_str().unwrap(), "orchestrator", "--out", stage.to_str().unwrap()]);
    let bundle = stage.read_dir().unwrap().flatten().next().unwrap().path();
    python(&[tools().join("build").join("publish_release.py").to_str().unwrap(), "local-catalog", "--bundle", bundle.to_str().unwrap(), "--out", served.to_str().unwrap()]);
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(bundle.join("bundle.json")).unwrap()).unwrap();
    manifest["version"].as_str().unwrap().to_string()
}

fn url(path: &Path) -> String { format!("file:///{}", path.to_string_lossy().replace('\\', "/").trim_start_matches('/')) }

#[test]
fn a_published_bundle_is_downloaded_verified_and_ready_to_install() {
    std::env::remove_var("RESPECTED_HUB_ALLOW_UNSIGNED");
    let work = Temp::new("work");
    let served = work.0.join("served");
    let version = publish(&work.0, &served);
    std::env::set_var("RESPECTED_HUB_CATALOG_URL", url(&served.join("catalog.json")));
    let managed = work.0.join("managed");
    let roots = Roots { managed: managed.clone(), catalog: managed.join("catalog") };

    // index: fetched, verified, cached; it names the product
    let index = catalog::refresh(&roots).expect("signed index accepted");
    assert_eq!(catalog::entry(&index, "orchestrator", "beta").unwrap().version, version);
    assert!(catalog::cached(&roots).is_some());
    assert!(lifecycle::installers(&roots, "orchestrator").0.is_empty(), "nothing downloaded yet");
    let mut status = serde_json::json!({ "product": "orchestrator", "install_state": "absent", "available": null });
    catalog::overlay_status(&roots, &mut status);
    assert_eq!(status["available"]["version"], version.as_str());
    assert_eq!(status["available"]["remote"], true);

    // archive: downloaded once, verified, unpacked into the cache the lifecycle reads; the zip itself is not kept
    catalog::ensure_bundle(&roots, "orchestrator", None).expect("download");
    let (found, skipped) = lifecycle::installers(&roots, "orchestrator");
    assert_eq!((found.len(), skipped.len()), (1, 0), "{skipped:?}");
    assert_eq!(found[0].version, version);
    assert!(std::fs::read_dir(managed.join("downloads")).unwrap().flatten().all(|e| !e.file_name().to_string_lossy().ends_with(".zip")));
    catalog::ensure_bundle(&roots, "orchestrator", Some(&version)).expect("repair of the installed version needs no network");

    // a damaged cached copy is dropped and downloaded again
    let payload_file = found[0].dir.join("payload").join(&found[0].files[0].0);
    std::fs::write(&payload_file, b"tampered").unwrap();
    catalog::ensure_bundle(&roots, "orchestrator", None).expect("re-download of a damaged copy");
    assert_ne!(std::fs::read(&payload_file).unwrap(), b"tampered");

    // a tampered archive is refused and leaves nothing behind
    catalog::forget(&roots, "orchestrator");
    let archive = served.read_dir().unwrap().flatten().map(|e| e.path()).find(|p| p.extension().map_or(false, |x| x == "zip")).unwrap();
    let original = std::fs::read(&archive).unwrap();
    let mut bad = original.clone();
    let middle = bad.len() / 2;
    bad[middle] ^= 0xff;
    std::fs::write(&archive, &bad).unwrap();
    assert_eq!(catalog::ensure_bundle(&roots, "orchestrator", None).unwrap_err().code, "VERIFY_FAILED");
    assert!(lifecycle::installers(&roots, "orchestrator").0.is_empty());
    std::fs::write(&archive, &original).unwrap();

    // a tampered index is refused; the cached verified one stays
    let index_file = served.join("catalog.json");
    let text = std::fs::read_to_string(&index_file).unwrap();
    std::fs::write(&index_file, text.replacen("\"schema\": 1", "\"schema\":  1", 1)).unwrap();
    assert_eq!(catalog::refresh(&roots).unwrap_err().code, "VERIFY_FAILED");
    assert!(catalog::cached(&roots).is_some());

    // an older index (lower sequence) than the cached one is refused: no rollback to old releases
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let newer_served = work.0.join("served-newer");
    std::fs::create_dir_all(&newer_served).unwrap();
    publish(&work.0.join("again"), &newer_served);
    std::fs::write(&index_file, text).unwrap(); // the original, older one
    std::env::set_var("RESPECTED_HUB_CATALOG_URL", url(&newer_served.join("catalog.json")));
    catalog::refresh(&roots).expect("newer index accepted");
    std::env::set_var("RESPECTED_HUB_CATALOG_URL", url(&index_file));
    assert_eq!(catalog::refresh(&roots).unwrap_err().code, "VERIFY_FAILED", "older sequence refused");
    std::env::remove_var("RESPECTED_HUB_CATALOG_URL");
}
