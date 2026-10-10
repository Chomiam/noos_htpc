use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

static APP_RUNNING: AtomicBool = AtomicBool::new(false);
static CURRENT_CHILD_PID: Mutex<Option<u32>> = Mutex::new(None);

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
        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                if let Ok(mut lock) = CURRENT_CHILD_PID.lock() {
                    *lock = Some(pid);
                }
                let status = child.wait();
                if let Ok(mut lock) = CURRENT_CHILD_PID.lock() {
                    *lock = None;
                }
                tracing::info!("Fin de la lecture disque : {:?}", status);
            }
            Err(e) => {
                tracing::error!("Erreur au lancement du lecteur disque : {:?}", e);
            }
        }
        APP_RUNNING.store(false, Ordering::SeqCst);
        let _ = app_handle.emit("app_state_changed", false);
        let _ = app_handle.emit("app_closed", "disc_player");
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
        "jellyfin" => {
            ensure_jellyfin_fullscreen_config();
            if Command::new("which").arg("jellyfin-tv").output().map(|o| o.status.success()).unwrap_or(false) {
                ("jellyfin-tv".to_string(), vec![])
            } else if Command::new("which").arg("jellyfin-desktop").output().map(|o| o.status.success()).unwrap_or(false) {
                ("jellyfin-desktop".to_string(), vec!["--tv".to_string(), "--fullscreen".to_string()])
            } else {
                ("jellyfin-media-player".to_string(), vec!["--tv".to_string(), "--fullscreen".to_string()])
            }
        }
        "noos-iptv" => ("noos-iptv".to_string(), vec![]),
        "retroarch" => ("retroarch".to_string(), vec![]),
        "es-de" => ("es-de".to_string(), vec![]),
        "sober" => {
            ensure_sober_console_config();
            // Nettoyage préventif des processus résiduels Sober pour éviter le verrouillage "instance already running"
            let _ = Command::new("pkill").args(["-9", "-x", "sober"]).status();
            let _ = Command::new("pkill").args(["-9", "-x", "sober_services"]).status();
            std::thread::sleep(std::time::Duration::from_millis(150));

            if Command::new("which").arg("sober").output().map(|o| o.status.success()).unwrap_or(false) {
                ("sober".to_string(), vec![])
            } else {
                (
                    "flatpak".to_string(),
                    vec![
                        "run".to_string(),
                        "--device=all".to_string(),
                        "--socket=wayland".to_string(),
                        "--socket=x11".to_string(),
                        "--socket=fallback-x11".to_string(),
                        "org.vinegarhq.Sober".to_string(),
                    ],
                )
            }
        }
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
        cmd.env("QT_WAYLAND_DISABLE_WINDOWDECORATION", "1");
        cmd.env("QT_QPA_PLATFORM", "wayland;xcb");
        cmd.env("QT_WAYLAND_SHELL_INTEGRATION", "xdg-shell");
        // Assainir l'environnement pour garantir une détection native du GPU et le bon fonctionnement de SDL2
        cmd.env_remove("AMD_VULKAN_ICD");
        cmd.env_remove("RADV_PERFTEST");
        cmd.env_remove("LIBGL_ALWAYS_SOFTWARE");
        cmd.env_remove("SDL_VIDEODRIVER");

        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                if let Ok(mut lock) = CURRENT_CHILD_PID.lock() {
                    *lock = Some(pid);
                }
                let status = child.wait();
                if let Ok(mut lock) = CURRENT_CHILD_PID.lock() {
                    *lock = None;
                }
                tracing::info!("Application terminée avec statut : {:?}", status);
            }
            Err(e) => {
                tracing::error!("Erreur au lancement de l'application : {:?}", e);
            }
        }

        APP_RUNNING.store(false, Ordering::SeqCst);
        let _ = app_handle.emit("app_state_changed", false);
        let _ = app_handle.emit("app_closed", app_id_clone);
    });

    Ok(true)
}

/// Action universelle déclenchée par le bouton HOME de la manette ou télécommande
pub fn trigger_home_action(app: &AppHandle) {
    let running = APP_RUNNING.load(Ordering::SeqCst);
    let child_pid = if let Ok(mut lock) = CURRENT_CHILD_PID.lock() {
        lock.take()
    } else {
        None
    };

    if running || child_pid.is_some() {
        tracing::info!("Bouton HOME pressé : interruption de l'application active pour retour au lanceur...");

        if let Some(pid) = child_pid {
            let _ = Command::new("kill").args(["-TERM", &pid.to_string()]).status();
        }

        // Fermeture des processus multimédias / jeux éventuels
        let _ = Command::new("pkill").args(["-TERM", "-f", "rocks.shy.VacuumTube"]).status();
        let _ = Command::new("pkill").args(["-TERM", "-f", "vacuumtube"]).status();
        let _ = Command::new("pkill").args(["-TERM", "-f", "jellyfin-desktop"]).status();
        let _ = Command::new("pkill").args(["-TERM", "-f", "jellyfin"]).status();
        let _ = Command::new("pkill").args(["-TERM", "-f", "mpv"]).status();
        let _ = Command::new("pkill").args(["-TERM", "-f", "retroarch"]).status();
        let _ = Command::new("pkill").args(["-TERM", "-f", "es-de"]).status();
        let _ = Command::new("pkill").args(["-TERM", "-x", "sober"]).status();
        let _ = Command::new("pkill").args(["-TERM", "-x", "sober_services"]).status();
        let _ = Command::new("pkill").args(["-TERM", "-f", "org.vinegarhq.Sober"]).status();

        APP_RUNNING.store(false, Ordering::SeqCst);
        let _ = app.emit("app_state_changed", false);
        let _ = app_handle_closed_helper(app);
    }

    // Notifier le frontend pour fermer les dialogues et recentrer le focus sur l'accueil
    let _ = app.emit("home_pressed", ());
}

