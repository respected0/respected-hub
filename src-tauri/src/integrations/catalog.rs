//! Static, compiled-in directory of third-party integrations.
//!
//! Separate from `lifecycle::catalog` (product installers). This catalog has no network access and
//! never installs anything; it only describes verified GitHub sources for the Discover view.

use crate::integrations::model::ClientId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Curated records shipped inside the binary, so the list works with the network offline.
pub const CATALOG_JSON: &str = include_str!("../../assets/third-party-catalog.json");

/// Separate signing domain: a product bundle signature can never be replayed as a third-party catalog signature.
pub const REMOTE_DOMAIN: &str = "respected-thirdparty-catalog-v1\n";
pub const MAX_REMOTE_BYTES: usize = 4 * 1024 * 1024;
pub const CHANNELS: [&str; 2] = ["stable", "beta"];
pub const REMOTE_URL: &str = "https://github.com/respected0/respected-hub/releases/download/third-party-catalog/catalog.json";
const MAX_SIGNATURE_BYTES: u64 = 4096;

/// What a third-party catalog signature proves: the metadata was not changed in transit. It says nothing about the
/// safety of the linked source code.
pub const SIGNATURE_SCOPE_NOTE: &str = "İmza yalnızca katalog metadata bütünlüğünü doğrular; bağlantılı kaynak kodun güvenli olduğunu garanti etmez.";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogKind {
    Mcp,
    Skill,
    Plugin,
    Rule,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: CatalogKind,
    pub description: String,
    pub category: String,
    pub source_url: String,
    pub supported_clients: Vec<ClientId>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub provenance: Option<String>,
}

/// A verified remote catalog document. Deliberately a different shape from the compiled-in list and from the Respected
/// product installer index (`lifecycle::catalog`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteCatalog {
    pub sequence: u64,
    pub channel: String,
    pub entries: Vec<CatalogEntry>,
}

/// Only a canonical GitHub repository link is accepted in MVP (no other host, no plain HTTP).
pub fn is_verified_source_url(url: &str) -> bool {
    let Some(path) = url.strip_prefix("https://github.com/") else { return false };
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) || path.contains('@') || path.contains('\\') {
        return false;
    }
    let mut parts = path.split('/');
    let (Some(owner), Some(repo)) = (parts.next(), parts.next()) else { return false };
    let repo = repo.split(['?', '#']).next().unwrap_or(repo);
    !owner.is_empty()
        && !repo.is_empty()
        && owner != "."
        && owner != ".."
        && repo != "."
        && repo != ".."
        && owner.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && repo.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Parse and validate a catalog document. Duplicate IDs and untrusted links make the whole document invalid.
pub fn parse_catalog(json: &str) -> Result<Vec<CatalogEntry>, String> {
    let entries: Vec<CatalogEntry> = serde_json::from_str(json).map_err(|e| format!("invalid catalog JSON: {e}"))?;
    validate_entries(entries)
}

