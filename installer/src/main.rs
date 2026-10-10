#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::disk::list_disks;
use commands::gpu::{detect_gpu, detect_hardware};
use commands::network::{scan_wifi, connect_wifi, get_network_status};
use commands::install::{start_installation, get_install_progress, reboot_system};

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_disks,
            detect_gpu,
            detect_hardware,
            scan_wifi,
            connect_wifi,
            get_network_status,
            start_installation,
            get_install_progress,
            reboot_system
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors de l'exécution de l'installateur Noos HTPC");
}
