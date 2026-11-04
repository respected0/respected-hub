//! The only IPC surface for third-party integrations. Every request is a closed shape:
//! no filesystem path, command line, URL or plan digest crosses this boundary.

use crate::integrations::catalog;
use crate::integrations::discovery::{self, Environment};
use crate::integrations::{inventory, projects, resolver};
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
        Action::Inventory => match projects::load(env) {
            Ok(projects) => {
                let inventory = inventory::scan(env, &projects);
                let resolution = resolver::resolve(&inventory);
                json!({ "ok": true, "result": { "read_only": true, "inventory": inventory, "resolution": resolution }, "error": Value::Null })
            }
            Err(message) => json!({ "ok": false, "result": Value::Null, "error": { "code": "projects_invalid", "message": message } }),
        },
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
    fn inventory_is_read_only_and_scans_all_five_clients() {
        let root = std::env::temp_dir().join(format!("hub-api-inventory-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("home/.claude")).unwrap();
        let env = Environment {
            home: root.join("home"), app_data: root.join("roaming"), local_app_data: root.join("local"),
            path: Vec::new(), path_ext: Vec::new(), vars: std::collections::BTreeMap::new(),
        };
        let reply = execute_with(Request { action: Action::Inventory }, &env);
        assert_eq!(reply["ok"], true);
        assert_eq!(reply["result"]["read_only"], true);
        assert_eq!(reply["result"]["inventory"]["clients"].as_array().unwrap().len(), 5);
        assert!(reply["result"]["inventory"]["assets"].as_array().unwrap().is_empty());
        assert!(reply["result"]["resolution"]["entries"].as_array().unwrap().is_empty());
        assert!(reply["result"]["inventory"]["issues"].as_array().unwrap().iter().all(|issue| issue["code"] != "scan_unavailable"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn inventory_reads_registered_projects_and_reports_missing_roots() {
        let root = std::env::temp_dir().join(format!("hub-api-projects-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let env = Environment {
            home: root.join("home"), app_data: root.join("roaming"), local_app_data: root.join("local"),
            path: Vec::new(), path_ext: Vec::new(), vars: std::collections::BTreeMap::new(),
        };
        std::fs::create_dir_all(root.join("home/.claude")).unwrap();
        let mut registered = Vec::new();
        for (index, name) in ["alpha", "beta", "gamma"].iter().enumerate() {
            let project_root = root.join("projects").join(name);
            std::fs::create_dir_all(&project_root).unwrap();
            std::fs::write(project_root.join(".mcp.json"), format!(
                r#"{{"mcpServers":{{"{name}":{{"command":"npx","args":["{index}"]}}}}}}"#)).unwrap();
            registered.push(crate::integrations::model::Project {
                id: format!("project:{name}"), root: project_root.to_string_lossy().into_owned(),
            });
        }
        let missing = root.join("projects/moved-away");
        registered.push(crate::integrations::model::Project {
            id: "project:missing".into(), root: missing.to_string_lossy().into_owned(),
        });
        let registry = projects::registry_file(&env);
        std::fs::create_dir_all(registry.parent().unwrap()).unwrap();
        std::fs::write(&registry, serde_json::to_vec(&registered).unwrap()).unwrap();
        let before: Vec<_> = registered[..3].iter()
            .map(|project| std::fs::read(std::path::Path::new(&project.root).join(".mcp.json")).unwrap()).collect();

        let reply = execute_with(Request { action: Action::Inventory }, &env);
        assert_eq!(reply["ok"], true);
        let entries = reply["result"]["resolution"]["entries"].as_array().unwrap();
        for name in ["alpha", "beta", "gamma"] {
            assert!(entries.iter().any(|entry| entry["project"]["id"] == format!("project:{name}")), "{name} missing");
        }
        assert!(reply["result"]["inventory"]["issues"].as_array().unwrap()
            .iter().any(|issue| issue["code"] == "project_missing"));
        let after: Vec<_> = registered[..3].iter()
            .map(|project| std::fs::read(std::path::Path::new(&project.root).join(".mcp.json")).unwrap()).collect();
        assert_eq!(before, after, "read-only inventory changed a project config");
        let _ = std::fs::remove_dir_all(&root);
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
