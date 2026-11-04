//! Cursor read-only adapter.
//!
//! Verified sources:
//! - user MCP: `~/.cursor/mcp.json` (`mcpServers`)
//! - project MCP: `<project>/.cursor/mcp.json`; the project entry wins for the same name
//! - skills: `.cursor/skills`, `.agents/skills` and the documented Claude/Codex compatibility roots

use super::common::{add_skills_recursive, entry_owner, issue, json_mcp_identity, read_json, read_only, source};
use crate::integrations::discovery::Environment;
use crate::integrations::model::{
    Asset, AssetKind, Binding, ClientId, Inventory, ObservedStatus, Owner, Project, Scope, SourceFormat,
};
use std::path::{Path, PathBuf};

pub fn scan(env: &Environment, projects: &[Project]) -> Inventory {
    let mut inventory = Inventory::default();
    read_only(&mut inventory, ClientId::Cursor, false, false, Some("Cursor toggle durumunu uygulama içinde tutar; yapılandırma dosyasında belgelenmiş alan yok ve değişiklik yeniden başlatma gerektirir."));
    add_mcp(&mut inventory, &env.home.join(".cursor/mcp.json"), Scope::Global, None);
    for root in skill_roots(&env.home) {
        add_skills_recursive(&mut inventory, ClientId::Cursor, &root, Scope::Global, None, Owner::User);
    }
    for project in projects {
        let root = PathBuf::from(&project.root);
        add_mcp(&mut inventory, &root.join(".cursor/mcp.json"), Scope::Project, Some(project.clone()));
        for skills in skill_roots(&root) {
            add_skills_recursive(&mut inventory, ClientId::Cursor, &skills, Scope::Project, Some(project.clone()), Owner::User);
        }
    }
    inventory
}

fn skill_roots(root: &Path) -> [PathBuf; 4] {
    [root.join(".cursor/skills"), root.join(".agents/skills"), root.join(".claude/skills"), root.join(".codex/skills")]
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
            issue(inventory, "unsupported_entry", path, "MCP girdisinde command veya url yok.");
            continue;
        };
        let owner = entry_owner(name, &identity);
        let asset = Asset::new(AssetKind::Mcp, name, &identity);
        inventory.add_binding(asset, Binding {
            asset_key: Asset::stable_key(AssetKind::Mcp, name, &identity),
            client: ClientId::Cursor,
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
