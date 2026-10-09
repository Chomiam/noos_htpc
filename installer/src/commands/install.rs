use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

static CURRENT_PROGRESS: Mutex<Option<InstallProgress>> = Mutex::new(None);

#[tauri::command]
pub async fn get_install_progress() -> Result<InstallProgress, String> {
    let guard = CURRENT_PROGRESS.lock().map_err(|e| e.to_string())?;
    guard.clone().ok_or_else(|| "Aucune installation en cours".to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallRequest {
    pub target_disk: String,
    pub gpu_profile: String,
    pub enable_hdr: bool,
    pub hostname: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallProgress {
    pub step: u8,
    pub total_steps: u8,
    pub step_name: String,
    pub percent: u8,
    pub log_line: String,
    pub packages_done: u32,
    pub packages_total: u32,
}

fn find_config_source() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("/etc/noos-htpc-source"),
        PathBuf::from("/home/chomiam/Projets/noos_htpc"),
        std::env::current_dir().unwrap_or_default(),
    ];
    for candidate in candidates {
        if candidate.join("flake.nix").exists() {
            return Some(candidate);
        }
    }
    None
}

fn parse_these_count(line: &str) -> Option<u32> {
    let lower = line.trim().to_lowercase();
    if lower.starts_with("these ") {
        let parts: Vec<&str> = lower.split_whitespace().collect();
        if parts.len() >= 3 {
            if let Ok(count) = parts[1].parse::<u32>() {
                if parts[2].contains("path") || parts[2].contains("derivation") {
                    return Some(count);
                }
            }
        }
    }
    None
}

fn is_package_progress_line(line: &str) -> bool {
    let lower = line.trim().to_lowercase();
    lower.starts_with("copying path '") ||
    lower.starts_with("fetching path '") ||
    lower.starts_with("building '") ||
    lower.starts_with("unpacking '") ||
    lower.contains("copying /nix/store/") ||
    lower.contains("fetching /nix/store/") ||
    lower.contains("building /nix/store/")
}

#[tauri::command]
pub async fn start_installation(app: AppHandle, req: InstallRequest) -> Result<bool, String> {
    tokio::task::spawn(async move {
        let emit_step = |step: u8, name: &str, percent: u8, log: &str, done: u32, total: u32| {
            let progress = InstallProgress {
                step,
                total_steps: 6,
                step_name: name.to_string(),
                percent,
                log_line: log.to_string(),
                packages_done: done,
                packages_total: total,
            };

            // 1. Mise à jour de l'état partagé (accessible via get_install_progress)
            if let Ok(mut guard) = CURRENT_PROGRESS.lock() {
                *guard = Some(progress.clone());
            }

            // 2. Émission d'événement Tauri
            let _ = app.emit("install_progress", progress.clone());

            // 3. Injection directe JavaScript dans la fenêtre WebKit (100% fiable)
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(payload_json) = serde_json::to_string(&progress) {
                    let script = format!("if (window.onInstallProgress) {{ window.onInstallProgress({}); }}", payload_json);
                    let _ = window.eval(&script);
                }
            }
        };

        // Étape 1 : Validation et préparation
        emit_step(1, "Préparation du stockage cible", 10, &format!("Cible sélectionnée : {}", req.target_disk), 0, 0);

        // Désactivation des swaps et démontage des montages résiduels éventuels
        let _ = Command::new("swapoff").args(["-a"]).status();
        let _ = Command::new("umount").args(["-R", "/mnt"]).status();
        let _ = Command::new("udevadm").args(["settle", "--timeout=5"]).status();

        let is_nvme = req.target_disk.contains("nvme") || req.target_disk.contains("mmcblk");
        let boot_part = if is_nvme { format!("{}p1", req.target_disk) } else { format!("{}1", req.target_disk) };
        let swap_part = if is_nvme { format!("{}p2", req.target_disk) } else { format!("{}2", req.target_disk) };
        let root_part = if is_nvme { format!("{}p3", req.target_disk) } else { format!("{}3", req.target_disk) };

        // Étape 2 : Partitionnement GPT / UEFI (EFI 1Go + SWAP 8Go + Root reste)
        emit_step(2, "Partitionnement GPT", 20, "Création des partitions EFI (1Go), SWAP (8Go) et Système (ext4)...", 0, 0);
        let _ = Command::new("wipefs").args(["-a", &req.target_disk]).status();

        let p1 = Command::new("parted")
            .args(["-s", &req.target_disk, "mklabel", "gpt"])
            .status();
        if p1.is_err() || !p1.unwrap().success() {
            emit_step(2, "Erreur Partitionnement", 20, "Impossible d'initialiser la table de partition GPT", 0, 0);
            return;
        }

        // Partition 1 : EFI 1 Go (1MiB à 1025MiB)
        let p2 = Command::new("parted")
            .args(["-s", &req.target_disk, "mkpart", "boot", "fat32", "1MiB", "1025MiB", "set", "1", "esp", "on"])
            .status();
        if p2.is_err() || !p2.unwrap().success() {
            emit_step(2, "Erreur Partitionnement", 20, "Échec de création de la partition EFI (1Go)", 0, 0);
            return;
        }

        // Partition 2 : SWAP 8 Go (1025MiB à 9217MiB)
        let p3 = Command::new("parted")
            .args(["-s", &req.target_disk, "mkpart", "swap", "linux-swap", "1025MiB", "9217MiB"])
            .status();
        if p3.is_err() || !p3.unwrap().success() {
            emit_step(2, "Erreur Partitionnement", 20, "Échec de création de la partition SWAP (8Go)", 0, 0);
            return;
        }

        // Partition 3 : Système NixOS (ext4, 9217MiB au reste du disque)
        let p4 = Command::new("parted")
            .args(["-s", &req.target_disk, "mkpart", "nixos", "ext4", "9217MiB", "100%"])
            .status();
        if p4.is_err() || !p4.unwrap().success() {
            emit_step(2, "Erreur Partitionnement", 20, "Échec de création de la partition Système", 0, 0);
            return;
        }

        let _ = Command::new("partprobe").arg(&req.target_disk).status();
        let _ = Command::new("udevadm").args(["settle", "--timeout=10"]).status();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Étape 3 : Formatage des systèmes de fichiers
        emit_step(3, "Formatage des partitions", 35, &format!("Formatage de {} (boot 1G), {} (swap 8G) et {} (root)...", boot_part, swap_part, root_part), 0, 0);
        let f1 = Command::new("mkfs.vfat").args(["-F", "32", "-n", "boot", &boot_part]).status();
        if f1.is_err() || !f1.unwrap().success() {
            emit_step(3, "Erreur Formatage", 35, "Échec du formatage FAT32 de la partition EFI (1Go)", 0, 0);
            return;
        }

        let f2 = Command::new("mkswap").args(["-L", "swap", &swap_part]).status();
        if f2.is_err() || !f2.unwrap().success() {
            emit_step(3, "Erreur Formatage", 35, "Échec de l'initialisation de la partition SWAP (8Go)", 0, 0);
            return;
        }

        let s1 = Command::new("swapon").arg(&swap_part).status();
        if s1.is_err() || !s1.unwrap().success() {
            emit_step(3, "Avertissement SWAP", 35, "Activation du SWAP différée", 0, 0);
        }

        let f3 = Command::new("mkfs.ext4").args(["-F", "-L", "nixos", &root_part]).status();
        if f3.is_err() || !f3.unwrap().success() {
            emit_step(3, "Erreur Formatage", 35, "Échec du formatage ext4 de la partition racine", 0, 0);
            return;
        }

        let _ = Command::new("udevadm").args(["settle", "--timeout=10"]).status();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Étape 4 : Montage sur /mnt
        emit_step(4, "Montage des partitions", 45, "Montage du système de fichiers sur /mnt...", 0, 0);
        let _ = Command::new("mkdir").args(["-p", "/mnt"]).status();
        let m1 = Command::new("mount").args(["-t", "ext4", &root_part, "/mnt"]).status();
        if m1.is_err() || !m1.unwrap().success() {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let m1_retry = Command::new("mount").args(["-t", "ext4", &root_part, "/mnt"]).status();
            if m1_retry.is_err() || !m1_retry.unwrap().success() {
                emit_step(4, "Erreur Montage", 45, "Impossible de monter la partition racine sur /mnt", 0, 0);
                return;
            }
        }

        let _ = Command::new("mkdir").args(["-p", "/mnt/boot"]).status();
        let m2 = Command::new("mount").args(["-t", "vfat", &boot_part, "/mnt/boot"]).status();
        if m2.is_err() || !m2.unwrap().success() {
            emit_step(4, "Erreur Montage", 45, "Impossible de monter la partition EFI sur /mnt/boot", 0, 0);
            return;
        }

        // Étape 5 : Déploiement de la configuration déclarative Noos HTPC
        emit_step(5, "Déploiement de Noos HTPC", 55, "Copie des fichiers de configuration...", 0, 0);
        let _ = Command::new("mkdir").args(["-p", "/mnt/etc/nixos"]).status();

        let config_src = match find_config_source() {
            Some(path) => path,
            None => {
                emit_step(5, "Erreur Source", 55, "Impossible de localiser les fichiers Noos HTPC (/etc/noos-htpc-source)", 0, 0);
                return;
            }
        };

        emit_step(5, "Déploiement de Noos HTPC", 58, &format!("Source détectée : {}", config_src.display()), 0, 0);

        let cp_res = Command::new("cp")
            .args(["-rT", config_src.to_str().unwrap(), "/mnt/etc/nixos"])
            .status();
        if cp_res.is_err() || !cp_res.unwrap().success() {
            emit_step(5, "Erreur Copie", 58, "Échec de la copie des fichiers de configuration", 0, 0);
            return;
        }

        // Rendre les fichiers modifiables (sortie du nix store readonly)
        let _ = Command::new("chmod").args(["-R", "u+w", "/mnt/etc/nixos"]).status();
        // Nettoyage des fichiers temporaires ou .git pour éviter les blocages de flake
        let _ = Command::new("rm").args(["-rf", "/mnt/etc/nixos/.git", "/mnt/etc/nixos/installer/target"]).status();

        // Génération automatique du hardware-configuration spécifique au matériel
        emit_step(5, "Configuration Matérielle", 62, "Détection du matériel cible via nixos-generate-config...", 0, 0);
        let _ = Command::new("mkdir").args(["-p", "/tmp/noos-hw"]).status();
        let _ = Command::new("nixos-generate-config").args(["--root", "/mnt", "--dir", "/tmp/noos-hw"]).status();
        let hw_file = Path::new("/tmp/noos-hw/hardware-configuration.nix");
        if hw_file.exists() {
            let _ = Command::new("cp")
                .args(["-f", "/tmp/noos-hw/hardware-configuration.nix", "/mnt/etc/nixos/hosts/htpc/hardware-configuration.nix"])
                .status();
        }

        // Injection déclarative du profil GPU, du hostname forcé "noos-htpc" et des fonctionnalités flakes
        let host_override = format!(
            "{{ lib, ... }}: {{\n  networking.hostName = lib.mkForce \"noos-htpc\";\n  hardware.noos-htpc.gpu.profile = lib.mkForce \"{}\";\n  hardware.noos-htpc.gpu.enableHDR = lib.mkForce {};\n  nix.settings.experimental-features = [ \"nix-command\" \"flakes\" ];\n}}\n",
            req.gpu_profile, req.enable_hdr
        );
        let _ = std::fs::write("/mnt/etc/nixos/hosts/htpc/host-settings.local.nix", host_override);

        // Étape 6 : Exécution de nixos-install avec streaming en direct et évaluation des paquets
        let mut packages_total: u32 = 0;
        let mut packages_done: u32 = 0;

        emit_step(6, "Installation du système NixOS", 70, "Lancement de nixos-install (téléchargement et compilation)...", 0, 0);

        let mut child = match Command::new("nixos-install")
            .args(["--flake", "/mnt/etc/nixos#htpc", "--no-root-passwd"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                emit_step(6, "Erreur Lancement", 70, &format!("Impossible d'exécuter nixos-install: {}", e), 0, 0);
                return;
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        if let Some(stdout) = child.stdout.take() {
            let tx_out = tx.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx_out.send(line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let tx_err = tx.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx_err.send(line);
                }
            });
        }

        drop(tx);

        let mut last_error_line = String::new();
        while let Some(line) = rx.recv().await {
            if line.contains("error:") || line.contains("FAILED") {
                last_error_line = line.clone();
            }

            // Détection du total de paquets / dérivations
            if let Some(count) = parse_these_count(&line) {
                packages_total += count;
            }

            // Détection d'un paquet terminé / construit / téléchargé
            if is_package_progress_line(&line) {
                packages_done += 1;
                if packages_total > 0 && packages_done > packages_total {
                    packages_total = packages_done;
                }
            }

            // Calcul dynamique du pourcentage (plage 70% à 99%)
            let current_percent = if packages_total > 0 {
                let ratio = (packages_done as f32) / (packages_total as f32);
                let p = 70.0 + (ratio * 29.0);
                p.min(99.0) as u8
            } else {
                72
            };

            emit_step(6, "Installation de Noos HTPC en cours...", current_percent, &line, packages_done, packages_total);
        }

        let status = child.wait();
        match status {
            Ok(s) if s.success() => {
                let final_done = if packages_total > 0 { packages_total } else { packages_done };
                emit_step(6, "Installation Terminée avec Succès !", 100, "Noos HTPC a été compilé et installé avec succès.", final_done, packages_total);
            }
            _ => {
                let msg = if !last_error_line.is_empty() {
                    format!("Échec de nixos-install : {}", last_error_line)
                } else {
                    "Une erreur est survenue lors de l'installation NixOS.".to_string()
                };
                emit_step(6, "Erreur durant l'installation", 85, &msg, packages_done, packages_total);
            }
        }
    });

    Ok(true)
}

#[tauri::command]
pub async fn reboot_system() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        let _ = Command::new("systemctl").arg("reboot").status();
    });
    Ok(())
}
