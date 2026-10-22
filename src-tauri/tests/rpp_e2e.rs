//! Real end-to-end proof that the Hub drives the real Orchestrator and SecondBrain engines over RPP 1. The products choose
//! their folders themselves (RESPECTED_APP_DIR / RESPECTED_DATA_DIR, the one override every product shares): this test points
//! them once at a disposable folder, so no user data is touched.
use respected_hub_lib::rpp::rpp_call;
use respected_hub_lib::Settings;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
mod common;
use common::{ecosystem, Temp};

const PRODUCTS: [&str; 2] = ["orchestrator", "secondbrain"];

/// One disposable product folder for the whole test binary (the variables are process-wide).
fn isolated() {
    static ROOT: OnceLock<Temp> = OnceLock::new();
    let root = ROOT.get_or_init(|| Temp::new("rpp-products"));
    std::env::set_var("RESPECTED_APP_DIR", root.0.join("app"));
    std::env::set_var("RESPECTED_DATA_DIR", root.0.join("data"));
}

fn settings_for(ecosystem: &Path) -> Settings {
    isolated();
    Settings { python: "python".into(), orchestrator_root: ecosystem.join("respected-orchestrator"),
        brain_root: ecosystem.join("respected-secondbrain"), brain_exe: None, orchestrator_exe: None }
}

fn call(settings: &Settings, product: &str, op: &str, payload: Option<Value>, operation_id: Option<String>) -> Value {
    let response = rpp_call(settings, product, op, payload, operation_id);
    for key in ["rpp", "ok", "operation_id", "result", "error"] {
        assert!(response.get(key).is_some(), "{product} {op} missing '{key}': {response}");
    }
    assert_eq!(response["rpp"], "1", "{product} {op}");
    response
}

fn unique_id(label: &str) -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("e2e-{label}-{nanos:x}")
}

#[test]
fn manifest_health_and_schema_follow_the_shared_contract_for_both_products() {
    let settings = settings_for(&ecosystem());
    for product in PRODUCTS {
        let manifest = call(&settings, product, "manifest", None, None);
        assert_eq!(manifest["ok"], true, "{product}: {manifest}");
        assert_eq!(manifest["result"]["rpp"], "1", "{product}");
        assert!(manifest["result"]["product_id"].as_str().map_or(false, |id| id.starts_with("respected-")), "{product}: {manifest}");
        for key in ["installed", "install_state", "roots", "capabilities", "product_version"] {
            assert!(manifest["result"].get(key).is_some(), "{product} manifest lacks {key}");
        }
        assert!(manifest["result"]["capabilities"].as_array().unwrap().iter().all(|op| !op.as_str().unwrap().starts_with("lifecycle.")), "{product}");

        let health = call(&settings, product, "health", None, None);
        let status = health["result"]["status"].as_str().unwrap_or("");
        assert!(["healthy", "degraded", "unhealthy"].contains(&status), "{product}: {status}");

        let fields = call(&settings, product, "settings.schema", None, None)["result"]["fields"].as_array().cloned().unwrap_or_default();
        assert!(!fields.is_empty(), "{product}");
        for field in &fields {
            assert!(field["label"].as_str().map_or(false, |l| !l.is_empty()) && field["help"].as_str().map_or(false, |h| !h.is_empty()), "{product}: {field}");
        }
    }
}

#[test]
fn validate_rejects_an_unknown_key_for_both_products() {
    let settings = settings_for(&ecosystem());
    let unknown = json!({ "settings": { "definitely_not_a_real_rpp_key": "x" } });
    for product in PRODUCTS {
        let response = call(&settings, product, "settings.validate", Some(unknown.clone()), None);
        assert_eq!((response["ok"].clone(), response["result"]["valid"].clone()), (json!(true), json!(false)), "{product}: {response}");
    }
}

#[test]
fn apply_readback_and_idempotent_replay_for_both_products() {
    let settings = settings_for(&ecosystem());
    for (product, key, value) in [("orchestrator", "default_timeout_seconds", json!(1200)), ("secondbrain", "summary_provider", json!("codex"))] {
        let payload = json!({ "settings": { key: value } });
        let operation_id = unique_id(product);
        let first = call(&settings, product, "settings.apply", Some(payload.clone()), Some(operation_id.clone()));
        assert_eq!(first["ok"], true, "{product}: {first}");
        let readback = call(&settings, product, "settings.readback", None, None);
        assert_eq!(readback["result"]["settings"][key], value, "{product}: {readback}");
        let replay = call(&settings, product, "settings.apply", Some(payload), Some(operation_id));
        assert_eq!(replay["result"], first["result"], "{product}: replay must return the stored receipt");
    }
}

#[test]
fn unknown_op_is_rejected_locally_for_both_products() {
    let settings = settings_for(&ecosystem());
    for product in PRODUCTS {
        let response = call(&settings, product, "definitely_unknown_op", None, None);
        assert_eq!(response["error"]["code"], "INVALID_OP", "{product}: {response}");
    }
}

#[test]
fn missing_engine_root_reports_engine_not_found() {
    let data = Temp::new("missing");
    let missing: PathBuf = data.0.clone();
    let settings = Settings { python: "python".into(), orchestrator_root: missing.join("no-orchestrator"), brain_root: missing.join("no-secondbrain"),
        brain_exe: None, orchestrator_exe: None };
    for product in PRODUCTS {
        assert_eq!(call(&settings, product, "manifest", None, None)["error"]["code"], "ENGINE_NOT_FOUND", "{product}");
    }
}
