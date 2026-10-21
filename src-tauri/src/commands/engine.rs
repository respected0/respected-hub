//! Engine, generic RPP and component-availability commands.
use super::resolved_settings;
use crate::{call_engine, list_campaigns, rpp, trust};
use serde_json::{json, Value};

#[tauri::command]
pub async fn engine_request(app: tauri::AppHandle, action: String, run_id: Option<String>, request: Option<Value>) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || call_engine(settings, &action, run_id, request)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn campaign_list(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || list_campaigns(&settings)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn rpp_request(app: tauri::AppHandle, product: String, op: String, payload: Option<Value>, operation_id: Option<String>) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || rpp::rpp_call(&settings, &product, &op, payload, operation_id)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn product_status(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    let brain = settings.brain_exe.as_ref().map_or(false, |exe| exe.is_file()) || (trust::dev_enabled() && settings.brain_root.join("src/respectedbrain/__init__.py").is_file());
    let orchestrator = settings.orchestrator_exe.as_ref().map_or(false, |exe| exe.is_file()) || (trust::dev_enabled() && settings.orchestrator_root.join("src/respected_orchestrator/__main__.py").is_file());
    let cli = rpp::orchestrator_cli_line(&settings);
    let engine = tauri::async_runtime::spawn_blocking(move || call_engine(settings, "status", None, None)).await.map_err(|e| e.to_string())?;
    Ok(json!({ "development": trust::dev_enabled(), "brain_available": brain, "orchestrator_available": orchestrator, "orchestrator_cli": cli,
        "hub_version": env!("CARGO_PKG_VERSION"), "engine": engine.unwrap_or_else(|error| json!({"error": error})) }))
}