fn app_handle_closed_helper(app: &AppHandle) {
    let _ = app.emit("app_closed", "home_pressed");
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

/// S'assure que le profil Jellyfin Desktop et libmpv sont configurés en plein écran absolu sans bordures
fn ensure_jellyfin_fullscreen_config() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/noos".to_string());
    let base_dir = std::path::PathBuf::from(&home).join(".local/share/jellyfin-desktop");
    let _ = std::fs::create_dir_all(&base_dir);

    let conf_content = r#"{
    "sections": {
        "main": {
            "allowBrowserZoom": true,
            "alwaysOnTop": false,
            "autodetectCertBundle": true,
            "checkForUpdates": false,
            "disablemouse": false,
            "enableInputRepeat": true,
            "enableMPV": true,
            "enableWindowsMediaIntegration": true,
            "enableWindowsTaskbarIntegration": true,
            "forceAlwaysFS": true,
            "forceFSScreen": "",
            "fullscreen": true,
            "hdmi_poweron": false,
            "ignoreSSLErrors": false,
            "layout": "tv",
            "logLevel": "info",
            "minimizeOnDefocus": false,
            "sdlEnabled": true,
            "showPowerOptions": true,
            "useOpenGL": false,
            "useSystemVideoCodecs": true,
            "userWebClient": "",
            "webMode": "desktop"
        },
        "video": {
            "allow_transcode_to_hevc": false,
            "always_force_transcode": false,
            "force_transcode_4k": false,
            "force_transcode_av1": false,
            "force_transcode_dovi": false,
            "force_transcode_hdr": false,
            "force_transcode_hevc": false,
            "force_transcode_hi10p": false,
            "hardwareDecoding": "auto-safe",
            "prefer_transcode_to_h265": false,
            "refreshrate.auto_switch": false,
            "sync_mode": "audio"
        }
    },
    "version": 7
}"#;

    let mpv_content = "vo=gpu-next\ngpu-context=wayland\ntarget-colorspace-hint=yes\ntone-mapping=auto\nhdr-compute-peak=yes\nhwdec=auto-safe\nfs=yes\nborder=no\nkeep-open=no\n";

    // 1. Configuration racine
    let _ = std::fs::write(base_dir.join("jellyfin-desktop.conf"), conf_content);
    let _ = std::fs::write(base_dir.join("mpv.conf"), mpv_content);

    // 2. Propagation à tous les sous-profils existants
    let profiles_dir = base_dir.join("profiles");
    if let Ok(entries) = std::fs::read_dir(&profiles_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let _ = std::fs::write(entry.path().join("jellyfin-desktop.conf"), conf_content);
                let _ = std::fs::write(entry.path().join("mpv.conf"), mpv_content);
            }
        }
    }

    // 3. Configuration MPV globale de l'utilisateur
    let mpv_conf_dir = std::path::PathBuf::from(&home).join(".config/mpv");
    let _ = std::fs::create_dir_all(&mpv_conf_dir);
    let _ = std::fs::write(mpv_conf_dir.join("mpv.conf"), mpv_content);
}

/// S'assure que Sober (Roblox) est préconfiguré pour démarrer directement en mode console TV avec manette
fn ensure_sober_console_config() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/noos".to_string());
    let sober_dir = std::path::PathBuf::from(&home).join(".var/app/org.vinegarhq.Sober/config/sober");
    let _ = std::fs::create_dir_all(&sober_dir);
    let conf_path = sober_dir.join("config.json");

    if let Ok(mut content) = std::fs::read_to_string(&conf_path) {
        content = content.replace("\"use_console_experience\": false", "\"use_console_experience\": true");
        content = content.replace("\"allow_gamepad_permission\": false", "\"allow_gamepad_permission\": true");
        content = content.replace("\"close_on_leave\": false", "\"close_on_leave\": true");
        content = content.replace("\"enable_hidpi\": false", "\"enable_hidpi\": true");
        content = content.replace("\"use_opengl\": true", "\"use_opengl\": false");
        let _ = std::fs::write(&conf_path, content);
    } else {
        let default_conf = r#"{
    "allow_gamepad_permission": true,
    "close_on_leave": true,
    "discord_rpc_enabled": false,
    "discord_rpc_show_join_button": false,
    "enable_gamemode": true,
    "enable_hidpi": true,
    "enable_mobile_home_screen": false,
    "graphics_optimization_mode": "quality",
    "server_location_indicator_enabled": false,
    "touch_mode": "off",
    "use_console_experience": true,
    "use_libsecret": false,
    "use_opengl": false
}"#;
        let _ = std::fs::write(&conf_path, default_conf);
    }
}

