//! Combine the five read-only client adapters into one inventory.
//!
//! A directory or file is only a source observation; the resolver decides what is effective.

use crate::integrations::adapters::{claude, codex, cursor, gemini, opencode};
use crate::integrations::discovery::{self, Environment};
use crate::integrations::model::{Inventory, Project, ScanIssue};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const BASELINE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Baseline {
    version: u32,
    fingerprints: BTreeMap<String, String>,
}

impl Default for Baseline {
    fn default() -> Self { Self { version: BASELINE_VERSION, fingerprints: BTreeMap::new() } }
}

pub fn baseline_file(env: &Environment) -> PathBuf {
    env.app_data.join("com.respected.hub").join("integration-baseline.json")
}

/// Previous scan fingerprints by stable path id. A corrupt or unknown store is
/// ignored: drift analysis must never block a read-only inventory.
pub fn load_baseline(env: &Environment) -> HashMap<String, String> {
    let file = baseline_file(env);
    let Ok(bytes) = fs::read(&file) else { return HashMap::new() };
    match serde_json::from_slice::<Baseline>(&bytes) {
        Ok(store) if store.version == BASELINE_VERSION => store.fingerprints.into_iter().collect(),
        _ => HashMap::new(),
    }
}

pub fn save_baseline(env: &Environment, inventory: &Inventory) -> Result<(), String> {
    let file = baseline_file(env);
    let store = Baseline {
        version: BASELINE_VERSION,
        fingerprints: inventory.bindings.iter()
            .filter_map(|binding| Some((binding.source.path_id.clone(), binding.source.fingerprint.clone()?)))
            .collect(),
    };
    fs::create_dir_all(file.parent().ok_or("Geçersiz baseline yolu")?).map_err(|error| error.to_string())?;
    let temporary = file.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(&store).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    fs::rename(temporary, file).map_err(|error| error.to_string())
}

pub fn scan(env: &Environment, projects: &[Project]) -> Inventory {
    let mut merged = Inventory::default();
    let mut assets = HashSet::new();
    let mut capabilities = HashSet::new();
    let mut issues = HashSet::new();

    for project in projects {
        if !Path::new(&project.root).is_dir() {
            let issue = ScanIssue {
                code: "project_missing".into(),
                message: "Kayıtlı proje klasörü bulunamadı; bu proje taranmadı.".into(),
                source_path: Some(project.root.clone()),
            };
            push_issue(&mut issues, &mut merged.issues, issue);
        }
    }

    for part in [claude::scan(env, projects), codex::scan(env, projects), cursor::scan(env, projects),
        gemini::scan(env, projects), opencode::scan(env, projects)] {
        for asset in part.assets {
            if assets.insert(asset.key.clone()) { merged.assets.push(asset); }
        }
        merged.bindings.extend(part.bindings);
        for capability in part.capabilities {
            if capabilities.insert(capability.client) { merged.capabilities.push(capability); }
        }
        for issue in part.issues { push_issue(&mut issues, &mut merged.issues, issue); }
    }

    let discovery = discovery::scan(env);
    merged.clients = discovery.clients;
    for issue in discovery.issues { push_issue(&mut issues, &mut merged.issues, issue); }
    merged
}

fn push_issue(seen: &mut HashSet<(String, Option<String>)>, out: &mut Vec<ScanIssue>, issue: ScanIssue) {
    if seen.insert((issue.code.clone(), issue.source_path.clone())) { out.push(issue); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::model::{AssetKind, ClientId, Scope};
    use crate::integrations::resolver::{drift, resolve};
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Temp(PathBuf);
    impl Temp {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("hub-inventory-{label}-{}-{nanos}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

    fn env(root: &Path) -> Environment {
        Environment {
            home: root.join("home"), app_data: root.join("roaming"), local_app_data: root.join("local"), path: Vec::new(),
            path_ext: Vec::new(), vars: BTreeMap::new(),
        }
    }

    fn write(path: &Path, body: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    #[test]
    fn same_mcp_in_two_clients_is_one_asset_and_two_bindings() {
        let temp = Temp::new("same-mcp");
        let env = env(&temp.0);
        write(&env.home.join(".claude.json"), r#"{"mcpServers":{"postgres":{"command":"npx","args":["pg"]}}}"#);
        write(&env.home.join(".codex/config.toml"), "[mcp_servers.postgres]\ncommand = \"npx\"\nargs = [\"pg\"]\n");
        let inventory = scan(&env, &[]);
        let mcp: Vec<_> = inventory.assets.iter().filter(|asset| asset.kind == AssetKind::Mcp).collect();
        assert_eq!(mcp.len(), 1);
        assert_eq!(inventory.bindings.iter().filter(|binding| binding.asset_key == mcp[0].key).count(), 2);
        assert_eq!(inventory.clients.len(), 5);
        assert_eq!(resolve(&inventory).entries.len(), 2);
    }

    #[test]
    fn project_scope_is_reported_when_the_caller_supplies_a_project() {
        let temp = Temp::new("project");
        let env = env(&temp.0);
        write(&env.home.join(".claude.json"), r#"{"mcpServers":{"postgres":{"command":"npx","args":["global"]}}}"#);
        let root = temp.0.join("Projeler/Üretim Projesi");
        write(&root.join(".mcp.json"), r#"{"mcpServers":{"postgres":{"command":"npx","args":["project"]}}}"#);
        let project = Project { id: "production".into(), root: root.to_string_lossy().into_owned() };
        let inventory = scan(&env, &[project]);
        assert!(inventory.bindings.iter().any(|binding| binding.client == ClientId::Claude && binding.scope == Scope::Project));
        let report = resolve(&inventory);
        assert!(report.entries.iter().any(|entry| entry.scope == Scope::Project));
        assert!(report.entries.iter().any(|entry| entry.scope == Scope::Global));
    }

    #[test]
    fn external_config_change_between_scans_is_source_changed() {
        let temp = Temp::new("drift");
        let env = env(&temp.0);
        let config = env.home.join(".claude.json");
        write(&config, r#"{"mcpServers":{"postgres":{"command":"npx","args":["pg"]}}}"#);
        let first = scan(&env, &[]);
        save_baseline(&env, &first).unwrap();
        let previous = load_baseline(&env);
        assert!(!previous.is_empty());

        write(&config, r#"{"mcpServers":{"postgres":{"command":"npx","args":["pg","--changed"]}}}"#);
        let second = scan(&env, &[]);
        let findings = drift(&second, &previous);
        assert!(findings.iter().any(|finding| finding.code == crate::integrations::resolver::DriftCode::SourceChanged));
    }
}
