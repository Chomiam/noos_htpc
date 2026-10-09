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
    pub hdr_capable: bool,
    pub app_is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscDriveInfo {
    pub device: String,
    pub name: String,
    pub transport: String,
    pub disc_inserted: bool,
    pub disc_type: String,
    pub disc_label: String,
}

pub fn detect_optical_drives() -> Option<DiscDriveInfo> {
    let sys_block = std::path::Path::new("/sys/block");
    if !sys_block.exists() {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(sys_block) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with("sr") {
                let dev_name = file_name;
                let dev_path = format!("/dev/{}", dev_name);
                let sys_dev = entry.path().join("device");

                // 1. Détection du type d'interface physique (USB vs SATA)
                let canonical = sys_dev.canonicalize().unwrap_or_else(|_| sys_dev.clone());
                let canonical_str = canonical.to_string_lossy().to_lowercase();
                let transport = if canonical_str.contains("/usb") || canonical_str.contains("usb") {
                    "USB".to_string()
                } else {
                    "SATA".to_string()
                };

                // 2. Modèle et marque du lecteur
                let vendor = std::fs::read_to_string(sys_dev.join("vendor"))
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let model = std::fs::read_to_string(sys_dev.join("model"))
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let drive_name = if !model.is_empty() {
                    format!("{} {}", vendor, model).trim().to_string()
                } else {
                    format!("Lecteur Optique {}", transport)
                };

                // 3. Détection de la présence d'un disque physique et de son type
                let (disc_inserted, disc_type, disc_label) = inspect_disc_media(&dev_path);

                return Some(DiscDriveInfo {
                    device: dev_path,
                    name: drive_name,
                    transport,
                    disc_inserted,
                    disc_type,
                    disc_label,
                });
            }
        }
    }

    None
}

fn inspect_disc_media(dev_path: &str) -> (bool, String, String) {
    // 1. Détection avancée via udevadm
    if let Ok(output) = Command::new("udevadm")
        .args(["info", "-q", "property", "-n", dev_path])
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            let mut has_media = false;
            let mut is_bd = false;
            let mut is_dvd = false;
            let mut is_cd = false;
            let mut label = String::new();
            let mut fs_type = String::new();

            for line in text.lines() {
                if line == "ID_CDROM_MEDIA=1" {
                    has_media = true;
                } else if line == "ID_CDROM_MEDIA_BD=1" {
                    is_bd = true;
                } else if line == "ID_CDROM_MEDIA_DVD=1" {
                    is_dvd = true;
                } else if line == "ID_CDROM_MEDIA_CD=1" {
                    is_cd = true;
                } else if line.starts_with("ID_FS_LABEL=") {
                    label = line.trim_start_matches("ID_FS_LABEL=").to_string();
                } else if line.starts_with("ID_FS_TYPE=") {
                    fs_type = line.trim_start_matches("ID_FS_TYPE=").to_string();
                }
            }

            if has_media {
                let disc_type = if is_bd {
                    "Blu-ray".to_string()
                } else if is_dvd {
                    "DVD-Vidéo".to_string()
                } else if is_cd && (fs_type.is_empty() || fs_type == "audio") {
                    "CD Audio".to_string()
                } else if !fs_type.is_empty() {
                    format!("Disque ({})", fs_type.to_uppercase())
                } else {
                    "Disque Média".to_string()
                };

                let clean_label = if !label.is_empty() {
                    label.replace('_', " ")
                } else {
                    disc_type.clone()
                };

                return (true, disc_type, clean_label);
            }
        }
    }

    // 2. Repli rapide via blkid
    if let Ok(output) = Command::new("blkid").arg(dev_path).output() {
        if output.status.success() {
            if let Ok(text) = String::from_utf8(output.stdout) {
                if !text.trim().is_empty() {
                    let mut label = "Film / Disque".to_string();
                    if let Some(start) = text.find("LABEL=\"") {
                        let rest = &text[start + 7..];
                        if let Some(end) = rest.find('\"') {
                            label = rest[..end].replace('_', " ");
                        }
                    }
                    let disc_type = if text.contains("udf") {
                        "DVD / Blu-ray".to_string()
                    } else {
                        "Disque".to_string()
                    };
                    return (true, disc_type, label);
                }
            }
        }
    }

    (false, "Aucun disque".to_string(), String::new())
}

