use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

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

#[tauri::command]
pub async fn start_installation(app: AppHandle, req: InstallRequest) -> Result<bool, String> {
    tokio::task::spawn(async move {
        let emit_step = |step: u8, name: &str, percent: u8, log: &str| {
            let _ = app.emit("install_progress", InstallProgress {
                step,
                total_steps: 6,
                step_name: name.to_string(),
                percent,
                log_line: log.to_string(),
            });
        };

        // Étape 1 : Validation et préparation
        emit_step(1, "Préparation du stockage cible", 10, &format!("Cible sélectionnée : {}", req.target_disk));

        // Démonter les montages résiduels éventuels
        let _ = Command::new("umount").args(["-R", "/mnt"]).status();
        let _ = Command::new("udevadm").args(["settle", "--timeout=5"]).status();

        let is_nvme = req.target_disk.contains("nvme") || req.target_disk.contains("mmcblk");
        let boot_part = if is_nvme { format!("{}p1", req.target_disk) } else { format!("{}1", req.target_disk) };
        let root_part = if is_nvme { format!("{}p2", req.target_disk) } else { format!("{}2", req.target_disk) };

        // Étape 2 : Partitionnement GPT / UEFI
        emit_step(2, "Partitionnement GPT", 20, "Création des partitions EFI (512Mo) et Système (ext4)...");
        let _ = Command::new("wipefs").args(["-a", &req.target_disk]).status();

        let p1 = Command::new("parted")
            .args(["-s", &req.target_disk, "mklabel", "gpt"])
            .status();
        if p1.is_err() || !p1.unwrap().success() {
            emit_step(2, "Erreur Partitionnement", 20, "Impossible d'initialiser la table de partition GPT");
            return;
        }

        let p2 = Command::new("parted")
            .args(["-s", &req.target_disk, "mkpart", "boot", "fat32", "1MiB", "512MiB", "set", "1", "esp", "on"])
            .status();
        if p2.is_err() || !p2.unwrap().success() {
            emit_step(2, "Erreur Partitionnement", 20, "Échec de création de la partition EFI");
            return;
        }

        let p3 = Command::new("parted")
            .args(["-s", &req.target_disk, "mkpart", "nixos", "ext4", "512MiB", "100%"])
            .status();
        if p3.is_err() || !p3.unwrap().success() {
            emit_step(2, "Erreur Partitionnement", 20, "Échec de création de la partition Système");
            return;
        }

        let _ = Command::new("partprobe").arg(&req.target_disk).status();
        let _ = Command::new("udevadm").args(["settle", "--timeout=10"]).status();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Étape 3 : Formatage des systèmes de fichiers
        emit_step(3, "Formatage des partitions", 35, &format!("Formatage de {} (boot) et {} (root)...", boot_part, root_part));
        let f1 = Command::new("mkfs.vfat").args(["-F", "32", "-n", "boot", &boot_part]).status();
        if f1.is_err() || !f1.unwrap().success() {
            emit_step(3, "Erreur Formatage", 35, "Échec du formatage FAT32 de la partition EFI");
            return;
        }

        let f2 = Command::new("mkfs.ext4").args(["-F", "-L", "nixos", &root_part]).status();
        if f2.is_err() || !f2.unwrap().success() {
            emit_step(3, "Erreur Formatage", 35, "Échec du formatage ext4 de la partition racine");
            return;
        }

        let _ = Command::new("udevadm").args(["settle", "--timeout=10"]).status();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Étape 4 : Montage sur /mnt
        emit_step(4, "Montage des partitions", 45, "Montage du système de fichiers sur /mnt...");
        let _ = Command::new("mkdir").args(["-p", "/mnt"]).status();
        let m1 = Command::new("mount").args(["-t", "ext4", &root_part, "/mnt"]).status();
        if m1.is_err() || !m1.unwrap().success() {
            // Tentative supplémentaire après pause
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let m1_retry = Command::new("mount").args(["-t", "ext4", &root_part, "/mnt"]).status();
            if m1_retry.is_err() || !m1_retry.unwrap().success() {
                emit_step(4, "Erreur Montage", 45, "Impossible de monter la partition racine sur /mnt");
                return;
            }
        }

        let _ = Command::new("mkdir").args(["-p", "/mnt/boot"]).status();
        let m2 = Command::new("mount").args(["-t", "vfat", &boot_part, "/mnt/boot"]).status();
        if m2.is_err() || !m2.unwrap().success() {
            emit_step(4, "Erreur Montage", 45, "Impossible de monter la partition EFI sur /mnt/boot");
            return;
        }

        // Étape 5 : Déploiement de la configuration déclarative Noos HTPC
        emit_step(5, "Déploiement de Noos HTPC", 55, "Copie des fichiers de configuration...");
        let _ = Command::new("mkdir").args(["-p", "/mnt/etc/nixos"]).status();

        let config_src = match find_config_source() {
            Some(path) => path,
            None => {
                emit_step(5, "Erreur Source", 55, "Impossible de localiser les fichiers Noos HTPC (/etc/noos-htpc-source)");
                return;
            }
        };

        emit_step(5, "Déploiement de Noos HTPC", 58, &format!("Source détectée : {}", config_src.display()));

        let cp_res = Command::new("cp")
            .args(["-rT", config_src.to_str().unwrap(), "/mnt/etc/nixos"])
            .status();
        if cp_res.is_err() || !cp_res.unwrap().success() {
            emit_step(5, "Erreur Copie", 58, "Échec de la copie des fichiers de configuration");
            return;
        }

        // Rendre les fichiers modifiables (sortie du nix store readonly)
        let _ = Command::new("chmod").args(["-R", "u+w", "/mnt/etc/nixos"]).status();
        // Nettoyage des fichiers temporaires ou .git pour éviter les blocages de flake
        let _ = Command::new("rm").args(["-rf", "/mnt/etc/nixos/.git", "/mnt/etc/nixos/installer/target"]).status();

        // Génération automatique du hardware-configuration spécifique au matériel
        emit_step(5, "Configuration Matérielle", 62, "Détection du matériel cible via nixos-generate-config...");
        let _ = Command::new("mkdir").args(["-p", "/tmp/noos-hw"]).status();
        let _ = Command::new("nixos-generate-config").args(["--root", "/mnt", "--dir", "/tmp/noos-hw"]).status();
        let hw_file = Path::new("/tmp/noos-hw/hardware-configuration.nix");
        if hw_file.exists() {
            let _ = Command::new("cp")
                .args(["-f", "/tmp/noos-hw/hardware-configuration.nix", "/mnt/etc/nixos/hosts/htpc/hardware-configuration.nix"])
                .status();
        }

        // Injection déclarative du profil GPU et du hostname forcé "noos-htpc"
        let host_override = format!(
            "{{ ... }}: {{\n  networking.hostName = \"noos-htpc\";\n  hardware.noos-htpc.gpu.profile = \"{}\";\n  hardware.noos-htpc.gpu.enableHDR = {};\n}}\n",
            req.gpu_profile, req.enable_hdr
        );
        let _ = std::fs::write("/mnt/etc/nixos/hosts/htpc/host-settings.local.nix", host_override);

        // Étape 6 : Exécution de nixos-install avec streaming en direct (stdout + stderr)
        emit_step(6, "Installation du système NixOS", 70, "Lancement de nixos-install (téléchargement et compilation)...");

        let mut child = match Command::new("nixos-install")
            .args(["--flake", "/mnt/etc/nixos#htpc", "--no-root-passwd"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                emit_step(6, "Erreur Lancement", 70, &format!("Impossible d'exécuter nixos-install: {}", e));
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
            let _ = app.emit("install_progress", InstallProgress {
                step: 6,
                total_steps: 6,
                step_name: "Installation de Noos HTPC en cours...".to_string(),
                percent: 85,
                log_line: line,
            });
        }

        let status = child.wait();
        match status {
            Ok(s) if s.success() => {
                emit_step(6, "Installation Terminée avec Succès !", 100, "Noos HTPC a été compilé et installé avec succès.");
            }
            _ => {
                let msg = if !last_error_line.is_empty() {
                    format!("Échec de nixos-install : {}", last_error_line)
                } else {
                    "Une erreur est survenue lors de l'installation NixOS.".to_string()
                };
                emit_step(6, "Erreur durant l'installation", 85, &msg);
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

