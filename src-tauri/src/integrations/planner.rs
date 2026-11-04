//! Read-only change planner for third-party integrations.
//!
//! A plan is a closed preview: it reads the current source, proves the expected
//! entry is still there, and returns a deterministic redacted diff plus hashes.
//! It never writes, creates backups or executes a client.

use crate::integrations::model::{
    AssetKind, Binding, ClientId, Inventory, Owner, Project, Scope, SourceFormat,
};
use crate::integrations::security::{self, Limits};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const PLAN_TTL_SECONDS: u64 = 300;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    RemoveMcp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanRequest {
    pub operation: Operation,
    pub asset_key: String,
    pub client: ClientId,
    pub scope: Scope,
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanTarget {
    pub path: String,
    pub path_id: String,
    pub format: SourceFormat,
    pub before_sha256: String,
    pub after_sha256: String,
    pub before: Value,
    pub after: Value,
    pub diff: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationPlan {
    pub plan_digest: String,
    pub operation: Operation,
    pub asset_key: String,
    pub name: String,
    pub kind: AssetKind,
    pub client: ClientId,
    pub scope: Scope,
    pub project: Option<Project>,
    pub owner: Owner,
    pub managed: bool,
    pub targets: Vec<PlanTarget>,
    pub affected_projects: Vec<Project>,
    pub risks: Vec<String>,
    pub created_at: u64,
    pub expires_at: u64,
    pub read_only: bool,
}

pub fn plan(inventory: &Inventory, projects: &[Project], request: &PlanRequest) -> Result<OperationPlan, PlanError> {
    plan_at(inventory, projects, request, unix_now())
}

pub fn plan_at(inventory: &Inventory, projects: &[Project], request: &PlanRequest, now: u64) -> Result<OperationPlan, PlanError> {
    validate_scope(request, projects)?;
    let binding = find_binding(inventory, request)?;
    let asset = inventory.assets.iter().find(|asset| asset.key == request.asset_key)
        .ok_or_else(|| fail("TARGET_NOT_FOUND", "Seçilen varlık envanterde bulunamadı; envanteri yenileyin."))?;
    let supported = inventory.capabilities.iter().find(|item| item.client == binding.client)
        .map(|item| item.plan).unwrap_or(false);
    if !supported || asset.kind != AssetKind::Mcp {
        return Err(fail("UNSUPPORTED_CAPABILITY", "Bu istemci veya varlık türü için değişiklik planı desteklenmiyor."));
    }
    if binding.managed || matches!(binding.source.owner, Owner::Product { .. } | Owner::Catalog) {
        return Err(fail("OWNER_PROTECTED", "Bu kayıt Respected ürününe veya kataloğa ait; harici motorla değiştirilemez."));
    }
    let container = container_for(binding.client);
    let text = security::read_text(Path::new(&binding.source.path), Limits::default())
        .map_err(|error| fail("SOURCE_CHANGED", &format!("Hedef dosya okunamadı: {}", error.message)))?;
    let before_sha256 = sha256(text.as_bytes());
    if binding.source.fingerprint.as_deref().is_some_and(|fingerprint| fingerprint != before_sha256) {
        return Err(fail("SOURCE_CHANGED", "Hedef dosya taramadan sonra değişti; planı yenileyin."));
    }
    let (before, after_sha256, diff) = match binding.source.format {
        SourceFormat::Json | SourceFormat::Jsonc | SourceFormat::Toml =>
            plan_document(&text, binding.source.format, container, &asset.name)?,
        _ => return Err(fail("UNSUPPORTED_CAPABILITY", "Bu kaynak biçimi için biçim korumalı plan henüz desteklenmiyor.")),
    };
    let mut risks = Vec::new();
    if binding.scope == Scope::Global && !projects.is_empty() {
        risks.push("Genel kayıt kaldırıldığında bu istemciyi kullanan tüm projeler etkilenebilir.".to_string());
    }
    if binding.source.format == SourceFormat::Toml {
        risks.push("TOML düzenlemesi biçim korumalıdır; yazma kapısı ürün sahibi onayına kadar kapalıdır.".to_string());
    }
    let affected_projects = match binding.scope {
        Scope::Global => projects.to_vec(),
        Scope::Project => binding.project.iter().cloned().collect(),
    };
    let mut result = OperationPlan {
        plan_digest: String::new(),
        operation: request.operation,
        asset_key: asset.key.clone(),
        name: asset.name.clone(),
        kind: asset.kind,
        client: binding.client,
        scope: binding.scope,
        project: binding.project.clone(),
        owner: binding.source.owner.clone(),
        managed: binding.managed,
        targets: vec![PlanTarget {
            path: binding.source.path.clone(),
            path_id: binding.source.path_id.clone(),
            format: binding.source.format,
            before_sha256,
            after_sha256,
            before,
            after: Value::Null,
            diff,
        }],
        affected_projects,
        risks,
        created_at: now,
        expires_at: now.saturating_add(PLAN_TTL_SECONDS),
        read_only: true,
    };
    result.plan_digest = digest_for(&result);
    Ok(result)
}

/// Verify a previously issued plan without touching the file. Apply will call this
/// and then re-derive the binding from a fresh inventory before writing.
pub fn verify(plan: &OperationPlan, now: u64) -> Result<(), PlanError> {
    if now >= plan.expires_at {
        return Err(fail("PLAN_EXPIRED", "Planın süresi doldu; yeni bir önizleme oluşturun."));
    }
    if digest_for(plan) != plan.plan_digest {
        return Err(fail("INVALID_PLAN", "Plan özeti doğrulanamadı; plan değiştirilmiş veya bozuk."));
    }
    if plan.managed || matches!(plan.owner, Owner::Product { .. } | Owner::Catalog) {
        return Err(fail("OWNER_PROTECTED", "Bu kayıt Respected ürününe veya kataloğa ait; harici motorla değiştirilemez."));
    }
    for target in &plan.targets {
        let bytes = fs::read(Path::new(&target.path))
            .map_err(|_| fail("SOURCE_CHANGED", "Hedef dosya artık okunamıyor; planı yenileyin."))?;
        if sha256(&bytes) != target.before_sha256 {
            return Err(fail("SOURCE_CHANGED", "Hedef dosya plandan sonra değişti; planı yenileyin."));
        }
    }
    Ok(())
}

fn validate_scope(request: &PlanRequest, projects: &[Project]) -> Result<(), PlanError> {
    match request.scope {
        Scope::Global if request.project.is_some() => Err(fail("BAD_REQUEST", "Genel kapsam proje kimliği taşıyamaz.")),
        Scope::Project => {
            let id = request.project.as_deref().ok_or_else(|| fail("BAD_REQUEST", "Proje kapsamı için proje kimliği gerekli."))?;
            projects.iter().any(|project| project.id == id)
                .then_some(())
                .ok_or_else(|| fail("BAD_REQUEST", "Proje kayıtlı değil."))
        }
        Scope::Global => Ok(()),
    }
}

fn find_binding<'a>(inventory: &'a Inventory, request: &PlanRequest) -> Result<&'a Binding, PlanError> {
    let matches: Vec<&Binding> = inventory.bindings.iter()
        .filter(|binding| binding.asset_key == request.asset_key && binding.client == request.client)
        .filter(|binding| match request.scope {
            Scope::Global => binding.scope == Scope::Global,
            Scope::Project => binding.scope == Scope::Project
                && binding.project.as_ref().map(|project| project.id.as_str()) == request.project.as_deref(),
        })
        .collect();
    match matches.as_slice() {
        [] => Err(fail("TARGET_NOT_FOUND", "Seçilen kayıt güncel envanterde bulunamadı.")),
        [binding] => Ok(*binding),
        _ => Err(fail("AMBIGUOUS_TARGET", "Aynı kayıt birden fazla dosyada bulundu; kaynak seçimi gerekli.")),
    }
}

