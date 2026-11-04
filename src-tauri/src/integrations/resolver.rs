//! Effective configuration resolver.
//!
//! Adapters append bindings from lowest to highest precedence. The resolver never
//! treats a file's existence as proof of use: a record becomes effective only when
//! it survives precedence, integrity and explicit disable checks.

use crate::integrations::model::{
    Asset, AssetKind, Binding, ClientId, Inventory, Owner, Project, ScanIssue, Scope, SourceIntegrity,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionState {
    Effective,
    Overridden,
    Disabled,
    Unknown,
    Untrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedEntry {
    pub asset_key: String,
    pub name: String,
    pub kind: AssetKind,
    pub client: ClientId,
    pub scope: Scope,
    pub project: Option<Project>,
    pub source_path: String,
    pub configured: bool,
    pub effective: bool,
    pub enabled: Option<bool>,
    pub managed: bool,
    pub owner: Owner,
    pub state: ResolutionState,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub entries: Vec<ResolvedEntry>,
    pub issues: Vec<ScanIssue>,
    pub untrusted: Vec<ScanIssue>,
}

/// Resolve every scanned binding to one observable state. Configured is always
/// true for a scanned record; effective is the answer to "would the client use
/// this declaration in its own context".
pub fn resolve(inventory: &Inventory) -> Resolution {
    let assets: HashMap<&str, &Asset> = inventory.assets.iter().map(|asset| (asset.key.as_str(), asset)).collect();
    let disabled_plugins: HashSet<&str> = inventory.bindings.iter()
        .filter(|binding| asset_of(binding, &assets).map(|asset| asset.kind == AssetKind::Plugin).unwrap_or(false))
        .filter(|binding| binding.enabled == Some(false))
        .map(|binding| binding.asset_key.as_str())
        .collect();

    let mut entries = Vec::with_capacity(inventory.bindings.len());
    for (index, binding) in inventory.bindings.iter().enumerate() {
        let Some(asset) = asset_of(binding, &assets) else { continue };
        let disabled_by_parent = binding.parent_asset_key.as_ref()
            .map(|parent| disabled_plugins.contains(parent.as_str()))
            .unwrap_or(false);
        let disabled = binding.enabled == Some(false) || disabled_by_parent;
        let unknown = binding.source.integrity != SourceIntegrity::Ok;
        let overridden = !disabled && !unknown
            && matches!(asset.kind, AssetKind::Mcp | AssetKind::Plugin)
            && inventory.bindings.iter().skip(index + 1).any(|later| overrides(later, binding, &assets));
        let state = if disabled {
            ResolutionState::Disabled
        } else if unknown {
            ResolutionState::Unknown
        } else if overridden {
            ResolutionState::Overridden
        } else {
            ResolutionState::Effective
        };
        entries.push(ResolvedEntry {
            asset_key: binding.asset_key.clone(),
            name: asset.name.clone(),
            kind: asset.kind,
            client: binding.client,
            scope: binding.scope,
            project: binding.project.clone(),
            source_path: binding.source.path.clone(),
            configured: true,
            effective: state == ResolutionState::Effective,
            enabled: binding.enabled,
            managed: binding.managed,
            owner: binding.source.owner.clone(),
            state,
        });
    }

    let untrusted = inventory.issues.iter().filter(|issue| issue.code == "project_untrusted").cloned().collect();
    Resolution { entries, issues: inventory.issues.clone(), untrusted }
}

/// Winning entries for one client and project context. Global records apply to
/// every project, but a project record with the same MCP/plugin name shadows
/// the global one inside that project only.
pub fn effective_for(report: &Resolution, client: ClientId, project: Option<&str>) -> Vec<ResolvedEntry> {
    let mut entries: Vec<ResolvedEntry> = report.entries.iter()
        .filter(|entry| entry.client == client && entry.state == ResolutionState::Effective)
        .filter(|entry| match entry.scope {
            Scope::Global => true,
            Scope::Project => project.is_some_and(|id| entry.project.as_ref().is_some_and(|p| p.id == id)),
        })
        .cloned()
        .collect();
    let shadowed: HashSet<(AssetKind, String)> = entries.iter()
        .filter(|entry| entry.scope == Scope::Project)
        .map(|entry| (entry.kind, entry.name.clone()))
        .collect();
    entries.retain(|entry| !(entry.scope == Scope::Global
        && matches!(entry.kind, AssetKind::Mcp | AssetKind::Plugin)
        && shadowed.contains(&(entry.kind, entry.name.clone()))));
    entries
}

fn asset_of<'a>(binding: &Binding, assets: &'a HashMap<&str, &'a Asset>) -> Option<&'a Asset> {
    assets.get(binding.asset_key.as_str()).copied()
}

fn overrides(later: &Binding, earlier: &Binding, assets: &HashMap<&str, &Asset>) -> bool {
    if later.client != earlier.client { return false; }
    let Some(later_asset) = asset_of(later, assets) else { return false };
    let Some(earlier_asset) = asset_of(earlier, assets) else { return false };
    if later_asset.kind != earlier_asset.kind || later_asset.name != earlier_asset.name { return false; }
    if later.parent_asset_key != earlier.parent_asset_key { return false; }
    match (earlier.scope, later.scope) {
        (Scope::Global, Scope::Global) => true,
        (Scope::Project, Scope::Project) => earlier.project.as_ref().map(|p| &p.id) == later.project.as_ref().map(|p| &p.id),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::model::{Owner, Source, SourceFormat};
    use std::path::Path;

    fn asset(kind: AssetKind, name: &str, identity: &str) -> Asset {
        Asset::new(kind, name, identity)
    }

    fn binding(asset: &Asset, client: ClientId, scope: Scope, project: Option<&str>, enabled: Option<bool>) -> Binding {
        Binding {
            asset_key: asset.key.clone(),
            client,
            scope,
            project: project.map(|id| Project { id: id.into(), root: format!("C:/work/{id}") }),
            source: Source::from_path(Path::new("C:/config.json"), Owner::User, SourceFormat::Json),
            status: crate::integrations::model::ObservedStatus::Configured,
            enabled,
            managed: false,
            parent_asset_key: None,
        }
    }

    #[test]
    fn project_scope_shadows_global_only_inside_that_project() {
        let global = asset(AssetKind::Mcp, "db", "command: global");
        let project = asset(AssetKind::Mcp, "db", "command: project");
        let mut inventory = Inventory::default();
        inventory.add_binding(global.clone(), binding(&global, ClientId::Cursor, Scope::Global, None, None));
        inventory.add_binding(project.clone(), binding(&project, ClientId::Cursor, Scope::Project, Some("a"), None));
        let report = resolve(&inventory);
        let a = effective_for(&report, ClientId::Cursor, Some("a"));
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].scope, Scope::Project);
        assert_eq!(effective_for(&report, ClientId::Cursor, None).len(), 1);
        assert_eq!(effective_for(&report, ClientId::Cursor, Some("b"))[0].scope, Scope::Global);
    }

    #[test]
    fn disabled_plugin_children_are_configured_but_not_effective() {
        let plugin = asset(AssetKind::Plugin, "demo", "plugin:demo");
        let child = asset(AssetKind::Skill, "child", "plugin:demo:child");
        let mut inventory = Inventory::default();
        inventory.add_binding(plugin.clone(), binding(&plugin, ClientId::Claude, Scope::Global, None, Some(false)));
        let mut child_binding = binding(&child, ClientId::Claude, Scope::Global, None, None);
        child_binding.parent_asset_key = Some(plugin.key.clone());
        inventory.add_binding(child, child_binding);
        let report = resolve(&inventory);
        assert!(report.entries.iter().all(|entry| entry.configured && !entry.effective));
        assert!(report.entries.iter().all(|entry| entry.state == ResolutionState::Disabled));
    }

    #[test]
    fn broken_source_stays_unknown_even_when_configured() {
        let asset = asset(AssetKind::Skill, "draft", "skill:draft");
        let mut inventory = Inventory::default();
        let mut item = binding(&asset, ClientId::Codex, Scope::Global, None, None);
        item.source.integrity = SourceIntegrity::BrokenLink;
        inventory.add_binding(asset, item);
        let report = resolve(&inventory);
        assert_eq!(report.entries[0].state, ResolutionState::Unknown);
        assert!(!report.entries[0].effective);
        assert!(effective_for(&report, ClientId::Codex, None).is_empty());
    }

    #[test]
    fn plugin_child_does_not_override_a_user_entry_with_the_same_name() {
        let user = asset(AssetKind::Mcp, "db", "command: user");
        let plugin = asset(AssetKind::Plugin, "demo", "plugin:demo");
        let child = asset(AssetKind::Mcp, "db", "command: bundled");
        let mut inventory = Inventory::default();
        inventory.add_binding(user.clone(), binding(&user, ClientId::Claude, Scope::Global, None, None));
        inventory.add_binding(plugin.clone(), binding(&plugin, ClientId::Claude, Scope::Global, None, Some(true)));
        let mut child_binding = binding(&child, ClientId::Claude, Scope::Global, None, None);
        child_binding.parent_asset_key = Some(plugin.key.clone());
        inventory.add_binding(child, child_binding);
        let report = resolve(&inventory);
        assert_eq!(report.entries.iter().filter(|entry| entry.effective).count(), 3);
    }

    #[test]
    fn untrusted_issues_are_returned_without_inventing_entries() {
        let mut inventory = Inventory::default();
        inventory.issues.push(ScanIssue {
            code: "project_untrusted".into(),
            message: "trust required".into(),
            source_path: Some("C:/work/a/.codex/config.toml".into()),
        });
        let report = resolve(&inventory);
        assert!(report.entries.is_empty());
        assert_eq!(report.untrusted.len(), 1);
    }
}
