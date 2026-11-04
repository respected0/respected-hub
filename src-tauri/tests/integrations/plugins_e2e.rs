//! DISC-006: plugin parent/child ownership, disabled plugins and broken manifests.
use respected_hub_lib::integrations::adapters::claude;
use respected_hub_lib::integrations::discovery::Environment;
use respected_hub_lib::integrations::model::{AssetKind, Inventory};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../common/mod.rs"]
mod common;
use common::Temp;

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

fn plugin<'a>(inventory: &'a Inventory, name: &str) -> &'a respected_hub_lib::integrations::model::Binding {
    let key = inventory.assets.iter().find(|asset| asset.kind == AssetKind::Plugin && asset.name == name).unwrap().key.clone();
    inventory.bindings.iter().find(|binding| binding.asset_key == key).unwrap()
}

fn fixture_plugin(home: &Path, folder: &str, name: &str, enabled: Option<bool>) {
    let root = home.join(".claude/plugins").join(folder);
    let config = match enabled {
        Some(false) => format!(r#"{{"name":"{name}","enabled":false}}"#),
        _ => format!(r#"{{"name":"{name}"}}"#),
    };
    write(&root, ".claude-plugin/plugin.json", &config);
    write(&root, "skills/alpha/SKILL.md", "---\nname: alpha\n---\n");
    write(&root, "skills/beta/SKILL.md", "---\nname: beta\n---\n");
    write(&root, "commands/review.md", "# review\n");
    write(&root, "agents/helper.md", "# helper\n");
    write(&root, "hooks/hooks.json", r#"{"hooks":{}}"#);
    write(&root, ".mcp.json", r#"{"mcpServers":{"db":{"command":"node","args":["db.js"]}}}"#);
}

#[test]
fn plugin_children_keep_their_parent_and_kind() {
    let temp = Temp::new("plugins-children");
    let home = temp.0.join("home");
    fixture_plugin(&home, "demo", "demo", None);
    let inventory = claude::scan(&env(&home), &[]);
    assert!(inventory.issues.is_empty(), "{:?}", inventory.issues);
    let parent = plugin(&inventory, "demo");
    assert_eq!(parent.parent_asset_key, None);
    let children = inventory.children_of(&parent.asset_key);
    assert_eq!(children.len(), 6, "{:?}", children);
    let kinds = children.iter().map(|child| inventory.assets.iter().find(|asset| asset.key == child.asset_key).unwrap().kind).collect::<Vec<_>>();
    for kind in [AssetKind::Skill, AssetKind::Command, AssetKind::Agent, AssetKind::Hook, AssetKind::Mcp] {
        assert!(kinds.contains(&kind), "missing {kind:?}: {kinds:?}");
    }
    assert!(children.iter().all(|child| child.source.path.contains("plugins")));
}

#[test]
fn equal_child_names_in_two_plugins_stay_owned_by_their_plugin() {
    let temp = Temp::new("plugins-siblings");
    let home = temp.0.join("home");
    fixture_plugin(&home, "first", "first", None);
    fixture_plugin(&home, "second", "second", None);
    let inventory = claude::scan(&env(&home), &[]);
    let first = plugin(&inventory, "first").asset_key.clone();
    let second = plugin(&inventory, "second").asset_key.clone();
    let reviews: Vec<_> = inventory.bindings.iter().filter(|binding| inventory.assets.iter().any(|asset| asset.key == binding.asset_key && asset.name == "review")).collect();
    assert_eq!(reviews.len(), 2);
    assert!(reviews.iter().any(|binding| binding.parent_asset_key.as_deref() == Some(&first)));
    assert!(reviews.iter().any(|binding| binding.parent_asset_key.as_deref() == Some(&second)));
    assert_ne!(reviews[0].asset_key, reviews[1].asset_key);
}

#[test]
fn a_disabled_plugin_keeps_files_visible_but_not_enabled() {
    let temp = Temp::new("plugins-disabled");
    let home = temp.0.join("home");
    fixture_plugin(&home, "off", "off", Some(false));
    let inventory = claude::scan(&env(&home), &[]);
    let parent = plugin(&inventory, "off");
    assert_eq!(parent.enabled, Some(false));
    assert_ne!(parent.status, respected_hub_lib::integrations::model::ObservedStatus::Enabled);
    assert_eq!(inventory.children_of(&parent.asset_key).len(), 6);
}

#[test]
fn a_broken_manifest_is_reported_without_inventing_children() {
    let temp = Temp::new("plugins-broken");
    let home = temp.0.join("home");
    write(&home, ".claude/plugins/broken/.claude-plugin/plugin.json", r#"{"name":"broken""#);
    write(&home, ".claude/plugins/broken/commands/review.md", "# review\n");
    let inventory = claude::scan(&env(&home), &[]);
    assert!(inventory.issues.iter().any(|issue| issue.code == "invalid_json"), "{:?}", inventory.issues);
    assert!(inventory.bindings.is_empty());
}
