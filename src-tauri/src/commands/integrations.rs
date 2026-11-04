//! Narrow, typed IPC for third-party integrations. Writes stay out until the M3 gate.
use crate::integrations::api::{self, Request};
use crate::integrations::discovery::Environment;
use crate::integrations::model::Project;
use crate::integrations::projects;
use serde_json::Value;

#[tauri::command]
pub async fn integrations_request(request: Request) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || api::execute(request)).await.map_err(|error| error.to_string())
}

/// Hub-owned project registry. The dialog path arrives here and is validated
/// before it is stored; no AI client configuration is touched.
#[tauri::command]
pub fn projects_list() -> Result<Vec<Project>, String> {
    projects::load(&Environment::current())
}

#[tauri::command]
pub fn projects_add(root: String) -> Result<Vec<Project>, String> {
    projects::add(&Environment::current(), &root)
}

#[tauri::command]
pub fn projects_remove(id: String) -> Result<Vec<Project>, String> {
    projects::remove(&Environment::current(), &id)
}
