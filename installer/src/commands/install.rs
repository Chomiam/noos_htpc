use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
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

        let is_nvme = req.target_disk.contains("nvme");
        let boot_part = if is_nvme { format!("{}p1", req.target_disk) } else { format!("{}1", req.target_disk) };
        let root_part = if is_nvme { format!("{}p2", req.target_disk) } else { format!("{}2", req.target_disk) };

        // Étape 2 : Partitionnement GPT / UEFI
        emit_step(2, "Partitionnement du disque en table GPT", 20, "Effacement et création des partitions EFI et Système...");
        let _ = Command::new("wipefs").args(["-a", &req.target_disk]).status();
        
        let p1 = Command::new("parted")
            .args(["-s", &req.target_disk, "mklabel", "gpt"])
            .status();
        if p1.is_err() || !p1.unwrap().success() {
            emit_step(2, "Erreur", 20, "Échec de l'initialisation de la table de partitions GPT");
            return;
        }

        let p2 = Command::new("parted")
            .args(["-s", &req.target_disk, "mkpart", "boot", "fat32", "1MiB", "512MiB", "set", "1", "esp", "on"])
            .status();
        let p3 = Command::new("parted")
            .args(["-s", &req.target_disk, "mkpart", "nixos", "ext4", "512MiB", "100%"])
            .status();
        let _ = Command::new("partprobe").arg(&req.target_disk).status();
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Étape 3 : Formatage des systèmes de fichiers
        emit_step(3, "Formatage des partitions", 35, &format!("Formatage de {} en FAT32 (boot) et {} en ext4 (root)...", boot_part, root_part));
        let _ = Command::new("mkfs.vfat").args(["-F", "32", "-n", "boot", &boot_part]).status();
        let _ = Command::new("mkfs.ext4").args(["-F", "-L", "nixos", &root_part]).status();

        // Étape 4 : Montage sur /mnt
        emit_step(4, "Montage des partitions sur /mnt", 45, "Montage du système de fichiers...");
        let _ = Command::new("mkdir").args(["-p", "/mnt"]).status();
        let _ = Command::new("mount").args([&root_part, "/mnt"]).status();
        let _ = Command::new("mkdir").args(["-p", "/mnt/boot"]).status();
        let _ = Command::new("mount").args([&boot_part, "/mnt/boot"]).status();

        // Étape 5 : Déploiement de la configuration déclarative Noos HTPC
        emit_step(5, "Déploiement de Noos HTPC", 55, "Génération de la configuration matérielle et injection du profil...");
        let _ = Command::new("mkdir").args(["-p", "/mnt/etc/nixos"]).status();
        let _ = Command::new("nixos-generate-config").args(["--root", "/mnt"]).status();

        // Copie des fichiers Noos HTPC dans /mnt/etc/nixos
        let _ = Command::new("cp")
            .args(["-rf", "/home/chomiam/Projets/noos_htpc/.", "/mnt/etc/nixos/"])
            .status();

        // Étape 6 : Exécution de nixos-install avec streaming des logs
        emit_step(6, "Installation et compilation du système NixOS", 70, "Lancement de nixos-install...");

        let mut child = match Command::new("nixos-install")
            .args(["--flake", "/mnt/etc/nixos#htpc", "--no-root-passwd"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                emit_step(6, "Erreur", 70, &format!("Impossible de lancer nixos-install: {}", e));
                return;
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                let _ = app.emit("install_progress", InstallProgress {
                    step: 6,
                    total_steps: 6,
                    step_name: "Installation de Noos HTPC en cours...".to_string(),
                    percent: 85,
                    log_line: line,
                });
            }
        }

        let status = child.wait();
        match status {
            Ok(s) if s.success() => {
                emit_step(6, "Installation Terminée avec Succès !", 100, "Noos HTPC est prêt. Vous pouvez redémarrer.");
            }
            _ => {
                emit_step(6, "Erreur durant nixos-install", 85, "Une erreur est survenue lors de la compilation NixOS.");
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
