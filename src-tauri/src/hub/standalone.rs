//! A product installed by its own installer, not by the Hub. The Hub only recognises it through the install record the product
//! wrote itself (`<app>\.respected\install.json`); the product owns its Installed-apps entry, updates and removal.
use crate::lifecycle;
use crate::{secondbrain, ORCHESTRATOR_EXE};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Standalone { pub version: String, pub app_dir: PathBuf, pub exe: PathBuf }

/// Where `RespectedOrchestrator-Windows-Setup.exe` puts the program.
pub fn orchestrator_dir() -> PathBuf { secondbrain::respected_root().join("RespectedOrchestrator").join("app") }

/// The Orchestrator installed by its own installer in its default place.
pub fn orchestrator() -> Option<Standalone> { at(&orchestrator_dir()) }

pub(crate) fn at(app_dir: &Path) -> Option<Standalone> {
    let doc: Value = serde_json::from_slice(&fs::read(app_dir.join(".respected").join("install.json")).ok()?).ok()?;
    if doc["product_id"] != "respected-orchestrator" || doc["distribution"] != true { return None; }
    let version = doc["product_version"].as_str().filter(|v| lifecycle::valid_version(v))?.to_string();
    let exe = app_dir.join(ORCHESTRATOR_EXE);
    exe.is_file().then(|| Standalone { version, app_dir: app_dir.to_path_buf(), exe })
}

impl Standalone {
    /// Same shape as `lifecycle::status`, without update offers: the product's own installer updates it.
    pub fn status(&self) -> Value {
        json!({ "product": "orchestrator", "installed": true, "install_state": "installed", "version": self.version, "source": "standalone",
            "signed": Value::Null, "problems": [], "available": Value::Null, "previous_versions": [], "catalog_skipped": [],
            "removed_available": false, "root": self.app_dir })
    }
}

/// SecondBrain installed by `RespectedSecondBrain-Windows-Setup.exe`: its uninstaller sits beside the program folder
/// (`RespectedBrain/installer/unins000.exe`). The product still runs through its own engine, so the Hub keeps updating
/// and repairing it; only the Installed-apps entry and the removal belong to that installer.
pub fn brain_has_installer(d: &secondbrain::BrainDirs) -> bool {
    d.app.parent().map_or(false, |root| root.join("installer").join("unins000.exe").is_file())
}

/// Why the Hub will not install, update or remove a product that has its own installer.
pub fn conflict() -> lifecycle::LifecycleError {
    lifecycle::LifecycleError { code: "LIFECYCLE_CONFLICT", message: "Ürün kendi kurucusuyla kurulmuş.".into(),
        hint: "Güncellemek veya kaldırmak için Windows'taki Yüklü uygulamalar listesini kullanın.".into() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(label: &str, record: Option<&str>, exe: bool) -> PathBuf {
        let p = std::env::temp_dir().join(format!("hub-standalone-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(p.join(".respected")).unwrap();
        if let Some(record) = record { fs::write(p.join(".respected").join("install.json"), record).unwrap(); }
        if exe { fs::write(p.join(ORCHESTRATOR_EXE), b"exe").unwrap(); }
        p
    }

    const RECORD: &str = r#"{"product_id":"respected-orchestrator","product_version":"0.1.0-beta.1","distribution":true,"files":{}}"#;

    #[test]
    fn a_complete_record_and_executable_are_recognised() {
        let found = at(&dir("ok", Some(RECORD), true)).unwrap();
        assert_eq!(found.version, "0.1.0-beta.1");
        assert_eq!(found.status()["source"], "standalone");
        assert_eq!(found.status()["install_state"], "installed");
    }

    #[test]
    fn a_secondbrain_uninstaller_beside_the_program_folder_marks_its_own_installer() {
        let base = std::env::temp_dir().join(format!("hub-brain-installer-{}", std::process::id()));
        let dirs = secondbrain::BrainDirs { app: base.join("app"), data: base.join("data") };
        assert!(!brain_has_installer(&dirs));
        fs::create_dir_all(base.join("installer")).unwrap();
        fs::write(base.join("installer").join("unins000.exe"), b"x").unwrap();
        assert!(brain_has_installer(&dirs));
    }

    #[test]
    fn anything_missing_or_foreign_is_not_standalone() {
        assert!(at(&dir("no-record", None, true)).is_none());
        assert!(at(&dir("no-exe", Some(RECORD), false)).is_none());
        assert!(at(&dir("other", Some(&RECORD.replace("respected-orchestrator", "respected-secondbrain")), true)).is_none());
        assert!(at(&dir("not-packaged", Some(&RECORD.replace("\"distribution\":true", "\"distribution\":false")), true)).is_none());
        assert!(at(&dir("bad-version", Some(&RECORD.replace("0.1.0-beta.1", "../x")), true)).is_none());
    }
}
