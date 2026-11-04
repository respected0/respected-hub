//! Plugin parent/child inventory. Children keep their own asset key and a `parent_asset_key`;
//! a disabled plugin keeps its files visible but is never reported as enabled.

use crate::integrations::adapters::common::add_skills;
use crate::integrations::model::{
    Asset, AssetKind, Binding, ClientId, Inventory, ObservedStatus, Owner, Project, Scope, Source,
    SourceFormat,
};
use crate::integrations::security::{self, Limits};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn add_children(
    inventory: &mut Inventory,
    client: ClientId,
    parent: &str,
    root: &Path,
    scope: Scope,
    project: Option<Project>,
    owner: Owner,
) {
    add_skills(inventory, client, &root.join("skills"), scope, project.clone(), Some(parent.to_string()), owner.clone());
    for (kind, folder) in [(AssetKind::Command, "commands"), (AssetKind::Agent, "agents")] {
        add_markdown_files(inventory, client, parent, &root.join(folder), kind, scope, project.clone(), owner.clone());
    }
    add_hook(inventory, client, parent, &root.join("hooks/hooks.json"), scope, project, owner);
}

fn add_markdown_files(
    inventory: &mut Inventory,
    client: ClientId,
    parent: &str,
    root: &Path,
    kind: AssetKind,
    scope: Scope,
    project: Option<Project>,
    owner: Owner,
) {
    if !root.exists() { return; }
    let walk = security::walk_files(root, Limits::default());
    inventory.issues.extend(walk.issues);
    for file in walk.files.into_iter().filter(|file| file.extension().map(|ext| ext.eq_ignore_ascii_case("md")).unwrap_or(false)) {
        let Some(stem) = file.file_stem().map(|name| name.to_string_lossy().into_owned()) else { continue };
        add_child(inventory, client, parent, &file, kind, &stem, scope, project.clone(), owner.clone());
    }
}

fn add_hook(
    inventory: &mut Inventory,
    client: ClientId,
    parent: &str,
    path: &Path,
    scope: Scope,
    project: Option<Project>,
    owner: Owner,
) {
    if fs::metadata(path).map(|metadata| metadata.is_file()).unwrap_or(false) {
        add_child(inventory, client, parent, path, AssetKind::Hook, "hooks", scope, project, owner);
    }
}

fn add_child(
    inventory: &mut Inventory,
    client: ClientId,
    parent: &str,
    path: &Path,
    kind: AssetKind,
    name: &str,
    scope: Scope,
    project: Option<Project>,
    owner: Owner,
) {
    let text = match security::read_text(path, Limits::default()) {
        Ok(text) => text,
        Err(error) => {
            inventory.issues.push(error);
            return;
        }
    };
    let format = if kind == AssetKind::Hook { SourceFormat::Json } else { SourceFormat::Markdown };
    let mut source = Source::from_path(path, owner, format);
    source.fingerprint = Some(digest(&text));
    let identity = format!("file:{}", source.path_id);
    let asset = Asset::new(kind, name, &identity);
    inventory.add_binding(asset, Binding {
        asset_key: Asset::stable_key(kind, name, &identity),
        client,
        scope,
        project,
        source,
        status: ObservedStatus::Configured,
        enabled: None,
        managed: false,
        parent_asset_key: Some(parent.to_string()),
    });
}

fn digest(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}
