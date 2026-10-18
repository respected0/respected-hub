// SecondBrain memory access for the Orchestrator: enable / disable / status.
// The grant token lives only inside `enable`, between the memory.grant reply and the Orchestrator
// settings.apply call. It is never returned, logged or placed in an error text (see `scrub`).
use crate::rpp::generate_operation_id;
use serde_json::{json, Map, Value};

pub type CallFn<'a> = &'a dyn Fn(&str, &str, Option<Value>, Option<String>) -> Value;

const MASK: &str = "***set***";

/// Removes the known secret and anything that looks like a grant token from text built from engine replies.
fn scrub(text: &str, secret: &str) -> String {
    let text = if secret.is_empty() { text.to_string() } else { text.replace(secret, "***") };
    text.split(' ').map(|word| if word.contains("rbm_") { "***" } else { word }).collect::<Vec<_>>().join(" ")
}

fn is_ok(reply: &Value) -> bool { reply["ok"] == true }

fn failure(reply: &Value, what: &str, secret: &str) -> String {
    let code = reply["error"]["code"].as_str().unwrap_or("UNKNOWN");
    let message = reply["error"]["message"].as_str().unwrap_or("yanıt yok");
    scrub(&format!("{what}: {code} — {message}"), secret)
}

struct Snapshot { supported: bool, enabled: bool, write: bool, write_capable: bool, grant_id: Option<String>, token_set: bool, readback_ok: bool, problems: Vec<String> }

fn snapshot(call: CallFn) -> Snapshot {
    let mut problems = Vec::new();
    let schema = call("orchestrator", "settings.schema", None, None);
    let has_field = if is_ok(&schema) {
        schema["result"]["fields"].as_array().map_or(false, |fields| fields.iter().any(|f| f["key"] == "memory_enabled"))
    } else { problems.push(failure(&schema, "Orchestrator ayar şeması", "")); false };
    let manifest = call("secondbrain", "manifest", None, None);
    let has_capability = if is_ok(&manifest) {
        // A protocol 1.0 manifest hides 1.1-only capabilities from `capabilities`; `capability_details` always lists them.
        ["capabilities", "capability_details"].iter().any(|key| manifest["result"][*key].as_array()
            .map_or(false, |caps| caps.iter().any(|c| *c == "memory_recall" || c["id"] == "memory_recall")))
    } else { problems.push(failure(&manifest, "SecondBrain bildirimi", "")); false };
    let write_capable = is_ok(&manifest) && ["capabilities", "capability_details"].iter().any(|key| manifest["result"][*key].as_array()
        .map_or(false, |caps| caps.iter().any(|c| *c == "memory_write" || c["id"] == "memory_write")));
    let readback = call("orchestrator", "settings.readback", None, None);
    let readback_ok = is_ok(&readback);
    if !readback_ok { problems.push(failure(&readback, "Orchestrator ayarları", "")); }
    let settings = &readback["result"]["settings"];
    Snapshot {
        supported: has_field && has_capability,
        enabled: settings["memory_enabled"] == true, write: settings["memory_write"] == true, write_capable,
        grant_id: settings["memory_grant_id"].as_str().filter(|s| !s.is_empty()).map(str::to_string),
        token_set: settings["memory_token"] == MASK,
        readback_ok, problems,
    }
}

pub fn status(call: CallFn) -> Value {
    let s = snapshot(call);
    json!({ "supported": s.supported, "enabled": s.enabled, "write": s.write, "write_supported": s.write_capable, "pending_revoke": !s.enabled && s.grant_id.is_some(),
        "grant_id": s.grant_id, "token_set": s.token_set, "problems": s.problems })
}

fn revoke(call: CallFn, grant_id: &str) -> Result<(), String> {
    let reply = call("secondbrain", "memory.revoke", Some(json!({ "grant_id": grant_id })), None);
    if is_ok(&reply) { Ok(()) } else { Err(failure(&reply, "İzin geri alınamadı", "")) }
}

