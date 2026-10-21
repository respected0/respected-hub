//! What the Hub does with the products it manages: where they live, their status, and plan/apply/rollback across the
//! catalog, the product modules and the Installed-apps entries.
use crate::lifecycle::{self, catalog};
use crate::{apply_orchestrator, rpp, secondbrain, trust, Settings, ORCHESTRATOR_EXE};
use serde_json::{json, Value};
use std::path::Path;

pub mod cleanup;
pub mod installer;
pub mod registration;
pub mod standalone;
pub mod startup;

/// `<Respected>\RespectedHub\managed` and its catalog cache (bundles downloaded from GitHub Releases). Only a development build
/// may point elsewhere or fall back to `<ecosystem>/catalog` built by release/tools.
pub fn standard_roots() -> lifecycle::Roots {
    let managed = trust::dev_abs("RESPECTED_HUB_MANAGED_ROOT").unwrap_or_else(|| secondbrain::respected_root().join("RespectedHub").join("managed"));
    let sibling = if trust::dev_enabled() { Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).map(|p| p.join("catalog")).filter(|p| p.is_dir()) } else { None };
    let catalog = trust::dev_abs("RESPECTED_HUB_CATALOG_ROOT").or(sibling).unwrap_or_else(|| managed.join("catalog"));
    lifecycle::Roots { managed, catalog }
}

/// Point settings at the Hub-managed copies: the packaged Orchestrator and SecondBrain's installed launcher.
pub fn overlay(roots: &lifecycle::Roots, dirs: &secondbrain::BrainDirs, settings: &mut Settings) {
    if let Some(exe) = lifecycle::installed_launcher(roots, "orchestrator", ORCHESTRATOR_EXE).or_else(|| standalone::orchestrator().map(|s| s.exe)) { settings.orchestrator_exe = Some(exe); }
    secondbrain::overlay(roots, dirs, settings);
}

/// Component status: SecondBrain comes from its own engine, the rest from the Hub-managed copies.
pub fn hub_status(roots: &lifecycle::Roots, d: &secondbrain::BrainDirs) -> Value {
    let products: Vec<Value> = lifecycle::PRODUCTS.iter().map(|p| {
        let mut status = if *p == "secondbrain" { secondbrain::status(roots, d) } else { lifecycle::status(roots, p).unwrap_or(Value::Null) };
        if let Some(own) = standalone_of(roots, p) { status = own.status(); }
        catalog::overlay_status(roots, &mut status);
        status
    }).collect();
    json!({ "ok": true, "result": { "products": products, "managed_root": roots.managed, "catalog_root": roots.catalog, "brain_app_dir": d.app, "brain_data_dir": d.data }, "error": Value::Null })
}

/// Download (once) the bundles an action needs: the newest release for install/update, the installed version for repair.
fn ensure_bundles(roots: &lifecycle::Roots, d: &secondbrain::BrainDirs, product: &str, action: &str) -> Result<(), lifecycle::LifecycleError> {
    match action {
        "install" | "update" => {
            catalog::ensure_bundle(roots, product, None)?;
        }
        "repair" => {
            catalog::ensure_bundle(roots, product, installed_version(roots, d, product).as_deref())?;
        }
        _ => {}
    }
    Ok(())
}

/// The installed version of a product, wherever it lives (the Hub's managed copy, SecondBrain's engine or the product's own installer).
pub fn installed_version(roots: &lifecycle::Roots, d: &secondbrain::BrainDirs, product: &str) -> Option<String> {
    if product == "secondbrain" { return secondbrain::status(roots, d)["version"].as_str().map(str::to_string); }
    lifecycle::read_current(roots, product).map(|c| c.0).or_else(|| standalone::orchestrator().map(|s| s.version))
}

/// The product's own installation, when it was installed by its own installer and the Hub holds no managed copy of it.
fn standalone_of(roots: &lifecycle::Roots, product: &str) -> Option<standalone::Standalone> {
    if product != "orchestrator" || lifecycle::read_current(roots, product).is_some() { return None; }
    standalone::orchestrator()
}

