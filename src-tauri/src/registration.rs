// Windows "Installed apps" entries for the products the Hub manages (like a game launcher's games): SecondBrain and
// Orchestrator are listed on their own; Uninstall opens the Hub's confirmation (`--uninstall=<product>`), Modify opens the
// product page (`--manage=<product>`, repair and settings live there). Entries always point at the running Hub executable,
// so `sync` after every lifecycle change and at start keeps them true. When the Hub itself is uninstalled but the products
// are kept, its uninstaller leaves a copy of the executable in `maintenance\` and runs `--sync-registrations` from there,
// so the entries keep working; that copy removes itself once the last product is gone.
use crate::brain::{self, BrainDirs};
use crate::lifecycle::{self, Roots};
use std::path::{Path, PathBuf};
use std::process::Command;

const KEY_ROOT: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall";
const ENTRIES: [(&str, &str, &str); 2] = [("secondbrain", "RespectedSecondBrain", "Respected SecondBrain"), ("orchestrator", "RespectedOrchestrator", "Respected Orchestrator")];

/// Production writes the real per-user list; a development build only an explicitly named test key (never the real one).
fn key_root() -> Option<String> {
    if crate::trust::dev_enabled() { crate::trust::dev_env("RESPECTED_HUB_ARP_ROOT").and_then(|v| v.into_string().ok()) } else { Some(KEY_ROOT.into()) }
}

fn reg(args: &[&str]) -> bool {
    let program = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows")).join("System32").join("reg.exe");
    let mut command = Command::new(program);
    command.args(args);
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
    command.output().map(|o| o.status.success()).unwrap_or(false)
}

/// Installed version of each Hub-managed product (None = not installed).
fn installed(roots: &Roots, d: &BrainDirs) -> [(usize, Option<String>); 2] {
    let brain = brain::status(roots, d);
    [(0, if brain["installed"] == true { brain["version"].as_str().map(str::to_string) } else { None }),
     (1, lifecycle::read_current(roots, "orchestrator").map(|c| c.0))]
}

/// Write the entries of installed products and delete the others. Returns how many products are installed.
pub fn sync(roots: &Roots, d: &BrainDirs) -> usize {
    let (Some(root), Ok(exe)) = (key_root(), std::env::current_exe()) else { return installed(roots, d).iter().filter(|(_, v)| v.is_some()).count() };
    let exe = exe.to_string_lossy().to_string();
    let mut count = 0;
    for (index, version) in installed(roots, d) {
        let (product, key, name) = ENTRIES[index];
        let key = format!(r"{root}\{key}");
        let Some(version) = version else { let _ = reg(&["delete", &key, "/f"]); continue };
        count += 1;
        let location = if product == "secondbrain" { d.app.to_string_lossy().to_string() } else { roots.managed.join(product).to_string_lossy().to_string() };
        let values = [("DisplayName", name.to_string()), ("DisplayVersion", version), ("Publisher", "Respected".into()), ("DisplayIcon", format!("{exe},0")),
            ("InstallLocation", location), ("UninstallString", format!("\"{exe}\" --uninstall={product}")), ("ModifyPath", format!("\"{exe}\" --manage={product}"))];
        for (value, data) in values { let _ = reg(&["add", &key, "/v", value, "/t", "REG_SZ", "/d", &data, "/f"]); }
        let _ = reg(&["add", &key, "/v", "NoRepair", "/t", "REG_DWORD", "/d", "1", "/f"]);
    }
    count
}

/// `<Respected>\RespectedHub\maintenance`: where a kept-products Hub uninstall leaves the executable.
pub fn maintenance_dir(roots: &Roots) -> PathBuf { roots.managed.parent().map(Path::to_path_buf).unwrap_or_else(|| roots.managed.clone()).join("maintenance") }

pub fn running_from_maintenance(roots: &Roots) -> bool {
    std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).map_or(false, |dir| dir == maintenance_dir(roots))
}

/// A full Hub is installed again: the maintenance copy is no longer needed (sync already pointed the entries back here).
pub fn drop_maintenance(roots: &Roots) {
    if !running_from_maintenance(roots) { let _ = std::fs::remove_dir_all(maintenance_dir(roots)); }
}

/// The maintenance copy after its last product was removed: delete itself and the Hub's leftover folders a moment after exit.
pub fn retire_maintenance(roots: &Roots) {
    let hub_dir = maintenance_dir(roots).parent().map(Path::to_path_buf).unwrap_or_default();
    let folder = |var: &str| std::env::var_os(var).map(PathBuf::from).unwrap_or_default().join("com.respected.hub");
    // only our own leftovers recursively; the parents only when empty (a Hub installed again meanwhile keeps its app folder)
    let script = format!("ping -n 3 127.0.0.1 >nul & rmdir /s /q \"{}\" & rmdir /s /q \"{}\" & rmdir /s /q \"{}\" & rmdir /s /q \"{}\" & rmdir \"{}\" & rmdir \"{}\"",
        maintenance_dir(roots).display(), roots.managed.display(), folder("LOCALAPPDATA").display(), folder("APPDATA").display(), hub_dir.display(), hub_dir.parent().unwrap_or(&hub_dir).display());
    let mut command = Command::new("cmd");
    command.args(["/c", &script]);
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000 | 0x00000008); } // no window, detached
    let _ = command.spawn();
}
