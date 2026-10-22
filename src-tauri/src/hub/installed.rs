//! What is installed, read from the products themselves: each product's installer puts it at
//! `%LOCALAPPDATA%\Respected\<Folder>\app`, its build writes `distribution.json` (with the version) there, and the product's own
//! `setup` leaves its install record. The Hub keeps no list of its own, so a product installed without the Hub is found the same way.
use std::fs;
use std::path::{Path, PathBuf};

pub struct Product {
    pub id: &'static str,
    pub name: &'static str,
    folder: &'static str,
    exe: &'static str,
    /// the product's own install record, relative to its folder (written by its `setup`, removed by its `uninstall`)
    record: &'static str,
}

pub const PRODUCTS: [Product; 2] = [
    Product { id: "orchestrator", name: "Orchestrator", folder: "RespectedOrchestrator", exe: "respected-orchestrator.exe", record: "app/.respected/install.json" },
    Product { id: "brain", name: "Brain", folder: "RespectedBrain", exe: "respected-brain.exe", record: "data/install-manifest.json" },
];

pub fn product(id: &str) -> Option<&'static Product> { PRODUCTS.iter().find(|p| p.id == id) }

pub fn respected_root() -> PathBuf {
    if let Some(root) = crate::trust::dev_abs("RESPECTED_HUB_PRODUCTS_ROOT") { return root; } // development tests only
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("Respected")
}

pub struct Installed { pub version: String, pub app_dir: PathBuf, pub exe: PathBuf }

impl Product {
    pub fn folder(&self) -> PathBuf { respected_root().join(self.folder) }
    pub fn app_dir(&self) -> PathBuf { self.folder().join("app") }
    pub fn exe(&self) -> PathBuf { self.app_dir().join(self.exe) }
    pub fn uninstaller(&self) -> PathBuf { self.app_dir().join("unins000.exe") }
    pub fn installed(&self) -> Option<Installed> { self.installed_in(&respected_root()) }
    pub(crate) fn app_dir_in(&self, root: &Path) -> PathBuf { root.join(self.folder).join("app") }

    /// Installed = program, build manifest and the product's own install record are all there.
    pub(crate) fn installed_in(&self, root: &Path) -> Option<Installed> {
        let folder = root.join(self.folder);
        let (app_dir, exe) = (folder.join("app"), folder.join("app").join(self.exe));
        if !exe.is_file() || !folder.join(self.record).is_file() { return None; }
        let document: serde_json::Value = serde_json::from_slice(&fs::read(app_dir.join("distribution.json")).ok()?).ok()?;
        let version = document["version"].as_str().filter(|v| crate::lifecycle::valid_version(v))?.to_string();
        Some(Installed { version, app_dir, exe })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_product_counts_as_installed_only_with_program_build_manifest_and_its_own_record() {
        let root = std::env::temp_dir().join(format!("hub-installed-{}-{}", std::process::id(), crate::lifecycle::stamp()));
        let brain = product("brain").unwrap();
        let at = |p: &str| root.join("RespectedBrain").join(p);
        assert!(brain.installed_in(&root).is_none());
        fs::create_dir_all(at("app")).unwrap();
        fs::create_dir_all(at("data")).unwrap();
        fs::write(at("app/respected-brain.exe"), b"exe").unwrap();
        fs::write(at("app/distribution.json"), br#"{"version": "0.1.0b1"}"#).unwrap();
        assert!(brain.installed_in(&root).is_none(), "no install record: the product's own setup never finished");
        fs::write(at("data/install-manifest.json"), b"{}").unwrap();
        assert_eq!(brain.installed_in(&root).unwrap().version, "0.1.0b1");
        let _ = fs::remove_dir_all(root);
    }
}
