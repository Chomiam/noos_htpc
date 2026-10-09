#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod home_button;
mod iptv;

use commands::{
    apply_system_update, check_for_updates, eject_disc, get_keyboard_config, get_optical_drive,
    get_system_info, get_upscale_info, launch_app, play_disc, power_action, restart_dashboard,
    set_keyboard_config, set_upscale_profile, toggle_hdr,
};
use iptv::{
    iptv_delete_profile, iptv_get_cache_summary, iptv_get_saved_profiles, iptv_login_and_sync,
};
use tauri::Emitter;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            home_button::start_home_button_listener(handle.clone());

            // Vérification périodique des mises à jour pour pastille rouge dynamique
            let handle_update = handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                loop {
                    if let Ok(info) = commands::check_for_updates("testing".to_string()).await {
                        let _ = handle_update.emit("update_badge_status", info.has_update);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(45)).await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            launch_app,
            get_system_info,
            toggle_hdr,
            power_action,
            get_optical_drive,
            play_disc,
            eject_disc,
            check_for_updates,
            apply_system_update,
            get_keyboard_config,
            set_keyboard_config,
            restart_dashboard,
            get_upscale_info,
            set_upscale_profile,
            iptv_get_saved_profiles,
            iptv_login_and_sync,
            iptv_delete_profile,
            iptv_get_cache_summary
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors de l'exécution du Dashboard Noos TV");
}