/* ========================================================================= */
/* MODULE DE MISE A JOUR DECLARATIVE NOOS HTPC (GIT, FLAKES & NIXOS-REBUILD) */
/* ========================================================================= */

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub current_branch: String,
    pub target_channel: String,
    pub commits: Vec<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateProgress {
    pub step: u8,
    pub total_steps: u8,
    pub step_name: String,
    pub percent: u8,
    pub log_line: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyboardConfig {
    pub layout: String, // "azerty" ou "qwerty"
}

fn find_repo_dir() -> std::path::PathBuf {
    let candidates = [
        "/etc/nixos",
        "/home/chomiam/Projets/noos_htpc",
        "/etc/noos-htpc-source",
    ];
    for c in candidates {
        let p = std::path::PathBuf::from(c);
        if p.join(".git").exists() || p.join("flake.nix").exists() {
            return p;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/etc/nixos"))
}

#[tauri::command]
pub async fn check_for_updates(channel: String) -> Result<UpdateInfo, String> {
    tokio::task::spawn_blocking(move || {
        let repo = find_repo_dir();
        let target_channel = if channel.is_empty() { "testing".to_string() } else { channel };

        // 1. Récupération des métadonnées distantes et des tags de release
        let _ = Command::new("git")
            .current_dir(&repo)
            .args(["fetch", "origin", &target_channel, "--tags", "--quiet"])
            .status();

        let head_commit = Command::new("git")
            .current_dir(&repo)
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let remote_commit = Command::new("git")
            .current_dir(&repo)
            .args(["rev-parse", "--short", &format!("origin/{}", target_channel)])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| head_commit.clone());

        let current_branch = Command::new("git")
            .current_dir(&repo)
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "testing".to_string());

        // 2. Vérification des commits d'écart
        let log_output = Command::new("git")
            .current_dir(&repo)
            .args([
                "log",
                &format!("HEAD..origin/{}", target_channel),
                "--pretty=format:%h • %s (%cr)",
                "-n",
                "15",
            ])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        let commits: Vec<String> = log_output
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        let has_update = !commits.is_empty();

        // 3. Résolution dynamique des versions et tags
        let current_version = Command::new("git")
            .current_dir(&repo)
            .args(["describe", "--tags", "--abbrev=0"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !s.is_empty() { Some(s) } else { None }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| format!("v{}", env!("CARGO_PKG_VERSION")));

        let remote_tag = Command::new("git")
            .current_dir(&repo)
            .args(["describe", "--tags", "--abbrev=0", &format!("origin/{}", target_channel)])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !s.is_empty() { Some(s) } else { None }
                } else {
                    None
                }
            });

        let latest_version = if let Some(tag) = remote_tag {
            if has_update && tag != current_version {
                tag
            } else if has_update {
                format!("{} ({})", current_version, remote_commit)
            } else {
                current_version.clone()
            }
        } else if has_update {
            format!("{} ({})", current_version, remote_commit)
        } else {
            current_version.clone()
        };

        let message = if has_update {
            format!(
                "Nouvelle version disponible sur le canal '{}' ({} nouveau(x) commit(s)).",
                target_channel,
                commits.len()
            )
        } else {
            format!(
                "Votre système Noos HTPC est parfaitement à jour sur le canal '{}'.",
                target_channel
            )
        };

        Ok(UpdateInfo {
            has_update,
            current_version,
            latest_version,
            current_branch,
            target_channel,
            commits,
            message,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn apply_system_update(app: AppHandle, channel: String) -> Result<bool, String> {
    tokio::task::spawn(async move {
        let repo = find_repo_dir();
        let target_channel = if channel.is_empty() { "testing".to_string() } else { channel };

        let emit_step = |step: u8, name: &str, percent: u8, log: &str| {
            let progress = UpdateProgress {
                step,
                total_steps: 4,
                step_name: name.to_string(),
                percent,
                log_line: log.to_string(),
            };
            let _ = app.emit("update_progress", progress);
        };

        // Étape 1 : Synchronisation Git des sources
        emit_step(1, "Synchronisation des sources Git", 15, &format!("Récupération de la branche origin/{}...", target_channel));
        let s1 = Command::new("git")
            .current_dir(&repo)
            .args(["fetch", "origin", &target_channel, "--tags"])
            .status();
        if s1.is_err() || !s1.unwrap().success() {
            emit_step(1, "Erreur de synchronisation", 15, "Impossible de joindre le dépôt GitHub distant.");
            return;
        }

        let _ = Command::new("git").current_dir(&repo).args(["checkout", &target_channel]).status();
        let s_pull = Command::new("git").current_dir(&repo).args(["pull", "origin", &target_channel]).status();
        if s_pull.is_err() || !s_pull.unwrap().success() {
            let _ = Command::new("git").current_dir(&repo).args(["reset", "--hard", &format!("origin/{}", target_channel)]).status();
        }

        // Étape 2 : Actualisation du fichier flake.lock
        emit_step(2, "Mise à jour déclarative de flake.lock", 40, "Actualisation des entrées du verrou Flake...");
        let s2 = Command::new("nix")
            .current_dir(&repo)
            .args(["flake", "update"])
            .status();
        if s2.is_err() || !s2.unwrap().success() {
            tracing::warn!("Mise à jour flake.lock en avertissement, continuation...");
        }

        // Étape 3 : Reconstruction déclarative NixOS sans mot de passe
        emit_step(3, "Reconstruction du système NixOS", 70, "Exécution de nixos-rebuild switch (sudo NOPASSWD)...");
        let flake_target = format!("{}#htpc", repo.display());
        let s3 = Command::new("sudo")
            .args(["nixos-rebuild", "switch", "--flake", &flake_target])
            .status();

        if s3.is_err() || !s3.unwrap().success() {
            emit_step(3, "Erreur Reconstruction", 70, "Échec de nixos-rebuild switch. Rollback actif.");
            return;
        }

        // Étape 4 : Finalisation et relance avec animation
        emit_step(4, "Mise à jour terminée", 100, "Le système est à jour. Préparation de la relance...");
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        let _ = app.emit("update_completed", true);
    });

    Ok(true)
}

/* ========================================================================= */
/* GESTION DE LA DISPOSITION DU CLAVIER VIRTUEL                              */
/* ========================================================================= */

#[tauri::command]
pub async fn get_keyboard_config() -> Result<KeyboardConfig, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/noos".to_string());
    let conf_path = std::path::PathBuf::from(&home).join(".config/noos-htpc/keyboard.json");

    if let Ok(data) = std::fs::read_to_string(&conf_path) {
        if let Ok(cfg) = serde_json::from_str::<KeyboardConfig>(&data) {
            return Ok(cfg);
        }
    }

    Ok(KeyboardConfig {
        layout: "azerty".to_string(),
    })
}

#[tauri::command]
pub async fn set_keyboard_config(config: KeyboardConfig) -> Result<bool, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/noos".to_string());
    let conf_dir = std::path::PathBuf::from(&home).join(".config/noos-htpc");
    let _ = std::fs::create_dir_all(&conf_dir);
    let conf_path = conf_dir.join("keyboard.json");

    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = std::fs::write(&conf_path, json);
    }

    // Notifier le démon noos-osk via son socket UNIX
    tokio::task::spawn_blocking(move || {
        use std::io::Write;
        use std::os::unix::net::UnixStream;

        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let sock_path = format!("{}/noos-osk.sock", runtime_dir);

        if let Ok(mut stream) = UnixStream::connect(&sock_path) {
            let msg = format!("SET_LAYOUT {}", config.layout.to_lowercase());
            let _ = stream.write_all(msg.as_bytes());
        }
    });

    Ok(true)
}

#[tauri::command]
pub async fn show_virtual_keyboard() -> Result<bool, String> {
    tokio::task::spawn_blocking(|| {
        use std::io::Write;
        use std::os::unix::net::UnixStream;

        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let sock_path = format!("{}/noos-osk.sock", runtime_dir);

        if let Ok(mut stream) = UnixStream::connect(&sock_path) {
            let _ = stream.write_all(b"SHOW");
            Ok(true)
        } else {
            Ok(false)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn hide_virtual_keyboard() -> Result<bool, String> {
    tokio::task::spawn_blocking(|| {
        use std::io::Write;
        use std::os::unix::net::UnixStream;

        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let sock_path = format!("{}/noos-osk.sock", runtime_dir);

        if let Ok(mut stream) = UnixStream::connect(&sock_path) {
            let _ = stream.write_all(b"HIDE");
            Ok(true)
        } else {
            Ok(false)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn restart_dashboard(app: AppHandle) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let _ = Command::new("sudo").args(["systemctl", "restart", "greetd"]).status();
    });
    let _ = app.emit("dashboard_restarting", ());
    Ok(())
}

/* ========================================================================= */
/* GESTION DES SOURCES ET SORTIES AUDIO (PIPEWIRE / WIREPLUMBER)             */
/* ========================================================================= */

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioSink {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub volume_percent: u32,
    pub icon: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioVolumeInfo {
    pub volume_percent: u32,
    pub is_muted: bool,
}

pub fn parse_volume_info(text: &str) -> (u32, bool) {
    let is_muted = text.contains("[MUTED]");
    let clean = text.replace("Volume:", "").replace("[MUTED]", "");
    let vol_float = clean.trim().parse::<f32>().unwrap_or(0.5);
    let volume_percent = (vol_float * 100.0).round().min(100.0) as u32;
    (volume_percent, is_muted)
}

fn get_current_volume_internal() -> (u32, bool) {
    if let Ok(output) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            return parse_volume_info(&text);
        }
    }
    (80, false)
}

#[tauri::command]
pub async fn get_master_volume() -> Result<AudioVolumeInfo, String> {
    tokio::task::spawn_blocking(|| {
        let (volume_percent, is_muted) = get_current_volume_internal();
        Ok(AudioVolumeInfo {
            volume_percent,
            is_muted,
        })
    })
    .await
    .map_err(|e| format!("Erreur thread volume : {}", e))?
}

#[tauri::command]
pub async fn set_master_volume(volume_percent: u32) -> Result<AudioVolumeInfo, String> {
    tokio::task::spawn_blocking(move || {
        let clamped = volume_percent.min(100);
        let vol_float = clamped as f32 / 100.0;
        let vol_str = format!("{:.2}", vol_float);

        let _ = Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &vol_str])
            .status();

        let (volume_percent, is_muted) = get_current_volume_internal();
        Ok(AudioVolumeInfo {
            volume_percent,
            is_muted,
        })
    })
    .await
    .map_err(|e| format!("Erreur thread réglage volume : {}", e))?
}

#[tauri::command]
pub async fn toggle_master_mute() -> Result<AudioVolumeInfo, String> {
    tokio::task::spawn_blocking(|| {
        let _ = Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
            .status();

        let (volume_percent, is_muted) = get_current_volume_internal();
        Ok(AudioVolumeInfo {
            volume_percent,
            is_muted,
        })
    })
    .await
    .map_err(|e| format!("Erreur thread mute : {}", e))?
}

pub fn parse_pw_dump_sinks(text: &str, current_vol: u32) -> Option<Vec<AudioSink>> {
    let data: serde_json::Value = serde_json::from_str(text).ok()?;
    let arr = data.as_array()?;

    let mut sinks = Vec::new();
    let mut parsed_card_ids = std::collections::HashSet::new();

    // 1. Détecter les cartes audio ALSA et leurs sorties physiques (HDMI, Haut-parleurs, Casque)
    for item in arr {
        let info = match item.get("info") {
            Some(i) => i,
            None => continue,
        };
        let props = match info.get("props") {
            Some(p) => p,
            None => continue,
        };
        let media_class = props.get("media.class").and_then(|v| v.as_str()).unwrap_or("");

        if media_class == "Audio/Device" {
            let card_id = item.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let card_desc = props.get("device.description").and_then(|v| v.as_str()).unwrap_or("Audio interne");

            let params = match info.get("params") {
                Some(p) => p,
                None => continue,
            };

            // Profil actif de la carte
            let active_profile_idx = params.get("Profile")
                .and_then(|p| p.as_array())
                .and_then(|a| a.first())
                .and_then(|p0| p0.get("index"))
                .and_then(|v| v.as_u64());

            // Profils disponibles (non désactivés)
            let mut avail_profiles = Vec::new();
            if let Some(enum_prof) = params.get("EnumProfile").and_then(|p| p.as_array()) {
                for prof in enum_prof {
                    let idx = prof.get("index").and_then(|v| v.as_u64()).unwrap_or(0);
                    let name = prof.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let avail = prof.get("available").and_then(|v| v.as_str()).unwrap_or("");
                    if avail != "no" && idx != 0 {
                        avail_profiles.push((idx, name.to_string()));
                    }
                }
            }

            // Routes de sortie (Output)
            if let Some(enum_route) = params.get("EnumRoute").and_then(|r| r.as_array()) {
                let mut card_has_routes = false;
                for route in enum_route {
                    let direction = route.get("direction").and_then(|v| v.as_str()).unwrap_or("");
                    if direction != "Output" {
                        continue;
                    }
                    let available = route.get("available").and_then(|v| v.as_str()).unwrap_or("");
                    // Ignorer les ports explicitement non branchés
                    if available == "no" {
                        continue;
                    }

                    let route_name = route.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let route_desc = route.get("description").and_then(|v| v.as_str()).unwrap_or("");

                    // Extraction du nom de l'équipement (ex: LG TV SSCR2) et détection des codecs Dolby
                    let mut prod_name = None;
                    let mut has_dolby = false;
                    if let Some(info_arr) = route.get("info").and_then(|v| v.as_array()) {
                        let mut i = 0;
                        while i < info_arr.len() {
                            if let Some(key) = info_arr[i].as_str() {
                                if key == "device.product.name" && i + 1 < info_arr.len() {
                                    prod_name = info_arr[i + 1].as_str().map(|s| s.to_string());
                                } else if key == "iec958.codecs.detected" && i + 1 < info_arr.len() {
                                    if let Some(codecs_str) = info_arr[i + 1].as_str() {
                                        if codecs_str.contains("AC3") || codecs_str.contains("EAC3") || codecs_str.contains("TrueHD") {
                                            has_dolby = true;
                                        }
                                    }
                                }
                            }
                            i += 1;
                        }
                    }

                    let route_profiles: Vec<u64> = route.get("profiles")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|x| x.as_u64()).collect())
                        .unwrap_or_default();

                    let is_active = active_profile_idx.map(|idx| route_profiles.contains(&idx)).unwrap_or(false);

                    // Meilleur profil cible pour cette route
                    let target_prof = route_profiles.iter().find(|&&p_idx| {
                        avail_profiles.iter().any(|(idx, name)| *idx == p_idx && name.contains("+input:"))
                    }).or_else(|| {
                        route_profiles.iter().find(|&&p_idx| {
                            avail_profiles.iter().any(|(idx, _)| *idx == p_idx)
                        })
                    }).copied().unwrap_or_else(|| route_profiles.first().copied().unwrap_or(1));

                    let is_hdmi = route_name.contains("hdmi") || route_desc.contains("HDMI");
                    let is_headphone = route_name.contains("headphone") || route_desc.contains("Casque");
                    let is_speaker = route_name.contains("speaker") || route_desc.contains("Haut-parleur");

                    let icon = if is_hdmi {
                        "📺".to_string()
                    } else if is_headphone {
                        "🎧".to_string()
                    } else if route_name.contains("spdif") || route_name.contains("optical") {
                        "🔊".to_string()
                    } else {
                        "🔈".to_string()
                    };

                    let (name, sub) = if let Some(p) = prod_name {
                        let bitperfect_tag = if has_dolby { " • Audio Bitperfect Dolby / PCM" } else { "" };
                        (format!("{} (HDMI)", p), format!("Écran TV{} • Port {}", bitperfect_tag, route_name))
                    } else if is_speaker {
                        (format!("{} (Haut-parleurs)", card_desc), "Sortie audio analogique interne / Ligne".to_string())
                    } else if is_headphone {
                        (format!("{} (Casque)", card_desc), "Prise casque audio 3.5mm".to_string())
                    } else {
                        (format!("{} ({})", card_desc, route_desc), format!("Sortie {}", route_name))
                    };

                    sinks.push(AudioSink {
                        id: format!("profile:{}:{}", card_id, target_prof),
                        name,
                        is_default: is_active,
                        volume_percent: current_vol,
                        icon,
                        description: sub,
                    });
                    card_has_routes = true;
                }

                if card_has_routes {
                    parsed_card_ids.insert(card_id);
                }
            }
        }
    }

    // 2. Détecter les sinks directs indépendants (ex: USB DAC, écouteurs Bluetooth) non couverts ci-dessus
    for item in arr {
        let info = match item.get("info") {
            Some(i) => i,
            None => continue,
        };
        let props = match info.get("props") {
            Some(p) => p,
            None => continue,
        };
        let media_class = props.get("media.class").and_then(|v| v.as_str()).unwrap_or("");

        if media_class == "Audio/Sink" {
            let node_id = item.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let dev_id = props.get("device.id").and_then(|v| v.as_u64()).unwrap_or(0);

            // Si cette carte a déjà été traitée avec ses profils/routes, ignorer pour éviter les doublons
            if parsed_card_ids.contains(&dev_id) {
                continue;
            }

            let raw_desc = props.get("node.description")
                .or_else(|| props.get("node.nick"))
                .or_else(|| props.get("node.name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Périphérique audio");

            let lower = raw_desc.to_lowercase();
            let icon = if lower.contains("hdmi") || lower.contains("tv") {
                "📺".to_string()
            } else if lower.contains("casque") || lower.contains("headphone") {
                "🎧".to_string()
            } else if lower.contains("usb") || lower.contains("dac") {
                "🎛️".to_string()
            } else if lower.contains("bluetooth") || lower.contains("ble") {
                "📶".to_string()
            } else {
                "🔈".to_string()
            };

            sinks.push(AudioSink {
                id: format!("sink:{}", node_id),
                name: raw_desc.to_string(),
                is_default: false,
                volume_percent: current_vol,
                icon,
                description: format!("Périphérique audio #{}", node_id),
            });
        }
    }

    if sinks.is_empty() {
        None
    } else {
        Some(sinks)
    }
}

pub fn parse_audio_sinks(text: &str) -> Vec<AudioSink> {
    let mut sinks = Vec::new();
    let mut in_sinks = false;

    for line in text.lines() {
        if line.contains("Sinks:") {
            in_sinks = true;
            continue;
        }
        if in_sinks {
            if line.contains("Sources:") || line.contains("Filters:") || line.contains("Streams:") || line.contains("Video") || line.contains("Settings") {
                in_sinks = false;
                continue;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed == "│" {
                continue;
            }

            let is_default = line.contains('*');
            let parts: Vec<&str> = line.split('.').collect();
            if parts.len() >= 2 {
                let id_digits: String = parts[0].chars().filter(|c| c.is_ascii_digit()).collect();
                if !id_digits.is_empty() {
                    let rest = parts[1..].join(".");
                    let name_and_vol: Vec<&str> = rest.split('[').collect();
                    let raw_name = name_and_vol[0].trim().to_string();

                    let mut vol_pct = 100;
                    if name_and_vol.len() > 1 && name_and_vol[1].contains("vol:") {
                        let vol_str = name_and_vol[1].replace("vol:", "").replace(']', "").trim().to_string();
                        if let Ok(vol) = vol_str.parse::<f32>() {
                            vol_pct = (vol * 100.0).round() as u32;
                        }
                    }

                    let lower = raw_name.to_lowercase();
                    let icon = if lower.contains("hdmi") || lower.contains("displayport") || lower.contains("tv") {
                        "📺".to_string()
                    } else if lower.contains("casque") || lower.contains("headphone") || lower.contains("analog") || lower.contains("nova") {
                        "🎧".to_string()
                    } else if lower.contains("spdif") || lower.contains("optical") || lower.contains("digital") {
                        "🔊".to_string()
                    } else if lower.contains("usb") || lower.contains("dac") {
                        "🎛️".to_string()
                    } else if lower.contains("bluetooth") || lower.contains("ble") {
                        "📶".to_string()
                    } else {
                        "🔈".to_string()
                    };

                    sinks.push(AudioSink {
                        id: format!("sink:{}", id_digits),
                        name: raw_name.clone(),
                        is_default,
                        volume_percent: vol_pct,
                        icon,
                        description: format!("Sortie audio PipeWire #{}", id_digits),
                    });
                }
            }
        }
    }

    if sinks.is_empty() {
        sinks.push(AudioSink {
            id: "sink:1".to_string(),
            name: "Sortie Audio HDMI / TV (Par défaut)".to_string(),
            is_default: true,
            volume_percent: 100,
            icon: "📺".to_string(),
            description: "Sortie principale par défaut".to_string(),
        });
    }

    sinks
}

#[tauri::command]
pub async fn get_audio_sinks() -> Result<Vec<AudioSink>, String> {
    tokio::task::spawn_blocking(|| {
        let (current_vol, _) = get_current_volume_internal();

        // 1. Détection avancée multi-profils via pw-dump (détecte HDMI même inactif)
        if let Ok(dump_out) = Command::new("pw-dump").output() {
            if dump_out.status.success() {
                let json_text = String::from_utf8_lossy(&dump_out.stdout);
                if let Some(sinks) = parse_pw_dump_sinks(&json_text, current_vol) {
                    if !sinks.is_empty() {
                        return Ok(sinks);
                    }
                }
            }
        }

        // 2. Repli vers wpctl status
        let output = Command::new("wpctl")
            .arg("status")
            .output();

        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            Ok(parse_audio_sinks(&text))
        } else {
            Ok(parse_audio_sinks(""))
        }
    })
    .await
    .map_err(|e| format!("Erreur thread audio : {}", e))?
}

#[tauri::command]
pub async fn set_audio_sink(sink_id: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        if sink_id.starts_with("profile:") {
            // Format: "profile:<card_id>:<profile_index>"
            let parts: Vec<&str> = sink_id.split(':').collect();
            if parts.len() >= 3 {
                let card_id = parts[1];
                let profile_index = parts[2];
                let _ = Command::new("wpctl")
                    .args(["set-profile", card_id, profile_index])
                    .status();
                // Attente courte pour l'initialisation du nouveau sink par PipeWire / WirePlumber
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
            Ok(true)
        } else {
            // Format direct "sink:<id>" ou "<id>"
            let raw_id = sink_id.trim_start_matches("sink:").trim();
            let status = Command::new("wpctl")
                .args(["set-default", raw_id])
                .status();

            match status {
                Ok(s) if s.success() => Ok(true),
                _ => {
                    let _ = Command::new("pactl")
                        .args(["set-default-sink", raw_id])
                        .status();
                    Ok(true)
                }
            }
        }
    })
    .await
    .map_err(|e| format!("Erreur thread changement sink : {}", e))?
}

/* ========================================================================= */
/* GESTION DES PROFILS D'UPSCALE MPV (AMD, NVIDIA, INTEL)                     */
/* ========================================================================= */

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpscaleProfile {
    pub id: String,
    pub name: String,
    pub brand: String, // "amd", "nvidia", "intel"
    pub level: String, // "simple", "moyen", "eleve"
    pub level_label: String,
    pub tech_tag: String,
    pub description: String,
    pub scale_method: String,
    pub shaders: Vec<String>,
    pub max_res_target: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpscaleSystemInfo {
    pub detected_brand: String,
    pub active_brand: String,
    pub detected_max_res: String,
    pub max_width: u32,
    pub max_height: u32,
    pub current_profile_id: String,
    pub profiles: Vec<UpscaleProfile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpscaleConfig {
    pub active_brand: String,
    pub active_profile_id: String,
}

pub fn detect_gpu_brand() -> String {
    // 1. DRM sysfs vendor ID check
    // 0x1002 = AMD, 0x10de = NVIDIA, 0x8086 = Intel
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path().join("device/vendor");
            if let Ok(vendor_str) = std::fs::read_to_string(path) {
                let v = vendor_str.trim().to_lowercase();
                if v == "0x1002" {
                    return "amd".to_string();
                } else if v == "0x10de" {
                    return "nvidia".to_string();
                } else if v == "0x8086" {
                    return "intel".to_string();
                }
            }
        }
    }

    // 2. Fallback via lspci
    if let Ok(output) = Command::new("lspci").output() {
        if let Ok(text) = String::from_utf8(output.stdout) {
            for line in text.lines() {
                let lower = line.to_lowercase();
                if lower.contains("vga") || lower.contains("3d controller") || lower.contains("display controller") {
                    if lower.contains("amd") || lower.contains("radeon") || lower.contains("advanced micro devices") {
                        return "amd".to_string();
                    }
                    if lower.contains("nvidia") || lower.contains("geforce") {
                        return "nvidia".to_string();
                    }
                    if lower.contains("intel") || lower.contains("arc") || lower.contains("iris") {
                        return "intel".to_string();
                    }
                }
            }
        }
    }

    "amd".to_string()
}

pub fn detect_max_resolution() -> (u32, u32, String) {
    let mut max_w = 0u32;
    let mut max_h = 0u32;

    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let modes_path = entry.path().join("modes");
            if let Ok(content) = std::fs::read_to_string(modes_path) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.trim().split('x').collect();
                    if parts.len() == 2 {
                        if let (Ok(w), Ok(h)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                            if (w as u64) * (h as u64) > (max_w as u64) * (max_h as u64) {
                                max_w = w;
                                max_h = h;
                            }
                        }
                    }
                }
            }
        }
    }

    if max_w == 0 || max_h == 0 {
        max_w = 1920;
        max_h = 1080;
    }

    let label = match (max_w, max_h) {
        (w, h) if w >= 3840 || h >= 2160 => format!("{}×{} (4K UHD)", w, h),
        (w, h) if w >= 2560 || h >= 1440 => format!("{}×{} (2K QHD)", w, h),
        (w, h) if w >= 1920 || h >= 1080 => format!("{}×{} (1080p FHD)", w, h),
        (w, h) => format!("{}×{}", w, h),
    };

    (max_w, max_h, label)
}

