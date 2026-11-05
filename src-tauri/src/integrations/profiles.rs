//! Re-applicable, secret-free integration profiles.
//!
//! A profile stores only the closed `PlanRequest` vocabulary: asset keys, client
//! ids, scopes and project ids. It never stores command lines, environment
//! values, credentials or file contents. Profiles live in Hub app data, never in
//! an AI client configuration, and are expanded through the existing planner so
//! re-applying an already satisfied profile produces `noop` plans.

use crate::integrations::discovery::Environment;
use crate::integrations::model::{Inventory, Project, Scope};
use crate::integrations::planner::{self, Operation, OperationPlan, PlanError, PlanRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PROFILE_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub requests: Vec<PlanRequest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Store {
    version: u32,
    profiles: Vec<Profile>,
}

impl Default for Store {
    fn default() -> Self { Self { version: PROFILE_VERSION, profiles: Vec::new() } }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilePlanResult {
    pub asset_key: String,
    pub plan: Option<OperationPlan>,
    pub error: Option<PlanError>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilePlan {
    pub profile_id: String,
    pub profile_name: String,
    pub results: Vec<ProfilePlanResult>,
    /// Asset names (or keys when unknown) whose binding was not found.
    pub missing_sources: Vec<String>,
    pub noop: bool,
    pub read_only: bool,
    pub created_at: u64,
}

pub fn registry_file(env: &Environment) -> PathBuf {
    env.app_data.join("com.respected.hub").join("profiles.json")
}

pub fn load(env: &Environment) -> Result<Vec<Profile>, String> {
    let file = registry_file(env);
    if !file.exists() { return Ok(Vec::new()); }
    let store: Store = serde_json::from_slice(&fs::read(&file).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Profil kaydı okunamadı: {e}"))?;
    if store.version != PROFILE_VERSION {
        return Err(format!("Desteklenmeyen profil sürümü: {}", store.version));
    }
    for profile in &store.profiles { validate(profile)?; }
    Ok(store.profiles)
}

/// Save or replace one profile by id. Entries are closed planner requests, so a
/// profile can never carry credentials, and only `bind_mcp` is accepted.
pub fn save(env: &Environment, profile: &Profile) -> Result<Vec<Profile>, String> {
    validate(profile)?;
    let file = registry_file(env);
    let mut store: Store = if file.exists() {
        serde_json::from_slice(&fs::read(&file).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Profil kaydı okunamadı: {e}"))?
    } else {
        Store::default()
    };
    if store.version != PROFILE_VERSION {
        return Err(format!("Desteklenmeyen profil sürümü: {}", store.version));
    }
    match store.profiles.iter_mut().find(|item| item.id == profile.id) {
        Some(existing) => *existing = profile.clone(),
        None => store.profiles.push(profile.clone()),
    }
    write(&file, &store)?;
    Ok(store.profiles)
}

/// Expand a profile into the existing planner. Every entry is reported; a
/// missing source or unsupported client never hides the remaining entries.
pub fn plan_profile(
    inventory: &Inventory,
    projects: &[Project],
    profile: &Profile,
    now: u64,
) -> ProfilePlan {
    let mut missing = Vec::new();
    let mut seen_missing = HashSet::new();
    let results: Vec<ProfilePlanResult> = profile.requests.iter().map(|request| {
        match planner::plan_at(inventory, projects, request, now) {
            Ok(plan) => ProfilePlanResult { asset_key: request.asset_key.clone(), plan: Some(plan), error: None },
            Err(error) => {
                if error.code == "TARGET_NOT_FOUND" && seen_missing.insert(request.asset_key.clone()) {
                    missing.push(asset_label(inventory, &request.asset_key));
                }
                ProfilePlanResult { asset_key: request.asset_key.clone(), plan: None, error: Some(error) }
            }
        }
    }).collect();
    let noop = !results.is_empty()
        && results.iter().all(|result| result.plan.as_ref().is_some_and(|plan| plan.noop));
    ProfilePlan {
        profile_id: profile.id.clone(),
        profile_name: profile.name.clone(),
        results,
        missing_sources: missing,
        noop,
        read_only: true,
        created_at: now,
    }
}

pub fn validate(profile: &Profile) -> Result<(), String> {
    let id = profile.id.trim();
    if id.is_empty() || id.len() > 64
        || !id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
    {
        return Err("Profil kimliği 1-64 güvenli ASCII karakter olmalı.".into());
    }
    if profile.name.trim().is_empty() || profile.name.len() > 120 {
        return Err("Profil adı 1-120 karakter olmalı.".into());
    }
    if profile.requests.is_empty() {
        return Err("Profil en az bir bağlama isteği içermeli.".into());
    }
    for request in &profile.requests {
        if request.operation != Operation::BindMcp {
            return Err("Profil yalnızca bağlama (bind_mcp) istekleri saklayabilir.".into());
        }
        if request.asset_key.trim().is_empty() {
            return Err("Profil isteği varlık anahtarı taşımalı.".into());
        }
        if request.target_client.is_some() && !request.target_clients.is_empty() {
            return Err("Hedef istemci tek alan veya liste olarak verilmeli.".into());
        }
        let clients: Vec<_> = request.target_client.into_iter().chain(request.target_clients.iter().copied()).collect();
        if clients.is_empty() {
            return Err("Profil isteği hedef istemci taşımalı.".into());
        }
        let mut seen = HashSet::new();
        if clients.iter().any(|client| !seen.insert(*client)) {
            return Err("Aynı hedef istemci birden fazla kez verilemez.".into());
        }
        match (request.scope, request.project.as_deref()) {
            (Scope::Global, Some(_)) => return Err("Genel kapsam proje kimliği taşıyamaz.".into()),
            (Scope::Project, None) => return Err("Proje kapsamı için proje kimliği gerekli.".into()),
            _ => {}
        }
        match (request.target_scope, request.target_project.as_deref()) {
            (Some(Scope::Project), None) => return Err("Hedef proje kapsamı için proje kimliği gerekli.".into()),
            (Some(Scope::Global), Some(_)) => return Err("Hedef genel kapsam proje kimliği taşıyamaz.".into()),
            (None, _) => return Err("Bağlama isteği hedef kapsam taşımalı.".into()),
            _ => {}
        }
    }
    Ok(())
}

fn asset_label(inventory: &Inventory, key: &str) -> String {
    inventory.assets.iter().find(|asset| asset.key == key)
        .map(|asset| asset.name.clone())
        .unwrap_or_else(|| key.to_string())
}

fn write(file: &Path, store: &Store) -> Result<(), String> {
    fs::create_dir_all(file.parent().ok_or("Geçersiz profil kaydı yolu")?).map_err(|e| e.to_string())?;
    let temporary = file.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(store).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(temporary, file).map_err(|e| e.to_string())
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::apply;
    use crate::integrations::model::ClientId;
    use crate::integrations::inventory;
    use std::collections::BTreeMap;

    struct Temp(PathBuf);
    impl Temp {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("hub-profiles-{label}-{}-{nanos}", std::process::id()));
            fs::create_dir_all(path.join("home")).unwrap();
            Self(path)
        }
        fn env(&self) -> Environment {
            Environment {
                home: self.0.join("home"),
                app_data: self.0.join("roaming"),
                local_app_data: self.0.join("local"),
                path: Vec::new(),
                path_ext: Vec::new(),
                vars: BTreeMap::new(),
            }
        }
    }
    impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

    fn two_projects(temp: &Temp) -> (Vec<Project>, Vec<PathBuf>) {
        let mut projects = Vec::new();
        let mut roots = Vec::new();
        for (id, label) in [("project:one", "one"), ("project:two", "two")] {
            let root = temp.0.join(format!("work/{label}"));
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join(".mcp.json"), format!(r#"{{"mcpServers":{{"local-{label}":{{"command":"x"}}}}}}"#)).unwrap();
            projects.push(Project { id: id.into(), root: root.to_string_lossy().into_owned() });
            roots.push(root);
        }
        (projects, roots)
    }

    fn bind_request(inventory: &Inventory, project: &str) -> PlanRequest {
        let asset = inventory.assets.iter().find(|asset| asset.name == "postgres").unwrap();
        PlanRequest {
            operation: Operation::BindMcp,
            asset_key: asset.key.clone(),
            client: ClientId::Claude,
            scope: Scope::Global,
            project: None,
            target_scope: Some(Scope::Project),
            target_project: Some(project.into()),
            target_client: None,
            target_clients: vec![ClientId::Claude],
        }
    }

    fn raw_request(asset_key: &str) -> PlanRequest {
        PlanRequest {
            operation: Operation::BindMcp,
            asset_key: asset_key.into(),
            client: ClientId::Claude,
            scope: Scope::Global,
            project: None,
            target_scope: Some(Scope::Project),
            target_project: Some("project:one".into()),
            target_client: None,
            target_clients: vec![ClientId::Claude],
        }
    }

    #[test]
    fn profile_applies_to_two_projects_and_reapplies_idempotently() {
        let temp = Temp::new("two-projects");
        let env = temp.env();
        fs::write(temp.0.join("home/.claude.json"), r#"{"mcpServers":{"postgres":{"command":"npx","env":{"API_KEY":"sk-live-secret"}},"keep":{"command":"x"}}}"#).unwrap();
        let (projects, roots) = two_projects(&temp);
        let inventory = inventory::scan(&env, &projects);
        let profile = Profile {
            id: "profile:team".into(),
            name: "Team setup".into(),
            requests: vec![bind_request(&inventory, "project:one"), bind_request(&inventory, "project:two")],
        };
        save(&env, &profile).unwrap();
        let stored = fs::read_to_string(registry_file(&env)).unwrap();
        assert!(!stored.contains("sk-live-secret"));
        assert_eq!(load(&env).unwrap(), vec![profile.clone()]);

        let first = plan_profile(&inventory, &projects, &profile, now());
        assert!(!first.noop);
        assert!(first.missing_sources.is_empty());
        assert_eq!(first.results.len(), 2);
        let source_before = fs::read(temp.0.join("home/.claude.json")).unwrap();
        for (index, result) in first.results.iter().enumerate() {
            let plan = result.plan.as_ref().unwrap();
            let applied = apply::apply_at(plan, &inventory, &projects, &format!("profile-tx-{index}"), plan.created_at).unwrap();
            assert_eq!(applied.files.len(), 1);
        }
        assert_eq!(source_before, fs::read(temp.0.join("home/.claude.json")).unwrap());
        for (root, label) in roots.iter().zip(["local-one", "local-two"]) {
            let text = fs::read_to_string(root.join(".mcp.json")).unwrap();
            assert!(text.contains("postgres"));
            assert!(text.contains(label), "previous entry was deleted: {text}");
        }

        let fresh = inventory::scan(&env, &projects);
        let before_reapply: Vec<Vec<u8>> = roots.iter().map(|root| fs::read(root.join(".mcp.json")).unwrap()).collect();
        for _ in 0..2 {
            let again = plan_profile(&fresh, &projects, &profile, now());
            assert!(again.noop, "reapply must be idempotent: {again:?}");
            for result in &again.results {
                let plan = result.plan.as_ref().unwrap();
                assert!(plan.targets.is_empty());
                apply::apply_at(plan, &fresh, &projects, "profile-noop", plan.created_at).unwrap();
            }
        }
        let after_reapply: Vec<Vec<u8>> = roots.iter().map(|root| fs::read(root.join(".mcp.json")).unwrap()).collect();
        assert_eq!(before_reapply, after_reapply);
    }

    #[test]
    fn missing_source_is_listed_without_blocking_the_other_entries() {
        let temp = Temp::new("missing");
        let env = temp.env();
        fs::write(temp.0.join("home/.claude.json"), r#"{"mcpServers":{"postgres":{"command":"npx"}}}"#).unwrap();
        let (projects, _) = two_projects(&temp);
        let inventory = inventory::scan(&env, &projects);
        let mut missing = bind_request(&inventory, "project:one");
        missing.asset_key = "mcp:missing".into();
        let profile = Profile {
            id: "profile:missing".into(),
            name: "Missing source".into(),
            requests: vec![missing, bind_request(&inventory, "project:two")],
        };
        let planned = plan_profile(&inventory, &projects, &profile, now());
        assert!(!planned.noop);
        assert_eq!(planned.missing_sources, vec!["mcp:missing".to_string()]);
        assert_eq!(planned.results[0].error.as_ref().unwrap().code, "TARGET_NOT_FOUND");
        assert!(planned.results[1].error.is_none());
        assert!(!planned.results[1].plan.as_ref().unwrap().plan_digest.is_empty());
    }

    #[test]
    fn differing_client_capabilities_are_reported_per_entry() {
        let temp = Temp::new("capabilities");
        let env = temp.env();
        fs::write(temp.0.join("home/.claude.json"), r#"{"mcpServers":{"postgres":{"command":"npx"}}}"#).unwrap();
        fs::create_dir_all(temp.0.join("home/.codex")).unwrap();
        fs::write(temp.0.join("home/.codex/config.toml"), "[mcp_servers.keep]\ncommand = \"x\"\n").unwrap();
        let (projects, _) = two_projects(&temp);
        let inventory = inventory::scan(&env, &projects);
        let mut toml_target = bind_request(&inventory, "project:one");
        toml_target.target_scope = Some(Scope::Global);
        toml_target.target_project = None;
        toml_target.target_clients = vec![ClientId::Codex];
        let profile = Profile {
            id: "profile:mixed".into(),
            name: "Mixed clients".into(),
            requests: vec![toml_target, bind_request(&inventory, "project:two")],
        };
        let planned = plan_profile(&inventory, &projects, &profile, now());
        assert_eq!(planned.results[0].error.as_ref().unwrap().code, "UNSUPPORTED_CAPABILITY");
        assert!(planned.results[1].error.is_none());
        assert!(planned.missing_sources.is_empty());
    }

    #[test]
    fn profiles_reject_destructive_operations_and_unknown_fields() {
        let temp = Temp::new("validation");
        let env = temp.env();
        let mut profile = Profile { id: "profile:bad".into(), name: "Bad".into(), requests: vec![raw_request("mcp:demo")] };
        profile.requests[0].operation = Operation::RemoveMcp;
        assert!(save(&env, &profile).is_err());
        fs::create_dir_all(registry_file(&env).parent().unwrap()).unwrap();
        fs::write(registry_file(&env), r#"{"version":1,"profiles":[],"extra":true}"#).unwrap();
        assert!(load(&env).is_err());
    }
}