fn plan_document(text: &str, format: SourceFormat, container: &str, name: &str) -> Result<(Value, String, Vec<String>), PlanError> {
    let body = text.strip_prefix('\u{feff}').unwrap_or(text);
    let (before, after_text) = match format {
        SourceFormat::Json => {
            let document: Value = serde_json::from_str(body)
                .map_err(|_| fail("SOURCE_CHANGED", "Hedef dosya artık geçerli JSON değil; planı yenileyin."))?;
            let entry = document.get(container).and_then(Value::as_object).and_then(|servers| servers.get(name)).cloned()
                .ok_or_else(|| fail("SOURCE_CHANGED", "Hedef kayıt dosyada artık yok; planı yenileyin."))?;
            if !entry.is_object() {
                return Err(fail("SOURCE_CHANGED", "Hedef kayıt beklenen biçimde değil; planı yenileyin."));
            }
            let after = crate::integrations::apply::remove_member(text, format, container, name)
                .map_err(|error| fail(error.code, &error.message))?;
            (security::redact_json(&entry), after)
        }
        SourceFormat::Jsonc => {
            let document = crate::integrations::apply::parse_jsonc(text)
                .map_err(|error| fail(error.code, &error.message))?;
            let entry = document.get(container).and_then(Value::as_object).and_then(|servers| servers.get(name)).cloned()
                .ok_or_else(|| fail("SOURCE_CHANGED", "Hedef kayıt dosyada artık yok; planı yenileyin."))?;
            if !entry.is_object() {
                return Err(fail("SOURCE_CHANGED", "Hedef kayıt beklenen biçimde değil; planı yenileyin."));
            }
            let after = crate::integrations::apply::remove_member(text, format, container, name)
                .map_err(|error| fail(error.code, &error.message))?;
            (security::redact_json(&entry), after)
        }
        SourceFormat::Toml => {
            let document: toml::Value = body.parse()
                .map_err(|_| fail("SOURCE_CHANGED", "Hedef dosya artık geçerli TOML değil; planı yenileyin."))?;
            let entry = document.get(container).and_then(toml::Value::as_table).and_then(|servers| servers.get(name)).cloned()
                .ok_or_else(|| fail("SOURCE_CHANGED", "Hedef kayıt dosyada artık yok; planı yenileyin."))?;
            let before = serde_json::to_value(&entry)
                .map_err(|_| fail("INTERNAL", "TOML kaydı önizlemeye dönüştürülemedi."))?;
            let after = crate::integrations::apply::remove_member(text, format, container, name)
                .map_err(|error| fail(error.code, &error.message))?;
            (security::redact_json(&before), after)
        }
        _ => return Err(fail("UNSUPPORTED_CAPABILITY", "Bu kaynak biçimi için biçim korumalı plan yok.")),
    };
    Ok((before, sha256(after_text.as_bytes()), vec![format!("- {container}.{name}")]))
}

