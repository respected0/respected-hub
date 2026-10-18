// SecondBrain memory search index: read its state and refresh it. Only the user's click may start a refresh;
// SecondBrain writes nothing but its own search index.
use crate::rpp::generate_operation_id;
use serde_json::{json, Value};

pub type CallFn<'a> = &'a dyn Fn(&str, &str, Option<Value>, Option<String>) -> Value;

fn is_ok(reply: &Value) -> bool { reply["ok"] == true }

fn code_of(reply: &Value) -> &str { reply["error"]["code"].as_str().unwrap_or("UNKNOWN") }

fn failure(reply: &Value, what: &str) -> String {
    format!("{what}: {} — {}", code_of(reply), reply["error"]["message"].as_str().unwrap_or("yanıt yok"))
}

/// A protocol 1.0 manifest hides 1.1-only capabilities from `capabilities`; `capability_details` always lists them.
fn supports_index(call: CallFn, problems: &mut Vec<String>) -> bool {
    let manifest = call("secondbrain", "manifest", None, None);
    if !is_ok(&manifest) { problems.push(failure(&manifest, "SecondBrain bildirimi")); return false; }
    ["capabilities", "capability_details"].iter().any(|key| manifest["result"][*key].as_array()
        .map_or(false, |caps| caps.iter().any(|c| *c == "memory_index" || c["id"] == "memory_index")))
}

pub fn status(call: CallFn) -> Value {
    let mut problems = Vec::new();
    let supported = supports_index(call, &mut problems);
    let mut shown = json!({ "supported": supported, "state": "unsupported", "indexed_files": 0, "changed_files": 0,
        "vault_id": Value::Null, "last_refresh": Value::Null, "problems": [] });
    if supported {
        let reply = call("secondbrain", "memory.index.status", Some(json!({})), None);
        if is_ok(&reply) {
            let result = &reply["result"];
            shown["state"] = result["index"]["state"].as_str().unwrap_or("unknown").into();
            shown["indexed_files"] = result["index"]["indexed_files"].as_u64().unwrap_or(0).into();
            shown["changed_files"] = result["index"]["changed_files"].as_u64().unwrap_or(0).into();
            shown["vault_id"] = result["vault_id"].clone();
            shown["last_refresh"] = result["last_refresh"].clone();
        } else if code_of(&reply) == "NO_VAULT" {
            shown["state"] = "no-vault".into();
        } else {
            shown["state"] = "unknown".into();
            problems.push(failure(&reply, "Dizin durumu okunamadı"));
        }
    }
    shown["problems"] = json!(problems);
    shown
}

