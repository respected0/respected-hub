//! Where the products are: the installed programs, or (development builds only) saved source checkouts. The Hub passes the
//! products no folders of its own: each product chooses its data folder itself.
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};
use tauri::Manager;
use crate::trust;

#[derive(Clone, Deserialize, Serialize)]
pub struct Settings {
    pub python: String,
    pub orchestrator_root: PathBuf,
    pub brain_root: PathBuf,
    /// The installed Brain program; absent means run the source with Python (development builds).
    #[serde(default)]
    pub brain_exe: Option<PathBuf>,
    /// The installed Orchestrator program; absent means the source checkout on Python (development builds).
    #[serde(default)]
    pub orchestrator_exe: Option<PathBuf>,
}

fn settings_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(|e| e.to_string())?.join("connections.json"))
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let file = settings_file(&app)?;
    // a saved connections.json only ever comes from a development build: a production Hub ignores it (source paths never run there)
    if trust::dev_enabled() && file.exists() {
        return serde_json::from_slice(&fs::read(file).map_err(|e| e.to_string())?).map_err(|e| e.to_string());
    }
    // Production: no source checkout - products are the installed programs only.
    let siblings = if trust::dev_enabled() {
        let hub = Path::new(env!("CARGO_MANIFEST_DIR")).parent().ok_or("Missing Hub root")?;
        hub.parent().ok_or("Missing product root")?.to_path_buf()
    } else {
        app.path().app_local_data_dir().map_err(|e| e.to_string())?.join("no-source-checkout")
    };
    Ok(Settings { python: "python".into(), orchestrator_root: siblings.join("respected-orchestrator"),
        brain_root: siblings.join("respected-brain"), brain_exe: None, orchestrator_exe: None })
}

fn validate(settings: &Settings) -> Result<(), String> {
    if settings.python.trim().is_empty() || !settings.orchestrator_root.is_absolute() {
        return Err("Explicit product paths and Python executable required".into());
    }
    if !settings.orchestrator_root.join("src/respected_orchestrator/__main__.py").is_file() {
        return Err("Independent Orchestrator source not found".into());
    }
    Ok(())
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    // These fields choose which programs the Hub starts (Python, source checkouts, Brain launcher): only a development build
    // lets them be edited, never a webview of an installed product.
    if !trust::dev_enabled() { return Err("Ürün kaynak bağlantıları yalnızca geliştirme derlemelerinde değiştirilir.".into()); }
    validate(&settings)?;
    let file = settings_file(&app)?;
    fs::create_dir_all(file.parent().ok_or("Invalid settings parent")?).map_err(|e| e.to_string())?;
    let temporary = file.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(temporary, file).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(python: &str, orchestrator_root: PathBuf) -> Settings {
        Settings { python: python.into(), orchestrator_root, brain_root: PathBuf::new(), brain_exe: None, orchestrator_exe: None }
    }

    #[test]
    fn validate_refuses_relative_paths_blank_python_and_a_missing_source() {
        let root = std::env::temp_dir().join(format!("hub-settings-{}", std::process::id()));
        let entry = root.join("src/respected_orchestrator");
        fs::create_dir_all(&entry).unwrap();
        assert!(validate(&settings("python", root.clone())).is_err(), "no __main__.py yet");
        fs::write(entry.join("__main__.py"), "").unwrap();
        assert!(validate(&settings("python", root.clone())).is_ok());
        assert!(validate(&settings("  ", root.clone())).is_err());
        assert!(validate(&settings("python", PathBuf::from("relative"))).is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}