fn validate_entries(entries: Vec<CatalogEntry>) -> Result<Vec<CatalogEntry>, String> {
    let mut ids = HashSet::new();
    for entry in &entries {
        if entry.id.trim().is_empty() || entry.name.trim().is_empty() || entry.category.trim().is_empty() {
            return Err("catalog entry has an empty id, name or category".into());
        }
        if !ids.insert(entry.id.as_str()) {
            return Err(format!("duplicate catalog id: {}", entry.id));
        }
        if !is_verified_source_url(&entry.source_url) {
            return Err(format!("catalog entry {} has an unverified source URL", entry.id));
        }
        if entry.supported_clients.is_empty() || entry.supported_clients.contains(&ClientId::Unknown) {
            return Err(format!("catalog entry {} has no valid supported client", entry.id));
        }
    }
    Ok(entries)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn signed_message(bytes: &[u8]) -> Vec<u8> {
    let mut message = REMOTE_DOMAIN.as_bytes().to_vec();
    message.extend_from_slice(bytes);
    message
}

/// Verify a detached `<key_id> <base64 signature>` line over the domain-separated catalog bytes.
/// Rejects oversized payloads before parsing, an older sequence than `previous`, a mismatching digest, unknown fields
/// of a catalog entry and every entry rule `parse_catalog` already enforces.
pub fn verify_remote_with(production: &[(String, Vec<u8>)], development: &[(String, Vec<u8>)], bytes: &[u8], signature_line: &str, previous: Option<u64>) -> Result<RemoteCatalog, String> {
    if bytes.len() > MAX_REMOTE_BYTES { return Err(format!("katalog çok büyük ({} bayt)", bytes.len())); }
    crate::trust::verify_detached_with(production, development, &signed_message(bytes), signature_line)?;
    parse_remote(bytes, previous)
}

pub fn verify_remote(bytes: &[u8], signature_line: &str, previous: Option<u64>) -> Result<RemoteCatalog, String> {
    if bytes.len() > MAX_REMOTE_BYTES { return Err(format!("katalog çok büyük ({} bayt)", bytes.len())); }
    crate::trust::verify_detached(&signed_message(bytes), signature_line)?;
    parse_remote(bytes, previous)
}

fn parse_remote(bytes: &[u8], previous: Option<u64>) -> Result<RemoteCatalog, String> {
    let doc: Value = serde_json::from_slice(bytes).map_err(|e| format!("katalog JSON değil ({e})"))?;
    if doc["schema"] != 1 { return Err("desteklenmeyen katalog şeması".into()); }
    let sequence = doc["sequence"].as_u64().ok_or("katalog sequence eksik")?;
    if let Some(old) = previous.filter(|old| sequence < *old) {
        return Err(format!("katalog geriye gidiyor ({sequence} < {old})"));
    }
    let channel = doc["channel"].as_str().ok_or("katalog kanalı eksik")?;
    if !CHANNELS.contains(&channel) { return Err(format!("bilinmeyen katalog kanalı: {channel}")); }
    let node = doc.get("entries").ok_or("katalog kayıtları eksik")?;
    let canonical = serde_json::to_string(node).map_err(|e| e.to_string())?;
    if doc["digest"].as_str() != Some(sha256_hex(canonical.as_bytes()).as_str()) {
        return Err("katalog özeti kayıtlarla uyuşmuyor".into());
    }
    let entries: Vec<CatalogEntry> = serde_json::from_value(node.clone()).map_err(|e| format!("invalid catalog JSON: {e}"))?;
    Ok(RemoteCatalog { sequence, channel: channel.to_string(), entries: validate_entries(entries)? })
}

/// Last downloaded-and-verified catalog, read offline. The signature is checked again on every read; a torn or
/// tampered cache yields `None` instead of unverified entries.
pub fn cached(managed: &Path) -> Option<RemoteCatalog> {
    cached_check(managed, |bytes, signature| verify_remote(bytes, signature, None))
}

fn cached_check(managed: &Path, verify: impl Fn(&[u8], &str) -> Result<RemoteCatalog, String>) -> Option<RemoteCatalog> {
    let (body, signature) = cache_files(managed);
    let bytes = std::fs::read(body).ok()?;
    let signature = std::fs::read_to_string(signature).ok()?;
    verify(&bytes, &signature).ok()
}

pub fn cache_files(managed: &Path) -> (PathBuf, PathBuf) {
    let dir = managed.join("integrations").join("third-party-catalog");
    (dir.join("catalog.json"), dir.join("catalog.json.sig"))
}

/// What the Discover view opens: the last verified download, otherwise the compiled-in list. No network.
pub fn load(managed: &Path) -> Result<(Vec<CatalogEntry>, bool, Option<u64>), String> {
    if let Some(catalog) = cached(managed) {
        return Ok((catalog.entries, true, Some(catalog.sequence)));
    }
    Ok((entries()?, false, None))
}

fn remote_url() -> String {
    crate::trust::dev_env("RESPECTED_HUB_THIRD_PARTY_CATALOG_URL")
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| REMOTE_URL.into())
}

/// Download, verify and atomically keep a newer signed catalog. The previous verified copy is restored if any write
/// fails, so an offline or broken refresh never destroys what already worked.
pub fn refresh(managed: &Path) -> Result<RemoteCatalog, String> {
    refresh_with(managed,
        |url, output, seconds, max| crate::lifecycle::catalog::curl(url, output, seconds, max),
        verify_remote)
}

