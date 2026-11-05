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
    #[serde(default)]
    pub drift: Vec<DriftFinding>,
}

/// Read-only drift categories. Nothing here repairs a file; callers only show
/// the source and the effect so the user can decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftCode {
    SourceChanged,
    DuplicateCopy,
    Shadowed,
    BrokenSymlink,
    OrphanTarget,
    TargetMissing,
    OwnerProtected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftFinding {
    pub code: DriftCode,
    pub asset_key: Option<String>,
    pub name: Option<String>,
    pub asset_kind: Option<AssetKind>,
    pub client: Option<ClientId>,
    pub scope: Option<Scope>,
    pub project: Option<Project>,
    pub source_path: Option<String>,
    pub message: String,
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
    Resolution { entries, issues: inventory.issues.clone(), untrusted, drift: Vec::new() }
}

/// Compare the scanned inventory with the fingerprints saved by the previous
/// scan, then classify copy, shadowing, ownership and link conflicts. Every
/// finding is explanatory: this function never writes or repairs anything.
pub fn drift(inventory: &Inventory, previous: &HashMap<String, String>) -> Vec<DriftFinding> {
    let assets: HashMap<&str, &Asset> = inventory.assets.iter().map(|asset| (asset.key.as_str(), asset)).collect();
    let mut findings = Vec::new();
    let mut seen = HashSet::new();

    for binding in &inventory.bindings {
        if let (Some(before), Some(after)) = (
            previous.get(&binding.source.path_id),
            binding.source.fingerprint.as_deref(),
        ) {
            if before != after {
                let asset = assets.get(binding.asset_key.as_str());
                push_drift(&mut findings, &mut seen, DriftFinding {
                    code: DriftCode::SourceChanged,
                    asset_key: Some(binding.asset_key.clone()),
                    name: asset.map(|asset| asset.name.clone()),
                    asset_kind: asset.map(|asset| asset.kind),
                    client: Some(binding.client),
                    scope: Some(binding.scope),
                    project: binding.project.clone(),
                    source_path: Some(binding.source.path.clone()),
                    message: "Kaynak son taramadan sonra değişti; Hub bu değişikliği otomatik düzeltmez, plan yenilenmeli.".into(),
                });
            }
        }
    }

    let mut copies: HashMap<(AssetKind, String), Vec<&Binding>> = HashMap::new();
    for binding in &inventory.bindings {
        let Some(asset) = assets.get(binding.asset_key.as_str()) else { continue };
        if asset.kind == AssetKind::Skill {
            copies.entry((asset.kind, asset.name.clone())).or_default().push(binding);
        }
    }
    for ((kind, name), bindings) in &copies {
        let paths: HashSet<&str> = bindings.iter().map(|binding| binding.source.path_id.as_str()).collect();
        if paths.len() < 2 { continue; }
        for binding in bindings {
            push_drift(&mut findings, &mut seen, DriftFinding {
                code: DriftCode::DuplicateCopy,
                asset_key: Some(binding.asset_key.clone()),
                name: Some(name.clone()),
                asset_kind: Some(*kind),
                client: Some(binding.client),
                scope: Some(binding.scope),
                project: binding.project.clone(),
                source_path: Some(binding.source.path.clone()),
                message: format!("Aynı adla {} farklı skill kopyası var; istemciler farklı içerik kullanabilir. Otomatik düzeltme yapılmaz.", paths.len()),
            });
        }
    }

    for binding in &inventory.bindings {
        let asset = assets.get(binding.asset_key.as_str());
        let code = match binding.source.integrity {
            SourceIntegrity::Missing => DriftCode::TargetMissing,
            SourceIntegrity::BrokenLink if binding.source.symlink_target.is_some() => DriftCode::OrphanTarget,
            SourceIntegrity::BrokenLink => DriftCode::BrokenSymlink,
            _ => continue,
        };
        let message = match code {
            DriftCode::TargetMissing => "Kaynağın hedefi bulunamadı; istemci bu kaydı kullanamaz.",
            DriftCode::OrphanTarget => "Symlink hedefi silinmiş; istemci bu skill'i çözemez.",
            _ => "Bozuk symlink; istemci bu kaydı kullanamaz.",
        };
        push_drift(&mut findings, &mut seen, DriftFinding {
            code,
            asset_key: Some(binding.asset_key.clone()),
            name: asset.map(|asset| asset.name.clone()),
            asset_kind: asset.map(|asset| asset.kind),
            client: Some(binding.client),
            scope: Some(binding.scope),
            project: binding.project.clone(),
            source_path: Some(binding.source.path.clone()),
            message: message.into(),
        });
    }

    let report = resolve(inventory);
    for entry in report.entries.iter().filter(|entry| entry.state == ResolutionState::Overridden) {
        push_drift(&mut findings, &mut seen, DriftFinding {
            code: DriftCode::Shadowed,
            asset_key: Some(entry.asset_key.clone()),
            name: Some(entry.name.clone()),
            asset_kind: Some(entry.kind),
            client: Some(entry.client),
            scope: Some(entry.scope),
            project: entry.project.clone(),
            source_path: Some(entry.source_path.clone()),
            message: "Aynı istemci ve kapsamda daha sonra gelen aynı adlı kayıt bu kaydı gölgeliyor.".into(),
        });
    }
    for project_entry in report.entries.iter()
        .filter(|entry| entry.effective && entry.scope == Scope::Project && matches!(entry.kind, AssetKind::Mcp | AssetKind::Plugin))
    {
        let Some(project) = &project_entry.project else { continue };
        for global in report.entries.iter().filter(|entry| entry.effective
            && entry.scope == Scope::Global
            && entry.client == project_entry.client
            && entry.kind == project_entry.kind
            && entry.name == project_entry.name
            && entry.asset_key != project_entry.asset_key)
        {
            push_drift(&mut findings, &mut seen, DriftFinding {
                code: DriftCode::Shadowed,
                asset_key: Some(global.asset_key.clone()),
                name: Some(global.name.clone()),
                asset_kind: Some(global.kind),
                client: Some(global.client),
                scope: Some(global.scope),
                project: Some(project.clone()),
                source_path: Some(global.source_path.clone()),
                message: format!("Proje {} içindeki aynı adlı kayıt bu global kaydı gölgeliyor.", project.id),
            });
        }
    }

    let conflicts: HashSet<(ClientId, AssetKind, String)> = findings.iter()
        .filter(|finding| matches!(finding.code, DriftCode::DuplicateCopy | DriftCode::Shadowed))
        .filter_map(|finding| Some((finding.client?, finding.asset_kind?, finding.name.clone()?)))
        .collect();
    for binding in &inventory.bindings {
        let Some(asset) = assets.get(binding.asset_key.as_str()) else { continue };
        let owned = binding.managed || matches!(binding.source.owner, Owner::Product { .. } | Owner::Catalog);
        if owned && conflicts.contains(&(binding.client, asset.kind, asset.name.clone())) {
            push_drift(&mut findings, &mut seen, DriftFinding {
                code: DriftCode::OwnerProtected,
                asset_key: Some(binding.asset_key.clone()),
                name: Some(asset.name.clone()),
                asset_kind: Some(asset.kind),
                client: Some(binding.client),
                scope: Some(binding.scope),
                project: binding.project.clone(),
                source_path: Some(binding.source.path.clone()),
                message: "Sahipli kayıt çakışıyor; Hub bu kaydın üzerine yazmaz ve otomatik düzeltme yapmaz.".into(),
            });
        }
    }

    for issue in &inventory.issues {
        let code = match issue.code.as_str() {
            "project_missing" => DriftCode::TargetMissing,
            "orphan_skill" => DriftCode::OrphanTarget,
            "broken_link" => DriftCode::BrokenSymlink,
            _ => continue,
        };
        push_drift(&mut findings, &mut seen, DriftFinding {
            code,
            asset_key: None,
            name: None,
            asset_kind: None,
            client: None,
            scope: None,
            project: None,
            source_path: issue.source_path.clone(),
            message: issue.message.clone(),
        });
    }

    findings
}