pub fn get_all_upscale_profiles(max_res_label: &str) -> Vec<UpscaleProfile> {
    vec![
        // AMD
        UpscaleProfile {
            id: "amd_simple".to_string(),
            name: "FidelityFX CAS".to_string(),
            brand: "amd".to_string(),
            level: "simple".to_string(),
            level_label: "Simple".to_string(),
            tech_tag: "FidelityFX CAS".to_string(),
            description: "Accentuation adaptative des contrastes AMD FidelityFX CAS. Traitement ultraléger garantissant une image nette sans surconsommation GPU.".to_string(),
            scale_method: "spline36".to_string(),
            shaders: vec!["CAS-scaled.glsl".to_string()],
            max_res_target: max_res_label.to_string(),
        },
        UpscaleProfile {
            id: "amd_moyen".to_string(),
            name: "AMD FSR Équilibré".to_string(),
            brand: "amd".to_string(),
            level: "moyen".to_string(),
            level_label: "Moyen".to_string(),
            tech_tag: "FSR (EASU + RCAS)".to_string(),
            description: "Super-résolution spatiale AMD FidelityFX Super Resolution (FSR). Combine reconstruction des contours et filtrage de netteté haute fidélité.".to_string(),
            scale_method: "ewa_lanczossharp".to_string(),
            shaders: vec!["FSR.glsl".to_string()],
            max_res_target: max_res_label.to_string(),
        },
        UpscaleProfile {
            id: "amd_eleve".to_string(),
            name: "AMD FSR Ultra Neuronal".to_string(),
            brand: "amd".to_string(),
            level: "eleve".to_string(),
            level_label: "Élevé".to_string(),
            tech_tag: "FSRCNNX 16 + FSR Ultra".to_string(),
            description: "Réseau de neurones convolutifs FSRCNNX 16 passes couplé au shader FSR. Clarté cinématographique maximale poussant le GPU à son plein potentiel.".to_string(),
            scale_method: "ewa_lanczossharp".to_string(),
            shaders: vec!["FSRCNNX_x2_16-0-4-1.glsl".to_string(), "FSR.glsl".to_string()],
            max_res_target: max_res_label.to_string(),
        },

        // NVIDIA
        UpscaleProfile {
            id: "nvidia_simple".to_string(),
            name: "NVIDIA Image Scaling (NIS)".to_string(),
            brand: "nvidia".to_string(),
            level: "simple".to_string(),
            level_label: "Simple".to_string(),
            tech_tag: "NVIDIA NIS".to_string(),
            description: "Filtre directionnel 6-taps avec netteté adaptative NVIDIA Image Scaling. Léger et fluide, conçu pour préserver le framerate.".to_string(),
            scale_method: "spline36".to_string(),
            shaders: vec!["NVScaler.glsl".to_string()],
            max_res_target: max_res_label.to_string(),
        },
        UpscaleProfile {
            id: "nvidia_moyen".to_string(),
            name: "RTX VSR / DLSS Équivalent".to_string(),
            brand: "nvidia".to_string(),
            level: "moyen".to_string(),
            level_label: "Moyen".to_string(),
            tech_tag: "NNEDI3 64 Neurones".to_string(),
            description: "Super-échantillonnage neuronal NNEDI3 64 neurones simulant le rendu DLSS / VSR. Reconstitution fidèle des textures et suppression des artefacts.".to_string(),
            scale_method: "ewa_lanczossharp".to_string(),
            shaders: vec!["nnedi3-nns64-win8x6.hook".to_string()],
            max_res_target: max_res_label.to_string(),
        },
        UpscaleProfile {
            id: "nvidia_eleve".to_string(),
            name: "RTX VSR Ultra Cinéma".to_string(),
            brand: "nvidia".to_string(),
            level: "eleve".to_string(),
            level_label: "Élevé".to_string(),
            tech_tag: "NNEDI3 128 + KrigBilateral".to_string(),
            description: "Architecture neuronale lourde NNEDI3 128 neurones combinée au reconstructeur de chrominance KrigBilateral et deband pour une précision 4K absolue.".to_string(),
            scale_method: "ewa_lanczossharp".to_string(),
            shaders: vec!["nnedi3-nns128-win8x6.hook".to_string(), "KrigBilateral.glsl".to_string()],
            max_res_target: max_res_label.to_string(),
        },

        // Intel
        UpscaleProfile {
            id: "intel_simple".to_string(),
            name: "Intel Adaptive CAS".to_string(),
            brand: "intel".to_string(),
            level: "simple".to_string(),
            level_label: "Simple".to_string(),
            tech_tag: "Intel CAS".to_string(),
            description: "Filtre d'accentuation adaptatif basse consommation pour GPU Intel Iris Xe et Arc. Réhausse les textures sans artefact de sur-accentuation.".to_string(),
            scale_method: "spline36".to_string(),
            shaders: vec!["CAS-scaled.glsl".to_string()],
            max_res_target: max_res_label.to_string(),
        },
        UpscaleProfile {
            id: "intel_moyen".to_string(),
            name: "Intel XeSS Équilibré".to_string(),
            brand: "intel".to_string(),
            level: "moyen".to_string(),
            level_label: "Moyen".to_string(),
            tech_tag: "XeSS Neural 8-Couches".to_string(),
            description: "Super-échantillonnage haute résolution assisté par réseau neuronal FSRCNNX 8 couches. Compromis optimal entre fluidité et piqué d'image.".to_string(),
            scale_method: "ewa_lanczossharp".to_string(),
            shaders: vec!["FSRCNNX_x2_8-0-4-1.glsl".to_string()],
            max_res_target: max_res_label.to_string(),
        },
        UpscaleProfile {
            id: "intel_eleve".to_string(),
            name: "Intel XeSS Ultra Fidélité".to_string(),
            brand: "intel".to_string(),
            level: "eleve".to_string(),
            level_label: "Élevé".to_string(),
            tech_tag: "XeSS Ultra 16 + SSim".to_string(),
            description: "Traitement neuronal intensif 16 couches FSRCNNX associé au rééchantillonnage perceptuel SSim. Clarté maximale sur écrans haute densité.".to_string(),
            scale_method: "ewa_lanczossharp".to_string(),
            shaders: vec!["FSRCNNX_x2_16-0-4-1.glsl".to_string(), "SSimDownscaler.glsl".to_string()],
            max_res_target: max_res_label.to_string(),
        },
    ]
}

