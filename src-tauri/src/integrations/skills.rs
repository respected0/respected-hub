//! Skill folder inventory: canonical identity across symlinks, manifest validation, orphans.
//!
//! A symlinked skill keeps its deployment path but takes the canonical target path as identity,
//! so the same skill in two clients is one asset while two copies stay separate. Nothing here
//! executes a skill or writes to it.

use crate::integrations::model::{
    Asset, AssetKind, Binding, ClientId, Inventory, ObservedStatus, Owner, Project, Scope, Source,
    SourceFormat, SourceIntegrity,
};
use crate::integrations::security::{self, Limits};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub fn inspect(
    inventory: &mut Inventory,
    client: ClientId,
    manifest: &Path,
    scope: Scope,
    project: Option<Project>,
    parent: Option<String>,
    owner: Owner,
) {
    match fs::metadata(manifest) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return,
        Err(error) if error.kind() == ErrorKind::NotFound => return,
        Err(error) => return issue(inventory, "unreadable", manifest, &error.to_string()),
    }
    let text = match security::read_text(manifest, Limits::default()) {
        Ok(text) => text,
        Err(error) => {
            inventory.issues.push(error);
            return;
        }
    };
    if text.trim().is_empty() {
        return issue(inventory, "skill_empty", manifest, "SKILL.md boş.");
    }
    if !frontmatter_complete(&text) {
        return issue(inventory, "skill_invalid_manifest", manifest, "SKILL.md frontmatter kapanmıyor.");
    }
    let name = manifest.parent().and_then(Path::file_name).map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    if name.is_empty() { return; }

    let canonical = fs::canonicalize(manifest);
    let (identity, integrity, status) = match canonical {
        Ok(path) => {
            let path_id = Source::from_path(&path, owner.clone(), SourceFormat::SkillDirectory).path_id;
            (format!("skill:{path_id}"), SourceIntegrity::Ok, ObservedStatus::Configured)
        }
        Err(error) => {
            issue(inventory, "orphan_skill", manifest, &format!("Skill hedefi çözülemedi: {error}"));
            (format!("skill:{}", Source::from_path(manifest, owner.clone(), SourceFormat::SkillDirectory).path_id), SourceIntegrity::BrokenLink, ObservedStatus::Unknown)
        }
    };
    let mut source = Source::from_path(manifest, owner, SourceFormat::SkillDirectory);
    source.fingerprint = Some(digest(&text));
    source.symlink_target = symlink_target(manifest);
    source.integrity = integrity;
    let asset = Asset::new(AssetKind::Skill, &name, &identity);
    inventory.add_binding(asset, Binding {
        asset_key: Asset::stable_key(AssetKind::Skill, &name, &identity),
        client,
        scope,
        project,
        source,
        status,
        enabled: None,
        managed: false,
        parent_asset_key: parent,
    });
}

fn symlink_target(manifest: &Path) -> Option<String> {
    let directory = manifest.parent()?;
    for path in [manifest, directory] {
        if fs::symlink_metadata(path).map(|metadata| metadata.file_type().is_symlink()).unwrap_or(false) {
            if let Ok(target) = fs::read_link(path) {
                let target = if target.is_absolute() { target } else { path.parent().unwrap_or(directory).join(target) };
                return Some(target.to_string_lossy().into_owned());
            }
        }
    }
    None
}

fn frontmatter_complete(text: &str) -> bool {
    let text = text.trim_start_matches('\u{feff}').trim_start();
    let Some(rest) = text.strip_prefix("---") else { return true };
    let rest = rest.strip_prefix("\r\n").or_else(|| rest.strip_prefix('\n')).unwrap_or(rest);
    rest.lines().any(|line| line.trim() == "---")
}

fn digest(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn issue(inventory: &mut Inventory, code: &str, path: &Path, message: &str) {
    inventory.issues.push(crate::integrations::model::ScanIssue {
        code: code.into(),
        message: message.into(),
        source_path: Some(path.to_string_lossy().into_owned()),
    });
}
