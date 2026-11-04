//! Helpers shared by read-only client adapters.

use crate::integrations::model::{
    Capability, ClientId, Inventory, Owner, Project, ScanIssue, Scope, Source, SourceFormat,
};
use crate::integrations::security::{self, Limits};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub(super) fn issue(inventory: &mut Inventory, code: &str, path: &Path, message: &str) {
    inventory.issues.push(ScanIssue {
        code: code.into(),
        message: message.into(),
        source_path: Some(path.to_string_lossy().into_owned()),
    });
}

pub(super) fn digest(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(super) fn source(path: &Path, owner: Owner, format: SourceFormat, fingerprint: Option<String>) -> Source {
    let mut source = Source::from_path(path, owner, format);
    source.fingerprint = fingerprint;
    source
}

/// Preview planning is available for the MCP formats the planner can read; apply/undo stay
/// closed until the M3 write gate. Enable/disable is opt-in per adapter and needs a
/// format-preserving, single-file edit with a readback.
pub(super) fn read_only(
    inventory: &mut Inventory,
    client: ClientId,
    enable: bool,
    disable: bool,
    toggle_reason: Option<&str>,
) {
    inventory.capabilities.push(Capability {
        client,
        read: true,
        plan: true,
        apply: false,
        undo: false,
        enable,
        disable,
        toggle_reason: toggle_reason.map(str::to_string),
    });
}

/// Read one JSON/JSONC file. Errors never include file contents, only the path and a fixed reason.
pub(super) fn read_json(inventory: &mut Inventory, path: &Path, format: SourceFormat) -> Option<(Value, String)> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return None,
        Err(error) => {
            issue(inventory, "unreadable", path, &error.to_string());
            return None;
        }
    };
    if metadata.file_type().is_symlink() && fs::metadata(path).is_err() {
        issue(inventory, "broken_link", path, "Bağlantı hedefi okunamadı.");
        return None;
    }
    let text = match security::read_text(path, Limits::default()) {
        Ok(text) => text,
        Err(error) => {
            inventory.issues.push(error);
            return None;
        }
    };
    let body = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let parsed = if format == SourceFormat::Jsonc { parse_jsonc(body) } else { serde_json::from_str(body) };
    match parsed {
        Ok(document) => Some((document, digest(&text))),
        Err(_) if format == SourceFormat::Jsonc => {
            issue(inventory, "invalid_jsonc", path, "Dosya geçerli JSONC değil.");
            None
        }
        Err(_) => {
            issue(inventory, "invalid_json", path, "Dosya geçerli JSON değil.");
            None
        }
    }
}

/// JSONC comments and trailing commas are removed outside strings; URLs such as `https://` stay intact.
pub(super) fn parse_jsonc(text: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&strip_jsonc(text))
}

fn strip_jsonc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    while let Some(ch) = chars.next() {
        if in_string {
            out.push(ch);
            if escaped { escaped = false; }
            else if ch == '\\' { escaped = true; }
            else if ch == '"' { in_string = false; }
            continue;
        }
        match ch {
            '"' => { in_string = true; out.push(ch); }
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                for next in chars.by_ref() {
                    if next == '\n' { out.push('\n'); break; }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                while let Some(next) = chars.next() {
                    if next == '*' && chars.peek() == Some(&'/') { chars.next(); break; }
                    if next == '\n' { out.push('\n'); }
                }
            }
            _ => out.push(ch),
        }
    }
    strip_trailing_commas(&out)
}

fn strip_trailing_commas(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if in_string {
            out.push(ch);
            if escaped { escaped = false; }
            else if ch == '\\' { escaped = true; }
            else if ch == '"' { in_string = false; }
            index += 1;
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            index += 1;
            continue;
        }
        if ch == ',' {
            let mut next = index + 1;
            while next < chars.len() && chars[next].is_whitespace() { next += 1; }
            if next < chars.len() && (chars[next] == '}' || chars[next] == ']') {
                index += 1;
                continue;
            }
        }
        out.push(ch);
        index += 1;
    }
    out
}

pub(super) fn json_mcp_identity(server: &Map<String, Value>) -> Option<String> {
    if let Some(command) = server.get("command").and_then(Value::as_str) {
        let args = server.get("args").and_then(Value::as_array).map(|items| {
            items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" ")
        }).unwrap_or_default();
        let identity = if args.is_empty() { format!("command: {command}") } else { format!("command: {command} {args}") };
        return Some(security::redact_text(&identity));
    }
    server.get("url").or_else(|| server.get("httpUrl")).and_then(Value::as_str)
        .map(|url| security::redact_text(&format!("url: {url}")))
}

/// A Respected entry stays visibly owned by its product. A matching name with a foreign
/// command is still the user's entry.
pub(super) fn entry_owner(name: &str, identity: &str) -> Owner {
    let identity = identity.to_lowercase();
    match name.trim().to_lowercase().as_str() {
        "respected-brain" | "respected-vault" if identity.contains("respected-brain") => Owner::Product { product: "brain".into() },
        "respected-orchestrator" if identity.contains("respected-orchestrator") => Owner::Product { product: "orchestrator".into() },
        _ => Owner::User,
    }
}

pub(crate) fn add_skills(
    inventory: &mut Inventory,
    client: ClientId,
    root: &Path,
    scope: Scope,
    project: Option<Project>,
    parent: Option<String>,
    owner: Owner,
) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return,
        Err(error) => {
            issue(inventory, "unreadable", root, &error.to_string());
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                issue(inventory, "unreadable", root, &error.to_string());
                continue;
            }
        };
        let directory = entry.path();
        match fs::metadata(&directory) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => continue,
            Err(error) => {
                let broke = fs::symlink_metadata(&directory).map(|meta| meta.file_type().is_symlink()).unwrap_or(false);
                issue(inventory, if broke { "broken_link" } else { "unreadable" }, &directory, &error.to_string());
                continue;
            }
        }
        add_skill_manifest(inventory, client, &directory.join("SKILL.md"), scope, project.clone(), parent.clone(), owner.clone());
    }
}

/// Cursor documents recursive skill roots (category folders included); every SKILL.md is one skill.
pub(super) fn add_skills_recursive(
    inventory: &mut Inventory,
    client: ClientId,
    root: &Path,
    scope: Scope,
    project: Option<Project>,
    owner: Owner,
) {
    if !root.exists() { return; }
    let walk = security::walk_files(root, Limits::default());
    inventory.issues.extend(walk.issues);
    for manifest in walk.files {
        if manifest.file_name().map(|name| name == "SKILL.md").unwrap_or(false) {
            add_skill_manifest(inventory, client, &manifest, scope, project.clone(), None, owner.clone());
        }
    }
}

fn add_skill_manifest(
    inventory: &mut Inventory,
    client: ClientId,
    manifest: &Path,
    scope: Scope,
    project: Option<Project>,
    parent: Option<String>,
    owner: Owner,
) {
    crate::integrations::skills::inspect(inventory, client, manifest, scope, project, parent, owner);
}
