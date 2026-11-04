//! Disposable client trees for discovery tests. Callers own the root (usually `common::Temp`);
//! nothing here reads or writes the real user profile.
#![allow(dead_code)]
use respected_hub_lib::integrations::model::ClientId;
use std::fs;
use std::path::{Path, PathBuf};

pub const PROJECT_DIR: &str = "Projeler/Üretim Projesi";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Normal,
    Broken,
    Collision,
}

pub struct Seeded {
    pub global: PathBuf,
    pub project: PathBuf,
    pub project_root: PathBuf,
}

/// Write one client's global and project configuration below `home`.
/// Broken cases leave the project file absent after writing a malformed global file.
pub fn seed(home: &Path, client: ClientId, case: Case) -> Seeded {
    let project_root = home.join(PROJECT_DIR);
    fs::create_dir_all(&project_root).unwrap();
    let (global, project, global_body, project_body) = layout(client, case);
    let global = home.join(global);
    let project = project_root.join(project);
    fs::create_dir_all(global.parent().unwrap()).unwrap();
    fs::write(&global, global_body).unwrap();
    if let Some(body) = project_body {
        fs::create_dir_all(project.parent().unwrap()).unwrap();
        fs::write(&project, body).unwrap();
    }
    Seeded { global, project, project_root }
}

fn layout(client: ClientId, case: Case) -> (&'static str, &'static str, &'static str, Option<&'static str>) {
    match (client, case) {
        (ClientId::Claude, Case::Normal) => (
            ".claude.json",
            ".mcp.json",
            r#"{
  "theme": "dark",
  "mcpServers": {
    "postgres": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-postgres"],
      "env": {"DATABASE_URL": "${DATABASE_URL}"}
    }
  }
}
"#,
            Some(r#"{
  "mcpServers": {
    "local-search": {"command": "node", "args": ["search.js"]}
  }
}
"#),
        ),
        (ClientId::Claude, Case::Broken) => (
            ".claude.json",
            ".mcp.json",
            r#"{"mcpServers": {"postgres": {"command": "npx", "args": ["#,
            None,
        ),
        (ClientId::Claude, Case::Collision) => (
            ".claude.json",
            ".mcp.json",
            r#"{"mcpServers": {"postgres": {"command": "npx", "args": ["global-postgres"]}}}"#,
            Some(r#"{"mcpServers": {"postgres": {"command": "npx", "args": ["project-postgres"]}}}"#),
        ),
        (ClientId::Codex, Case::Normal) => (
            ".codex/config.toml",
            ".codex/config.toml",
            r#"[mcp_servers.postgres]
command = "npx"
args = ["@modelcontextprotocol/server-postgres"]

[mcp_servers.postgres.env]
DATABASE_URL = "${DATABASE_URL}"
"#,
            Some(r#"[mcp_servers.local-search]
command = "node"
args = ["search.js"]
"#),
        ),
        (ClientId::Codex, Case::Broken) => (
            ".codex/config.toml",
            ".codex/config.toml",
            r#"[mcp_servers.postgres
command = "npx"
"#,
            None,
        ),
        (ClientId::Codex, Case::Collision) => (
            ".codex/config.toml",
            ".codex/config.toml",
            r#"[mcp_servers.postgres]
command = "npx"
args = ["global-postgres"]
"#,
            Some(r#"[mcp_servers.postgres]
command = "npx"
args = ["project-postgres"]
"#),
        ),
        (other, _) => panic!("no fixture layout for {}", other.as_str()),
    }
}

/// Create a directory symlink that points back at `root`. Platforms without symlink
/// permission return an error, which tests treat as an explicit skip.
pub fn symlink_loop(root: &Path) -> std::io::Result<PathBuf> {
    let link = root.join("loop");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(root, &link)?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(root, &link)?;
    Ok(link)
}
