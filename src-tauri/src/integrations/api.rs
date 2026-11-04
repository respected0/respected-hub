//! The only IPC surface for third-party integrations. Every request is a closed shape:
//! no filesystem path, command line, URL or plan digest crosses this boundary.

use crate::integrations::model::{Inventory, ScanIssue};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Inventory,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub action: Action,
}

pub fn execute(request: Request) -> Value {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_inventory_action_is_accepted() {
        assert!(serde_json::from_str::<Request>(r#"{"action":"inventory"}"#).is_ok());
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
}
