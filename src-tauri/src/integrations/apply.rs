//! Transactional writer for a planned third-party integration change.
//!
//! The writer accepts only a server-issued `OperationPlan`; it never takes a path
//! from the UI. It re-verifies the plan digest, TTL and source hashes, re-derives
//! every binding from a fresh inventory, edits text without reserializing the
//! document, then performs allowlisted atomic replaces with sidecar backups,
//! readback and an undo journal.

use crate::integrations::model::{
    AssetKind, Binding, ClientId, Inventory, Owner, Project, Scope, SourceFormat,
};
use crate::integrations::planner::{self, Operation, OperationPlan};
use crate::integrations::security::{self, Limits};
use crate::integrations::undo::{self, Journal, JournalFile};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AppliedFile {
    pub path: String,
    pub path_id: String,
    pub before_sha256: String,
    pub after_sha256: String,
    pub backup_name: String,
    pub bytes_before: usize,
    pub bytes_after: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Applied {
    pub plan_digest: String,
    pub transaction_id: String,
    pub files: Vec<AppliedFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ApplyError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_backup: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditError {
    pub code: &'static str,
    pub message: String,
}

struct Prepared {
    path: PathBuf,
    path_id: String,
    text: String,
    after_text: String,
    before_sha256: String,
    after_sha256: String,
    permissions: fs::Permissions,
    backup: PathBuf,
}

/// Apply a plan and record it in the managed integration journal, so the write can
/// be undone and a killed process can be recovered on the next start.
pub fn apply(
    plan: &OperationPlan,
    inventory: &Inventory,
    projects: &[Project],
    transaction_id: &str,
) -> Result<Applied, ApplyError> {
    apply_at_journaled(plan, inventory, projects, transaction_id, unix_now(), &Journal::default())
}

/// Apply without a journal. Only tests use this; production goes through `apply`.
pub fn apply_at(
    plan: &OperationPlan,
    inventory: &Inventory,
    projects: &[Project],
    transaction_id: &str,
    now: u64,
) -> Result<Applied, ApplyError> {
    apply_with(plan, inventory, projects, transaction_id, now, &mut default_writer, None)
}

pub fn apply_at_journaled(
    plan: &OperationPlan,
    inventory: &Inventory,
    projects: &[Project],
    transaction_id: &str,
    now: u64,
    journal: &Journal,
) -> Result<Applied, ApplyError> {
    apply_with(plan, inventory, projects, transaction_id, now, &mut default_writer, Some(journal))
}

fn apply_with<F>(
    plan: &OperationPlan,
    inventory: &Inventory,
    projects: &[Project],
    transaction_id: &str,
    now: u64,
    writer: &mut F,
    journal: Option<&Journal>,
) -> Result<Applied, ApplyError>
where
    F: FnMut(&Path, &[u8]) -> io::Result<()>,
{
    validate_transaction_id(transaction_id)?;
    planner::verify(plan, now).map_err(|error| ApplyError {
        code: error.code,
        message: error.message,
        recovery_backup: None,
    })?;
    if plan.targets.is_empty() {
        return Err(apply_error("INVALID_PLAN", "Plan hi\u{00e7}bir hedef dosya i\u{00e7}ermiyor."));
    }
    let source = fresh_binding(plan, inventory, projects)?;
    let container = container_for(plan.client);
    let entry_text = if matches!(plan.operation, Operation::MoveMcp | Operation::CopyMcp) {
        let text = security::read_text(Path::new(&source.source.path), Limits::default())
            .map_err(|error| apply_error("SOURCE_CHANGED", &error.message))?;
        let hash = sha256(text.as_bytes());
        if source.source.fingerprint.as_deref().is_some_and(|fingerprint| fingerprint != hash) {
            return Err(apply_error("SOURCE_CHANGED", "Kaynak dosya taramadan sonra de\u{011f}i\u{015f}ti; plan\u{0131} yenileyin."));
        }
        Some(read_entry(&text, source.source.format, container, &plan.name)
            .map_err(|error| ApplyError { code: error.code.into(), message: error.message, recovery_backup: None })?.0)
    } else {
        None
    };

    let mut prepared = Vec::new();
    for target in &plan.targets {
        let binding = binding_by_path_id(inventory, plan.client, &target.path_id)?;
        if binding.managed || matches!(binding.source.owner, Owner::Product { .. } | Owner::Catalog) {
            return Err(apply_error("OWNER_PROTECTED", "Hedef kay\u{0131}t Respected \u{00fc}r\u{00fc}n\u{00fc}ne veya katalo\u{011f}a ait."));
        }
        let path = PathBuf::from(&binding.source.path);
        if !same_file(Path::new(&target.path), &path)? {
            return Err(apply_error("TARGET_NOT_FOUND", "Plan hedefi g\u{00fc}ncel kaynak dosyas\u{0131}yla ayn\u{0131} de\u{011f}il."));
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| apply_error("TARGET_NOT_FOUND", "Hedef dosya art\u{0131}k okunam\u{0131}yor."))?;
        if metadata.file_type().is_symlink() {
            return Err(apply_error("SYMLINK_TARGET", "Hedef symlink; g\u{00fc}venli yazma i\u{00e7}in reddedildi."));
        }
        if !metadata.is_file() {
            return Err(apply_error("UNSUPPORTED_CAPABILITY", "Hedef normal bir dosya de\u{011f}il."));
        }
        let text = security::read_text(&path, Limits::default())
            .map_err(|error| apply_error("SOURCE_CHANGED", &error.message))?;
        let before_sha256 = sha256(text.as_bytes());
        if before_sha256 != target.before_sha256
            || binding.source.fingerprint.as_deref().is_some_and(|fingerprint| fingerprint != before_sha256)
        {
            return Err(apply_error("SOURCE_CHANGED", "Hedef dosya plan veya taramadan sonra de\u{011f}i\u{015f}ti; plan\u{0131} yenileyin."));
        }
        let is_source = binding.source.path_id == source.source.path_id;
        let after_text = match plan.operation {
            Operation::RemoveMcp => remove_member(&text, target.format, container, &plan.name),
            Operation::EnableMcp => set_enabled(&text, target.format, container, &plan.name, true),
            Operation::DisableMcp => set_enabled(&text, target.format, container, &plan.name, false),
            Operation::MoveMcp if is_source => remove_member(&text, target.format, container, &plan.name),
            Operation::MoveMcp | Operation::CopyMcp => {
                if is_source {
                    return Err(apply_error("BAD_REQUEST", "Kopyalama plan\u{0131} kaynak dosyay\u{0131} de\u{011f}i\u{015f}tiremez."));
                }
                let entry = entry_text.as_deref().unwrap_or_default();
                insert_member(&text, target.format, container, &plan.name, entry)
            }
        }.map_err(|error| ApplyError {
            code: error.code.into(),
            message: error.message,
            recovery_backup: None,
        })?;
        let after_sha256 = sha256(after_text.as_bytes());
        if after_sha256 != target.after_sha256 {
            return Err(apply_error("INVALID_PLAN", "Plan \u{00f6}nizlemesi deterministik d\u{00fc}zenlemeyle e\u{015f}le\u{015f}miyor."));
        }
        let backup = backup_path(&path, transaction_id)?;
        prepared.push(Prepared {
            path,
            path_id: target.path_id.clone(),
            text,
            after_text,
            before_sha256,
            after_sha256,
            permissions: metadata.permissions(),
            backup,
        });
    }

    let _guard = undo::WRITE_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    for item in &prepared {
        let bytes = fs::read(&item.path)
            .map_err(|_| apply_error("SOURCE_CHANGED", "Hedef dosya art\u{0131}k okunam\u{0131}yor; plan\u{0131} yenileyin."))?;
        if sha256(&bytes) != item.before_sha256 {
            return Err(apply_error("SOURCE_CHANGED", "Hedef dosya kilitten \u{00f6}nce de\u{011f}i\u{015f}ti; plan\u{0131} yenileyin."));
        }
    }

    if let Some(journal) = journal {
        let files = prepared.iter().map(|item| JournalFile {
            path: item.path.clone(),
            path_id: item.path_id.clone(),
            before_sha256: item.before_sha256.clone(),
            after_sha256: item.after_sha256.clone(),
            backup_name: item.backup.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default(),
            bytes_before: item.text.len(),
            bytes_after: item.after_text.len(),
        }).collect();
        journal.prepare(&plan.plan_digest, &operation_name(plan.operation), transaction_id, files, unix_now())
            .map_err(|error| apply_error(&error.code, &error.message))?;
    }

    let mut backups = Vec::new();
    for item in &prepared {
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&item.backup) {
            Ok(file) => file,
            Err(error) => {
                cleanup_backups(&backups);
                if let Some(journal) = journal { let _ = journal.mark_failed(transaction_id); }
                let code = if error.kind() == io::ErrorKind::AlreadyExists { "TRANSACTION_REUSED" } else { "WRITE_FAILED" };
                return Err(apply_error(code, &format!("Yedek olu\u{015f}turulamad\u{0131}: {error}")));
            }
        };
        if let Err(error) = file.write_all(item.text.as_bytes()).and_then(|_| file.sync_all()) {
            drop(file);
            let _ = fs::remove_file(&item.backup);
            cleanup_backups(&backups);
            if let Some(journal) = journal { let _ = journal.mark_failed(transaction_id); }
            return Err(apply_error("WRITE_FAILED", &format!("Yedek yaz\u{0131}lamad\u{0131}: {error}")));
        }
        drop(file);
        backups.push(item.backup.clone());
    }

    let mut written = Vec::new();
    for (index, item) in prepared.iter().enumerate() {
        let temp = temp_path(&item.path, transaction_id)?;
        let result = writer(&temp, item.after_text.as_bytes())
            .and_then(|_| fs::set_permissions(&temp, item.permissions.clone()))
            .and_then(|_| fs::rename(&temp, &item.path));
        if let Err(error) = result {
            let _ = fs::remove_file(&temp);
            return rollback(&prepared, &written, journal, transaction_id, "WRITE_FAILED", format!("Ge\u{00e7}ici dosya yaz\u{0131}lamad\u{0131}: {error}"));
        }
        sync_parent(&item.path);
        written.push(index);
    }

    for item in &prepared {
        let ok = fs::read(&item.path)
            .map(|bytes| bytes == item.after_text.as_bytes() && sha256(&bytes) == item.after_sha256)
            .unwrap_or(false);
        if !ok {
            return rollback(&prepared, &written, journal, transaction_id, "READBACK_FAILED", "Yazma sonras\u{0131} do\u{011f}rulama ba\u{015f}ar\u{0131}s\u{0131}z; \u{00f6}nceki dosyalar geri y\u{00fc}klendi.".to_string());
        }
    }

    if let Some(journal) = journal {
        if let Err(error) = journal.mark_committed(transaction_id) {
            return rollback(&prepared, &written, Some(journal), transaction_id, "WRITE_FAILED", format!("\u{0130}\u{015f}lem g\u{00fc}nl\u{00fc}\u{011f}\u{00fc} tamamlanamad\u{0131}: {}", error.message));
        }
    }

    let files = prepared.iter().map(|item| AppliedFile {
        path: item.path.to_string_lossy().into_owned(),
        path_id: item.path_id.clone(),
        before_sha256: item.before_sha256.clone(),
        after_sha256: item.after_sha256.clone(),
        backup_name: item.backup.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default(),
        bytes_before: item.text.len(),
        bytes_after: item.after_text.len(),
    }).collect();
    Ok(Applied { plan_digest: plan.plan_digest.clone(), transaction_id: transaction_id.to_string(), files })
}

fn rollback(
    prepared: &[Prepared],
    written: &[usize],
    journal: Option<&Journal>,
    transaction_id: &str,
    code: &str,
    message: String,
) -> Result<Applied, ApplyError> {
    let mut recovery = None;
    let mut restored = vec![false; prepared.len()];
    for &index in written {
        let item = &prepared[index];
        match restore(&item.path, &item.backup, item.permissions.clone()) {
            Ok(()) => {
                restored[index] = true;
                let _ = fs::remove_file(&item.backup);
            }
            Err(_) => recovery = Some(item.backup.to_string_lossy().into_owned()),
        }
    }
    for (index, item) in prepared.iter().enumerate() {
        if written.contains(&index) && !restored[index] {
            continue;
        }
        let _ = fs::remove_file(&item.backup);
    }
    if let Some(journal) = journal {
        let _ = journal.mark_failed(transaction_id);
    }
    if let Some(backup) = recovery {
        return Err(ApplyError {
            code: "RECOVERY_REQUIRED".into(),
            message: format!("{message} Geri y\u{00fc}kleme tamamlanamad\u{0131}; yedek korunuyor."),
            recovery_backup: Some(backup),
        });
    }
    Err(apply_error(code, &message))
}

fn cleanup_backups(backups: &[PathBuf]) {
    for backup in backups {
        let _ = fs::remove_file(backup);
    }
}

fn operation_name(operation: Operation) -> String {
    serde_json::to_value(operation).ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn binding_by_path_id<'a>(
    inventory: &'a Inventory,
    client: ClientId,
    path_id: &str,
) -> Result<&'a Binding, ApplyError> {
    let matches: Vec<&Binding> = inventory.bindings.iter()
        .filter(|binding| binding.client == client && binding.source.path_id == path_id)
        .collect();
    match matches.as_slice() {
        [binding] => Ok(*binding),
        [] => Err(apply_error("TARGET_NOT_FOUND", "Plan hedefi g\u{00fc}ncel envanterde bulunamad\u{0131}.")),
        _ => Err(apply_error("AMBIGUOUS_TARGET", "Ayn\u{0131} dosya birden fazla kay\u{0131}tla e\u{015f}le\u{015f}iyor.")),
    }
}

