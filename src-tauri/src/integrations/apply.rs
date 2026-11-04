//! Transactional writer for one planned third-party integration change.
//!
//! The writer accepts only a server-issued `OperationPlan`; it never takes a path
//! from the UI. It re-verifies the plan digest, TTL and source hash, re-derives the
//! binding from a fresh inventory, edits text without reserializing the document,
//! then performs an allowlisted atomic replace with a sidecar backup and readback.

use crate::integrations::model::{
    AssetKind, Binding, ClientId, Inventory, Owner, Project, Scope, SourceFormat,
};
use crate::integrations::planner::{self, Operation, OperationPlan};
use crate::integrations::security::{self, Limits};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static APPLY_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Applied {
    pub plan_digest: String,
    pub transaction_id: String,
    pub path_id: String,
    pub before_sha256: String,
    pub after_sha256: String,
    pub backup_name: String,
    pub bytes_before: usize,
    pub bytes_after: usize,
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

/// Apply a remove_mcp plan. This is an internal engine entry point; the IPC
/// `integrations.apply` action stays closed until the write gate is approved.
pub fn apply(
    plan: &OperationPlan,
    inventory: &Inventory,
    projects: &[Project],
    transaction_id: &str,
) -> Result<Applied, ApplyError> {
    apply_at(plan, inventory, projects, transaction_id, unix_now())
}

pub fn apply_at(
    plan: &OperationPlan,
    inventory: &Inventory,
    projects: &[Project],
    transaction_id: &str,
    now: u64,
) -> Result<Applied, ApplyError> {
    apply_with(plan, inventory, projects, transaction_id, now, &mut default_writer)
}

fn apply_with<F>(
    plan: &OperationPlan,
    inventory: &Inventory,
    projects: &[Project],
    transaction_id: &str,
    now: u64,
    writer: &mut F,
) -> Result<Applied, ApplyError>
where
    F: FnMut(&Path, &[u8]) -> io::Result<()>,
{
    validate_transaction_id(transaction_id)?;
    if plan.operation != Operation::RemoveMcp {
        return Err(apply_error("UNSUPPORTED_CAPABILITY", "Bu işlem için transactional writer yok."));
    }
    planner::verify(plan, now).map_err(|error| ApplyError {
        code: error.code,
        message: error.message,
        recovery_backup: None,
    })?;
    if plan.targets.len() != 1 {
        return Err(apply_error("UNSUPPORTED_CAPABILITY", "Bu sürüm yalnızca tek dosyalık işlemleri uygular."));
    }
    let target = &plan.targets[0];
    let binding = fresh_binding(plan, inventory, projects)?;
    if binding.managed || matches!(binding.source.owner, Owner::Product { .. } | Owner::Catalog) {
        return Err(apply_error("OWNER_PROTECTED", "Hedef kayıt Respected ürününe veya kataloğa ait."));
    }
    if binding.source.path_id != target.path_id {
        return Err(apply_error("TARGET_NOT_FOUND", "Plan hedefi güncel envanterdeki kaynakla eşleşmiyor."));
    }
    let path = PathBuf::from(&binding.source.path);
    if !same_file(Path::new(&target.path), &path)? {
        return Err(apply_error("TARGET_NOT_FOUND", "Plan hedefi güncel kaynak dosyasıyla aynı değil."));
    }
    let metadata = fs::symlink_metadata(&path)
        .map_err(|_| apply_error("TARGET_NOT_FOUND", "Hedef dosya artık okunamıyor."))?;
    if metadata.file_type().is_symlink() {
        return Err(apply_error("SYMLINK_TARGET", "Hedef symlink; güvenli yazma için reddedildi."));
    }
    if !metadata.is_file() {
        return Err(apply_error("UNSUPPORTED_CAPABILITY", "Hedef normal bir dosya değil."));
    }
    let text = security::read_text(&path, Limits::default())
        .map_err(|error| apply_error("SOURCE_CHANGED", &error.message))?;
    let before_sha256 = sha256(text.as_bytes());
    if before_sha256 != target.before_sha256
        || binding.source.fingerprint.as_deref() != Some(before_sha256.as_str())
    {
        return Err(apply_error("SOURCE_CHANGED", "Hedef dosya plan veya taramadan sonra değişti; planı yenileyin."));
    }
    let after_text = remove_member(&text, target.format, container_for(plan.client), &plan.name).map_err(|error| ApplyError {
        code: error.code.into(),
        message: error.message,
        recovery_backup: None,
    })?;
    let after_sha256 = sha256(after_text.as_bytes());
    if after_sha256 != target.after_sha256 {
        return Err(apply_error("INVALID_PLAN", "Plan önizlemesi deterministik düzenlemeyle eşleşmiyor."));
    }

    let _guard = APPLY_LOCK.lock().map_err(|_| apply_error("INTERNAL", "Yazma kilidi kullanılamıyor."))?;
    let backup = backup_path(&path, transaction_id)?;
    let mut backup_file = OpenOptions::new().write(true).create_new(true).open(&backup)
        .map_err(|error| match error.kind() {
            io::ErrorKind::AlreadyExists => apply_error("TRANSACTION_REUSED", "Bu işlem kimliği daha önce kullanıldı."),
            _ => apply_error("WRITE_FAILED", &format!("Yedek oluşturulamadı: {error}")),
        })?;
    if let Err(error) = backup_file.write_all(text.as_bytes()).and_then(|_| backup_file.sync_all()) {
        let _ = fs::remove_file(&backup);
        return Err(apply_error("WRITE_FAILED", &format!("Yedek yazılamadı: {error}")));
    }
    drop(backup_file);

    let temp = temp_path(&path, transaction_id)?;
    if let Err(error) = writer(&temp, after_text.as_bytes()) {
        let _ = fs::remove_file(&temp);
        let _ = fs::remove_file(&backup);
        return Err(apply_error("WRITE_FAILED", &format!("Geçici dosya yazılamadı: {error}")));
    }
    if let Err(error) = fs::set_permissions(&temp, metadata.permissions()) {
        let _ = fs::remove_file(&temp);
        let _ = fs::remove_file(&backup);
        return Err(apply_error("WRITE_FAILED", &format!("Dosya izinleri korunamadı: {error}")));
    }
    if let Err(error) = fs::rename(&temp, &path) {
        let _ = fs::remove_file(&temp);
        let _ = fs::remove_file(&backup);
        return Err(apply_error("WRITE_FAILED", &format!("Atomik değiştirme başarısız: {error}")));
    }
    sync_parent(&path);

    let readback = fs::read(&path).map_err(|error| apply_error("READBACK_FAILED", &format!("Sonuç okunamadı: {error}")))?;
    if sha256(&readback) != after_sha256 || readback != after_text.as_bytes() {
        return match restore(&path, &backup, metadata.permissions()) {
            Ok(()) => {
                let _ = fs::remove_file(&backup);
                Err(apply_error("READBACK_FAILED", "Yazma sonrası doğrulama başarısız; önceki dosya geri yüklendi."))
            }
            Err(error) => Err(ApplyError {
                code: "RECOVERY_REQUIRED".into(),
                message: format!("Yazma doğrulanamadı ve geri yükleme başarısız: {error}"),
                recovery_backup: Some(backup.to_string_lossy().into_owned()),
            }),
        };
    }
    Ok(Applied {
        plan_digest: plan.plan_digest.clone(),
        transaction_id: transaction_id.to_string(),
        path_id: target.path_id.clone(),
        before_sha256,
        after_sha256,
        backup_name: backup.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default(),
        bytes_before: text.len(),
        bytes_after: after_text.len(),
    })
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
        Asset, Binding, Capability, ObservedStatus, Owner, Scope, Source,
    };
    use crate::integrations::planner::{plan, PlanRequest};

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
        inventory.capabilities.push(Capability { client, read: true, plan: true, apply: false, undo: false });
        inventory
    }

    fn make_plan(inventory: &Inventory, client: ClientId) -> OperationPlan {
        let request = PlanRequest {
            operation: Operation::RemoveMcp,
            asset_key: inventory.assets[0].key.clone(),
            client,
            scope: Scope::Global,
            project: None,
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
        assert_eq!(applied.before_sha256, sha256(&before));
        assert_eq!(applied.after_sha256, sha256(&fs::read(&path).unwrap()));
        let expected = "{\n  \"theme\": \"dark\",\n  \"mcpServers\": {\n    \"keep\": { \"command\": \"keep\" },\n    \"after\": { \"command\": \"after\" }\n  },\n  \"tail\": true\n}\n";
        assert_eq!(fs::read_to_string(&path).unwrap(), expected);
        assert_eq!(fs::read(temp.0.join(&applied.backup_name)).unwrap(), before);
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
        let error = apply_with(&plan, &inventory, &[], "tx-full", plan.created_at, &mut writer).unwrap_err();
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
        let error = apply_with(&plan, &inventory, &[], "tx-readback", plan.created_at, &mut writer).unwrap_err();
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
}