fn refresh_with(
    managed: &Path,
    fetch: impl Fn(&str, &Path, u32, u64) -> Result<(), String>,
    verify: impl Fn(&[u8], &str, Option<u64>) -> Result<RemoteCatalog, String>,
) -> Result<RemoteCatalog, String> {
    let (body, signature) = cache_files(managed);
    let dir = body.parent().ok_or("katalog dizini geçersiz")?;
    std::fs::create_dir_all(dir).map_err(|error| format!("katalog dizini oluşturulamadı: {error}"))?;
    let staged_body = dir.join("catalog.json.body.part");
    let staged_signature = dir.join("catalog.json.sig.part");
    let cleanup = |_: &String| {
        let _ = std::fs::remove_file(&staged_body);
        let _ = std::fs::remove_file(&staged_signature);
    };
    let url = remote_url();
    fetch(&url, &staged_body, 60, MAX_REMOTE_BYTES as u64)
        .and_then(|_| fetch(&format!("{url}.sig"), &staged_signature, 60, MAX_SIGNATURE_BYTES))
        .map_err(|error| format!("katalog alınamadı: {error}"))
        .inspect_err(cleanup)?;
    let bytes = std::fs::read(&staged_body).map_err(|error| format!("katalog okunamadı: {error}"))?;
    let signature_text = std::fs::read_to_string(&staged_signature).map_err(|error| format!("imza okunamadı: {error}"))?;
    cleanup(&String::new());
    let previous = cached_check(managed, |bytes, signature| verify(bytes, signature, None)).map(|catalog| catalog.sequence);
    let catalog = verify(&bytes, &signature_text, previous).map_err(|error| format!("katalog doğrulanamadı: {error}"))?;
    let old = std::fs::read(&body).ok().zip(std::fs::read(&signature).ok());
    let restore = |old: Option<(Vec<u8>, Vec<u8>)>| match old {
        Some((old_body, old_signature)) => {
            let _ = crate::lifecycle::write_atomic(&body, &old_body);
            let _ = crate::lifecycle::write_atomic(&signature, &old_signature);
        }
        None => {
            let _ = std::fs::remove_file(&body);
            let _ = std::fs::remove_file(&signature);
        }
    };
    if let Err(error) = crate::lifecycle::write_atomic(&body, &bytes).and_then(|_| crate::lifecycle::write_atomic(&signature, signature_text.as_bytes())) {
        restore(old);
        return Err(format!("katalog yazılamadı: {}", error.message));
    }
    if cached_check(managed, |bytes, signature| verify(bytes, signature, None)).as_ref() != Some(&catalog) {
        restore(old);
        return Err("yazılan katalog yeniden doğrulanamadı".into());
    }
    Ok(catalog)
}

