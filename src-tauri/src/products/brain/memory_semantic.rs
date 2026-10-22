// Optional local semantic search of Brain: status, enable (confirm -> model download -> vector build) and disable.
// FTS search keeps working in every state. The download goes through the user's local embedding service only.
use super::CallFn;
use crate::rpp::generate_operation_id;
use serde_json::{json, Value};


fn is_ok(reply: &Value) -> bool { reply["ok"] == true }

fn failure(reply: &Value, what: &str) -> String {
    let hint = reply["error"]["hint"].as_str().filter(|h| !h.is_empty()).map(|h| format!(" ({h})")).unwrap_or_default();
    format!("{what}: {} — {}{hint}", reply["error"]["code"].as_str().unwrap_or("UNKNOWN"), reply["error"]["message"].as_str().unwrap_or("yanıt yok"))
}

fn supported(call: CallFn) -> bool { super::offers(&call("brain", "manifest", None, None), "memory.semantic.status") }

pub fn status(call: CallFn) -> Value {
    if !supported(call) { return json!({ "supported": false, "state": "unsupported" }); }
    let reply = call("brain", "memory.semantic.status", Some(json!({})), None);
    if !is_ok(&reply) {
        let state = if reply["error"]["code"] == "NO_VAULT" { "no-vault" } else { "unknown" };
        return json!({ "supported": true, "state": state, "problem": failure(&reply, "Anlamsal arama durumu okunamadı") });
    }
    let mut shown = reply["result"].clone();
    shown["supported"] = true.into();
    shown
}

pub fn enable(call: CallFn) -> Result<Value, String> {
    if !supported(call) { return Err("Kurulu Brain anlamsal aramayı desteklemiyor; önce güncelleyin.".into()); }
    let steps: [(&str, Value, &str); 3] = [
        ("memory.semantic.configure", json!({ "confirm": true, "enabled": true }), "Anlamsal arama açılamadı"),
        ("memory.semantic.pull", json!({ "confirm": true }), "Model indirilemedi"),
        ("memory.index.refresh", json!({ "confirm": true }), "Vektör dizini oluşturulamadı"),
    ];
    for (op, payload, what) in steps {
        let id = (op == "memory.index.refresh").then(generate_operation_id);
        let reply = call("brain", op, Some(payload), id);
        if !is_ok(&reply) { return Err(failure(&reply, what)); }
    }
    Ok(status(call))
}

pub fn disable(call: CallFn) -> Result<Value, String> {
    let reply = call("brain", "memory.semantic.configure", Some(json!({ "confirm": true, "enabled": false })), None);
    if !is_ok(&reply) { return Err(failure(&reply, "Anlamsal arama kapatılamadı")); }
    Ok(status(call))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn fake<'a>(log: &'a RefCell<Vec<String>>, fail_at: Option<&'static str>) -> impl Fn(&str, &str, Option<Value>, Option<String>) -> Value + 'a {
        move |_, op, payload, _| {
            log.borrow_mut().push(op.to_string());
            if Some(op) == fail_at { return json!({ "ok": false, "result": Value::Null, "error": { "code": "PROVIDER_UNAVAILABLE", "message": "kapalı", "hint": "Ollama" } }); }
            match op {
                "manifest" => json!({ "ok": true, "result": { "capabilities": ["memory.semantic.status"] }, "error": Value::Null }),
                "memory.semantic.status" => json!({ "ok": true, "result": { "state": "ready", "enabled": true }, "error": Value::Null }),
                _ => { assert!(payload.map_or(true, |p| p["confirm"] == true || p == json!({}))); json!({ "ok": true, "result": {}, "error": Value::Null }) }
            }
        }
    }

    #[test]
    fn enable_runs_configure_pull_refresh_in_order() {
        let log = RefCell::new(vec![]);
        let done = enable(&fake(&log, None)).unwrap();
        assert_eq!(done["state"], "ready");
        let ops = log.borrow().iter().filter(|o| o.starts_with("memory.")).cloned().collect::<Vec<_>>();
        assert_eq!(ops, ["memory.semantic.configure", "memory.semantic.pull", "memory.index.refresh", "memory.semantic.status"]);
    }

    #[test]
    fn a_failed_download_stops_before_the_vector_build_and_explains() {
        let log = RefCell::new(vec![]);
        let error = enable(&fake(&log, Some("memory.semantic.pull"))).unwrap_err();
        assert!(error.contains("Model indirilemedi") && error.contains("kapalı"), "the product's own explanation is shown: {error}");
        assert!(!log.borrow().iter().any(|o| o == "memory.index.refresh"));
    }

    #[test]
    fn an_old_brain_is_reported_unsupported() {
        let call = |_: &str, _: &str, _: Option<Value>, _: Option<String>| json!({ "ok": true, "result": { "capabilities": [] }, "error": Value::Null });
        assert_eq!(status(&call)["state"], "unsupported");
        assert!(enable(&call).is_err());
    }
}
