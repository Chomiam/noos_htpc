use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuDetectionResult {
    pub detected_name: String,
    pub vendor: String,
    pub recommended_profile: String,
    pub hdr_supported: bool,
}

#[tauri::command]
pub async fn detect_gpu() -> Result<GpuDetectionResult, String> {
    tokio::task::spawn_blocking(|| {
        let is_vm = std::fs::read_to_string("/sys/class/dmi/id/product_name")
            .unwrap_or_default()
            .to_lowercase()
            .contains("qemu")
            || std::fs::read_to_string("/sys/class/dmi/id/sys_vendor")
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

        let output = Command::new("lspci")
            .output()
            .map_err(|e| format!("Impossible d'exécuter lspci: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();

        let mut vendor = if is_vm { "Machine Virtuelle (VM)".to_string() } else { "Inconnu".to_string() };
        let mut profile = if is_vm { "vm".to_string() } else { "generic".to_string() };
        let mut hdr = false;
        let mut name = if is_vm { "Affichage Virtuel (QEMU / KVM)".to_string() } else { "Carte graphique standard".to_string() };

        for line in stdout.lines() {
            if line.contains("vga compatible controller") || line.contains("3d controller") || line.contains("display controller") {
                name = line.to_string();

                if line.contains("qxl") || line.contains("virtio") || line.contains("vmware") || line.contains("virtualbox") || line.contains("red hat") || line.contains("bochs") {
                    vendor = "Machine Virtuelle (VM)".to_string();
                    profile = "vm".to_string();
                    hdr = false;
                    break;
                } else if line.contains("amd") || line.contains("advanced micro devices") || line.contains("radeon") {
                    vendor = "AMD".to_string();
                    profile = "amd".to_string();
                    hdr = true;
                    break;
                } else if line.contains("intel") {
                    vendor = "Intel".to_string();
                    profile = "intel".to_string();
                    hdr = false; // HDR expérimental limité sur iGPU Intel sous Linux
                    break;
                } else if line.contains("nvidia") {
                    vendor = "NVIDIA".to_string();
                    // Détection legacy simple : GeForce 6xx / 7xx / 8xx
                    if line.contains("gt 7") || line.contains("gtx 7") || line.contains("gtx 6") {
                        profile = "nvidia-legacy".to_string();
                        hdr = false;
                    } else {
                        profile = "nvidia".to_string();
                        hdr = true;
                    }
                    break;
                }
            }
        }

        if is_vm && profile == "generic" {
            vendor = "Machine Virtuelle (VM)".to_string();
            profile = "vm".to_string();
            hdr = false;
        }

        Ok(GpuDetectionResult {
            detected_name: name,
            vendor,
            recommended_profile: profile,
            hdr_supported: hdr,
        })
    })
    .await
    .map_err(|e| format!("Erreur thread détection GPU: {}", e))?
}
