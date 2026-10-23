// Remote catalog on GitHub Releases. One signed index (`catalog.json` + `catalog.json.sig`, release key) names the newest
// Hub and product versions per channel (stable / beta). A product archive is downloaded only when the person installs,
// updates or repairs it; it is checked against the index (size + SHA-256), unpacked, checked again by the bundle's own
// Ed25519 signature (lifecycle::read_bundle) and then kept in the local catalog cache that lifecycle/brain already read.
// The kept copy makes repair work offline; an update deletes the older one.
//
// Frozen contract (every shipped Hub reads it): INDEX_URL, the index schema 1 fields below and the signature line format.
use crate::lifecycle::{self, LifecycleError, Roots};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const INDEX_URL: &str = "https://github.com/respected0/respected-hub/releases/download/catalog/catalog.json";
const INDEX_DOMAIN: &str = "respected-catalog-v1\n";
pub const HUB_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;
const CHANNELS: [&str; 2] = ["beta", "stable"];

fn err(code: &'static str, message: impl Into<String>, hint: impl Into<String>) -> LifecycleError {
    LifecycleError { code, message: message.into(), hint: hint.into() }
}

fn index_url() -> String { crate::trust::dev_env("RESPECTED_HUB_CATALOG_URL").and_then(|v| v.into_string().ok()).unwrap_or_else(|| INDEX_URL.into()) }
fn index_path(roots: &Roots) -> PathBuf { roots.managed.join("catalog-index.json") }
fn signature_path(roots: &Roots) -> PathBuf { roots.managed.join("catalog-index.json.sig") }
fn preferences_path(roots: &Roots) -> PathBuf { roots.managed.join("hub-preferences.json") }
fn download_dir(roots: &Roots) -> PathBuf { roots.managed.join("downloads") }

/// Which releases this Hub offers: "beta" (default while everything is beta) also sees stable ones.
pub fn channel(roots: &Roots) -> String {
    fs::read(preferences_path(roots)).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .and_then(|v| v["channel"].as_str().map(str::to_string)).filter(|c| CHANNELS.contains(&c.as_str())).unwrap_or_else(|| "beta".into())
}

pub fn set_channel(roots: &Roots, channel: &str) -> Result<(), LifecycleError> {
    if !CHANNELS.contains(&channel) { return Err(err("BAD_REQUEST", "Bilinmeyen kanal.", "beta veya stable seçin.")); }
    lifecycle::write_atomic(&preferences_path(roots), &serde_json::to_vec_pretty(&json!({ "channel": channel })).unwrap_or_default())
}

/// Windows' own curl (System32, never a PATH lookup). Production downloads are https only, redirects included.
fn curl(url: &str, output: &Path, max_seconds: u32) -> Result<(), String> {
    let program = if cfg!(windows) { std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows")).join("System32").join("curl.exe") } else { PathBuf::from("curl") };
    let mut command = Command::new(program);
    command.args(["--location", "--fail", "--silent", "--show-error", "--max-time", &max_seconds.to_string()]);
    if !crate::trust::dev_enabled() { command.args(["--proto", "=https", "--proto-redir", "=https"]); }
    command.arg("--output").arg(output).arg(url);
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
    let out = command.output().map_err(|e| format!("curl başlatılamadı: {e}"))?;
    if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().chars().take(300).collect()) }
}

fn verified_index(bytes: &[u8], signature: &str) -> Result<Value, String> {
    let mut message = INDEX_DOMAIN.as_bytes().to_vec();
    message.extend_from_slice(bytes);
    crate::trust::verify_detached(&message, signature)?;
    let index: Value = serde_json::from_slice(bytes).map_err(|e| format!("katalog JSON değil ({e})"))?;
    if index["schema"] != 1 || !index["sequence"].is_u64() { return Err("desteklenmeyen katalog şeması".into()); }
    Ok(index)
}

/// The last index this Hub downloaded and verified (no network).
pub fn cached(roots: &Roots) -> Option<Value> {
    let bytes = fs::read(index_path(roots)).ok()?;
    let signature = fs::read_to_string(signature_path(roots)).ok()?;
    verified_index(&bytes, &signature).ok()
}

