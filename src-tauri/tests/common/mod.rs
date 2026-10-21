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
