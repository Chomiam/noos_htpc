use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub model: String,
    pub bus_type: String,
    pub is_rotational: bool,
}

#[derive(Debug, Deserialize)]
struct LsblkOutput {
    blockdevices: Vec<LsblkDevice>,
}

#[derive(Debug, Deserialize)]
struct LsblkDevice {
    name: String,
    path: Option<String>,
    size: Option<u64>,
    #[serde(rename = "type")]
    device_type: String,
    model: Option<String>,
    tran: Option<String>,
    rota: Option<bool>,
}

fn format_bytes(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const TIB: u64 = 1024 * GIB;

    if bytes >= TIB {
        format!("{:.1} To", (bytes as f64) / (TIB as f64))
    } else {
        format!("{:.1} Go", (bytes as f64) / (GIB as f64))
    }
}

#[tauri::command]
pub async fn list_disks() -> Result<Vec<DiskInfo>, String> {
    tokio::task::spawn_blocking(|| {
        let output = Command::new("lsblk")
            .args(["--json", "-b", "-o", "NAME,PATH,SIZE,TYPE,MODEL,TRAN,ROTA"])
            .output()
            .map_err(|e| format!("Impossible d'exécuter lsblk: {}", e))?;

        if !output.status.success() {
            return Err("Échec de la commande lsblk".to_string());
        }

        let parsed: LsblkOutput = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("Erreur de désérialisation du JSON lsblk: {}", e))?;

        let mut disks = Vec::new();
        for dev in parsed.blockdevices {
            // Filtrer les lecteurs virtuels, loop, zram et les CD-ROM
            if dev.device_type == "disk"
                && !dev.name.starts_with("loop")
                && !dev.name.starts_with("ram")
                && !dev.name.starts_with("sr")
                && !dev.name.starts_with("zram")
            {
                let bytes = dev.size.unwrap_or(0);
                let path = dev.path.unwrap_or_else(|| format!("/dev/{}", dev.name));
                let model = dev.model.unwrap_or_else(|| "Disque inconnu".to_string()).trim().to_string();
                let bus = dev.tran.unwrap_or_else(|| "Interne".to_string());

                disks.push(DiskInfo {
                    name: dev.name,
                    path,
                    size_bytes: bytes,
                    size_human: format_bytes(bytes),
                    model,
                    bus_type: bus.to_uppercase(),
                    is_rotational: dev.rota.unwrap_or(false),
                });
            }
        }

        Ok(disks)
    })
    .await
    .map_err(|e| format!("Erreur du thread disque: {}", e))?
}
