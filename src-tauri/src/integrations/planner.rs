//! Read-only change planner for third-party integrations.
//!
//! A plan is a closed preview: it reads the current source, proves the expected
//! entry is still there, and returns a deterministic redacted diff plus hashes.
//! It never writes, creates backups or executes a client.

use crate::integrations::model::{
    Asset, AssetKind, Binding, ClientId, Inventory, Owner, Project, Scope, SourceFormat,
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
    DetachMcp,
    EnableMcp,
    DisableMcp,
    MoveMcp,
    CopyMcp,
    BindMcp,
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
    #[serde(default)]
    pub target_scope: Option<Scope>,
    #[serde(default)]
    pub target_project: Option<String>,
    #[serde(default)]
    pub target_client: Option<ClientId>,
    #[serde(default)]
    pub target_clients: Vec<ClientId>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<ClientId>,
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
    #[serde(default)]
    pub target_client: Option<ClientId>,
    #[serde(default)]
    pub source_path_id: Option<String>,
    #[serde(default)]
    pub noop: bool,
}

pub fn plan(inventory: &Inventory, projects: &[Project], request: &PlanRequest) -> Result<OperationPlan, PlanError> {
    plan_at(inventory, projects, request, unix_now())
}

pub fn plan_at(inventory: &Inventory, projects: &[Project], request: &PlanRequest, now: u64) -> Result<OperationPlan, PlanError> {
    validate_scope(request, projects)?;
    let binding = find_binding(inventory, request)?;
    let asset = inventory.assets.iter().find(|asset| asset.key == request.asset_key)
        .ok_or_else(|| fail("TARGET_NOT_FOUND", "Seçilen varlık envanterde bulunamadı; envanteri yenileyin."))?;
    let capability = inventory.capabilities.iter().find(|item| item.client == binding.client);
    let supported = capability.map(|item| match request.operation {
        Operation::RemoveMcp | Operation::DetachMcp => item.plan,
        Operation::EnableMcp => item.enable,
        Operation::DisableMcp => item.disable,
        Operation::MoveMcp | Operation::CopyMcp | Operation::BindMcp => item.plan,
    }).unwrap_or(false);
    if !supported || asset.kind != AssetKind::Mcp {
        let reason = capability.and_then(|item| item.toggle_reason.as_deref())
            .filter(|_| !matches!(request.operation, Operation::RemoveMcp | Operation::DetachMcp))
            .map(|reason| format!(" {reason}"))
            .unwrap_or_default();
        return Err(fail("UNSUPPORTED_CAPABILITY", &format!("Bu istemci veya varlık türü için değişiklik planı desteklenmiyor.{reason}")));
    }
    if binding.managed || matches!(binding.source.owner, Owner::Product { .. } | Owner::Catalog) {
        return Err(fail("OWNER_PROTECTED", "Bu kayıt Respected ürününe veya kataloğa ait; harici motorla değiştirilemez."));
    }
    if matches!(request.operation, Operation::RemoveMcp | Operation::DetachMcp) && is_symlink(&binding.source.path) {
        return Err(fail("SYMLINK_TARGET", "Hedef symlink; güvenli ayırma için reddedildi."));
    }
    if request.operation == Operation::BindMcp {
        return plan_bind(inventory, projects, request, binding, asset, now);
    }
    if matches!(request.operation, Operation::MoveMcp | Operation::CopyMcp) {
        return plan_transfer(inventory, projects, request, binding, asset, now);
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
            plan_document(&text, binding.source.format, container, &asset.name, request.operation)?,
        _ => return Err(fail("UNSUPPORTED_CAPABILITY", "Bu kaynak biçimi için biçim korumalı plan henüz desteklenmiyor.")),
    };
    let mut risks = Vec::new();
    if matches!(binding.scope, Scope::Global) && !projects.is_empty() {
        let effect = if matches!(request.operation, Operation::RemoveMcp | Operation::DetachMcp) { "ayrıldığında" } else { "değiştirildiğinde" };
        risks.push(format!("Genel kayıt {effect} bu istemciyi kullanan tüm projeler etkilenebilir."));
    }
    if matches!(request.operation, Operation::EnableMcp | Operation::DisableMcp) {
        risks.push("İstemci yeniden başlatılmadan yeni etkinlik durumu görünmeyebilir.".to_string());
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
            client: None,
        }],
        affected_projects,
        risks,
        created_at: now,
        expires_at: now.saturating_add(PLAN_TTL_SECONDS),
        read_only: true,
        target_client: None,
        source_path_id: None,
        noop: false,
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
    validate_scope_pair(request.scope, request.project.as_deref(), projects, "Kaynak")?;
    let transfer = matches!(request.operation, Operation::MoveMcp | Operation::CopyMcp | Operation::BindMcp);
    match (transfer, request.target_scope) {
        (true, None) => return Err(fail("BAD_REQUEST", "Taşıma/kopyalama için hedef kapsam gerekli.")),
        (false, Some(_)) => return Err(fail("BAD_REQUEST", "Bu işlem hedef kapsam taşıyamaz.")),
        _ => {
            if let Some(scope) = request.target_scope {
                validate_scope_pair(scope, request.target_project.as_deref(), projects, "Hedef")?;
            }
        }
    }
    match (request.operation == Operation::BindMcp, requested_target_clients(request)?) {
        (true, None) => Err(fail("BAD_REQUEST", "Bind icin hedef istemci gerekli.")),
        (false, Some(_)) => Err(fail("BAD_REQUEST", "Bu islem hedef istemci tasiyamaz.")),
        _ => Ok(()),
    }
}

fn requested_target_clients(request: &PlanRequest) -> Result<Option<Vec<ClientId>>, PlanError> {
    if request.target_client.is_some() && !request.target_clients.is_empty() {
        return Err(fail("BAD_REQUEST", "Hedef istemci tek alan veya liste olarak verilmeli."));
    }
    let mut clients = request.target_clients.clone();
    if let Some(client) = request.target_client {
        clients.push(client);
    }
    if clients.is_empty() {
        return Ok(None);
    }
    let mut seen = std::collections::HashSet::new();
    if clients.iter().any(|client| !seen.insert(*client)) {
        return Err(fail("BAD_REQUEST", "Ayni hedef istemci birden fazla kez verilemez."));
    }
    Ok(Some(clients))
}

