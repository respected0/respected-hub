//! Authenticity of catalog bundles: Ed25519 over a canonical statement (product, kind, version, schema, digest).
//!
//! Fail closed: a bundle installs only when its signature verifies against a key this build trusts.
//! * PRODUCTION keys are compiled in (`PRODUCTION_KEYS`, provisioned by the release owner). Empty today, so a production
//!   build trusts no bundle until a release key exists.
//! * DEVELOPMENT keys and the explicit `RESPECTED_HUB_ALLOW_UNSIGNED=1` opt-in exist only in debug builds or builds made
//!   with `--features dev-trust`. In a production build that code is compiled out: an unsigned or dev-signed bundle
//!   is rejected there whatever the environment says.

use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::{json, Map, Value};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trust { Production, Development, UnsignedAllowed }

impl Trust {
    pub fn as_str(self) -> &'static str {
        match self { Trust::Production => "production", Trust::Development => "development", Trust::UnsignedAllowed => "unsigned" }
    }
}

/// Public release keys, one `key_id base64-public-key` per line, compiled in from release-keys.txt (empty until the release
/// owner runs common/release/tools/signing/release_keys.py). Never put a private key there.
const PRODUCTION_KEYS: &str = include_str!("../release-keys.txt");

const DOMAIN: &str = "respected-bundle-v1\n";

pub fn dev_enabled() -> bool { cfg!(any(debug_assertions, feature = "dev-trust")) }

/// Development-only environment overrides (isolated test roots, fake user home). A production build has none of them:
/// the variable is not even read, so nothing in the environment can redirect where it installs, what it trusts or whose home it uses.
pub fn dev_env(name: &str) -> Option<std::ffi::OsString> { if dev_enabled() { std::env::var_os(name) } else { None } }

pub fn dev_abs(name: &str) -> Option<PathBuf> { dev_env(name).map(PathBuf::from).filter(|p| p.is_absolute()) }

fn allow_unsigned() -> bool { dev_enabled() && std::env::var("RESPECTED_HUB_ALLOW_UNSIGNED").map_or(false, |v| v == "1") }

fn parse_key_lines(text: &str) -> Vec<(String, Vec<u8>)> {
    text.lines().filter(|line| !line.trim_start().starts_with('#')).filter_map(|line| {
        let mut parts = line.split_whitespace();
        let (id, key) = (parts.next()?, parts.next()?);
        Some((id.to_string(), STANDARD.decode(key).ok()?))
    }).collect()
}

fn dev_keys() -> Vec<(String, Vec<u8>)> {
    if !dev_enabled() { return Vec::new(); }
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(path) = std::env::var_os("RESPECTED_HUB_DEV_TRUST") { candidates.push(PathBuf::from(path)); }
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) { candidates.push(dir.join("dev-trust.pub")); }
    #[cfg(debug_assertions)]
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.build-cache/signing/dev-ed25519.pub"));
    candidates.iter().find_map(|p| std::fs::read_to_string(p).ok()).map(|t| parse_key_lines(&t)).unwrap_or_default()
}

/// The exact bytes that are signed; must match common/release/tools/signing/bundle_signing.py `statement`.
pub fn statement(doc: &Value, product: &str, kind: &str, version: &str, digest: &str) -> Vec<u8> {
    let mut body = Map::new(); // sorted keys, compact: identical to json.dumps(sort_keys=True, separators=(",", ":"))
    body.insert("schema".into(), doc["schema"].clone());
    body.insert("product".into(), json!(product));
    body.insert("kind".into(), json!(kind));
    body.insert("version".into(), json!(version));
    body.insert("bundle_digest".into(), json!(digest));
    let mut out = DOMAIN.as_bytes().to_vec();
    out.extend(serde_json::to_string(&Value::Object(body)).unwrap_or_default().into_bytes());
    out
}

