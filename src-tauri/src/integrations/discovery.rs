//! Read-only AI client discovery. A config folder alone is evidence of a source, never of an enabled integration.

use crate::integrations::model::{ClientId, ClientState, ObservedStatus, ScanIssue};
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const TARGETS: [ClientId; 5] = [ClientId::Claude, ClientId::Codex, ClientId::Cursor, ClientId::Gemini, ClientId::OpenCode];

#[derive(Clone, Debug)]
pub struct Environment {
    pub home: PathBuf,
    pub app_data: PathBuf,
    pub local_app_data: PathBuf,
    pub path: Vec<PathBuf>,
    pub path_ext: Vec<String>,
    pub vars: BTreeMap<String, String>,
}

impl Environment {
    pub fn current() -> Self {
        let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).unwrap_or_default();
        let app_data = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| {
            if cfg!(windows) { home.join("AppData/Roaming") }
            else if cfg!(target_os = "macos") { home.join("Library/Application Support") }
            else { home.join(".config") }
        });
        let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|| {
            if cfg!(windows) { home.join("AppData/Local") }
            else if cfg!(target_os = "macos") { home.join("Library/Application Support") }
            else { home.join(".local/share") }
        });
        let path = std::env::var_os("PATH").map(|value| std::env::split_paths(&value).collect()).unwrap_or_default();
        let path_ext = if cfg!(windows) {
            std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT".into()).split(';').map(str::to_string).collect()
        } else { vec![String::new()] };
        let vars = ["CLAUDE_CONFIG_DIR", "CODEX_HOME", "XDG_CONFIG_HOME", "OPENCODE_CONFIG"]
            .into_iter().filter_map(|key| std::env::var(key).ok().map(|value| (key.to_string(), value))).collect();
        Self { home, app_data, local_app_data, path, path_ext, vars }
    }

    fn var(&self, key: &str) -> Option<&Path> { self.vars.get(key).map(Path::new) }
}

#[derive(Debug, Default)]
pub struct Discovery {
    pub clients: Vec<ClientState>,
    pub issues: Vec<ScanIssue>,
}

pub fn scan(env: &Environment) -> Discovery {
    let mut discovery = Discovery::default();
    for client in TARGETS {
        let (configured_paths, unreadable) = existing_config_paths(env, client);
        for path in unreadable { discovery.issues.push(issue(path, "Yapılandırma yolu okunamadı.")); }
        let executable = find_program(env, client);
        let installed = executable.is_some();
        let configured = !configured_paths.is_empty();
        let status = if configured { ObservedStatus::Configured } else if installed { ObservedStatus::Installed } else { ObservedStatus::Unknown };
        discovery.clients.push(ClientState {
            client,
            installed,
            configured,
            status,
            version: None,
            executable: executable.map(|path| path.to_string_lossy().into_owned()),
            config_paths: configured_paths.iter().map(|path| path.to_string_lossy().into_owned()).collect(),
        });
    }
    discovery
}

/// The legacy wizard answer: a configuration folder/file exists. It never claims that a client or an MCP is enabled.
pub fn presence(env: &Environment) -> BTreeMap<String, bool> {
    let mut found = BTreeMap::new();
    for client in TARGETS {
        found.insert(client.as_str().to_string(), !existing_config_paths(env, client).0.is_empty());
    }
    found.insert("antigravity".into(), env.home.join(".gemini").is_dir()
        && fs::read_dir(env.home.join(".gemini")).map(|entries| entries.flatten().any(|entry| entry.path().is_dir()
            && entry.file_name().to_string_lossy().to_lowercase().starts_with("antigravity"))).unwrap_or(false));
    found.insert("windsurf".into(), env.home.join(".codeium").is_dir());
    found.insert("claude-desktop".into(), env.app_data.join("Claude").is_dir());
    let storage = env.app_data.join("Code/User/globalStorage");
    found.insert("vscode-agents".into(), storage.join("saoudrizwan.claude-dev").is_dir() || storage.join("rooveterinaryinc.roo-cline").is_dir());
    found
}

fn existing_config_paths(env: &Environment, client: ClientId) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut found = Vec::new();
    let mut unreadable = Vec::new();
    for (path, kind) in config_candidates(env, client) {
        match fs::metadata(&path) {
            Ok(metadata) if metadata.is_dir() == (kind == ConfigKind::Directory) => found.push(path),
            Ok(_) => unreadable.push(path),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(_) => unreadable.push(path),
        }
    }
    (found, unreadable)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfigKind { Directory, File }

fn config_candidates(env: &Environment, client: ClientId) -> Vec<(PathBuf, ConfigKind)> {
    match client {
        ClientId::Claude => env.var("CLAUDE_CONFIG_DIR").map(|path| vec![(path.to_path_buf(), ConfigKind::Directory)]).unwrap_or_else(|| vec![(env.home.join(".claude"), ConfigKind::Directory), (env.home.join(".claude.json"), ConfigKind::File)]),
        ClientId::Codex => vec![(env.var("CODEX_HOME").map(Path::to_path_buf).unwrap_or_else(|| env.home.join(".codex")), ConfigKind::Directory)],
        ClientId::Cursor => vec![(env.home.join(".cursor"), ConfigKind::Directory)],
        ClientId::Gemini => vec![(env.home.join(".gemini"), ConfigKind::Directory)],
        ClientId::OpenCode => env.var("OPENCODE_CONFIG").map(|path| vec![(path.to_path_buf(), ConfigKind::File)]).unwrap_or_else(|| {
            let root = env.var("XDG_CONFIG_HOME").map(Path::to_path_buf).unwrap_or_else(|| env.home.join(".config"));
            vec![(root.join("opencode"), ConfigKind::Directory)]
        }),
        ClientId::Unknown => Vec::new(),
    }
}