fn validate_scope_pair(scope: Scope, project: Option<&str>, projects: &[Project], label: &str) -> Result<(), PlanError> {
    match scope {
        Scope::Global if project.is_some() => Err(fail("BAD_REQUEST", &format!("{label} genel kapsamı proje kimliği taşıyamaz."))),
        Scope::Project => {
            let id = project.ok_or_else(|| fail("BAD_REQUEST", &format!("{label} proje kapsamı için proje kimliği gerekli.")))?;
            projects.iter().any(|item| item.id == id)
                .then_some(())
                .ok_or_else(|| fail("BAD_REQUEST", &format!("{label} proje kayıtlı değil.")))
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


fn plan_bind(
    inventory: &Inventory,
    projects: &[Project],
    request: &PlanRequest,
    binding: &Binding,
    asset: &Asset,
    now: u64,
) -> Result<OperationPlan, PlanError> {
    let target_clients = requested_target_clients(request)?
        .ok_or_else(|| fail("BAD_REQUEST", "Bind icin hedef istemci gerekli."))?;
    let target_scope = request.target_scope.ok_or_else(|| fail("BAD_REQUEST", "Bind icin hedef kapsam gerekli."))?;
    let target_project = match target_scope {
        Scope::Global => None,
        Scope::Project => projects.iter()
            .find(|project| Some(project.id.as_str()) == request.target_project.as_deref())
            .cloned(),
    };
    if !matches!(binding.source.format, SourceFormat::Json | SourceFormat::Jsonc | SourceFormat::Toml) {
        return Err(fail("UNSUPPORTED_CAPABILITY", "Bu kaynak bicimi ice alinamiyor."));
    }
    if is_symlink(&binding.source.path) {
        return Err(fail("SYMLINK_TARGET", "Kaynak yapilandirma dosyasi symlink; guvenli ice alma icin reddedildi."));
    }

    let mut destinations = Vec::new();
    let mut destination_paths = std::collections::HashSet::new();
    for target_client in target_clients.iter().copied() {
        if !inventory.capabilities.iter().any(|item| item.client == target_client && item.plan) {
            return Err(fail("UNSUPPORTED_CAPABILITY", "Hedef istemci bind islemini desteklemiyor."));
        }
        let in_target = |candidate: &Binding| candidate.client == target_client
            && candidate.scope == target_scope
            && (target_scope == Scope::Global
                || candidate.project.as_ref().map(|project| project.id.as_str())
                    == target_project.as_ref().map(|project| project.id.as_str()));
        let target_bindings: Vec<&Binding> = inventory.bindings.iter().filter(|candidate| in_target(candidate)).collect();
        let mut path_ids: Vec<&str> = target_bindings.iter().map(|candidate| candidate.source.path_id.as_str()).collect();
        path_ids.sort_unstable();
        path_ids.dedup();
        if path_ids.is_empty() {
            return Err(fail("TARGET_NOT_FOUND", "Hedef istemcide ice alinacak yapilandirma dosyasi bulunamadi."));
        }
        if path_ids.len() > 1 {
            return Err(fail("AMBIGUOUS_TARGET", "Hedef istemcide birden fazla yapilandirma dosyasi var; kaynak secimi gerekli."));
        }
        let destination = target_bindings.iter().find(|candidate| candidate.source.path_id == path_ids[0])
            .copied().ok_or_else(|| fail("TARGET_NOT_FOUND", "Hedef yapilandirma dosyasi bulunamadi."))?;
        if same_path(&binding.source.path, &destination.source.path) {
            return Err(fail("BAD_REQUEST", "Kaynak ve hedef ayni yapilandirma dosyasi."));
        }
        if (binding.source.format == SourceFormat::Toml) != (destination.source.format == SourceFormat::Toml) {
            return Err(fail("UNSUPPORTED_CAPABILITY", "JSON ve TOML istemcileri arasinda ice alma henuz desteklenmiyor."));
        }
        let same_name = |candidate: &&Binding| candidate.asset_key != asset.key && inventory.assets.iter()
            .any(|other| other.key == candidate.asset_key && other.name == asset.name);
        if target_bindings.iter().any(same_name) {
            return Err(fail("TARGET_EXISTS", "Hedef kapsamda ayni adli farkli bir kayit zaten var."));
        }
        if target_bindings.iter().any(|candidate| candidate.asset_key == asset.key) {
            continue;
        }
        if is_symlink(&destination.source.path) {
            return Err(fail("SYMLINK_TARGET", "Hedef yapilandirma dosyasi symlink; guvenli yazma icin reddedildi."));
        }
        if !destination_paths.insert(destination.source.path_id.clone()) {
            return Err(fail("AMBIGUOUS_TARGET", "Birden fazla hedef istemci ayni yapilandirma dosyasini gosteriyor."));
        }
        destinations.push((target_client, destination));
    }

    let noop = destinations.is_empty();
    let entry = if noop {
        None
    } else {
        let source_text = security::read_text(Path::new(&binding.source.path), Limits::default())
            .map_err(|error| fail("SOURCE_CHANGED", &format!("Kaynak dosya okunamadi: {}", error.message)))?;
        let source_hash = sha256(source_text.as_bytes());
        if binding.source.fingerprint.as_deref().is_some_and(|fingerprint| fingerprint != source_hash) {
            return Err(fail("SOURCE_CHANGED", "Kaynak dosya taramadan sonra degisti; plani yenileyin."));
        }
        Some(crate::integrations::apply::read_entry(
            &source_text, binding.source.format, container_for(binding.client), &asset.name,
        ).map_err(|error| fail(error.code, &error.message))?)
    };

    let mut targets = Vec::new();
    for (target_client, destination) in destinations {
        let destination_text = security::read_text(Path::new(&destination.source.path), Limits::default())
            .map_err(|error| fail("SOURCE_CHANGED", &format!("Hedef dosya okunamadi: {}", error.message)))?;
        let destination_hash = sha256(destination_text.as_bytes());
        if destination.source.fingerprint.as_deref().is_some_and(|fingerprint| fingerprint != destination_hash) {
            return Err(fail("SOURCE_CHANGED", "Hedef dosya taramadan sonra degisti; plani yenileyin."));
        }
        let (entry_text, entry_value) = entry.as_ref().ok_or_else(|| fail("INVALID_PLAN", "Ice alma girdisi hazirlanamadi."))?;
        let after = crate::integrations::apply::insert_member(
            &destination_text, destination.source.format, container_for(target_client), &asset.name, entry_text,
        ).map_err(|error| fail(error.code, &error.message))?;
        targets.push(PlanTarget {
            path: destination.source.path.clone(),
            path_id: destination.source.path_id.clone(),
            format: destination.source.format,
            before_sha256: destination_hash,
            after_sha256: sha256(after.as_bytes()),
            before: Value::Null,
            after: security::redact_json(entry_value),
            diff: vec![format!("+ {}.{} ({})", container_for(target_client), asset.name, scope_label(target_scope, target_project.as_ref()))],
            client: Some(target_client),
        });
    }
    let affected_projects = if noop {
        Vec::new()
    } else {
        match target_scope {
            Scope::Global => projects.to_vec(),
            Scope::Project => target_project.iter().cloned().collect(),
        }
    };
    let mut risks = Vec::new();
    if noop {
        risks.push("Bu kayit hedef kapsamda zaten bagli; yazma yapilmayacak.".to_string());
    } else {
        risks.push("Kaynak dosya degismez; mevcut gizli degerler hedefe kopyalanir ve onizlemede gizlenir.".to_string());
    }
    if target_clients.len() > 1 {
        risks.push("Toplu islem ya hep ya hic uygulanir; bir hedef basarisizsa yazilan dosyalar geri alinir.".to_string());
    }
    let mut result = OperationPlan {
        plan_digest: String::new(),
        operation: request.operation,
        asset_key: asset.key.clone(),
        name: asset.name.clone(),
        kind: asset.kind,
        client: target_clients[0],
        scope: target_scope,
        project: target_project.clone(),
        owner: binding.source.owner.clone(),
        managed: false,
        targets,
        affected_projects,
        risks,
        created_at: now,
        expires_at: now.saturating_add(PLAN_TTL_SECONDS),
        read_only: true,
        target_client: (target_clients.len() == 1).then_some(target_clients[0]),
        source_path_id: Some(binding.source.path_id.clone()),
        noop,
    };
    result.plan_digest = digest_for(&result);
    Ok(result)
}

fn is_symlink(path: &str) -> bool {
    fs::symlink_metadata(Path::new(path)).map(|metadata| metadata.file_type().is_symlink()).unwrap_or(false)
}

fn plan_transfer(
    inventory: &Inventory,
    projects: &[Project],
    request: &PlanRequest,
    binding: &Binding,
    asset: &Asset,
    now: u64,
) -> Result<OperationPlan, PlanError> {
    let target_scope = request.target_scope.ok_or_else(|| fail("BAD_REQUEST", "Hedef kapsam gerekli."))?;
    let target_project = match target_scope {
        Scope::Global => None,
        Scope::Project => projects.iter()
            .find(|project| Some(project.id.as_str()) == request.target_project.as_deref())
            .cloned(),
    };
    if target_scope == binding.scope
        && target_project.as_ref().map(|project| project.id.as_str())
            == binding.project.as_ref().map(|project| project.id.as_str())
    {
        return Err(fail("BAD_REQUEST", "Kaynak ve hedef kapsam aynı."));
    }
    let in_target = |candidate: &Binding| candidate.client == binding.client
        && candidate.scope == target_scope
        && (target_scope == Scope::Global
            || candidate.project.as_ref().map(|project| project.id.as_str())
                == target_project.as_ref().map(|project| project.id.as_str()));
    if inventory.bindings.iter().any(|candidate| candidate.asset_key == asset.key && in_target(candidate)) {
        return Err(fail("TARGET_EXISTS", "Hedef kapsamda aynı MCP kaydı zaten var; önce onu kaldırın."));
    }
    let destination = inventory.bindings.iter().find(|candidate| in_target(candidate))
        .ok_or_else(|| fail("TARGET_NOT_FOUND", "Hedef kapsamda bu istemci için okunabilir yapılandırma dosyası yok; etkileri önizlemek için hedef dosya mevcut olmalı."))?;
    if same_path(&binding.source.path, &destination.source.path) {
        return Err(fail("BAD_REQUEST", "Kaynak ve hedef aynı dosyayı gösteriyor."));
    }

    let source_text = security::read_text(Path::new(&binding.source.path), Limits::default())
        .map_err(|error| fail("SOURCE_CHANGED", &format!("Kaynak dosya okunamadı: {}", error.message)))?;
    let source_hash = sha256(source_text.as_bytes());
    if binding.source.fingerprint.as_deref().is_some_and(|fingerprint| fingerprint != source_hash) {
        return Err(fail("SOURCE_CHANGED", "Kaynak dosya taramadan sonra değişti; planı yenileyin."));
    }
    let destination_text = security::read_text(Path::new(&destination.source.path), Limits::default())
        .map_err(|error| fail("SOURCE_CHANGED", &format!("Hedef dosya okunamadı: {}", error.message)))?;
    let destination_hash = sha256(destination_text.as_bytes());
    if destination.source.fingerprint.as_deref().is_some_and(|fingerprint| fingerprint != destination_hash) {
        return Err(fail("SOURCE_CHANGED", "Hedef dosya taramadan sonra değişti; planı yenileyin."));
    }

    let container = container_for(binding.client);
    let (entry_text, entry_value) = crate::integrations::apply::read_entry(
        &source_text, binding.source.format, container, &asset.name,
    ).map_err(|error| fail(error.code, &error.message))?;
    let destination_after = crate::integrations::apply::insert_member(
        &destination_text, destination.source.format, container, &asset.name, &entry_text,
    ).map_err(|error| fail(error.code, &error.message))?;

    let mut targets = Vec::new();
    if request.operation == Operation::MoveMcp {
        let source_after = crate::integrations::apply::remove_member(
            &source_text, binding.source.format, container, &asset.name,
        ).map_err(|error| fail(error.code, &error.message))?;
        targets.push(PlanTarget {
            path: binding.source.path.clone(),
            path_id: binding.source.path_id.clone(),
            format: binding.source.format,
            before_sha256: source_hash,
            after_sha256: sha256(source_after.as_bytes()),
            before: security::redact_json(&entry_value),
            after: Value::Null,
            diff: vec![format!("- {container}.{}", asset.name)],
            client: None,
        });
    }
    targets.push(PlanTarget {
        path: destination.source.path.clone(),
        path_id: destination.source.path_id.clone(),
        format: destination.source.format,
        before_sha256: destination_hash,
        after_sha256: sha256(destination_after.as_bytes()),
        before: Value::Null,
        after: security::redact_json(&entry_value),
        diff: vec![format!("+ {container}.{} ({})", asset.name, scope_label(target_scope, target_project.as_ref()))],
        client: None,
    });

    let global_changed = target_scope == Scope::Global
        || (request.operation == Operation::MoveMcp && binding.scope == Scope::Global);
    let mut affected_projects = if global_changed { projects.to_vec() } else { Vec::new() };
    if !global_changed {
        if let Some(project) = binding.project.clone() {
            affected_projects.push(project);
        }
        if let Some(project) = target_project.clone() {
            if !affected_projects.iter().any(|item| item.id == project.id) {
                affected_projects.push(project);
            }
        }
    }
    let mut risks = Vec::new();
    if request.operation == Operation::MoveMcp && binding.scope == Scope::Global {
        risks.push("Genel kayıt kaldırıldığında bu istemciyi kullanan diğer projelerde de görünmez olur.".to_string());
    }
    if request.operation == Operation::CopyMcp && binding.scope == Scope::Global {
        risks.push("Proje kopyası aynı adlı genel kaydı yalnız bu projede gölgeler; diğer projeler genel kaydı kullanmaya devam eder.".to_string());
    }
    if target_scope == Scope::Global {
        risks.push("Hedef genel kayıt tüm projelerde görünür olur.".to_string());
    }
    risks.push("İstemci yeniden başlatılmadan yeni kapsam görünmeyebilir.".to_string());

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
        targets,
        affected_projects,
        risks,
        created_at: now,
        expires_at: now.saturating_add(PLAN_TTL_SECONDS),
        read_only: true,
        target_client: None,
        source_path_id: None,
        noop: false,
    };
    result.plan_digest = digest_for(&result);
    Ok(result)
}

fn same_path(left: &str, right: &str) -> bool {
    let normalize = |value: &str| value.replace('\\', "/").trim_end_matches('/').to_string();
    let left = normalize(left);
    let right = normalize(right);
    if cfg!(windows) { left.to_lowercase() == right.to_lowercase() } else { left == right }
}

fn scope_label(scope: Scope, project: Option<&Project>) -> String {
    match (scope, project) {
        (Scope::Global, _) => "genel".to_string(),
        (Scope::Project, Some(project)) => format!("proje: {}", project.id),
        _ => "proje".to_string(),
    }
}

fn plan_document(
    text: &str,
    format: SourceFormat,
    container: &str,
    name: &str,
    operation: Operation,
) -> Result<(Value, String, Vec<String>), PlanError> {
    let body = text.strip_prefix('\u{feff}').unwrap_or(text);
    let enabled = operation == Operation::EnableMcp;
    let (entry, after_text) = match format {
        SourceFormat::Json => {
            let document: Value = serde_json::from_str(body)
                .map_err(|_| fail("SOURCE_CHANGED", "Hedef dosya artık geçerli JSON değil; planı yenileyin."))?;
            let entry = document.get(container).and_then(Value::as_object).and_then(|servers| servers.get(name)).cloned()
                .ok_or_else(|| fail("SOURCE_CHANGED", "Hedef kayıt dosyada artık yok; planı yenileyin."))?;
            if !entry.is_object() {
                return Err(fail("SOURCE_CHANGED", "Hedef kayıt beklenen biçimde değil; planı yenileyin."));
            }
            let after = match operation {
                Operation::RemoveMcp | Operation::DetachMcp => crate::integrations::apply::remove_member(text, format, container, name),
                _ => crate::integrations::apply::set_enabled(text, format, container, name, enabled),
            }.map_err(|error| fail(error.code, &error.message))?;
            (entry, after)
        }
        SourceFormat::Jsonc => {
            let document = crate::integrations::apply::parse_jsonc(text)
                .map_err(|error| fail(error.code, &error.message))?;
            let entry = document.get(container).and_then(Value::as_object).and_then(|servers| servers.get(name)).cloned()
                .ok_or_else(|| fail("SOURCE_CHANGED", "Hedef kayıt dosyada artık yok; planı yenileyin."))?;
            if !entry.is_object() {
                return Err(fail("SOURCE_CHANGED", "Hedef kayıt beklenen biçimde değil; planı yenileyin."));
            }
            let after = match operation {
                Operation::RemoveMcp | Operation::DetachMcp => crate::integrations::apply::remove_member(text, format, container, name),
                _ => crate::integrations::apply::set_enabled(text, format, container, name, enabled),
            }.map_err(|error| fail(error.code, &error.message))?;
            (entry, after)
        }
        SourceFormat::Toml => {
            let document: toml::Value = body.parse()
                .map_err(|_| fail("SOURCE_CHANGED", "Hedef dosya artık geçerli TOML değil; planı yenileyin."))?;
            let entry = document.get(container).and_then(toml::Value::as_table).and_then(|servers| servers.get(name)).cloned()
                .ok_or_else(|| fail("SOURCE_CHANGED", "Hedef kayıt dosyada artık yok; planı yenileyin."))?;
            let before = serde_json::to_value(&entry)
                .map_err(|_| fail("INTERNAL", "TOML kaydı önizlemeye dönüştürülemedi."))?;
            let after = match operation {
                Operation::RemoveMcp | Operation::DetachMcp => crate::integrations::apply::remove_member(text, format, container, name),
                _ => crate::integrations::apply::set_enabled(text, format, container, name, enabled),
            }.map_err(|error| fail(error.code, &error.message))?;
            (before, after)
        }
        _ => return Err(fail("UNSUPPORTED_CAPABILITY", "Bu kaynak biçimi için format korumalı plan yok.")),
    };
    let diff = if matches!(operation, Operation::RemoveMcp | Operation::DetachMcp) {
        vec![format!("- {container}.{name}")]
    } else {
        vec![format!("~ {container}.{name}.enabled = {enabled}")]
    };
    Ok((security::redact_json(&entry), sha256(after_text.as_bytes()), diff))
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
            target_scope: None,
            target_project: None,
            target_client: None,
            target_clients: Vec::new(),
        }
    }

    fn codex_fixture(label: &str) -> (Temp, Environment) {
        let temp = Temp::new(label);
        let dir = temp.0.join("home/.codex");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("config.toml"), "# codex comment\n[mcp_servers.postgres]\ncommand = \"npx\"\nenabled = true\n").unwrap();
        let env = temp.env();
        (temp, env)
    }

    fn toggle_selector(inventory: &Inventory, name: &str, client: ClientId, operation: Operation) -> PlanRequest {
        let asset = inventory.assets.iter().find(|asset| asset.name == name).unwrap();
        PlanRequest {
            operation,
            asset_key: asset.key.clone(),
            client,
            scope: Scope::Global,
            project: None,
            target_scope: None,
            target_project: None,
            target_client: None,
            target_clients: Vec::new(),
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
    fn detach_plan_only_targets_the_requested_client() {
        let (temp, env) = codex_fixture("detach-isolated");
        let claude = temp.0.join("home/.claude.json");
        fs::write(&claude, r#"{"mcpServers":{"postgres":{"command":"npx","args":["server"]}}}"#).unwrap();
        let inventory = inventory::scan(&env, &[]);
        let codex = temp.0.join("home/.codex/config.toml");
        let claude_before = fs::read(&claude).unwrap();
        let codex_before = fs::read(&codex).unwrap();
        let plan = plan(&inventory, &[], &toggle_selector(&inventory, "postgres", ClientId::Claude, Operation::DetachMcp)).unwrap();
        assert_eq!(plan.operation, Operation::DetachMcp);
        assert_eq!(plan.targets.len(), 1);
        assert!(plan.targets[0].path.ends_with(".claude.json"));
        assert_eq!(plan.targets[0].diff, vec!["- mcpServers.postgres".to_string()]);
        assert_eq!(fs::read(&claude).unwrap(), claude_before);
        assert_eq!(fs::read(&codex).unwrap(), codex_before);
        assert_eq!(share_owner(&inventory, &plan), Some(ClientId::Codex));
    }

    fn share_owner(inventory: &Inventory, plan: &OperationPlan) -> Option<ClientId> {
        inventory.bindings.iter()
            .find(|binding| binding.client != plan.client
                && inventory.assets.iter().any(|asset| asset.key == binding.asset_key && asset.name == plan.name))
            .map(|binding| binding.client)
    }

    #[test]
    fn detach_rejects_symlink_at_plan_time() {
        let (temp, env) = fixture("detach-symlink");
        let inventory = inventory::scan(&env, &[]);
        let real = temp.0.join("real.json");
        fs::rename(temp.config(), &real).unwrap();
        if link_file(&real, &temp.config()).is_err() {
            eprintln!("symlink privilege unavailable; symlink test skipped");
            return;
        }
        let error = plan(&inventory, &[], &toggle_selector(&inventory, "postgres", ClientId::Claude, Operation::DetachMcp)).unwrap_err();
        assert_eq!(error.code, "SYMLINK_TARGET");
    }

    #[test]
    fn detach_rejects_source_changed_after_scan() {
        let (temp, env) = fixture("detach-changed");
        let inventory = inventory::scan(&env, &[]);
        fs::write(temp.config(), r#"{"mcpServers":{"postgres":{"command":"changed"}}}"#).unwrap();
        let error = plan(&inventory, &[], &toggle_selector(&inventory, "postgres", ClientId::Claude, Operation::DetachMcp)).unwrap_err();
        assert_eq!(error.code, "SOURCE_CHANGED");
    }

    #[test]
    fn detach_keeps_respected_ownership_protected() {
        let (_temp, env) = fixture("detach-owned");
        let inventory = inventory::scan(&env, &[]);
        let error = plan(&inventory, &[], &toggle_selector(&inventory, "respected-brain", ClientId::Claude, Operation::DetachMcp)).unwrap_err();
        assert_eq!(error.code, "OWNER_PROTECTED");
    }

    #[cfg(windows)]
    fn link_file(target: &Path, link: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(target, link)
    }
    #[cfg(unix)]
    fn link_file(target: &Path, link: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(target, link)
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
    #[test]
    fn codex_toggle_plan_is_read_only_and_explains_restart() {
        let (temp, env) = codex_fixture("toggle");
        let inventory = inventory::scan(&env, &[]);
        let path = temp.0.join("home/.codex/config.toml");
        let before = fs::read(&path).unwrap();
        let plan = plan(&inventory, &[], &toggle_selector(&inventory, "postgres", ClientId::Codex, Operation::DisableMcp)).unwrap();
        assert_eq!(fs::read(&path).unwrap(), before, "planning must not write");
        assert_eq!(plan.targets[0].diff, vec!["~ mcp_servers.postgres.enabled = false".to_string()]);
        assert!(plan.risks.iter().any(|risk| risk.contains("yeniden ba")));
        assert_ne!(plan.targets[0].before_sha256, plan.targets[0].after_sha256);
        verify(&plan, plan.created_at).unwrap();
    }

    #[test]
    fn codex_toggle_does_not_touch_other_client_files_for_same_asset() {
        let (temp, env) = codex_fixture("isolated");
        let claude = temp.0.join("home/.claude.json");
        let claude_text = r#"{"mcpServers":{"postgres":{"command":"npx","enabled":true}}}"#;
        fs::write(&claude, claude_text).unwrap();
        let inventory = inventory::scan(&env, &[]);
        let codex = temp.0.join("home/.codex/config.toml");
        let claude_before = fs::read(&claude).unwrap();
        let codex_before = fs::read(&codex).unwrap();
        let plan = plan(&inventory, &[], &toggle_selector(&inventory, "postgres", ClientId::Codex, Operation::DisableMcp)).unwrap();
        assert!(plan.targets[0].path.contains(".codex") && plan.targets[0].path.ends_with("config.toml"));
        assert_eq!(fs::read(&claude).unwrap(), claude_before);
        assert_eq!(fs::read(&codex).unwrap(), codex_before);
    }

    #[test]
    fn unsupported_clients_reject_toggle_without_touching_files() {
        let temp = Temp::new("unsupported-toggle");
        let home = temp.0.join("home");
        fs::create_dir_all(home.join(".cursor")).unwrap();
        fs::create_dir_all(home.join(".gemini")).unwrap();
        let claude = home.join(".claude.json");
        let cursor = home.join(".cursor/mcp.json");
        let gemini = home.join(".gemini/settings.json");
        let text = r#"{"mcpServers":{"postgres":{"command":"npx","enabled":true}}}"#;
        fs::write(&claude, text).unwrap();
        fs::write(&cursor, text).unwrap();
        fs::write(&gemini, text).unwrap();
        let env = temp.env();
        let inventory = inventory::scan(&env, &[]);
        for (client, path) in [(ClientId::Claude, &claude), (ClientId::Cursor, &cursor), (ClientId::Gemini, &gemini)] {
            let before = fs::read(path).unwrap();
            let error = plan(&inventory, &[], &toggle_selector(&inventory, "postgres", client, Operation::EnableMcp)).unwrap_err();
            assert_eq!(error.code, "UNSUPPORTED_CAPABILITY");
            assert_eq!(fs::read(path).unwrap(), before);
        }
    }

    struct ScopePaths {
        global: PathBuf,
        project: PathBuf,
    }

    fn write_scope_fixture(
        client: ClientId,
        temp: &Temp,
        root: &PathBuf,
        global_has: bool,
        project_has: bool,
        project_file: bool,
    ) -> ScopePaths {
        let _ = fs::remove_dir_all(root);
        fs::create_dir_all(root).unwrap();
        let global_entry = r#"{"command":"npx","args":["server"],"env":{"TOKEN":"sk-live-secret"}}"#;
        let local_entry = r#"{"command":"local"}"#;
        let paths = match client {
            ClientId::Claude => ScopePaths {
                global: temp.0.join("home/.claude.json"),
                project: root.join(".mcp.json"),
            },
            ClientId::Codex => {
                fs::create_dir_all(root.join(".codex")).unwrap();
                ScopePaths { global: temp.0.join("home/.codex/config.toml"), project: root.join(".codex/config.toml") }
            }
            ClientId::OpenCode => {
                fs::create_dir_all(temp.0.join("home/.config/opencode")).unwrap();
                ScopePaths { global: temp.0.join("home/.config/opencode/opencode.jsonc"), project: root.join("opencode.jsonc") }
            }
            _ => unreachable!(),
        };
        let mut global_text = match client {
            ClientId::Claude => format!(
                "{{\"mcpServers\":{{\"other\":{local_entry}{}}}}}\n",
                if global_has { format!(",\"postgres\":{global_entry}") } else { String::new() }
            ),
            ClientId::Codex => format!(
                "[projects.'{}']\ntrust_level = \"trusted\"\n\n[mcp_servers.other]\ncommand = \"local\"\n{}",
                root.to_string_lossy(),
                if global_has { "\n[mcp_servers.postgres]\ncommand = \"npx\"\nargs = [\"server\"]\n[mcp_servers.postgres.env]\nTOKEN = \"sk-live-secret\"\n" } else { "" }
            ),
            ClientId::OpenCode => format!(
                "{{\n  // global comment\n  \"mcp\": {{\"other\":{local_entry}{}}}\n}}\n",
                if global_has { format!(",\"postgres\":{global_entry}") } else { String::new() }
            ),
            _ => unreachable!(),
        };
        if client == ClientId::Codex && !global_has {
            global_text.push('\n');
        }
        fs::create_dir_all(paths.global.parent().unwrap()).unwrap();
        fs::write(&paths.global, global_text).unwrap();
        if project_file {
            let project_text = match client {
                ClientId::Claude => format!(
                    "{{\n  \"mcpServers\": {{\"local\":{local_entry}{}}}\n}}\n",
                    if project_has { format!(",\"postgres\":{global_entry}") } else { String::new() }
                ),
                ClientId::OpenCode => format!(
                    "{{\n  // project comment\n  \"mcp\": {{\"local\":{local_entry}{}}}\n}}\n",
                    if project_has { format!(",\"postgres\":{global_entry}") } else { String::new() }
                ),
                ClientId::Codex => format!(
                    "[mcp_servers.local]\ncommand = \"local\"\n{}",
                    if project_has { "\n[mcp_servers.postgres]\ncommand = \"npx\"\nargs = [\"server\"]\n[mcp_servers.postgres.env]\nTOKEN = \"sk-live-secret\"\n" } else { "" }
                ),
                _ => unreachable!(),
            };
            fs::create_dir_all(paths.project.parent().unwrap()).unwrap();
            fs::write(&paths.project, project_text).unwrap();
        }
        paths
    }

    fn scope_request(
        inventory: &Inventory,
        name: &str,
        client: ClientId,
        operation: Operation,
        scope: Scope,
        project: Option<String>,
        target_scope: Scope,
        target_project: Option<String>,
    ) -> PlanRequest {
        let asset = inventory.assets.iter().find(|asset| asset.name == name).unwrap();
        PlanRequest {
            operation,
            asset_key: asset.key.clone(),
            client,
            scope,
            project,
            target_scope: Some(target_scope),
            target_project,
            target_client: None,
            target_clients: Vec::new(),
        }
    }

    fn has_binding(inventory: &Inventory, client: ClientId, name: &str, scope: Scope, project: &str) -> bool {
        inventory.bindings.iter().any(|binding| {
            binding.client == client
                && binding.scope == scope
                && binding.project.as_ref().map(|item| item.id.as_str()) == Some(project)
                && inventory.assets.iter().any(|asset| asset.key == binding.asset_key && asset.name == name)
        })
    }

    fn simulate_transfer(client: ClientId, paths: &ScopePaths, plan: &OperationPlan) -> (String, String) {
        let container = container_for(client);
        let source_path = if plan.scope == Scope::Global { &paths.global } else { &paths.project };
        let (entry, _) = crate::integrations::apply::read_entry(
            &fs::read_to_string(source_path).unwrap(),
            if client == ClientId::Codex { SourceFormat::Toml } else { SourceFormat::Jsonc },
            container,
            &plan.name,
        ).unwrap();
        let global_text = fs::read_to_string(&paths.global).unwrap();
        let project_text = fs::read_to_string(&paths.project).unwrap();
        let global_format = if client == ClientId::Codex { SourceFormat::Toml } else { SourceFormat::Jsonc };
        let project_format = if client == ClientId::Codex { SourceFormat::Toml } else { SourceFormat::Jsonc };
        let insert_global = || crate::integrations::apply::insert_member(&global_text, global_format, container, &plan.name, &entry).unwrap();
        let insert_project = || crate::integrations::apply::insert_member(&project_text, project_format, container, &plan.name, &entry).unwrap();
        let remove_global = || crate::integrations::apply::remove_member(&global_text, global_format, container, &plan.name).unwrap();
        let remove_project = || crate::integrations::apply::remove_member(&project_text, project_format, container, &plan.name).unwrap();
        let (global_after, project_after) = match (plan.operation, plan.scope) {
            (Operation::CopyMcp, Scope::Global) => (global_text.clone(), insert_project()),
            (Operation::CopyMcp, _) => (insert_global(), project_text.clone()),
            (Operation::MoveMcp, Scope::Global) => (remove_global(), insert_project()),
            (Operation::MoveMcp, _) => (insert_global(), remove_project()),
            _ => unreachable!(),
        };
        (global_after, project_after)
    }

    #[test]
    fn supported_clients_cover_six_scope_scenarios() {
        for client in [ClientId::Claude, ClientId::Codex, ClientId::OpenCode] {
            let root = std::env::temp_dir().join(format!("hub-scope-{}-{}", client.as_str(), std::process::id()));
            let _ = fs::remove_dir_all(&root);
            let temp = Temp::new(&format!("scope-{}", client.as_str()));
            let project_root = temp.0.join("work/demo");
            let project = Project { id: "project:demo".into(), root: project_root.to_string_lossy().into_owned() };

            for (operation, source_scope, target_scope) in [
                (Operation::CopyMcp, Scope::Global, Scope::Project),
                (Operation::MoveMcp, Scope::Global, Scope::Project),
                (Operation::CopyMcp, Scope::Project, Scope::Global),
                (Operation::MoveMcp, Scope::Project, Scope::Global),
            ] {
                let paths = write_scope_fixture(client, &temp, &project_root, source_scope == Scope::Global, source_scope == Scope::Project, true);
                let env = temp.env();
                let inventory = inventory::scan(&env, std::slice::from_ref(&project));
                let source_project = if source_scope == Scope::Project { Some(project.id.clone()) } else { None };
                let target_project = if target_scope == Scope::Project { Some(project.id.clone()) } else { None };
                let request = scope_request(&inventory, "postgres", client, operation, source_scope, source_project, target_scope, target_project);
                let plan = plan(&inventory, std::slice::from_ref(&project), &request).unwrap();
                verify(&plan, plan.created_at).unwrap();
                assert_eq!(plan.read_only, true);
                assert_eq!(plan.targets.len(), if operation == Operation::MoveMcp { 2 } else { 1 });
                let in_project = target_scope == Scope::Project;
                assert!(!plan.affected_projects.is_empty());
                let _ = in_project;
                assert!(plan.risks.iter().any(|risk| risk.contains("yeniden ba")));
                let (global_after, project_after) = simulate_transfer(client, &paths, &plan);
                if operation == Operation::MoveMcp && source_scope == Scope::Global {
                    assert!(!global_after.contains("postgres"));
                    assert!(project_after.contains("postgres"));
                } else if operation == Operation::MoveMcp {
                    assert!(!project_after.contains("postgres"));
                    assert!(global_after.contains("postgres"));
                } else {
                    assert!(global_after.contains("postgres") && project_after.contains("postgres"));
                }
                if client == ClientId::Codex {
                    let transferred = if target_scope == Scope::Project { &project_after } else { &global_after };
                    assert!(transferred.contains("[mcp_servers.postgres.env]"), "nested env kept");
                    assert!(!transferred.contains("\n[env]"), "env not lifted to root");
                }
                fs::write(&paths.global, global_after).unwrap();
                fs::write(&paths.project, project_after).unwrap();
                let reread = inventory::scan(&env, std::slice::from_ref(&project));
                let project_expected = !(operation == Operation::MoveMcp && source_scope == Scope::Project);
                assert_eq!(has_binding(&reread, client, "postgres", Scope::Project, &project.id), project_expected, "{client:?} project binding");
                let global_expected = match (operation, source_scope) {
                    (Operation::CopyMcp, _) => true,
                    (Operation::MoveMcp, Scope::Global) => false,
                    (Operation::MoveMcp, Scope::Project) => true,
                    _ => true,
                };
                assert_eq!(reread.bindings.iter().any(|binding| binding.client == client && binding.scope == Scope::Global
                    && reread.assets.iter().any(|asset| asset.key == binding.asset_key && asset.name == "postgres")), global_expected, "{client:?} global binding");
            }

            let paths = write_scope_fixture(client, &temp, &project_root, true, true, true);
            let env = temp.env();
            let inventory = inventory::scan(&env, std::slice::from_ref(&project));
            let before_global = fs::read(&paths.global).unwrap();
            let before_project = fs::read(&paths.project).unwrap();
            let request = scope_request(&inventory, "postgres", client, Operation::CopyMcp, Scope::Global, None, Scope::Project, Some(project.id.clone()));
            let error = plan(&inventory, std::slice::from_ref(&project), &request).unwrap_err();
            assert_eq!(error.code, "TARGET_EXISTS");
            assert_eq!(fs::read(&paths.global).unwrap(), before_global);
            assert_eq!(fs::read(&paths.project).unwrap(), before_project);

            let paths = write_scope_fixture(client, &temp, &project_root, true, false, false);
            let env = temp.env();
            let inventory = inventory::scan(&env, std::slice::from_ref(&project));
            let request = scope_request(&inventory, "postgres", client, Operation::CopyMcp, Scope::Global, None, Scope::Project, Some(project.id.clone()));
            let error = plan(&inventory, std::slice::from_ref(&project), &request).unwrap_err();
            assert_eq!(error.code, "TARGET_NOT_FOUND");
            let _ = paths;
            let _ = fs::remove_dir_all(&root);
        }
    }

    #[test]
    fn codex_untrusted_project_cannot_receive_scope_copy() {
        let temp = Temp::new("scope-untrusted");
        let project_root = temp.0.join("work/untrusted");
        fs::create_dir_all(project_root.join(".codex")).unwrap();
        let global = temp.0.join("home/.codex/config.toml");
        fs::create_dir_all(global.parent().unwrap()).unwrap();
        fs::write(&global, "[mcp_servers.postgres]\ncommand = \"npx\"\n").unwrap();
        fs::write(project_root.join(".codex/config.toml"), "[mcp_servers.local]\ncommand = \"local\"\n").unwrap();
        let project = Project { id: "project:untrusted".into(), root: project_root.to_string_lossy().into_owned() };
        let env = temp.env();
        let inventory = inventory::scan(&env, std::slice::from_ref(&project));
        assert!(inventory.issues.iter().any(|issue| issue.code == "project_untrusted"));
        let request = scope_request(&inventory, "postgres", ClientId::Codex, Operation::CopyMcp, Scope::Global, None, Scope::Project, Some(project.id.clone()));
        let error = plan(&inventory, std::slice::from_ref(&project), &request).unwrap_err();
        assert_eq!(error.code, "TARGET_NOT_FOUND");
    }

    #[test]
    fn owned_source_cannot_be_moved_or_copied() {
        let (temp, env) = fixture("scope-owned");
        let project_root = temp.0.join("work/demo");
        fs::create_dir_all(&project_root).unwrap();
        let project = Project { id: "project:demo".into(), root: project_root.to_string_lossy().into_owned() };
        fs::write(project_root.join(".mcp.json"), r#"{"mcpServers":{"local":{"command":"x"}}}"#).unwrap();
        let inventory = inventory::scan(&env, std::slice::from_ref(&project));
        let request = scope_request(&inventory, "respected-brain", ClientId::Claude, Operation::CopyMcp, Scope::Global, None, Scope::Project, Some(project.id.clone()));
        let error = plan(&inventory, std::slice::from_ref(&project), &request).unwrap_err();
        assert_eq!(error.code, "OWNER_PROTECTED");
    }

    fn bind_request(
        inventory: &Inventory,
        name: &str,
        client: ClientId,
        target_client: ClientId,
        target_project: Option<&str>,
    ) -> PlanRequest {
        let asset = inventory.assets.iter().find(|asset| asset.name == name).unwrap();
        PlanRequest {
            operation: Operation::BindMcp,
            asset_key: asset.key.clone(),
            client,
            scope: Scope::Global,
            project: None,
            target_scope: Some(Scope::Project),
            target_project: target_project.map(str::to_string),
            target_client: Some(target_client),
            target_clients: Vec::new(),
        }
    }

    #[test]
    fn bind_preview_targets_only_the_project_and_keeps_source() {
        let (temp, env) = fixture("bind-preview");
        let project_root = temp.0.join("work/demo");
        fs::create_dir_all(&project_root).unwrap();
        fs::write(project_root.join(".mcp.json"), r#"{"mcpServers":{"local":{"command":"x"}}}"#).unwrap();
        let project = Project { id: "project:demo".into(), root: project_root.to_string_lossy().into_owned() };
        let inventory = inventory::scan(&env, std::slice::from_ref(&project));
        let source_before = fs::read(temp.config()).unwrap();
        let request = bind_request(&inventory, "postgres", ClientId::Claude, ClientId::Claude, Some(&project.id));
        let plan = plan(&inventory, std::slice::from_ref(&project), &request).unwrap();
        assert_eq!(source_before, fs::read(temp.config()).unwrap());
        assert_eq!(plan.client, ClientId::Claude);
        assert_eq!(plan.scope, Scope::Project);
        assert_eq!(plan.affected_projects, vec![project]);
        assert_eq!(plan.targets.len(), 1);
        assert!(plan.targets[0].path.ends_with(".mcp.json"));
        assert!(plan.targets[0].after.to_string().contains("[redacted]"));
        assert!(!plan.noop);
    }

    #[test]
    fn bind_is_idempotent_when_target_already_has_the_same_asset() {
        let (temp, env) = fixture("bind-duplicate");
        let project_root = temp.0.join("work/demo");
        fs::create_dir_all(&project_root).unwrap();
        fs::write(project_root.join(".mcp.json"), r#"{"mcpServers":{"postgres":{"command":"npx","args":["server-postgres"],"env":{"API_KEY":"sk-live-secret"}},"local":{"command":"x"}}}"#).unwrap();
        let project = Project { id: "project:demo".into(), root: project_root.to_string_lossy().into_owned() };
        let inventory = inventory::scan(&env, std::slice::from_ref(&project));
        let request = bind_request(&inventory, "postgres", ClientId::Claude, ClientId::Claude, Some(&project.id));
        let plan = plan(&inventory, std::slice::from_ref(&project), &request).unwrap();
        assert!(plan.noop);
        assert!(plan.targets.is_empty());
    }

    #[test]
    fn bind_without_a_target_config_is_rejected() {
        let (temp, env) = fixture("bind-missing");
        let project_root = temp.0.join("work/empty");
        fs::create_dir_all(&project_root).unwrap();
        let project = Project { id: "project:empty".into(), root: project_root.to_string_lossy().into_owned() };
        let inventory = inventory::scan(&env, std::slice::from_ref(&project));
        let request = bind_request(&inventory, "postgres", ClientId::Claude, ClientId::Claude, Some(&project.id));
        let error = plan(&inventory, std::slice::from_ref(&project), &request).unwrap_err();
        assert_eq!(error.code, "TARGET_NOT_FOUND");
    }

    #[test]
    fn bind_to_two_clients_keeps_each_target_separate() {
        let temp = Temp::new("bind-two-clients");
        let env = temp.env();
        fs::write(temp.config(), r#"{"mcpServers":{"postgres":{"command":"npx","args":["server"]}}}"#).unwrap();
        fs::create_dir_all(env.home.join(".cursor")).unwrap();
        fs::write(env.home.join(".cursor/mcp.json"), r#"{"mcpServers":{"keep":{"command":"x"}}}"#).unwrap();
        fs::create_dir_all(env.home.join(".gemini")).unwrap();
        fs::write(env.home.join(".gemini/settings.json"), r#"{"mcpServers":{"keep":{"command":"x"}}}"#).unwrap();
        let inventory = inventory::scan(&env, &[]);
        let asset = inventory.assets.iter().find(|asset| asset.name == "postgres").unwrap();
        let request = PlanRequest {
            operation: Operation::BindMcp,
            asset_key: asset.key.clone(),
            client: ClientId::Claude,
            scope: Scope::Global,
            project: None,
            target_scope: Some(Scope::Global),
            target_project: None,
            target_client: None,
            target_clients: vec![ClientId::Cursor, ClientId::Gemini],
        };
        let plan = plan(&inventory, &[], &request).unwrap();
        assert_eq!(plan.targets.len(), 2);
        assert_eq!(plan.target_client, None);
        let cursor = plan.targets.iter().find(|target| target.client == Some(ClientId::Cursor)).unwrap();
        let gemini = plan.targets.iter().find(|target| target.client == Some(ClientId::Gemini)).unwrap();
        assert!(cursor.path.ends_with("mcp.json"));
        assert!(gemini.path.ends_with("settings.json"));
        assert!(plan.risks.iter().any(|risk| risk.contains("ya hep ya hic")));
    }

    #[test]
    fn bind_rejects_duplicate_or_conflicting_target_clients() {
        let (_temp, env) = fixture("bind-target-validation");
        let inventory = inventory::scan(&env, &[]);
        let asset = inventory.assets.iter().find(|asset| asset.name == "postgres").unwrap();
        let mut request = PlanRequest {
            operation: Operation::BindMcp,
            asset_key: asset.key.clone(),
            client: ClientId::Claude,
            scope: Scope::Global,
            project: None,
            target_scope: Some(Scope::Global),
            target_project: None,
            target_client: None,
            target_clients: vec![ClientId::Cursor, ClientId::Cursor],
        };
        assert_eq!(plan(&inventory, &[], &request).unwrap_err().code, "BAD_REQUEST");
        request.target_clients = vec![ClientId::Cursor];
        request.target_client = Some(ClientId::Gemini);
        assert_eq!(plan(&inventory, &[], &request).unwrap_err().code, "BAD_REQUEST");
    }
    #[test]
    fn bind_rejects_respected_owned_source() {
        let (temp, env) = fixture("bind-owned");
        let project_root = temp.0.join("work/demo");
        fs::create_dir_all(&project_root).unwrap();
        fs::write(project_root.join(".mcp.json"), r#"{"mcpServers":{"local":{"command":"x"}}}"#).unwrap();
        let project = Project { id: "project:demo".into(), root: project_root.to_string_lossy().into_owned() };
        let inventory = inventory::scan(&env, std::slice::from_ref(&project));
        let request = bind_request(&inventory, "respected-brain", ClientId::Claude, ClientId::Claude, Some(&project.id));
        let error = plan(&inventory, std::slice::from_ref(&project), &request).unwrap_err();
        assert_eq!(error.code, "OWNER_PROTECTED");
    }
}