pub fn enable(call: CallFn, brain_command: (Vec<String>, Map<String, Value>), write: bool) -> Result<Value, String> {
    let before = snapshot(call);
    if !before.supported { return Err("Bu sürümde bellek erişimi desteklenmiyor (Orchestrator veya SecondBrain eski).".into()); }
    if write && !before.write_capable { return Err("Kurulu SecondBrain not yazmayı (memory_write) desteklemiyor; önce güncelleyin.".into()); }
    if let (true, Some(id)) = (before.enabled, &before.grant_id) { return Ok(json!({ "enabled": true, "grant_id": id })); }
    // A disabled install that still records a grant: try to close it before opening a new one.
    if let Some(old) = &before.grant_id { let _ = revoke(call, old); }

    let granted = call("secondbrain", "memory.grant", Some(json!({ "label": "Orchestrator", "write": write })), Some(generate_operation_id()));
    if !is_ok(&granted) { return Err(failure(&granted, "Bellek izni verilemedi", "")); }
    let grant_id = granted["result"]["grant_id"].as_str().unwrap_or("").to_string();
    let token = granted["result"]["token"].as_str().unwrap_or("").to_string();
    drop(granted);
    if grant_id.is_empty() || token.is_empty() {
        if !grant_id.is_empty() { let _ = revoke(call, &grant_id); }
        return Err("SecondBrain beklenen izin yanıtını vermedi.".into());
    }
    let undo = |reason: String| -> Result<Value, String> {
        let note = if revoke(call, &grant_id).is_err() { " Yeni izin geri alınamadı; SecondBrain'de açık kalmış olabilir." } else { "" };
        Err(format!("{}{note}", scrub(&reason, &token)))
    };

    let (argv, env) = brain_command;
    let command = json!({ "argv": argv, "env": env }).to_string();
    let payload = json!({ "settings": { "memory_enabled": true, "memory_command": command, "memory_grant_id": grant_id, "memory_token": token, "memory_write": write } });
    let validated = call("orchestrator", "settings.validate", Some(payload.clone()), None);
    if !is_ok(&validated) || validated["result"]["valid"] != true {
        let detail = validated["result"]["errors"].as_array().map(|errors| errors.iter().filter_map(|e| Some(format!("{}: {}", e["key"].as_str()?, e["message"].as_str()?))).collect::<Vec<_>>().join("; ")).unwrap_or_default();
        return undo(if is_ok(&validated) { format!("Ayarlar doğrulanamadı: {}", detail.replace(token.as_str(), "***")) } else { failure(&validated, "Ayarlar doğrulanamadı", &token) });
    }
    let applied = call("orchestrator", "settings.apply", Some(payload), Some(generate_operation_id()));
    if !is_ok(&applied) { return undo(failure(&applied, "Ayarlar uygulanamadı", &token)); }
    let readback = call("orchestrator", "settings.readback", None, None);
    let settings = &readback["result"]["settings"];
    if !is_ok(&readback) || settings["memory_enabled"] != true || settings["memory_token"] != MASK {
        return undo("Orchestrator ayarları doğrulanamadı".into());
    }
    Ok(json!({ "enabled": true, "grant_id": grant_id }))
}

/// SecondBrain moved to a new versioned folder: point an enabled Orchestrator memory provider at it (audit P1 #21).
pub fn refresh_command(call: CallFn, brain_command: (Vec<String>, Map<String, Value>)) {
    let before = snapshot(call);
    if !before.enabled { return; }
    let (argv, env) = brain_command;
    let payload = json!({ "settings": { "memory_command": json!({ "argv": argv, "env": env }).to_string() } });
    let _ = call("orchestrator", "settings.apply", Some(payload), Some(generate_operation_id()));
}

