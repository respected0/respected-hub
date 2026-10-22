//! Shared by the end-to-end tests: disposable folders and the ecosystem root holding the three repositories.
#![allow(dead_code)]
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A temporary folder that disappears when dropped.
pub struct Temp(pub PathBuf);

impl Temp {
    pub fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let path = std::env::temp_dir().join(format!("hub-e2e-{label}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Temp(path)
    }
}

impl Drop for Temp {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
}

pub fn ecosystem() -> PathBuf {
    std::env::var_os("RESPECTED_ECOSYSTEM_ROOT").map(PathBuf::from).unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(".."))
}

/// Point the products this test starts at `data` (and a program folder beside it) through RESPECTED_DATA_DIR / RESPECTED_APP_DIR,
/// the one override every product shares. The variables are process-wide, so tests that need their own folders hold this lock.
pub fn product_folders(data: &Path) -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("RESPECTED_DATA_DIR", data);
    std::env::set_var("RESPECTED_APP_DIR", data.with_extension("app"));
    guard
}
