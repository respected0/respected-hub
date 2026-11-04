//! DISC-007: effective scope resolution through every target adapter.
use respected_hub_lib::integrations::adapters::{claude, codex, cursor, gemini, opencode};
use respected_hub_lib::integrations::discovery::Environment;
use respected_hub_lib::integrations::model::{ClientId, Inventory, Project, Scope};
use respected_hub_lib::integrations::resolver::{effective_for, resolve, ResolvedEntry};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../common/mod.rs"]
mod common;
use common::Temp;

const PROJECT: &str = "Projeler/Kapsam";

type Scan = fn(&Environment, &[Project]) -> Inventory;

fn env(home: &Path) -> Environment {
    Environment {
        home: home.to_path_buf(),
        app_data: home.join("roaming"),
        local_app_data: home.join("local"),
        path: Vec::new(),
        path_ext: Vec::new(),
        vars: BTreeMap::new(),
    }
}

fn write(root: &Path, relative: &str, body: &str) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

fn json(entries: &[(&str, &str)]) -> String {
    let servers = entries.iter()
        .map(|(name, file)| format!("\"{name}\":{{\"command\":\"node\",\"args\":[\"{file}\"]}}"))
        .collect::<Vec<_>>().join(",");
    format!("{{\"mcpServers\":{{{servers}}}}}")
}

fn opencode_json(entries: &[(&str, &str)]) -> String {
    let servers = entries.iter()
        .map(|(name, file)| format!("\"{name}\":{{\"command\":[\"node\",\"{file}\"]}}"))
        .collect::<Vec<_>>().join(",");
    format!("{{\"mcp\":{{{servers}}}}}")
}

fn toml(entries: &[(&str, &str)], root: &Path, trusted: bool) -> String {
    let mut body = String::new();
    for (name, file) in entries {
        body.push_str(&format!("[mcp_servers.{name}]\ncommand = \"node\"\nargs = [\"{file}\"]\n\n"));
    }
    if trusted {
        body.push_str(&format!("[projects.\"{}\"]\ntrust_level = \"trusted\"\n", root.to_string_lossy().replace('\\', "/")));
    }
    body
}

fn scan_project(scan: Scan, home: &Path, root: &Path) -> Inventory {
    scan(&env(home), &[Project { id: "kapsam".into(), root: root.to_string_lossy().into_owned() }])
}

fn names(entries: &[ResolvedEntry]) -> Vec<String> {
    let mut names = entries.iter().map(|entry| entry.name.clone()).collect::<Vec<_>>();
    names.sort();
    names
}

fn case(label: &str, body: impl FnOnce()) {
    let _ = label;
    body();
}

fn run_json_client(client: ClientId, scan: Scan, global_rel: &str, project_rel: &str) {
    case("global-only", || {
        let temp = Temp::new("resolver-global");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &json(&[("alpha", "global.js")]));
        let report = resolve(&scan_project(scan, &home, &root));
        let entries = effective_for(&report, client, Some("kapsam"));
        assert_eq!(names(&entries), vec!["alpha"], "global-only");
        assert_eq!(entries[0].scope, Scope::Global);
    });
    case("project-only", || {
        let temp = Temp::new("resolver-project");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&root, project_rel, &json(&[("alpha", "project.js")]));
        let report = resolve(&scan_project(scan, &home, &root));
        let entries = effective_for(&report, client, Some("kapsam"));
        assert_eq!(names(&entries), vec!["alpha"], "project-only");
        assert_eq!(entries[0].scope, Scope::Project);
    });
    case("override", || {
        let temp = Temp::new("resolver-override");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &json(&[("alpha", "global.js")]));
        write(&root, project_rel, &json(&[("alpha", "project.js")]));
        let report = resolve(&scan_project(scan, &home, &root));
        let entries = effective_for(&report, client, Some("kapsam"));
        assert_eq!(entries.len(), 1, "override keeps one winner");
        assert!(entries[0].source_path.ends_with(project_rel), "project wins: {}", entries[0].source_path);
        assert_eq!(effective_for(&report, client, None).len(), 1, "global context keeps its own entry");
    });
    case("coexist", || {
        let temp = Temp::new("resolver-coexist");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &json(&[("alpha", "global.js")]));
        write(&root, project_rel, &json(&[("beta", "project.js")]));
        let report = resolve(&scan_project(scan, &home, &root));
        assert_eq!(names(&effective_for(&report, client, Some("kapsam"))), vec!["alpha", "beta"], "coexist");
    });
    case("other-project", || {
        let temp = Temp::new("resolver-other");
        let home = temp.0.join("home");
        let root = temp.0.join("Baska");
        write(&home, global_rel, &json(&[("alpha", "global.js")]));
        let report = resolve(&scan_project(scan, &home, &root));
        assert_eq!(names(&effective_for(&report, client, Some("kapsam"))), vec!["alpha"], "other-project");
    });
    case("invalid-project", || {
        let temp = Temp::new("resolver-invalid");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &json(&[("alpha", "global.js")]));
        write(&root, project_rel, "{");
        let report = resolve(&scan_project(scan, &home, &root));
        assert_eq!(names(&effective_for(&report, client, Some("kapsam"))), vec!["alpha"], "invalid-project");
        assert!(report.issues.iter().any(|issue| issue.code == "invalid_json"), "invalid file must be reported");
    });
}

