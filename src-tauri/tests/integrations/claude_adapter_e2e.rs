//! DISC-002: Claude Code observations, source ownership and zero-write behaviour.
use respected_hub_lib::integrations::adapters::claude;
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

fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(entries) = fs::read_dir(root) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, files);
            } else {
                files.insert(path.to_string_lossy().into_owned(), fs::read(&path).unwrap());
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, &mut files);
    files
}

#[test]
fn thirteen_observations_match_bindings_sources_and_ownership() {
    let temp = Temp::new("claude-adapter");
    let home = temp.0.join("home");
    let project_root = temp.0.join(PROJECT);
    let environment = env(&home);
    write(&home, ".claude.json", r#"{
  "theme": "dark",
  "mcpServers": {
    "postgres": {"command": "npx", "args": ["pg"]},
    "shared": {"command": "node", "args": ["shared.js"]},
    "respected-brain": {"command": "C:/RespectedBrain/respected-brain.exe", "args": ["mcp"]},
    "respected-orchestrator": {"command": "C:/RespectedOrchestrator/respected-orchestrator.exe", "args": ["mcp"]},
    "remote": {"url": "https://example.invalid/mcp?token=sk-live-abc"}
  }
}"#);
    let project_mcp = write(&project_root, ".mcp.json", r#"{"mcpServers": {
  "local-search": {"command": "node", "args": ["search.js"]},
  "shared": {"command": "node", "args": ["shared.js"]},
  "postgres-project": {"command": "npx", "args": ["pg-project"]}
}}"#);
    write(&home, ".claude/skills/notes/SKILL.md", "---\nname: notes\n---\n");
    write(&project_root, ".claude/skills/deploy/SKILL.md", "---\nname: deploy\n---\n");
    write(&home, ".claude/plugins/demo/.claude-plugin/plugin.json", r#"{"name":"demo","mcpServers":"./.mcp.json","skills":"./skills"}"#);
    write(&home, ".claude/plugins/demo/.mcp.json", r#"{"mcpServers":{"db":{"command":"node","args":["db.js"]}}}"#);
    write(&home, ".claude/plugins/demo/skills/helper/SKILL.md", "---\nname: helper\n---\n");

    let before = snapshot(&temp.0);
    let projects = vec![Project { id: "production".into(), root: project_root.to_string_lossy().into_owned() }];
    let inventory = claude::scan(&environment, &projects);
    let after = snapshot(&temp.0);

    assert_eq!(before, after, "the adapter wrote to a Claude source");
    assert_eq!(inventory.bindings.len(), 13, "{:?}", inventory.bindings);
    assert_eq!(inventory.issues, Vec::new());

    let shared: Vec<_> = inventory.bindings.iter().filter(|binding| inventory.assets.iter()
        .any(|asset| asset.key == binding.asset_key && asset.name == "shared")).collect();
    assert_eq!(shared.len(), 2);
    assert_eq!(shared[0].asset_key, shared[1].asset_key);
    assert!(shared.iter().any(|binding| binding.scope == Scope::Global && binding.source.path.ends_with(".claude.json")));
    assert!(shared.iter().any(|binding| binding.scope == Scope::Project && binding.source.path == project_mcp.to_string_lossy()));

    let brain = inventory.bindings.iter().find(|binding| binding.source.path.ends_with(".claude.json")
        && inventory.assets.iter().any(|asset| asset.key == binding.asset_key && asset.name == "respected-brain")).unwrap();
    assert_eq!(brain.source.owner, Owner::Product { product: "brain".into() });
    let orchestrator = inventory.bindings.iter().find(|binding| inventory.assets.iter()
        .any(|asset| asset.key == binding.asset_key && asset.name == "respected-orchestrator")).unwrap();
    assert_eq!(orchestrator.source.owner, Owner::Product { product: "orchestrator".into() });

    let plugin = inventory.bindings.iter().find(|binding| inventory.assets.iter()
        .any(|asset| asset.key == binding.asset_key && asset.kind == AssetKind::Plugin)).unwrap();
    let children = inventory.children_of(&plugin.asset_key);
    assert_eq!(children.len(), 2, "plugin MCP and Skill children missing");
    assert!(children.iter().any(|binding| inventory.assets.iter().any(|asset| asset.key == binding.asset_key && asset.kind == AssetKind::Mcp)));
    assert!(children.iter().any(|binding| inventory.assets.iter().any(|asset| asset.key == binding.asset_key && asset.kind == AssetKind::Skill)));

    let remote = inventory.assets.iter().find(|asset| asset.name == "remote").unwrap();
    assert!(!remote.identity.contains("sk-live-abc"), "secret leaked into the asset identity");
    assert!(remote.identity.contains("[redacted]"));
    assert!(inventory.bindings.iter().all(|binding| binding.source.fingerprint.is_some()));
    assert!(inventory.bindings.iter().all(|binding| binding.status != respected_hub_lib::integrations::model::ObservedStatus::Enabled));
}

#[test]
fn commented_and_malformed_files_report_their_exact_source() {
    let temp = Temp::new("claude-adapter-broken");
    let home = temp.0.join("home");
    let project_root = temp.0.join(PROJECT);
    let environment = env(&home);
    let global = write(&home, ".claude.json", r#"{"mcpServers": {"postgres": {"command": "npx", "args": ["#);
    let project = write(&project_root, ".mcp.json", "// Claude does not accept comments here\n{\"mcpServers\":{}}\n");
    let projects = vec![Project { id: "broken".into(), root: project_root.to_string_lossy().into_owned() }];

    let inventory = claude::scan(&environment, &projects);

    assert!(inventory.bindings.is_empty());
    for (code, path) in [("invalid_json", &global), ("invalid_json", &project)] {
        assert!(inventory.issues.iter().any(|issue| issue.code == code
            && issue.source_path.as_deref() == Some(path.to_string_lossy().as_ref())), "{:?}", inventory.issues);
    }
    assert!(inventory.issues.iter().all(|issue| !issue.message.contains("sk-live") && !issue.message.contains("postgres")));
}

#[test]
fn same_name_with_different_commands_stays_two_assets() {
    let temp = Temp::new("claude-adapter-collision");
    let home = temp.0.join("home");
    let project_root = temp.0.join(PROJECT);
    let environment = env(&home);
    write(&home, ".claude.json", r#"{"mcpServers":{"db":{"command":"node","args":["global.js"]}}}"#);
    write(&project_root, ".mcp.json", r#"{"mcpServers":{"db":{"command":"node","args":["project.js"]}}}"#);

    let inventory = claude::scan(&environment, &[Project { id: "collision".into(), root: project_root.to_string_lossy().into_owned() }]);

    let names: Vec<_> = inventory.assets.iter().filter(|asset| asset.name == "db").collect();
    assert_eq!(names.len(), 2);
    assert_ne!(names[0].key, names[1].key);
    assert_eq!(inventory.bindings.len(), 2);
}

#[test]
fn installed_plugin_state_comes_from_the_enabled_plugins_map() {
    let temp = Temp::new("claude-plugin-enabled");
    let home = temp.0.join("home");
    let environment = env(&home);
    write(&home, ".claude/settings.json", r#"{"enabledPlugins":{
  "demo@demo-market": true,
  "off@demo-market": false
}}"#);
    write(&home, ".claude/plugins/cache/demo-market/demo/1.0.0/.claude-plugin/plugin.json", r#"{"name":"demo","enabled":false}"#);
    write(&home, ".claude/plugins/cache/demo-market/off/2.0.0/.claude-plugin/plugin.json", r#"{"name":"off"}"#);
    write(&home, ".claude/plugins/cache/demo-market/missing/3.0.0/.claude-plugin/plugin.json", r#"{"name":"missing","enabled":true}"#);

    let inventory = claude::scan(&environment, &[]);

    let enabled = |name: &str| inventory.bindings.iter().find(|binding| inventory.assets.iter()
        .any(|asset| asset.key == binding.asset_key && asset.name == name)).and_then(|binding| binding.enabled);
    assert_eq!(enabled("demo"), Some(true), "the settings map must override the manifest hint");
    assert_eq!(enabled("off"), Some(false));
    assert_eq!(enabled("missing"), Some(false), "a cache plugin absent from the map is not loaded");
}

#[test]
fn marketplace_clones_are_not_reported_as_installed_plugins() {
    let temp = Temp::new("claude-plugin-marketplace");
    let home = temp.0.join("home");
    let environment = env(&home);
    write(&home, ".claude/plugins/cache/market/demo/1.0.0/.claude-plugin/plugin.json", r#"{"name":"demo"}"#);
    write(&home, ".claude/plugins/marketplaces/market/plugins/other/.claude-plugin/plugin.json", r#"{"name":"other"}"#);

    let inventory = claude::scan(&environment, &[]);

    let plugins: Vec<_> = inventory.assets.iter().filter(|asset| asset.kind == AssetKind::Plugin).map(|asset| asset.name.as_str()).collect();
    assert_eq!(plugins, vec!["demo"]);
}
