//! The only IPC surface for third-party integrations. Every request is a closed shape:
//! no filesystem path, command line, URL or plan digest crosses this boundary.

use crate::integrations::catalog;
use crate::integrations::discovery::{self, Environment};
use crate::integrations::model::{Inventory, ScanIssue};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Inventory,
    Clients,
    Catalog,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub action: Action,
}

pub fn execute(request: Request) -> Value { execute_with(request, &Environment::current()) }

pub fn execute_with(request: Request, env: &Environment) -> Value {
    match request.action {
        Action::Inventory => {
            let inventory = Inventory {
                issues: vec![ScanIssue {
                    code: "scan_unavailable".into(),
                    message: "The read-only inventory scanner is not available in this build.".into(),
                    source_path: None,
                }],
                ..Inventory::default()
            };
            json!({ "ok": true, "result": { "read_only": true, "inventory": inventory }, "error": Value::Null })
        }
        Action::Clients => {
            let discovery = discovery::scan(env);
            json!({ "ok": true, "result": { "read_only": true, "clients": discovery.clients, "issues": discovery.issues }, "error": Value::Null })
        }
        Action::Catalog => match catalog::entries() {
            Ok(entries) => json!({ "ok": true, "result": { "read_only": true, "entries": entries }, "error": Value::Null }),
            Err(message) => json!({ "ok": false, "result": Value::Null, "error": { "code": "catalog_invalid", "message": message } }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_closed_actions_are_accepted() {
        assert!(serde_json::from_str::<Request>(r#"{"action":"inventory"}"#).is_ok());
        assert!(serde_json::from_str::<Request>(r#"{"action":"clients"}"#).is_ok());
        assert!(serde_json::from_str::<Request>(r#"{"action":"catalog"}"#).is_ok());
        for body in [
            r#"{"action":"apply"}"#,
            r#"{"action":"undo"}"#,
            r#"{"action":"shell"}"#,
            r#"{"action":"C:\\Windows\\System32\\cmd.exe"}"#,
            r#"{"action":"https://example.invalid"}"#,
            r#"{"action":"inventory","path":"C:\\Users\\person\\.claude.json"}"#,
            r#"{"action":"inventory","command":"npx evil"}"#,
            r#"{"action":"inventory","url":"https://example.invalid"}"#,
            r#"{"action":"inventory","plan_digest":"deadbeef"}"#,
        ] {
            assert!(serde_json::from_str::<Request>(body).is_err(), "accepted {body}");
        }
    }

    #[test]
    fn inventory_is_read_only_and_reports_the_missing_scanner() {
        let reply = execute(Request { action: Action::Inventory });
        assert_eq!(reply["ok"], true);
        assert_eq!(reply["result"]["read_only"], true);
        assert_eq!(reply["result"]["inventory"]["issues"][0]["code"], "scan_unavailable");
        assert!(reply["result"]["inventory"]["assets"].as_array().unwrap().is_empty());
    }

    #[test]
    fn catalog_is_compiled_in_and_needs_no_network() {
        let reply = execute(Request { action: Action::Catalog });
        assert_eq!(reply["ok"], true);
        assert_eq!(reply["result"]["read_only"], true);
        let entries = reply["result"]["entries"].as_array().unwrap();
        assert!(entries.len() >= 50, "only {} catalog entries", entries.len());
        for entry in entries {
            assert!(entry["source_url"].as_str().unwrap().starts_with("https://github.com/"));
        }
    }

    #[test]
    fn clients_are_read_only_and_report_all_five_targets() {
        let root = std::env::temp_dir().join(format!("hub-api-clients-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("home/.claude")).unwrap();
        let env = Environment {
            home: root.join("home"), app_data: root.join("roaming"), local_app_data: root.join("local"),
            path: Vec::new(), path_ext: Vec::new(), vars: std::collections::BTreeMap::new(),
        };
        let reply = execute_with(Request { action: Action::Clients }, &env);
        assert_eq!(reply["result"]["read_only"], true);
        assert_eq!(reply["result"]["clients"].as_array().unwrap().len(), 5);
        assert_eq!(reply["result"]["clients"][0]["status"], "configured");
        assert_eq!(reply["result"]["clients"][0]["configured"], true);
        assert_eq!(reply["result"]["clients"][0]["installed"], false);
        let _ = std::fs::remove_dir_all(&root);
    }
}
