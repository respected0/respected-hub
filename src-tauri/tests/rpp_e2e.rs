//! Real end-to-end proof that the Hub drives the real Orchestrator and SecondBrain
//! engines over RPP 1.0. Every run uses temporary data roots; no user data is touched.
use respected_hub_lib::rpp::{rpp_call, rpp_call_version};
use respected_hub_lib::Settings;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const PRODUCTS: [&str; 2] = ["orchestrator", "secondbrain"];

struct TempRoot(PathBuf);

impl TempRoot {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let path = std::env::temp_dir().join(format!("hub-rpp-e2e-{label}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path).expect("create temp data root");
        TempRoot(path)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
}

fn ecosystem_root() -> PathBuf {
    std::env::var_os("RESPECTED_ECOSYSTEM_ROOT").map(PathBuf::from).unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
    })
}

fn product_root(ecosystem: &Path, product: &str) -> PathBuf {
    match product {
        "orchestrator" => ecosystem.join("respected-orchestrator"),
        _ => ecosystem.join("respected-secondbrain"),
    }
}

fn settings_for(ecosystem: &Path, data_root: &Path) -> Settings {
    Settings {
        python: "python".into(),
        orchestrator_root: product_root(ecosystem, "orchestrator"),
        brain_root: product_root(ecosystem, "secondbrain"),
        data_root: data_root.to_path_buf(),
        brain_data_root: Some(data_root.to_path_buf()), brain_exe: None, brain_app_dir: None, orchestrator_exe: None,
    }
}

fn call(settings: &Settings, product: &str, op: &str, payload: Option<Value>, operation_id: Option<String>) -> Value {
    let response = rpp_call(settings, product, op, payload, operation_id);
    for key in ["rpp", "ok", "operation_id", "result", "error"] {
        assert!(response.get(key).is_some(), "{product} {op} missing '{key}': {response}");
    }
    let expected = if op.starts_with("lifecycle.") || op == "operation.receipt" { "1.1" } else { "1.0" };
    assert_eq!(response["rpp"], expected, "{product} {op}");
    response
}

fn unique_id(label: &str) -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("e2e-{label}-{nanos:x}")
}

fn first_non_secret_field(schema: &Value) -> Value {
    schema["result"]["fields"].as_array().and_then(|fields| {
        fields.iter().find(|field| field.get("secret") != Some(&Value::Bool(true))).cloned()
    }).expect("settings.schema must expose at least one non-secret field")
}

fn sample_value(field: &Value) -> Value {
    if let Some(member) = field["enum"].as_array().and_then(|values| values.first()) { return member.clone(); }
    let default = &field["default"];
    match field["type"].as_str() {
        Some("boolean") if default.is_boolean() => default.clone(),
        Some("integer") if default.is_number() => default.clone(),
        Some("string") if default.is_string() => default.clone(),
        Some("boolean") => Value::Bool(true),
        Some("integer") => json!(1),
        _ => json!("rpp-e2e"),
    }
}

#[test]
fn manifest_health_and_schema_are_reachable_for_both_products() {
    let ecosystem = ecosystem_root();
    for product in PRODUCTS {
        let data = TempRoot::new(product);
        let settings = settings_for(&ecosystem, &data.0);

        let manifest = call(&settings, product, "manifest", None, None);
        assert_eq!(manifest["ok"], true, "{product}: {manifest}");
        assert!(manifest["result"]["product_id"].is_string(), "{product}: {manifest}");
        assert_eq!(manifest["result"]["protocol"]["min"], "1.0", "{product}");
        let max = manifest["result"]["protocol"]["max"].as_str().unwrap_or("");
        assert!(["1.0", "1.1"].contains(&max), "{product}: protocol max {max}");
        assert_eq!(manifest["result"]["settings_supported"], true, "{product}");

        let health = call(&settings, product, "health", None, None);
        assert_eq!(health["ok"], true, "{product}: {health}");
        let status = health["result"]["status"].as_str().unwrap_or("");
        assert!(["healthy", "degraded", "unhealthy"].contains(&status), "{product}: {status}");
        assert!(health["result"]["checks"].is_array(), "{product}: {health}");

        let schema = call(&settings, product, "settings.schema", None, None);
        assert_eq!(schema["ok"], true, "{product}: {schema}");
        assert!(schema["result"]["fields"].as_array().map(|f| !f.is_empty()).unwrap_or(false), "{product}: {schema}");
    }
}