#[tauri::command]
pub async fn get_optical_drive() -> Result<Option<DiscDriveInfo>, String> {
    Ok(detect_optical_drives())
}

#[tauri::command]
pub async fn play_disc(app: AppHandle, device: Option<String>) -> Result<bool, String> {
    let drive_info = detect_optical_drives()
        .ok_or_else(|| "Aucun lecteur optique détecté".to_string())?;

    let dev_path = device.unwrap_or(drive_info.device);
    let disc_type = drive_info.disc_type.to_lowercase();

    let (target, extra_args): (String, Vec<String>) = if disc_type.contains("blu-ray") || disc_type.contains("bd") {
        ("bd://".to_string(), vec![format!("--bluray-device={}", dev_path)])
    } else if disc_type.contains("dvd") {
        ("dvd://".to_string(), vec![format!("--dvd-device={}", dev_path)])
    } else if disc_type.contains("cd") {
        ("cdda://".to_string(), vec![format!("--cdrom-device={}", dev_path)])
    } else {
        ("dvd://".to_string(), vec![format!("--dvd-device={}", dev_path)])
    };

    APP_RUNNING.store(true, Ordering::SeqCst);
    let _ = app.emit("app_state_changed", true);

    let app_handle = app.clone();
    tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new("mpv");
        cmd.arg(&target);
        for arg in extra_args {
            cmd.arg(arg);
        }
        cmd.args(["--fs", "--profile=auto-upscale"]);

        tracing::info!("Lancement de la lecture du disque : {:?}", cmd);
        let status = cmd.status();
        APP_RUNNING.store(false, Ordering::SeqCst);
        let _ = app_handle.emit("app_state_changed", false);
        let _ = app_handle.emit("app_closed", "disc_player");
        tracing::info!("Fin de la lecture disque : {:?}", status);
    });

    Ok(true)
}

#[tauri::command]
pub async fn eject_disc(device: Option<String>) -> Result<bool, String> {
    let dev = device.unwrap_or_else(|| {
        detect_optical_drives()
            .map(|d| d.device)
            .unwrap_or_else(|| "/dev/sr0".to_string())
    });
    tokio::task::spawn_blocking(move || {
        let _ = Command::new("eject").arg(&dev).status();
    });
    Ok(true)
}

#[tauri::command]
pub async fn launch_app(app: AppHandle, app_id: String) -> Result<bool, String> {
    if APP_RUNNING.load(Ordering::SeqCst) {
        return Err("Une application est déjà en cours d'exécution".to_string());
    }

    if app_id == "disc_player" {
        return play_disc(app, None).await;
    }

    let (program, args): (String, Vec<String>) = match app_id.as_str() {
        "vacuumtube" => {
            if Command::new("which").arg("vacuumtube").output().map(|o| o.status.success()).unwrap_or(false) {
                ("vacuumtube".to_string(), vec![])
            } else {
                (
                    "flatpak".to_string(),
                    vec![
                        "run".to_string(),
                        "--nosocket=x11".to_string(),
                        "--socket=wayland".to_string(),
                        "rocks.shy.VacuumTube".to_string(),
                        "--ozone-platform-hint=auto".to_string(),
                        "--ozone-platform=wayland".to_string(),
                        "--enable-features=WaylandWindowDecorations".to_string(),
                    ],
                )
            }
        }
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
        let mut cmd = Command::new(&program);
        cmd.args(&args);

        // Garantir les variables de session Wayland et DBus
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        cmd.env("XDG_RUNTIME_DIR", &runtime_dir);
        if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err() {
            cmd.env("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}/bus", runtime_dir));
        }
        if std::env::var("WAYLAND_DISPLAY").is_err() {
            cmd.env("WAYLAND_DISPLAY", "wayland-0");
        }

        let status = cmd.status();
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
    let mut hdr_capable = false;
    if let Ok(output) = Command::new("kscreen-doctor").arg("-j").output() {
        if let Ok(text) = String::from_utf8(output.stdout) {
            if text.contains("\"hdr\":true") || text.contains("\"hdr\": true") {
                hdr_enabled = true;
            }
            if text.contains("\"hdrCapable\":true") || text.contains("\"hdrCapable\": true") || text.contains("\"hdr\":") {
                hdr_capable = true;
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
        hdr_capable,
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
