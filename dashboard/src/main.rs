#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod home_button;
mod iptv;

use commands::{
    apply_system_update, check_for_updates, eject_disc, get_audio_sinks, get_keyboard_config,
    get_master_volume, get_optical_drive, get_system_info, get_upscale_info,
    hide_virtual_keyboard, launch_app, play_disc, power_action, restart_dashboard,
    set_audio_sink, set_keyboard_config, set_master_volume, set_upscale_profile,
    show_virtual_keyboard, toggle_hdr, toggle_master_mute,
};
use iptv::{
    iptv_delete_profile, iptv_get_cache_summary, iptv_get_catalog, iptv_get_channel_epg,
    iptv_get_favorites, iptv_get_filters_data, iptv_get_player_settings, iptv_get_saved_profiles,
    iptv_get_series_details, iptv_login_and_sync, iptv_play_stream, iptv_save_hidden_categories,
    iptv_save_player_settings, iptv_toggle_favorite,
};
use tauri::Emitter;

fn main() {
    // Forcer l'accélération matérielle et le compositing GPU WebKitGTK
    std::env::remove_var("WEBKIT_DISABLE_COMPOSITING_MODE");
    std::env::set_var("WEBKIT_FORCE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_ENABLE_GPU_PROCESS", "1");
    std::env::set_var("WEBKIT_ENABLE_ACCELERATED_2D_CANVAS", "1");
    std::env::set_var("WEBKIT_ENABLE_WEBGL", "1");

    // Support SSL/TLS pour WebKitGTK / GIO (indispensable pour charger les images HTTPS TMDB sur NixOS)
    if std::env::var("GIO_EXTRA_MODULES").is_err() {
        if let Ok(paths) = std::fs::read_dir("/nix/store") {
            for entry in paths.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.contains("glib-networking") && !name_str.ends_with(".drv") {
                    let modules_path = entry.path().join("lib/gio/modules");
                    if modules_path.exists() {
                        std::env::set_var("GIO_EXTRA_MODULES", modules_path.to_string_lossy().to_string());
                        break;
                    }
                }
            }
        }
    }
    if std::env::var("SSL_CERT_FILE").is_err() && std::path::Path::new("/etc/ssl/certs/ca-bundle.crt").exists() {
        std::env::set_var("SSL_CERT_FILE", "/etc/ssl/certs/ca-bundle.crt");
    }
    if std::env::var("NIX_SSL_CERT_FILE").is_err() && std::path::Path::new("/etc/ssl/certs/ca-bundle.crt").exists() {
        std::env::set_var("NIX_SSL_CERT_FILE", "/etc/ssl/certs/ca-bundle.crt");
    }

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
            get_audio_sinks,
            set_audio_sink,
            get_master_volume,
            set_master_volume,
            toggle_master_mute,
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
            iptv_get_cache_summary,
            iptv_get_catalog,
            iptv_get_channel_epg,
            iptv_get_series_details,
            iptv_get_filters_data,
            iptv_save_hidden_categories,
            iptv_get_favorites,
            iptv_toggle_favorite,
            iptv_play_stream,
            iptv_get_player_settings,
            iptv_save_player_settings,
            show_virtual_keyboard,
            hide_virtual_keyboard
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors de l'exécution du Dashboard Noos TV");
}
