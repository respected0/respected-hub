//! OpenCode read-only adapter.
//!
//! Verified sources, lowest to highest precedence:
//! - legacy global `~/.local/share/opencode/opencode.json(c)`
//! - global `$XDG_CONFIG_HOME/opencode/opencode.json(c)` (default `~/.config/opencode`)
//! - `$OPENCODE_CONFIG` custom file
//! - project `<project>/opencode.json(c)`
//! - skills: `.opencode/skills`, `.claude/skills`, `.agents/skills`

use super::common::{add_skills, entry_owner, issue, read_json, read_only, source};
use crate::integrations::discovery::Environment;
use crate::integrations::model::{
    Asset, AssetKind, Binding, ClientId, Inventory, ObservedStatus, Owner, Project, Scope, SourceFormat,
};
use crate::integrations::security;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn scan(env: &Environment, projects: &[Project]) -> Inventory {
    let mut inventory = Inventory::default();
    read_only(&mut inventory, ClientId::OpenCode, true, true, None);
    let config_home = env.vars.get("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| env.home.join(".config"));
    for (path, scope, project) in global_layers(env, &config_home) {
        add_mcp(&mut inventory, &path, scope, project);
    }
    for root in skill_roots(&config_home.join("opencode"), &env.home, &env.vars) {
        add_skills(&mut inventory, ClientId::OpenCode, &root, Scope::Global, None, None, Owner::User);
    }
    for project in projects {
        let root = PathBuf::from(&project.root);
        for name in ["opencode.json", "opencode.jsonc"] {
            add_mcp(&mut inventory, &root.join(name), Scope::Project, Some(project.clone()));
        }
        for skills in skill_roots(&root.join(".opencode"), &root, &env.vars) {
            add_skills(&mut inventory, ClientId::OpenCode, &skills, Scope::Project, Some(project.clone()), None, Owner::User);
        }
    }
    inventory
}

fn global_layers(env: &Environment, config_home: &Path) -> Vec<(PathBuf, Scope, Option<Project>)> {
    let mut layers = Vec::new();
    let legacy = env.home.join(".local/share/opencode");
    for name in ["opencode.json", "opencode.jsonc"] {
        layers.push((legacy.join(name), Scope::Global, None));
    }
    let global = config_home.join("opencode");
    for name in ["opencode.json", "opencode.jsonc"] {
        layers.push((global.join(name), Scope::Global, None));
    }
    if let Some(custom) = env.vars.get("OPENCODE_CONFIG") {
        layers.push((PathBuf::from(custom), Scope::Global, None));
    }
    layers
}

fn skill_roots(config_root: &Path, root: &Path, vars: &std::collections::BTreeMap<String, String>) -> Vec<PathBuf> {
    let mut roots = vec![config_root.join("skills")];
    if !truthy_vars(vars) {
        roots.push(root.join(".claude/skills"));
    }
    roots.push(root.join(".agents/skills"));
    roots
}

fn truthy_vars(vars: &std::collections::BTreeMap<String, String>) -> bool {
    vars.get("OPENCODE_DISABLE_CLAUDE_CODE_SKILLS").map(|value| {
        matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes")
    }).unwrap_or(false)
}

fn format(path: &Path) -> SourceFormat {
    if path.extension().map(|ext| ext.eq_ignore_ascii_case("jsonc")).unwrap_or(false) { SourceFormat::Jsonc } else { SourceFormat::Json }
}

fn add_mcp(inventory: &mut Inventory, path: &Path, scope: Scope, project: Option<Project>) {
    let Some((document, fingerprint)) = read_json(inventory, path, format(path)) else { return };
    let Some(servers) = document.get("mcp") else { return };
    let Some(servers) = servers.as_object() else {
        issue(inventory, "unsupported_config", path, "mcp bir nesne olmalı.");
        return;
    };
    for (name, server) in servers {
        let Some(server) = server.as_object() else {
            issue(inventory, "unsupported_entry", path, "MCP girdisi bir nesne olmalı.");
            continue;
        };
        let Some(identity) = identity(server) else {
            issue(inventory, "unsupported_entry", path, "MCP girdisinde command veya url yok.");
            continue;
        };
        let owner = entry_owner(name, &identity);
        let asset = Asset::new(AssetKind::Mcp, name, &identity);
        inventory.add_binding(asset, Binding {
            asset_key: Asset::stable_key(AssetKind::Mcp, name, &identity),
            client: ClientId::OpenCode,
            scope,
            project: project.clone(),
            source: source(path, owner, format(path), Some(fingerprint.clone())),
            status: ObservedStatus::Configured,
            enabled: server.get("enabled").and_then(Value::as_bool),
            managed: false,
            parent_asset_key: None,
        });
    }
}

fn identity(server: &serde_json::Map<String, Value>) -> Option<String> {
    match server.get("command") {
        Some(Value::Array(items)) => {
            let command = items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" ");
            if command.is_empty() { return None; }
            Some(security::redact_text(&format!("command: {command}")))
        }
        Some(Value::String(command)) => Some(security::redact_text(&format!("command: {command}"))),
        _ => server.get("url").and_then(Value::as_str).map(|url| security::redact_text(&format!("url: {url}"))),
    }
}