fn container_for(client: ClientId) -> &'static str {
    match client {
        ClientId::Codex => "mcp_servers",
        ClientId::OpenCode => "mcp",
        _ => "mcpServers",
    }
}

fn digest_for(plan: &OperationPlan) -> String {
    let mut copy = plan.clone();
    copy.plan_digest.clear();
    serde_json::to_vec(&copy).map(|bytes| sha256(&bytes)).unwrap_or_default()
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or(0)
}

fn fail(code: &str, message: &str) -> PlanError {
    PlanError { code: code.into(), message: message.into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::discovery::Environment;
    use crate::integrations::inventory;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    struct Temp(PathBuf);
    impl Temp {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("hub-planner-{label}-{}-{nanos}", std::process::id()));
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
        fn config(&self) -> PathBuf { self.0.join("home/.claude.json") }
    }
    impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

    fn fixture(label: &str) -> (Temp, Environment) {
        let temp = Temp::new(label);
        fs::write(temp.config(), r#"{"mcpServers":{"postgres":{"command":"npx","args":["server-postgres"],"env":{"API_KEY":"sk-live-secret"}},"respected-brain":{"command":"respected-brain"}}}"#).unwrap();
        let env = temp.env();
        (temp, env)
    }

    fn selector(inventory: &Inventory, name: &str) -> PlanRequest {
        let asset = inventory.assets.iter().find(|asset| asset.name == name).unwrap();
        PlanRequest {
            operation: Operation::RemoveMcp,
            asset_key: asset.key.clone(),
            client: ClientId::Claude,
            scope: Scope::Global,
            project: None,
        }
    }

    #[test]
    fn plan_is_read_only_redacted_and_includes_hash_diff_and_projects() {
        let (temp, env) = fixture("preview");
        let project_root = temp.0.join("work/demo");
        fs::create_dir_all(&project_root).unwrap();
        let projects = vec![Project { id: "project:demo".into(), root: project_root.to_string_lossy().into_owned() }];
        let inventory = inventory::scan(&env, &projects);
        let before = fs::read(temp.config()).unwrap();
        let plan = plan(&inventory, &projects, &selector(&inventory, "postgres")).unwrap();
        let after = fs::read(temp.config()).unwrap();
        assert_eq!(before, after, "planning must not write");
        assert_eq!(plan.read_only, true);
        assert_eq!(plan.targets.len(), 1);
        assert_eq!(plan.targets[0].diff, vec!["- mcpServers.postgres".to_string()]);
        assert_eq!(plan.affected_projects, projects);
        assert!(!plan.targets[0].before.to_string().contains("sk-live-secret"));
        assert!(plan.targets[0].before.to_string().contains("[redacted]"));
        assert_ne!(plan.targets[0].before_sha256, plan.targets[0].after_sha256);
        assert!(!plan.plan_digest.is_empty());
        verify(&plan, plan.created_at).unwrap();
    }

    #[test]
    fn respected_owned_entry_is_protected() {
        let (_temp, env) = fixture("owned");
        let inventory = inventory::scan(&env, &[]);
        let error = plan(&inventory, &[], &selector(&inventory, "respected-brain")).unwrap_err();
        assert_eq!(error.code, "OWNER_PROTECTED");
    }

    #[test]
    fn forged_expired_and_changed_plans_are_rejected() {
        let (temp, env) = fixture("stale");
        let inventory = inventory::scan(&env, &[]);
        let request = selector(&inventory, "postgres");
        let stale = plan_at(&inventory, &[], &request, 100).unwrap();

        let mut forged = stale.clone();
        forged.plan_digest = "deadbeef".into();
        assert_eq!(verify(&forged, 200).unwrap_err().code, "INVALID_PLAN");
        assert_eq!(verify(&stale, stale.expires_at).unwrap_err().code, "PLAN_EXPIRED");

        fs::write(temp.config(), r#"{"mcpServers":{"postgres":{"command":"changed"}}}"#).unwrap();
        assert_eq!(verify(&stale, 200).unwrap_err().code, "SOURCE_CHANGED");
    }

    #[test]
    fn unsupported_capability_is_rejected() {
        let (_temp, env) = fixture("unsupported");
        let mut inventory = inventory::scan(&env, &[]);
        for capability in &mut inventory.capabilities {
            if capability.client == ClientId::Claude { capability.plan = false; }
        }
        let error = plan(&inventory, &[], &selector(&inventory, "postgres")).unwrap_err();
        assert_eq!(error.code, "UNSUPPORTED_CAPABILITY");
    }
}