fn fresh_binding<'a>(
    plan: &OperationPlan,
    inventory: &'a Inventory,
    projects: &[Project],
) -> Result<&'a Binding, ApplyError> {
    if !inventory.assets.iter().any(|asset| asset.key == plan.asset_key && asset.kind == AssetKind::Mcp) {
        return Err(apply_error("TARGET_NOT_FOUND", "Plan varlığı güncel envanterde yok."));
    }
    if plan.scope == Scope::Project {
        let id = plan.project.as_ref().map(|project| project.id.as_str());
        if !projects.iter().any(|project| Some(project.id.as_str()) == id) {
            return Err(apply_error("TARGET_NOT_FOUND", "Plan projesi kayıtlı değil."));
        }
    }
    let matches: Vec<&Binding> = inventory.bindings.iter()
        .filter(|binding| binding.asset_key == plan.asset_key && binding.client == plan.client)
        .filter(|binding| match plan.scope {
            Scope::Global => binding.scope == Scope::Global,
            Scope::Project => binding.scope == Scope::Project
                && binding.project.as_ref().map(|project| project.id.as_str())
                    == plan.project.as_ref().map(|project| project.id.as_str()),
        })
        .collect();
    match matches.as_slice() {
        [binding] => Ok(*binding),
        [] => Err(apply_error("TARGET_NOT_FOUND", "Plan kaydı güncel envanterde bulunamadı.")),
        _ => Err(apply_error("AMBIGUOUS_TARGET", "Aynı kayıt birden fazla dosyada bulundu.")),
    }
}

fn validate_transaction_id(id: &str) -> Result<(), ApplyError> {
    if id.is_empty() || id.len() > 64 || !id.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') {
        return Err(apply_error("BAD_REQUEST", "İşlem kimliği 1-64 güvenli karakter olmalı."));
    }
    Ok(())
}

fn same_file(first: &Path, second: &Path) -> Result<bool, ApplyError> {
    let first = fs::canonicalize(first).map_err(|_| apply_error("TARGET_NOT_FOUND", "Plan yolu çözümlenemedi."))?;
    let second = fs::canonicalize(second).map_err(|_| apply_error("TARGET_NOT_FOUND", "Kaynak yolu çözümlenemedi."))?;
    #[cfg(windows)]
    return Ok(first.to_string_lossy().eq_ignore_ascii_case(&second.to_string_lossy()));
    #[cfg(not(windows))]
    Ok(first == second)
}

fn backup_path(path: &Path, transaction_id: &str) -> Result<PathBuf, ApplyError> {
    sibling(path, transaction_id, "backup")
}

fn temp_path(path: &Path, transaction_id: &str) -> Result<PathBuf, ApplyError> {
    sibling(path, transaction_id, "tmp")
}

fn sibling(path: &Path, transaction_id: &str, tag: &str) -> Result<PathBuf, ApplyError> {
    let name = path.file_name().and_then(|name| name.to_str())
        .ok_or_else(|| apply_error("UNSUPPORTED_CAPABILITY", "Hedef dosya adı Unicode değil."))?;
    Ok(path.with_file_name(format!(".{name}.respected-{transaction_id}.{tag}")))
}

