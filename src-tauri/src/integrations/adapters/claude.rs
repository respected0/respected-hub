//! Claude Code read-only adapter.
//!
//! Verified sources:
//! - user MCP: `~/.claude.json` (`mcpServers`)
//! - project MCP: `<project>/.mcp.json` (`mcpServers`)
//! - personal skills: `~/.claude/skills/<name>/SKILL.md`
//! - project skills: `<project>/.claude/skills/<name>/SKILL.md`
//! - plugins: `~/.claude/plugins/**/.claude-plugin/plugin.json`

use super::common::{add_skills, entry_owner, issue, json_mcp_identity, read_json, source};
use crate::integrations::discovery::Environment;
use crate::integrations::model::{
    Asset, AssetKind, Binding, ClientId, Inventory, ObservedStatus, Owner, Project, Scope, Source, SourceFormat,
};
use crate::integrations::security::{self, Limits};
use crate::integrations::plugins;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const PLUGIN_DEPTH: usize = 8;

/// Read every verified Claude Code source for the user and the explicitly supplied projects.
/// No project is discovered by walking the disk.
pub fn scan(env: &Environment, projects: &[Project]) -> Inventory {
    let mut inventory = Inventory::default();
    let config = env.vars.get("CLAUDE_CONFIG_DIR").map(PathBuf::from).unwrap_or_else(|| env.home.join(".claude"));
    let user_mcp = if env.vars.contains_key("CLAUDE_CONFIG_DIR") { config.join(".claude.json") } else { env.home.join(".claude.json") };

    if let Some((document, fingerprint)) = read_json(&mut inventory, &user_mcp, SourceFormat::Json) {
        add_mcp_document(&mut inventory, &user_mcp, &document, &fingerprint, Scope::Global, None, None);
    }
    add_skills(&mut inventory, ClientId::Claude, &config.join("skills"), Scope::Global, None, None, Owner::User);
    add_plugins(&mut inventory, &config.join("plugins"));

    for project in projects {
        let root = PathBuf::from(&project.root);
        let path = root.join(".mcp.json");
        if let Some((document, fingerprint)) = read_json(&mut inventory, &path, SourceFormat::Json) {
            add_mcp_document(&mut inventory, &path, &document, &fingerprint, Scope::Project, Some(project.clone()), None);
        }
        add_skills(&mut inventory, ClientId::Claude, &root.join(".claude/skills"), Scope::Project, Some(project.clone()), None, Owner::User);
    }
    inventory
}

fn add_mcp_document(
    inventory: &mut Inventory,
    path: &Path,
    document: &Value,
    fingerprint: &str,
    scope: Scope,
    project: Option<Project>,
    parent: Option<String>,
) {
    let Some(servers) = document.get("mcpServers") else { return };
    let Some(servers) = servers.as_object() else {
        issue(inventory, "unsupported_config", path, "mcpServers bir nesne olmalı.");
        return;
    };
    for (name, server) in servers {
        let Some(server) = server.as_object() else {
            issue(inventory, "unsupported_entry", path, "MCP girdisi bir nesne olmalı.");
            continue;
        };
        let Some(identity) = json_mcp_identity(server) else {
            issue(inventory, "unsupported_entry", path, "MCP girdisinde command veya url yok.");
            continue;
        };
        let owner = entry_owner(name, &identity);
        let asset = Asset::new(AssetKind::Mcp, name, &identity);
        let source = source(path, owner, SourceFormat::Json, Some(fingerprint.to_string()));
        inventory.add_binding(asset, Binding {
            asset_key: Asset::stable_key(AssetKind::Mcp, name, &identity),
            client: ClientId::Claude,
            scope,
            project: project.clone(),
            source,
            status: ObservedStatus::Configured,
            enabled: None,
            managed: false,
            parent_asset_key: parent.clone(),
        });
    }
}

fn add_plugins(inventory: &mut Inventory, root: &Path) {
    if !fs::metadata(root).map(|metadata| metadata.is_dir()).unwrap_or(false) { return; }
    let walk = security::walk_files(root, Limits { max_depth: PLUGIN_DEPTH, ..Limits::default() });
    inventory.issues.extend(walk.issues);
    let mut seen = HashSet::new();
    for file in walk.files {
        let is_manifest = file.file_name().map(|name| name == "plugin.json").unwrap_or(false)
            && file.parent().and_then(Path::file_name).map(|name| name == ".claude-plugin").unwrap_or(false);
        let Some(plugin_root) = file.parent().and_then(Path::parent) else { continue };
        if is_manifest && seen.insert(Source::from_path(plugin_root, Owner::User, SourceFormat::Manifest).path_id) {
            add_plugin(inventory, &file, plugin_root);
        }
    }
}

fn add_plugin(inventory: &mut Inventory, manifest_path: &Path, root: &Path) {
    let Some((manifest, fingerprint)) = read_json(inventory, manifest_path, SourceFormat::Json) else { return };
    let name = manifest.get("name").and_then(Value::as_str).filter(|name| !name.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| root.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| "plugin".into()));
    let source = source(manifest_path, Owner::User, SourceFormat::Manifest, Some(fingerprint.clone()));
    let identity = format!("plugin:{name}:{}", source.path_id);
    let asset = Asset::new(AssetKind::Plugin, &name, &identity);
    let parent = asset.key.clone();
    // shortcut: the manifest `enabled` hint is read locally; Claude's enabledPlugins map is M3 work.
    let enabled = manifest.get("enabled").and_then(Value::as_bool);
    inventory.add_binding(asset, Binding {
        asset_key: parent.clone(),
        client: ClientId::Claude,
        scope: Scope::Global,
        project: None,
        source,
        status: ObservedStatus::Configured,
        enabled,
        managed: false,
        parent_asset_key: None,
    });

    match manifest.get("mcpServers") {
        Some(Value::String(relative)) => {
            let path = root.join(relative);
            if let Some((document, file_fingerprint)) = read_json(inventory, &path, SourceFormat::Json) {
                add_mcp_document(inventory, &path, &document, &file_fingerprint, Scope::Global, None, Some(parent.clone()));
            }
        }
        Some(Value::Object(_)) => {
            add_mcp_document(inventory, manifest_path, &manifest, &fingerprint, Scope::Global, None, Some(parent.clone()));
        }
        Some(_) => issue(inventory, "unsupported_config", manifest_path, "mcpServers bir yol veya nesne olmalı."),
        None => {
            let path = root.join(".mcp.json");
            if let Some((document, file_fingerprint)) = read_json(inventory, &path, SourceFormat::Json) {
                add_mcp_document(inventory, &path, &document, &file_fingerprint, Scope::Global, None, Some(parent.clone()));
            }
        }
    }

    plugins::add_children(inventory, ClientId::Claude, &parent, root, Scope::Global, None, Owner::User);
}
