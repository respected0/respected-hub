//! Where the products are: saved connections (development builds only) and the data folders the Hub passes to them.
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};
use tauri::Manager;
use crate::{brain, trust};

#[derive(Clone, Deserialize, Serialize)]
pub struct Settings {
    pub python: String,
    pub orchestrator_root: PathBuf,
    pub brain_root: PathBuf,
    pub data_root: PathBuf,
    #[serde(default)]
    pub brain_data_root: Option<PathBuf>,
    /// Frozen SecondBrain launcher (installed or staged) that serves RPP; absent means run the source with Python.
    #[serde(default)]
    pub brain_exe: Option<PathBuf>,
    /// Application folder the launcher should treat as its install location (RESPECTED_APP_DIR).
    #[serde(default)]
    pub brain_app_dir: Option<PathBuf>,
    /// Packaged Orchestrator (the Hub-managed copy's respected-orchestrator.exe); absent means the source checkout on Python.
    #[serde(default)]
    pub orchestrator_exe: Option<PathBuf>,
}

fn settings_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(|e| e.to_string())?.join("connections.json"))
}

/// Default data_root for a fresh install. An explicit absolute path from
/// RESPECTED_ORCHESTRATOR_DATA_ROOT wins; a saved connections.json still wins
/// over this because get_settings returns it before reaching here. Pure, unit-tested.
pub(crate) fn default_data_root(env_value: Option<&str>, fallback: PathBuf) -> PathBuf {
    match env_value {
        Some(value) if !value.trim().is_empty() && Path::new(value).is_absolute() => PathBuf::from(value),
        _ => fallback,
    }
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let file = settings_file(&app)?;
    // a saved connections.json only ever comes from a development build: a production Hub ignores it (source paths never run there)
    if trust::dev_enabled() && file.exists() {
        return serde_json::from_slice(&fs::read(file).map_err(|e| e.to_string())?).map_err(|e| e.to_string());
    }
    let data_base = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    // Production: no source checkout and no development data folder - products come from the managed, verified copies only.
    let (siblings, data_root) = if trust::dev_enabled() {
        let hub = Path::new(env!("CARGO_MANIFEST_DIR")).parent().ok_or("Missing Hub root")?;
        (hub.parent().ok_or("Missing product root")?.to_path_buf(),
         default_data_root(trust::dev_env("RESPECTED_ORCHESTRATOR_DATA_ROOT").and_then(|v| v.into_string().ok()).as_deref(), data_base.with_file_name("RespectedOrchestratorDevelopment")))
    } else {
        (data_base.join("no-source-checkout"), brain::respected_root().join("RespectedOrchestrator").join("data"))
    };
    Ok(Settings { python: "python".into(), orchestrator_root: siblings.join("respected-orchestrator"),
        brain_root: siblings.join("respected-secondbrain"), data_root,
        brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None })
}

fn validate(settings: &Settings) -> Result<(), String> {
    if settings.python.trim().is_empty() || !settings.orchestrator_root.is_absolute() || !settings.data_root.is_absolute() {
        return Err("Explicit product/data paths and Python executable required".into());
    }
    if !settings.orchestrator_root.join("src/respected_orchestrator/__main__.py").is_file() {
        return Err("Independent Orchestrator source not found".into());
    }
    Ok(())
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    // These fields choose which programs the Hub starts (Python, source checkouts, SecondBrain launcher): only a development build
    // lets them be edited, never a webview of an installed product.
    if !trust::dev_enabled() { return Err("Ürün kaynak bağlantıları yalnızca geliştirme derlemelerinde değiştirilir.".into()); }
    validate(&settings)?;
    let file = settings_file(&app)?;
    fs::create_dir_all(file.parent().ok_or("Invalid settings parent")?).map_err(|e| e.to_string())?;
    let temporary = file.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(temporary, file).map_err(|e| e.to_string())
}