fn default_writer(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn restore(path: &Path, backup: &Path, permissions: fs::Permissions) -> io::Result<()> {
    let bytes = fs::read(backup)?;
    let temp = path.with_file_name(format!(
        ".{}.respected-restore-{}",
        path.file_name().and_then(|name| name.to_str()).unwrap_or("config"),
        std::process::id()
    ));
    let _ = fs::remove_file(&temp);
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::set_permissions(&temp, permissions)?;
    drop(file);
    if let Err(error) = fs::rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    let readback = fs::read(path)?;
    if readback != bytes {
        return Err(io::Error::new(io::ErrorKind::Other, "restored bytes do not match backup"));
    }
    sync_parent(path);
    Ok(())
}

fn sync_parent(path: &Path) {
    if let Some(parent) = path.parent() {
        if let Ok(directory) = File::open(parent) {
            let _ = directory.sync_all();
        }
    }
}

fn apply_error(code: &str, message: &str) -> ApplyError {
    ApplyError { code: code.into(), message: message.into(), recovery_backup: None }
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or(0)
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn container_for(client: ClientId) -> &'static str {
    match client {
        ClientId::Codex => "mcp_servers",
        ClientId::OpenCode => "mcp",
        _ => "mcpServers",
    }
}

/// Remove one member while keeping every other byte (comments, unknown keys,
/// ordering, indentation and line endings) unchanged.
pub fn remove_member(text: &str, format: SourceFormat, container: &str, name: &str) -> Result<String, EditError> {
    match format {
        SourceFormat::Json => edit_json(text, container, name, false),
        SourceFormat::Jsonc => edit_json(text, container, name, true),
        SourceFormat::Toml => edit_toml(text, container, name),
        _ => Err(edit_error("UNSUPPORTED_FORMAT", "Bu kaynak biçimi için biçim korumalı yazma yok.")),
    }
}

/// Set only the native `enabled` field of one MCP entry. Every other byte is kept and
/// the result is parsed again before it is returned.
pub fn set_enabled(
    text: &str,
    format: SourceFormat,
    container: &str,
    name: &str,
    enabled: bool,
) -> Result<String, EditError> {
    match format {
        SourceFormat::Json => edit_json_enabled(text, container, name, false, enabled),
        SourceFormat::Jsonc => edit_json_enabled(text, container, name, true, enabled),
        SourceFormat::Toml => edit_toml_enabled(text, container, name, enabled),
        _ => Err(edit_error("UNSUPPORTED_FORMAT", "Bu kaynak biçimi için biçim korumalı toggle yok.")),
    }
}

/// Read one MCP entry as its exact JSON text (or serialized TOML table) plus a parsed value.
pub fn read_entry(
    text: &str,
    format: SourceFormat,
    container: &str,
    name: &str,
) -> Result<(String, Value), EditError> {
    let body = strip_bom(text);
    match format {
        SourceFormat::Json | SourceFormat::Jsonc => {
            let jsonc = format == SourceFormat::Jsonc;
            let document = if jsonc {
                parse_jsonc(body)?
            } else {
                serde_json::from_str(body).map_err(|_| edit_error("INVALID_FORMAT", "JSON \u{00e7}\u{00f6}z\u{00fc}mlenemedi."))?
            };
            let entry = document.get(container).and_then(Value::as_object)
                .and_then(|servers| servers.get(name)).cloned()
                .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef kay\u{0131}t dosyada art\u{0131}k yok."))?;
            if !entry.is_object() {
                return Err(edit_error("SOURCE_CHANGED", "Hedef kay\u{0131}t beklenen bi\u{00e7}imde de\u{011f}il."));
            }
            let mut scanner = JsonScan::new(body, jsonc);
            scanner.skip()?;
            let (_, root_members, _) = scanner.parse_object()?;
            let section = root_members.iter().find(|member| member.key == container)
                .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef b\u{00f6}l\u{00fc}m dosyada yok."))?;
            let mut inner = JsonScan::new(body, jsonc);
            inner.index = section.value_start;
            let (_, members, _) = inner.parse_object()?;
            let member = members.iter().find(|member| member.key == name)
                .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef kay\u{0131}t dosyada yok."))?;
            Ok((body[member.value_start..member.value_end].to_string(), entry))
        }
        SourceFormat::Toml => {
            let document: toml::Value = body.parse()
                .map_err(|_| edit_error("INVALID_FORMAT", "TOML \u{00e7}\u{00f6}z\u{00fc}mlenemedi."))?;
            let entry = document.get(container).and_then(toml::Value::as_table)
                .and_then(|servers| servers.get(name)).cloned()
                .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef kay\u{0131}t dosyada art\u{0131}k yok."))?;
            let value = serde_json::to_value(&entry)
                .map_err(|_| edit_error("INVALID_FORMAT", "TOML kayd\u{0131} \u{00f6}nizlemeye d\u{00f6}n\u{00fc}\u{015f}t\u{00fc}r\u{00fc}lemedi."))?;
            let text = toml::to_string(&entry)
                .map_err(|_| edit_error("INVALID_FORMAT", "TOML kayd\u{0131} yaz\u{0131}lamad\u{0131}."))?;
            Ok((text, value))
        }
        _ => Err(edit_error("UNSUPPORTED_FORMAT", "Bu kaynak bi\u{00e7}imi i\u{00e7}in kapsam ta\u{015f}\u{0131}ma yok.")),
    }
}

/// Insert one MCP entry into an existing container. The source entry text is copied as-is
/// for JSON/JSONC; TOML reuses the source table values. Every other byte is kept.
pub fn insert_member(
    text: &str,
    format: SourceFormat,
    container: &str,
    name: &str,
    entry: &str,
) -> Result<String, EditError> {
    match format {
        SourceFormat::Json => insert_json_member(text, container, name, entry, false),
        SourceFormat::Jsonc => insert_json_member(text, container, name, entry, true),
        SourceFormat::Toml => insert_toml_member(text, container, name, entry),
        _ => Err(edit_error("UNSUPPORTED_FORMAT", "Bu kaynak bi\u{00e7}imi i\u{00e7}in kapsam ta\u{015f}\u{0131}ma yok.")),
    }
}

fn insert_json_member(
    text: &str,
    container: &str,
    name: &str,
    entry: &str,
    jsonc: bool,
) -> Result<String, EditError> {
    let body = strip_bom(text);
    let document = if jsonc {
        parse_jsonc(body)?
    } else {
        serde_json::from_str(body).map_err(|_| edit_error("INVALID_FORMAT", "JSON \u{00e7}\u{00f6}z\u{00fc}mlenemedi."))?
    };
    let servers = document.get(container).and_then(Value::as_object)
        .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef dosyada MCP b\u{00f6}l\u{00fc}m\u{00fc} yok."))?;
    if servers.contains_key(name) {
        return Err(edit_error("TARGET_EXISTS", "Hedef kapsamda ayn\u{0131} adl\u{0131} kay\u{0131}t zaten var."));
    }
    let mut scanner = JsonScan::new(body, jsonc);
    scanner.skip()?;
    let (_, root_members, _) = scanner.parse_object()?;
    let section = root_members.iter().find(|member| member.key == container)
        .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef b\u{00f6}l\u{00fc}m dosyada yok."))?;
    if body.as_bytes().get(section.value_start) != Some(&b'{') {
        return Err(edit_error("SOURCE_CHANGED", "Hedef b\u{00f6}l\u{00fc}m nesne de\u{011f}il."));
    }
    let mut inner = JsonScan::new(body, jsonc);
    inner.index = section.value_start;
    let (object_start, members, _) = inner.parse_object()?;
    let key = serde_json::to_string(name)
        .map_err(|_| edit_error("INVALID_FORMAT", "Kay\u{0131}t ad\u{0131} JSON'a d\u{00f6}n\u{00fc}\u{015f}t\u{00fc}r\u{00fc}lemedi."))?;
    let (at, inserted) = if let Some(first) = members.first() {
        let line_start = body[..first.key_start].rfind('\n').map(|index| index + 1).unwrap_or(0);
        let indent = &body[line_start..first.key_start];
        if !indent.is_empty() && indent.chars().all(|ch| ch == ' ' || ch == '\t') {
            let newline = body[line_start..].find('\n').map(|index| line_start + index).unwrap_or(body.len());
            let eol = if newline > line_start && body.as_bytes().get(newline - 1) == Some(&b'\r') { "\r\n" } else { "\n" };
            (line_start, format!("{indent}{key}: {entry},{eol}"))
        } else {
            (first.key_start, format!("{key}: {entry}, "))
        }
    } else {
        (object_start + 1, format!("{key}: {entry}"))
    };
    let mut after = String::with_capacity(body.len() + inserted.len());
    after.push_str(&body[..at]);
    after.push_str(&inserted);
    after.push_str(&body[at..]);
    let parsed = if jsonc {
        parse_jsonc(&after)?
    } else {
        serde_json::from_str(&after).map_err(|error| edit_error("INVALID_FORMAT", &format!("D\u{00fc}zenlenmi\u{015f} JSON ge\u{00e7}ersiz: {error}")))?
    };
    if parsed.get(container).and_then(Value::as_object).and_then(|servers| servers.get(name)).is_none() {
        return Err(edit_error("INVALID_FORMAT", "Eklenen kay\u{0131}t do\u{011f}rulanamad\u{0131}."));
    }
    Ok(reattach_bom(text, after))
}

fn insert_toml_member(
    text: &str,
    container: &str,
    name: &str,
    entry: &str,
) -> Result<String, EditError> {
    let body = strip_bom(text);
    let document: toml::Value = body.parse()
        .map_err(|_| edit_error("INVALID_FORMAT", "TOML \u{00e7}\u{00f6}z\u{00fc}mlenemedi."))?;
    if document.get(container).and_then(toml::Value::as_table).and_then(|servers| servers.get(name)).is_some() {
        return Err(edit_error("TARGET_EXISTS", "Hedef kapsamda ayn\u{0131} adl\u{0131} kay\u{0131}t zaten var."));
    }
    let value: toml::Value = entry.parse()
        .map_err(|_| edit_error("INVALID_FORMAT", "Ta\u{015f}\u{0131}nan TOML kayd\u{0131} \u{00e7}\u{00f6}z\u{00fc}mlenemedi."))?;
    let mut servers = toml::value::Table::new();
    servers.insert(name.to_string(), value);
    let mut root = toml::value::Table::new();
    root.insert(container.to_string(), toml::Value::Table(servers));
    let section = toml::to_string(&toml::Value::Table(root))
        .map_err(|_| edit_error("INVALID_FORMAT", "TOML kayd\u{0131} yaz\u{0131}lamad\u{0131}."))?;
    let mut after = body.to_string();
    if !after.ends_with('\n') { after.push('\n'); }
    after.push('\n');
    after.push_str(&section);
    let parsed: toml::Value = after.parse()
        .map_err(|_| edit_error("INVALID_FORMAT", "Eklenmi\u{015f} TOML ge\u{00e7}ersiz."))?;
    if parsed.get(container).and_then(toml::Value::as_table).and_then(|servers| servers.get(name)).is_none() {
        return Err(edit_error("INVALID_FORMAT", "Eklenen TOML kayd\u{0131} do\u{011f}rulanamad\u{0131}."));
    }
    Ok(reattach_bom(text, after))
}

fn edit_json_enabled(
    text: &str,
    container: &str,
    name: &str,
    jsonc: bool,
    enabled: bool,
) -> Result<String, EditError> {
    let body = strip_bom(text);
    let document = if jsonc {
        parse_jsonc(body)?
    } else {
        serde_json::from_str(body).map_err(|_| edit_error("INVALID_FORMAT", "JSON çözümlenemedi."))?
    };
    if document.get(container).and_then(Value::as_object)
        .and_then(|servers| servers.get(name)).and_then(Value::as_object).is_none()
    {
        return Err(edit_error("SOURCE_CHANGED", "Hedef kayıt dosyada nesne olarak yok."));
    }

    let mut scanner = JsonScan::new(body, jsonc);
    scanner.skip()?;
    let (_, root_members, _) = scanner.parse_object()?;
    let section = root_members.iter().find(|member| member.key == container)
        .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef bölüm dosyada yok."))?;
    if body.as_bytes().get(section.value_start) != Some(&b'{') {
        return Err(edit_error("SOURCE_CHANGED", "Hedef bölüm nesne değil."));
    }
    let mut inner = JsonScan::new(body, jsonc);
    inner.index = section.value_start;
    let (_, members, _) = inner.parse_object()?;
    let index = members.iter().position(|member| member.key == name)
        .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef kayıt dosyada yok."))?;
    let server = &members[index];
    if body.as_bytes().get(server.value_start) != Some(&b'{') {
        return Err(edit_error("SOURCE_CHANGED", "Hedef kayıt nesne değil."));
    }
    let mut server_scan = JsonScan::new(body, jsonc);
    server_scan.index = server.value_start;
    let (object_start, server_members, _) = server_scan.parse_object()?;

    let literal = if enabled { "true" } else { "false" };
    let after = if let Some(member) = server_members.iter().find(|member| member.key == "enabled") {
        if !matches!(body.as_bytes().get(member.value_start), Some(b't') | Some(b'f')) {
            return Err(edit_error("UNSUPPORTED_FORMAT", "enabled alanı boolean değil; güvenli toggle yok."));
        }
        format!("{}{}{}", &body[..member.value_start], literal, &body[member.value_end..])
    } else if let Some(first) = server_members.first() {
        let line_start = body[..first.key_start].rfind('\n').map(|index| index + 1).unwrap_or(0);
        let indent = &body[line_start..first.key_start];
        if !indent.is_empty() && indent.chars().all(|ch| ch == ' ' || ch == '\t') {
            let newline = body[line_start..].find('\n').map(|index| line_start + index).unwrap_or(body.len());
            let eol = if newline > line_start && body.as_bytes().get(newline - 1) == Some(&b'\r') { "\r\n" } else { "\n" };
            format!("{}{}\"enabled\": {literal},{eol}{}", &body[..line_start], indent, &body[line_start..])
        } else {
            format!("{}\"enabled\": {literal},{}", &body[..object_start + 1], &body[object_start + 1..])
        }
    } else {
        format!("{}\"enabled\": {literal}{}", &body[..object_start + 1], &body[object_start + 1..])
    };

    let parsed = if jsonc { parse_jsonc(&after)? } else {
        serde_json::from_str(&after).map_err(|_| edit_error("INVALID_FORMAT", "Düzenlenmiş JSON geçersiz."))?
    };
    let actual = parsed.get(container).and_then(Value::as_object).and_then(|servers| servers.get(name))
        .and_then(Value::as_object).and_then(|entry| entry.get("enabled")).and_then(Value::as_bool);
    if actual != Some(enabled) {
        return Err(edit_error("INVALID_FORMAT", "JSON toggle hedef duruma ulaşmadı."));
    }
    Ok(reattach_bom(text, after))
}

fn edit_toml_enabled(text: &str, container: &str, name: &str, enabled: bool) -> Result<String, EditError> {
    let body = strip_bom(text);
    let document: toml::Value = body.parse()
        .map_err(|_| edit_error("INVALID_FORMAT", "TOML çözümlenemedi."))?;
    let entry = document.get(container).and_then(toml::Value::as_table).and_then(|table| table.get(name))
        .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef TOML kaydı dosyada yok."))?;
    if !entry.is_table() {
        return Err(edit_error("UNSUPPORTED_FORMAT", "Hedef TOML kaydı tablo değil."));
    }
    if let Some(value) = entry.as_table().and_then(|table| table.get("enabled")) {
        if !matches!(value, toml::Value::Boolean(_)) {
            return Err(edit_error("UNSUPPORTED_FORMAT", "enabled alanı boolean değil; güvenli toggle yok."));
        }
    }

    let literal = if enabled { "true" } else { "false" };
    let lines = line_spans(body);
    let target = vec![container.to_string(), name.to_string()];
    let mut headers = Vec::new();
    for (index, (start, _, line)) in lines.iter().enumerate() {
        if let Some((path, array)) = toml_header(line) {
            headers.push((index, *start, path, array));
        }
    }
    let starts_target = |path: &[String]| path.len() >= target.len() && path[..target.len()] == target[..];
    let mut table_span: Option<(usize, usize)> = None;
    let mut header_end: Option<usize> = None;
    for (position, (_, start, path, array)) in headers.iter().enumerate() {
        if !starts_target(path) { continue; }
        if *array {
            return Err(edit_error("UNSUPPORTED_FORMAT", "Hedef TOML tablo dizisi; güvenli toggle yok."));
        }
        if path != &target { continue; }
        if table_span.is_some() {
            return Err(edit_error("SOURCE_CHANGED", "Hedef TOML tablosu birden fazla kez tanımlanmış."));
        }
        let end = headers.iter().skip(position + 1)
            .find(|(_, _, candidate, _)| !starts_target(candidate))
            .map(|(_, next_start, _, _)| *next_start)
            .unwrap_or(body.len());
        table_span = Some((*start, end));
        header_end = lines.iter().find(|(line_start, _, _)| line_start == start).map(|(_, line_end, _)| *line_end);
    }

    let mut value_span: Option<(usize, usize)> = None;
    let mut inline_span: Option<(usize, usize)> = None;
    let mut current: Option<Vec<String>> = None;
    for (start, end, line) in &lines {
        if let Some((path, _)) = toml_header(line) {
            current = Some(path);
            continue;
        }
        let Some((key_text, equals)) = split_assignment(line) else { continue };
        let Some(path) = parse_key_path(key_text) else { continue };
        let inside_target = current.as_deref() == Some(&target[..]);
        let dotted_target = current.as_deref() != Some(&target[..])
            && path.len() == 3 && path[0] == container && path[1] == name && path[2] == "enabled";
        if (inside_target && path.len() == 1 && path[0] == "enabled") || dotted_target {
            if value_span.is_some() {
                return Err(edit_error("SOURCE_CHANGED", "enabled TOML içinde birden fazla kez tanımlanmış."));
            }
            let mut value_start = *start + equals + 1;
            while matches!(body.as_bytes().get(value_start), Some(b' ') | Some(b'\t')) { value_start += 1; }
            let mut value_end = value_start;
            while matches!(body.as_bytes().get(value_end), Some(b'a'..=b'z')) { value_end += 1; }
            value_span = Some((value_start, value_end));
            continue;
        }
        if inside_target && path.first().map(String::as_str) == Some("enabled") {
            return Err(edit_error("UNSUPPORTED_FORMAT", "enabled alt tablosu; güvenli toggle yok."));
        }
        let inline_target = (current.as_deref() == Some(&[container.to_string()][..])
            && path.len() == 1 && path[0] == name)
            || (current.is_none() && path.len() == 2 && path[0] == container && path[1] == name);
        if inline_target {
            let value_start = *start + equals + 1;
            let trimmed = body[value_start..*end].trim_start();
            if !trimmed.starts_with('{') {
                return Err(edit_error("UNSUPPORTED_FORMAT", "Hedef TOML satır içi tablo değil."));
            }
            let brace = *end - trimmed.len();
            let value_end = toml_inline_end(body, brace)?;
            inline_span = Some((brace, value_end));
        }
    }

    if table_span.is_some() && inline_span.is_some() {
        return Err(edit_error("SOURCE_CHANGED", "Hedef TOML hem tablo hem satır olarak tanımlı."));
    }
    let after = if let Some((start, end)) = value_span {
        format!("{}{}{}", &body[..start], literal, &body[end..])
    } else if let Some((start, _end)) = table_span {
        let insert = header_end.unwrap_or(start);
        let eol = if body[..insert].ends_with("\r\n") { "\r\n" } else { "\n" };
        format!("{}enabled = {literal}{eol}{}", &body[..insert], &body[insert..])
    } else if let Some((start, end)) = inline_span {
        let replacement = set_inline_enabled(&body[start..end], literal)?;
        format!("{}{}{}", &body[..start], replacement, &body[end..])
    } else {
        return Err(edit_error("UNSUPPORTED_FORMAT", "Desteklenen TOML toggle biçimi bulunamadı."));
    };

    let parsed: toml::Value = after.parse()
        .map_err(|error| edit_error("INVALID_FORMAT", &format!("Düzenlenmiş TOML geçersiz: {error}")))?;
    let actual = parsed.get(container).and_then(toml::Value::as_table).and_then(|table| table.get(name))
        .and_then(toml::Value::as_table).and_then(|table| table.get("enabled")).and_then(toml::Value::as_bool);
    if actual != Some(enabled) {
        return Err(edit_error("INVALID_FORMAT", "TOML toggle hedef duruma ulaşmadı."));
    }
    Ok(reattach_bom(text, after))
}

fn set_inline_enabled(inner: &str, literal: &str) -> Result<String, EditError> {
    if !inner.starts_with('{') || !inner.ends_with('}') {
        return Err(edit_error("UNSUPPORTED_FORMAT", "Satır içi tablo bekleniyordu."));
    }
    let bytes = inner.as_bytes();
    let mut index = 1;
    let mut in_string = false;
    let mut escaped = false;
    let mut depth = 0usize;
    let mut member_start = 1usize;
    while index + 1 < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped { escaped = false; }
            else if byte == b'\\' { escaped = true; }
            else if byte == b'"' { in_string = false; }
            index += 1;
            continue;
        }
        match byte {
            b'"' => { in_string = true; index += 1; }
            b'{' | b'[' => { depth += 1; index += 1; }
            b'}' | b']' => { depth = depth.saturating_sub(1); index += 1; }
            b'=' if depth == 0 => {
                let key = inner[member_start..index].trim();
                if parse_key_path(key).as_deref() == Some(&["enabled".to_string()][..]) {
                    let mut value_start = index + 1;
                    while matches!(bytes.get(value_start), Some(b' ') | Some(b'\t')) { value_start += 1; }
                    let mut value_end = value_start;
                    while matches!(bytes.get(value_end), Some(b'a'..=b'z')) { value_end += 1; }
                    return Ok(format!("{}{}{}", &inner[..value_start], literal, &inner[value_end..]));
                }
                index += 1;
            }
            b',' if depth == 0 => { member_start = index + 1; index += 1; }
            _ => index += 1,
        }
    }
    let content = inner[1..inner.len() - 1].trim();
    if content.is_empty() {
        Ok(format!("{{ enabled = {literal} }}"))
    } else {
        Ok(format!("{{ enabled = {literal}, {}", &inner[1..]))
    }
}
pub fn parse_jsonc(text: &str) -> Result<Value, EditError> {
    let body = strip_bom(text);
    let cleaned = strip_jsonc_text(body)?;
    serde_json::from_str(&cleaned).map_err(|error| edit_error("INVALID_FORMAT", &format!("JSONC çözümlenemedi: {error}")))
}