#[test]
fn claude_resolves_six_scope_cases() {
    run_json_client(ClientId::Claude, claude::scan, ".claude.json", ".mcp.json");
}

#[test]
fn cursor_resolves_six_scope_cases() {
    run_json_client(ClientId::Cursor, cursor::scan, ".cursor/mcp.json", ".cursor/mcp.json");
}

#[test]
fn gemini_resolves_six_scope_cases() {
    run_json_client(ClientId::Gemini, gemini::scan, ".gemini/settings.json", ".gemini/settings.json");
}

#[test]
fn codex_resolves_six_scope_cases() {
    let global_rel = ".codex/config.toml";
    let project_rel = ".codex/config.toml";
    case("global-only", || {
        let temp = Temp::new("resolver-codex-global");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &toml(&[("alpha", "global.js")], &root, true));
        let report = resolve(&scan_project(codex::scan, &home, &root));
        assert_eq!(names(&effective_for(&report, ClientId::Codex, Some("kapsam"))), vec!["alpha"], "codex global-only");
    });
    case("project-only", || {
        let temp = Temp::new("resolver-codex-project");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &toml(&[], &root, true));
        write(&root, project_rel, &toml(&[("alpha", "project.js")], &root, false));
        let report = resolve(&scan_project(codex::scan, &home, &root));
        let entries = effective_for(&report, ClientId::Codex, Some("kapsam"));
        assert_eq!(names(&entries), vec!["alpha"], "codex project-only");
        assert_eq!(entries[0].scope, Scope::Project);
    });
    case("override", || {
        let temp = Temp::new("resolver-codex-override");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &toml(&[("alpha", "global.js")], &root, true));
        write(&root, project_rel, &toml(&[("alpha", "project.js")], &root, false));
        let report = resolve(&scan_project(codex::scan, &home, &root));
        let entries = effective_for(&report, ClientId::Codex, Some("kapsam"));
        assert_eq!(entries.len(), 1, "codex override keeps one winner");
        assert!(entries[0].source_path.ends_with(project_rel), "codex project wins");
    });
    case("coexist", || {
        let temp = Temp::new("resolver-codex-coexist");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &toml(&[("alpha", "global.js")], &root, true));
        write(&root, project_rel, &toml(&[("beta", "project.js")], &root, false));
        let report = resolve(&scan_project(codex::scan, &home, &root));
        assert_eq!(names(&effective_for(&report, ClientId::Codex, Some("kapsam"))), vec!["alpha", "beta"], "codex coexist");
    });
    case("other-project", || {
        let temp = Temp::new("resolver-codex-other");
        let home = temp.0.join("home");
        let root = temp.0.join("Baska");
        write(&home, global_rel, &toml(&[("alpha", "global.js")], &root, true));
        let report = resolve(&scan_project(codex::scan, &home, &root));
        assert_eq!(names(&effective_for(&report, ClientId::Codex, Some("kapsam"))), vec!["alpha"], "codex other-project");
    });
    case("untrusted", || {
        let temp = Temp::new("resolver-codex-untrusted");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &toml(&[("alpha", "global.js")], &root, false));
        write(&root, project_rel, &toml(&[("beta", "project.js")], &root, false));
        let report = resolve(&scan_project(codex::scan, &home, &root));
        assert_eq!(names(&effective_for(&report, ClientId::Codex, Some("kapsam"))), vec!["alpha"], "untrusted project ignored");
        assert_eq!(report.untrusted.len(), 1, "untrusted project is reported");
    });
}

