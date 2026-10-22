//! SecondBrain memory commands: access grant, index, semantic search, capture and the workspace calls.
use super::resolved_settings;
use crate::secondbrain::{memory_index, memory_semantic};
use crate::{rpp, Settings};
use serde_json::{json, Value};
use std::time::Duration;

fn memory_reply(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => json!({ "ok": true, "result": value, "error": Value::Null }),
        Err(message) => json!({ "ok": false, "result": Value::Null,
            "error": { "code": "MEMORY_ACCESS", "message": message, "hint": "SecondBrain ve Orchestrator'ın kurulu ve güncel olduğunu denetleyin." } }),
    }
}

/// One Orchestrator memory operation (`memory.connect|disconnect|status`): the Orchestrator asks SecondBrain for the grant
/// itself, exactly as `respected-orchestrator memory connect` does without the Hub.
fn memory_link(settings: &Settings, op: &str, payload: Option<Value>) -> Value {
    let reply = rpp::rpp_call_trusted(settings, "orchestrator", op, payload, Some(rpp::generate_operation_id()));
    if reply["ok"] == true { return json!({ "ok": true, "result": reply["result"], "error": Value::Null }); }
    memory_reply(Err(reply["error"]["message"].as_str().unwrap_or("Orchestrator yanıt vermedi.").to_string()))
}

#[tauri::command]
pub async fn memory_access_status(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || memory_link(&settings, "memory.status", None)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn memory_access_enable(app: tauri::AppHandle, write: Option<bool>) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || memory_link(&settings, "memory.connect", Some(json!({ "write": write.unwrap_or(false) })))).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn memory_access_disable(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || memory_link(&settings, "memory.disconnect", None)).await.map_err(|e| e.to_string())
}

fn index_reply(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => json!({ "ok": true, "result": value, "error": Value::Null }),
        Err(message) => json!({ "ok": false, "result": Value::Null,
            "error": { "code": "MEMORY_INDEX", "message": message, "hint": "SecondBrain'in kurulu ve güncel olduğunu denetleyin." } }),
    }
}

#[tauri::command]
pub async fn memory_index_status(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted(&settings, p, o, pl, id);
        index_reply(Ok(memory_index::status(&call)))
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn memory_index_refresh(app: tauri::AppHandle, force: Option<bool>) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        // A refresh can take minutes on a big vault; only that op gets the long timeout.
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted_timeout(&settings, p, o, pl, id,
            if o == "memory.index.refresh" { Duration::from_secs(15 * 60) } else { Duration::from_secs(30) });
        index_reply(memory_index::refresh(&call, force.unwrap_or(false)))
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn memory_semantic_status(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted(&settings, p, o, pl, id);
        index_reply(Ok(memory_semantic::status(&call)))
    }).await.map_err(|e| e.to_string())
}

fn semantic_call<'a>(settings: &'a Settings) -> impl Fn(&str, &str, Option<Value>, Option<String>) -> Value + 'a {
    // The model download and the first vector build are slow; only those ops get long timeouts.
    move |p, o, pl, id| rpp::rpp_call_trusted_timeout(settings, p, o, pl, id, match o {
        "memory.semantic.pull" => Duration::from_secs(65 * 60),
        "memory.index.refresh" => Duration::from_secs(30 * 60),
        _ => Duration::from_secs(30),
    })
}

#[tauri::command]
pub async fn memory_semantic_enable(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || index_reply(memory_semantic::enable(&semantic_call(&settings)))).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn memory_semantic_disable(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || index_reply(memory_semantic::disable(&semantic_call(&settings)))).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn memory_capture_run(app: tauri::AppHandle) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        // Summarising sessions runs the user's AI CLI; it can take many minutes.
        let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| rpp::rpp_call_trusted_timeout(&settings, p, o, pl, id, Duration::from_secs(30 * 60));
        let reply = call("secondbrain", "memory.capture.run", Some(json!({ "confirm": true })), Some(rpp::generate_operation_id()));
        if reply["ok"] == true { return json!({ "ok": true, "result": reply["result"], "error": Value::Null }); }
        index_reply(Err(format!("{}: {}", reply["error"]["code"].as_str().unwrap_or("UNKNOWN"), reply["error"]["message"].as_str().unwrap_or("yanıt yok"))))
    }).await.map_err(|e| e.to_string())
}

/// The product's own reply (ok/result/error with code, message, hint) for the memory workspace commands.
async fn workspace_call(app: tauri::AppHandle, op: &'static str, payload: Value, with_id: bool, timeout: Duration) -> Result<Value, String> {
    let settings = resolved_settings(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let id = with_id.then(rpp::generate_operation_id);
        let reply = rpp::rpp_call_trusted_timeout(&settings, "secondbrain", op, Some(payload), id, timeout);
        json!({ "ok": reply["ok"] == true, "result": reply["result"], "error": reply["error"] })
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn memory_search(app: tauri::AppHandle, query: String, limit: Option<u32>, mode: Option<String>) -> Result<Value, String> {
    workspace_call(app, "memory.search", json!({ "query": query, "limit": limit.unwrap_or(15), "mode": mode.unwrap_or_else(|| "auto".into()) }), false, Duration::from_secs(60)).await
}

#[tauri::command]
pub async fn memory_note_read(app: tauri::AppHandle, path: String) -> Result<Value, String> {
    workspace_call(app, "memory.note.read", json!({ "path": path }), false, Duration::from_secs(30)).await
}

#[tauri::command]
pub async fn memory_capture_note(app: tauri::AppHandle, title: Option<String>, content: String) -> Result<Value, String> {
    workspace_call(app, "memory.capture.note", json!({ "title": title.unwrap_or_default(), "content": content }), true, Duration::from_secs(60)).await
}

#[tauri::command]
pub async fn memory_briefing_run(app: tauri::AppHandle) -> Result<Value, String> {
    workspace_call(app, "memory.briefing.run", json!({ "confirm": true }), false, Duration::from_secs(30 * 60)).await
}
