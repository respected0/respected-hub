//! Every product is installed, updated, repaired and removed through its own signed installer (`<Product>-Windows-Setup.exe`).
//!
//! The Hub takes the installer from the catalog (a bundle of kind `installer`), checks the bundle signature and the file hash,
//! and runs it silently with the same switches a person or a script would use (`/VAULT=`, `/SET=key=value`, `/PURGE=1`). The
//! installer owns the program folder, the Installed-apps entry and the uninstaller, so the result is the same whether the person
//! ran it or the Hub did. Nothing is kept for undo: an installer replaces the program in place, an uninstaller removes it.
use super::installed::{self, Installed, Product};
use crate::lifecycle::{self, Bundle, LifecycleError, Roots};
use serde_json::{json, Value};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(900);
const SILENT: [&str; 3] = ["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"];

type Runner<'a> = &'a dyn Fn(&Path, &[String]) -> Result<(), String>;
type Checker<'a> = &'a dyn Fn(&Installed) -> Result<(), String>;

fn err(code: &'static str, message: impl Into<String>, hint: impl Into<String>) -> LifecycleError { LifecycleError { code, message: message.into(), hint: hint.into() } }

fn product(id: &str) -> Result<&'static Product, LifecycleError> {
    installed::product(id).ok_or_else(|| err("BAD_REQUEST", format!("Bilinmeyen ürün: {id}."), "orchestrator veya brain kullanın."))
}

/// The newest installer bundle of `product` in the catalog cache.
pub fn newest(roots: &Roots, product: &str) -> Option<Bundle> { lifecycle::installers(roots, product).0.into_iter().next() }

/// The installer's own switches for what the person chose: `/VAULT=` (Brain's notes folder), one `/SET=key=value` per
/// product setting, `/PURGE=1` for a removal that also deletes settings and data. Anything else is refused, never passed on.
fn switches(product: &str, action: &str, params: &Value) -> Result<Vec<String>, LifecycleError> {
    let bad = |m: &str| err("BAD_REQUEST", m.to_string(), "Seçimleri gözden geçirip yeniden deneyin.");
    let mut out: Vec<String> = SILENT.iter().map(|s| s.to_string()).collect();
    let params = if params.is_null() { json!({}) } else { params.clone() };
    let Some(map) = params.as_object() else { return Err(bad("Kurulum seçimleri bir nesne olmalı.")) };
    for key in map.keys() {
        if !matches!(key.as_str(), "vault_path" | "settings" | "purge") { return Err(bad("Bilinmeyen kurulum seçimi.")); }
    }
    if action == "uninstall" {
        if map.get("purge") == Some(&json!(true)) { out.push("/PURGE=1".into()); }
        return Ok(out);
    }
    if action != "install" { return Ok(out); } // update and repair keep everything the person set; the installer only replaces the program
    if let Some(vault) = map.get("vault_path") {
        let vault = vault.as_str().filter(|v| product == "brain" && Path::new(v).is_absolute() && !v.contains('"'))
            .ok_or_else(|| bad("Not kasası mutlak bir klasör yolu olmalı."))?;
        out.push(format!("/VAULT={vault}"));
    }
    for (key, value) in map.get("settings").and_then(Value::as_object).into_iter().flatten() {
        let plain = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.';
        if key.is_empty() || !key.chars().all(plain) { return Err(bad("Geçersiz ayar adı.")); }
        let text = match value { Value::Bool(b) => b.to_string(), Value::Number(n) => n.to_string(), Value::String(s) => s.clone(), _ => return Err(bad("Ayar değeri metin, sayı ya da doğru/yanlış olmalı.")) };
        if text.contains('"') || text.chars().any(char::is_control) { return Err(bad("Ayar değerinde tırnak ya da kontrol karakteri olamaz.")); }
        out.push(format!("/SET={key}={text}"));
    }
    Ok(out)
}

