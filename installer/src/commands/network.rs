use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WifiAccessPoint {
    pub ssid: String,
    pub signal_percent: u8,
    pub security: String,
    pub is_connected: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkStatus {
    pub is_connected: bool,
    pub connection_type: String,
    pub ip_address: String,
}

#[tauri::command]
pub async fn scan_wifi() -> Result<Vec<WifiAccessPoint>, String> {
    tokio::task::spawn_blocking(|| {
        let output = Command::new("nmcli")
            .args(["-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "device", "wifi", "list"])
            .output();

        let mut networks = Vec::new();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let in_use = parts[0] == "*";
                    let ssid = parts[1].trim().to_string();
                    if !ssid.is_empty() {
                        let signal: u8 = parts[2].trim().parse().unwrap_or(50);
                        let security = parts[3].trim().to_string();

                        networks.push(WifiAccessPoint {
                            ssid,
                            signal_percent: signal,
                            security,
                            is_connected: in_use,
                        });
                    }
                }
            }
        }
        Ok(networks)
    })
    .await
    .map_err(|e| format!("Erreur thread Wi-Fi: {}", e))?
}

#[tauri::command]
pub async fn connect_wifi(ssid: String, password: Option<String>) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new("nmcli");
        cmd.args(["device", "wifi", "connect", &ssid]);

        if let Some(ref pwd) = password {
            if !pwd.is_empty() {
                cmd.args(["password", pwd]);
            }
        }

        let output = cmd.output().map_err(|e| format!("Échec d'exécution nmcli: {}", e))?;
        if output.status.success() {
            Ok(true)
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(format!("Impossible de se connecter au réseau: {}", err))
        }
    })
    .await
    .map_err(|e| format!("Erreur thread connexion Wi-Fi: {}", e))?
}

#[tauri::command]
pub async fn get_network_status() -> Result<NetworkStatus, String> {
    tokio::task::spawn_blocking(|| {
        let output = Command::new("ip")
            .args(["route", "get", "1.1.1.1"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                let is_wifi = text.contains("wlan") || text.contains("wlp");
                let conn_type = if is_wifi { "Wi-Fi" } else { "Ethernet Filaire" };
                
                let mut ip = "Inconnue".to_string();
                if let Some(src_pos) = text.find("src ") {
                    let rest = &text[src_pos + 4..];
                    if let Some(space_pos) = rest.find(' ') {
                        ip = rest[..space_pos].trim().to_string();
                    }
                }

                Ok(NetworkStatus {
                    is_connected: true,
                    connection_type: conn_type.to_string(),
                    ip_address: ip,
                })
            }
            _ => Ok(NetworkStatus {
                is_connected: false,
                connection_type: "Aucune".to_string(),
                ip_address: "Non assignée".to_string(),
            }),
        }
    })
    .await
    .map_err(|e| format!("Erreur thread statut réseau: {}", e))?
}