/// Download, verify and keep the newest index. An older sequence than the cached one is refused (no rollback to old releases).
pub fn refresh(roots: &Roots) -> Result<Value, LifecycleError> {
    let offline = |m: String| err("CATALOG_UNAVAILABLE", format!("Katalog alınamadı: {m}"), "İnternet bağlantınızı kontrol edip yeniden deneyin.");
    let dir = download_dir(roots);
    fs::create_dir_all(&dir).map_err(|e| offline(e.to_string()))?;
    let (body, sig) = (dir.join("catalog.json.part"), dir.join("catalog.json.sig.part"));
    let url = index_url();
    curl(&url, &body, 60).and_then(|_| curl(&format!("{url}.sig"), &sig, 60)).map_err(offline)?;
    let bytes = fs::read(&body).map_err(|e| offline(e.to_string()))?;
    let signature = fs::read_to_string(&sig).map_err(|e| offline(e.to_string()))?;
    let _ = (fs::remove_file(&body), fs::remove_file(&sig));
    let index = verified_index(&bytes, &signature).map_err(|m| err("VERIFY_FAILED", format!("Katalog doğrulanamadı: {m}"), "Kurulu sürüm değiştirilmedi; daha sonra yeniden deneyin."))?;
    if let Some(previous) = cached(roots) {
        if index["sequence"].as_u64() < previous["sequence"].as_u64() { return Err(err("VERIFY_FAILED", "İndirilen katalog elimizdekinden eski.", "Daha sonra yeniden deneyin.")); }
    }
    lifecycle::write_atomic(&signature_path(roots), signature.as_bytes())?;
    lifecycle::write_atomic(&index_path(roots), &bytes)?;
    Ok(index)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry { pub version: String, pub url: String, pub sha256: String, pub size: u64, pub min_hub: Option<String>, pub notes: Option<String> }

impl Entry {
    pub fn to_json(&self) -> Value { json!({ "version": self.version, "size": self.size, "min_hub": self.min_hub, "notes": self.notes }) }
}

fn parse_entry(node: &Value) -> Option<Entry> {
    let version = node["version"].as_str()?.to_string();
    let url = node["url"].as_str()?.to_string();
    let sha256 = node["sha256"].as_str()?.to_ascii_lowercase();
    let size = node["size"].as_u64()?;
    let valid = (1..=32).contains(&version.len()) && version.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+'))
        && (url.starts_with("https://") || crate::trust::dev_enabled()) && sha256.len() == 64 && sha256.bytes().all(|b| b.is_ascii_hexdigit()) && size > 0 && size <= MAX_ARCHIVE;
    // notes are plain text for display only, cut to a sane length
    let notes = node["notes"].as_str().map(|text| text.trim().chars().take(600).collect::<String>()).filter(|text| !text.is_empty());
    valid.then(|| Entry { version, url, sha256, size, min_hub: node["min_hub"].as_str().map(str::to_string), notes })
}

/// Newest release of `name` (a product or "hub") this channel may install.
pub fn entry(index: &Value, name: &str, channel: &str) -> Option<Entry> {
    let node = if name == "hub" { &index["hub"] } else { &index["products"][name] };
    let wanted: &[&str] = if channel == "beta" { &["stable", "beta"] } else { &["stable"] };
    wanted.iter().filter_map(|c| parse_entry(&node[*c])).max_by(|a, b| lifecycle::version_key(&a.version).cmp(&lifecycle::version_key(&b.version)))
}

fn newer(candidate: &str, than: &str) -> bool { lifecycle::version_key(candidate) > lifecycle::version_key(than) }

/// Same release whatever the spelling (`0.1.0b1` from Python metadata, `0.1.0-beta.1` in the catalog).
fn same(a: &str, b: &str) -> bool { lifecycle::version_key(a) == lifecycle::version_key(b) }

/// Every cached installer bundle of `product`.
fn cached_versions(roots: &Roots, product: &str) -> Vec<lifecycle::Bundle> { lifecycle::installers(roots, product).0 }

/// Make sure the catalog cache holds `product` at `want` (repair: the installed version) or else at the newest index release.
/// Network is used only when that bundle is not cached yet.
pub fn ensure_bundle(roots: &Roots, product: &str, want: Option<&str>) -> Result<(), LifecycleError> {
    let mut have = cached_versions(roots, product);
    // a damaged cached copy is dropped so it is downloaded again instead of blocking repair forever
    have.retain(|b| { let intact = lifecycle::tree_problems(&b.dir.join("payload"), b).is_empty(); if !intact { let _ = fs::remove_dir_all(&b.dir); } intact });
    if let Some(version) = want { if have.iter().any(|b| same(&b.version, version)) { return Ok(()); } }
    // the index is refreshed at Hub start and by "check for updates"; a plan (the wizard asks for one per toggle) must not hit the network
    let index = cached(roots).or_else(|| refresh(roots).ok());
    let target = index.as_ref().and_then(|i| entry(i, product, &channel(roots)));
    let Some(target) = target else {
        return if have.is_empty() || want.is_some() { Err(err("CATALOG_UNAVAILABLE", "Bu ürünün paketi indirilemedi.", "İnternet bağlantınızı kontrol edip yeniden deneyin.")) } else { Ok(()) };
    };
    if let Some(version) = want { if !same(&target.version, version) { return Err(err("CATALOG_UNAVAILABLE", format!("{version} sürümünün paketi artık yayında değil."), "Ürünü güncelleyin; güncelleme bozuk dosyaları da yeniler.")); } }
    if have.iter().any(|b| !newer(&target.version, &b.version)) { return Ok(()); }
    if let Some(min) = &target.min_hub { if newer(min, HUB_VERSION) { return Err(err("HUB_TOO_OLD", format!("Bu sürüm Hub {min} veya daha yenisini gerektiriyor."), "Önce Hub'ı güncelleyin.")); } }
    download_bundle(roots, product, &target)
}

fn download_bundle(roots: &Roots, product: &str, target: &Entry) -> Result<(), LifecycleError> {
    let failed = |m: String| err("CATALOG_UNAVAILABLE", format!("Paket indirilemedi: {m}"), "İnternet bağlantınızı kontrol edip yeniden deneyin.");
    let dir = download_dir(roots);
    fs::create_dir_all(&dir).map_err(|e| failed(e.to_string()))?;
    // the local copy keeps the release's own file name (the last part of the signed URL), so it is the same file under one name
    let name = target.url.rsplit('/').next().filter(|n| n.ends_with(".zip") && n.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)))
        .ok_or_else(|| failed("Katalogdaki paket adresi geçersiz.".into()))?;
    let archive = dir.join(name);
    let result = (|| {
        curl(&target.url, &archive, 1800).map_err(failed)?;
        let size = fs::metadata(&archive).map(|m| m.len()).unwrap_or(0);
        let sha = lifecycle::sha256_file(&archive).map_err(|e| failed(e.to_string()))?;
        if size != target.size || sha != target.sha256 { return Err(err("VERIFY_FAILED", "İndirilen paket katalogdaki özetle uyuşmuyor.", "Hiçbir şey kurulmadı; yeniden deneyin.")); }
        unpack(roots, product, target, &archive)
    })();
    let _ = fs::remove_file(&archive);
    result
}

