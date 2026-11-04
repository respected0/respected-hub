//! Narrow, typed IPC for third-party integrations. Writes stay out until the M3 gate.
use crate::integrations::api::{self, Request};
use serde_json::Value;

#[tauri::command]
pub fn integrations_request(request: Request) -> Value {
    api::execute(request)
}
