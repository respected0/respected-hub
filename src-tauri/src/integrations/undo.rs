//! Third-party transaction journal, undo and crash recovery.
//!
//! This journal is deliberately separate from the product lifecycle journal: it
//! records only integration file writes and their sidecar backups, so product
//! updates and third-party edits can never roll each other back.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Serializes every integration write and undo in this process.
pub(crate) static WRITE_LOCK: Mutex<()> = Mutex::new(());

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordStatus {
    Prepared,
    Committed,
    Failed,
    RolledBack,
    Conflict,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalFile {
    pub path: PathBuf,
    pub path_id: String,
    pub before_sha256: String,
    pub after_sha256: String,
    pub backup_name: String,
    pub bytes_before: usize,
    pub bytes_after: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub schema_version: u32,
    pub transaction_id: String,
    pub plan_digest: String,
    pub operation: String,
    pub status: RecordStatus,
    pub created_at: u64,
    pub files: Vec<JournalFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UndoError {
    pub code: String,
    pub message: String,
}

impl UndoError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self { code: code.into(), message: message.into() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UndoResult {
    pub transaction_id: String,
    pub restored: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RecoveryOutcome {
    pub transaction_id: String,
    pub status: RecordStatus,
    pub restored: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct RecoveryReport {
    pub outcomes: Vec<RecoveryOutcome>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Journal {
    root: PathBuf,
}

impl Default for Journal {
    fn default() -> Self {
        Self::at(crate::hub::standard_roots().managed.join("integrations"))
    }
}

impl Journal {
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn prepare(
        &self,
        plan_digest: &str,
        operation: &str,
        transaction_id: &str,
        files: Vec<JournalFile>,
        now: u64,
    ) -> Result<Record, UndoError> {
        validate_id(transaction_id)?;
        if self.record_path(transaction_id).exists() {
            return Err(UndoError::new("TRANSACTION_REUSED", "Bu i\u{015f}lem kimli\u{011f}i daha \u{00f6}nce kullan\u{0131}ld\u{0131}."));
        }
        let record = Record {
            schema_version: SCHEMA_VERSION,
            transaction_id: transaction_id.to_string(),
            plan_digest: plan_digest.to_string(),
            operation: operation.to_string(),
            status: RecordStatus::Prepared,
            created_at: now,
            files,
        };
        fs::create_dir_all(&self.root)
            .map_err(|error| UndoError::new("WRITE_FAILED", format!("\u{0130}\u{015f}lem g\u{00fc}nl\u{00fc}\u{011f}\u{00fc} olu\u{015f}turulamad\u{0131}: {error}")))?;
        self.write(&record)?;
        Ok(record)
    }

    pub fn mark_committed(&self, transaction_id: &str) -> Result<Record, UndoError> {
        self.set_status(transaction_id, RecordStatus::Committed)
    }

    pub fn mark_failed(&self, transaction_id: &str) -> Result<Record, UndoError> {
        self.set_status(transaction_id, RecordStatus::Failed)
    }

    pub fn read(&self, transaction_id: &str) -> Result<Record, UndoError> {
        validate_id(transaction_id)?;
        let path = self.record_path(transaction_id);
        let bytes = fs::read(&path)
            .map_err(|_| UndoError::new("JOURNAL_NOT_FOUND", "\u{0130}\u{015f}lem g\u{00fc}nl\u{00fc}k kayd\u{0131} bulunamad\u{0131}."))?;
        let record: Record = serde_json::from_slice(&bytes)
            .map_err(|_| UndoError::new("INVALID_JOURNAL", "\u{0130}\u{015f}lem g\u{00fc}nl\u{00fc}\u{011f}\u{00fc} bozuk; g\u{00fc}venli bi\u{00e7}imde okunamad\u{0131}."))?;
        if record.schema_version != SCHEMA_VERSION {
            return Err(UndoError::new("INVALID_JOURNAL", format!("Desteklenmeyen i\u{015f}lem g\u{00fc}nl\u{00fc}\u{011f}\u{00fc} \u{015f}emas\u{0131}: {}.", record.schema_version)));
        }
        if record.transaction_id != transaction_id {
            return Err(UndoError::new("INVALID_JOURNAL", "G\u{00fc}nl\u{00fc}k kayd\u{0131} i\u{015f}lem kimli\u{011f}iyle e\u{015f}le\u{015f}miyor."));
        }
        Ok(record)
    }

    /// Restore every file of a committed transaction. All files are checked before
    /// any byte is written, so an externally edited file blocks the whole undo.
    pub fn undo(&self, transaction_id: &str) -> Result<UndoResult, UndoError> {
        let _guard = WRITE_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let mut record = self.read(transaction_id)?;
        match record.status {
            RecordStatus::Committed => {}
            RecordStatus::RolledBack => {
                return Err(UndoError::new("ALREADY_ROLLED_BACK", "Bu i\u{015f}lem zaten geri al\u{0131}nd\u{0131}."));
            }
            _ => {
                return Err(UndoError::new("NOT_COMMITTED", "Yaln\u{0131}zca tamamlanm\u{0131}\u{015f} i\u{015f}lemler geri al\u{0131}nabilir."));
            }
        }
        let work = preflight(&record)?;
        let mut restored = Vec::new();
        for item in work {
            atomic_replace(&item.path, &item.original, Some(item.permissions))
                .map_err(|error| UndoError::new("RESTORE_FAILED", format!("Dosya geri y\u{00fc}klenemedi ({}): {error}", item.path.display())))?;
            restored.push(item.path.to_string_lossy().into_owned());
        }
        record.status = RecordStatus::RolledBack;
        self.write(&record)?;
        Ok(UndoResult { transaction_id: transaction_id.to_string(), restored })
    }

    /// Roll back transactions that were prepared but never committed, which is what
    /// a process kill at any write point leaves behind.
    pub fn recover(&self) -> RecoveryReport {
        let _guard = WRITE_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let mut report = RecoveryReport::default();
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(_) => return report,
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || !name.ends_with(".json") {
                continue;
            }
            let transaction_id = name.trim_end_matches(".json").to_string();
            let mut record = match self.read(&transaction_id) {
                Ok(record) => record,
                Err(error) => {
                    report.errors.push(format!("{transaction_id}: {} {}", error.code, error.message));
                    continue;
                }
            };
            if record.status != RecordStatus::Prepared {
                continue;
            }
            let mut restored = Vec::new();
            let mut conflicts = Vec::new();
            for file in &record.files {
                let path = PathBuf::from(&file.path);
                let _ = fs::remove_file(sibling_temp(&path, &transaction_id));
                let bytes = match fs::read(&path) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        conflicts.push(format!("{}: {error}", file.path.display()));
                        continue;
                    }
                };
                let current = hash(&bytes);
                if current == file.before_sha256 {
                    continue;
                }
                if current != file.after_sha256 {
                    conflicts.push(file.path.to_string_lossy().into_owned());
                    continue;
                }
                let backup = match backup_path(&path, &file.backup_name) {
                    Ok(backup) => backup,
                    Err(_) => {
                        conflicts.push(file.path.to_string_lossy().into_owned());
                        continue;
                    }
                };
                let original = match fs::read(&backup) {
                    Ok(original) if hash(&original) == file.before_sha256 => original,
                    _ => {
                        conflicts.push(file.path.to_string_lossy().into_owned());
                        continue;
                    }
                };
                let permissions = fs::metadata(&path).map(|meta| meta.permissions()).ok();
                if atomic_replace(&path, &original, permissions).is_err() {
                    conflicts.push(file.path.to_string_lossy().into_owned());
                    continue;
                }
                restored.push(file.path.to_string_lossy().into_owned());
            }
            record.status = if conflicts.is_empty() { RecordStatus::RolledBack } else { RecordStatus::Conflict };
            if let Err(error) = self.write(&record) {
                report.errors.push(format!("{transaction_id}: {}", error.message));
            }
            report.outcomes.push(RecoveryOutcome { transaction_id, status: record.status, restored, conflicts });
        }
        report
    }

    fn set_status(&self, transaction_id: &str, status: RecordStatus) -> Result<Record, UndoError> {
        let mut record = self.read(transaction_id)?;
        record.status = status;
        self.write(&record)?;
        Ok(record)
    }

    fn record_path(&self, transaction_id: &str) -> PathBuf {
        self.root.join(format!("{transaction_id}.json"))
    }

    fn write(&self, record: &Record) -> Result<(), UndoError> {
        fs::create_dir_all(&self.root)
            .map_err(|error| UndoError::new("WRITE_FAILED", format!("\u{0130}\u{015f}lem g\u{00fc}nl\u{00fc}\u{011f}\u{00fc} olu\u{015f}turulamad\u{0131}: {error}")))?;
        let path = self.record_path(&record.transaction_id);
        let temp = self.root.join(format!(".{}.json.tmp", record.transaction_id));
        let bytes = serde_json::to_vec_pretty(record)
            .map_err(|_| UndoError::new("WRITE_FAILED", "\u{0130}\u{015f}lem g\u{00fc}nl\u{00fc}\u{011f}\u{00fc} serile\u{015f}tirilemedi."))?;
        let _ = fs::remove_file(&temp);
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)
            .map_err(|error| UndoError::new("WRITE_FAILED", format!("\u{0130}\u{015f}lem g\u{00fc}nl\u{00fc}\u{011f}\u{00fc} yaz\u{0131}lamad\u{0131}: {error}")))?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| UndoError::new("WRITE_FAILED", format!("\u{0130}\u{015f}lem g\u{00fc}nl\u{00fc}\u{011f}\u{00fc} yaz\u{0131}lamad\u{0131}: {error}")))?;
        drop(file);
        fs::rename(&temp, &path)
            .map_err(|error| UndoError::new("WRITE_FAILED", format!("\u{0130}\u{015f}lem g\u{00fc}nl\u{00fc}\u{011f}\u{00fc} de\u{011f}i\u{015f}tirilemedi: {error}")))?;
        sync_parent(&path);
        Ok(())
    }
}

struct Work {
    path: PathBuf,
    original: Vec<u8>,
    permissions: fs::Permissions,
}

fn preflight(record: &Record) -> Result<Vec<Work>, UndoError> {
    let mut work = Vec::new();
    for file in &record.files {
        let path = PathBuf::from(&file.path);
        let bytes = fs::read(&path)
            .map_err(|_| UndoError::new("CONFLICT", format!("Dosya art\u{0131}k okunam\u{0131}yor: {}", file.path.display())))?;
        let current = hash(&bytes);
        if current == file.before_sha256 {
            continue;
        }
        if current != file.after_sha256 {
            return Err(UndoError::new("CONFLICT", format!("Dosya d\u{0131}\u{015f}ar\u{0131}dan de\u{011f}i\u{015f}ti; \u{00fc}zerine yaz\u{0131}lmad\u{0131}: {}", file.path.display())));
        }
        let backup = backup_path(&path, &file.backup_name)?;
        let original = fs::read(&backup)
            .map_err(|_| UndoError::new("BACKUP_MISSING", format!("Yedek bulunamad\u{0131}: {}", backup.display())))?;
        if hash(&original) != file.before_sha256 {
            return Err(UndoError::new("BACKUP_INVALID", format!("Yedek i\u{00e7}eri\u{011f}i beklenen \u{00f6}nceki s\u{00fc}r\u{00fc}mle e\u{015f}le\u{015f}miyor: {}", backup.display())));
        }
        let permissions = fs::metadata(&path)
            .map_err(|_| UndoError::new("CONFLICT", format!("Dosya izinleri okunam\u{0131}yor: {}", file.path.display())))?
            .permissions();
        work.push(Work { path, original, permissions });
    }
    Ok(work)
}

fn backup_path(path: &Path, backup_name: &str) -> Result<PathBuf, UndoError> {
    if backup_name.is_empty() || backup_name.contains('/') || backup_name.contains('\\') {
        return Err(UndoError::new("INVALID_JOURNAL", "Yedek ad\u{0131} g\u{00fc}venli de\u{011f}il."));
    }
    Ok(path.with_file_name(backup_name))
}

fn sibling_temp(path: &Path, transaction_id: &str) -> PathBuf {
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("config");
    path.with_file_name(format!(".{name}.respected-{transaction_id}.tmp"))
}

fn atomic_replace(path: &Path, bytes: &[u8], permissions: Option<fs::Permissions>) -> io::Result<()> {
    let name = path.file_name().and_then(|name| name.to_str()).ok_or_else(|| io::Error::new(io::ErrorKind::Other, "non-unicode file name"))?;
    let temp = path.with_file_name(format!(".{name}.respected-undo-{}", std::process::id()));
    let _ = fs::remove_file(&temp);
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    if let Some(permissions) = permissions {
        fs::set_permissions(&temp, permissions)?;
    }
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

fn validate_id(transaction_id: &str) -> Result<(), UndoError> {
    if transaction_id.is_empty()
        || transaction_id.len() > 64
        || !transaction_id.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(UndoError::new("BAD_REQUEST", "\u{0130}\u{015f}lem kimli\u{011f}i 1-64 g\u{00fc}venli karakter olmal\u{0131}."));
    }
    Ok(())
}

fn hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Temp(PathBuf);
    impl Temp {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("hub-undo-{label}-{}-{nanos}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn corrupt_and_future_records_are_reported_without_panicking() {
        let temp = Temp::new("corrupt");
        let journal = Journal::at(&temp.0);
        fs::write(temp.0.join("broken.json"), b"{not json").unwrap();
        let future = serde_json::json!({
            "schema_version": 99,
            "transaction_id": "future",
            "plan_digest": "x",
            "operation": "remove_mcp",
            "status": "prepared",
            "created_at": 0,
            "files": []
        });
        fs::write(temp.0.join("future.json"), future.to_string()).unwrap();
        let report = journal.recover();
        assert_eq!(report.errors.len(), 2);
        assert!(report.outcomes.is_empty());
    }

    #[test]
    fn transaction_ids_cannot_escape_the_journal() {
        let temp = Temp::new("id");
        let journal = Journal::at(&temp.0);
        let error = journal.prepare("digest", "remove_mcp", "../../escape", Vec::new(), 0).unwrap_err();
        assert_eq!(error.code, "BAD_REQUEST");
        assert!(!temp.0.parent().unwrap().join("escape.json").exists());
    }
}