fn strip_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

fn edit_json(text: &str, container: &str, name: &str, jsonc: bool) -> Result<String, EditError> {
    let body = strip_bom(text);
    let mut scanner = JsonScan::new(body, jsonc);
    scanner.skip()?;
    let (_, root_members, _) = scanner.parse_object()?;
    scanner.skip()?;
    if scanner.index != body.len() {
        return Err(edit_error("INVALID_FORMAT", "JSON belgesinde kök nesneden sonra veri var."));
    }
    let section = root_members.iter().find(|member| member.key == container)
        .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef bölüm dosyada yok."))?;
    if body.as_bytes().get(section.value_start) != Some(&b'{') {
        return Err(edit_error("SOURCE_CHANGED", "Hedef bölüm nesne değil."));
    }
    let mut inner = JsonScan::new(body, jsonc);
    inner.index = section.value_start;
    let (_, members, _) = inner.parse_object()?;
    let index = members.iter().position(|member| member.key == name)
        .ok_or_else(|| edit_error("SOURCE_CHANGED", "Hedef kayıt dosyada yok."))?;
    let member = &members[index];
    let (start, end) = if let Some(comma) = member.comma_after {
        if let Some(next) = members.get(index + 1) {
            (member.key_start, next.key_start)
        } else {
            (member.key_start, member.value_end.max(comma + 1))
        }
    } else if index > 0 {
        let previous = &members[index - 1];
        (previous.comma_after.ok_or_else(|| edit_error("INVALID_FORMAT", "JSON ayırıcıları tutarsız."))?, member.value_end)
    } else {
        (member.key_start, member.value_end)
    };
    let mut after = String::with_capacity(body.len());
    after.push_str(&body[..start]);
    after.push_str(&body[end..]);
    if jsonc {
        parse_jsonc(&after)?;
    } else {
        serde_json::from_str::<Value>(&after)
            .map_err(|error| edit_error("INVALID_FORMAT", &format!("Düzenlenmiş JSON geçersiz: {error}")))?;
    }
    Ok(reattach_bom(text, after))
}

fn reattach_bom(original: &str, body: String) -> String {
    if original.starts_with('\u{feff}') { format!("\u{feff}{body}") } else { body }
}

struct Member {
    key: String,
    key_start: usize,
    value_start: usize,
    value_end: usize,
    comma_after: Option<usize>,
}

struct JsonScan<'a> {
    text: &'a str,
    bytes: &'a [u8],
    index: usize,
    jsonc: bool,
}

impl<'a> JsonScan<'a> {
    fn new(text: &'a str, jsonc: bool) -> Self {
        Self { text, bytes: text.as_bytes(), index: 0, jsonc }
    }