/// The shipped catalog. Reading it performs no filesystem or network I/O.
pub fn entries() -> Result<Vec<CatalogEntry>, String> {
    parse_catalog(CATALOG_JSON)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::json;

    fn key(seed: u8) -> (SigningKey, Vec<(String, Vec<u8>)>) {
        let signing = SigningKey::from_bytes(&[seed; 32]);
        (signing.clone(), vec![("k1".into(), signing.verifying_key().to_bytes().to_vec())])
    }

    fn entry(id: &str) -> Value {
        json!({
            "id": id, "name": "Demo", "type": "mcp", "description": "demo", "category": "test",
            "source_url": "https://github.com/example/demo", "supported_clients": ["claude"]
        })
    }

    fn sign(seed: u8, sequence: u64, channel: &str, digest: Option<&str>, entries: Value) -> (Vec<u8>, String) {
        let canonical = serde_json::to_string(&entries).unwrap();
        let digest = digest.map(str::to_string).unwrap_or_else(|| sha256_hex(canonical.as_bytes()));
        let doc = json!({ "schema": 1, "sequence": sequence, "channel": channel, "digest": digest, "entries": entries });
        let bytes = serde_json::to_vec(&doc).unwrap();
        let mut message = REMOTE_DOMAIN.as_bytes().to_vec();
        message.extend_from_slice(&bytes);
        let (signing, _) = key(seed);
        (bytes, format!("k1 {}", STANDARD.encode(signing.sign(&message).to_bytes())))
    }

    #[test]
    fn a_signed_catalog_is_accepted_and_keeps_sequence_and_channel() {
        let (_, keys) = key(7);
        let (bytes, signature) = sign(7, 3, "beta", None, json!([entry("a"), entry("b")]));
        let catalog = verify_remote_with(&keys, &[], &bytes, &signature, Some(2)).unwrap();
        assert_eq!((catalog.sequence, catalog.channel.as_str(), catalog.entries.len()), (3, "beta", 2));
        assert_eq!(catalog.entries[0].id, "a");
    }

    #[test]
    fn bad_signature_unknown_key_and_mixed_key_are_rejected() {
        let (_, keys) = key(7);
        let (_, other) = key(8);
        let (bytes, signature) = sign(7, 1, "stable", None, json!([entry("a")]));
        let mut raw = STANDARD.decode(signature.split_whitespace().nth(1).unwrap()).unwrap();
        raw[0] ^= 1;
        let tampered = format!("k1 {}", STANDARD.encode(raw));
        assert!(verify_remote_with(&keys, &[], &bytes, &tampered, None).is_err());
        assert!(verify_remote_with(&other, &[], &bytes, &signature, None).is_err());
        assert!(verify_remote_with(&[], &keys, &bytes, &signature, None).is_ok(), "development keys are trusted in a dev build");
    }

    #[test]
    fn a_product_bundle_signature_cannot_cross_into_the_third_party_domain() {
        let (signing, keys) = key(7);
        let (bytes, signature) = sign(7, 1, "stable", None, json!([entry("a")]));
        // same key, but the product bundle domain: never valid for a third-party catalog
        let bundle = crate::trust::statement(&json!({ "schema": 1 }), "orchestrator", "source", "0.1.0", &"ab".repeat(32));
        let wrong = format!("k1 {}", STANDARD.encode(signing.sign(&bundle).to_bytes()));
        assert!(verify_remote_with(&keys, &[], &bytes, &wrong, None).is_err());
        // and the third-party signature cannot pass as a bundle signature
        assert!(crate::trust::verify_detached_with(&keys, &[], &bundle, &signature).is_err());
    }

    #[test]
    fn downgrade_and_replay_are_rejected_but_the_same_sequence_is_allowed() {
        let (_, keys) = key(7);
        let (bytes, signature) = sign(7, 4, "stable", None, json!([entry("a")]));
        assert!(verify_remote_with(&keys, &[], &bytes, &signature, Some(5)).is_err(), "older sequence must not replace a newer one");
        assert!(verify_remote_with(&keys, &[], &bytes, &signature, Some(4)).is_ok());
        assert!(verify_remote_with(&keys, &[], &bytes, &signature, None).is_ok());
    }

    #[test]
    fn a_corrupt_digest_is_rejected_even_with_a_valid_signature() {
        let (_, keys) = key(7);
        let (bytes, signature) = sign(7, 1, "stable", Some(&"00".repeat(32)), json!([entry("a")]));
        assert!(verify_remote_with(&keys, &[], &bytes, &signature, None).is_err());
    }

    #[test]
    fn oversized_payloads_are_rejected_before_parsing() {
        let (_, keys) = key(7);
        let bytes = vec![b'{'; MAX_REMOTE_BYTES + 1];
        assert!(verify_remote_with(&keys, &[], &bytes, "k1 AAAA", None).unwrap_err().contains("çok büyük"));
    }

    #[test]
    fn a_mixed_channel_is_rejected_and_a_relabelled_document_breaks_the_signature() {
        let (_, keys) = key(7);
        let (unknown, signature) = sign(7, 1, "nightly", None, json!([entry("a")]));
        assert!(verify_remote_with(&keys, &[], &unknown, &signature, None).unwrap_err().contains("kanal"));
        let (_, signature) = sign(7, 1, "beta", None, json!([entry("a")]));
        let stable = serde_json::to_vec(&json!({ "schema": 1, "sequence": 1, "channel": "stable",
            "digest": "00".repeat(32),
            "entries": [entry("a")] })).unwrap();
        assert!(verify_remote_with(&keys, &[], &stable, &signature, None).is_err());
    }

    #[test]
    fn the_last_verified_catalog_opens_offline_and_a_tampered_cache_is_ignored() {
        let root = std::env::temp_dir().join(format!("hub-thirdparty-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (_, keys) = key(7);
        let (bytes, signature) = sign(7, 9, "beta", None, json!([entry("a")]));
        let (body, sig) = cache_files(&root);
        std::fs::create_dir_all(body.parent().unwrap()).unwrap();
        std::fs::write(&body, &bytes).unwrap();
        std::fs::write(&sig, &signature).unwrap();
        let offline = |b: &[u8], s: &str| verify_remote_with(&keys, &[], b, s, None);
        assert_eq!(cached_check(&root, offline).unwrap().sequence, 9);
        std::fs::write(&body, bytes.iter().copied().chain(std::iter::once(b' ')).collect::<Vec<_>>()).unwrap();
        assert!(cached_check(&root, offline).is_none(), "a tampered cache must not produce entries");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_compiled_catalog_still_parses_without_the_network() {
        assert!(entries().unwrap().len() >= 50);
    }

    #[test]
    fn a_verified_download_replaces_the_cache_and_failed_refreshes_leave_it_intact() {
        let root = std::env::temp_dir().join(format!("hub-thirdparty-refresh-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (_, keys) = key(7);
        let verify = |bytes: &[u8], signature: &str, previous: Option<u64>| verify_remote_with(&keys, &[], bytes, signature, previous);
        let (bytes, signature) = sign(7, 2, "beta", None, json!([entry("fresh")]));
        let download = |_: &str, output: &Path, _: u32, _: u64| {
            if output.extension().is_some_and(|ext| ext == "part") && output.to_string_lossy().ends_with("sig.part") {
                std::fs::write(output, &signature)
            } else {
                std::fs::write(output, &bytes)
            }.map_err(|error| error.to_string())
        };
        let catalog = refresh_with(&root, download, verify).unwrap();
        assert_eq!((catalog.sequence, catalog.entries[0].id.as_str()), (2, "fresh"));
        assert_eq!(cached_check(&root, |b, s| verify(b, s, None)).unwrap(), catalog);
        assert_eq!(load(&root).unwrap().2, None, "test keys are not production trust roots");

        let failed = refresh_with(&root, |_, _, _, _| Err("timeout".into()), verify);
        assert!(failed.unwrap_err().contains("timeout"));
        let (old, signature) = sign(7, 1, "beta", None, json!([entry("old")]));
        let downgrade = |_: &str, output: &Path, _: u32, _: u64| {
            std::fs::write(output, if output.to_string_lossy().ends_with("sig.part") { signature.as_bytes() } else { old.as_slice() })
                .map_err(|error| error.to_string())
        };
        assert!(refresh_with(&root, downgrade, verify).is_err());
        assert_eq!(cached_check(&root, |b, s| verify(b, s, None)).unwrap().sequence, 2, "a failed refresh changed the cache");

        let partial = |_: &str, output: &Path, _: u32, _: u64| {
            let written = if output.to_string_lossy().ends_with("sig.part") { signature.as_bytes() } else { &bytes[..bytes.len() / 2] };
            std::fs::write(output, written).map_err(|error| error.to_string())
        };
        assert!(refresh_with(&root, partial, verify).is_err());
        assert_eq!(cached_check(&root, |b, s| verify(b, s, None)).unwrap().sequence, 2, "a partial download changed the cache");

        let bad_signature = |_: &str, output: &Path, _: u32, _: u64| {
            std::fs::write(output, if output.to_string_lossy().ends_with("sig.part") { b"k1 AAAA" as &[u8] } else { bytes.as_slice() })
                .map_err(|error| error.to_string())
        };
        assert!(refresh_with(&root, bad_signature, verify).is_err());
        assert_eq!(cached_check(&root, |b, s| verify(b, s, None)).unwrap().sequence, 2, "a bad signature changed the cache");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_empty_cache_opens_the_compiled_catalog_without_network_or_signature_claim() {
        let root = std::env::temp_dir().join(format!("hub-thirdparty-static-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (entries, verified, sequence) = load(&root).unwrap();
        assert!(entries.len() >= 50);
        assert_eq!((verified, sequence), (false, None));
        let _ = std::fs::remove_dir_all(&root);
    }
}
