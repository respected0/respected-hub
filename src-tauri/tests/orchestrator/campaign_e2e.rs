//! Real engine proof for Hub campaign tracking. Every run uses a temporary data
//! root; no user data is touched and no campaign worker is ever started.
use respected_hub_lib::{call_engine, list_campaigns, valid_campaign_id, Settings};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
#[path = "../common/mod.rs"]
mod common;
use common::{ecosystem, Temp};

fn settings_for(ecosystem: &Path, data_root: &Path) -> Settings {
    Settings {
        python: "python".into(),
        orchestrator_root: ecosystem.join("respected-orchestrator"),
        brain_root: ecosystem.join("respected-secondbrain"),
        data_root: data_root.to_path_buf(),
        brain_data_root: Some(data_root.to_path_buf()), brain_exe: None, brain_app_dir: None, orchestrator_exe: None,
    }
}

fn campaign_folder(data_root: &Path, id: &str) -> PathBuf {
    let path = data_root.join("campaigns").join(id);
    std::fs::create_dir_all(&path).expect("create campaign fixture");
    path
}

#[test]
fn campaign_identity_rules_match_the_engine() {
    assert!(valid_campaign_id("camp-20260101000000-aaaaaaaa"));
    assert!(valid_campaign_id("camp-a"));
    assert!(valid_campaign_id(&format!("camp-{}", "a".repeat(60))));
    assert!(!valid_campaign_id("camp-"));
    assert!(!valid_campaign_id("camp-ABC"));
    assert!(!valid_campaign_id("campaign-x"));
    assert!(!valid_campaign_id("camp-x/y"));
    assert!(!valid_campaign_id("camp- x"));
    assert!(!valid_campaign_id(&format!("camp-{}", "a".repeat(61))));
}

#[test]
fn campaign_list_returns_only_valid_directories_newest_first() {
    let data = Temp::new("list");
    let settings = settings_for(&ecosystem(), &data.0);
    campaign_folder(&data.0, "camp-20260101000000-aaaaaaaa");
    campaign_folder(&data.0, "camp-20260202000000-bbbbbbbb");
    // A regular file with a valid-looking name is not a campaign directory.
    std::fs::write(data.0.join("campaigns").join("camp-20260303000000-cccccccc"), b"not a directory").unwrap();
    // Invalid names and hidden files are ignored.
    campaign_folder(&data.0, "not-a-campaign");
    campaign_folder(&data.0, "camp-UPPER");
    std::fs::write(data.0.join("campaigns").join("notes.txt"), b"x").unwrap();

    let listed = list_campaigns(&settings).unwrap();
    assert_eq!(listed, vec![
        "camp-20260202000000-bbbbbbbb".to_string(),
        "camp-20260101000000-aaaaaaaa".to_string(),
    ]);

    let missing = Temp::new("list-missing");
    let empty = list_campaigns(&settings_for(&ecosystem(), &missing.0)).unwrap();
    assert!(empty.is_empty(), "{empty:?}");
}

#[test]
fn campaign_status_reads_the_engine_for_a_fixture_without_a_supervisor() {
    let data = Temp::new("status");
    let settings = settings_for(&ecosystem(), &data.0);
    let id = "camp-20260101000000-aaaaaaaa";
    campaign_folder(&data.0, id);

    let value = call_engine(settings, "campaign-status", Some(id.into()), None).expect("campaign-status should answer");
    assert_eq!(value["campaign_id"], id, "{value}");
    assert_eq!(value["status"], "interrupted", "{value}");
    assert_eq!(value["supervisor_alive"], false, "{value}");
    assert!(value["events"].is_array(), "{value}");
}

#[test]
fn invalid_campaign_identity_is_rejected_before_any_process_spawns() {
    let data = Temp::new("reject");
    let mut settings = settings_for(&ecosystem(), &data.0);
    // A missing executable would surface a spawn error if validation ran first.
    settings.python = "definitely-not-a-real-python".into();

    let error = call_engine(settings, "campaign-status", Some("not-a-campaign".into()), None).unwrap_err();
    assert!(error.contains("Invalid campaign identity"), "{error}");

    let data = Temp::new("reject-body");
    let mut settings = settings_for(&ecosystem(), &data.0);
    settings.python = "definitely-not-a-real-python".into();
    assert!(call_engine(settings, "campaign-start", None, None).unwrap_err().contains("JSON body"));
}

#[test]
fn campaign_start_with_an_invalid_spec_returns_the_engine_validation_error() {
    let data = Temp::new("spec");
    let settings = settings_for(&ecosystem(), &data.0);
    // An empty workstream list fails engine validation long before a worker could start.
    let error = call_engine(settings, "campaign-start", None, Some(json!({ "workstreams": [] }))).unwrap_err();
    assert!(error.contains("workstreams"), "{error}");
    // The fixture root must stay untouched: no campaign directory was created.
    assert!(!data.0.join("campaigns").exists(), "no campaign should have been started");
}

#[test]
fn campaign_start_rejects_a_run_identity() {
    let data = Temp::new("run-id");
    let settings = settings_for(&ecosystem(), &data.0);
    let request: Option<Value> = Some(json!({ "workstreams": [] }));
    assert!(call_engine(settings, "campaign-start", Some("camp-x".into()), request).is_err());
}