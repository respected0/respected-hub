//! CORE-003: every client fixture set stays under a disposable root and covers
//! the normal, broken and same-name/different-source cases.
use respected_hub_lib::integrations::model::ClientId;
use serde_json::Value as Json;
use std::fs;
use std::path::Path;

#[path = "../common/mod.rs"]
mod common;
mod fixtures;
use common::Temp;
use fixtures::{Case, PROJECT_DIR};

fn seed(label: &str, client: ClientId, case: Case) -> (Temp, fixtures::Seeded) {
    let root = Temp::new(label);
    let seeded = fixtures::seed(&root.0, client, case);
    (root, seeded)
}

fn json(path: &Path) -> Json {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn toml(path: &Path) -> toml::Value {
    fs::read_to_string(path).unwrap().parse().unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn every_client_has_normal_broken_and_collision_fixtures() {
    for client in [ClientId::Claude, ClientId::Codex] {
        let (_root, normal) = seed(&format!("{}-normal", client.as_str()), client, Case::Normal);
        assert!(normal.global.is_file(), "{client:?} global fixture missing");
        assert!(normal.project.is_file(), "{client:?} project fixture missing");
        let global = fs::read_to_string(&normal.global).unwrap();
        assert!(global.contains("postgres") && global.contains("${DATABASE_URL}"), "{client:?} normal fixture lost its secret reference");

        let (_root, broken) = seed(&format!("{}-broken", client.as_str()), client, Case::Broken);
        let body = fs::read_to_string(&broken.global).unwrap();
        match client {
            ClientId::Claude => assert!(serde_json::from_str::<Json>(&body).is_err(), "Claude broken fixture parsed"),
            ClientId::Codex => assert!(body.parse::<toml::Value>().is_err(), "Codex broken fixture parsed"),
            other => panic!("unexpected client {other:?}"),
        }
        assert!(!broken.project.exists(), "{client:?} broken fixture should not imply a project config");

        let (_root, collision) = seed(&format!("{}-collision", client.as_str()), client, Case::Collision);
        assert_eq!(names(client, &collision.global), names(client, &collision.project), "{client:?} collision fixture names differ");
        assert_ne!(commands(client, &collision.global), commands(client, &collision.project), "{client:?} collision fixture commands match");
    }
}

fn names(client: ClientId, path: &Path) -> Vec<String> {
    match client {
        ClientId::Claude => json(path)["mcpServers"].as_object().unwrap().keys().cloned().collect(),
        ClientId::Codex => toml(path)["mcp_servers"].as_table().unwrap().keys().cloned().collect(),
        other => panic!("unexpected client {other:?}"),
    }
}

fn commands(client: ClientId, path: &Path) -> Vec<String> {
    match client {
        ClientId::Claude => json(path)["mcpServers"].as_object().unwrap().values()
            .map(|server| server["args"][0].as_str().unwrap().to_string()).collect(),
        ClientId::Codex => toml(path)["mcp_servers"].as_table().unwrap().values()
            .map(|server| server["args"][0].as_str().unwrap().to_string()).collect(),
        other => panic!("unexpected client {other:?}"),
    }
}

#[test]
fn fixtures_use_unicode_and_space_paths_without_touching_the_real_profile() {
    let real = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(std::path::PathBuf::from);
    let watched: Vec<_> = real.iter().flat_map(|home| [
        home.join(".claude.json"), home.join(".claude/settings.json"), home.join(".codex/config.toml"),
        home.join(".cursor/mcp.json"), home.join(".gemini/settings.json"),
    ]).map(|path| (path.clone(), fs::read(&path).ok())).collect();
    let (root, seeded) = seed("unicode-space", ClientId::Claude, Case::Normal);
    assert!(root.0.starts_with(std::env::temp_dir()), "fixture root is not disposable: {}", root.0.display());
    for (path, before) in watched {
        assert_eq!(fs::read(&path).ok(), before, "real profile file changed: {}", path.display());
    }
    assert!(seeded.project_root.ends_with(PROJECT_DIR), "project fixture lost its unicode/space path");
    assert!(seeded.project_root.to_string_lossy().contains("Üretim Projesi"));
}

#[test]
fn symlink_loop_fixture_is_created_or_explicitly_skipped() {
    let root = Temp::new("symlink-loop");
    match fixtures::symlink_loop(&root.0) {
        Ok(link) => assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink()),
        Err(error) => eprintln!("symlink loop skipped: {error}"),
    }
}

#[cfg(unix)]
#[test]
fn permission_denied_fixture_is_an_io_error() {
    use std::os::unix::fs::PermissionsExt;
    let root = Temp::new("permission-denied");
    let closed = root.0.join("closed");
    fs::create_dir_all(&closed).unwrap();
    fs::set_permissions(&closed, fs::Permissions::from_mode(0o000)).unwrap();
    assert!(fs::read_dir(&closed).is_err());
    fs::set_permissions(&closed, fs::Permissions::from_mode(0o700)).unwrap();
}