pub fn apply_mpv_profile(profile: &UpscaleProfile, max_res_label: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/noos".to_string());
    
    let mut config_text = format!(
        "# === NOOS UPSCALE PROFILE : {} ({}) ===\n\
         # Marque GPU : {} | Résolution Écran Max : {}\n\
         profile=gpu-hq\n\
         scale={}\n\
         cscale=ewa_lanczossharp\n\
         dscale=mitchell\n\
         correct-downscaling=yes\n\
         linear-downscaling=yes\n\
         glsl-shaders-clr\n",
        profile.name,
        profile.tech_tag,
        profile.brand.to_uppercase(),
        max_res_label,
        profile.scale_method
    );

    for s in &profile.shaders {
        config_text.push_str(&format!("glsl-shader=\"/etc/mpv/shaders/{}\"\n", s));
    }

    if profile.level == "eleve" {
        config_text.push_str("deband=yes\ndeband-iterations=4\ndeband-threshold=48\ndeband-range=16\ndeband-grain=48\n");
    }

    config_text.push_str("# === END NOOS UPSCALE PROFILE ===\n");

    // 1. Déploiement dans ~/.config/mpv/mpv.conf
    let mpv_dir = std::path::PathBuf::from(&home).join(".config/mpv");
    let _ = std::fs::create_dir_all(&mpv_dir);
    let mpv_conf = mpv_dir.join("mpv.conf");

    let current_mpv = std::fs::read_to_string(&mpv_conf).unwrap_or_default();
    let cleaned_mpv = clean_upscale_section(&current_mpv);
    let new_mpv = format!("{}\n{}", cleaned_mpv.trim(), config_text);
    let _ = std::fs::write(&mpv_conf, new_mpv.trim_start());

    // 2. Déploiement dans ~/.config/jellyfin-media-player/mpv.conf
    let jmp_dir = std::path::PathBuf::from(&home).join(".config/jellyfin-media-player");
    let _ = std::fs::create_dir_all(&jmp_dir);
    let jmp_conf = jmp_dir.join("mpv.conf");

    let current_jmp = std::fs::read_to_string(&jmp_conf).unwrap_or_default();
    let cleaned_jmp = clean_upscale_section(&current_jmp);
    let new_jmp = format!("{}\n{}", cleaned_jmp.trim(), config_text);
    let _ = std::fs::write(&jmp_conf, new_jmp.trim_start());
}

