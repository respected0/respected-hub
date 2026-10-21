//! A product installed, updated, repaired and removed through its own signed installer (`RespectedOrchestrator-Windows-Setup.exe`).
//!
//! The Hub downloads the installer as a catalog bundle of kind `installer`, checks the bundle signature and the file hash exactly as it
//! does for a distribution, then runs it silently. The installer owns the program folder, the Installed-apps entry and the
//! uninstaller, so the product ends up identical whether the person ran the installer or the Hub did. There is no undo copy:
//! the previous version is replaced in place, and a removal runs the product's own uninstaller.
use super::standalone::{self, Standalone};
use crate::lifecycle::{self, Bundle, LifecycleError, Roots};
use crate::orchestrator::mcp;
use crate::{engine_verifier, Settings};
use serde_json::{json, Value};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const PRODUCT: &str = "orchestrator";
const TIMEOUT: Duration = Duration::from_secs(600);
const SILENT: [&str; 3] = ["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"];

type Runner<'a> = &'a dyn Fn(&Path, &[&str]) -> Result<(), String>;
type Checker<'a> = &'a dyn Fn(&Standalone) -> Result<(), String>;

fn err(code: &'static str, message: impl Into<String>, hint: impl Into<String>) -> LifecycleError { LifecycleError { code, message: message.into(), hint: hint.into() } }

/// The newest installer bundle in the catalog cache.
fn newest(roots: &Roots) -> Option<Bundle> { lifecycle::catalog_of(roots, PRODUCT, "installer").0.into_iter().next() }

/// Whether the installer route handles `action` for `product`. A product the installer put there is removed by its uninstaller (no
/// catalog needed); install, update and repair run the catalog's installer bundle, which also moves a Hub-managed copy of an older
/// Hub over to the installer-owned layout. Removing a Hub-managed copy stays with the Hub's own lifecycle.
pub fn applies(roots: &Roots, product: &str, action: &str) -> bool {
    if product != PRODUCT { return false; }
    let managed = lifecycle::read_current(roots, product).is_some();
    if action == "uninstall" { return !managed && standalone::orchestrator().is_some(); }
    newest(roots).is_some() && (!managed || action != "install")
}

/// The Hub-managed copy of an older Hub, once the installer has put the product in its own place: its files go, its MCP launcher stays.
fn retire_managed_copy(roots: &Roots) {
    let dir = roots.managed.join(PRODUCT);
    for name in ["versions", "removed"] { let _ = std::fs::remove_dir_all(dir.join(name)); }
    let _ = std::fs::remove_file(dir.join("current.json"));
}

pub fn plan(roots: &Roots, action: &str) -> Result<Value, LifecycleError> { plan_at(roots, action, &standalone::orchestrator_dir()) }

fn plan_at(roots: &Roots, action: &str, app_dir: &Path) -> Result<Value, LifecycleError> {
    let installed = standalone::at(app_dir).map(|s| s.version).or_else(|| lifecycle::read_current(roots, PRODUCT).map(|c| c.0));
    let available = newest(roots);
    let mut steps = Vec::new();
    let step = |id: &str, summary: &str, reversible: bool, reason: Option<&str>| json!({ "id": id, "summary": summary, "reversible": reversible, "irreversible_reason": reason });
    match action {
        "install" | "update" | "repair" => {
            let Some(bundle) = &available else { return Err(err("BAD_REQUEST", "Katalogda bu ürünün kurucusu yok.", "Ürün kataloğunu yenileyin (Ayarlar › Güncellemeleri denetle).")) };
            match (action, &installed) {
                ("install", Some(_)) => return Err(err("LIFECYCLE_CONFLICT", "Ürün zaten kurulu.", "Güncelleme veya onarım kullanın.")),
                ("update" | "repair", None) => return Err(err("LIFECYCLE_CONFLICT", "Ürün kurulu değil.", "Önce kurun.")),
                ("update", Some(current)) if lifecycle::version_key(&bundle.version) <= lifecycle::version_key(current) => return Err(err("LIFECYCLE_CONFLICT", "Güncelleme yok; kurulu sürüm güncel.", "Sorun varsa onarımı kullanın.")),
                _ => {}
            }
            steps.push(step("stage", "Ürünün kurucusunu güvenli bir ara klasöre kopyala ve imzalı paket özetiyle doğrula", true, None));
            steps.push(step("install", "Kurucuyu sessiz modda çalıştır; program klasörünü, Yüklü uygulamalar girdisini ve kaldırıcıyı kurucu yazar", action == "install", if action == "install" { None } else { Some("Önceki sürüm yerinde değiştirilir; geri alma kopyası tutulmaz") }));
            steps.push(step("verify", "Ürünün kendi RPP yanıtıyla (manifest + sağlık) kurulumu doğrula", true, None));
        }
        "uninstall" => {
            if installed.is_none() { return Err(err("LIFECYCLE_CONFLICT", "Ürün kurulu değil.", "Kaldırılacak bir şey yok.")); }
            steps.push(step("remove", "Ürünün kendi kaldırıcısını çalıştır; program dosyaları ve Yüklü uygulamalar girdisi kalkar", false, Some("Kaldırma geri alınamaz; yeniden kurmak gerekir")));
            steps.push(step("keep-notes", "Not kasanız ve notlarınız silinmez", true, None));
        }
        _ => return Err(err("BAD_REQUEST", format!("Bilinmeyen eylem: {action}."), "install, update, repair veya uninstall kullanın.")),
    }
    Ok(json!({ "product": PRODUCT, "action": action, "dry_run": true, "from_version": installed,
        "to_version": available.map(|b| b.version), "plan": { "steps": steps } }))
}

pub fn apply(roots: &Roots, settings: &Settings, action: &str, id: &str) -> Result<Value, LifecycleError> {
    let verify = |installed: &Standalone| engine_verifier(settings)(PRODUCT, &installed.app_dir);
    apply_with(roots, settings, action, id, &standalone::orchestrator_dir(), &run_silently, &verify)
}

fn apply_with(roots: &Roots, settings: &Settings, action: &str, id: &str, app_dir: &Path, run: Runner, verify: Checker) -> Result<Value, LifecycleError> {
    if !lifecycle::valid_operation_id(id) { return Err(err("BAD_REQUEST", "İşlem kimliği geçersiz.", "8-64 karakter; harf, rakam, _ veya -.")); }
    if let Some(stored) = lifecycle::read_journal(roots, id) {
        if stored["product"] == PRODUCT && stored["action"] == action { return Ok(stored); }
        return Err(err("LIFECYCLE_CONFLICT", "Bu işlem kimliği başka bir eylem için kullanılmış.", "Yeni bir işlem kimliği kullanın."));
    }
    if !lifecycle::is_idle(roots, PRODUCT) { return Err(err("LIFECYCLE_CONFLICT", "Bu ürün için başka bir işlem sürüyor.", "Bitmesini bekleyin.")); }
    let planned = plan_at(roots, action, app_dir)?;
    let mut entry = lifecycle::begin(roots, id, PRODUCT, action, planned["from_version"].clone(), planned["to_version"].clone())?;
    match run_action(roots, settings, action, id, app_dir, &mut entry, run, verify) {
        Ok(()) => lifecycle::finish(roots, entry, "applied", None),
        Err(error) => { let _ = lifecycle::finish(roots, entry, "failed", Some(&error)); Err(error) }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_action(roots: &Roots, settings: &Settings, action: &str, id: &str, app_dir: &Path, entry: &mut Value, run: Runner, verify: Checker) -> Result<(), LifecycleError> {
    if action == "uninstall" {
        mcp::disable_all(roots, settings, id);
        let uninstaller = app_dir.join("unins000.exe");
        if !uninstaller.is_file() { return Err(err("VERIFY_FAILED", "Ürünün kaldırıcısı bulunamadı.", "Windows'taki Yüklü uygulamalar listesinden kaldırın.")); }
        lifecycle::set_status(roots, entry, "switching")?;
        run(&uninstaller, &SILENT).map_err(|m| err("LIFECYCLE_PARTIAL", format!("Kaldırıcı tamamlanamadı: {m}"), "Ürün çalışıyorsa kapatıp yeniden deneyin."))?;
        // the uninstaller's second stage may still be deleting files when the first returns
        for _ in 0..60 {
            if standalone::at(app_dir).is_none() { return Ok(()); }
            std::thread::sleep(Duration::from_secs(1));
        }
        return Err(err("VERIFY_FAILED", "Kaldırma sonrası ürün hâlâ kurulu görünüyor.", "Kaldırıcı günlüğüne bakın."));
    }
    let Some(bundle) = newest(roots) else { return Err(err("BAD_REQUEST", "Katalogda bu ürünün kurucusu yok.", "Ürün kataloğunu yenileyin.")) };
    let staging = roots.managed.join("installer-staging").join(lifecycle::stamp().to_string());
    let outcome = (|| {
        lifecycle::stage_bundle(&bundle, &staging)?;
        let problems = lifecycle::tree_problems(&staging, &bundle);
        if !problems.is_empty() { return Err(err("VERIFY_FAILED", format!("Kurucu doğrulanamadı: {}", problems.join("; ")), "Yeniden deneyin.")); }
        lifecycle::set_status(roots, entry, "switching")?;
        run(&staging.join(&bundle.files[0].0), &SILENT).map_err(|m| err("LIFECYCLE_PARTIAL", format!("Kurucu tamamlanamadı: {m}"), "Ürün çalışıyorsa kapatıp yeniden deneyin."))?;
        lifecycle::set_status(roots, entry, "verifying")?;
        let installed = standalone::at(app_dir).ok_or_else(|| err("VERIFY_FAILED", "Kurulum sonrası ürün kaydı bulunamadı.", "Kurucu günlüğüne bakın."))?;
        if lifecycle::version_key(&installed.version) != lifecycle::version_key(&bundle.version) { return Err(err("VERIFY_FAILED", format!("Kurulu sürüm {} beklenen {} değil.", installed.version, bundle.version), "Yeniden deneyin.")); }
        verify(&installed).map_err(|m| err("VERIFY_FAILED", format!("Ürün kurulumdan sonra çalışmadı: {m}"), "Ürünü onarın."))
    })();
    let _ = std::fs::remove_dir_all(&staging);
    outcome?;
    retire_managed_copy(roots);
    mcp::refresh_launcher(roots, settings);
    entry["to_version"] = json!(bundle.version);
    Ok(())
}

/// SecondBrain installed by its own installer is removed by that installer's uninstaller (it runs the product's own `uninstall`).
pub fn plan_brain_removal() -> Value {
    json!({ "product": "secondbrain", "action": "uninstall", "dry_run": true, "from_version": Value::Null, "to_version": Value::Null, "plan": { "steps": [
        { "id": "remove", "summary": "SecondBrain'in kendi kaldırıcısını çalıştır; program dosyaları ve Yüklü uygulamalar girdisi kalkar", "reversible": false, "irreversible_reason": "Kaldırma geri alınamaz; yeniden kurmak gerekir" },
        { "id": "keep-notes", "summary": "Not kasanız ve notlarınız silinmez", "reversible": true, "irreversible_reason": Value::Null }] } })
}

pub fn remove_brain(roots: &Roots, dirs: &crate::secondbrain::BrainDirs, id: &str) -> Result<Value, LifecycleError> {
    let uninstaller = dirs.app.parent().map(|root| root.join("installer").join("unins000.exe")).unwrap_or_default();
    remove_brain_with(roots, id, &uninstaller, &run_silently, &|| crate::secondbrain::status(roots, dirs)["installed"] != true, 60)
}

/// The uninstaller returns before the product has finished (the running program removes itself after it exits): wait for it to be gone.
fn remove_brain_with(roots: &Roots, id: &str, uninstaller: &Path, run: Runner, gone: &dyn Fn() -> bool, patience_seconds: u64) -> Result<Value, LifecycleError> {
    if !lifecycle::valid_operation_id(id) { return Err(err("BAD_REQUEST", "İşlem kimliği geçersiz.", "8-64 karakter; harf, rakam, _ veya -.")); }
    if let Some(stored) = lifecycle::read_journal(roots, id) { return if stored["product"] == "secondbrain" && stored["action"] == "uninstall" { Ok(stored) } else { Err(err("LIFECYCLE_CONFLICT", "Bu işlem kimliği başka bir eylem için kullanılmış.", "Yeni bir işlem kimliği kullanın.")) }; }
    if !uninstaller.is_file() { return Err(err("VERIFY_FAILED", "SecondBrain kaldırıcısı bulunamadı.", "Windows'taki Yüklü uygulamalar listesinden kaldırın.")); }
    let mut entry = lifecycle::begin(roots, id, "secondbrain", "uninstall", Value::Null, Value::Null)?;
    let outcome = (|| {
        lifecycle::set_status(roots, &mut entry, "switching")?;
        run(uninstaller, &SILENT).map_err(|m| err("LIFECYCLE_PARTIAL", format!("Kaldırıcı tamamlanamadı: {m}"), "SecondBrain çalışıyorsa kapatıp yeniden deneyin."))?;
        for _ in 0..=patience_seconds {
            if gone() { return Ok(()); }
            std::thread::sleep(Duration::from_secs(1));
        }
        Err(err("VERIFY_FAILED", "Kaldırma sonrası ürün hâlâ kurulu görünüyor.", "Kaldırıcı günlüğüne bakın."))
    })();
    match outcome { Ok(()) => lifecycle::finish(roots, entry, "applied", None), Err(e) => { let _ = lifecycle::finish(roots, entry, "failed", Some(&e)); Err(e) } }
}

/// Run an installer or uninstaller to its end. No pipes: Inno's second-stage processes outlive the first and would hold them.
fn run_silently(exe: &Path, args: &[&str]) -> Result<(), String> {
    let mut command = Command::new(exe);
    command.args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
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
    use std::rc::Rc;
    use std::fs;
    use std::path::PathBuf;

    const SETUP: &str = "RespectedOrchestrator-Windows-Setup.exe";

    fn temp(label: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("hub-installer-{label}-{}-{}", std::process::id(), lifecycle::stamp()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn settings() -> Settings {
        Settings { python: "python".into(), orchestrator_root: PathBuf::from("x"), brain_root: PathBuf::from("y"), data_root: temp("data"),
            brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None }
    }

    /// A catalog with one hand-made, unsigned (development opt-in) installer bundle.
    fn fixture(version: &str) -> (Roots, PathBuf) {
        let roots = Roots { managed: temp("managed"), catalog: temp("catalog") };
        let dir = roots.catalog.join(format!("orchestrator-{version}"));
        fs::create_dir_all(dir.join("payload")).unwrap();
        fs::write(dir.join("payload").join(SETUP), b"fake installer").unwrap();
        let file = json!({ "path": SETUP, "size": 14, "sha256": lifecycle::sha256_file(&dir.join("payload").join(SETUP)).unwrap() });
        let canonical = serde_json::to_string(&json!([file.clone()])).unwrap();
        let digest: String = Sha256::digest(canonical.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
        std::env::set_var("RESPECTED_HUB_ALLOW_UNSIGNED", "1");
        fs::write(dir.join("bundle.json"), serde_json::to_vec(&json!({ "schema": 1, "product": "orchestrator", "kind": "installer", "version": version,
            "files": [file], "bundle_digest": digest, "signature": Value::Null })).unwrap()).unwrap();
        (roots, temp("app"))
    }

    /// Stands in for Inno: the installer writes the program and its record, the uninstaller removes the record.
    fn fake_run(app_dir: &Path, version: &str, calls: &Rc<RefCell<Vec<String>>>) -> impl Fn(&Path, &[&str]) -> Result<(), String> {
        let (app_dir, version) = (app_dir.to_path_buf(), version.to_string());
        let calls = Rc::clone(calls);
        move |exe, args| {
            assert_eq!(args, SILENT);
            let name = exe.file_name().unwrap().to_string_lossy().to_string();
            calls.borrow_mut().push(name.clone());
            if name == "unins000.exe" { let _ = fs::remove_dir_all(app_dir.join(".respected")); return Ok(()); }
            assert!(exe.is_file(), "the staged installer exists while it runs");
            fs::create_dir_all(app_dir.join(".respected")).unwrap();
            fs::write(app_dir.join("respected-orchestrator.exe"), b"exe").unwrap();
            fs::write(app_dir.join("unins000.exe"), b"unins").unwrap();
            fs::write(app_dir.join(".respected").join("install.json"), json!({ "product_id": "respected-orchestrator", "product_version": version, "distribution": true }).to_string()).unwrap();
            Ok(())
        }
    }

    fn ok(_: &Standalone) -> Result<(), String> { Ok(()) }

    #[test]
    fn install_runs_the_staged_installer_then_update_and_uninstall_follow_the_product_record() {
        let (roots, app) = fixture("0.2.0");
        let calls = Rc::new(RefCell::new(Vec::new()));
        let run = fake_run(&app, "0.2.0", &calls);
        let applied = apply_with(&roots, &settings(), "install", "op-install-1", &app, &run, &ok).unwrap();
        assert_eq!((applied["status"].as_str(), applied["to_version"].as_str()), (Some("applied"), Some("0.2.0")));
        assert_eq!(standalone::at(&app).unwrap().version, "0.2.0");
        assert_eq!(*calls.borrow(), vec![SETUP.to_string()]);
        assert!(!roots.managed.join("installer-staging").read_dir().map_or(false, |mut d| d.next().is_some()), "the staged copy is removed");

        assert_eq!(apply_with(&roots, &settings(), "install", "op-install-1", &app, &run, &ok).unwrap()["status"], "applied", "same id replays");
        assert_eq!(calls.borrow().len(), 1);
        assert_eq!(apply_with(&roots, &settings(), "install", "op-install-2", &app, &run, &ok).unwrap_err().code, "LIFECYCLE_CONFLICT", "already installed");
        assert_eq!(apply_with(&roots, &settings(), "update", "op-update-1", &app, &run, &ok).unwrap_err().code, "LIFECYCLE_CONFLICT", "same version is not an update");

        let removed = apply_with(&roots, &settings(), "uninstall", "op-remove-1", &app, &run, &ok).unwrap();
        assert_eq!(removed["status"], "applied");
        assert!(standalone::at(&app).is_none());
        assert_eq!(calls.borrow().last().unwrap(), "unins000.exe");
    }

    #[test]
    fn an_update_moves_a_hub_managed_copy_of_an_older_hub_to_the_installer_layout() {
        let (roots, app) = fixture("0.2.0");
        let product = roots.managed.join("orchestrator");
        fs::create_dir_all(product.join("versions").join("0.1.0")).unwrap();
        fs::create_dir_all(product.join("mcp")).unwrap();
        fs::write(product.join("current.json"), br#"{"version":"0.1.0","bundle_digest":""}"#).unwrap();
        assert!(applies(&roots, "orchestrator", "update") && !applies(&roots, "orchestrator", "install") && !applies(&roots, "orchestrator", "uninstall"));
        let calls = Rc::new(RefCell::new(Vec::new()));
        let run = fake_run(&app, "0.2.0", &calls);
        let applied = apply_with(&roots, &settings(), "update", "op-migrate-1", &app, &run, &ok).unwrap();
        assert_eq!((applied["status"].as_str(), applied["from_version"].as_str()), (Some("applied"), Some("0.1.0")));
        assert_eq!(standalone::at(&app).unwrap().version, "0.2.0");
        assert!(lifecycle::read_current(&roots, "orchestrator").is_none() && !product.join("versions").exists());
        assert!(product.join("mcp").is_dir(), "the MCP launcher folder stays: AI clients point at it");
    }

    #[test]
    fn a_failed_verification_or_installer_is_reported_and_leaves_a_failed_journal_entry() {
        let (roots, app) = fixture("0.2.0");
        let calls = Rc::new(RefCell::new(Vec::new()));
        let run = fake_run(&app, "0.2.0", &calls);
        let bad = |_: &Standalone| -> Result<(), String> { Err("kırık".into()) };
        let error = apply_with(&roots, &settings(), "install", "op-install-3", &app, &run, &bad).unwrap_err();
        assert_eq!(error.code, "VERIFY_FAILED");
        assert_eq!(lifecycle::read_journal(&roots, "op-install-3").unwrap()["status"], "failed");
        let refused = |_: &Path, _: &[&str]| -> Result<(), String> { Err("çıkış 1".into()) };
        assert_eq!(apply_with(&roots, &settings(), "install", "op-install-4", &temp("app2"), &refused, &ok).unwrap_err().code, "LIFECYCLE_PARTIAL");
    }

    #[test]
    fn a_tampered_catalog_installer_never_runs() {
        let (roots, app) = fixture("0.2.0");
        fs::write(roots.catalog.join("orchestrator-0.2.0").join("payload").join(SETUP), b"tampered!!!!!!").unwrap();
        let ran = RefCell::new(false);
        let run = |_: &Path, _: &[&str]| -> Result<(), String> { *ran.borrow_mut() = true; Ok(()) };
        assert!(apply_with(&roots, &settings(), "install", "op-install-5", &app, &run, &ok).is_err());
        assert!(!*ran.borrow());
    }

    #[test]
    fn a_secondbrain_removal_runs_its_uninstaller_and_waits_for_the_product_to_be_gone() {
        let roots = Roots { managed: temp("brain-m"), catalog: temp("brain-c") };
        let uninstaller = temp("brain-u").join("unins000.exe");
        fs::write(&uninstaller, b"x").unwrap();
        let ran = RefCell::new(0);
        let run = |_: &Path, args: &[&str]| -> Result<(), String> { assert_eq!(args, SILENT); *ran.borrow_mut() += 1; Ok(()) };
        let polls = RefCell::new(0);
        let gone = || { *polls.borrow_mut() += 1; *polls.borrow() >= 2 };
        assert_eq!(remove_brain_with(&roots, "op-brain-1", &uninstaller, &run, &gone, 5).unwrap()["status"], "applied");
        assert_eq!((*ran.borrow(), *polls.borrow()), (1, 2));
        assert_eq!(remove_brain_with(&roots, "op-brain-1", &uninstaller, &run, &gone, 5).unwrap()["status"], "applied", "same id replays");
        assert_eq!(*ran.borrow(), 1);
        let never = || false;
        assert_eq!(remove_brain_with(&roots, "op-brain-2", &uninstaller, &run, &never, 0).unwrap_err().code, "VERIFY_FAILED");
        assert_eq!(remove_brain_with(&roots, "op-brain-3", &temp("none").join("unins000.exe"), &run, &gone, 0).unwrap_err().code, "VERIFY_FAILED");
    }

    #[test]
    fn the_plan_names_the_installer_steps_and_refuses_what_cannot_apply() {
        let (roots, app) = fixture("0.2.0");
        let planned = plan_at(&roots, "install", &app).unwrap();
        assert_eq!(planned["to_version"], "0.2.0");
        assert_eq!(planned["plan"]["steps"].as_array().unwrap().len(), 3);
        assert_eq!(plan_at(&roots, "uninstall", &app).unwrap_err().code, "LIFECYCLE_CONFLICT");
        assert_eq!(plan_at(&roots, "repair", &app).unwrap_err().code, "LIFECYCLE_CONFLICT");
        assert_eq!(plan_at(&roots, "explode", &app).unwrap_err().code, "BAD_REQUEST");
    }
}
