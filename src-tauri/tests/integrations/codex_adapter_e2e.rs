//! DISC-003: Codex TOML observations, project trust and zero-write behaviour.
use respected_hub_lib::integrations::adapters::{claude, codex};
use respected_hub_lib::integrations::discovery::Environment;
use respected_hub_lib::integrations::model::{AssetKind, Owner, Project, Scope};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../common/mod.rs"]
mod common;
use common::Temp;

const PROJECT: &str = "Projeler/Üretim Projesi";

fn write(root: &Path, relative: &str, body: &str) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

fn env(home: &Path) -> Environment {
    Environment {
        home: home.to_path_buf(),
        app_data: home.join("roaming"),
        local_app_data: home.join("local"),
        path: Vec::new(),
        path_ext: Vec::new(),
        vars: BTreeMap::new(),
    }
}

fn toml_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[test]
fn eleven_observations_match_trust_scopes_and_secret_redaction() {
    let temp = Temp::new("codex-adapter");
    let home = temp.0.join("home");
    let project_root = temp.0.join(PROJECT);
    let environment = env(&home);
    let trusted = toml_path(&project_root);
    write(&home, ".codex/config.toml", &format!(r#"[mcp_servers.postgres]
command = "npx"
args = ["pg"]

[mcp_servers.shared]
command = "node"
args = ["shared.js"]

[mcp_servers.respected-brain]
command = "C:/RespectedBrain/respected-brain.exe"
args = ["mcp"]

[mcp_servers.respected-orchestrator]
command = "C:/RespectedOrchestrator/respected-orchestrator.exe"
args = ["mcp"]

[mcp_servers.remote]
url = "https://example.invalid/mcp?token=sk-live-abc"

[mcp_servers.disabled]
command = "node"
args = ["disabled.js"]
enabled = false

[mcp_servers.leaky]
command = "node"
args = ["--token", "sk-live-abc"]

[projects."{trusted}"]
trust_level = "trusted"
"#));
    let project_config = write(&project_root, ".codex/config.toml", r#"[mcp_servers.local-search]
command = "node"
args = ["search.js"]

[mcp_servers.shared]
command = "node"
args = ["shared.js"]
"#);
    write(&home, ".codex/skills/notes/SKILL.md", "---\nname: notes\n---\n");
    write(&project_root, ".codex/skills/deploy/SKILL.md", "---\nname: deploy\n---\n");

    let projects = vec![Project { id: "production".into(), root: project_root.to_string_lossy().into_owned() }];
    let inventory = codex::scan(&environment, &projects);

    assert_eq!(inventory.bindings.len(), 11, "{:?}", inventory.bindings);
    assert!(inventory.issues.is_empty(), "{:?}", inventory.issues);
    let shared: Vec<_> = inventory.bindings.iter().filter(|binding| inventory.assets.iter()
        .any(|asset| asset.key == binding.asset_key && asset.name == "shared")).collect();
    assert_eq!(shared.len(), 2);
    assert_eq!(shared[0].asset_key, shared[1].asset_key);
    assert!(shared.iter().any(|binding| binding.scope == Scope::Global));
    assert!(shared.iter().any(|binding| binding.scope == Scope::Project && binding.source.path == project_config.to_string_lossy()));

    let brain = inventory.bindings.iter().find(|binding| inventory.assets.iter()
        .any(|asset| asset.key == binding.asset_key && asset.name == "respected-brain")).unwrap();
    assert_eq!(brain.source.owner, Owner::Product { product: "brain".into() });
    let orchestrator = inventory.bindings.iter().find(|binding| inventory.assets.iter()
        .any(|asset| asset.key == binding.asset_key && asset.name == "respected-orchestrator")).unwrap();
    assert_eq!(orchestrator.source.owner, Owner::Product { product: "orchestrator".into() });

    let disabled = inventory.assets.iter().find(|asset| asset.name == "disabled").unwrap();
    let disabled_binding = inventory.bindings.iter().find(|binding| binding.asset_key == disabled.key).unwrap();
    assert_eq!(disabled_binding.enabled, Some(false));
    assert_ne!(disabled_binding.status, respected_hub_lib::integrations::model::ObservedStatus::Enabled);
    for name in ["remote", "leaky"] {
        let asset = inventory.assets.iter().find(|asset| asset.name == name).unwrap();
        assert!(!asset.identity.contains("sk-live-abc"), "secret leaked from {name}: {}", asset.identity);
        assert!(asset.identity.contains("[redacted]"));
    }
    assert!(inventory.bindings.iter().all(|binding| binding.source.fingerprint.is_some()));
    assert!(inventory.bindings.iter().filter(|binding| binding.parent_asset_key.is_some()).all(|binding| inventory.assets.iter()
        .any(|asset| asset.kind == AssetKind::Skill && asset.key == binding.asset_key)));
}

#[test]
fn an_untrusted_project_is_reported_and_never_read() {
    let temp = Temp::new("codex-untrusted");
    let home = temp.0.join("home");
    let project_root = temp.0.join(PROJECT);
    let environment = env(&home);
    let path = toml_path(&project_root);
    write(&home, ".codex/config.toml", &format!("[projects.\"{path}\"]\ntrust_level = \"untrusted\"\n"));
    let project_config = write(&project_root, ".codex/config.toml", "[mcp_servers.hidden]\ncommand = \"node\"\n");
    write(&project_root, ".codex/skills/hidden/SKILL.md", "---\nname: hidden\n---\n");

    let inventory = codex::scan(&environment, &[Project { id: "untrusted".into(), root: project_root.to_string_lossy().into_owned() }]);

    assert!(inventory.bindings.is_empty());
    assert!(inventory.issues.iter().any(|issue| issue.code == "project_untrusted"
        && issue.source_path.as_deref() == Some(project_config.to_string_lossy().as_ref())));
}

#[test]
fn invalid_toml_names_its_file_without_echoing_content() {
    let temp = Temp::new("codex-broken");
    let home = temp.0.join("home");
    let path = write(&home, ".codex/config.toml", "[mcp_servers.broken\ncommand = \"sk-live-secret\"\n");
    let inventory = codex::scan(&env(&home), &[]);
    assert!(inventory.bindings.is_empty());
    let issue = inventory.issues.iter().find(|issue| issue.code == "invalid_toml").unwrap();
    assert_eq!(issue.source_path.as_deref().map(Path::new), Some(path.as_path()));
    assert!(!issue.message.contains("sk-live-secret"));
}

#[test]
fn the_same_mcp_in_two_clients_keeps_one_asset_key() {
    let temp = Temp::new("codex-cross-client");
    let home = temp.0.join("home");
    let environment = env(&home);
    write(&home, ".claude.json", r#"{"mcpServers":{"postgres":{"command":"npx","args":["pg"]}}}"#);
    write(&home, ".codex/config.toml", "[mcp_servers.postgres]\ncommand = \"npx\"\nargs = [\"pg\"]\n");

    let claude_inventory = claude::scan(&environment, &[]);
    let codex_inventory = codex::scan(&environment, &[]);
    let claude_key = claude_inventory.assets.iter().find(|asset| asset.name == "postgres").unwrap().key.clone();
    let codex_key = codex_inventory.assets.iter().find(|asset| asset.name == "postgres").unwrap().key.clone();
    assert_eq!(claude_key, codex_key);
}
