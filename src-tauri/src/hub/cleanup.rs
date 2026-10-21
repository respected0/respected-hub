//! Removing the products the Hub installed when the Hub itself is uninstalled (`respected-hub.exe --uninstall-products`).
//!
//! The Hub is the only UI that can undo a product's AI-client registrations (SecondBrain hooks / MCP entry / scheduled task /
//! shortcut, Orchestrator MCP entries). Without this, uninstalling the Hub would leave them pointing at products nobody can
//! manage any more. It goes through the same journaled lifecycle paths as the UI: SecondBrain's own uninstaller (keeps the vault
//! and data, restores foreign config entries), then Orchestrator; finally the Hub's managed folder is dropped.
//! Vaults and notes are never touched. The products' own data (settings, state, backups, Orchestrator runs) goes with them
//! in a production build; a removal can not be undone.

use crate::lifecycle::{self, Roots};
use super::hub_apply;
use crate::{secondbrain, rpp, Settings};
use serde_json::{json, Value};

/// The product's own data folders a removal deletes: only in a production build, and only a folder carrying the product's own
/// name (`<...>/RespectedBrain/data`, `<...>/RespectedOrchestrator/data`) together with that named parent. Never a vault.
pub fn data_folders_to_delete(production: bool, dirs: &secondbrain::BrainDirs, settings: &Settings, product: &str) -> Vec<std::path::PathBuf> {
    let own = |dir: &std::path::Path, name: &str| dir.file_name().map_or(false, |n| n == name);
    let owned = |data: &std::path::Path, folder: &str| own(data, "data") && data.parent().map_or(false, |p| own(p, folder));
    let pair = |data: &std::path::Path| std::iter::once(data.to_path_buf()).chain(data.parent().map(std::path::Path::to_path_buf)).collect::<Vec<_>>();
    if !production { return Vec::new(); }
    match product {
        "secondbrain" if owned(&dirs.data, "RespectedBrain") => pair(&dirs.data),
        "orchestrator" if owned(&settings.data_root, "RespectedOrchestrator") => pair(&settings.data_root),
        _ => Vec::new(),
    }
}

/// After a successful per-product uninstall: leave nothing of ours behind except the vault and its notes (no undo copies, staging,
/// product data/backups). Data folders are only removed in production builds and
/// only when they carry the product's own folder name.
pub fn after_uninstall(roots: &Roots, dirs: &secondbrain::BrainDirs, settings: &Settings, product: &str, id: &str) {
    let _ = std::fs::remove_dir_all(roots.managed.join(product));
    crate::lifecycle::catalog::forget(roots, product);
    for folder in data_folders_to_delete(!crate::trust::dev_enabled(), dirs, settings, product) { let _ = std::fs::remove_dir_all(folder); }
    if let Some(mut entry) = lifecycle::read_journal(roots, id) { entry["undo"] = Value::Null; let _ = lifecycle::save_journal(roots, &entry); }
    if !crate::trust::dev_enabled() { let _ = std::fs::remove_dir(secondbrain::respected_root()); } // only when empty
}

/// Uninstall every Hub-managed product. Returns one entry per step; `ok` is false when any step failed.
pub fn remove_products(roots: &Roots, dirs: &secondbrain::BrainDirs, settings: &Settings) -> Value {
    let mut steps: Vec<Value> = Vec::new();
    let mut ok = true;
    let record = |product: &str, outcome: Result<Value, lifecycle::LifecycleError>, steps: &mut Vec<Value>, ok: &mut bool| match outcome {
        Ok(_) => steps.push(json!({ "product": product, "ok": true })),
        Err(error) => { *ok = false; steps.push(json!({ "product": product, "ok": false, "error": error.message })); }
    };
    // AI-client registrations of the Orchestrator MCP server first (also done by its uninstall; harmless twice).
    crate::orchestrator::mcp::disable_all(roots, settings, &rpp::generate_operation_id());

    let brain_installed = secondbrain::status(roots, dirs)["installed"] == true;
    if brain_installed {
        let outcome = hub_apply(roots, dirs, settings.clone(), "secondbrain", "uninstall", &json!({}), &rpp::generate_operation_id());
        record("secondbrain", outcome, &mut steps, &mut ok);
    }
    if lifecycle::read_current(roots, "orchestrator").is_some() {
        let outcome = hub_apply(roots, dirs, settings.clone(), "orchestrator", "uninstall", &json!({}), &rpp::generate_operation_id());
        record("orchestrator", outcome, &mut steps, &mut ok);
    }
    if ok {
        // everything undone: the Hub's managed copies, staging and backups are no longer needed
        let _ = std::fs::remove_dir_all(&roots.managed);
    }
    json!({ "ok": ok, "steps": steps, "kept": "not kasaları ve notlar silinmedi" })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn settings(data: &str) -> Settings {
        Settings { python: "python".into(), orchestrator_root: PathBuf::from("x"), brain_root: PathBuf::from("y"), data_root: PathBuf::from(data),
            brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None }
    }

    #[test]
    fn a_production_removal_deletes_only_the_products_own_named_data_folders() {
        let dirs = secondbrain::BrainDirs { app: PathBuf::from("C:/L/RespectedBrain/app"), data: PathBuf::from("C:/L/RespectedBrain/data") };
        let orchestrator = settings("C:/L/RespectedOrchestrator/data");
        assert_eq!(data_folders_to_delete(true, &dirs, &orchestrator, "secondbrain"), vec![PathBuf::from("C:/L/RespectedBrain/data"), PathBuf::from("C:/L/RespectedBrain")]);
        assert_eq!(data_folders_to_delete(true, &dirs, &orchestrator, "orchestrator"), vec![PathBuf::from("C:/L/RespectedOrchestrator/data"), PathBuf::from("C:/L/RespectedOrchestrator")]);
        assert!(data_folders_to_delete(true, &dirs, &orchestrator, "unknown").is_empty());
    }

    #[test]
    fn a_folder_without_the_products_name_or_a_development_build_is_never_deleted() {
        let foreign = secondbrain::BrainDirs { app: PathBuf::from("D:/Notes/app"), data: PathBuf::from("D:/Notes/data") };
        assert!(data_folders_to_delete(true, &foreign, &settings("D:/Notes/data"), "secondbrain").is_empty(), "a data folder under someone else's folder");
        assert!(data_folders_to_delete(true, &foreign, &settings("D:/Vault"), "orchestrator").is_empty(), "not named data");
        let own = secondbrain::BrainDirs { app: PathBuf::from("C:/L/RespectedBrain/app"), data: PathBuf::from("C:/L/RespectedBrain/data") };
        assert!(data_folders_to_delete(false, &own, &settings("C:/L/RespectedOrchestrator/data"), "secondbrain").is_empty(), "development builds keep data");
    }
}
