#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod home_button;

use commands::{
    eject_disc, get_optical_drive, get_system_info, launch_app, play_disc, power_action, toggle_hdr,
};

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            home_button::start_home_button_listener(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            launch_app,
            get_system_info,
            toggle_hdr,
            power_action,
            get_optical_drive,
            play_disc,
            eject_disc
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors de l'exécution du Dashboard Noos TV");
}