    fn skip(&mut self) -> Result<(), EditError> {
        loop {
            while self.index < self.bytes.len() && self.bytes[self.index].is_ascii_whitespace() {
                self.index += 1;
            }
            if !self.jsonc || self.index + 1 >= self.bytes.len() || self.bytes[self.index] != b'/' {
                return Ok(());
            }
            match self.bytes.get(self.index + 1) {
                Some(b'/') => {
                    self.index += 2;
                    while self.index < self.bytes.len() && self.bytes[self.index] != b'\n' { self.index += 1; }
                }
                Some(b'*') => {
                    self.index += 2;
                    while self.index + 1 < self.bytes.len()
                        && !(self.bytes[self.index] == b'*' && self.bytes[self.index + 1] == b'/')
                    {
                        self.index += 1;
                    }
                    if self.index + 1 >= self.bytes.len() {
                        return Err(edit_error("INVALID_FORMAT", "JSONC yorumu kapanmıyor."));
                    }
                    self.index += 2;
                }
                _ => return Ok(()),
            }
        }
    }

    fn parse_object(&mut self) -> Result<(usize, Vec<Member>, usize), EditError> {
        let start = self.index;
        if self.bytes.get(start) != Some(&b'{') {
            return Err(edit_error("INVALID_FORMAT", "JSON nesne bekleniyordu."));
        }
        self.index += 1;
        let mut members = Vec::new();
        loop {
            self.skip()?;
            match self.bytes.get(self.index) {
                Some(b'}') => {
                    self.index += 1;
                    return Ok((start, members, self.index));
                }
                Some(b'"') => {}
                _ => return Err(edit_error("INVALID_FORMAT", "JSON anahtarı bekleniyordu.")),
            }
            let key_start = self.index;
            let key = self.parse_string()?;
            self.skip()?;
            if self.bytes.get(self.index) != Some(&b':') {
                return Err(edit_error("INVALID_FORMAT", "JSON anahtarından sonra ':' yok."));
            }
            self.index += 1;
            self.skip()?;
            let value_start = self.index;
            let value_end = self.parse_value()?;
            self.skip()?;
            let comma_after = if self.bytes.get(self.index) == Some(&b',') {
                let comma = self.index;
                self.index += 1;
                Some(comma)
            } else {
                None
            };
            members.push(Member { key, key_start, value_start, value_end, comma_after });
            self.skip()?;
            if self.bytes.get(self.index) == Some(&b'}') {
                self.index += 1;
                return Ok((start, members, self.index));
            }
            if members.last().and_then(|member| member.comma_after).is_none() {
                return Err(edit_error("INVALID_FORMAT", "JSON üyeleri arasında ',' yok."));
            }
        }
    }

    fn parse_array(&mut self) -> Result<usize, EditError> {
        self.index += 1;
        loop {
            self.skip()?;
            if self.bytes.get(self.index) == Some(&b']') {
                self.index += 1;
                return Ok(self.index);
            }
            self.parse_value()?;
            self.skip()?;
            match self.bytes.get(self.index) {
                Some(b',') => self.index += 1,
                Some(b']') => { self.index += 1; return Ok(self.index); }
                _ => return Err(edit_error("INVALID_FORMAT", "JSON dizisinde ayırıcı hatası.")),
            }
        }
    }

    fn parse_value(&mut self) -> Result<usize, EditError> {
        match self.bytes.get(self.index) {
            Some(b'{') => {
                let (_, _, end) = self.parse_object()?;
                Ok(end)
            }
            Some(b'[') => self.parse_array(),
            Some(b'"') => {
                self.parse_string()?;
                Ok(self.index)
            }
            Some(b't') => self.literal("true"),
            Some(b'f') => self.literal("false"),
            Some(b'n') => self.literal("null"),
            Some(byte) if *byte == b'-' || byte.is_ascii_digit() => {
                while self.index < self.bytes.len() {
                    let byte = self.bytes[self.index];
                    if byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'.' | b'e' | b'E') { self.index += 1; } else { break; }
                }
                Ok(self.index)
            }
            _ => Err(edit_error("INVALID_FORMAT", "JSON değeri bekleniyordu.")),
        }
    }

    fn literal(&mut self, expected: &str) -> Result<usize, EditError> {
        if self.text[self.index..].starts_with(expected) {
            self.index += expected.len();
            Ok(self.index)
        } else {
            Err(edit_error("INVALID_FORMAT", "JSON sabiti geçersiz."))
        }
    }

    fn parse_string(&mut self) -> Result<String, EditError> {
        let start = self.index;
        self.index += 1;
        while self.index < self.bytes.len() {
            match self.bytes[self.index] {
                b'\\' => self.index += 2,
                b'"' => {
                    self.index += 1;
                    return serde_json::from_str::<String>(&self.text[start..self.index])
                        .map_err(|error| edit_error("INVALID_FORMAT", &format!("JSON dizesi geçersiz: {error}")));
                }
                _ => self.index += 1,
            }
        }
        Err(edit_error("INVALID_FORMAT", "JSON dizesi kapanmıyor."))
    }
}

fn strip_jsonc_text(text: &str) -> Result<String, EditError> {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                let start = index;
                index += 1;
                while index < bytes.len() {
                    match bytes[index] {
                        b'\\' => index += 2,
                        b'"' => { index += 1; break; }
                        _ => index += 1,
                    }
                }
                output.extend_from_slice(&bytes[start..index]);
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' { index += 1; }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                    if bytes[index] == b'\n' { output.push(b'\n'); }
                    index += 1;
                }
                if index + 1 >= bytes.len() { return Err(edit_error("INVALID_FORMAT", "JSONC yorumu kapanmıyor.")); }
                index += 2;
            }
            byte => { output.push(byte); index += 1; }
        }
    }
    let mut cleaned = Vec::with_capacity(output.len());
    let mut index = 0;
    let mut in_string = false;
    while index < output.len() {
        let byte = output[index];
        if in_string {
            cleaned.push(byte);
            if byte == b'\\' && index + 1 < output.len() {
                cleaned.push(output[index + 1]);
                index += 2;
                continue;
            }
            if byte == b'"' { in_string = false; }
            index += 1;
            continue;
        }
        if byte == b'"' { in_string = true; cleaned.push(byte); index += 1; continue; }
        if byte == b',' {
            let mut look = index + 1;
            while look < output.len() && output[look].is_ascii_whitespace() { look += 1; }
            if matches!(output.get(look), Some(b'}') | Some(b']')) { index += 1; continue; }
        }
        cleaned.push(byte);
        index += 1;
    }
    String::from_utf8(cleaned).map_err(|_| edit_error("INVALID_FORMAT", "JSONC metni UTF-8 değil."))
}

fn edit_toml(text: &str, container: &str, name: &str) -> Result<String, EditError> {
    let body = strip_bom(text);
    let lines = line_spans(body);
    let mut headers = Vec::new();
    for (index, (start, _, line)) in lines.iter().enumerate() {
        if let Some((path, array)) = toml_header(line) {
            headers.push((index, *start, path, array));
        }
    }
    let target = vec![container.to_string(), name.to_string()];
    let starts_target = |path: &[String]| path.len() >= target.len() && path[..target.len()] == target[..];
    let mut table_span: Option<(usize, usize)> = None;
    for (position, (line_index, start, path, array)) in headers.iter().enumerate() {
        if !starts_target(path) { continue; }
        if *array {
            return Err(edit_error("UNSUPPORTED_FORMAT", "Hedef TOML tablo dizisi; güvenli düzenleme yok."));
        }
        if path != &target {
            if !headers.iter().any(|(_, _, candidate, _)| candidate == &target) {
                return Err(edit_error("UNSUPPORTED_FORMAT", "Hedef yalnızca alt tablolarda görünüyor."));
            }
            continue;
        }
        if table_span.is_some() {
            return Err(edit_error("SOURCE_CHANGED", "Hedef TOML tablosu birden fazla kez tanımlanmış."));
        }
        let end = headers.iter().skip(position + 1)
            .find(|(_, _, candidate, _)| !starts_target(candidate))
            .map(|(_, next_start, _, _)| *next_start)
            .unwrap_or(body.len());
        table_span = Some((*start, end));
        let _ = line_index;
    }

    let mut inline = Vec::new();
    let mut current: Option<Vec<String>> = None;
    for (start, end, line) in &lines {
        if let Some((path, _)) = toml_header(line) {
            current = Some(path);
            continue;
        }
        let Some((key_text, equals)) = split_assignment(line) else { continue };
        let Some(path) = parse_key_path(key_text) else { continue };
        if path.len() > 1 && path[0] == container && path[1] == name && current.as_deref() != Some(&target[..]) {
            return Err(edit_error("UNSUPPORTED_FORMAT", "Hedef TOML noktalı anahtar; güvenli düzenleme yok."));
        }
        let inside_container = current.as_deref() == Some(&[container.to_string()][..]);
        if inside_container && path.first().map(String::as_str) == Some(name) {
            if path.len() != 1 {
                return Err(edit_error("UNSUPPORTED_FORMAT", "Hedef TOML alt anahtarları; güvenli düzenleme yok."));
            }
            let value_start = *start + equals + 1;
            let value_end = toml_value_end(body, value_start)?;
            inline.push((*start, value_end.max(*end)));
        }
    }

    let after = match (table_span, inline.as_slice()) {
        (Some(_), [_, ..]) => return Err(edit_error("SOURCE_CHANGED", "Hedef TOML hem tablo hem satır olarak tanımlı.")),
        (Some((start, end)), []) => format!("{}{}", &body[..start], &body[end..]),
        (None, []) => return Err(edit_error("SOURCE_CHANGED", "Hedef TOML kaydı dosyada yok.")),
        (None, spans) => {
            let mut output = String::with_capacity(body.len());
            let mut cursor = 0;
            for (start, end) in spans {
                if *start < cursor { return Err(edit_error("INVALID_FORMAT", "TOML satır aralıkları çakışıyor.")); }
                output.push_str(&body[cursor..*start]);
                cursor = *end;
            }
            output.push_str(&body[cursor..]);
            output
        }
    };
    let document: toml::Value = after.parse()
        .map_err(|error| edit_error("INVALID_FORMAT", &format!("Düzenlenmiş TOML geçersiz: {error}")))?;
    let remains = document.get(container).and_then(toml::Value::as_table)
        .and_then(|table| table.get(name)).is_some();
    if remains {
        return Err(edit_error("INVALID_FORMAT", "TOML düzenlemesi hedefi kaldıramadı."));
    }
    Ok(reattach_bom(text, after))
}

fn line_spans(body: &str) -> Vec<(usize, usize, &str)> {
    let mut spans = Vec::new();
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        spans.push((start, offset, line));
    }
    spans
}