pub fn disable(call: CallFn) -> Result<Value, String> {
    let before = snapshot(call);
    if !before.readback_ok { return Err("Orchestrator ayarları okunamadı; bellek erişimi kapatılamadı.".into()); }
    if !before.enabled && before.grant_id.is_none() && !before.token_set {
        return Ok(json!({ "enabled": false, "revoked": false, "pending_revoke": false }));
    }
    let revoked = match &before.grant_id { Some(id) => revoke(call, id).is_ok(), None => true };
    let mut settings = json!({ "memory_enabled": false, "memory_token": "", "memory_command": "", "memory_write": false });
    // Keep the grant id while SecondBrain could not revoke it, so the revoke can be retried.
    if revoked { settings["memory_grant_id"] = json!(""); }
    let applied = call("orchestrator", "settings.apply", Some(json!({ "settings": settings })), Some(generate_operation_id()));
    if !is_ok(&applied) { return Err(failure(&applied, "Ayarlar uygulanamadı", "")); }
    Ok(json!({ "enabled": false, "revoked": before.grant_id.is_some() && revoked, "pending_revoke": !revoked }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    const TOKEN: &str = "rbm_SECRET123456";

    struct Fake {
        log: RefCell<Vec<(String, String, Option<Value>)>>, settings: RefCell<Map<String, Value>>,
        has_schema: Cell<bool>, capability: Cell<bool>, apply_ok: Cell<bool>, revoke_ok: Cell<bool>, grants: Cell<u32>,
    }

    impl Fake {
        fn new() -> Self {
            Fake { log: RefCell::new(vec![]), settings: RefCell::new(Map::new()), has_schema: Cell::new(true), capability: Cell::new(true),
                apply_ok: Cell::new(true), revoke_ok: Cell::new(true), grants: Cell::new(0) }
        }
        fn ok(result: Value) -> Value { json!({ "ok": true, "result": result, "error": Value::Null }) }
        fn err(code: &str, message: &str) -> Value { json!({ "ok": false, "result": Value::Null, "error": { "code": code, "message": message } }) }
        fn call(&self, product: &str, op: &str, payload: Option<Value>, _id: Option<String>) -> Value {
            self.log.borrow_mut().push((product.into(), op.into(), payload.clone()));
            match (product, op) {
                ("orchestrator", "settings.schema") => Self::ok(json!({ "fields": [{ "key": if self.has_schema.get() { "memory_enabled" } else { "other" } }] })),
                ("secondbrain", "manifest") => Self::ok(json!({ "capabilities": [if self.capability.get() { "memory_recall" } else { "other" }] })),
                ("orchestrator", "settings.readback") => Self::ok(json!({ "settings": Value::Object(self.settings.borrow().clone()) })),
                ("orchestrator", "settings.validate") => Self::ok(json!({ "valid": true, "errors": [] })),
                ("orchestrator", "settings.apply") => {
                    let given = payload.as_ref().and_then(|p| p["settings"].as_object()).cloned().unwrap_or_default();
                    if !self.apply_ok.get() {
                        return Self::err("APPLY_FAILED", &format!("reddedildi {}", given.get("memory_token").and_then(Value::as_str).unwrap_or("")));
                    }
                    for (key, value) in given {
                        let stored = if key == "memory_token" { json!(if value == "" { "" } else { MASK }) } else { value };
                        self.settings.borrow_mut().insert(key, stored);
                    }
                    Self::ok(json!({ "applied": true }))
                }
                ("secondbrain", "memory.grant") => {
                    self.grants.set(self.grants.get() + 1);
                    Self::ok(json!({ "grant_id": format!("grant-{}", self.grants.get()), "token": TOKEN }))
                }
                ("secondbrain", "memory.revoke") if self.revoke_ok.get() => Self::ok(json!({ "grant_id": "x", "revoked": true })),
                ("secondbrain", "memory.revoke") => Self::err("BRAIN_DOWN", "SecondBrain kapalı"),
                _ => Self::err("INVALID_OP", "bilinmiyor"),
            }
        }
        fn ops(&self) -> Vec<String> { self.log.borrow().iter().map(|(p, o, _)| format!("{p} {o}")).collect() }
        fn payload_of(&self, op: &str) -> Value { self.log.borrow().iter().rev().find(|(_, o, _)| o == op).and_then(|(_, _, p)| p.clone()).unwrap() }
    }

    fn command() -> (Vec<String>, Map<String, Value>) {
        let mut env = Map::new();
        env.insert("PYTHONDONTWRITEBYTECODE".into(), json!("1"));
        (vec!["C:/brain/respectedbrain.exe".into(), "rpp".into()], env)
    }

    #[test]
    fn enable_hands_the_token_only_to_the_orchestrator_apply() {
        let fake = Fake::new();
        let call = |p: &str, o: &str, pl: Option<Value>, i: Option<String>| fake.call(p, o, pl, i);
        let enabled = enable(&call, command(), false).unwrap();
        assert_eq!(enabled, json!({ "enabled": true, "grant_id": "grant-1" }));
        let applied = fake.payload_of("settings.apply");
        assert_eq!(applied["settings"]["memory_token"], TOKEN);
        assert_eq!(applied["settings"]["memory_grant_id"], "grant-1");
        assert_eq!(applied["settings"]["memory_enabled"], true);
        let parsed: Value = serde_json::from_str(applied["settings"]["memory_command"].as_str().unwrap()).unwrap();
        assert_eq!(parsed["argv"][1], "rpp");
        assert_eq!(parsed["env"]["PYTHONDONTWRITEBYTECODE"], "1");
        assert!(!fake.ops().contains(&"secondbrain memory.revoke".to_string()));
        let shown = status(&call);
        assert_eq!(shown["enabled"], true);
        assert_eq!(shown["token_set"], true);
        assert_eq!(shown["grant_id"], "grant-1");
        assert_eq!(shown["supported"], true);
        assert!(!enabled.to_string().contains(TOKEN) && !shown.to_string().contains(TOKEN));
        assert!(!disable(&call).unwrap().to_string().contains(TOKEN));
    }

    #[test]
    fn write_permission_is_a_separate_opt_in_and_needs_the_capability() {
        let fake = Fake::new();
        let call = |p: &str, o: &str, pl: Option<Value>, i: Option<String>| fake.call(p, o, pl, i);
        enable(&call, command(), false).unwrap();
        assert_eq!(fake.payload_of("memory.grant")["write"], false);
        assert_eq!(fake.payload_of("settings.apply")["settings"]["memory_write"], false);
        disable(&call).unwrap();
        let denied = enable(&call, command(), true).unwrap_err();
        assert!(denied.contains("memory_write"), "{denied}");  // fake manifest lacks the capability
        assert_eq!(fake.grants.get(), 1);
    }

    #[test]
    fn enable_revokes_the_new_grant_when_apply_fails_and_never_leaks_the_token() {
        let fake = Fake::new();
        fake.apply_ok.set(false);
        let call = |p: &str, o: &str, pl: Option<Value>, i: Option<String>| fake.call(p, o, pl, i);
        let error = enable(&call, command(), false).unwrap_err();
        assert!(!error.contains(TOKEN), "{error}");
        assert_eq!(fake.payload_of("memory.revoke"), json!({ "grant_id": "grant-1" }));
        assert!(fake.settings.borrow().get("memory_enabled").is_none());
    }

    #[test]
    fn enable_refuses_before_any_grant_when_unsupported() {
        for (schema, capability) in [(true, false), (false, true)] {
            let fake = Fake::new();
            fake.has_schema.set(schema);
            fake.capability.set(capability);
            let call = |p: &str, o: &str, pl: Option<Value>, i: Option<String>| fake.call(p, o, pl, i);
            assert!(enable(&call, command(), false).is_err());
            assert!(!fake.ops().iter().any(|o| o.contains("memory.grant")), "{:?}", fake.ops());
            assert_eq!(status(&call)["supported"], false);
        }
    }

    #[test]
    fn disable_revokes_then_clears_and_reports_a_failed_revoke() {
        let fake = Fake::new();
        let call = |p: &str, o: &str, pl: Option<Value>, i: Option<String>| fake.call(p, o, pl, i);
        enable(&call, command(), false).unwrap();
        let done = disable(&call).unwrap();
        assert_eq!(done, json!({ "enabled": false, "revoked": true, "pending_revoke": false }));
        let shown = status(&call);
        assert_eq!((shown["enabled"].clone(), shown["token_set"].clone(), shown["grant_id"].clone(), shown["pending_revoke"].clone()),
            (json!(false), json!(false), Value::Null, json!(false)));

        enable(&call, command(), false).unwrap();
        fake.revoke_ok.set(false);
        let pending = disable(&call).unwrap();
        assert_eq!(pending, json!({ "enabled": false, "revoked": false, "pending_revoke": true }));
        let shown = status(&call);
        assert_eq!(shown["enabled"], false);
        assert_eq!(shown["token_set"], false);
        assert_eq!(shown["grant_id"], "grant-2");
        assert_eq!(shown["pending_revoke"], true);
    }

    #[test]
    fn disable_is_idempotent_when_already_disabled() {
        let fake = Fake::new();
        let call = |p: &str, o: &str, pl: Option<Value>, i: Option<String>| fake.call(p, o, pl, i);
        assert_eq!(disable(&call).unwrap(), json!({ "enabled": false, "revoked": false, "pending_revoke": false }));
        assert!(!fake.ops().iter().any(|o| o.contains("revoke") || o.contains("apply")), "{:?}", fake.ops());
    }
}
