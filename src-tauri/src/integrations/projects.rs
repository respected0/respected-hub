//! Hub-owned registry of project roots the person explicitly selected.
//!
//! The registry lives in Hub app data, never in an AI client configuration. Only
//! these roots are handed to adapters; the Hub never walks the disk for projects.

use crate::integrations::discovery::Environment;
use crate::integrations::model::Project;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub fn registry_file(env: &Environment) -> PathBuf {
    env.app_data.join("com.respected.hub").join("projects.json")
}

pub fn load(env: &Environment) -> Result<Vec<Project>, String> {
    read(&registry_file(env))
}

fn read(file: &Path) -> Result<Vec<Project>, String> {
    if !file.exists() { return Ok(Vec::new()); }
    serde_json::from_slice(&fs::read(file).map_err(|e| e.to_string())?).map_err(|e| format!("Proje kaydı okunamadı: {e}"))
}

/// Register one explicitly chosen directory. A missing, relative or non-directory
/// path is refused; a symlink is stored as its resolved target.
pub fn add(env: &Environment, root: &str) -> Result<Vec<Project>, String> {
    let project = validate(root)?;
    let file = registry_file(env);
    let mut projects = read(&file)?;
    if !projects.iter().any(|item| same_root(&item.root, &project.root)) {
        projects.push(project);
        write(&file, &projects)?;
    }
    Ok(projects)
}

pub fn remove(env: &Environment, id: &str) -> Result<Vec<Project>, String> {
    let file = registry_file(env);
    let mut projects = read(&file)?;
    let before = projects.len();
    projects.retain(|item| item.id != id);
    if projects.len() != before { write(&file, &projects)?; }
    Ok(projects)
}

fn validate(root: &str) -> Result<Project, String> {
    let path = Path::new(root.trim());
    if !path.is_absolute() { return Err("Proje kökü mutlak bir klasör olmalı.".into()); }
    let canonical = fs::canonicalize(path).map_err(|_| "Proje klasörü bulunamadı veya okunamadı.".to_string())?;
    let metadata = fs::metadata(&canonical).map_err(|_| "Proje klasörü okunamadı.".to_string())?;
    if !metadata.is_dir() { return Err("Seçilen yol bir klasör değil.".into()); }
    let root = display_path(&canonical);
    let mut hasher = Sha256::new();
    hasher.update(root.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    Ok(Project { id: format!("project:{}", &digest[..16]), root })
}

/// Windows canonicalize returns a `\\?\` verbatim path; client configs use the plain form.
fn display_path(path: &Path) -> String {
    let value = path.to_string_lossy().into_owned();
    if cfg!(windows) {
        if let Some(rest) = value.strip_prefix(r"\\?\UNC\") { return format!(r"\\{rest}"); }
        if let Some(rest) = value.strip_prefix(r"\\?\") { return rest.to_string(); }
    }
    value
}

fn same_root(left: &str, right: &str) -> bool {
    if cfg!(windows) { left.eq_ignore_ascii_case(right) } else { left == right }
}

fn write(file: &Path, projects: &[Project]) -> Result<(), String> {
    fs::create_dir_all(file.parent().ok_or("Geçersiz proje kaydı yolu")?).map_err(|e| e.to_string())?;
    let temporary = file.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(projects).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(temporary, file).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Temp(PathBuf);
    impl Temp {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("hub-projects-{label}-{}-{nanos}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

    fn env(root: &Path) -> Environment {
        Environment {
            home: root.join("home"), app_data: root.join("roaming"), local_app_data: root.join("local"),
            path: Vec::new(), path_ext: Vec::new(), vars: BTreeMap::new(),
        }
    }

    #[test]
    fn missing_relative_and_file_paths_are_refused() {
        let temp = Temp::new("validate");
        let env = env(&temp.0);
        assert!(add(&env, "relative/project").is_err());
        assert!(add(&env, temp.0.join("missing").to_str().unwrap()).is_err());
        let file = temp.0.join("note.txt");
        fs::write(&file, "x").unwrap();
        assert!(add(&env, file.to_str().unwrap()).is_err());
        assert!(load(&env).unwrap().is_empty());
    }

    #[test]
    fn add_is_idempotent_and_remove_only_drops_the_requested_project() {
        let temp = Temp::new("add-remove");
        let env = env(&temp.0);
        let first = temp.0.join("one"); let second = temp.0.join("two");
        fs::create_dir_all(&first).unwrap(); fs::create_dir_all(&second).unwrap();
        let projects = add(&env, first.to_str().unwrap()).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(add(&env, first.to_str().unwrap()).unwrap().len(), 1);
        let projects = add(&env, second.to_str().unwrap()).unwrap();
        assert_eq!(projects.len(), 2);
        let left = remove(&env, &projects[0].id).unwrap();
        assert_eq!(left.len(), 1);
        assert_eq!(fs::canonicalize(&left[0].root).unwrap(), fs::canonicalize(second).unwrap());
    }

    #[test]
    fn registered_root_that_disappears_fails_safely() {
        let temp = Temp::new("deleted");
        let env = env(&temp.0);
        let root = temp.0.join("gone");
        fs::create_dir_all(&root).unwrap();
        add(&env, root.to_str().unwrap()).unwrap();
        fs::remove_dir_all(&root).unwrap();
        let projects = load(&env).unwrap();
        assert_eq!(projects.len(), 1);
        assert!(!Path::new(&projects[0].root).exists());
    }

    #[test]
    fn symlink_root_is_stored_as_its_target() {
        let temp = Temp::new("symlink");
        let env = env(&temp.0);
        let target = temp.0.join("target"); let link = temp.0.join("link");
        fs::create_dir_all(&target).unwrap();
        if link_dir(&target, &link).is_err() {
            eprintln!("symlink privilege unavailable; symlink root test skipped");
            return;
        }
        let projects = add(&env, link.to_str().unwrap()).unwrap();
        assert_eq!(fs::canonicalize(&projects[0].root).unwrap(), fs::canonicalize(&target).unwrap());
    }

    #[cfg(windows)]
    fn link_dir(target: &Path, link: &Path) -> std::io::Result<()> { std::os::windows::fs::symlink_dir(target, link) }
    #[cfg(unix)]
    fn link_dir(target: &Path, link: &Path) -> std::io::Result<()> { std::os::unix::fs::symlink(target, link) }

    #[cfg(windows)]
    #[test]
    fn windows_root_is_stored_without_a_verbatim_prefix() {
        let temp = Temp::new("verbatim");
        let env = env(&temp.0);
        let projects = add(&env, temp.0.to_str().unwrap()).unwrap();
        assert!(!projects[0].root.starts_with(r"\\?\"), "{}", projects[0].root);
        assert!(Path::new(&projects[0].root).is_absolute());
    }

    #[test]
    fn an_unwritable_registry_location_reports_a_safe_error() {
        let temp = Temp::new("denied");
        let env = env(&temp.0);
        let project = temp.0.join("project");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&env.app_data).unwrap();
        fs::write(env.app_data.join("com.respected.hub"), "not a folder").unwrap();
        assert!(add(&env, project.to_str().unwrap()).is_err());
    }
}
