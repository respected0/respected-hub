//! Adversarial proof that the catalog is fail-closed: only a bundle whose Ed25519 signature verifies against a trusted
//! key is offered. Real bundles from the real build tool, temporary roots only.
use respected_hub_lib::lifecycle::{self, Roots};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Temp(PathBuf);
impl Temp {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let path = std::env::temp_dir().join(format!("hub-trust-{label}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Temp(path)
    }
}
impl Drop for Temp { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

fn eco() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..") }

fn python(args: &[&str]) {
    let done = Command::new("python").arg("-B").args(args).status().expect("python");
    assert!(done.success(), "python {args:?}");
}

/// Fresh signed orchestrator bundle in its own catalog; returns (catalog, bundle dir).
fn fresh() -> (Temp, PathBuf) {
    let catalog = Temp::new("catalog");
    python(&[eco().join("respected-hub/release/tools/build_bundle.py").to_str().unwrap(), "orchestrator", "--out", catalog.0.to_str().unwrap()]);
    let bundle = catalog.0.read_dir().unwrap().flatten().next().unwrap().path();
    (catalog, bundle)
}

fn offered(catalog: &Temp) -> (usize, Vec<String>) {
    let roots = Roots { managed: std::env::temp_dir().join("hub-trust-unused"), catalog: catalog.0.clone() };
    let (found, skipped) = lifecycle::catalog(&roots, "orchestrator");
    (found.len(), skipped)
}

fn edit(bundle: &Path, change: impl FnOnce(&mut Value)) {
    let path = bundle.join("bundle.json");
    let mut doc: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    change(&mut doc);
    std::fs::write(path, serde_json::to_vec(&doc).unwrap()).unwrap();
}

#[test]
fn only_a_bundle_signed_by_a_trusted_key_is_offered() {
    std::env::remove_var("RESPECTED_HUB_ALLOW_UNSIGNED");
    // 1. genuine development-signed bundle: offered, with its trust level
    let (catalog, bundle) = fresh();
    let (count, skipped) = offered(&catalog);
    assert_eq!((count, skipped.is_empty()), (1, true), "{skipped:?}");
    let roots = Roots { managed: std::env::temp_dir().join("hub-trust-unused"), catalog: catalog.0.clone() };
    assert_eq!(lifecycle::status(&roots, "orchestrator").unwrap()["available"]["trust"], "development");

    // 2. the signature cannot be moved to another version or product
    let original: Value = serde_json::from_slice(&std::fs::read(bundle.join("bundle.json")).unwrap()).unwrap();
    edit(&bundle, |d| d["version"] = json!("9.9.9"));
    assert_eq!(offered(&catalog).0, 0, "version swap must break the signature");
    std::fs::write(bundle.join("bundle.json"), serde_json::to_vec(&original).unwrap()).unwrap();
    edit(&bundle, |d| d["kind"] = json!("source"));
    assert_eq!(offered(&catalog).0, 0, "only packaged distributions are installable, whatever the signature covers");
    std::fs::write(bundle.join("bundle.json"), serde_json::to_vec(&original).unwrap()).unwrap();

    // 3. a changed payload list (even with a recomputed digest) is not covered by the old signature
    edit(&bundle, |d| {
        d["files"].as_array_mut().unwrap().pop();
        let canonical = serde_json::to_string(&d["files"]).unwrap();
        use sha2::{Digest, Sha256};
        d["bundle_digest"] = json!(Sha256::digest(canonical.as_bytes()).iter().map(|b| format!("{b:02x}")).collect::<String>());
    });
    assert_eq!(offered(&catalog).0, 0, "re-digested payload must break the signature");
    std::fs::write(bundle.join("bundle.json"), serde_json::to_vec(&original).unwrap()).unwrap();

    // 4. a tampered signature, an unknown algorithm, a stripped signature
    edit(&bundle, |d| { let v = d["signature"]["value"].as_str().unwrap().to_string(); d["signature"]["value"] = json!(format!("A{}", &v[1..])); });
    assert_eq!(offered(&catalog).0, 0, "flipped signature bytes");
    std::fs::write(bundle.join("bundle.json"), serde_json::to_vec(&original).unwrap()).unwrap();
    edit(&bundle, |d| d["signature"]["alg"] = json!("rsa"));
    assert_eq!(offered(&catalog).0, 0, "unknown algorithm");
    std::fs::write(bundle.join("bundle.json"), serde_json::to_vec(&original).unwrap()).unwrap();
    edit(&bundle, |d| d["signature"] = Value::Null);
    let (count, skipped) = offered(&catalog);
    assert_eq!(count, 0, "stripped signature must not install: {skipped:?}");
    assert!(skipped[0].contains("imzasız"), "{skipped:?}");

    // 5. an attacker re-signs with their own key: the key id is unknown to this build
    let attacker = Temp::new("attacker-key");
    let key = attacker.0.join("attacker.key");
    python(&["-c", &format!("import base64,sys; sys.path.insert(0, r'{}'); \
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey as K; from cryptography.hazmat.primitives import serialization as s; \
        open(r'{}','w').write(base64.b64encode(K.generate().private_bytes(s.Encoding.Raw, s.PrivateFormat.Raw, s.NoEncryption())).decode())",
        eco().join("respected-hub").join("release").join("tools").display(), key.display())]);
    std::fs::write(bundle.join("bundle.json"), serde_json::to_vec(&original).unwrap()).unwrap();
    python(&[eco().join("respected-hub/release/tools/bundle_signing.py").to_str().unwrap(), "sign", bundle.to_str().unwrap(), "--key", key.to_str().unwrap()]);
    let (count, skipped) = offered(&catalog);
    assert_eq!(count, 0, "attacker-signed bundle must not be offered");
    assert!(skipped[0].contains("güvenilir değil"), "{skipped:?}");

    // 6. the unsigned bundle only exists behind the explicit development opt-in (and this is a debug build)
    edit(&bundle, |d| d["signature"] = Value::Null);
    assert_eq!(offered(&catalog).0, 0);
    std::env::set_var("RESPECTED_HUB_ALLOW_UNSIGNED", "1");
    let (count, _) = offered(&catalog);
    assert_eq!(count, 1, "explicit development opt-in accepts a local unsigned bundle");
    assert_eq!(lifecycle::status(&roots, "orchestrator").unwrap()["available"]["trust"], "unsigned");
    std::env::remove_var("RESPECTED_HUB_ALLOW_UNSIGNED");
}
