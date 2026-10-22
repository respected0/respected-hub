//! Product lifecycle, catalog, Hub update and activity commands.
use super::{lifecycle_reply, lifecycle_roots, resolved_settings};
use crate::hub::{hub_apply, hub_plan, hub_status};
use crate::lifecycle::catalog;
use crate::{lifecycle, orchestrator::mcp as orchestrator_mcp};
use serde_json::{json, Value};
use std::time::Duration;

#[tauri::command]
pub async fn lifecycle_status(app: tauri::AppHandle) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || hub_status(&roots, &settings)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lifecycle_plan(app: tauri::AppHandle, product: String, action: String, params: Option<Value>) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(hub_plan(&roots, &product, &action, &params.unwrap_or(Value::Null)))).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lifecycle_apply(app: tauri::AppHandle, product: String, action: String, operation_id: String, params: Option<Value>) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(hub_apply(&roots, settings, &product, &action, &params.unwrap_or(Value::Null), &operation_id))).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn catalog_refresh(app: tauri::AppHandle) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(catalog::refresh(&roots).map(|_| catalog::hub_update(&roots)))).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn catalog_set_channel(app: tauri::AppHandle, channel: String) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(catalog::set_channel(&roots, &channel).map(|_| catalog::hub_update(&roots)))).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hub_update_status(app: tauri::AppHandle) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || json!({ "ok": true, "result": catalog::hub_update(&roots), "error": Value::Null })).await.map_err(|e| e.to_string())
}

/// The person confirmed "install the Hub update": start the verified installer and close so it can replace this executable.
#[tauri::command]
pub async fn hub_update_install(app: tauri::AppHandle) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    let reply = tauri::async_runtime::spawn_blocking(move || lifecycle_reply(catalog::install_hub_update(&roots).map(|_| json!({ "started": true })))).await.map_err(|e| e.to_string())?;
    if reply["ok"] == true { std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(800)); app.exit(0); }); }
    Ok(reply)
}

#[tauri::command]
pub async fn orchestrator_mcp_plan(app: tauri::AppHandle, desired: Value) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(orchestrator_mcp::plan(&settings, &desired))).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn orchestrator_mcp_apply(app: tauri::AppHandle, desired: Value, plan_digest: String, operation_id: String) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || lifecycle_reply(orchestrator_mcp::apply(&settings, &desired, &plan_digest, &operation_id))).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn activity_list(app: tauri::AppHandle, limit: Option<usize>) -> Result<Value, String> {
    let roots = lifecycle_roots(&app)?;
    tauri::async_runtime::spawn_blocking(move || json!({ "ok": true, "result": lifecycle::activity(&roots, limit.unwrap_or(50).min(200)), "error": Value::Null })).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn startup_intent() -> Option<Value> { crate::hub::startup::intent(std::env::args()) }
