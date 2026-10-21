#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if std::env::args().any(|arg| arg == "--uninstall-products") { std::process::exit(respected_hub_lib::headless_uninstall_products()); }
    if std::env::args().any(|arg| arg == "--sync-registrations") { std::process::exit(respected_hub_lib::headless_sync_registrations()); }
    respected_hub_lib::run()
}
