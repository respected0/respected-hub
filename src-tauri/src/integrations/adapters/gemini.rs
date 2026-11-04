//! Gemini CLI read-only adapter.
//!
//! Verified sources:
//! - user settings: `~/.gemini/settings.json` (`mcpServers`)
//! - project settings: `<project>/.gemini/settings.json`; project values override user values
//! - skills: `.gemini/skills` and the documented `.agents/skills` alias

use super::common::{add_skills, entry_owner, issue, json_mcp_identity, read_json, read_only, source};
use crate::integrations::discovery::Environment;
use crate::integrations::model::{
    Asset, AssetKind, Binding, ClientId, Inventory, ObservedStatus, Owner, Project, Scope, SourceFormat,
};
use std::path::{Path, PathBuf};

pub fn scan(env: &Environment, projects: &[Project]) -> Inventory {
    let mut inventory = Inventory::default();
    read_only(&mut inventory, ClientId::Gemini);
    add_mcp(&mut inventory, &env.home.join(".gemini/settings.json"), Scope::Global, None);
    for root in skill_roots(&env.home) {
        add_skills(&mut inventory, ClientId::Gemini, &root, Scope::Global, None, None, Owner::User);
    }
    for project in projects {
        let root = PathBuf::from(&project.root);
        add_mcp(&mut inventory, &root.join(".gemini/settings.json"), Scope::Project, Some(project.clone()));
        for skills in skill_roots(&root) {
            add_skills(&mut inventory, ClientId::Gemini, &skills, Scope::Project, Some(project.clone()), None, Owner::User);
        }
    }
    inventory
}

fn skill_roots(root: &Path) -> [PathBuf; 2] {
    [root.join(".gemini/skills"), root.join(".agents/skills")]
}

fn add_mcp(inventory: &mut Inventory, path: &Path, scope: Scope, project: Option<Project>) {
    let Some((document, fingerprint)) = read_json(inventory, path, SourceFormat::Json) else { return };
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
            issue(inventory, "unsupported_entry", path, "MCP girdisinde command, url veya httpUrl yok.");
            continue;
        };
        let owner = entry_owner(name, &identity);
        let asset = Asset::new(AssetKind::Mcp, name, &identity);
        inventory.add_binding(asset, Binding {
            asset_key: Asset::stable_key(AssetKind::Mcp, name, &identity),
            client: ClientId::Gemini,
            scope,
            project: project.clone(),
            source: source(path, owner, SourceFormat::Json, Some(fingerprint.clone())),
            status: ObservedStatus::Configured,
            enabled: None,
            managed: false,
            parent_asset_key: None,
        });
    }
}
