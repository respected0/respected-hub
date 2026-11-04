//! Helpers shared by read-only client adapters.

use crate::integrations::model::{
    Asset, AssetKind, Binding, ClientId, Inventory, ObservedStatus, Owner, Project, ScanIssue, Scope,
    Source, SourceFormat,
};
use crate::integrations::security::{self, Limits};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub(super) fn issue(inventory: &mut Inventory, code: &str, path: &Path, message: &str) {
    inventory.issues.push(ScanIssue {
        code: code.into(),
        message: message.into(),
        source_path: Some(path.to_string_lossy().into_owned()),
    });
}

pub(super) fn digest(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(super) fn source(path: &Path, owner: Owner, format: SourceFormat, fingerprint: Option<String>) -> Source {
    let mut source = Source::from_path(path, owner, format);
    source.fingerprint = fingerprint;
    source
}

/// A Respected entry stays visibly owned by its product. A matching name with a foreign
/// command is still the user's entry.
pub(super) fn entry_owner(name: &str, identity: &str) -> Owner {
    let identity = identity.to_lowercase();
    match name.trim().to_lowercase().as_str() {
        "respected-brain" | "respected-vault" if identity.contains("respected-brain") => Owner::Product { product: "brain".into() },
        "respected-orchestrator" if identity.contains("respected-orchestrator") => Owner::Product { product: "orchestrator".into() },
        _ => Owner::User,
    }
}

pub(super) fn add_skills(
    inventory: &mut Inventory,
    client: ClientId,
    root: &Path,
    scope: Scope,
    project: Option<Project>,
    parent: Option<String>,
    owner: Owner,
) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return,
        Err(error) => {
            issue(inventory, "unreadable", root, &error.to_string());
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                issue(inventory, "unreadable", root, &error.to_string());
                continue;
            }
        };
        let directory = entry.path();
        match fs::metadata(&directory) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => continue,
            Err(error) => {
                let broke = fs::symlink_metadata(&directory).map(|meta| meta.file_type().is_symlink()).unwrap_or(false);
                issue(inventory, if broke { "broken_link" } else { "unreadable" }, &directory, &error.to_string());
                continue;
            }
        }
        let manifest = directory.join("SKILL.md");
        match fs::metadata(&manifest) {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => continue,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => {
                issue(inventory, "unreadable", &manifest, &error.to_string());
                continue;
            }
        }
        let text = match security::read_text(&manifest, Limits::default()) {
            Ok(text) => text,
            Err(error) => {
                inventory.issues.push(error);
                continue;
            }
        };
        let name = directory.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        if name.is_empty() { continue; }
        let source = source(&manifest, owner.clone(), SourceFormat::SkillDirectory, Some(digest(&text)));
        let identity = format!("skill:{}", source.path_id);
        let asset = Asset::new(AssetKind::Skill, &name, &identity);
        inventory.add_binding(asset, Binding {
            asset_key: Asset::stable_key(AssetKind::Skill, &name, &identity),
            client,
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