pub fn plan(roots: &Roots, id: &str, action: &str, params: &Value) -> Result<Value, LifecycleError> {
    plan_in(roots, &installed::respected_root(), id, action, params)
}

fn plan_in(roots: &Roots, root: &Path, id: &str, action: &str, params: &Value) -> Result<Value, LifecycleError> {
    let product = product(id)?;
    let installed = product.installed_in(root).map(|i| i.version);
    let available = newest(roots, id);
    switches(id, action, params)?;
    let step = |key: &str, summary: String, reversible: bool, reason: Option<&str>| json!({ "id": key, "summary": summary, "reversible": reversible, "irreversible_reason": reason });
    let mut steps = Vec::new();
    match action {
        "install" | "update" | "repair" => {
            let Some(bundle) = &available else { return Err(err("BAD_REQUEST", "Katalogda bu ürünün kurucusu yok.", "Ürün kataloğunu yenileyin (Ayarlar › Güncellemeleri denetle).")) };
            match (action, &installed) {
                ("install", Some(_)) => return Err(err("LIFECYCLE_CONFLICT", format!("{} zaten kurulu.", product.name), "Güncelleme veya onarım kullanın.")),
                ("update" | "repair", None) => return Err(err("LIFECYCLE_CONFLICT", format!("{} kurulu değil.", product.name), "Önce kurun.")),
                ("update", Some(current)) if lifecycle::version_key(&bundle.version) <= lifecycle::version_key(current) => return Err(err("LIFECYCLE_CONFLICT", "Güncelleme yok; kurulu sürüm güncel.", "Sorun varsa onarımı kullanın.")),
                _ => {}
            }
            if action == "update" { if let Some(current) = &installed { if let Some(problem) = crate::lifecycle::catalog::path_problem(roots, id, current) { return Err(problem); } } }
            steps.push(step("verify", format!("{}'in imzalı kurucusunu (v{}) doğrula", product.name, bundle.version), true, None));
            steps.push(step("install", format!("Kurucuyu sessiz çalıştır: {}", if action == "install" { "program, ayarlar ve Yüklü uygulamalar girdisi" } else { "program yerinde yenilenir, ayarlar korunur" }), true,
                if action == "install" { None } else { Some("Önceki sürüm yerinde değiştirilir") }));
            steps.push(step("check", "Ürünün kendisine kurulumu doğrulat".into(), true, None));
        }
        "uninstall" => {
            if installed.is_none() { return Err(err("LIFECYCLE_CONFLICT", format!("{} kurulu değil.", product.name), "Kaldırılacak bir şey yok.")); }
            let purge = params.get("purge") == Some(&json!(true));
            steps.push(step("remove", format!("{}'in kendi kaldırıcısını çalıştır{}", product.name, if purge { "; ayarlar ve çalışma verileri de silinir" } else { "; ayarlar ve çalışma verileri kalır" }), false, Some("Kaldırma geri alınamaz; yeniden kurmak gerekir")));
            steps.push(step("keep-notes", "Not kasan ve notların hiçbir durumda silinmez".into(), true, None));
        }
        _ => return Err(err("BAD_REQUEST", format!("Bilinmeyen eylem: {action}."), "install, update, repair veya uninstall kullanın.")),
    }
    Ok(json!({ "product": id, "action": action, "dry_run": true, "from_version": installed,
        "to_version": available.map(|b| b.version), "plan": { "steps": steps } }))
}

pub fn apply(roots: &Roots, id: &str, action: &str, params: &Value, operation_id: &str, verify: Checker) -> Result<Value, LifecycleError> {
    apply_in(roots, &installed::respected_root(), id, action, params, operation_id, &run_silently, verify)
}

