//! Shared inventory model. Client adapters fill these types; the UI only consumes them.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Mcp,
    Skill,
    Plugin,
    Hook,
    Command,
    Agent,
}

impl AssetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mcp => "mcp",
            Self::Skill => "skill",
            Self::Plugin => "plugin",
            Self::Hook => "hook",
            Self::Command => "command",
            Self::Agent => "agent",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientId {
    Claude,
    Codex,
    Cursor,
    Gemini,
    OpenCode,
    Unknown,
}

impl ClientId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
            Self::Gemini => "gemini",
            Self::OpenCode => "opencode",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Global,
    Project,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub root: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Owner {
    User,
    Product { product: String },
    Catalog,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFormat {
    Json,
    Jsonc,
    Toml,
    SkillDirectory,
    Markdown,
    Manifest,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceIntegrity {
    Ok,
    Missing,
    BrokenLink,
    Unreadable,
    Unsupported,
}

/// Five separate observations; a directory existing is never evidence of `Enabled` or `UsageProven`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservedStatus {
    Unknown,
    Installed,
    Configured,
    Enabled,
    ConnectionVerified,
    UsageProven,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// Lossy display path. Non-Unicode paths are shown with replacement characters.
    pub path: String,
    /// Stable identity of the path bytes (Windows: UTF-16 units, Unix: raw bytes).
    pub path_id: String,
    pub fingerprint: Option<String>,
    /// Symlink target for a deployed skill; the original location stays in `path`.
    #[serde(default)]
    pub symlink_target: Option<String>,
    pub owner: Owner,
    pub format: SourceFormat,
    pub integrity: SourceIntegrity,
}

impl Source {
    pub fn from_path(path: &Path, owner: Owner, format: SourceFormat) -> Self {
        Self {
            path: path.to_string_lossy().into_owned(),
            path_id: path_identity(path),
            fingerprint: None,
            symlink_target: None,
            owner,
            format,
            integrity: SourceIntegrity::Ok,
        }
    }

    pub fn missing(path: &Path, owner: Owner, format: SourceFormat) -> Self {
        let mut source = Self::from_path(path, owner, format);
        source.integrity = SourceIntegrity::Missing;
        source
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub key: String,
    pub name: String,
    pub kind: AssetKind,
    /// Stable identity hint: MCP command/URL, Skill source, Plugin package id.
    pub identity: String,
    pub description: Option<String>,
}

impl Asset {
    pub fn stable_key(kind: AssetKind, name: &str, identity: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(kind.as_str().as_bytes());
        hasher.update(b"\n");
        hasher.update(normalize_name(name).as_bytes());
        hasher.update(b"\n");
        hasher.update(identity.trim().as_bytes());
        format!("{}:{}", kind.as_str(), hex(hasher.finalize()))
    }

    pub fn new(kind: AssetKind, name: &str, identity: &str) -> Self {
        Self {
            key: Self::stable_key(kind, name, identity),
            name: name.trim().to_string(),
            kind,
            identity: identity.trim().to_string(),
            description: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub asset_key: String,
    pub client: ClientId,
    pub scope: Scope,
    pub project: Option<Project>,
    pub source: Source,
    pub status: ObservedStatus,
    pub enabled: Option<bool>,
    /// True only when this Hub wrote the entry; adapters never set it in M1/M2.
    pub managed: bool,
    /// Plugin parent key; children keep their own asset key and binding.
    pub parent_asset_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientState {
    pub client: ClientId,
    pub installed: bool,
    pub configured: bool,
    pub status: ObservedStatus,
    pub version: Option<String>,
    pub executable: Option<String>,
    pub config_paths: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub client: ClientId,
    pub read: bool,
    pub plan: bool,
    pub apply: bool,
    pub undo: bool,
    /// Native or documented enable support for an MCP entry.
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub disable: bool,
    /// User-facing reason when a toggle cannot be guaranteed.
    #[serde(default)]
    pub toggle_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanIssue {
    pub code: String,
    pub message: String,
    pub source_path: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub assets: Vec<Asset>,
    pub bindings: Vec<Binding>,
    pub clients: Vec<ClientState>,
    pub capabilities: Vec<Capability>,
    pub issues: Vec<ScanIssue>,
}

impl Inventory {
    pub fn add_binding(&mut self, asset: Asset, binding: Binding) {
        if !self.assets.iter().any(|item| item.key == asset.key) {
            self.assets.push(asset);
        }
        self.bindings.push(binding);
    }

    pub fn children_of(&self, parent_key: &str) -> Vec<&Binding> {
        self.bindings.iter().filter(|item| item.parent_asset_key.as_deref() == Some(parent_key)).collect()
    }
}

fn normalize_name(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn path_identity(path: &Path) -> String {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        return hex(path.as_os_str().encode_wide().flat_map(|unit| unit.to_le_bytes()).collect::<Vec<_>>());
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return hex(path.as_os_str().as_bytes());
    }
    #[allow(unreachable_code)]
    hex(path.to_string_lossy().as_bytes())
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    let mut output = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(path: &str, integrity: SourceIntegrity) -> Source {
        let mut source = Source::from_path(Path::new(path), Owner::User, SourceFormat::Json);
        source.integrity = integrity;
        source
    }

    fn binding(asset: &Asset, client: ClientId, scope: Scope, path: &str) -> Binding {
        Binding {
            asset_key: asset.key.clone(),
            client,
            scope,
            project: (scope == Scope::Project).then(|| Project { id: "demo".into(), root: "C:\\work\\demo".into() }),
            source: source(path, SourceIntegrity::Ok),
            status: ObservedStatus::Configured,
            enabled: None,
            managed: false,
            parent_asset_key: None,
        }
    }

    #[test]
    fn one_asset_across_clients_round_trips() {
        let asset = Asset::new(AssetKind::Mcp, "postgres", "npx @modelcontextprotocol/server-postgres");
        let mut inventory = Inventory::default();
        inventory.add_binding(asset.clone(), binding(&asset, ClientId::Claude, Scope::Global, "C:\\home\\.claude.json"));
        inventory.add_binding(asset.clone(), binding(&asset, ClientId::Codex, Scope::Global, "C:\\home\\.codex\\config.toml"));
        let json = serde_json::to_string(&inventory).unwrap();
        let read: Inventory = serde_json::from_str(&json).unwrap();
        assert_eq!(read.assets.len(), 1);
        assert_eq!(read.bindings.len(), 2);
        assert!(read.bindings.iter().all(|item| item.asset_key == asset.key));
    }

    #[test]
    fn same_name_different_source_does_not_collide() {
        let first = Asset::new(AssetKind::Mcp, "search", "command: search-a");
        let second = Asset::new(AssetKind::Mcp, "search", "command: search-b");
        assert_ne!(first.key, second.key);
    }

    #[test]
    fn plugin_children_keep_parent_relation() {
        let plugin = Asset::new(AssetKind::Plugin, "demo-plugin", "github:demo/plugin");
        let first = Asset::new(AssetKind::Mcp, "db", "command: db");
        let second = Asset::new(AssetKind::Mcp, "cache", "command: cache");
        let mut inventory = Inventory::default();
        inventory.add_binding(plugin.clone(), binding(&plugin, ClientId::Claude, Scope::Global, "C:\\home\\.claude\\plugins\\demo"));
        for child in [&first, &second] {
            let mut item = binding(child, ClientId::Claude, Scope::Global, "C:\\home\\.claude\\plugins\\demo\\mcp.json");
            item.parent_asset_key = Some(plugin.key.clone());
            inventory.add_binding((*child).clone(), item);
        }
        assert_eq!(inventory.children_of(&plugin.key).len(), 2);
    }

    #[test]
    fn global_and_project_bindings_coexist() {
        let asset = Asset::new(AssetKind::Mcp, "time", "command: time");
        let mut inventory = Inventory::default();
        inventory.add_binding(asset.clone(), binding(&asset, ClientId::Cursor, Scope::Global, "C:\\home\\.cursor\\mcp.json"));
        inventory.add_binding(asset.clone(), binding(&asset, ClientId::Cursor, Scope::Project, "C:\\work\\demo\\.cursor\\mcp.json"));
        assert_eq!(inventory.bindings.iter().filter(|item| item.scope == Scope::Project).count(), 1);
        assert_eq!(inventory.bindings.len(), 2);
    }

    #[test]
    fn broken_and_missing_sources_stay_unknown() {
        let asset = Asset::new(AssetKind::Skill, "draft", "github:demo/skill");
        for integrity in [SourceIntegrity::BrokenLink, SourceIntegrity::Missing] {
            let mut item = binding(&asset, ClientId::Codex, Scope::Global, "C:\\home\\.codex\\skills\\draft");
            item.source.integrity = integrity;
            item.status = ObservedStatus::Unknown;
            let json = serde_json::to_string(&item).unwrap();
            let read: Binding = serde_json::from_str(&json).unwrap();
            assert_eq!(read.source.integrity, integrity);
            assert_eq!(read.status, ObservedStatus::Unknown);
        }
    }

    #[test]
    fn path_identity_is_stable_with_unicode_and_spaces() {
        let path = Path::new("C:\\Users\\Demo User\\AppData\\Roaming\\claude\\mcp ayarları.json");
        let first = Source::from_path(path, Owner::User, SourceFormat::Json);
        let second = Source::from_path(path, Owner::User, SourceFormat::Json);
        assert_eq!(first.path_id, second.path_id);
        assert!(!first.path_id.is_empty());
        assert!(first.path.contains("Demo User"));
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_path_does_not_panic() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let path = Path::new(OsStr::from_bytes(b"/tmp/\xff-config.json"));
        let source = Source::from_path(path, Owner::User, SourceFormat::Json);
        assert!(!source.path_id.is_empty());
    }
}