fn clean_upscale_section(content: &str) -> String {
    let start_tag = "# === NOOS UPSCALE PROFILE";
    let end_tag = "# === END NOOS UPSCALE PROFILE ===";

    if let (Some(start_idx), Some(end_idx)) = (content.find(start_tag), content.find(end_tag)) {
        let after_end = end_idx + end_tag.len();
        format!("{}{}", &content[..start_idx], &content[after_end..])
    } else {
        content.to_string()
    }
}

#[tauri::command]
pub async fn get_upscale_info(brand: Option<String>) -> Result<UpscaleSystemInfo, String> {
    let detected_brand = detect_gpu_brand();
    let (max_w, max_h, max_res_label) = detect_max_resolution();

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/noos".to_string());
    let upscale_file = std::path::PathBuf::from(&home).join(".config/noos-htpc/upscale.json");

    let saved_config: Option<UpscaleConfig> = if let Ok(data) = std::fs::read_to_string(&upscale_file) {
        serde_json::from_str(&data).ok()
    } else {
        None
    };

    let active_brand = brand.unwrap_or_else(|| {
        if let Some(ref cfg) = saved_config {
            cfg.active_brand.clone()
        } else {
            detected_brand.clone()
        }
    });

    let current_profile_id = if let Some(ref cfg) = saved_config {
        if cfg.active_brand == active_brand {
            cfg.active_profile_id.clone()
        } else {
            format!("{}_moyen", active_brand)
        }
    } else {
        format!("{}_moyen", active_brand)
    };

    let all_profiles = get_all_upscale_profiles(&max_res_label);
    let profiles: Vec<UpscaleProfile> = all_profiles
        .into_iter()
        .filter(|p| p.brand == active_brand)
        .collect();

    Ok(UpscaleSystemInfo {
        detected_brand,
        active_brand,
        detected_max_res: max_res_label,
        max_width: max_w,
        max_height: max_h,
        current_profile_id,
        profiles,
    })
}

