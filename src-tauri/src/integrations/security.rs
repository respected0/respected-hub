//! Read-only security primitives: secret redaction, bounded reads and symlink containment.
//! Nothing here executes a client, a command or third-party code.

use crate::integrations::model::ScanIssue;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const SECRET_MARKERS: [&str; 11] = [
    "apikey", "token", "secret", "passw", "credential", "authorization", "cookie", "session",
    "privatekey", "accesskey", "clientsecret",
];
const TOKEN_PREFIXES: [&str; 7] = ["sk-", "ghp_", "github_pat_", "xoxb-", "xoxp-", "AKIA", "AIza"];
const ASSIGNMENTS: [&str; 8] = ["password=", "password:", "token=", "token:", "secret=", "secret:", "api_key=", "apikey:"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_depth: usize,
    pub max_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self { Self { max_depth: 16, max_bytes: 1024 * 1024 } }
}

#[derive(Debug, Default)]
pub struct Walk {
    pub files: Vec<PathBuf>,
    pub issues: Vec<ScanIssue>,
}

pub fn is_secret_key(name: &str) -> bool {
    let normalized = name.chars().filter(|c| c.is_ascii_alphanumeric()).flat_map(char::to_lowercase).collect::<String>();
    SECRET_MARKERS.iter().any(|marker| normalized.contains(marker))
}

fn is_reference(value: &str) -> bool {
    let value = value.trim();
    (value.starts_with("${") && value.ends_with('}')) || (value.starts_with('%') && value.ends_with('%'))
        || (value.starts_with('$') && value.len() > 1 && value[1..].chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
}

/// Redact secret-looking values while keeping environment references such as `${DATABASE_URL}` readable.
pub fn redact_text(text: &str) -> String {
    if is_reference(text) { return text.to_string(); }
    let mut out = text.to_string();
    for prefix in TOKEN_PREFIXES { out = redact_tokens(&out, prefix, true); }
    out = redact_tokens(&out, "bearer ", false);
    for assignment in ASSIGNMENTS { out = redact_tokens(&out, assignment, false); }
    out
}

pub fn redact_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut redacted = Map::new();
            for (key, value) in map {
                let next = if is_secret_key(key) {
                    match value.as_str() {
                        Some(text) if is_reference(text) => Value::String(text.to_string()),
                        _ => Value::String("[redacted]".into()),
                    }
                } else {
                    redact_json(value)
                };
                redacted.insert(key.clone(), next);
            }
            Value::Object(redacted)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact_json).collect()),
        Value::String(text) => Value::String(redact_text(text)),
        other => other.clone(),
    }
}

/// Malformed configs are replaced with a fixed marker instead of showing a value we could not classify.
pub fn redact_json_str(text: &str) -> String {
    serde_json::from_str::<Value>(text).map(|value| redact_json(&value).to_string()).unwrap_or_else(|_| "[unparseable config]".into())
}

pub fn read_text(path: &Path, limits: Limits) -> Result<String, ScanIssue> {
    let size = fs::metadata(path).map(|meta| meta.len()).map_err(|error| issue("unreadable", path, error.to_string()))?;
    if size > limits.max_bytes { return Err(issue("too_large", path, format!("{size} bytes exceeds {}", limits.max_bytes))); }
    let bytes = fs::read(path).map_err(|error| issue("unreadable", path, error.to_string()))?;
    String::from_utf8(bytes).map_err(|error| issue("invalid_utf8", path, error.to_string()))
}

/// Bounded, non-following traversal. Symlinks are followed only inside the canonical root;
/// escapes, loops, unreadable folders and depth overruns become explicit issues.
pub fn walk_files(root: &Path, limits: Limits) -> Walk {
    let mut walk = Walk::default();
    let canonical_root = match fs::canonicalize(root) {
        Ok(path) => path,
        Err(error) => {
            walk.issues.push(issue("unreadable", root, error.to_string()));
            return walk;
        }
    };
    let mut visited = HashSet::from([canonical_root.clone()]);
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((directory, depth)) = stack.pop() {
        if depth > limits.max_depth {
            walk.issues.push(issue("too_deep", &directory, format!("depth {depth} exceeds {}", limits.max_depth)));
            continue;
        }
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                walk.issues.push(issue("unreadable", &directory, error.to_string()));
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    walk.issues.push(issue("unreadable", &directory, error.to_string()));
                    continue;
                }
            };
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) => {
                    walk.issues.push(issue("unreadable", &path, error.to_string()));
                    continue;
                }
            };
            if metadata.file_type().is_symlink() {
                match fs::canonicalize(&path) {
                    Ok(target) if !target.starts_with(&canonical_root) => {
                        walk.issues.push(issue("symlink_escape", &path, target.display().to_string()));
                    }
                    Ok(target) if target.is_dir() => {
                        if !visited.insert(target.clone()) {
                            walk.issues.push(issue("symlink_loop", &path, target.display().to_string()));
                        } else {
                            stack.push((path, depth + 1));
                        }
                    }
                    Ok(_) => walk.files.push(path),
                    Err(error) => walk.issues.push(issue("symlink_unresolved", &path, error.to_string())),
                }
            } else if metadata.is_dir() {
                stack.push((path, depth + 1));
            } else {
                walk.files.push(path);
            }
        }
    }
    walk
}