#[allow(clippy::too_many_arguments)]
fn apply_in(roots: &Roots, root: &Path, id: &str, action: &str, params: &Value, operation_id: &str, run: Runner, verify: Checker) -> Result<Value, LifecycleError> {
    if !lifecycle::valid_operation_id(operation_id) { return Err(err("BAD_REQUEST", "İşlem kimliği geçersiz.", "8-64 karakter; harf, rakam, _ veya -.")); }
    if let Some(stored) = lifecycle::read_journal(roots, operation_id) {
        if stored["product"] == id && stored["action"] == action { return Ok(stored); }
        return Err(err("LIFECYCLE_CONFLICT", "Bu işlem kimliği başka bir eylem için kullanılmış.", "Yeni bir işlem kimliği kullanın."));
    }
    if !lifecycle::is_idle(roots, id) { return Err(err("LIFECYCLE_CONFLICT", "Bu ürün için başka bir işlem sürüyor.", "Bitmesini bekleyin.")); }
    let planned = plan_in(roots, root, id, action, params)?;
    let args = switches(id, action, params)?;
    let mut entry = lifecycle::begin(roots, operation_id, id, action, planned["from_version"].clone(), planned["to_version"].clone())?;
    match run_action(roots, root, id, action, &args, &mut entry, run, verify) {
        Ok(()) => lifecycle::finish(roots, entry, "applied", None),
        Err(error) => { let _ = lifecycle::finish(roots, entry, "failed", Some(&error)); Err(error) }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_action(roots: &Roots, root: &Path, id: &str, action: &str, args: &[String], entry: &mut Value, run: Runner, verify: Checker) -> Result<(), LifecycleError> {
    let product = product(id)?;
    if action == "uninstall" {
        let uninstaller = uninstaller_in(&product.app_dir_in(root));
        if !uninstaller.is_file() { return Err(err("VERIFY_FAILED", "Ürünün kaldırıcısı bulunamadı.", "Windows'taki Yüklü uygulamalar listesinden kaldırın.")); }
        lifecycle::set_status(roots, entry, "switching")?;
        run(&uninstaller, args).map_err(|m| err("LIFECYCLE_PARTIAL", format!("Kaldırıcı tamamlanamadı: {m}"), "Ürün çalışıyorsa kapatıp yeniden deneyin."))?;
        // Inno's uninstaller hands over to a copy of itself and returns at once: wait for the product to be gone
        for _ in 0..90 {
            if product.installed_in(root).is_none() { return Ok(()); }
            std::thread::sleep(Duration::from_secs(1));
        }
        return Err(err("VERIFY_FAILED", "Kaldırma sonrası ürün hâlâ kurulu görünüyor.", "Kaldırıcı günlüğüne bakın."));
    }
    let Some(bundle) = newest(roots, id) else { return Err(err("BAD_REQUEST", "Katalogda bu ürünün kurucusu yok.", "Ürün kataloğunu yenileyin.")) };
    let staging = roots.managed.join("installer-staging").join(lifecycle::stamp().to_string());
    let outcome = (|| {
        lifecycle::stage_bundle(&bundle, &staging)?;
        let problems = lifecycle::tree_problems(&staging, &bundle);
        if !problems.is_empty() { return Err(err("VERIFY_FAILED", format!("Kurucu doğrulanamadı: {}", problems.join("; ")), "Yeniden deneyin.")); }
        lifecycle::set_status(roots, entry, "switching")?;
        run(&staging.join(&bundle.files[0].0), args).map_err(|m| err("LIFECYCLE_PARTIAL", format!("Kurucu tamamlanamadı: {m}"), "Ürün çalışıyorsa kapatıp yeniden deneyin; ürün kendi değişikliklerini geri alır."))?;
        lifecycle::set_status(roots, entry, "verifying")?;
        let installed = product.installed_in(root).ok_or_else(|| err("VERIFY_FAILED", "Kurulumdan sonra ürünün kaydı bulunamadı.", "Kurucu günlüğüne bakın."))?;
        if lifecycle::version_key(&installed.version) != lifecycle::version_key(&bundle.version) { return Err(err("VERIFY_FAILED", format!("Kurulu sürüm {} beklenen {} değil.", installed.version, bundle.version), "Yeniden deneyin.")); }
        verify(&installed).map_err(|m| err("VERIFY_FAILED", format!("Ürün kurulumdan sonra yanıt vermedi: {m}"), "Ürünü onarın."))
    })();
    let _ = std::fs::remove_dir_all(&staging);
    outcome?;
    entry["to_version"] = json!(bundle.version);
    Ok(())
}

/// Inno numbers its uninstaller files (unins000, unins001...) when an interrupted install left a broken pair behind (an empty
/// `.dat`): the newest pair that is not broken is the one the Installed-apps entry points to.
fn uninstaller_in(app: &Path) -> std::path::PathBuf {
    let mut found: Vec<std::path::PathBuf> = std::fs::read_dir(app).into_iter().flatten().flatten().map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "exe") && p.file_stem().and_then(|s| s.to_str()).is_some_and(|s| s.len() == 8 && s.starts_with("unins") && s[5..].bytes().all(|b| b.is_ascii_digit())))
        .filter(|p| std::fs::metadata(p.with_extension("dat")).map_or(true, |m| m.len() > 0)).collect();
    found.sort();
    found.pop().unwrap_or_else(|| app.join("unins000.exe"))
}