pub fn refresh(call: CallFn, force: bool) -> Result<Value, String> {
    let mut problems = Vec::new();
    if !supports_index(call, &mut problems) {
        return Err(problems.into_iter().next().unwrap_or_else(|| "Bu SecondBrain sürümü dizin yenilemeyi desteklemiyor.".into()));
    }
    let reply = call("secondbrain", "memory.index.refresh", Some(json!({ "confirm": true, "force": force })), Some(generate_operation_id()));
    if !is_ok(&reply) {
        return Err(match code_of(&reply) {
            "INDEX_BUSY" => "Dizin başka bir SecondBrain işlemi tarafından kullanılıyor; birazdan tekrar deneyin.".to_string(),
            "NO_VAULT" => "Kayıtlı bir kasa yok.".to_string(),
            _ => failure(&reply, "Dizin yenilenemedi"),
        });
    }
    let r = &reply["result"];
    Ok(json!({ "indexed": r["indexed"], "skipped": r["skipped"], "deleted": r["deleted"], "total": r["total"],
        "duration_seconds": r["duration_seconds"], "state": r["index"]["state"] }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Fake { log: RefCell<Vec<(String, Option<Value>)>>, capability: Option<&'static str>, status: Value, refresh: Value }

    fn ok(result: Value) -> Value { json!({ "ok": true, "result": result, "error": Value::Null }) }
    fn err(code: &str) -> Value { json!({ "ok": false, "result": Value::Null, "error": { "code": code, "message": "ileti" } }) }

    impl Fake {
        fn new(capability: Option<&'static str>) -> Self {
            Fake { log: RefCell::new(vec![]), capability, status: ok(json!({ "vault_id": "v1", "index": { "state": "fresh", "indexed_files": 3, "checked_files": 3, "changed_files": 0 },
                "last_refresh": { "at": "t", "indexed": 3 } })),
                refresh: ok(json!({ "vault_id": "v1", "indexed": 2, "skipped": 1, "deleted": 0, "total": 3, "duration_seconds": 0.5, "force": false, "replayed": false,
                    "index": { "state": "fresh" } })) }
        }
        fn call(&self, op: &str, payload: Option<Value>) -> Value {
            self.log.borrow_mut().push((op.into(), payload));
            match op {
                "manifest" => ok(match self.capability {
                    Some("details") => json!({ "capabilities": [], "capability_details": [{ "id": "memory_index" }] }),
                    Some(_) => json!({ "capabilities": ["memory_index"] }),
                    None => json!({ "capabilities": ["other"], "capability_details": [{ "id": "other" }] }),
                }),
                "memory.index.status" => self.status.clone(),
                "memory.index.refresh" => self.refresh.clone(),
                _ => err("INVALID_OP"),
            }
        }
        fn ops(&self) -> Vec<String> { self.log.borrow().iter().map(|(o, _)| o.clone()).collect() }
    }

    #[test]
    fn unsupported_brain_reports_unsupported_and_never_refreshes() {
        let fake = Fake::new(None);
        let call = |_: &str, o: &str, p: Option<Value>, _: Option<String>| fake.call(o, p);
        let shown = status(&call);
        assert_eq!((shown["supported"].clone(), shown["state"].clone()), (json!(false), json!("unsupported")));
        assert!(refresh(&call, false).is_err());
        assert_eq!(fake.ops(), ["manifest", "manifest"]);
    }

    #[test]
    fn status_reads_the_index_state_for_both_capability_lists() {
        for capability in ["details", "plain"] {
            let fake = Fake::new(Some(capability));
            let call = |_: &str, o: &str, p: Option<Value>, _: Option<String>| fake.call(o, p);
            let shown = status(&call);
            assert_eq!(shown["supported"], true, "{capability}");
            assert_eq!(shown["state"], "fresh");
            assert_eq!(shown["indexed_files"], 3);
            assert_eq!(shown["vault_id"], "v1");
            assert_eq!(shown["last_refresh"]["indexed"], 3);
            assert_eq!(shown["problems"], json!([]));
        }
    }

    #[test]
    fn status_without_a_vault_is_a_state_not_an_error() {
        let mut fake = Fake::new(Some("details"));
        fake.status = err("NO_VAULT");
        let call = |_: &str, o: &str, p: Option<Value>, _: Option<String>| fake.call(o, p);
        let shown = status(&call);
        assert_eq!(shown["state"], "no-vault");
        assert_eq!(shown["problems"], json!([]));
        assert_eq!(shown["last_refresh"], Value::Null);
    }

    #[test]
    fn status_failure_is_reported_as_a_problem() {
        let mut fake = Fake::new(Some("details"));
        fake.status = err("INDEX_INVALID");
        let call = |_: &str, o: &str, p: Option<Value>, _: Option<String>| fake.call(o, p);
        let shown = status(&call);
        assert_eq!(shown["state"], "unknown");
        assert_eq!(shown["problems"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn refresh_sends_exactly_confirm_and_force_and_returns_the_summary() {
        for force in [false, true] {
            let fake = Fake::new(Some("details"));
            let ids = RefCell::new(vec![]);
            let call = |_: &str, o: &str, p: Option<Value>, id: Option<String>| { if o == "memory.index.refresh" { ids.borrow_mut().push(id); } fake.call(o, p) };
            let done = refresh(&call, force).unwrap();
            assert_eq!(done, json!({ "indexed": 2, "skipped": 1, "deleted": 0, "total": 3, "duration_seconds": 0.5, "state": "fresh" }));
            let payload = fake.log.borrow().iter().find(|(o, _)| o == "memory.index.refresh").and_then(|(_, p)| p.clone()).unwrap();
            assert_eq!(payload, json!({ "confirm": true, "force": force }));
            assert!(ids.borrow()[0].as_deref().map_or(false, |id| id.len() >= 8));
        }
    }

    #[test]
    fn refresh_errors_are_turkish_by_code() {
        for (code, text) in [("INDEX_BUSY", "Dizin başka bir SecondBrain işlemi tarafından kullanılıyor; birazdan tekrar deneyin."),
            ("NO_VAULT", "Kayıtlı bir kasa yok."), ("BAD_REQUEST", "Dizin yenilenemedi: BAD_REQUEST — ileti")] {
            let mut fake = Fake::new(Some("details"));
            fake.refresh = err(code);
            let call = |_: &str, o: &str, p: Option<Value>, _: Option<String>| fake.call(o, p);
            assert_eq!(refresh(&call, false).unwrap_err(), text);
        }
    }
}
