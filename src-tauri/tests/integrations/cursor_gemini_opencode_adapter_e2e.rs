//! DISC-004: Cursor, Gemini CLI and OpenCode observations, layering and zero-write behaviour.
use respected_hub_lib::integrations::adapters::{cursor, gemini, opencode};
use respected_hub_lib::integrations::discovery::Environment;
use respected_hub_lib::integrations::model::{AssetKind, ObservedStatus, Owner, Project, Scope, SourceFormat};
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

fn env(home: &Path, vars: &[(&str, &Path)]) -> Environment {
    Environment {
        home: home.to_path_buf(),
        app_data: home.join("roaming"),
        local_app_data: home.join("local"),
        path: Vec::new(),
        path_ext: Vec::new(),
        vars: vars.iter().map(|(key, value)| (key.to_string(), value.to_string_lossy().into_owned())).collect::<BTreeMap<_, _>>(),
    }
}

fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(entries) = fs::read_dir(root) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { visit(&path, files); }
            else { files.insert(path.to_string_lossy().into_owned(), fs::read(&path).unwrap()); }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, &mut files);
    files
}

fn binding<'a>(inventory: &'a respected_hub_lib::integrations::model::Inventory, name: &str) -> Vec<&'a respected_hub_lib::integrations::model::Binding> {
    inventory.bindings.iter().filter(|binding| inventory.assets.iter()
        .any(|asset| asset.key == binding.asset_key && asset.name == name)).collect()
}

fn same_path(left: &Path, right: &str) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

#[test]
fn cursor_parses_ten_observations_project_override_and_nested_skills() {
    let temp = Temp::new("cursor-adapter");
    let home = temp.0.join("home");
    let project_root = temp.0.join(PROJECT);
    let environment = env(&home, &[]);
    write(&home, ".cursor/mcp.json", r#"{
  "mcpServers": {
    "postgres": {"command": "npx", "args": ["pg"]},
    "remote": {"url": "https://example.invalid/mcp?token=sk-live-abc"},
    "shared": {"command": "node", "args": ["global.js"]},
    "respected-brain": {"command": "C:/RespectedBrain/respected-brain.exe", "args": ["mcp"]}
  }
}"#);
    write(&home, ".cursor/skills/notes/SKILL.md", "---\nname: notes\n---\n");
    write(&home, ".cursor/skills/shipping/deploy/SKILL.md", "---\nname: deploy\n---\n");
    let project_mcp = write(&project_root, ".cursor/mcp.json", r#"{
  "mcpServers": {
    "shared": {"command": "node", "args": ["project.js"]},
    "local": {"command": "node", "args": ["local.js"]}
  }
}"#);
    write(&project_root, ".cursor/skills/review/SKILL.md", "---\nname: review\n---\n");
    write(&project_root, ".claude/skills/compat/SKILL.md", "---\nname: compat\n---\n");

    let before = snapshot(&temp.0);
    let inventory = cursor::scan(&environment, &[Project { id: "production".into(), root: project_root.to_string_lossy().into_owned() }]);
    let after = snapshot(&temp.0);

    assert_eq!(inventory.bindings.len(), 10, "{:?}", inventory.bindings);
    assert!(inventory.issues.is_empty(), "{:?}", inventory.issues);
    assert_eq!(before, after, "adapter wrote to the fixture");
    let shared = binding(&inventory, "shared");
    assert_eq!(shared.len(), 2);
    assert!(shared.iter().any(|item| item.scope == Scope::Global));
    assert!(shared.iter().any(|item| item.scope == Scope::Project && item.source.path == project_mcp.to_string_lossy()));
    assert_ne!(shared[0].asset_key, shared[1].asset_key);
    let deploy = inventory.assets.iter().find(|asset| asset.name == "deploy").unwrap();
    assert_eq!(deploy.kind, AssetKind::Skill);
    let compat = binding(&inventory, "compat");
    assert_eq!(compat.len(), 1);
    assert!(compat[0].source.path.replace('\\', "/").ends_with(".claude/skills/compat/SKILL.md"));
    let brain = binding(&inventory, "respected-brain");
    assert_eq!(brain[0].source.owner, Owner::Product { product: "brain".into() });
    let remote = inventory.assets.iter().find(|asset| asset.name == "remote").unwrap();
    assert!(remote.identity.contains("[redacted]") && !remote.identity.contains("sk-live-abc"));
    assert!(inventory.capabilities.iter().any(|item| item.client == respected_hub_lib::integrations::model::ClientId::Cursor
        && item.read && !item.plan && !item.apply && !item.undo));
}

