use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};

static APP_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub storage_free_gb: u32,
    pub storage_total_gb: u32,
    pub memory_used_mb: u32,
    pub memory_total_mb: u32,
    pub wifi_connected: bool,
    pub wifi_ssid: String,
    pub hdr_enabled: bool,
    pub app_is_active: bool,
}

#[tauri::command]
pub async fn launch_app(app: AppHandle, app_id: String) -> Result<bool, String> {
    if APP_RUNNING.load(Ordering::SeqCst) {
        return Err("Une application est déjà en cours d'exécution".to_string());
    }

    let (program, args): (String, Vec<String>) = match app_id.as_str() {
        "vacuumtube" => {
            if Command::new("which").arg("vacuumtube").output().map(|o| o.status.success()).unwrap_or(false) {
                ("vacuumtube".to_string(), vec![])
            } else {
                ("flatpak".to_string(), vec!["run".to_string(), "rocks.shy.VacuumTube".to_string()])
            }
        }
        "pear-desktop" => ("pear-desktop".to_string(), vec![]),
        "noos-iptv" => ("noos-iptv".to_string(), vec![]),
        "retroarch" => ("retroarch".to_string(), vec![]),
        "es-de" => ("es-de".to_string(), vec![]),
        "thunar" => ("thunar".to_string(), vec!["/home/noos".to_string()]),
        _ => return Err(format!("Application inconnue : {}", app_id)),
    };

    APP_RUNNING.store(true, Ordering::SeqCst);
    let _ = app.emit("app_state_changed", true);

    let app_handle = app.clone();
    let app_id_clone = app_id.clone();

    tokio::task::spawn_blocking(move || {
        tracing::info!("Lancement de l'application : {} {:?}", program, args);
        let status = Command::new(&program).args(&args).status();
        APP_RUNNING.store(false, Ordering::SeqCst);
        let _ = app_handle.emit("app_state_changed", false);
        let _ = app_handle.emit("app_closed", app_id_clone);
        tracing::info!("Application terminée avec statut : {:?}", status);
    });

    Ok(true)
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let hostname = std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "noos-htpc".to_string());

    let mut storage_free_gb = 0;
    let mut storage_total_gb = 0;
    if let Ok(output) = Command::new("df").args(["-BG", "/"]).output() {
        if let Ok(text) = String::from_utf8(output.stdout) {
            if let Some(line) = text.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    storage_total_gb = parts[1].trim_end_matches('G').parse().unwrap_or(0);
                    storage_free_gb = parts[3].trim_end_matches('G').parse().unwrap_or(0);
                }
            }
        }
    }

    let mut memory_total_mb = 0;
    let mut memory_used_mb = 0;
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0;
        let mut avail_kb = 0;
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line.split_whitespace().nth(1).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                avail_kb = line.split_whitespace().nth(1).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
            }
        }
        memory_total_mb = total_kb / 1024;
        let used_kb = total_kb.saturating_sub(avail_kb);
        memory_used_mb = used_kb / 1024;
    }

    let mut wifi_connected = false;
    let mut wifi_ssid = String::new();
    if let Ok(output) = Command::new("nmcli").args(["-t", "-f", "active,ssid", "dev", "wifi"]).output() {
        if let Ok(text) = String::from_utf8(output.stdout) {
            for line in text.lines() {
                if line.starts_with("yes:") {
                    wifi_connected = true;
                    wifi_ssid = line.trim_start_matches("yes:").to_string();
                    break;
                }
            }
        }
    }

    let mut hdr_enabled = false;
    if let Ok(output) = Command::new("kscreen-doctor").arg("-j").output() {
        if let Ok(text) = String::from_utf8(output.stdout) {
            if text.contains("\"hdr\":true") || text.contains("\"hdr\": true") {
                hdr_enabled = true;
            }
        }
    }

    Ok(SystemInfo {
        hostname,
        storage_free_gb,
        storage_total_gb,
        memory_used_mb,
        memory_total_mb,
        wifi_connected,
        wifi_ssid,
        hdr_enabled,
        app_is_active: APP_RUNNING.load(Ordering::SeqCst),
    })
}

#[tauri::command]
pub async fn toggle_hdr(enable: bool) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let action = if enable { "hdr.enable" } else { "hdr.disable" };
        let _ = Command::new("kscreen-doctor").args([&format!("output.1.{}", action)]).status();
        let _ = Command::new("kscreen-doctor").args([&format!("output.HDMI-A-1.{}", action)]).status();
        let _ = Command::new("kscreen-doctor").args([&format!("output.DP-1.{}", action)]).status();
    });
    Ok(enable)
}

#[tauri::command]
pub async fn power_action(action: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        match action.as_str() {
            "poweroff" | "shutdown" => {
                let _ = Command::new("systemctl").arg("poweroff").status();
            }
            "reboot" => {
                let _ = Command::new("systemctl").arg("reboot").status();
            }
            "suspend" => {
                let _ = Command::new("systemctl").arg("suspend").status();
            }
            _ => {}
        }
    });
    Ok(())
}
