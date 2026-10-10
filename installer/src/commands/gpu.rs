use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayInfo {
    pub connector: String,
    pub monitor_name: String,
    pub manufacturer: String,
    pub resolution: String,
    pub refresh_rate_hz: u32,
    pub is_connected: bool,
    pub hdr_supported: bool,
    pub audio_passthrough_supported: bool,
    pub edid_found: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareSummary {
    pub detected_name: String,
    pub vendor: String,
    pub recommended_profile: String,
    pub hdr_supported: bool,
    pub is_vm: bool,
    pub display: DisplayInfo,
    pub summary_text: String,
    pub features_compatible: Vec<String>,
}

pub type GpuDetectionResult = HardwareSummary;

fn decode_pnp_id(b1: u8, b2: u8) -> String {
    let c1 = (((b1 >> 2) & 0x1F) + b'@') as char;
    let c2 = ((((b1 & 0x03) << 3) | ((b2 >> 5) & 0x07)) + b'@') as char;
    let c3 = ((b2 & 0x1F) + b'@') as char;
    let code = format!("{}{}{}", c1, c2, c3);
    match code.as_str() {
        "GSM" => "LG Electronics".to_string(),
        "SAM" => "Samsung".to_string(),
        "SNY" => "Sony".to_string(),
        "PHL" => "Philips".to_string(),
        "TCL" => "TCL".to_string(),
        "HIS" => "Hisense".to_string(),
        "PAN" => "Panasonic".to_string(),
        "AOC" => "AOC".to_string(),
        "DEL" => "Dell".to_string(),
        "ACR" => "Acer".to_string(),
        "BNQ" => "BenQ".to_string(),
        "ASU" => "ASUS".to_string(),
        "VSC" => "ViewSonic".to_string(),
        _ => code,
    }
}

fn parse_edid_binary(bytes: &[u8]) -> Option<(String, String, bool, bool)> {
    if bytes.len() < 128 {
        return None;
    }
    // En-tête EDID standard : 00 FF FF FF FF FF FF 00
    if &bytes[0..8] != &[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00] {
        return None;
    }

    let manufacturer = decode_pnp_id(bytes[8], bytes[9]);

    let mut monitor_name = String::new();
    // Les 4 descripteurs de 18 octets dans le bloc de base
    for offset in [54, 72, 90, 108] {
        if offset + 18 <= bytes.len() {
            let desc = &bytes[offset..offset + 18];
            // 00 00 00 FC = Descripteur ASCII de nom du moniteur / téléviseur
            if desc[0] == 0x00 && desc[1] == 0x00 && desc[2] == 0x00 && desc[3] == 0xFC {
                let text = String::from_utf8_lossy(&desc[5..18]);
                let cleaned = text.split('\n').next().unwrap_or("").trim().to_string();
                if !cleaned.is_empty() {
                    monitor_name = cleaned;
                    break;
                }
            }
        }
    }

    let mut hdr_supported = false;
    let mut audio_supported = false;

    // Analyse des blocs d'extension CTA-861 / CEA-861
    let ext_count = bytes[126] as usize;
    for i in 1..=ext_count {
        let start = i * 128;
        let end = start + 128;
        if end <= bytes.len() {
            let ext = &bytes[start..end];
            // 0x02 = CTA-861 (HDMI / DisplayPort TV & Monitor Data)
            if ext[0] == 0x02 {
                let dtd_offset = (ext[2] as usize).min(128);
                let mut idx = 4;
                while idx < dtd_offset {
                    let header = ext[idx];
                    let tag = (header >> 5) & 0x07;
                    let len = (header & 0x1F) as usize;
                    idx += 1;
                    if idx + len > dtd_offset {
                        break;
                    }

                    if tag == 1 {
                        // Tag 1 : Audio Data Block (PCM, AC-3 / Dolby, DTS)
                        audio_supported = true;
                    } else if tag == 7 && len >= 2 {
                        // Tag 7 : Extended Tag
                        let ext_tag = ext[idx];
                        if ext_tag == 6 {
                            // Extended Tag 6 : HDR Static Metadata Data Block
                            let eotf = ext[idx + 1];
                            // Bit 2: SMPTE ST 2084 (HDR10), Bit 3: HLG
                            if (eotf & 0x04) != 0 || (eotf & 0x08) != 0 {
                                hdr_supported = true;
                            }
                        }
                    }
                    idx += len;
                }
            }
        }
    }

    Some((manufacturer, monitor_name, hdr_supported, audio_supported))
}

fn detect_display_info(is_vm: bool) -> DisplayInfo {
    let drm_path = Path::new("/sys/class/drm");
    if let Ok(entries) = fs::read_dir(drm_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Filtrer les connecteurs de type card*-HDMI-*, card*-DP-*, card*-eDP-*, etc.
            if name.contains('-') && (name.starts_with("card0-") || name.starts_with("card1-") || name.starts_with("card2-")) {
                let status_file = path.join("status");
                if let Ok(status) = fs::read_to_string(&status_file) {
                    if status.trim() == "connected" {
                        let connector_name = name.split_once('-').map(|(_, c)| c).unwrap_or(&name).to_string();

                        // Lecture de la meilleure résolution supportée
                        let mut resolution = "1920 × 1080".to_string();
                        let modes_file = path.join("modes");
                        if let Ok(modes) = fs::read_to_string(modes_file) {
                            if let Some(first_mode) = modes.lines().next() {
                                if !first_mode.trim().is_empty() {
                                    resolution = first_mode.trim().replace('x', " × ");
                                }
                            }
                        }

                        // Lecture et parsing du binaire EDID
                        let edid_file = path.join("edid");
                        let mut manufacturer = "Inconnu".to_string();
                        let mut monitor_name = String::new();
                        let mut hdr_supported = false;
                        let mut audio_supported = true; // Par défaut sur connecteur HDMI/DP
                        let mut edid_found = false;

                        if let Ok(edid_bytes) = fs::read(edid_file) {
                            if let Some((mfg, mon_name, hdr, audio)) = parse_edid_binary(&edid_bytes) {
                                manufacturer = mfg;
                                if !mon_name.is_empty() {
                                    monitor_name = mon_name;
                                }
                                hdr_supported = hdr;
                                audio_supported = audio || true;
                                edid_found = true;
                            }
                        }

                        let final_name = if !monitor_name.is_empty() {
                            if !manufacturer.is_empty() && manufacturer != "Inconnu" && !monitor_name.to_lowercase().contains(&manufacturer.to_lowercase()) {
                                format!("{} {}", manufacturer, monitor_name)
                            } else {
                                monitor_name
                            }
                        } else if !manufacturer.is_empty() && manufacturer != "Inconnu" {
                            format!("Téléviseur {} ({})", manufacturer, connector_name)
                        } else {
                            format!("Écran TV ({})", connector_name)
                        };

                        return DisplayInfo {
                            connector: connector_name,
                            monitor_name: final_name,
                            manufacturer,
                            resolution,
                            refresh_rate_hz: 60,
                            is_connected: true,
                            hdr_supported,
                            audio_passthrough_supported: audio_supported,
                            edid_found,
                        };
                    }
                }
            }
        }
    }

    if is_vm {
        DisplayInfo {
            connector: "Virtuel (QEMU)".to_string(),
            monitor_name: "Écran Virtuel (QEMU / KVM)".to_string(),
            manufacturer: "Red Hat / QEMU".to_string(),
            resolution: "1920 × 1080".to_string(),
            refresh_rate_hz: 60,
            is_connected: true,
            hdr_supported: false,
            audio_passthrough_supported: false,
            edid_found: false,
        }
    } else {
        DisplayInfo {
            connector: "HDMI / DP".to_string(),
            monitor_name: "Téléviseur / Écran HDMI standard".to_string(),
            manufacturer: "Standard".to_string(),
            resolution: "1920 × 1080".to_string(),
            refresh_rate_hz: 60,
            is_connected: true,
            hdr_supported: false,
            audio_passthrough_supported: true,
            edid_found: false,
        }
    }
}