fn find_program(env: &Environment, client: ClientId) -> Option<PathBuf> {
    let names: &[&str] = match client {
        ClientId::Claude => &["claude"],
        ClientId::Codex => &["codex"],
        ClientId::Cursor => &["cursor"],
        ClientId::Gemini => &["gemini"],
        ClientId::OpenCode => &["opencode"],
        ClientId::Unknown => return None,
    };
    for name in names {
        for directory in &env.path {
            for extension in &env.path_ext {
                let candidate = directory.join(format!("{name}{extension}"));
                if candidate.is_file() { return Some(candidate); }
            }
        }
    }
    if client == ClientId::Cursor {
        for path in [env.local_app_data.join("Programs/Cursor/Cursor.exe"), env.local_app_data.join("Programs/cursor/Cursor.exe")] {
            if path.is_file() { return Some(path); }
        }
    }
    None
}

fn issue(path: PathBuf, message: &str) -> ScanIssue {
    ScanIssue { code: "unreadable".into(), message: message.into(), source_path: Some(path.to_string_lossy().into_owned()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Temp(PathBuf);
    impl Temp {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("hub-discovery-{label}-{}-{nanos}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

    fn env(root: &Path) -> Environment {
        Environment {
            home: root.join("home"), app_data: root.join("roaming"), local_app_data: root.join("local"),
            path: vec![root.join("bin")], path_ext: vec![if cfg!(windows) { ".EXE".into() } else { String::new() }], vars: BTreeMap::new(),
        }
    }

    fn program(env: &Environment, name: &str) {
        fs::create_dir_all(&env.path[0]).unwrap();
        fs::write(env.path[0].join(format!("{name}{}", env.path_ext[0])), b"fake").unwrap();
    }

    fn client(discovery: &Discovery, id: ClientId) -> &ClientState { discovery.clients.iter().find(|item| item.client == id).unwrap() }

    #[test]
    fn a_config_folder_alone_is_configured_but_never_enabled() {
        let temp = Temp::new("folder-only");
        let env = env(&temp.0);
        fs::create_dir_all(env.home.join(".claude")).unwrap();
        let discovery = scan(&env);
        let claude = client(&discovery, ClientId::Claude);
        assert!(claude.configured && !claude.installed);
        assert_eq!(claude.status, ObservedStatus::Configured);
        assert_ne!(claude.status, ObservedStatus::Enabled);
        assert!(!claude.config_paths.is_empty());
        assert_eq!(discovery.clients.len(), 5);
    }

    #[test]
    fn a_program_without_config_is_only_installed() {
        let temp = Temp::new("program-only");
        let env = env(&temp.0);
        program(&env, "codex");
        let discovery = scan(&env);
        let codex = client(&discovery, ClientId::Codex);
        assert!(codex.installed && !codex.configured);
        assert_eq!(codex.status, ObservedStatus::Installed);
        assert!(codex.executable.as_ref().unwrap().ends_with(if cfg!(windows) { "codex.EXE" } else { "codex" }));
    }

    #[test]
    fn custom_profile_appdata_and_config_override_are_used() {
        let temp = Temp::new("custom-paths");
        let mut env = env(&temp.0);
        fs::create_dir_all(env.home.join(".cursor")).unwrap();
        fs::create_dir_all(env.local_app_data.join("Programs/Cursor")).unwrap();
        fs::write(env.local_app_data.join("Programs/Cursor/Cursor.exe"), b"fake").unwrap();
        let custom = temp.0.join("custom-claude");
        fs::create_dir_all(&custom).unwrap();
        env.vars.insert("CLAUDE_CONFIG_DIR".into(), custom.to_string_lossy().into_owned());
        let discovery = scan(&env);
        assert!(client(&discovery, ClientId::Cursor).installed && client(&discovery, ClientId::Cursor).configured);
        assert!(client(&discovery, ClientId::Claude).configured);
        assert_eq!(client(&discovery, ClientId::Claude).config_paths.len(), 1);
    }

    #[test]
    fn a_broken_config_path_is_a_controlled_issue() {
        let temp = Temp::new("broken");
        let mut env = env(&temp.0);
        let blocker = temp.0.join("blocker");
        fs::write(&blocker, b"file, not a folder").unwrap();
        env.vars.insert("CLAUDE_CONFIG_DIR".into(), blocker.to_string_lossy().into_owned());
        let discovery = scan(&env);
        assert!(discovery.issues.iter().any(|issue| issue.code == "unreadable" && issue.source_path.as_deref().unwrap().contains("blocker")));
        assert!(!client(&discovery, ClientId::Claude).configured);
        assert_eq!(client(&discovery, ClientId::Claude).status, ObservedStatus::Unknown);
    }

    #[test]
    fn the_wizard_map_keeps_its_existing_keys_and_adds_opencode() {
        let temp = Temp::new("presence");
        let env = env(&temp.0);
        fs::create_dir_all(env.home.join(".claude")).unwrap();
        fs::create_dir_all(env.home.join(".codex")).unwrap();
        fs::create_dir_all(env.home.join(".gemini/antigravity-tools")).unwrap();
        fs::create_dir_all(env.home.join(".config/opencode")).unwrap();
        fs::create_dir_all(env.app_data.join("Code/User/globalStorage/saoudrizwan.claude-dev")).unwrap();
        let found = presence(&env);
        for key in ["claude", "codex", "gemini", "antigravity", "cursor", "windsurf", "claude-desktop", "vscode-agents", "opencode"] {
            assert!(found.contains_key(key), "missing {key}");
        }
        assert_eq!(found["claude"], true);
        assert_eq!(found["antigravity"], true);
        assert_eq!(found["cursor"], false);
    }
}