fn toml_header(line: &str) -> Option<(Vec<String>, bool)> {
    let trimmed = line.trim_start();
    let array = trimmed.starts_with("[[");
    let opening = if array { 2 } else if trimmed.starts_with('[') { 1 } else { return None };
    let bytes = trimmed.as_bytes();
    let mut index = opening;
    let mut quote = None;
    while index < bytes.len() {
        let byte = bytes[index];
        match quote {
            Some(marker) if byte == marker => quote = None,
            Some(b'"') if byte == b'\\' => { index += 1; }
            Some(_) => {}
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None if byte == b']' => {
                let close = if array { index + 1 } else { index };
                if array && bytes.get(index + 1) != Some(&b']') { return None; }
                let rest = trimmed[close + 1..].trim_start();
                if !rest.is_empty() && !rest.starts_with('#') { return None; }
                return parse_key_path(&trimmed[opening..index]).map(|path| (path, array));
            }
            None => {}
        }
        index += 1;
    }
    None
}

fn split_assignment(line: &str) -> Option<(&str, usize)> {
    let bytes = line.as_bytes();
    let mut quote = None;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        match quote {
            Some(marker) if byte == marker => quote = None,
            Some(b'"') if byte == b'\\' => index += 1,
            Some(_) => {}
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None if byte == b'=' => return Some((line[..index].trim(), index)),
            None if byte == b'#' => return None,
            None => {}
        }
        index += 1;
    }
    None
}

fn parse_key_path(text: &str) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut quote = None;
    let mut index = 0;
    while index <= bytes.len() {
        let byte = bytes.get(index).copied();
        match quote {
            Some(marker) if byte == Some(marker) => quote = None,
            Some(b'"') if byte == Some(b'\\') => index += 1,
            Some(_) => {}
            None if byte == Some(b'"') || byte == Some(b'\'') => quote = byte,
            None if byte == Some(b'.') || byte.is_none() => {
                let part = text[start..index].trim();
                if part.is_empty() { return None; }
                parts.push(unquote_key(part)?);
                start = index + 1;
            }
            None => {}
        }
        index += 1;
    }
    if quote.is_some() || parts.is_empty() { None } else { Some(parts) }
}

fn unquote_key(text: &str) -> Option<String> {
    if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        return serde_json::from_str::<String>(text).ok();
    }
    if text.len() >= 2 && text.starts_with('\'') && text.ends_with('\'') {
        return Some(text[1..text.len() - 1].to_string());
    }
    if text.contains(['"', '\'', '[', ']', '{', '}']) { None } else { Some(text.to_string()) }
}

fn toml_inline_end(body: &str, start: usize) -> Result<usize, EditError> {
    let bytes = body.as_bytes();
    let mut index = start;
    let mut depth = 0i32;
    let mut quote = None;
    while index < bytes.len() {
        let byte = bytes[index];
        if quote.is_some() {
            if quote == Some(b'"') && byte == b'\\' { index += 2; continue; }
            if Some(byte) == quote { quote = None; }
            index += 1;
            continue;
        }
        match byte {
            b'"' | b'\'' => quote = Some(byte),
            b'{' | b'[' => depth += 1,
            b'}' | b']' => {
                depth -= 1;
                if depth == 0 { return Ok(index + 1); }
                if depth < 0 { break; }
            }
            _ => {}
        }
        index += 1;
    }
    Err(edit_error("UNSUPPORTED_FORMAT", "TOML satır içi tablo kapanmıyor."))
}

fn toml_value_end(body: &str, value_start: usize) -> Result<usize, EditError> {
    let bytes = body.as_bytes();
    let mut index = value_start;
    let mut depth = 0i32;
    let mut quote = None;
    while index < bytes.len() {
        let byte = bytes[index];
        if quote.is_some() {
            if quote == Some(b'"') && byte == b'\\' { index += 2; continue; }
            if Some(byte) == quote { quote = None; }
            index += 1;
            continue;
        }
        match byte {
            b'"' | b'\'' => {
                if bytes.get(index + 1) == Some(&byte) && bytes.get(index + 2) == Some(&byte) {
                    return Err(edit_error("UNSUPPORTED_FORMAT", "Çok satırlı TOML dizesi; güvenli düzenleme yok."));
                }
                quote = Some(byte);
            }
            b'{' | b'[' => depth += 1,
            b'}' | b']' => {
                depth -= 1;
                if depth < 0 { return Err(edit_error("INVALID_FORMAT", "TOML değer kapanışı fazla.")); }
            }
            b'#' => {
                while index < bytes.len() && bytes[index] != b'\n' { index += 1; }
                continue;
            }
            b'\n' if depth == 0 => return Ok(index + 1),
            _ => {}
        }
        index += 1;
    }
    if quote.is_some() || depth != 0 {
        Err(edit_error("INVALID_FORMAT", "TOML değeri kapanmıyor."))
    } else {
        Ok(body.len())
    }
}