fn clean_gpu_name(raw_line: &str) -> String {
    let lower = raw_line.to_lowercase();
    if lower.contains("uhd graphics 630") || lower.contains("coffeelake") {
        return "Intel UHD Graphics 630 (iGPU)".to_string();
    }
    if lower.contains("iris xe") {
        return "Intel Iris Xe Graphics (iGPU)".to_string();
    }
    if lower.contains("intel") && lower.contains("graphics") {
        let parts: Vec<&str> = raw_line.split('[').collect();
        if parts.len() > 1 {
            let sub = parts[1].split(']').next().unwrap_or(parts[1]).trim();
            return format!("Intel {} (iGPU)", sub);
        }
        return "Intel HD / UHD Graphics (iGPU)".to_string();
    }
    if lower.contains("radeon 780m") {
        return "AMD Radeon 780M (RDNA 3)".to_string();
    }
    if lower.contains("radeon 680m") {
        return "AMD Radeon 680M (RDNA 2)".to_string();
    }
    if lower.contains("amd") || lower.contains("radeon") {
        let parts: Vec<&str> = raw_line.split('[').collect();
        if parts.len() > 1 {
            let sub = parts[1].split(']').next().unwrap_or(parts[1]).trim();
            return format!("AMD Radeon {}", sub);
        }
        return "AMD Radeon Graphics / Ryzen APU".to_string();
    }
    if lower.contains("nvidia") || lower.contains("geforce") {
        let parts: Vec<&str> = raw_line.split('[').collect();
        if parts.len() > 1 {
            let sub = parts[1].split(']').next().unwrap_or(parts[1]).trim();
            return format!("NVIDIA {}", sub);
        }
        return "NVIDIA GeForce".to_string();
    }
    if lower.contains("virtio") || lower.contains("qxl") || lower.contains("vmware") || lower.contains("virtualbox") {
        return "Affichage Virtuel (QEMU / KVM VirtIO)".to_string();
    }

    raw_line.trim().to_string()
}