fn redact_tokens(text: &str, marker: &str, include_marker: bool) -> String {
    let marker = marker.to_ascii_lowercase();
    let lower = text.to_ascii_lowercase();
    let mut output = String::new();
    let mut cursor = 0usize;
    let mut search = 0usize;
    while let Some(relative) = lower[search..].find(&marker) {
        let start = search + relative;
        let value_start = if include_marker { start } else { start + marker.len() };
        let mut end = value_start;
        while end < text.len() {
            let next = text[end..].chars().next().unwrap();
            if next.is_whitespace() || matches!(next, '"' | '\'' | ',' | '}' | ']' | ';' | ')') { break; }
            end += next.len_utf8();
        }
        if end == value_start {
            search = start + marker.len();
            continue;
        }
        output.push_str(&text[cursor..if include_marker { start } else { start + marker.len() }]);
        output.push_str("[redacted]");
        cursor = end;
        search = end;
    }
    output.push_str(&text[cursor..]);
    output
}

fn issue(code: &str, path: &Path, message: String) -> ScanIssue {
    ScanIssue { code: code.into(), message, source_path: Some(path.to_string_lossy().into_owned()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::model::{Owner, Source, SourceFormat};
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Temp(PathBuf);
    impl Temp {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("hub-security-{label}-{}-{nanos}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

    #[test]
    fn known_secret_values_are_redacted_but_references_survive() {
        let config = json!({
            "mcpServers": { "postgres": { "env": {
                "API_KEY": "sk-live-1234567890",
                "GITHUB_TOKEN": "ghp_abcdef",
                "PASSWORD": "hunter2",
                "DATABASE_URL": "${DATABASE_URL}",
                "NORMAL": "visible"
            }}},
            "authorization": "Bearer abc.def",
            "client_secret": "top-secret"
        });
        let text = redact_json(&config).to_string();
        for leaked in ["sk-live-1234567890", "ghp_abcdef", "hunter2", "abc.def", "top-secret"] {
            assert!(!text.contains(leaked), "leaked {leaked}: {text}");
        }
        assert!(text.contains("${DATABASE_URL}") && text.contains("visible"), "{text}");
        assert!(redact_text("password=hunter2 token: sk-live-xyz").contains("[redacted]"));
    }

    #[test]
    fn malformed_config_never_echoes_an_unclassified_value() {
        assert_eq!(redact_json_str(r#"{"token":"sk-live-abc""#), "[unparseable config]");
        assert_eq!(redact_json_str(r#"{"token":"hunter2""#), "[unparseable config]");
    }

    #[test]
    fn source_identity_and_ownership_survive_serialization() {
        let source = Source::from_path(Path::new("C:\\Users\\Demo User\\.claude.json"), Owner::Unknown, SourceFormat::Json);
        let value = serde_json::to_value(source).unwrap();
        assert_eq!(value["owner"]["kind"], "unknown");
        assert!(!value["path_id"].as_str().unwrap().is_empty());
        assert!(value["path"].as_str().unwrap().contains("Demo User"));
    }

    #[test]
    fn oversized_files_and_deep_folders_report_limits() {
        let root = Temp::new("limits");
        fs::write(root.0.join("large.json"), vec![b'x'; 32]).unwrap();
        assert_eq!(read_text(&root.0.join("large.json"), Limits { max_depth: 2, max_bytes: 8 }).unwrap_err().code, "too_large");
        let mut deep = root.0.clone();
        for _ in 0..4 { deep = deep.join("d"); fs::create_dir_all(&deep).unwrap(); }
        assert!(walk_files(&root.0, Limits { max_depth: 1, max_bytes: 1024 }).issues.iter().any(|issue| issue.code == "too_deep"));
    }

    #[test]
    fn symlinks_that_escape_or_loop_are_reported() {
        let root = Temp::new("symlink-root");
        let outside = Temp::new("symlink-outside");
        fs::write(outside.0.join("secret.json"), b"{}").unwrap();
        let escape = root.0.join("escape");
        let result = link(&outside.0, &escape).and_then(|_| { let walk = walk_files(&root.0, Limits::default()); Ok(walk) });
        match result {
            Ok(walk) => assert!(walk.issues.iter().any(|issue| issue.code == "symlink_escape"), "{:?}", walk.issues),
            Err(_) => eprintln!("symlink privilege unavailable; escape test skipped"),
        }
        let looped = root.0.join("loop");
        if link(&root.0, &looped).is_ok() {
            let walk = walk_files(&root.0, Limits::default());
            assert!(walk.issues.iter().any(|issue| issue.code == "symlink_loop"), "{:?}", walk.issues);
        }
    }

    #[cfg(windows)]
    fn link(target: &Path, link: &Path) -> std::io::Result<()> { std::os::windows::fs::symlink_dir(target, link) }
    #[cfg(unix)]
    fn link(target: &Path, link: &Path) -> std::io::Result<()> { std::os::unix::fs::symlink(target, link) }
}