#[test]
fn gemini_merges_user_and_project_settings_with_alias_skills() {
    let temp = Temp::new("gemini-adapter");
    let home = temp.0.join("home");
    let project_root = temp.0.join(PROJECT);
    let environment = env(&home, &[]);
    write(&home, ".gemini/settings.json", r#"{
  "theme": "Default",
  "mcpServers": {
    "postgres": {"command": "npx", "args": ["pg"]},
    "remote": {"url": "https://example.invalid/sse?token=sk-live-abc"},
    "http": {"httpUrl": "https://example.invalid/http"},
    "shared": {"command": "node", "args": ["global.js"]}
  }
}"#);
    write(&home, ".gemini/skills/user-skill/SKILL.md", "---\nname: user-skill\n---\n");
    write(&home, ".agents/skills/alias-skill/SKILL.md", "---\nname: alias-skill\n---\n");
    let project_settings = write(&project_root, ".gemini/settings.json", r#"{
  "mcpServers": {
    "shared": {"command": "node", "args": ["project.js"]},
    "local": {"command": "node", "args": ["local.js"]}
  }
}"#);
    write(&project_root, ".gemini/skills/work/SKILL.md", "---\nname: work\n---\n");
    write(&project_root, ".agents/skills/alias2/SKILL.md", "---\nname: alias2\n---\n");

    let before = snapshot(&temp.0);
    let inventory = gemini::scan(&environment, &[Project { id: "production".into(), root: project_root.to_string_lossy().into_owned() }]);
    let after = snapshot(&temp.0);

    assert_eq!(inventory.bindings.len(), 10, "{:?}", inventory.bindings);
    assert!(inventory.issues.is_empty(), "{:?}", inventory.issues);
    assert_eq!(before, after, "adapter wrote to the fixture");
    let shared = binding(&inventory, "shared");
    assert_eq!(shared.len(), 2);
    assert!(shared.iter().any(|item| item.scope == Scope::Project && item.source.path == project_settings.to_string_lossy()));
    for name in ["remote", "http"] {
        let asset = inventory.assets.iter().find(|asset| asset.name == name).unwrap();
        assert!(!asset.identity.contains("sk-live-abc"));
    }
    assert!(inventory.assets.iter().any(|asset| asset.name == "alias-skill" && asset.kind == AssetKind::Skill));
    assert!(inventory.assets.iter().any(|asset| asset.name == "alias2" && asset.kind == AssetKind::Skill));
}

