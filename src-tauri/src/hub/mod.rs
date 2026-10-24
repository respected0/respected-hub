//! What the Hub does with the products: show what is installed (read from the products), offer the catalog's installers and
//! run them. The Hub keeps nothing about a product; its own folder holds only the download cache and its activity journal.
use crate::lifecycle::{self, catalog};
use crate::{rpp, trust, Settings};
use serde_json::{json, Value};
use std::path::Path;

pub mod installed;
pub mod installer;
pub mod startup;

/// `<Respected>\RespectedHub\managed`: the Hub's own download cache (catalog bundles) and activity journal. Only a development
/// build may point elsewhere or fall back to `<ecosystem>/catalog` built by common/release/tools.
pub fn standard_roots() -> lifecycle::Roots {
    let managed = trust::dev_abs("RESPECTED_HUB_MANAGED_ROOT").unwrap_or_else(|| installed::respected_root().join("RespectedHub").join("managed"));
    let sibling = if trust::dev_enabled() { Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).map(|p| p.join("catalog")).filter(|p| p.is_dir()) } else { None };
    let catalog = trust::dev_abs("RESPECTED_HUB_CATALOG_ROOT").or(sibling).unwrap_or_else(|| managed.join("catalog"));
    lifecycle::Roots { managed, catalog }
}

/// Point settings at the installed products' own programs.
pub fn overlay(settings: &mut Settings) {
    if let Some(i) = installed::product("orchestrator").and_then(|p| p.installed()) { settings.orchestrator_exe = Some(i.exe); }
    if let Some(i) = installed::product("brain").and_then(|p| p.installed()) { settings.brain_exe = Some(i.exe); }
}

/// The product's own word on its installation (its manifest): a damaged program reports `repair-needed`.
fn product_state(settings: &Settings, product: &str, found: &installed::Installed) -> String {
    let mut live = settings.clone();
    if product == "orchestrator" { live.orchestrator_exe = Some(found.exe.clone()); } else { live.brain_exe = Some(found.exe.clone()); }
    let reply = rpp::rpp_call(&live, product, "manifest", None, None);
    reply["result"]["install_state"].as_str().filter(|_| reply["ok"] == true).unwrap_or("repair-needed").to_string()
}

fn product_status(roots: &lifecycle::Roots, settings: &Settings, product: &installed::Product) -> Value {
    let current = product.installed();
    let (offered, skipped) = lifecycle::installers(roots, product.id);
    let newest = offered.first();
    let state = match (&current, newest) {
        (None, _) => "absent".to_string(),
        (Some(i), _) if product_state(settings, product.id, i) == "repair-needed" => "repair-needed".to_string(),
        (Some(i), Some(n)) if lifecycle::version_key(&n.version) > lifecycle::version_key(&i.version) => "update-available".to_string(),
        _ => "installed".to_string(),
    };
    let mut status = json!({
        "product": product.id, "installed": current.is_some(), "install_state": state, "version": current.as_ref().map(|i| i.version.clone()),
        "root": current.as_ref().map(|i| i.app_dir.to_string_lossy().to_string()),
        "recorded_at": current.as_ref().and_then(|i| i.recorded_at.clone()),
        "available": newest.map(|b| json!({ "version": b.version, "signed": b.signed, "trust": b.trust })), "catalog_skipped": skipped,
    });
    catalog::overlay_status(roots, &mut status);
    status
}

pub fn hub_status(roots: &lifecycle::Roots, settings: &Settings) -> Value {
    let products: Vec<Value> = installed::PRODUCTS.iter().map(|p| product_status(roots, settings, p)).collect();
    json!({ "ok": true, "result": { "products": products, "catalog_root": roots.catalog }, "error": Value::Null })
}

/// Download (once) the installer an action needs: the newest release; a removal needs none.
fn ensure_installer(roots: &lifecycle::Roots, product: &str, action: &str) -> Result<(), lifecycle::LifecycleError> {
    if action == "uninstall" { return Ok(()); }
    catalog::ensure_bundle(roots, product, None)
}

pub fn hub_plan(roots: &lifecycle::Roots, product: &str, action: &str, params: &Value) -> Result<Value, lifecycle::LifecycleError> {
    ensure_installer(roots, product, action)?;
    installer::plan(roots, product, action, params)
}

pub fn hub_apply(roots: &lifecycle::Roots, settings: Settings, product: &str, action: &str, params: &Value, id: &str) -> Result<Value, lifecycle::LifecycleError> {
    ensure_installer(roots, product, action)?;
    // after the installer: the product must answer for itself (its own manifest says it is installed)
    let verify = |found: &installed::Installed| -> Result<(), String> {
        let mut live = settings.clone();
        if product == "orchestrator" { live.orchestrator_exe = Some(found.exe.clone()); } else { live.brain_exe = Some(found.exe.clone()); }
        let reply = rpp::rpp_call_trusted(&live, product, "manifest", None, None);
        if reply["ok"] != true { return Err(reply["error"]["message"].as_str().unwrap_or("yanıt yok").to_string()); }
        if reply["result"]["installed"] != true { return Err(format!("ürün durumu: {}", reply["result"]["install_state"])); }
        Ok(())
    };
    let applied = installer::apply(roots, product, action, params, id, &verify)?;
    // only the installer of what is installed now stays in the download cache (offline repair needs it); a removal keeps none
    match installed::product(product).and_then(|p| p.installed()) { Some(i) => catalog::prune(roots, product, &i.version), None => catalog::forget(roots, product) }
    Ok(applied)
}
