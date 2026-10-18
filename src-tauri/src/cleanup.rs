//! Removing the products the Hub installed when the Hub itself is uninstalled (`respected-hub.exe --uninstall-products`).
//!
//! The Hub is the only UI that can undo a product's AI-client registrations (SecondBrain hooks / MCP entry / scheduled task /
//! shortcut, Orchestrator MCP entries). Without this, uninstalling the Hub would leave them pointing at products nobody can
//! manage any more. It goes through the same journaled lifecycle paths as the UI: SecondBrain's own uninstaller (keeps the vault
//! and data, restores foreign config entries), Orchestrator, then the runtime; finally the Hub's managed folder is dropped.
//! Notes, vaults and Orchestrator run data are never touched.

use crate::lifecycle::{self, Roots};
use crate::{brain, hub_apply, rpp, Settings};
use serde_json::{json, Value};

/// After a successful per-product uninstall: leave nothing of ours behind except the vault and its notes (no undo copies, staging,
/// product data/backups, or the Python runtime once no product needs it). Data folders are only removed in production builds and
/// only when they carry the product's own folder name.
pub fn after_uninstall(roots: &Roots, dirs: &brain::BrainDirs, settings: &Settings, product: &str, id: &str) {
    let _ = std::fs::remove_dir_all(roots.managed.join(product));
    if !crate::trust::dev_enabled() {
        let own = |dir: &std::path::Path, name: &str| dir.file_name().map_or(false, |n| n == name);
        match product {
            "secondbrain" if own(&dirs.data, "data") && dirs.data.parent().map_or(false, |p| own(p, "RespectedBrain")) => { let _ = std::fs::remove_dir_all(&dirs.data); if let Some(folder) = dirs.data.parent() { let _ = std::fs::remove_dir_all(folder); } }
            "orchestrator" if own(&settings.data_root, "data") && settings.data_root.parent().map_or(false, |p| own(p, "RespectedOrchestrator")) => { let _ = std::fs::remove_dir_all(&settings.data_root); if let Some(folder) = settings.data_root.parent() { let _ = std::fs::remove_dir_all(folder); } }
            _ => {}
        }
    }
    if let Some(mut entry) = lifecycle::read_journal(roots, id) { entry["undo"] = Value::Null; let _ = lifecycle::save_journal(roots, &entry); }
    if !crate::trust::dev_enabled() { let _ = std::fs::remove_dir(brain::respected_root()); } // only when empty
    if product == "orchestrator" && lifecycle::read_current(roots, "runtime").is_some() {
        let runtime_id: String = format!("rt-{id}").chars().take(64).collect();
        let _ = hub_apply(roots, dirs, settings.clone(), "runtime", "uninstall", &json!({}), &runtime_id);
    }
}

/// Uninstall every Hub-managed product. Returns one entry per step; `ok` is false when any step failed.
pub fn remove_products(roots: &Roots, dirs: &brain::BrainDirs, settings: &Settings) -> Value {
    let mut steps: Vec<Value> = Vec::new();
    let mut ok = true;
    let mut record = |product: &str, outcome: Result<Value, lifecycle::LifecycleError>, steps: &mut Vec<Value>, ok: &mut bool| match outcome {
        Ok(_) => steps.push(json!({ "product": product, "ok": true })),
        Err(error) => { *ok = false; steps.push(json!({ "product": product, "ok": false, "error": error.message })); }
    };
    // AI-client registrations of the Orchestrator MCP server first (also done by its uninstall; harmless twice).
    crate::mcp_access::disable_all(roots, settings, &rpp::generate_operation_id());

    let brain_installed = brain::status(roots, dirs)["installed"] == true;
    if brain_installed {
        let outcome = hub_apply(roots, dirs, settings.clone(), "secondbrain", "uninstall", &json!({}), &rpp::generate_operation_id());
        record("secondbrain", outcome, &mut steps, &mut ok);
    }
    for product in ["orchestrator", "runtime"] {
        if lifecycle::read_current(roots, product).is_some() {
            let outcome = hub_apply(roots, dirs, settings.clone(), product, "uninstall", &json!({}), &rpp::generate_operation_id());
            record(product, outcome, &mut steps, &mut ok);
        }
    }
    if ok {
        // everything undone: the Hub's managed copies, staging and backups are no longer needed
        let _ = std::fs::remove_dir_all(&roots.managed);
    }
    json!({ "ok": ok, "steps": steps, "kept": "notlar, kasalar ve Orchestrator çalışma kayıtları silinmedi" })
}
