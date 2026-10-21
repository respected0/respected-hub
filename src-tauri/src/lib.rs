pub mod commands;
pub mod hub;
pub mod lifecycle;
pub mod process;
pub mod products;
pub mod rpp;
pub mod settings;
pub mod trust;

// product modules keep their short paths (crate::secondbrain, respected_hub_lib::orchestrator, ...)
pub use products::{orchestrator, secondbrain};
pub use orchestrator::{apply_orchestrator, call_engine, engine_verifier, list_campaigns, valid_campaign_id, ORCHESTRATOR_EXE};
pub use hub::{hub_apply, hub_plan, hub_rollback, hub_status, overlay, standard_roots, sweep};
pub use hub::startup::{headless_sync_registrations, headless_uninstall_products};
pub use settings::{get_settings, Settings};
pub(crate) use process::{pump, PumpError};

pub fn run() {
    tauri::Builder::default()
        // a second window would run startup recovery against the first one's live install (audit P1 #9): focus the existing one instead
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            use tauri::{Emitter, Manager};
            if let Some(window) = app.webview_windows().values().next() { let _ = window.unminimize(); let _ = window.set_focus(); }
            // a product installer started meanwhile: tell the open window which product the user came to install
            if let Some(wanted) = hub::startup::intent(args.into_iter()) { let _ = app.emit("startup-intent", wanted); }
        }))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![settings::get_settings, settings::save_settings,
            commands::engine::engine_request, commands::engine::rpp_request, commands::engine::product_status, commands::engine::campaign_list,
            commands::manage::startup_intent, commands::manage::lifecycle_status, commands::manage::lifecycle_plan,
            commands::manage::lifecycle_apply, commands::manage::lifecycle_rollback, commands::manage::lifecycle_purge_backups,
            commands::manage::activity_list, commands::manage::catalog_refresh, commands::manage::catalog_set_channel,
            commands::manage::hub_update_status, commands::manage::hub_update_install, commands::manage::orchestrator_mcp_plan,
            commands::manage::orchestrator_mcp_apply,
            commands::memory::memory_access_status, commands::memory::memory_access_enable, commands::memory::memory_access_disable,
            commands::memory::memory_index_status, commands::memory::memory_index_refresh, commands::memory::memory_semantic_status,
            commands::memory::memory_semantic_enable, commands::memory::memory_semantic_disable, commands::memory::memory_capture_run,
            commands::memory::memory_search, commands::memory::memory_note_read, commands::memory::memory_capture_note,
            commands::memory::memory_briefing_run])
        // recovery can take minutes (engine call): never hold the window back for it
        .setup(|app| { if let Ok(roots) = commands::lifecycle_roots(app.handle()) {
            // the catalog first and on its own: recovery below can take minutes, and the window must learn about products at once
            let (handle, catalog_roots) = (app.handle().clone(), roots.clone());
            std::thread::spawn(move || { use tauri::Emitter; let _ = lifecycle::catalog::refresh(&catalog_roots); let _ = handle.emit("catalog-updated", ()); });
            std::thread::spawn(move || {
                let dirs = secondbrain::dirs();
                secondbrain::recover_if_interrupted(&roots, &dirs);
                let _ = lifecycle::recover(&roots);
                sweep(&roots, &dirs);                 // a folder an MCP server still held at update time goes now
                hub::registration::sync(&roots, &dirs);   // Installed-apps entries follow what is really installed
                hub::registration::drop_maintenance(&roots);
            });
        } Ok(()) })
        .run(tauri::generate_context!()).expect("Respected Hub startup failed");
}