/// Run an installer or uninstaller to its end. No pipes: Inno's second-stage processes outlive the first and would hold them.
fn run_silently(exe: &Path, args: &[String]) -> Result<(), String> {
    let mut command = Command::new(exe);
    command.args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    crate::rpp::apply_test_home(&mut command); // isolated test runs: the product's setup writes AI-tool configs into the fake home
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); } // CREATE_NO_WINDOW
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let deadline = Instant::now() + TIMEOUT;
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            return if status.success() { Ok(()) } else { Err(format!("çıkış kodu {}", status.code().map_or("?".to_string(), |c| c.to_string()))) };
        }
        if Instant::now() >= deadline { let _ = child.kill(); let _ = child.wait(); return Err("zaman aşımı".into()); }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::cell::RefCell;
    use std::fs;
    use std::path::PathBuf;
    use std::rc::Rc;

    fn temp(label: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("hub-installer-{label}-{}-{}", std::process::id(), lifecycle::stamp()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    /// A catalog with one hand-made, unsigned (development opt-in) installer bundle.
    fn fixture(id: &str, setup: &str, version: &str) -> Roots {
        let roots = Roots { managed: temp("managed"), catalog: temp("catalog") };
        let dir = roots.catalog.join(format!("{id}-{version}"));
        fs::create_dir_all(dir.join("payload")).unwrap();
        fs::write(dir.join("payload").join(setup), b"fake installer").unwrap();
        let file = json!({ "path": setup, "size": 14, "sha256": lifecycle::sha256_file(&dir.join("payload").join(setup)).unwrap() });
        let digest: String = Sha256::digest(serde_json::to_string(&json!([file.clone()])).unwrap().as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
        std::env::set_var("RESPECTED_HUB_ALLOW_UNSIGNED", "1");
        fs::write(dir.join("bundle.json"), serde_json::to_vec(&json!({ "schema": 1, "product": id, "kind": "installer", "version": version,
            "files": [file], "bundle_digest": digest, "signature": Value::Null })).unwrap()).unwrap();
        roots
    }

    /// Stands in for Inno and the product's own setup: the installer lays down the program and the product writes its record;
    /// the uninstaller (a handed-over copy, so it returns first) removes them a moment later.
    fn fake_run(root: &Path, id: &'static str, version: &str, calls: &Rc<RefCell<Vec<Vec<String>>>>) -> impl Fn(&Path, &[String]) -> Result<(), String> {
        let (root, version, calls) = (root.to_path_buf(), version.to_string(), Rc::clone(calls));
        move |exe, args| {
            let mut call = vec![exe.file_name().unwrap().to_string_lossy().to_string()];
            call.extend(args.iter().cloned());
            calls.borrow_mut().push(call);
            let app = installed::product(id).unwrap().app_dir_in(&root);
            let record = if id == "brain" { app.parent().unwrap().join("data").join("install-manifest.json") } else { app.join(".respected").join("install.json") };
            if exe.ends_with("unins000.exe") {
                let (app, record) = (app.clone(), record.clone());
                std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(300)); let _ = fs::remove_file(record); let _ = fs::remove_dir_all(app); });
                return Ok(());
            }
            assert!(exe.is_file(), "the staged installer exists while it runs");
            fs::create_dir_all(record.parent().unwrap()).unwrap();
            fs::create_dir_all(&app).unwrap();
            fs::write(app.join(if id == "brain" { "respected-brain.exe" } else { "respected-orchestrator.exe" }), b"exe").unwrap();
            fs::write(app.join("unins000.exe"), b"unins").unwrap();
            fs::write(app.join("distribution.json"), json!({ "version": version }).to_string()).unwrap();
            fs::write(&record, b"{}").unwrap();
            Ok(())
        }
    }

    fn ok(_: &Installed) -> Result<(), String> { Ok(()) }

    #[test]
    fn install_passes_the_persons_choices_as_installer_switches_then_update_and_uninstall_follow_the_product() {
        let roots = fixture("brain", "RespectedBrain-Windows-Setup.exe", "0.2.0");
        let root = temp("products");
        let calls = Rc::new(RefCell::new(Vec::new()));
        let run = fake_run(&root, "brain", "0.2.0", &calls);
        let params = json!({ "vault_path": "C:\\Notlar\\RespectedOS", "settings": { "assistant_name": "Jarvis", "integrations.mcp": true, "transcript_retention_days": 7 } });
        let applied = apply_in(&roots, &root, "brain", "install", &params, "op-install-1", &run, &ok).unwrap();
        assert_eq!((applied["status"].as_str(), applied["to_version"].as_str()), (Some("applied"), Some("0.2.0")));
        assert_eq!(calls.borrow()[0], ["RespectedBrain-Windows-Setup.exe", "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/VAULT=C:\\Notlar\\RespectedOS",
            "/SET=assistant_name=Jarvis", "/SET=integrations.mcp=true", "/SET=transcript_retention_days=7"]);
        assert!(!roots.managed.join("installer-staging").read_dir().map_or(false, |mut d| d.next().is_some()), "the staged copy is removed");
        assert_eq!(apply_in(&roots, &root, "brain", "install", &params, "op-install-1", &run, &ok).unwrap()["status"], "applied", "same id replays");
        assert_eq!(calls.borrow().len(), 1);
        assert_eq!(apply_in(&roots, &root, "brain", "install", &params, "op-install-2", &run, &ok).unwrap_err().code, "LIFECYCLE_CONFLICT", "already installed");
        assert_eq!(apply_in(&roots, &root, "brain", "update", &Value::Null, "op-update-1", &run, &ok).unwrap_err().code, "LIFECYCLE_CONFLICT", "same version is not an update");

        let removed = apply_in(&roots, &root, "brain", "uninstall", &json!({ "purge": true }), "op-remove-1", &run, &ok).unwrap();
        assert_eq!(removed["status"], "applied");
        assert_eq!(calls.borrow().last().unwrap()[1..], ["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/PURGE=1"]);
        assert!(installed::product("brain").unwrap().installed_in(&root).is_none());
    }

    #[test]
    fn the_uninstaller_is_the_newest_working_pair_after_an_interrupted_install() {
        let app = temp("unins");
        assert_eq!(uninstaller_in(&app), app.join("unins000.exe"), "nothing there: the usual name");
        fs::write(app.join("unins000.exe"), b"x").unwrap();
        fs::write(app.join("unins000.dat"), b"").unwrap(); // the install was killed while writing it
        fs::write(app.join("unins001.exe"), b"x").unwrap();
        fs::write(app.join("unins001.dat"), b"data").unwrap();
        assert_eq!(uninstaller_in(&app), app.join("unins001.exe"));
        fs::remove_file(app.join("unins001.dat")).unwrap();
        fs::write(app.join("unins001.dat"), b"").unwrap();
        assert_eq!(uninstaller_in(&app), app.join("unins000.exe"), "no pair is whole: fall back to the first name");
    }

    #[test]
    fn choices_that_could_reach_beyond_the_installer_switches_are_refused() {
        for params in [json!({ "vault_path": "relative" }), json!({ "settings": { "a b": "x" } }), json!({ "settings": { "user_bio": "say \"hi\"" } }),
                       json!({ "settings": { "x": { "nested": 1 } } }), json!({ "argv": ["/DIR=C:\\"] }), json!("text")] {
            assert_eq!(switches("brain", "install", &params).unwrap_err().code, "BAD_REQUEST", "{params}");
        }
        assert_eq!(switches("orchestrator", "install", &json!({ "vault_path": "C:\\x" })).unwrap_err().code, "BAD_REQUEST", "only Brain has a vault");
        assert_eq!(switches("brain", "update", &json!({ "settings": { "user_name": "x" } })).unwrap(), SILENT.map(String::from), "update keeps the settings");
    }

    #[test]
    fn a_failed_check_or_installer_is_reported_and_a_tampered_installer_never_runs() {
        let roots = fixture("orchestrator", "RespectedOrchestrator-Windows-Setup.exe", "0.2.0");
        let root = temp("products");
        let calls = Rc::new(RefCell::new(Vec::new()));
        let run = fake_run(&root, "orchestrator", "0.2.0", &calls);
        let bad = |_: &Installed| -> Result<(), String> { Err("kırık".into()) };
        assert_eq!(apply_in(&roots, &root, "orchestrator", "install", &Value::Null, "op-install-3", &run, &bad).unwrap_err().code, "VERIFY_FAILED");
        assert_eq!(lifecycle::read_journal(&roots, "op-install-3").unwrap()["status"], "failed");
        let refused = |_: &Path, _: &[String]| -> Result<(), String> { Err("çıkış 1".into()) };
        assert_eq!(apply_in(&roots, &temp("p2"), "orchestrator", "install", &Value::Null, "op-install-4", &refused, &ok).unwrap_err().code, "LIFECYCLE_PARTIAL");
        fs::write(roots.catalog.join("orchestrator-0.2.0").join("payload").join("RespectedOrchestrator-Windows-Setup.exe"), b"tampered!!!!!!").unwrap();
        let ran = RefCell::new(false);
        let spy = |_: &Path, _: &[String]| -> Result<(), String> { *ran.borrow_mut() = true; Ok(()) };
        assert!(apply_in(&roots, &temp("p3"), "orchestrator", "install", &Value::Null, "op-install-5", &spy, &ok).is_err());
        assert!(!*ran.borrow());
    }

    #[test]
    fn the_plan_names_the_installer_steps_and_refuses_what_cannot_apply() {
        let roots = fixture("orchestrator", "RespectedOrchestrator-Windows-Setup.exe", "0.2.0");
        let root = temp("products");
        let planned = plan_in(&roots, &root, "orchestrator", "install", &Value::Null).unwrap();
        assert_eq!((planned["to_version"].as_str(), planned["plan"]["steps"].as_array().unwrap().len()), (Some("0.2.0"), 3));
        assert_eq!(plan_in(&roots, &root, "orchestrator", "uninstall", &Value::Null).unwrap_err().code, "LIFECYCLE_CONFLICT");
        assert_eq!(plan_in(&roots, &root, "orchestrator", "repair", &Value::Null).unwrap_err().code, "LIFECYCLE_CONFLICT");
        assert_eq!(plan_in(&roots, &root, "orchestrator", "explode", &Value::Null).unwrap_err().code, "BAD_REQUEST");
        assert_eq!(plan_in(&roots, &root, "notes", "install", &Value::Null).unwrap_err().code, "BAD_REQUEST");
    }
}