/// Unpack with Windows' own tar into a scratch folder, accept it only if it is exactly the signed bundle the index named.
fn unpack(roots: &Roots, product: &str, target: &Entry, archive: &Path) -> Result<(), LifecycleError> {
    let scratch = roots.catalog.join(format!(".unpack-{}", lifecycle::stamp()));
    let bad = |m: String| err("VERIFY_FAILED", format!("Paket açılamadı: {m}"), "Hiçbir şey kurulmadı; yeniden deneyin.");
    fs::create_dir_all(&scratch).map_err(|e| bad(e.to_string()))?;
    let result = (|| {
        let program = if cfg!(windows) { std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows")).join("System32").join("tar.exe") } else { PathBuf::from("tar") };
        let mut command = Command::new(program);
        command.arg("-xf").arg(archive).arg("-C").arg(&scratch);
        #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
        let out = command.output().map_err(|e| bad(e.to_string()))?;
        if !out.status.success() { return Err(bad(String::from_utf8_lossy(&out.stderr).trim().chars().take(300).collect())); }
        let bundle = lifecycle::read_bundle(&scratch)?;
        if bundle.product != product || !same(&bundle.version, &target.version) {
            return Err(bad("katalogdaki ürün/sürüm ile uyuşmuyor".into()));
        }
        let problems = lifecycle::tree_problems(&scratch.join("payload"), &bundle);
        if !problems.is_empty() { return Err(bad(problems.join("; "))); }
        let destination = roots.catalog.join(format!("{product}-{}", target.version));
        if destination.exists() { fs::remove_dir_all(&destination).map_err(|e| bad(e.to_string()))?; }
        fs::rename(&scratch, &destination).map_err(|e| bad(e.to_string()))
    })();
    if scratch.exists() { let _ = fs::remove_dir_all(&scratch); }
    result
}

/// After a successful install/update keep only the bundle of the version now installed (offline repair needs that one).
pub fn prune(roots: &Roots, product: &str, keep: &str) {
    if !roots.catalog.starts_with(&roots.managed) { return; } // a development catalog folder outside the Hub's own is never pruned
    for bundle in cached_versions(roots, product) { if !same(&bundle.version, keep) { let _ = fs::remove_dir_all(&bundle.dir); } }
}

/// Remove every cached bundle of `product` (uninstall: nothing of ours stays).
pub fn forget(roots: &Roots, product: &str) {
    if !roots.catalog.starts_with(&roots.managed) { return; } // a development catalog folder outside the Hub's own is never pruned
    for bundle in cached_versions(roots, product) { let _ = fs::remove_dir_all(&bundle.dir); }
}

/// Status overlay from the cached index: a newer release shows as available (and as an update when installed).
pub fn overlay_status(roots: &Roots, status: &mut Value) {
    let Some(index) = cached(roots) else { return };
    let channel = channel(roots);
    let Some(product) = status["product"].as_str().map(str::to_string) else { return };
    let Some(release) = entry(&index, &product, &channel) else { return };
    let shown = status["available"]["version"].as_str().map(str::to_string);
    if shown.as_deref().map_or(true, |v| newer(&release.version, v)) {
        status["available"] = json!({ "version": release.version, "remote": true, "size": release.size, "signed": true, "trust": "production", "notes": release.notes });
    }
    if status["install_state"] == "installed" && status["version"].as_str().map_or(false, |v| newer(&release.version, v)) {
        status["install_state"] = json!("update-available");
    }
}

/// Hub self-update offer: the newest Hub release of this channel when it is newer than the running one.
pub fn hub_update(roots: &Roots) -> Value {
    let channel = channel(roots);
    let release = cached(roots).and_then(|index| entry(&index, "hub", &channel)).filter(|e| newer(&e.version, HUB_VERSION));
    json!({ "current": HUB_VERSION, "channel": channel, "available": release.map(|e| e.to_json()), "checked": cached(roots).is_some() })
}

/// Download and verify the newer Hub installer, then start it (progress window only; it closes and reopens the Hub).
pub fn install_hub_update(roots: &Roots) -> Result<(), LifecycleError> {
    let channel = channel(roots);
    let Some(release) = cached(roots).and_then(|index| entry(&index, "hub", &channel)).filter(|e| newer(&e.version, HUB_VERSION)) else {
        return Err(err("LIFECYCLE_CONFLICT", "Hub güncel.", "Güncellemeleri denetleyin."));
    };
    let dir = download_dir(roots);
    fs::create_dir_all(&dir).map_err(|e| err("INTERNAL", e.to_string(), "Disk alanını kontrol edin."))?;
    let setup = dir.join(format!("RespectedHub-Setup-{}.exe", release.version));
    curl(&release.url, &setup, 1800).map_err(|m| err("CATALOG_UNAVAILABLE", format!("Hub güncellemesi indirilemedi: {m}"), "İnternet bağlantınızı kontrol edin."))?;
    let ok = fs::metadata(&setup).map(|m| m.len()).ok() == Some(release.size) && lifecycle::sha256_file(&setup).ok().as_deref() == Some(release.sha256.as_str());
    if !ok { let _ = fs::remove_file(&setup); return Err(err("VERIFY_FAILED", "İndirilen Hub kurulumu katalogdaki özetle uyuşmuyor.", "Hiçbir şey değiştirilmedi.")); }
    Command::new(&setup).args(["/SILENT", "/NOCANCEL", "/RELAUNCH=1"]).spawn().map_err(|e| err("INTERNAL", format!("Kurulum başlatılamadı: {e}"), "Kurulumu indirip elle çalıştırın."))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index() -> Value {
        let release = |v: &str| json!({ "version": v, "url": "https://example.invalid/x.zip", "sha256": "ab".repeat(32), "size": 10 });
        json!({ "schema": 1, "sequence": 3, "hub": { "stable": release("0.1.0"), "beta": release("0.2.0-beta.1") },
                "products": { "orchestrator": { "stable": Value::Null, "beta": release("0.1.0-beta.1") }, "brain": { "stable": release("0.1.0"), "beta": release("0.1.0-beta.2") } } })
    }

    #[test]
    fn channels_pick_the_newest_allowed_release() {
        let i = index();
        assert_eq!(entry(&i, "hub", "beta").unwrap().version, "0.2.0-beta.1");
        assert_eq!(entry(&i, "hub", "stable").unwrap().version, "0.1.0");
        assert_eq!(entry(&i, "orchestrator", "beta").unwrap().version, "0.1.0-beta.1");
        assert!(entry(&i, "orchestrator", "stable").is_none(), "a beta-only product is not offered on the stable channel");
        assert_eq!(entry(&i, "brain", "beta").unwrap().version, "0.1.0", "a final release outranks its own betas");
        assert!(entry(&i, "runtime", "beta").is_none());
    }

    #[test]
    fn malformed_entries_are_ignored() {
        let mut i = index();
        i["products"]["orchestrator"]["beta"]["sha256"] = json!("short");
        assert!(entry(&i, "orchestrator", "beta").is_none());
        i["hub"]["beta"]["size"] = json!(0);
        assert_eq!(entry(&i, "hub", "beta").unwrap().version, "0.1.0");
    }
}