#[tauri::command]
pub async fn set_upscale_profile(profile_id: String) -> Result<bool, String> {
    let (_, _, max_res_label) = detect_max_resolution();
    let all_profiles = get_all_upscale_profiles(&max_res_label);

    let profile = all_profiles
        .into_iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| format!("Profil d'upscaling '{}' inconnu.", profile_id))?;

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/noos".to_string());
    let conf_dir = std::path::PathBuf::from(&home).join(".config/noos-htpc");
    let _ = std::fs::create_dir_all(&conf_dir);
    let upscale_file = conf_dir.join("upscale.json");

    let cfg = UpscaleConfig {
        active_brand: profile.brand.clone(),
        active_profile_id: profile.id.clone(),
    };

    if let Ok(json) = serde_json::to_string_pretty(&cfg) {
        let _ = std::fs::write(&upscale_file, json);
    }

    apply_mpv_profile(&profile, &max_res_label);

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_audio_sinks() {
        let sample = r#"
PipeWire 'pipewire-0' [1.6.8, chomiam@pop-os]
Audio
 ├─ Devices:
 │      91. Family 17h/19h HD Audio Controller  [alsa]
 │      92. Arctis Nova 7X                      [alsa]
 │      94. HDA ATI HDMI                        [alsa]
 │  
 ├─ Sinks:
 │      46. HDA ATI HDMI Digital Stereo (HDMI 3) [vol: 0.40]
 │  *   59. Arctis Nova 7X Stéréo analogique  [vol: 0.75]
 │      88. Family 17h/19h HD Audio Controller Stéréo analogique [vol: 0.40]
 │  
 ├─ Sources:
 │  *   54. Arctis Nova 7X Mono                 [vol: 0.73]
 │     104. Family 17h/19h HD Audio Controller Stéréo analogique [vol: 1.00]
"#;

        let sinks = parse_audio_sinks(sample);
        assert_eq!(sinks.len(), 3);

        assert_eq!(sinks[0].id, "sink:46");
        assert_eq!(sinks[0].name, "HDA ATI HDMI Digital Stereo (HDMI 3)");
        assert!(!sinks[0].is_default);
        assert_eq!(sinks[0].volume_percent, 40);
        assert_eq!(sinks[0].icon, "📺");

        assert_eq!(sinks[1].id, "sink:59");
        assert_eq!(sinks[1].name, "Arctis Nova 7X Stéréo analogique");
        assert!(sinks[1].is_default);
        assert_eq!(sinks[1].volume_percent, 75);
        assert_eq!(sinks[1].icon, "🎧");

        assert_eq!(sinks[2].id, "sink:88");
        assert_eq!(sinks[2].name, "Family 17h/19h HD Audio Controller Stéréo analogique");
        assert!(!sinks[2].is_default);
        assert_eq!(sinks[2].volume_percent, 40);
    }

    #[test]
    fn test_parse_volume_info() {
        let (vol1, muted1) = parse_volume_info("Volume: 0.40");
        assert_eq!(vol1, 40);
        assert!(!muted1);

        let (vol2, muted2) = parse_volume_info("Volume: 0.85 [MUTED]");
        assert_eq!(vol2, 85);
        assert!(muted2);
    }

    #[test]
    fn test_parse_pw_dump_sinks() {
        let sample_json = r#"[
            {
                "id": 45,
                "type": "PipeWire:Interface:Device",
                "info": {
                    "props": {
                        "media.class": "Audio/Device",
                        "device.description": "Audio interne"
                    },
                    "params": {
                        "Profile": [
                            { "index": 1, "name": "output:analog-stereo+input:analog-stereo" }
                        ],
                        "EnumProfile": [
                            { "index": 1, "name": "output:analog-stereo+input:analog-stereo", "available": "yes" },
                            { "index": 3, "name": "output:hdmi-stereo+input:analog-stereo", "available": "yes" }
                        ],
                        "EnumRoute": [
                            {
                                "direction": "Output",
                                "name": "analog-output-speaker",
                                "description": "Haut-parleurs",
                                "available": "unknown",
                                "profiles": [ 1 ],
                                "info": []
                            },
                            {
                                "direction": "Output",
                                "name": "hdmi-output-0",
                                "description": "HDMI / DisplayPort",
                                "available": "yes",
                                "profiles": [ 3 ],
                                "info": [
                                    2,
                                    "device.product.name", "LG TV SSCR2",
                                    "iec958.codecs.detected", "[\"PCM\",\"AC3\",\"EAC3\",\"TrueHD\"]"
                                ]
                            }
                        ]
                    }
                }
            }
        ]"#;

        let sinks = parse_pw_dump_sinks(sample_json, 75).expect("Parsing pw-dump valide");
        assert_eq!(sinks.len(), 2);

        // Sortie Haut-parleurs
        assert_eq!(sinks[0].name, "Audio interne (Haut-parleurs)");
        assert_eq!(sinks[0].id, "profile:45:1");
        assert!(sinks[0].is_default);
        assert_eq!(sinks[0].icon, "🔈");

        // Sortie LG TV SSCR2 HDMI
        assert_eq!(sinks[1].name, "LG TV SSCR2 (HDMI)");
        assert_eq!(sinks[1].id, "profile:45:3");
        assert!(!sinks[1].is_default);
        assert_eq!(sinks[1].icon, "📺");
        assert!(sinks[1].description.contains("Dolby / PCM"));
    }
}



