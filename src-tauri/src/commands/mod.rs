//! Tauri commands the window invokes by name, one file per area. Each resolves the Hub-managed roots and settings,
//! then hands the blocking work to `hub` or a product module.
use crate::hub::{overlay, standard_roots};
use crate::{get_settings, lifecycle, secondbrain, Settings};
use serde_json::{json, Value};

pub mod engine;
pub mod manage;
pub mod memory;

pub(crate) fn lifecycle_roots(_app: &tauri::AppHandle) -> Result<lifecycle::Roots, String> { Ok(standard_roots()) }

/// Saved connections with each Hub-managed product pointing at its active verified copy.
pub(crate) fn resolved_settings(app: &tauri::AppHandle) -> Result<Settings, String> {
    let mut settings = get_settings(app.clone())?;
    overlay(&lifecycle_roots(app)?, &secondbrain::dirs(), &mut settings);
    Ok(settings)
}

pub(crate) fn lifecycle_reply(result: Result<Value, lifecycle::LifecycleError>) -> Value {
    match result { Ok(value) => json!({ "ok": true, "result": value, "error": Value::Null }), Err(error) => json!({ "ok": false, "result": Value::Null, "error": error.to_json() }) }
}
