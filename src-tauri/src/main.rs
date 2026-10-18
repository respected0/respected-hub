#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if std::env::args().any(|arg| arg == "--uninstall-products") { std::process::exit(respected_hub_lib::headless_uninstall_products()); }
    respected_hub_lib::run()
}
