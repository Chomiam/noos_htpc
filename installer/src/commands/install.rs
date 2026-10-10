use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
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

fn append_installer_log(msg: &str) {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/noos-installer.log")
    {
        let _ = writeln!(file, "{}", msg);
    }
}

fn run_cmd(cmd: &str, args: &[&str], desc: &str) -> Result<String, String> {
    append_installer_log(&format!("[EXEC] {} {}", cmd, args.join(" ")));
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| {
            let err = format!("Impossible d'exécuter '{}' ({}) : {}", cmd, desc, e);
            append_installer_log(&format!("[ERROR] {}", err));
            err
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        if !stdout.is_empty() {
            append_installer_log(&format!("[STDOUT] {}", stdout));
        }
        Ok(stdout)
    } else {
        let err_detail = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("Code d'erreur {}", output.status.code().unwrap_or(-1))
        };
        let err_msg = format!("Échec de '{}' ({}) : {}", cmd, desc, err_detail);
        append_installer_log(&format!("[FAIL] {}", err_msg));
        Err(err_msg)
    }
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
            append_installer_log(&format!("[Step {}/6] [{}] ({}%) {}", step, name, percent, log));
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

        append_installer_log("==================================================================");
        append_installer_log("             NOOS HTPC - DÉMARRAGE DE L'INSTALLATION             ");
        append_installer_log("==================================================================");

        // ==============================================================================
        // ÉTAPE 1 : CONTRÔLES PRÉ-VOL & PRÉPARATION DU STOCKAGE CIBLE
        // ==============================================================================
        emit_step(1, "Contrôles Pré-vol & Initialisation", 5, &format!("Validation du disque cible : {}", req.target_disk), 0, 0);

        // Contrôle 1.1 : Vérifier la présence du disque cible dans /dev
        if !Path::new(&req.target_disk).exists() {
            emit_step(1, "Erreur Contrôle Disque", 5, &format!("Le disque sélectionné '{}' n'existe pas ou est déconnecté.", req.target_disk), 0, 0);
            return;
        }

        // Contrôle 1.2 : Vérification du mode UEFI de la machine
        let is_uefi = Path::new("/sys/firmware/efi").exists();
        if is_uefi {
            emit_step(1, "Contrôle Matériel", 8, "Mode de démarrage : UEFI 64-bit conforme détecté", 0, 0);
        } else {
            emit_step(1, "Avertissement Matériel", 8, "Attention : Amorçage Legacy/BIOS détecté (UEFI recommandé)", 0, 0);
        }

        // Contrôle 1.3 : Vérification de la connectivité réseau
        let net_check = Command::new("ping").args(["-c", "1", "-W", "2", "cache.nixos.org"]).status();
        if net_check.map(|s| s.success()).unwrap_or(false) {
            emit_step(1, "Contrôle Réseau", 12, "Connectivité internet validée (Cache binaire NixOS joignable)", 0, 0);
        } else {
            emit_step(1, "Contrôle Réseau", 12, "Mode hors-ligne ou réseau restreint : installation via sources locales", 0, 0);
        }

        // Désactivation des swaps et démontage des partitions résiduelles
        emit_step(1, "Préparation du stockage", 15, "Démontage des systèmes de fichiers existants...", 0, 0);
        let _ = Command::new("swapoff").args(["-a"]).status();
        let _ = Command::new("umount").args(["-l", "-R", "/mnt"]).status();
        let _ = Command::new("umount").args(["-f", "-R", "/mnt"]).status();
        let _ = Command::new("udevadm").args(["settle", "--timeout=5"]).status();

        let is_nvme = req.target_disk.contains("nvme") || req.target_disk.contains("mmcblk");
        let boot_part = if is_nvme { format!("{}p1", req.target_disk) } else { format!("{}1", req.target_disk) };
        let swap_part = if is_nvme { format!("{}p2", req.target_disk) } else { format!("{}2", req.target_disk) };
        let root_part = if is_nvme { format!("{}p3", req.target_disk) } else { format!("{}3", req.target_disk) };

        // ==============================================================================
        // ÉTAPE 2 : PARTITIONNEMENT GPT & CONTRÔLE DE CONFORMITÉ DES PARTITIONS
        // ==============================================================================
        emit_step(2, "Partitionnement GPT", 20, "Effacement des anciennes signatures de partition...", 0, 0);
        let _ = Command::new("wipefs").args(["-a", "-f", &req.target_disk]).status();

        emit_step(2, "Partitionnement GPT", 22, "Création de la table de partition GPT...", 0, 0);
        if let Err(e) = run_cmd("parted", &["-s", &req.target_disk, "mklabel", "gpt"], "Création GPT") {
            emit_step(2, "Erreur Partitionnement", 22, &e, 0, 0);
            return;
        }

        // Partition 1 : EFI 1 Go (ESP)
        emit_step(2, "Partitionnement GPT", 24, "Création de la partition EFI (1 Go, ESP)...", 0, 0);
        if let Err(e) = run_cmd("parted", &["-s", &req.target_disk, "mkpart", "boot", "fat32", "1MiB", "1025MiB", "set", "1", "esp", "on"], "Partition EFI") {
            emit_step(2, "Erreur Partitionnement", 24, &e, 0, 0);
            return;
        }

        // Partition 2 : SWAP 8 Go
        emit_step(2, "Partitionnement GPT", 26, "Création de la partition SWAP (8 Go)...", 0, 0);
        if let Err(e) = run_cmd("parted", &["-s", &req.target_disk, "mkpart", "swap", "linux-swap", "1025MiB", "9217MiB"], "Partition SWAP") {
            emit_step(2, "Erreur Partitionnement", 26, &e, 0, 0);
            return;
        }

        // Partition 3 : Système NixOS (Reste du disque en ext4)
        emit_step(2, "Partitionnement GPT", 28, "Création de la partition Système NixOS (ext4)...", 0, 0);
        if let Err(e) = run_cmd("parted", &["-s", &req.target_disk, "mkpart", "nixos", "ext4", "9217MiB", "100%"], "Partition Système") {
            emit_step(2, "Erreur Partitionnement", 28, &e, 0, 0);
            return;
        }

        // Forcer le rechargement de la table des partitions par le noyau
        let _ = Command::new("partprobe").arg(&req.target_disk).status();
        let _ = Command::new("udevadm").args(["settle", "--timeout=10"]).status();
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

        // Contrôle de validation 2.1 : Vérifier la présence des 3 nœuds de partition dans /dev
        emit_step(2, "Contrôle des Partitions", 30, "Vérification de la détection noyau des partitions...", 0, 0);
        let mut partitions_ok = false;
        for _ in 0..10 {
            if Path::new(&boot_part).exists() && Path::new(&swap_part).exists() && Path::new(&root_part).exists() {
                partitions_ok = true;
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            let _ = Command::new("udevadm").args(["settle", "--timeout=2"]).status();
        }

        if !partitions_ok {
            emit_step(2, "Erreur Contrôle Partitions", 30, &format!("Les partitions n'ont pas été détectées dans /dev ({}, {}, {})", boot_part, swap_part, root_part), 0, 0);
            return;
        }
        emit_step(2, "Partitions Validées", 32, &format!("Nœuds vérifiés : EFI ({}), SWAP ({}), Système ({})", boot_part, swap_part, root_part), 0, 0);

        // ==============================================================================
        // ÉTAPE 3 : FORMATAGE & CONTRÔLE DES SYSTÈMES DE FICHIERS
        // ==============================================================================
        emit_step(3, "Formatage des Partitions", 35, &format!("Formatage de la partition EFI en FAT32 ({})", boot_part), 0, 0);
        if let Err(e) = run_cmd("mkfs.vfat", &["-F", "32", "-n", "boot", &boot_part], "Formatage EFI FAT32") {
            emit_step(3, "Erreur Formatage EFI", 35, &e, 0, 0);
            return;
        }

        emit_step(3, "Formatage des Partitions", 38, &format!("Initialisation de la partition SWAP ({})", swap_part), 0, 0);
        if let Err(e) = run_cmd("mkswap", &["-L", "swap", &swap_part], "Initialisation SWAP") {
            emit_step(3, "Erreur Formatage SWAP", 38, &e, 0, 0);
            return;
        }

        let _ = Command::new("swapon").arg(&swap_part).status();

        emit_step(3, "Formatage des Partitions", 42, &format!("Formatage de la partition Système en ext4 ({})", root_part), 0, 0);
        if let Err(e) = run_cmd("mkfs.ext4", &["-F", "-L", "nixos", &root_part], "Formatage ext4 Système") {
            emit_step(3, "Erreur Formatage Système", 42, &e, 0, 0);
            return;
        }

        let _ = Command::new("udevadm").args(["settle", "--timeout=10"]).status();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Contrôle de validation 3.1 : Vérification des signatures via blkid
        emit_step(3, "Contrôle des Systèmes de Fichiers", 45, "Validation des systèmes de fichiers créés via blkid...", 0, 0);
        let blkid_check = run_cmd("blkid", &[&boot_part, &swap_part, &root_part], "Vérification blkid");
        match blkid_check {
            Ok(out) => {
                append_installer_log(&format!("[CHECK BLKID]\n{}", out));
                emit_step(3, "Systèmes de Fichiers Validés", 48, "Partition EFI (boot), SWAP et Système (nixos) certifiées", 0, 0);
            }
            Err(e) => {
                append_installer_log(&format!("[WARN BLKID] {}", e));
            }
        }

        // ==============================================================================
        // ÉTAPE 4 : MONTAGE DES SYSTÈMES DE FICHIERS & CONTRÔLE
        // ==============================================================================
        emit_step(4, "Montage des Partitions", 50, "Création du point de montage /mnt...", 0, 0);
        let _ = Command::new("mkdir").args(["-p", "/mnt"]).status();

        emit_step(4, "Montage des Partitions", 52, &format!("Montage de la racine {} sur /mnt...", root_part), 0, 0);
        if let Err(first_err) = run_cmd("mount", &["-t", "ext4", &root_part, "/mnt"], "Montage /mnt") {
            append_installer_log(&format!("[WARN MONTAGE] {}", first_err));
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            if let Err(retry_err) = run_cmd("mount", &["-t", "ext4", &root_part, "/mnt"], "Montage /mnt (nouvel essai)") {
                emit_step(4, "Erreur Montage Racine", 52, &retry_err, 0, 0);
                return;
            }
        }

        emit_step(4, "Montage des Partitions", 54, &format!("Montage de la partition EFI {} sur /mnt/boot...", boot_part), 0, 0);
        let _ = Command::new("mkdir").args(["-p", "/mnt/boot"]).status();
        if let Err(e) = run_cmd("mount", &["-t", "vfat", &boot_part, "/mnt/boot"], "Montage /mnt/boot") {
            emit_step(4, "Erreur Montage EFI", 54, &e, 0, 0);
            return;
        }

        // Contrôle de validation 4.1 : Vérification que /mnt et /mnt/boot sont des points de montage effectifs
        emit_step(4, "Contrôle des Montages", 56, "Vérification des points de montage actifs...", 0, 0);
        let check_mnt = run_cmd("findmnt", &["/mnt"], "Contrôle findmnt /mnt");
        let check_boot = run_cmd("findmnt", &["/mnt/boot"], "Contrôle findmnt /mnt/boot");
        if check_mnt.is_err() || check_boot.is_err() {
            emit_step(4, "Erreur Contrôle Montage", 56, "Le système de fichiers cible n'est pas correctement monté sur /mnt", 0, 0);
            return;
        }
        emit_step(4, "Points de Montage Certifiés", 58, "Arborescence /mnt (racine) et /mnt/boot (EFI) opérationnelle", 0, 0);

        // ==============================================================================
        // ÉTAPE 5 : DÉPLOIEMENT DE LA CONFIGURATION DÉCLARATIVE NOOS HTPC & CONTRÔLE
        // ==============================================================================
        emit_step(5, "Déploiement de Noos HTPC", 60, "Recherche de la source de configuration...", 0, 0);
        let _ = Command::new("mkdir").args(["-p", "/mnt/etc/nixos"]).status();

        let config_src = match find_config_source() {
            Some(path) => path,
            None => {
                emit_step(5, "Erreur Source Introuvable", 60, "Impossible de localiser les fichiers sources Noos HTPC (/etc/noos-htpc-source)", 0, 0);
                return;
            }
        };

        // Contrôle 5.1 : Vérifier la validité des fichiers indispensables dans la source
        emit_step(5, "Contrôle Source Noos", 61, &format!("Validation de la source : {}", config_src.display()), 0, 0);
        if !config_src.join("flake.nix").exists() || !config_src.join("hosts/htpc/configuration.nix").exists() {
            emit_step(5, "Erreur Source Invalide", 61, "La source de configuration trouvée est incomplète (flake.nix manquant)", 0, 0);
            return;
        }

        // Résolution du chemin canonique (suit les symlinks comme /etc/noos-htpc-source -> /nix/store/...-source)
        let real_source = std::fs::canonicalize(&config_src).unwrap_or_else(|_| config_src.clone());
        emit_step(5, "Copie des Fichiers", 63, &format!("Déploiement depuis : {}", real_source.display()), 0, 0);

        let src_pattern = format!("{}/.", real_source.display());
        let cp_res = Command::new("cp")
            .args(["-aL", &src_pattern, "/mnt/etc/nixos/"])
            .output();

        match cp_res {
            Ok(output) if output.status.success() => {
                append_installer_log("[SUCCESS] Copie des fichiers système réussie");
            }
            Ok(output) => {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                emit_step(5, "Erreur Copie Fichiers", 63, &format!("Échec de la copie des fichiers de configuration : {}", err_msg.trim()), 0, 0);
                return;
            }
            Err(e) => {
                emit_step(5, "Erreur Exécution Copie", 63, &format!("Impossible d'exécuter la commande cp : {}", e), 0, 0);
                return;
            }
        }

        // Rendre les fichiers modifiables (sortie du store lecture seule)
        let _ = Command::new("chmod").args(["-R", "u+w", "/mnt/etc/nixos"]).status();
        let _ = Command::new("chmod").args(["-R", "a+rX", "/mnt/etc/nixos"]).status();

        // Nettoyage des dossiers temporaires ou .git pour éviter les blocages de flake
        let _ = Command::new("rm").args(["-rf", "/mnt/etc/nixos/.git", "/mnt/etc/nixos/installer/target", "/mnt/etc/nixos/dashboard/target"]).status();

        // Contrôle de validation 5.2 : Vérification de la présence effective de flake.nix dans /mnt
        emit_step(5, "Contrôle Post-Copie", 65, "Vérification de l'arborescence /mnt/etc/nixos...", 0, 0);
        if !Path::new("/mnt/etc/nixos/flake.nix").exists() {
            emit_step(5, "Erreur Contrôle Copie", 65, "Le fichier /mnt/etc/nixos/flake.nix est introuvable après la copie", 0, 0);
            return;
        }

        // Génération automatique du hardware-configuration spécifique au matériel
        emit_step(5, "Configuration Matérielle", 66, "Détection du matériel cible via nixos-generate-config...", 0, 0);
        let _ = Command::new("mkdir").args(["-p", "/tmp/noos-hw"]).status();
        if let Err(e) = run_cmd("nixos-generate-config", &["--root", "/mnt", "--dir", "/tmp/noos-hw"], "Génération matérielle") {
            emit_step(5, "Avertissement Matériel", 66, &format!("Génération assistée: {}", e), 0, 0);
        }

        let hw_file = Path::new("/tmp/noos-hw/hardware-configuration.nix");
        if hw_file.exists() {
            let _ = Command::new("cp")
                .args(["-f", "/tmp/noos-hw/hardware-configuration.nix", "/mnt/etc/nixos/hosts/htpc/hardware-configuration.local.nix"])
                .status();
            emit_step(5, "Configuration Matérielle", 68, "Pilotes noyau et partitions spécifiques à ce PC appliqués (hardware-configuration.local.nix)", 0, 0);
        }

        // Injection déclarative du profil GPU sélectionné, du hostname forcé et des options
        let host_override = format!(
            "# Configuration matérielle déclarative spécifique à cette machine.\n# Unique à ce PC physique et immunisé contre les mises à jour GitHub (noos-update).\n{{ lib, ... }}: {{\n  networking.hostName = lib.mkForce \"noos-htpc\";\n  hardware.noos-htpc.gpu.profile = lib.mkForce \"{}\";\n  hardware.noos-htpc.gpu.enableHDR = lib.mkForce {};\n  nix.settings.experimental-features = [ \"nix-command\" \"flakes\" ];\n}}\n",
            req.gpu_profile, req.enable_hdr
        );
        if let Err(e) = std::fs::write("/mnt/etc/nixos/hosts/htpc/host-settings.local.nix", &host_override) {
            emit_step(5, "Erreur Paramètres Locaux", 69, &format!("Impossible d'écrire host-settings.local.nix : {}", e), 0, 0);
            return;
        }

        // Initialisation du dépôt Git local pour permettre les futures mises à jour via noos-update
        if !Path::new("/mnt/etc/nixos/.git").exists() {
            let _ = Command::new("git").args(["init", "-b", "testing"]).current_dir("/mnt/etc/nixos").status();
            let _ = Command::new("git").args(["remote", "add", "origin", "https://github.com/Chomiam/noos_htpc.git"]).current_dir("/mnt/etc/nixos").status();
            let _ = Command::new("git").args(["config", "user.name", "Noos Installer"]).current_dir("/mnt/etc/nixos").status();
            let _ = Command::new("git").args(["config", "user.email", "installer@noos-htpc.local"]).current_dir("/mnt/etc/nixos").status();
            let _ = Command::new("git").args(["add", "."]).current_dir("/mnt/etc/nixos").status();
            let _ = Command::new("git").args(["commit", "-m", "chore: configuration initiale Noos HTPC"]).current_dir("/mnt/etc/nixos").status();
        }

        emit_step(5, "Configuration Noos Prête", 70, &format!("Profil graphique : {} | HDR : {} (Protégé des mises à jour)", req.gpu_profile.to_uppercase(), if req.enable_hdr { "Activé" } else { "Désactivé" }), 0, 0);

        // ==============================================================================
        // ÉTAPE 6 : INSTALLATION NIXOS (nixos-install) AVEC STREAMING & CONTRÔLE FINAL
        // ==============================================================================
        let mut packages_total: u32 = 0;
        let mut packages_done: u32 = 0;

        emit_step(6, "Installation du Système NixOS", 72, "Lancement de nixos-install (téléchargement et compilation)...", 0, 0);

        let mut child = match Command::new("nixos-install")
            .args(["--no-channel-copy", "--impure", "--flake", "/mnt/etc/nixos#htpc", "--no-root-passwd"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                emit_step(6, "Erreur Lancement Installation", 72, &format!("Impossible d'exécuter nixos-install: {}", e), 0, 0);
                return;
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        if let Some(stdout) = child.stdout.take() {
            let tx_out = tx.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    append_installer_log(&format!("[NIXOS-INSTALL] {}", line));
                    let _ = tx_out.send(line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let tx_err = tx.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    append_installer_log(&format!("[NIXOS-ERR] {}", line));
                    let _ = tx_err.send(line);
                }
            });
        }

        drop(tx);

        let mut last_error_line = String::new();
        let mut last_logged_line = String::new();
        let mut duplicate_count: u32 = 0;

        while let Some(line) = rx.recv().await {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

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

            // Calcul dynamique du pourcentage (plage 72% à 98%)
            let current_percent = if packages_total > 0 {
                let ratio = (packages_done as f32) / (packages_total as f32);
                let p = 72.0 + (ratio * 26.0);
                p.min(98.0) as u8
            } else {
                75
            };

            // Formater un message court et lisible pour l'en-tête de progression
            let display_msg = if trimmed.starts_with("copying path '") {
                let pkg = trimmed.replace("copying path '", "").replace("'", "");
                let short_pkg = pkg.split('/').last().unwrap_or(&pkg);
                format!("Téléchargement : {}", short_pkg)
            } else if trimmed.starts_with("building '") {
                let drv = trimmed.replace("building '", "").replace("'", "");
                let short_drv = drv.split('/').last().unwrap_or(&drv);
                format!("Compilation : {}", short_drv)
            } else {
                trimmed.to_string()
            };

            // Éviter de saturer l'interface avec des messages consécutifs rigoureusement identiques
            if trimmed == last_logged_line {
                duplicate_count += 1;
                // Si la ligne se répète, ne la renvoyer que par paliers de 20 pour rafraîchir le compteur de paquets
                if duplicate_count % 20 != 0 {
                    continue;
                }
            } else {
                last_logged_line = trimmed.to_string();
                duplicate_count = 1;
            }

            emit_step(6, "Installation de Noos HTPC en cours...", current_percent, &display_msg, packages_done, packages_total);
        }

        let status = child.wait();
        match status {
            Ok(s) if s.success() => {
                // Contrôle de validation 6.1 (Post-installation) : Vérification du bootloader EFI et du système installé
                emit_step(6, "Contrôle Final du Système", 99, "Vérification des entrées de démarrage EFI et du profil système...", packages_total, packages_total);
                
                let efi_installed = Path::new("/mnt/boot/EFI").exists() || Path::new("/mnt/boot/efi").exists();
                let system_installed = Path::new("/mnt/nix/var/nix/profiles/system").exists() || Path::new("/mnt/run/current-system").exists();

                if efi_installed {
                    append_installer_log("[CHECK OK] Bootloader EFI certifié dans /mnt/boot/EFI");
                }
                if system_installed {
                    append_installer_log("[CHECK OK] Profil système NixOS certifié dans /mnt/nix/var/nix/profiles/system");
                }

                // Sauvegarde persistante des logs d'installation sur le disque installé
                let _ = Command::new("mkdir").args(["-p", "/mnt/var/log"]).status();
                let _ = Command::new("cp").args(["-f", "/tmp/noos-installer.log", "/mnt/var/log/noos-installer.log"]).status();
                let _ = Command::new("sync").status();

                let final_done = if packages_total > 0 { packages_total } else { packages_done };
                emit_step(6, "Installation Terminée avec Succès !", 100, "Noos HTPC a été compilé et installé avec succès.", final_done, packages_total);
                append_installer_log("==================================================================");
                append_installer_log("       INSTALLATION TERMINÉE AVEC SUCCÈS (LOGS DANS /var/log)     ");
                append_installer_log("==================================================================");
            }
            _ => {
                let msg = if !last_error_line.is_empty() {
                    format!("Échec de nixos-install : {}", last_error_line)
                } else {
                    "Une erreur est survenue lors de l'exécution de nixos-install.".to_string()
                };
                emit_step(6, "Erreur durant l'installation", 85, &msg, packages_done, packages_total);
                append_installer_log(&format!("[FATAL ERROR] {}", msg));

                // Sauvegarde du log même en cas d'erreur
                let _ = Command::new("mkdir").args(["-p", "/mnt/var/log"]).status();
                let _ = Command::new("cp").args(["-f", "/tmp/noos-installer.log", "/mnt/var/log/noos-installer.log"]).status();
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