fn push_drift(findings: &mut Vec<DriftFinding>, seen: &mut HashSet<String>, finding: DriftFinding) {
    let key = format!(
        "{:?}|{}|{:?}|{:?}|{:?}",
        finding.code,
        finding.source_path.as_deref().unwrap_or_default(),
        finding.client,
        finding.scope,
        finding.project.as_ref().map(|project| project.id.as_str()),
    );
    if seen.insert(key) { findings.push(finding); }
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

    fn binding_at(asset: &Asset, client: ClientId, path: &str, fingerprint: Option<&str>) -> Binding {
        let mut item = binding(asset, client, Scope::Global, None, None);
        item.source = Source::from_path(Path::new(path), Owner::User, SourceFormat::SkillDirectory);
        item.source.fingerprint = fingerprint.map(str::to_string);
        item
    }

    #[test]
    fn changed_source_since_previous_scan_is_reported() {
        let asset = asset(AssetKind::Mcp, "db", "command: db");
        let mut inventory = Inventory::default();
        let mut item = binding(&asset, ClientId::Claude, Scope::Global, None, None);
        item.source.fingerprint = Some("new".into());
        let mut previous = HashMap::new();
        previous.insert(item.source.path_id.clone(), "old".to_string());
        inventory.add_binding(asset, item);
        let report = drift(&inventory, &previous);
        assert!(report.iter().any(|finding| finding.code == DriftCode::SourceChanged));
    }

    #[test]
    fn same_skill_name_with_different_copies_is_duplicate_copy() {
        let first = asset(AssetKind::Skill, "draft", "skill:first");
        let second = asset(AssetKind::Skill, "draft", "skill:second");
        let mut inventory = Inventory::default();
        inventory.add_binding(first.clone(), binding_at(&first, ClientId::Claude, "C:/claude/draft/SKILL.md", Some("a")));
        inventory.add_binding(second.clone(), binding_at(&second, ClientId::Codex, "C:/codex/draft/SKILL.md", Some("b")));
        let report = drift(&inventory, &HashMap::new());
        assert_eq!(report.iter().filter(|finding| finding.code == DriftCode::DuplicateCopy).count(), 2);
    }

    #[test]
    fn project_copy_shadowing_a_global_copy_is_reported() {
        let global = asset(AssetKind::Mcp, "db", "command: global");
        let project = asset(AssetKind::Mcp, "db", "command: project");
        let mut inventory = Inventory::default();
        inventory.add_binding(global.clone(), binding(&global, ClientId::Cursor, Scope::Global, None, None));
        inventory.add_binding(project.clone(), binding(&project, ClientId::Cursor, Scope::Project, Some("a"), None));
        let report = drift(&inventory, &HashMap::new());
        let shadowed: Vec<_> = report.iter().filter(|finding| finding.code == DriftCode::Shadowed).collect();
        assert_eq!(shadowed.len(), 1);
        assert_eq!(shadowed[0].asset_key.as_deref(), Some(global.key.as_str()));
    }

    #[test]
    fn broken_symlink_orphan_target_and_missing_project_are_classified() {
        let broken = asset(AssetKind::Skill, "broken", "skill:broken");
        let orphan = asset(AssetKind::Skill, "orphan", "skill:orphan");
        let missing = asset(AssetKind::Skill, "missing", "skill:missing");
        let mut inventory = Inventory::default();
        let mut broken_binding = binding_at(&broken, ClientId::Codex, "C:/codex/broken/SKILL.md", Some("a"));
        broken_binding.source.integrity = SourceIntegrity::BrokenLink;
        let mut orphan_binding = binding_at(&orphan, ClientId::Codex, "C:/codex/orphan/SKILL.md", Some("b"));
        orphan_binding.source.integrity = SourceIntegrity::BrokenLink;
        orphan_binding.source.symlink_target = Some("C:/gone".into());
        let mut missing_binding = binding_at(&missing, ClientId::Codex, "C:/codex/missing/SKILL.md", Some("c"));
        missing_binding.source.integrity = SourceIntegrity::Missing;
        inventory.add_binding(broken, broken_binding);
        inventory.add_binding(orphan, orphan_binding);
        inventory.add_binding(missing, missing_binding);
        inventory.issues.push(ScanIssue {
            code: "project_missing".into(), message: "Kayıtlı proje klasörü bulunamadı.".into(),
            source_path: Some("C:/work/gone".into()),
        });
        let report = drift(&inventory, &HashMap::new());
        assert!(report.iter().any(|finding| finding.code == DriftCode::BrokenSymlink));
        assert!(report.iter().any(|finding| finding.code == DriftCode::OrphanTarget));
        assert_eq!(report.iter().filter(|finding| finding.code == DriftCode::TargetMissing).count(), 2);
    }

    #[test]
    fn owned_entry_in_a_conflict_is_protected() {
        let owned = asset(AssetKind::Skill, "respected-skill", "skill:owned");
        let copy = asset(AssetKind::Skill, "respected-skill", "skill:copy");
        let mut inventory = Inventory::default();
        let mut owned_binding = binding_at(&owned, ClientId::Claude, "C:/claude/respected-skill/SKILL.md", Some("a"));
        owned_binding.managed = true;
        owned_binding.source.owner = Owner::Product { product: "orchestrator".into() };
        inventory.add_binding(owned, owned_binding);
        inventory.add_binding(copy.clone(), binding_at(&copy, ClientId::Codex, "C:/codex/respected-skill/SKILL.md", Some("b")));
        let report = drift(&inventory, &HashMap::new());
        assert!(report.iter().any(|finding| finding.code == DriftCode::OwnerProtected));
    }
}
