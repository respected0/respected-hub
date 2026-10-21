//! How the Hub is started: the window's intent from the command line, and the two no-window entry points the uninstaller uses.
use super::{cleanup, overlay, registration, standard_roots};
use crate::settings::default_data_root;
use crate::{lifecycle, secondbrain, trust, Settings};
use serde_json::{json, Value};
use std::{fs, path::{Path, PathBuf}};

/// What the person came for: `--install=<product>` (product installer), `--uninstall=<product>` / `--manage=<product>`
/// (Windows Installed apps). Unknown products and actions are ignored.
pub(crate) fn intent(args: impl Iterator<Item = String>) -> Option<Value> {
    args.filter_map(|arg| ["install", "uninstall", "manage"].into_iter().find_map(|action| arg.strip_prefix(&format!("--{action}=")).map(|p| (action, p.to_string()))))
        .find(|(_, product)| lifecycle::PRODUCTS.contains(&product.as_str()))
        .map(|(action, product)| json!({ "action": action, "product": product }))
}

/// `respected-hub.exe --uninstall-products` (no window): called by the Hub's uninstaller when the user chose to remove the
/// products too. Returns the process exit code and leaves a log beside the Hub's data.
pub fn headless_uninstall_products() -> i32 {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let base = local.join("com.respected.hub");
    let roots = standard_roots();
    let dirs = secondbrain::dirs();
    let data_root = if trust::dev_enabled() {
        default_data_root(trust::dev_env("RESPECTED_ORCHESTRATOR_DATA_ROOT").and_then(|v| v.into_string().ok()).as_deref(), base.with_file_name("RespectedOrchestratorDevelopment"))
    } else { secondbrain::respected_root().join("RespectedOrchestrator").join("data") };
    let mut settings = Settings { python: "python".into(), orchestrator_root: base.join("no-source-checkout").join("respected-orchestrator"),
        brain_root: base.join("no-source-checkout").join("respected-secondbrain"), data_root, brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None };
    overlay(&roots, &dirs, &mut settings);
    let report = cleanup::remove_products(&roots, &dirs, &settings);
    registration::sync(&roots, &dirs);
    let log_dir = roots.managed.parent().map(Path::to_path_buf).unwrap_or(base); // beside the managed folder: the Hub's own data folder
    let _ = fs::create_dir_all(&log_dir);
    let _ = fs::write(log_dir.join("uninstall-products.log"), serde_json::to_vec_pretty(&report).unwrap_or_default());
    if report["ok"] == true { 0 } else { 1 }
}

/// `respected-hub.exe --sync-registrations` (no window): the Hub uninstaller runs this from the maintenance copy when the
/// person keeps the products, so their Installed-apps entries point at an executable that still exists.
pub fn headless_sync_registrations() -> i32 { registration::sync(&standard_roots(), &secondbrain::dirs()); 0 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intents_accept_only_known_products_and_actions() {
        let args = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>().into_iter();
        assert_eq!(intent(args(&["hub.exe", "--install=secondbrain"])), Some(json!({ "action": "install", "product": "secondbrain" })));
        assert_eq!(intent(args(&["hub.exe", "--uninstall=orchestrator"])), Some(json!({ "action": "uninstall", "product": "orchestrator" })));
        assert_eq!(intent(args(&["hub.exe", "--manage=secondbrain"])), Some(json!({ "action": "manage", "product": "secondbrain" })));
        assert_eq!(intent(args(&["hub.exe", "--install=../evil"])), None);
        assert_eq!(intent(args(&["hub.exe", "--uninstall-products"])), None);
        assert_eq!(intent(args(&["hub.exe", "--uninstall=runtime"])), None, "only the two products can be named");
        assert_eq!(intent(args(&["hub.exe"])), None);
    }
}