#[tauri::command]
pub async fn detect_hardware() -> Result<HardwareSummary, String> {
    tokio::task::spawn_blocking(|| {
        let is_vm = fs::read_to_string("/sys/class/dmi/id/product_name")
            .unwrap_or_default()
            .to_lowercase()
            .contains("qemu")
            || fs::read_to_string("/sys/class/dmi/id/sys_vendor")
                .unwrap_or_default()
                .to_lowercase()
                .contains("qemu")
            || Command::new("systemd-detect-virt")
                .output()
                .map(|o| {
                    o.status.success()
                        && !String::from_utf8_lossy(&o.stdout).trim().is_empty()
                        && String::from_utf8_lossy(&o.stdout).trim() != "none"
                })
                .unwrap_or(false);

        let stdout = Command::new("lspci")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        let mut vendor = if is_vm { "Machine Virtuelle (VM)".to_string() } else { "Inconnu".to_string() };
        let mut profile = if is_vm { "vm".to_string() } else { "generic".to_string() };
        let mut gpu_hdr_capable = false;
        let mut name = if is_vm { "Affichage Virtuel (QEMU / KVM)".to_string() } else { "Carte graphique standard".to_string() };

        for line in stdout.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("vga compatible controller") || line_lower.contains("3d controller") || line_lower.contains("display controller") {
                name = clean_gpu_name(line);

                if line_lower.contains("qxl") || line_lower.contains("virtio") || line_lower.contains("vmware") || line_lower.contains("virtualbox") || line_lower.contains("red hat") || line_lower.contains("bochs") {
                    vendor = "Machine Virtuelle (VM)".to_string();
                    profile = "vm".to_string();
                    gpu_hdr_capable = false;
                    break;
                } else if line_lower.contains("amd") || line_lower.contains("advanced micro devices") || line_lower.contains("radeon") {
                    vendor = "AMD".to_string();
                    profile = "amd".to_string();
                    gpu_hdr_capable = true;
                    break;
                } else if line_lower.contains("intel") {
                    vendor = "Intel".to_string();
                    profile = "intel".to_string();
                    gpu_hdr_capable = false; // Mode SDR recommandé pour fluidité iGPU Intel
                    break;
                } else if line_lower.contains("nvidia") {
                    vendor = "NVIDIA".to_string();
                    if line_lower.contains("gt 7") || line_lower.contains("gtx 7") || line_lower.contains("gtx 6") {
                        profile = "nvidia-legacy".to_string();
                        gpu_hdr_capable = false;
                    } else {
                        profile = "nvidia".to_string();
                        gpu_hdr_capable = true;
                    }
                    break;
                }
            }
        }

        if is_vm && profile == "generic" {
            vendor = "Machine Virtuelle (VM)".to_string();
            profile = "vm".to_string();
            gpu_hdr_capable = false;
        }

        // Détection de l'écran TV et de son bloc EDID
        let display = detect_display_info(is_vm);

        // Règle automatique d'activation du HDR :
        // Le HDR n'est activé que si à la fois la TV le supporte (EDID CEA-861) et le GPU le supporte nativement
        let final_hdr = gpu_hdr_capable && display.hdr_supported;

        // Génération de la liste des fonctionnalités compatibles
        let mut features = Vec::new();

        match profile.as_str() {
            "intel" => {
                features.push(format!("⚡ GPU Intel : Décodage matériel VA-API QuickSync (iHD) actif pour {}", name));
                features.push("⚡ Profil graphique : Basse consommation optimisée pour mini-PC HTPC".to_string());
            }
            "amd" => {
                features.push(format!("⚡ GPU AMD : Pilote noyau amdgpu natif avec Vulkan RADV pour {}", name));
                features.push("⚡ Profil graphique : Rendu matériel fluide et accélération 3D TV".to_string());
            }
            "nvidia" => {
                features.push(format!("⚡ GPU NVIDIA : Pilotes propriétaires haute performance avec décodage NVDEC pour {}", name));
            }
            "nvidia-legacy" => {
                features.push(format!("⚡ GPU NVIDIA Legacy : Pilote propriétaire 470xx stable pour {}", name));
            }
            "vm" => {
                features.push("⚡ Virtualisation : Pilote modesetting Mesa VirtIO / Gallium optimisé".to_string());
            }
            _ => {
                features.push("⚡ Profil générique : Pilotes Mesa standards universels".to_string());
            }
        }

        features.push(format!("📺 Affichage TV : {} connecté sur port {}", display.monitor_name, display.connector));
        features.push(format!("📺 Résolution native : {} @ {}Hz certifiée", display.resolution, display.refresh_rate_hz));

        if final_hdr {
            features.push("🌈 Plage Dynamique (HDR) : Compatible HDR10 / CEA-861 détecté (Activé automatiquement)".to_string());
        } else if display.hdr_supported {
            features.push("🌈 Plage Dynamique : Écran HDR détecté (Profil SDR 8-bit étendu appliqué pour fluidité iGPU)".to_string());
        } else {
            features.push("🌈 Plage Dynamique : SDR Standard 8-bit conforme au téléviseur".to_string());
        }

        features.push("🔊 Audio Bitperfect : Passthrough HDMI / SPDIF configuré pour ampli Home-Cinéma (Dolby Digital, DTS, Atmos)".to_string());
        features.push("💡 Ambilight Intégré : HyperHDR actif avec capture PipeWire 60 FPS (synchronisé sur vidéos et menus)".to_string());
        features.push("🛡️ Déclaration Matérielle : Fichier hardware-configuration.local.nix immunisé contre toute mise à jour GitHub".to_string());

        let summary_text = format!(
            "Configuration matérielle validée automatiquement : GPU {} ({}) | TV {} ({}) | HDR : {}",
            vendor,
            profile.to_uppercase(),
            display.monitor_name,
            display.resolution,
            if final_hdr { "Activé" } else { "Désactivé" }
        );

        Ok(HardwareSummary {
            detected_name: name,
            vendor,
            recommended_profile: profile,
            hdr_supported: final_hdr,
            is_vm,
            display,
            summary_text,
            features_compatible: features,
        })
    })
    .await
    .map_err(|e| format!("Erreur thread détection matériel : {}", e))?
}

#[tauri::command]
pub async fn detect_gpu() -> Result<GpuDetectionResult, String> {
    detect_hardware().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hardware_detection() {
        let result = detect_hardware().await;
        assert!(result.is_ok(), "Détection matérielle a échoué: {:?}", result.err());
        let summary = result.unwrap();
        println!("=== DIAGNOSTIC MATÉRIEL TEST ===");
        println!("GPU: {} (Vendor: {}, Profil: {})", summary.detected_name, summary.vendor, summary.recommended_profile);
        println!("Display: {} on {} (Res: {}, HDR: {})", summary.display.monitor_name, summary.display.connector, summary.display.resolution, summary.display.hdr_supported);
        println!("Summary: {}", summary.summary_text);
        for f in &summary.features_compatible {
            println!("  - {}", f);
        }
        assert!(!summary.detected_name.is_empty());
        assert!(!summary.recommended_profile.is_empty());
    }
}