/// A SecondBrain that has its own installer is removed through that installer's uninstaller, so its Installed-apps entry goes too.
fn owns_removal(d: &secondbrain::BrainDirs, product: &str, action: &str) -> bool { product == "secondbrain" && action == "uninstall" && standalone::brain_has_installer(d) }

pub fn hub_plan(roots: &lifecycle::Roots, d: &secondbrain::BrainDirs, settings: &Settings, product: &str, action: &str, params: &Value) -> Result<Value, lifecycle::LifecycleError> {
    if owns_removal(d, product, action) { return Ok(installer::plan_brain_removal()); }
    ensure_bundles(roots, d, product, action)?;
    if installer::applies(roots, product, action) { return installer::plan(roots, action); }
    if standalone_of(roots, product).is_some() { return Err(standalone::conflict()); }
    if product == "secondbrain" { return secondbrain::plan(roots, d, settings, action, params); }
    lifecycle::plan(roots, product, action)
}

pub fn hub_apply(roots: &lifecycle::Roots, d: &secondbrain::BrainDirs, settings: Settings, product: &str, action: &str, params: &Value, id: &str) -> Result<Value, lifecycle::LifecycleError> {
    ensure_bundles(roots, d, product, action)?;
    if standalone_of(roots, product).is_some() && !installer::applies(roots, product, action) { return Err(standalone::conflict()); }
    let applied = if owns_removal(d, product, action) { installer::remove_brain(roots, d, id) } else { hub_apply_inner(roots, d, settings.clone(), product, action, params, id) };
    if applied.is_ok() {
        if action == "uninstall" { cleanup::after_uninstall(roots, d, &settings, product, id); } else { sweep(roots, d); }
        registration::sync(roots, d);
    }
    // a removal also deleted the product's data and the undo copy: the result must not offer "Geri al" either
    applied.map(|mut result| { if action == "uninstall" && result.get("undo").is_some() { result["undo"] = Value::Null; } result })
}

/// No leftovers: only the installed version of each component stays, in the managed folder and in the catalog cache.
pub fn sweep(roots: &lifecycle::Roots, d: &secondbrain::BrainDirs) {
    for product in lifecycle::PRODUCTS {
        lifecycle::prune_old_versions(roots, product);
        if let Some(version) = installed_version(roots, d, product) { catalog::prune(roots, product, &version); }
    }
}

fn hub_apply_inner(roots: &lifecycle::Roots, d: &secondbrain::BrainDirs, settings: Settings, product: &str, action: &str, params: &Value, id: &str) -> Result<Value, lifecycle::LifecycleError> {
    if product == "secondbrain" {
        let applied = secondbrain::apply(roots, d, &settings, action, params, id);
        if applied.is_ok() && matches!(action, "install" | "update" | "repair") {
            let mut fresh = settings.clone();
            secondbrain::overlay(roots, d, &mut fresh);
            if let Some(command) = rpp::brain_rpp_command(&fresh) {
                secondbrain::memory_access::refresh_command(&|p: &str, o: &str, pl: Option<Value>, i: Option<String>| rpp::rpp_call_trusted(&fresh, p, o, pl, i), command);
            }
        }
        return applied;
    }
    if installer::applies(roots, product, action) { return installer::apply(roots, &settings, action, id); }
    // shortcut: the Hub-managed copy stays for catalogs that carry no installer bundle yet, delete it (and lifecycle::apply's Orchestrator use) once a release publishes one
    apply_orchestrator(roots, settings, action, id)
}

pub fn hub_rollback(roots: &lifecycle::Roots, d: &secondbrain::BrainDirs, id: &str) -> Result<Value, lifecycle::LifecycleError> {
    match lifecycle::read_journal(roots, id) {
        Some(entry) if entry["undo"]["engine"] == true => secondbrain::rollback(roots, d, id),
        _ => lifecycle::rollback(roots, id),
    }
}