#[test]
fn invalid_files_name_their_source_without_echoing_content() {
    let cursor_temp = Temp::new("cursor-broken");
    let cursor_home = cursor_temp.0.join("home");
    let cursor_file = write(&cursor_home, ".cursor/mcp.json", r#"{"mcpServers": {"broken": {"command": "sk-live-cursor-secret""#);
    let cursor_inventory = cursor::scan(&env(&cursor_home, &[]), &[]);
    assert!(cursor_inventory.bindings.is_empty());
    let issue = cursor_inventory.issues.iter().find(|issue| issue.code == "invalid_json").unwrap();
    assert_eq!(issue.source_path.as_deref().map(Path::new), Some(cursor_file.as_path()));
    assert!(!issue.message.contains("sk-live-cursor-secret"));

    let gemini_temp = Temp::new("gemini-broken");
    let gemini_home = gemini_temp.0.join("home");
    let gemini_file = write(&gemini_home, ".gemini/settings.json", r#"{"mcpServers": {"broken": "sk-live-gemini-secret"}"#);
    let gemini_inventory = gemini::scan(&env(&gemini_home, &[]), &[]);
    assert!(gemini_inventory.bindings.is_empty());
    let issue = gemini_inventory.issues.iter().find(|issue| issue.code == "invalid_json").unwrap();
    assert_eq!(issue.source_path.as_deref().map(Path::new), Some(gemini_file.as_path()));
    assert!(!issue.message.contains("sk-live-gemini-secret"));

    let opencode_temp = Temp::new("opencode-broken");
    let opencode_home = opencode_temp.0.join("home");
    let opencode_file = write(&opencode_home, ".config/opencode/opencode.jsonc", r#"{ "mcp": { "broken": { "command": ["sk-live-opencode-secret"] } // missing brace"#);
    let opencode_inventory = opencode::scan(&env(&opencode_home, &[]), &[]);
    assert!(opencode_inventory.bindings.is_empty());
    let issue = opencode_inventory.issues.iter().find(|issue| issue.code == "invalid_jsonc").unwrap();
    assert_eq!(issue.source_path.as_deref().map(Path::new), Some(opencode_file.as_path()));
    assert!(!issue.message.contains("sk-live-opencode-secret"));
}

#[test]
fn opencode_reads_jsonc_and_every_documented_layer() {
    let temp = Temp::new("opencode-adapter");
    let home = temp.0.join("home");
    let project_root = temp.0.join(PROJECT);
    let custom = temp.0.join("custom/opencode.json");
    let environment = env(&home, &[("OPENCODE_CONFIG", &custom)]);
    let global = write(&home, ".config/opencode/opencode.jsonc", r#"{
  // local and remote servers can be mixed
  "mcp": {
    "postgres": {"type": "local", "command": ["npx", "pg"]},
    "remote": {"type": "remote", "url": "https://example.invalid/mcp?token=sk-live-abc"},
    "shared": {"type": "local", "command": ["node", "global.js"]},
  },
}"#);
    write(&temp.0, "custom/opencode.json", r#"{"mcp":{"custom-only":{"type":"local","command":["node","custom.js"]},"shared":{"type":"local","command":["node","custom.js"]}}}"#);
    let project_file = write(&project_root, "opencode.jsonc", r#"{
  "mcp": {
    "shared": {"type": "local", "command": ["node", "project.js"], "enabled": false}
  }
}"#);
    write(&home, ".config/opencode/skills/global-skill/SKILL.md", "---\nname: global-skill\n---\n");
    write(&home, ".claude/skills/claude-skill/SKILL.md", "---\nname: claude-skill\n---\n");
    write(&home, ".agents/skills/agent-skill/SKILL.md", "---\nname: agent-skill\n---\n");
    write(&project_root, ".opencode/skills/project-skill/SKILL.md", "---\nname: project-skill\n---\n");
    write(&project_root, ".claude/skills/project-claude/SKILL.md", "---\nname: project-claude\n---\n");
    write(&project_root, ".agents/skills/project-agent/SKILL.md", "---\nname: project-agent\n---\n");

    let before = snapshot(&temp.0);
    let inventory = opencode::scan(&environment, &[Project { id: "production".into(), root: project_root.to_string_lossy().into_owned() }]);
    let after = snapshot(&temp.0);

    assert_eq!(inventory.bindings.len(), 12, "{:?}", inventory.bindings);
    assert!(inventory.issues.is_empty(), "{:?}", inventory.issues);
    assert_eq!(before, after, "adapter wrote to the fixture");
    let shared = binding(&inventory, "shared");
    assert_eq!(shared.len(), 3);
    assert!(shared.iter().any(|item| same_path(&global, &item.source.path)));
    assert!(shared.iter().any(|item| same_path(&custom, &item.source.path)));
    let project_shared = shared.iter().find(|item| same_path(&project_file, &item.source.path)).unwrap();
    assert_eq!(project_shared.scope, Scope::Project);
    assert_eq!(project_shared.enabled, Some(false));
    assert_ne!(project_shared.status, ObservedStatus::Enabled);
    assert_eq!(project_shared.source.format, SourceFormat::Jsonc);
    let remote = inventory.assets.iter().find(|asset| asset.name == "remote").unwrap();
    assert!(remote.identity.contains("[redacted]") && !remote.identity.contains("sk-live-abc"));
    assert!(inventory.assets.iter().any(|asset| asset.name == "project-claude" && asset.kind == AssetKind::Skill));
}

#[test]
fn absent_configs_are_empty_not_errors() {
    let temp = Temp::new("absent-clients");
    let home = temp.0.join("home");
    let environment = env(&home, &[]);
    for inventory in [cursor::scan(&environment, &[]), gemini::scan(&environment, &[]), opencode::scan(&environment, &[])] {
        assert!(inventory.bindings.is_empty());
        assert!(inventory.issues.is_empty(), "{:?}", inventory.issues);
        assert_eq!(inventory.capabilities.len(), 1);
    }
}