fn edit_error(code: &'static str, message: &str) -> EditError {
    EditError { code, message: message.into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::model::{
        Asset, Binding, Capability, ObservedStatus, Owner, Project, Scope, Source,
    };
    use crate::integrations::planner::{plan, PlanRequest};
    use crate::integrations::undo::{Journal, JournalFile, RecordStatus};
    use std::sync::{Arc, Barrier};

    struct Temp(PathBuf);
    impl Temp {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("hub-apply-{label}-{}-{nanos}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

    fn inventory_for(path: &Path, format: SourceFormat, client: ClientId) -> Inventory {
        let asset = Asset::new(AssetKind::Mcp, "postgres", "command: npx server-postgres");
        let mut source = Source::from_path(path, Owner::User, format);
        source.fingerprint = Some(sha256(&fs::read(path).unwrap()));
        let binding = Binding {
            asset_key: asset.key.clone(),
            client,
            scope: Scope::Global,
            project: None,
            source,
            status: ObservedStatus::Configured,
            enabled: None,
            managed: false,
            parent_asset_key: None,
        };
        let mut inventory = Inventory::default();
        inventory.add_binding(asset, binding);
        inventory.capabilities.push(Capability { client, read: true, plan: true, apply: false, undo: false, enable: true, disable: true, toggle_reason: None });
        inventory
    }

    fn make_plan(inventory: &Inventory, client: ClientId) -> OperationPlan {
        let request = PlanRequest {
            operation: Operation::RemoveMcp,
            asset_key: inventory.assets[0].key.clone(),
            client,
            scope: Scope::Global,
            project: None,
            target_scope: None,
            target_project: None,
        };
        plan(inventory, &[], &request).unwrap()
    }

    fn make_toggle_plan(inventory: &Inventory, client: ClientId, operation: Operation) -> OperationPlan {
        let request = PlanRequest {
            operation,
            asset_key: inventory.assets[0].key.clone(),
            client,
            scope: Scope::Global,
            project: None,
            target_scope: None,
            target_project: None,
        };
        plan(inventory, &[], &request).unwrap()
    }

    fn fixture(label: &str, file: &str, text: &str, format: SourceFormat, client: ClientId) -> (Temp, PathBuf, Inventory) {
        let temp = Temp::new(label);
        let path = temp.0.join(file);
        fs::write(&path, text).unwrap();
        let inventory = inventory_for(&path, format, client);
        (temp, path, inventory)
    }

    #[test]
    fn json_removal_keeps_unknown_keys_order_and_formatting() {
        let text = "{\n  \"theme\": \"dark\",\n  \"mcpServers\": {\n    \"keep\": { \"command\": \"keep\" },\n    \"postgres\": { \"command\": \"npx\", \"args\": [\"server-postgres\"] },\n    \"after\": { \"command\": \"after\" }\n  },\n  \"tail\": true\n}\n";
        let (temp, path, inventory) = fixture("json", "config.json", text, SourceFormat::Json, ClientId::Claude);
        let plan = make_plan(&inventory, ClientId::Claude);
        let before = fs::read(&path).unwrap();
        let applied = apply_at(&plan, &inventory, &[], "tx-json", plan.created_at).unwrap();
        assert_eq!(applied.files[0].before_sha256, sha256(&before));
        assert_eq!(applied.files[0].after_sha256, sha256(&fs::read(&path).unwrap()));
        let expected = "{\n  \"theme\": \"dark\",\n  \"mcpServers\": {\n    \"keep\": { \"command\": \"keep\" },\n    \"after\": { \"command\": \"after\" }\n  },\n  \"tail\": true\n}\n";
        assert_eq!(fs::read_to_string(&path).unwrap(), expected);
        assert_eq!(fs::read(temp.0.join(&applied.files[0].backup_name)).unwrap(), before);
        assert!(serde_json::from_str::<Value>(&fs::read_to_string(&path).unwrap()).is_ok());
        assert!(!inventory.capabilities[0].apply);
    }

    #[test]
    fn jsonc_removal_keeps_comments_and_unknown_sections() {
        let text = "{\n  // top comment\n  \"mcp\": {\n    /* keep this block */\n    \"postgres\": { \"command\": \"npx\" }, // remove me\n    \"keep\": { \"command\": \"keep\" }\n  },\n  \"unknown\": { \"value\": 1 }\n}\n";
        let (_temp, path, inventory) = fixture("jsonc", "opencode.jsonc", text, SourceFormat::Jsonc, ClientId::OpenCode);
        let plan = make_plan(&inventory, ClientId::OpenCode);
        apply_at(&plan, &inventory, &[], "tx-jsonc", plan.created_at).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("// top comment"));
        assert!(after.contains("/* keep this block */"));
        assert!(after.contains("\"unknown\": { \"value\": 1 }"));
        assert!(!after.contains("postgres"));
        assert!(parse_jsonc(&after).is_ok());
    }

    #[test]
    fn toml_removal_keeps_comments_and_other_tables() {
        let text = "# root comment\n[mcp_servers.keep]\ncommand = \"keep\"\n\n# postgres section\n[mcp_servers.postgres]\ncommand = \"npx\"\nargs = [\"server-postgres\"] # inline comment\n\n[other]\nvalue = 1 # other note\n";
        let (_temp, path, inventory) = fixture("toml", "config.toml", text, SourceFormat::Toml, ClientId::Codex);
        let plan = make_plan(&inventory, ClientId::Codex);
        apply_at(&plan, &inventory, &[], "tx-toml", plan.created_at).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("# root comment"));
        assert!(after.contains("[mcp_servers.keep]"));
        assert!(after.contains("# other note"));
        assert!(!after.contains("[mcp_servers.postgres]"));
        assert!(after.parse::<toml::Value>().is_ok());
    }

    #[test]
    fn toml_inline_entry_is_removed_without_touching_neighbours() {
        let text = "[mcp_servers]\npostgres = { command = \"npx\", args = [\"server-postgres\"] } # remove\nkeep = { command = \"keep\" }\n";
        let edited = remove_member(text, SourceFormat::Toml, "mcp_servers", "postgres").unwrap();
        assert_eq!(edited, "[mcp_servers]\nkeep = { command = \"keep\" }\n");
    }

    #[test]
    fn stale_source_is_rejected_without_writing() {
        let text = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\"}}}";
        let (_temp, path, inventory) = fixture("stale", "config.json", text, SourceFormat::Json, ClientId::Claude);
        let plan = make_plan(&inventory, ClientId::Claude);
        let changed = "{\"mcpServers\":{\"postgres\":{\"command\":\"changed\"}}}";
        fs::write(&path, changed).unwrap();
        let error = apply_at(&plan, &inventory, &[], "tx-stale", plan.created_at).unwrap_err();
        assert_eq!(error.code, "SOURCE_CHANGED");
        assert_eq!(fs::read_to_string(&path).unwrap(), changed);
    }

    #[test]
    fn simulated_disk_full_leaves_original_and_no_temp_or_backup() {
        let text = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\"}}}";
        let (temp, path, inventory) = fixture("full", "config.json", text, SourceFormat::Json, ClientId::Claude);
        let plan = make_plan(&inventory, ClientId::Claude);
        let mut writer = |target: &Path, bytes: &[u8]| -> io::Result<()> {
            let mut file = OpenOptions::new().write(true).create_new(true).open(target)?;
            file.write_all(&bytes[..bytes.len() / 2])?;
            file.sync_all()?;
            Err(io::Error::new(io::ErrorKind::Other, "disk full (simulated)"))
        };
        let error = apply_with(&plan, &inventory, &[], "tx-full", plan.created_at, &mut writer, None).unwrap_err();
        assert_eq!(error.code, "WRITE_FAILED");
        assert_eq!(fs::read_to_string(&path).unwrap(), text);
        let leftovers: Vec<_> = fs::read_dir(&temp.0).unwrap().flatten()
            .filter(|entry| entry.file_name().to_string_lossy().contains(".respected-"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }

    #[test]
    fn failed_readback_restores_the_original_bytes() {
        let text = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\"}}}";
        let (_temp, path, inventory) = fixture("readback", "config.json", text, SourceFormat::Json, ClientId::Claude);
        let plan = make_plan(&inventory, ClientId::Claude);
        let mut writer = |target: &Path, _bytes: &[u8]| -> io::Result<()> {
            let mut file = OpenOptions::new().write(true).create_new(true).open(target)?;
            file.write_all(b"{}\n")?;
            file.sync_all()
        };
        let error = apply_with(&plan, &inventory, &[], "tx-readback", plan.created_at, &mut writer, None).unwrap_err();
        assert_eq!(error.code, "READBACK_FAILED");
        assert_eq!(fs::read_to_string(&path).unwrap(), text);
    }

    #[test]
    fn symlink_target_is_rejected() {
        let text = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\"}}}";
        let (temp, path, inventory) = fixture("symlink", "config.json", text, SourceFormat::Json, ClientId::Claude);
        let plan = make_plan(&inventory, ClientId::Claude);
        let real = temp.0.join("real.json");
        fs::rename(&path, &real).unwrap();
        if link_file(&real, &path).is_err() {
            eprintln!("symlink privilege unavailable; symlink test skipped");
            return;
        }
        let error = apply_at(&plan, &inventory, &[], "tx-symlink", plan.created_at).unwrap_err();
        assert_eq!(error.code, "SYMLINK_TARGET");
        assert_eq!(fs::read_to_string(&real).unwrap(), text);
    }

    #[cfg(windows)]
    fn link_file(target: &Path, link: &Path) -> io::Result<()> {
        std::os::windows::fs::symlink_file(target, link)
    }
    #[cfg(unix)]
    fn link_file(target: &Path, link: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    #[test]
    fn readonly_target_fails_closed_and_keeps_bytes() {
        let text = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\"}}}";
        let (_temp, path, inventory) = fixture("readonly", "config.json", text, SourceFormat::Json, ClientId::Claude);
        let plan = make_plan(&inventory, ClientId::Claude);
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions).unwrap();
        let error = apply_at(&plan, &inventory, &[], "tx-readonly", plan.created_at).unwrap_err();
        assert_eq!(error.code, "WRITE_FAILED");
        assert_eq!(fs::read_to_string(&path).unwrap(), text);
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&path, permissions).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn path_case_differences_do_not_bypass_the_allowlist() {
        let text = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\"}}}";
        let (temp, path, _inventory) = fixture("case", "Config.json", text, SourceFormat::Json, ClientId::Claude);
        let upper = PathBuf::from(path.to_string_lossy().to_uppercase());
        let inventory = inventory_for(&upper, SourceFormat::Json, ClientId::Claude);
        let plan = make_plan(&inventory, ClientId::Claude);
        apply_at(&plan, &inventory, &[], "tx-case", plan.created_at).unwrap();
        assert!(!fs::read_to_string(temp.0.join("Config.json")).unwrap().contains("postgres"));
    }

    #[cfg(unix)]
    #[test]
    fn file_permissions_are_preserved() {
        use std::os::unix::fs::PermissionsExt;
        let text = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\"}}}";
        let (_temp, path, inventory) = fixture("mode", "config.json", text, SourceFormat::Json, ClientId::Claude);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        let plan = make_plan(&inventory, ClientId::Claude);
        apply_at(&plan, &inventory, &[], "tx-mode", plan.created_at).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o640);
    }
    #[test]
    fn jsonc_disable_keeps_comments_and_other_entries() {
        let text = "{\n  // top comment\n  \"mcp\": {\n    /* postgres */\n    \"postgres\": { \"command\": \"npx\", \"enabled\": true }, // state\n    \"keep\": { \"command\": \"keep\", \"enabled\": true }\n  },\n  \"unknown\": { \"value\": 1 }\n}\n";
        let (_temp, path, inventory) = fixture("jsonc-toggle", "opencode.jsonc", text, SourceFormat::Jsonc, ClientId::OpenCode);
        let plan = make_toggle_plan(&inventory, ClientId::OpenCode, Operation::DisableMcp);
        apply_at(&plan, &inventory, &[], "tx-toggle-jsonc", plan.created_at).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("// top comment"));
        assert!(after.contains("/* postgres */"));
        assert!(after.contains("\"postgres\": { \"command\": \"npx\", \"enabled\": false }"));
        assert!(after.contains("\"keep\": { \"command\": \"keep\", \"enabled\": true }"));
        assert_eq!(plan.targets[0].diff, vec!["~ mcp.postgres.enabled = false".to_string()]);
        assert!(parse_jsonc(&after).is_ok());
        assert!(!inventory.capabilities[0].apply && !inventory.capabilities[0].undo);
    }

    #[test]
    fn toml_disable_keeps_comments_and_other_tables() {
        let text = "# root comment\n[mcp_servers.keep]\ncommand = \"keep\"\nenabled = true\n\n# postgres section\n[mcp_servers.postgres]\ncommand = \"npx\"\nenabled = true # keep me\n\n[other]\nvalue = 1 # other note\n";
        let (_temp, path, inventory) = fixture("toml-toggle", "config.toml", text, SourceFormat::Toml, ClientId::Codex);
        let plan = make_toggle_plan(&inventory, ClientId::Codex, Operation::DisableMcp);
        apply_at(&plan, &inventory, &[], "tx-toggle-toml", plan.created_at).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("# root comment"));
        assert!(after.contains("# other note"));
        assert!(after.contains("enabled = false # keep me"));
        assert!(after.parse::<toml::Value>().is_ok());
    }

    #[test]
    fn toml_toggle_adds_missing_enabled_after_header() {
        let text = "[mcp_servers.postgres]\ncommand = \"npx\"\n\n[other]\nvalue = 1\n";
        let (_temp, path, inventory) = fixture("toml-add", "config.toml", text, SourceFormat::Toml, ClientId::Codex);
        let plan = make_toggle_plan(&inventory, ClientId::Codex, Operation::DisableMcp);
        apply_at(&plan, &inventory, &[], "tx-add-toml", plan.created_at).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.starts_with("[mcp_servers.postgres]\nenabled = false\ncommand = \"npx\""));
        assert!(after.contains("[other]\nvalue = 1"));
        assert!(after.parse::<toml::Value>().is_ok());
    }

    #[test]
    fn toml_inline_toggle_changes_only_target_entry() {
        let text = "[mcp_servers]\npostgres = { command = \"npx\", enabled = true }\nkeep = { command = \"keep\", enabled = true }\n";
        let edited = set_enabled(text, SourceFormat::Toml, "mcp_servers", "postgres", false).unwrap();
        assert!(edited.contains("postgres = { command = \"npx\", enabled = false }"));
        assert!(edited.contains("keep = { command = \"keep\", enabled = true }"));
        assert!(edited.parse::<toml::Value>().is_ok());
    }

    #[test]
    fn non_boolean_enabled_is_rejected_without_edit() {
        let error = set_enabled("[mcp_servers.postgres]\nenabled = \"yes\"\n", SourceFormat::Toml, "mcp_servers", "postgres", false).unwrap_err();
        assert_eq!(error.code, "UNSUPPORTED_FORMAT");
        let json = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\",\"enabled\":1}}}";
        let error = set_enabled(json, SourceFormat::Json, "mcpServers", "postgres", false).unwrap_err();
        assert_eq!(error.code, "UNSUPPORTED_FORMAT");
    }

    fn outcome<'a>(report: &'a crate::integrations::undo::RecoveryReport, transaction_id: &str) -> &'a crate::integrations::undo::RecoveryOutcome {
        report.outcomes.iter().find(|outcome| outcome.transaction_id == transaction_id).unwrap()
    }

    fn journal_files(plan: &OperationPlan, transaction_id: &str) -> Vec<JournalFile> {
        plan.targets.iter().map(|target| {
            let backup = backup_path(Path::new(&target.path), transaction_id).unwrap();
            JournalFile {
                path: PathBuf::from(&target.path),
                path_id: target.path_id.clone(),
                before_sha256: target.before_sha256.clone(),
                after_sha256: target.after_sha256.clone(),
                backup_name: backup.file_name().unwrap().to_string_lossy().into_owned(),
                bytes_before: 0,
                bytes_after: 0,
            }
        }).collect()
    }

    fn transfer_fixture(
        label: &str,
        operation: Operation,
        client: ClientId,
        format: SourceFormat,
        global_text: &str,
        project_text: &str,
    ) -> (Temp, PathBuf, PathBuf, Inventory, Vec<Project>, OperationPlan) {
        let temp = Temp::new(label);
        let global = temp.0.join("global.json");
        let project_dir = temp.0.join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let project_file = project_dir.join(".mcp.json");
        fs::write(&global, global_text).unwrap();
        fs::write(&project_file, project_text).unwrap();
        let project = Project { id: "p1".into(), root: project_dir.to_string_lossy().into_owned() };

        let asset = Asset::new(AssetKind::Mcp, "postgres", "command: npx server-postgres");
        let destination_asset = Asset::new(AssetKind::Mcp, "local", "command: local");
        let mut global_source = Source::from_path(&global, Owner::User, format);
        global_source.fingerprint = Some(sha256(&fs::read(&global).unwrap()));
        let mut project_source = Source::from_path(&project_file, Owner::User, format);
        project_source.fingerprint = Some(sha256(&fs::read(&project_file).unwrap()));
        let mut inventory = Inventory::default();
        inventory.add_binding(asset.clone(), Binding {
            asset_key: asset.key.clone(),
            client,
            scope: Scope::Global,
            project: None,
            source: global_source,
            status: ObservedStatus::Configured,
            enabled: None,
            managed: false,
            parent_asset_key: None,
        });
        inventory.add_binding(destination_asset.clone(), Binding {
            asset_key: destination_asset.key.clone(),
            client,
            scope: Scope::Project,
            project: Some(project.clone()),
            source: project_source,
            status: ObservedStatus::Configured,
            enabled: None,
            managed: false,
            parent_asset_key: None,
        });
        inventory.capabilities.push(Capability {
            client, read: true, plan: true, apply: true, undo: true,
            enable: true, disable: true, toggle_reason: None,
        });
        let request = PlanRequest {
            operation,
            asset_key: asset.key.clone(),
            client,
            scope: Scope::Global,
            project: None,
            target_scope: Some(Scope::Project),
            target_project: Some(project.id.clone()),
        };
        let projects = vec![project];
        let plan = plan(&inventory, &projects, &request).unwrap();
        (temp, global, project_file, inventory, projects, plan)
    }

    const GLOBAL_JSON: &str = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\",\"args\":[\"server\"]},\"other\":{\"command\":\"x\"}}}";
    const PROJECT_JSON: &str = "{\"mcpServers\":{\"local\":{\"command\":\"local\"}}}";

    #[test]
    fn move_then_undo_restores_both_scope_hashes() {
        let (temp, global, project_file, inventory, projects, plan) = transfer_fixture(
            "move-undo", Operation::MoveMcp, ClientId::Claude, SourceFormat::Json, GLOBAL_JSON, PROJECT_JSON,
        );
        let global_before = fs::read(&global).unwrap();
        let project_before = fs::read(&project_file).unwrap();
        let journal = Journal::at(temp.0.join("journal"));
        let applied = apply_at_journaled(&plan, &inventory, &projects, "tx-move", plan.created_at, &journal).unwrap();
        assert_eq!(applied.files.len(), 2);
        assert!(!fs::read_to_string(&global).unwrap().contains("postgres"));
        assert!(fs::read_to_string(&project_file).unwrap().contains("postgres"));
        assert_eq!(journal.read("tx-move").unwrap().status, RecordStatus::Committed);

        let undo = journal.undo("tx-move").unwrap();
        assert_eq!(undo.restored.len(), 2);
        assert_eq!(sha256(&fs::read(&global).unwrap()), sha256(&global_before));
        assert_eq!(sha256(&fs::read(&project_file).unwrap()), sha256(&project_before));
        assert_eq!(journal.read("tx-move").unwrap().status, RecordStatus::RolledBack);
    }

    #[test]
    fn copy_toml_preserves_nested_env_and_undo_restores() {
        let global = "[mcp_servers.other]\ncommand = \"x\"\n\n[mcp_servers.postgres]\ncommand = \"npx\"\n[mcp_servers.postgres.env]\nTOKEN = \"sk-live\"\n";
        let project = "[mcp_servers.local]\ncommand = \"local\"\n";
        let (temp, global_path, project_path, inventory, projects, plan) = transfer_fixture(
            "copy-toml", Operation::CopyMcp, ClientId::Codex, SourceFormat::Toml, global, project,
        );
        let global_before = fs::read(&global_path).unwrap();
        let project_before = fs::read(&project_path).unwrap();
        let journal = Journal::at(temp.0.join("journal"));
        apply_at_journaled(&plan, &inventory, &projects, "tx-copy", plan.created_at, &journal).unwrap();
        let after = fs::read_to_string(&project_path).unwrap();
        assert!(after.contains("[mcp_servers.postgres.env]"));
        assert!(after.contains("TOKEN = \"sk-live\""));
        assert!(!after.contains("\n[env]"));
        journal.undo("tx-copy").unwrap();
        assert_eq!(fs::read(&global_path).unwrap(), global_before);
        assert_eq!(fs::read(&project_path).unwrap(), project_before);
    }

    #[test]
    fn crash_points_recover_without_data_loss() {
        let text = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\"},\"keep\":{\"command\":\"keep\"}}}";
        let (temp, path, inventory) = fixture("crash", "config.json", text, SourceFormat::Json, ClientId::Claude);
        let plan = make_plan(&inventory, ClientId::Claude);
        let after = remove_member(text, SourceFormat::Json, "mcpServers", "postgres").unwrap();
        let journal = Journal::at(temp.0.join("journal"));

        journal.prepare(&plan.plan_digest, "remove_mcp", "tx-before-backup", journal_files(&plan, "tx-before-backup"), plan.created_at).unwrap();
        let report = journal.recover();
        assert_eq!(outcome(&report, "tx-before-backup").status, RecordStatus::RolledBack);
        assert_eq!(fs::read_to_string(&path).unwrap(), text);

        journal.prepare(&plan.plan_digest, "remove_mcp", "tx-mid-write", journal_files(&plan, "tx-mid-write"), plan.created_at).unwrap();
        let name = path.file_name().unwrap().to_str().unwrap();
        fs::write(path.with_file_name(format!(".{name}.respected-tx-mid-write.tmp")), &after[..after.len() / 2]).unwrap();
        let report = journal.recover();
        assert_eq!(outcome(&report, "tx-mid-write").status, RecordStatus::RolledBack);
        assert_eq!(fs::read_to_string(&path).unwrap(), text);
        assert!(!path.with_file_name(format!(".{name}.respected-tx-mid-write.tmp")).exists());

        journal.prepare(&plan.plan_digest, "remove_mcp", "tx-before-readback", journal_files(&plan, "tx-before-readback"), plan.created_at).unwrap();
        let backup = backup_path(&path, "tx-before-readback").unwrap();
        fs::write(&backup, text).unwrap();
        fs::write(&path, &after).unwrap();
        let report = journal.recover();
        assert_eq!(outcome(&report, "tx-before-readback").status, RecordStatus::RolledBack);
        assert_eq!(fs::read_to_string(&path).unwrap(), text);
        assert_eq!(sha256(&fs::read(&path).unwrap()), plan.targets[0].before_sha256);
    }

    #[test]
    fn partial_multi_file_failure_rolls_back_the_first_file() {
        let (temp, global, project_file, inventory, projects, plan) = transfer_fixture(
            "partial", Operation::MoveMcp, ClientId::Claude, SourceFormat::Json, GLOBAL_JSON, PROJECT_JSON,
        );
        let global_before = fs::read(&global).unwrap();
        let project_before = fs::read(&project_file).unwrap();
        let journal = Journal::at(temp.0.join("journal"));
        let mut calls = 0;
        let mut writer = |target: &Path, bytes: &[u8]| -> io::Result<()> {
            calls += 1;
            if calls == 2 {
                return Err(io::Error::new(io::ErrorKind::Other, "disk full (simulated)"));
            }
            let mut file = OpenOptions::new().write(true).create_new(true).open(target)?;
            file.write_all(bytes)?;
            file.sync_all()
        };
        let error = apply_with(&plan, &inventory, &projects, "tx-partial", plan.created_at, &mut writer, Some(&journal)).unwrap_err();
        assert_eq!(error.code, "WRITE_FAILED");
        assert_eq!(fs::read(&global).unwrap(), global_before);
        assert_eq!(fs::read(&project_file).unwrap(), project_before);
        assert_eq!(journal.read("tx-partial").unwrap().status, RecordStatus::Failed);
    }

    #[test]
    fn external_change_blocks_undo_without_writing() {
        let text = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\"},\"keep\":{\"command\":\"keep\"}}}";
        let (temp, path, inventory) = fixture("conflict", "config.json", text, SourceFormat::Json, ClientId::Claude);
        let plan = make_plan(&inventory, ClientId::Claude);
        let journal = Journal::at(temp.0.join("journal"));
        apply_at_journaled(&plan, &inventory, &[], "tx-conflict", plan.created_at, &journal).unwrap();
        let external = "{\"mcpServers\":{\"postgres\":{\"command\":\"externally-edited\"}}}";
        fs::write(&path, external).unwrap();
        let error = journal.undo("tx-conflict").unwrap_err();
        assert_eq!(error.code, "CONFLICT");
        assert_eq!(fs::read_to_string(&path).unwrap(), external);
        assert_eq!(journal.read("tx-conflict").unwrap().status, RecordStatus::Committed);
    }

    #[test]
    fn concurrent_applies_serialize_without_corruption() {
        let text = "{\"mcpServers\":{\"postgres\":{\"command\":\"npx\"},\"keep\":{\"command\":\"keep\"}}}";
        let (_temp, path, inventory) = fixture("concurrent", "config.json", text, SourceFormat::Json, ClientId::Claude);
        let plan = make_plan(&inventory, ClientId::Claude);
        let expected = remove_member(text, SourceFormat::Json, "mcpServers", "postgres").unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let results: Vec<_> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..2).map(|index| {
                let barrier = Arc::clone(&barrier);
                let inventory = inventory.clone();
                let plan = plan.clone();
                let path = path.clone();
                scope.spawn(move || {
                    barrier.wait();
                    let result = apply_at(&plan, &inventory, &[], &format!("tx-concurrent-{index}"), plan.created_at);
                    (result, path)
                })
            }).collect();
            handles.into_iter().map(|handle| handle.join().unwrap().0).collect()
        });
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(fs::read_to_string(&path).unwrap(), expected);
        for result in results.into_iter().filter_map(Result::err) {
            assert!(matches!(result.code.as_str(), "SOURCE_CHANGED" | "PLAN_EXPIRED"), "{}", result.code);
        }
    }

}
