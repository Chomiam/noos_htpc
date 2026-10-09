#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::{get_system_info, launch_app, power_action, toggle_hdr};

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            launch_app,
            get_system_info,
            toggle_hdr,
            power_action
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors de l'exécution du Dashboard Noos TV");
}
