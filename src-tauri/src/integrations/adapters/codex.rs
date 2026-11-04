//! Codex read-only adapter.
//!
//! Verified sources:
//! - user config: `$CODEX_HOME/config.toml` or `~/.codex/config.toml` (`[mcp_servers.<name>]`)
//! - user skills: `$CODEX_HOME/skills/<name>/SKILL.md`
//! - project config: `<project>/.codex/config.toml`, only when `[projects."<path>"]` is trusted
//! - project skills: `<project>/.codex/skills/<name>/SKILL.md`

use super::common::{add_skills, digest, entry_owner, issue, source};
use crate::integrations::discovery::Environment;
use crate::integrations::model::{
    Asset, AssetKind, Binding, ClientId, Inventory, ObservedStatus, Owner, Project, Scope, SourceFormat,
};
use crate::integrations::security::{self, Limits};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use toml::Value;

pub fn scan(env: &Environment, projects: &[Project]) -> Inventory {
    let mut inventory = Inventory::default();
    let root = env.vars.get("CODEX_HOME").map(PathBuf::from).unwrap_or_else(|| env.home.join(".codex"));
    let global_path = root.join("config.toml");
    let global = read_toml(&mut inventory, &global_path);
    if let Some((document, fingerprint)) = &global {
        add_mcp_document(&mut inventory, &global_path, document, fingerprint, Scope::Global, None);
    }
    add_skills(&mut inventory, ClientId::Codex, &root.join("skills"), Scope::Global, None, None, Owner::User);

    for project in projects {
        let project_root = PathBuf::from(&project.root);
        let config_path = project_root.join(".codex/config.toml");
        if !trusted(global.as_ref().map(|(document, _)| document), &project_root) {
            if config_path.is_file() {
                issue(&mut inventory, "project_untrusted", &config_path, "Codex projesi güvenilir olarak işaretlenmemiş.");
            }
            continue;
        }
        if let Some((document, fingerprint)) = read_toml(&mut inventory, &config_path) {
            add_mcp_document(&mut inventory, &config_path, &document, &fingerprint, Scope::Project, Some(project.clone()));
        }
        add_skills(&mut inventory, ClientId::Codex, &project_root.join(".codex/skills"), Scope::Project, Some(project.clone()), None, Owner::User);
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
) {
    let Some(servers) = document.get("mcp_servers") else { return };
    let Some(servers) = servers.as_table() else {
        issue(inventory, "unsupported_config", path, "mcp_servers bir tablo olmalı.");
        return;
    };
    for (name, server) in servers {
        let Some(server) = server.as_table() else {
            issue(inventory, "unsupported_entry", path, "MCP girdisi bir tablo olmalı.");
            continue;
        };
        let Some(identity) = mcp_identity(server) else {
            issue(inventory, "unsupported_entry", path, "MCP girdisinde command veya url yok.");
            continue;
        };
        let owner = entry_owner(name, &identity);
        let asset = Asset::new(AssetKind::Mcp, name, &identity);
        let source = source(path, owner, SourceFormat::Toml, Some(fingerprint.to_string()));
        inventory.add_binding(asset, Binding {
            asset_key: Asset::stable_key(AssetKind::Mcp, name, &identity),
            client: ClientId::Codex,
            scope,
            project: project.clone(),
            source,
            status: ObservedStatus::Configured,
            enabled: server.get("enabled").and_then(Value::as_bool),
            managed: false,
            parent_asset_key: None,
        });
    }
}

fn mcp_identity(server: &toml::map::Map<String, Value>) -> Option<String> {
    if let Some(command) = server.get("command").and_then(Value::as_str) {
        let args = server.get("args").and_then(Value::as_array).map(|items| {
            items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" ")
        }).unwrap_or_default();
        let identity = if args.is_empty() { format!("command: {command}") } else { format!("command: {command} {args}") };
        return Some(security::redact_text(&identity));
    }
    server.get("url").and_then(Value::as_str).map(|url| security::redact_text(&format!("url: {url}")))
}

fn trusted(config: Option<&Value>, project_root: &Path) -> bool {
    let Some(projects) = config.and_then(|config| config.get("projects")).and_then(Value::as_table) else { return false };
    let wanted = normalize(project_root);
    projects.iter().any(|(path, entry)| {
        normalize(Path::new(path)) == wanted
            && entry.get("trust_level").and_then(Value::as_str) == Some("trusted")
    })
}

fn normalize(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    let value = value.trim_end_matches('/').to_string();
    if cfg!(windows) { value.to_lowercase() } else { value }
}

fn read_toml(inventory: &mut Inventory, path: &Path) -> Option<(Value, String)> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return None,
        Err(error) => {
            issue(inventory, "unreadable", path, &error.to_string());
            return None;
        }
    };
    if metadata.file_type().is_symlink() && fs::metadata(path).is_err() {
        issue(inventory, "broken_link", path, "Bağlantı hedefi okunamadı.");
        return None;
    }
    let text = match security::read_text(path, Limits::default()) {
        Ok(text) => text,
        Err(error) => {
            inventory.issues.push(error);
            return None;
        }
    };
    let body = text.strip_prefix('\u{feff}').unwrap_or(&text);
    match body.parse::<Value>() {
        Ok(document) => Some((document, digest(&text))),
        Err(_) => {
            issue(inventory, "invalid_toml", path, "Dosya geçerli TOML değil.");
            None
        }
    }
}
