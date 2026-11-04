//! DISC-005: skill folders, symlink identity, orphans and malformed manifests.
use respected_hub_lib::integrations::adapters::{claude, codex};
use respected_hub_lib::integrations::discovery::Environment;
use respected_hub_lib::integrations::model::{AssetKind, Inventory};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../common/mod.rs"]
mod common;
use common::Temp;

fn write(root: &Path, relative: &str, body: &str) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

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

fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(entries) = fs::read_dir(root) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { visit(&path, files); }
            else if let Ok(bytes) = fs::read(&path) { files.insert(path.to_string_lossy().into_owned(), bytes); }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, &mut files);
    files
}

fn skill<'a>(inventory: &'a Inventory, name: &str) -> Vec<&'a respected_hub_lib::integrations::model::Binding> {
    let keys = inventory.assets.iter().filter(|asset| asset.kind == AssetKind::Skill && asset.name == name).map(|asset| asset.key.clone()).collect::<Vec<_>>();
    inventory.bindings.iter().filter(|binding| keys.contains(&binding.asset_key)).collect()
}

#[cfg(windows)]
fn link(target: &Path, link: &Path) -> std::io::Result<()> {
    fs::create_dir_all(link.parent().unwrap())?;
    std::os::windows::fs::symlink_dir(target, link)
}
#[cfg(unix)]
fn link(target: &Path, link: &Path) -> std::io::Result<()> {
    fs::create_dir_all(link.parent().unwrap())?;
    std::os::unix::fs::symlink(target, link)
}

#[test]
fn a_symlinked_skill_in_two_clients_is_one_asset_with_both_paths() {
    let temp = Temp::new("skills-symlink");
    let home = temp.0.join("home");
    let external = temp.0.join("external-repo/skills/shared");
    write(&external, "SKILL.md", "---\nname: shared\n---\nbody\n");
    if link(&external, &home.join(".claude/skills/shared")).is_err() || link(&external, &home.join(".codex/skills/shared")).is_err() {
        eprintln!("symlink privilege unavailable; skill symlink test skipped");
        return;
    }
    let environment = env(&home);
    let before = snapshot(&temp.0);
    let claude_inventory = claude::scan(&environment, &[]);
    let codex_inventory = codex::scan(&environment, &[]);
    assert_eq!(before, snapshot(&temp.0), "adapter wrote to the fixture");

    let claude_skill = skill(&claude_inventory, "shared");
    let codex_skill = skill(&codex_inventory, "shared");
    assert_eq!(claude_skill.len(), 1, "{:?}", claude_skill);
    assert_eq!(codex_skill.len(), 1, "{:?}", codex_skill);
    assert_eq!(claude_skill[0].asset_key, codex_skill[0].asset_key, "same target must be one asset");
    assert!(claude_skill[0].source.path.replace('\\', "/").ends_with(".claude/skills/shared/SKILL.md"));
    assert!(codex_skill[0].source.path.replace('\\', "/").ends_with(".codex/skills/shared/SKILL.md"));
    let target = Path::new(claude_skill[0].source.symlink_target.as_deref().unwrap());
    assert_eq!(fs::canonicalize(target).unwrap(), fs::canonicalize(&external).unwrap());
    assert!(claude_skill[0].source.fingerprint.as_deref().unwrap().len() == 64);
}

#[test]
fn copies_with_the_same_content_stay_separate() {
    let temp = Temp::new("skills-copy");
    let home = temp.0.join("home");
    let body = "---\nname: copy\n---\nbody\n";
    write(&home, ".claude/skills/copy/SKILL.md", body);
    write(&home, ".codex/skills/copy/SKILL.md", body);
    let environment = env(&home);
    let claude_inventory = claude::scan(&environment, &[]);
    let codex_inventory = codex::scan(&environment, &[]);
    let claude_skill = skill(&claude_inventory, "copy");
    let codex_skill = skill(&codex_inventory, "copy");
    assert_ne!(claude_skill[0].asset_key, codex_skill[0].asset_key, "copies must not merge");
    assert_ne!(claude_skill[0].source.path_id, codex_skill[0].source.path_id);
}

#[test]
fn broken_symlinks_and_invalid_manifests_are_reported_not_merged() {
    let temp = Temp::new("skills-invalid");
    let home = temp.0.join("home");
    write(&home, ".claude/skills/empty/SKILL.md", "");
    write(&home, ".claude/skills/open/SKILL.md", "---\nname: open\n");
    let mut inventory = claude::scan(&env(&home), &[]);
    let codes = inventory.issues.iter().map(|issue| issue.code.as_str()).collect::<Vec<_>>();
    assert!(codes.contains(&"skill_empty"), "{codes:?}");
    assert!(codes.contains(&"skill_invalid_manifest"), "{codes:?}");
    assert!(inventory.bindings.iter().all(|binding| !binding.source.path.contains("empty") && !binding.source.path.contains("open")));

    let gone = temp.0.join("gone");
    if link(&gone, &home.join(".claude/skills/gone")).is_ok() {
        inventory = claude::scan(&env(&home), &[]);
        assert!(inventory.issues.iter().any(|issue| issue.code == "broken_link"), "{:?}", inventory.issues);
        assert!(skill(&inventory, "gone").is_empty());
    } else {
        eprintln!("symlink privilege unavailable; broken link test skipped");
    }
}