#[test]
fn opencode_resolves_six_scope_cases() {
    let global_rel = ".config/opencode/opencode.json";
    let project_rel = "opencode.json";
    case("global-only", || {
        let temp = Temp::new("resolver-opencode-global");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &opencode_json(&[("alpha", "global.js")]));
        let report = resolve(&scan_project(opencode::scan, &home, &root));
        assert_eq!(names(&effective_for(&report, ClientId::OpenCode, Some("kapsam"))), vec!["alpha"], "opencode global-only");
    });
    case("project-only", || {
        let temp = Temp::new("resolver-opencode-project");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&root, project_rel, &opencode_json(&[("alpha", "project.js")]));
        let report = resolve(&scan_project(opencode::scan, &home, &root));
        let entries = effective_for(&report, ClientId::OpenCode, Some("kapsam"));
        assert_eq!(names(&entries), vec!["alpha"], "opencode project-only");
        assert_eq!(entries[0].scope, Scope::Project);
    });
    case("override", || {
        let temp = Temp::new("resolver-opencode-override");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &opencode_json(&[("alpha", "global.js")]));
        write(&root, project_rel, &opencode_json(&[("alpha", "project.js")]));
        let report = resolve(&scan_project(opencode::scan, &home, &root));
        let entries = effective_for(&report, ClientId::OpenCode, Some("kapsam"));
        assert_eq!(entries.len(), 1, "opencode override keeps one winner");
        assert!(entries[0].source_path.ends_with(project_rel), "opencode project wins");
    });
    case("coexist", || {
        let temp = Temp::new("resolver-opencode-coexist");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, global_rel, &opencode_json(&[("alpha", "global.js")]));
        write(&root, project_rel, &opencode_json(&[("beta", "project.js")]));
        let report = resolve(&scan_project(opencode::scan, &home, &root));
        assert_eq!(names(&effective_for(&report, ClientId::OpenCode, Some("kapsam"))), vec!["alpha", "beta"], "opencode coexist");
    });
    case("other-project", || {
        let temp = Temp::new("resolver-opencode-other");
        let home = temp.0.join("home");
        let root = temp.0.join("Baska");
        write(&home, global_rel, &opencode_json(&[("alpha", "global.js")]));
        let report = resolve(&scan_project(opencode::scan, &home, &root));
        assert_eq!(names(&effective_for(&report, ClientId::OpenCode, Some("kapsam"))), vec!["alpha"], "opencode other-project");
    });
    case("global-layers", || {
        let temp = Temp::new("resolver-opencode-layers");
        let home = temp.0.join("home");
        let root = temp.0.join(PROJECT);
        write(&home, ".local/share/opencode/opencode.json", &opencode_json(&[("alpha", "legacy.js")]));
        write(&home, global_rel, &opencode_json(&[("alpha", "config.js")]));
        let report = resolve(&scan_project(opencode::scan, &home, &root));
        let entries = effective_for(&report, ClientId::OpenCode, None);
        assert_eq!(entries.len(), 1, "opencode highest global layer wins");
        assert!(entries[0].source_path.replace('\\', "/").ends_with(global_rel), "config layer wins");
    });
}
