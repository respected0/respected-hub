pub mod commands;
pub mod hub;
pub mod lifecycle;
pub mod process;
pub mod products;
pub mod rpp;
pub mod settings;
pub mod trust;

// product modules keep their short paths (crate::brain, respected_hub_lib::orchestrator, ...)
pub use products::{orchestrator, brain};
pub use orchestrator::{call_engine, list_campaigns, valid_campaign_id};
pub use hub::{hub_apply, hub_plan, hub_status, overlay, standard_roots};
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
            commands::engine::engine_request, commands::engine::rpp_request, commands::engine::product_status, commands::engine::development_build, commands::engine::campaign_list,
            commands::manage::startup_intent, commands::manage::lifecycle_status, commands::manage::lifecycle_plan, commands::manage::detect_ai_clients,
            commands::manage::lifecycle_apply, commands::manage::activity_list, commands::manage::catalog_refresh, commands::manage::catalog_set_channel,
            commands::manage::hub_update_status, commands::manage::hub_update_install, commands::manage::orchestrator_mcp_plan,
            commands::manage::orchestrator_mcp_apply,
            commands::memory::memory_access_status, commands::memory::memory_access_enable, commands::memory::memory_access_disable,
            commands::memory::memory_index_status, commands::memory::memory_index_refresh, commands::memory::memory_semantic_status,
            commands::memory::memory_semantic_enable, commands::memory::memory_semantic_disable, commands::memory::memory_capture_run,
            commands::memory::memory_search, commands::memory::memory_note_read, commands::memory::memory_capture_note,
            commands::memory::memory_briefing_run])
        .setup(|app| { if let Ok(roots) = commands::lifecycle_roots(app.handle()) {
            lifecycle::close_interrupted(&roots); // a run this Hub was closed in the middle of: the product's installer kept it consistent
            let handle = app.handle().clone();
            std::thread::spawn(move || { use tauri::Emitter; let _ = lifecycle::catalog::refresh(&roots); let _ = handle.emit("catalog-updated", ()); });
        } Ok(()) })
        .run(tauri::generate_context!()).expect("Respected Hub startup failed");
}