fn verify_with(production: &[(String, Vec<u8>)], development: &[(String, Vec<u8>)], doc: &Value, product: &str, kind: &str, version: &str, digest: &str, allow_unsigned: bool) -> Result<Trust, String> {
    let signature = &doc["signature"];
    if signature.is_null() {
        return if allow_unsigned { Ok(Trust::UnsignedAllowed) } else { Err("paket imzasız; bu derleme yalnızca imzalı paketleri kurar".into()) };
    }
    if signature["alg"].as_str() != Some("ed25519") { return Err("desteklenmeyen imza algoritması".into()); }
    check(production, development, signature["key_id"].as_str().unwrap_or(""), signature["value"].as_str().unwrap_or(""), &statement(doc, product, kind, version, digest))
}

fn check(production: &[(String, Vec<u8>)], development: &[(String, Vec<u8>)], id: &str, value: &str, message: &[u8]) -> Result<Trust, String> {
    let (key, trust) = production.iter().find(|(k, _)| k == id).map(|(_, key)| (key, Trust::Production))
        .or_else(|| development.iter().find(|(k, _)| k == id).map(|(_, key)| (key, Trust::Development)))
        .ok_or_else(|| format!("imza anahtarı bu derlemede güvenilir değil ({id})"))?;
    let key: [u8; 32] = key.as_slice().try_into().map_err(|_| "geçersiz ortak anahtar".to_string())?;
    let key = VerifyingKey::from_bytes(&key).map_err(|_| "geçersiz ortak anahtar".to_string())?;
    let raw = STANDARD.decode(value).map_err(|_| "imza çözülemedi".to_string())?;
    let raw: [u8; 64] = raw.as_slice().try_into().map_err(|_| "imza uzunluğu geçersiz".to_string())?;
    key.verify(message, &Signature::from_bytes(&raw)).map_err(|_| "imza doğrulanamadı".to_string())?;
    Ok(trust)
}

pub fn verify(doc: &Value, product: &str, kind: &str, version: &str, digest: &str) -> Result<Trust, String> {
    verify_with(&parse_key_lines(PRODUCTION_KEYS), &dev_keys(), doc, product, kind, version, digest, allow_unsigned())
}