#[test]
fn validate_rejects_an_unknown_key_for_both_products() {
    let ecosystem = ecosystem_root();
    let unknown = json!({ "settings": { "definitely_not_a_real_rpp_key": "x" } });
    for product in PRODUCTS {
        let data = TempRoot::new(product);
        let settings = settings_for(&ecosystem, &data.0);
        let response = call(&settings, product, "settings.validate", Some(unknown.clone()), None);
        assert_eq!(response["ok"], true, "{product}: {response}");
        assert_eq!(response["result"]["valid"], false, "{product}: {response}");
        assert!(response["result"]["errors"].as_array().map(|e| !e.is_empty()).unwrap_or(false), "{product}: {response}");
    }
}

#[test]
fn apply_readback_and_idempotent_replay_for_both_products() {
    let ecosystem = ecosystem_root();
    for product in PRODUCTS {
        let data = TempRoot::new(product);
        let settings = settings_for(&ecosystem, &data.0);

        let schema = call(&settings, product, "settings.schema", None, None);
        let field = first_non_secret_field(&schema);
        let key = field["key"].as_str().expect("schema key").to_string();
        let value = sample_value(&field);
        let mut inner = serde_json::Map::new();
        inner.insert(key.clone(), value.clone());
        let payload = json!({ "settings": Value::Object(inner) });

        let operation_id = unique_id(product);
        let first = call(&settings, product, "settings.apply", Some(payload.clone()), Some(operation_id.clone()));
        assert_eq!(first["ok"], true, "{product}: {first}");

        let readback = call(&settings, product, "settings.readback", None, None);
        assert_eq!(readback["ok"], true, "{product}: {readback}");
        assert_eq!(readback["result"]["source"], "file", "{product}: {readback}");
        assert_eq!(readback["result"]["settings"][key.as_str()], value, "{product}: {readback}");

        let replay = call(&settings, product, "settings.apply", Some(payload.clone()), Some(operation_id.clone()));
        assert_eq!(replay["ok"], true, "{product}: {replay}");
        assert_eq!(replay["result"], first["result"], "{product}: replay must return the stored receipt");
    }
}

#[test]
fn unknown_op_is_rejected_locally_for_both_products() {
    let ecosystem = ecosystem_root();
    for product in PRODUCTS {
        let data = TempRoot::new(product);
        let settings = settings_for(&ecosystem, &data.0);
        let response = call(&settings, product, "definitely_unknown_op", None, None);
        assert_eq!(response["ok"], false, "{product}: {response}");
        assert_eq!(response["error"]["code"], "INVALID_OP", "{product}: {response}");
        assert_eq!(response["result"], Value::Null, "{product}: {response}");
    }
}

#[test]
fn missing_engine_root_reports_engine_not_found() {
    let data = TempRoot::new("missing");
    let settings = Settings {
        python: "python".into(),
        orchestrator_root: data.0.join("no-orchestrator"),
        brain_root: data.0.join("no-secondbrain"),
        data_root: data.0.clone(),
        brain_data_root: Some(data.0.clone()), brain_exe: None, brain_app_dir: None, orchestrator_exe: None,
    };
    for product in PRODUCTS {
        let response = call(&settings, product, "manifest", None, None);
        assert_eq!(response["ok"], false, "{product}: {response}");
        assert_eq!(response["error"]["code"], "ENGINE_NOT_FOUND", "{product}: {response}");
    }
}

#[test]
fn protocol_mismatch_is_surfaced_by_the_engine() {
    let ecosystem = ecosystem_root();
    for product in PRODUCTS {
        let data = TempRoot::new(product);
        let settings = settings_for(&ecosystem, &data.0);
        let response = rpp_call_version(&settings, product, "manifest", None, None, "0.9");
        assert_eq!(response["ok"], false, "{product}: {response}");
        assert_eq!(response["error"]["code"], "PROTOCOL_UNSUPPORTED", "{product}: {response}");
    }
}

#[test]
fn lifecycle_plan_and_unknown_receipt_work_over_v11_for_both_products() {
    let ecosystem = ecosystem_root();
    for product in PRODUCTS {
        let data = TempRoot::new(product);
        let settings = settings_for(&ecosystem, &data.0);
        let manifest = call(&settings, product, "manifest", None, None);
        if manifest["result"]["protocol"]["max"] != "1.1" { continue; }

        let plan = call(&settings, product, "lifecycle.plan", Some(json!({ "action": "install", "target": { "app_root": data.0.join("app").to_string_lossy() } })), None);
        assert_eq!(plan["ok"], true, "{product}: {plan}");
        assert_eq!(plan["result"]["dry_run"], true, "{product}: {plan}");
        assert!(plan["result"]["plan"]["steps"].is_array(), "{product}: {plan}");

        let missing = call(&settings, product, "operation.receipt", Some(json!({ "operation_id": "no-such-operation-1" })), None);
        assert_eq!(missing["ok"], false, "{product}: {missing}");
        assert_eq!(missing["error"]["code"], "RECEIPT_NOT_FOUND", "{product}: {missing}");
    }
}