/// A detached `<key_id> <base64 signature>` line over `message` (the remote catalog index; same format as SHA256SUMS.sig).
/// No unsigned opt-in: an index is always signed.
pub fn verify_detached(message: &[u8], signature_line: &str) -> Result<Trust, String> {
    let mut parts = signature_line.split_whitespace();
    let (Some(id), Some(value), None) = (parts.next(), parts.next(), parts.next()) else { return Err("imza satırı geçersiz".into()) };
    check(&parse_key_lines(PRODUCTION_KEYS), &dev_keys(), id, value, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Produced by common/release/tools/signing/bundle_signing.py with the private key [7; 32]: proves both implementations sign the same bytes.
    const PUB: &str = "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw=";
    const SIG: &str = "winWeIImDdUOidM9i2+2OBsy1StDQfbA60AT+CkMESP7YC6Q9RT39bVRRrrLCDac1K+v/dYyqY67zesIVqB+Cw==";
    const DIGEST: &str = "abababababababababababababababababababababababababababababababab";

    fn doc(signature: Value) -> Value { json!({ "schema": 1, "product": "orchestrator", "version": "0.1.0", "bundle_digest": DIGEST, "signature": signature }) }
    fn good() -> Value { json!({ "alg": "ed25519", "key_id": "k1", "value": SIG }) }
    fn keys() -> Vec<(String, Vec<u8>)> { vec![("k1".into(), STANDARD.decode(PUB).unwrap())] }

    #[test]
    fn a_python_made_signature_verifies_and_reports_the_key_kind() {
        let d = doc(good());
        assert_eq!(verify_with(&keys(), &[], &d, "orchestrator", "source", "0.1.0", DIGEST, false), Ok(Trust::Production));
        assert_eq!(verify_with(&[], &keys(), &d, "orchestrator", "source", "0.1.0", DIGEST, false), Ok(Trust::Development));
    }

    #[test]
    fn the_signature_is_bound_to_product_version_kind_and_digest() {
        let d = doc(good());
        for (product, kind, version, digest) in [("brain", "source", "0.1.0", DIGEST), ("orchestrator", "distribution", "0.1.0", DIGEST),
                                                  ("orchestrator", "source", "0.1.1", DIGEST), ("orchestrator", "source", "0.1.0", "cd".repeat(32).as_str())] {
            assert!(verify_with(&keys(), &[], &d, product, kind, version, digest, false).is_err(), "{product} {kind} {version}");
        }
    }

    #[test]
    fn unknown_keys_bad_signatures_and_unsigned_bundles_fail_closed() {
        let tampered = doc(json!({ "alg": "ed25519", "key_id": "k1", "value": SIG.replacen('w', "x", 1) }));
        assert!(verify_with(&keys(), &[], &tampered, "orchestrator", "source", "0.1.0", DIGEST, false).is_err());
        assert!(verify_with(&[], &[], &doc(good()), "orchestrator", "source", "0.1.0", DIGEST, true).is_err(), "unknown key stays rejected even with the unsigned opt-in");
        assert!(verify_with(&keys(), &[], &doc(json!({ "alg": "rsa", "key_id": "k1", "value": SIG })), "orchestrator", "source", "0.1.0", DIGEST, false).is_err());
        assert!(verify_with(&keys(), &[], &doc(Value::Null), "orchestrator", "source", "0.1.0", DIGEST, false).is_err());
        assert_eq!(verify_with(&keys(), &[], &doc(Value::Null), "orchestrator", "source", "0.1.0", DIGEST, true), Ok(Trust::UnsignedAllowed));
    }

    // Only compiled into (and therefore only proven by) a PRODUCTION build: `cargo test --release --lib production_build` without dev-trust.
    #[cfg(not(any(debug_assertions, feature = "dev-trust")))]
    #[test]
    fn production_build_has_no_development_trust_and_no_unsigned_bypass() {
        std::env::set_var("RESPECTED_HUB_ALLOW_UNSIGNED", "1");
        std::env::set_var("RESPECTED_HUB_DEV_TRUST", "dev-trust.pub");
        assert!(!dev_enabled() && !allow_unsigned() && dev_keys().is_empty());
        // none of the development environment overrides exists in a production build, whatever the environment says
        let absolute = std::env::temp_dir().join("elsewhere");
        for name in ["RESPECTED_HUB_TEST_HOME", "RESPECTED_HUB_MANAGED_ROOT", "RESPECTED_HUB_CATALOG_ROOT", "RESPECTED_HUB_PRODUCTS_ROOT",
                     "RESPECTED_APP_DIR", "RESPECTED_DATA_DIR"] {
            std::env::set_var(name, &absolute);
            assert!(dev_abs(name).is_none() && dev_env(name).is_none(), "{name}");
        }
        let mut command = std::process::Command::new("x");
        crate::rpp::apply_test_home(&mut command);
        assert!(command.get_envs().next().is_none(), "the test home must not reach product processes");
        assert_eq!(crate::hub::installed::product("brain").unwrap().app_dir(), std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_else(|| std::env::temp_dir().into())).join("Respected").join("RespectedBrain").join("app"));
        assert!(verify(&doc(Value::Null), "orchestrator", "source", "0.1.0", DIGEST).is_err());
    }

    #[test]
    fn every_release_key_line_is_a_valid_ed25519_key() {
        let provisioned = parse_key_lines(PRODUCTION_KEYS);
        let lines = PRODUCTION_KEYS.lines().filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#')).count();
        assert_eq!(provisioned.len(), lines, "every non-comment line of release-keys.txt must be a valid key");
        assert!(provisioned.iter().all(|(id, key)| !id.is_empty() && key.len() == 32));
    }
}
