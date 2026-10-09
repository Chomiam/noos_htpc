use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

const MAGIC_HEADER: &[u8; 8] = b"NOOSIPTV";

/* ========================================================================= */
/* STRUCTURES DE DONNÉES IPTV                                                */
/* ========================================================================= */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvCredentials {
    pub name: String,
    pub server_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvProfilePublic {
    pub id: String,
    pub name: String,
    pub server_url: String,
    pub username: String,
    pub last_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvProfileInternal {
    pub id: String,
    pub name: String,
    pub server_url: String,
    pub username: String,
    pub password: String,
    pub last_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvSyncProgress {
    pub step: u8,
    pub total_steps: u8,
    pub step_name: String,
    pub percent: u8,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvSyncResult {
    pub success: bool,
    pub profile_id: String,
    pub profile_name: String,
    pub message: String,
    pub channels_count: usize,
    pub movies_count: usize,
    pub series_count: usize,
    pub exp_date: Option<String>,
    pub max_connections: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IptvCacheSummary {
    pub has_cache: bool,
    pub profile_id: String,
    pub last_synced: u64,
    pub channels_count: usize,
    pub movies_count: usize,
    pub series_count: usize,
    pub status: String,
    pub exp_date: Option<String>,
}

/* ========================================================================= */
/* MODULE CRYPTOGRAPHIQUE SÉCURISÉ (SHA-256 + CHACHA20 + HMAC)               */
/* ========================================================================= */

fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let mut msg = data.to_vec();
    let bit_len = (msg.len() as u64) * 8;
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_val = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_val.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_val = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_val);
    }

    let mut out = [0u8; 32];
    for (i, val) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut key_block = [0u8; 64];
    if key.len() > 64 {
        let digest = sha256_digest(key);
        key_block[..32].copy_from_slice(&digest);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut o_key_pad = [0x5cu8; 64];
    let mut i_key_pad = [0x36u8; 64];
    for i in 0..64 {
        o_key_pad[i] ^= key_block[i];
        i_key_pad[i] ^= key_block[i];
    }

    let mut inner = Vec::with_capacity(64 + message.len());
    inner.extend_from_slice(&i_key_pad);
    inner.extend_from_slice(message);
    let inner_hash = sha256_digest(&inner);

    let mut outer = Vec::with_capacity(64 + 32);
    outer.extend_from_slice(&o_key_pad);
    outer.extend_from_slice(&inner_hash);
    sha256_digest(&outer)
}

fn chacha20_quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);

    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}

fn chacha20_process(key: &[u8; 32], nonce: &[u8; 12], counter: u32, data: &mut [u8]) {
    let constants = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];
    let mut key_words = [0u32; 8];
    for i in 0..8 {
        key_words[i] = u32::from_le_bytes([key[i * 4], key[i * 4 + 1], key[i * 4 + 2], key[i * 4 + 3]]);
    }
    let mut nonce_words = [0u32; 3];
    for i in 0..3 {
        nonce_words[i] = u32::from_le_bytes([nonce[i * 4], nonce[i * 4 + 1], nonce[i * 4 + 2], nonce[i * 4 + 3]]);
    }

    let mut current_counter = counter;
    for chunk in data.chunks_mut(64) {
        let state = [
            constants[0], constants[1], constants[2], constants[3],
            key_words[0], key_words[1], key_words[2], key_words[3],
            key_words[4], key_words[5], key_words[6], key_words[7],
            current_counter, nonce_words[0], nonce_words[1], nonce_words[2],
        ];

        let mut working = state;
        for _ in 0..10 {
            // Colonnes
            chacha20_quarter_round(&mut working, 0, 4, 8, 12);
            chacha20_quarter_round(&mut working, 1, 5, 9, 13);
            chacha20_quarter_round(&mut working, 2, 6, 10, 14);
            chacha20_quarter_round(&mut working, 3, 7, 11, 15);
            // Diagonales
            chacha20_quarter_round(&mut working, 0, 5, 10, 15);
            chacha20_quarter_round(&mut working, 1, 6, 11, 12);
            chacha20_quarter_round(&mut working, 2, 7, 8, 13);
            chacha20_quarter_round(&mut working, 3, 4, 9, 14);
        }

        let mut keystream = [0u8; 64];
        for i in 0..16 {
            let word = working[i].wrapping_add(state[i]);
            keystream[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }

        for (i, byte) in chunk.iter_mut().enumerate() {
            *byte ^= keystream[i];
        }

        current_counter = current_counter.wrapping_add(1);
    }
}

/* ========================================================================= */
/* DÉRIVATION DE CLÉ MACHINE SÉCURISÉE & CHIFFREMENT AU REPOS                */
/* ========================================================================= */

fn get_machine_identifier() -> String {
    let candidates = [
        "/etc/machine-id",
        "/var/lib/dbus/machine-id",
        "/etc/hostid",
    ];
    for path in candidates {
        if let Ok(content) = fs::read_to_string(path) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // Empreinte de secours matérielle locale
    let mut fallback = "noos-htpc-default-secret-salt-2026".to_string();
    if let Ok(hostname) = fs::read_to_string("/etc/hostname") {
        fallback.push_str(hostname.trim());
    }
    fallback
}

fn derive_key(salt: &[u8; 16]) -> [u8; 32] {
    let machine_id = get_machine_identifier();
    let mut current = Vec::new();
    current.extend_from_slice(machine_id.as_bytes());
    current.extend_from_slice(salt);

    let mut key = sha256_digest(&current);
    // KDF à itérations multiples (10 000 rondes)
    for i in 0..10_000u32 {
        let mut loop_input = Vec::with_capacity(32 + 4);
        loop_input.extend_from_slice(&key);
        loop_input.extend_from_slice(&i.to_le_bytes());
        key = sha256_digest(&loop_input);
    }
    key
}

fn get_random_bytes(len: usize) -> Vec<u8> {
    if let Ok(mut f) = File::open("/dev/urandom") {
        let mut buf = vec![0u8; len];
        if f.read_exact(&mut buf).is_ok() {
            return buf;
        }
    }
    // Secours pseudo-aléatoire basé sur horloge système haute résolution
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let hash = sha256_digest(&now.to_le_bytes());
    hash[..len.min(32)].to_vec()
}

pub fn encrypt_profiles_payload(plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let salt_vec = get_random_bytes(16);
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&salt_vec[..16]);

    let nonce_vec = get_random_bytes(12);
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&nonce_vec[..12]);

    let key = derive_key(&salt);

    let mut ciphertext = plaintext.to_vec();
    chacha20_process(&key, &nonce, 1, &mut ciphertext);

    let mut mac_input = Vec::new();
    mac_input.extend_from_slice(MAGIC_HEADER);
    mac_input.extend_from_slice(&salt);
    mac_input.extend_from_slice(&nonce);
    mac_input.extend_from_slice(&ciphertext);
    let hmac = hmac_sha256(&key, &mac_input);

    let mut out = Vec::with_capacity(8 + 16 + 12 + 32 + ciphertext.len());
    out.extend_from_slice(MAGIC_HEADER);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&hmac);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn decrypt_profiles_payload(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < (8 + 16 + 12 + 32) {
        return Err("Fichier de profils IPTV corrompu ou trop court.".to_string());
    }

    if &data[..8] != MAGIC_HEADER {
        return Err("En-tête de chiffrement invalide (non-Noos IPTV).".to_string());
    }

    let mut salt = [0u8; 16];
    salt.copy_from_slice(&data[8..24]);

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&data[24..36]);

    let expected_hmac = &data[36..68];
    let ciphertext = &data[68..];

    let key = derive_key(&salt);

    let mut mac_input = Vec::new();
    mac_input.extend_from_slice(MAGIC_HEADER);
    mac_input.extend_from_slice(&salt);
    mac_input.extend_from_slice(&nonce);
    mac_input.extend_from_slice(ciphertext);
    let computed_hmac = hmac_sha256(&key, &mac_input);

    // Comparaison en temps constant pour éviter toute attaque par canal auxiliaire
    let mut diff = 0u8;
    for (a, b) in expected_hmac.iter().zip(computed_hmac.iter()) {
        diff |= a ^ b;
    }
    if diff != 0 {
        return Err("Échec d'authentification HMAC : données chiffrées altérées ou clé machine modifiée.".to_string());
    }

    let mut plaintext = ciphertext.to_vec();
    chacha20_process(&key, &nonce, 1, &mut plaintext);
    Ok(plaintext)
}

/* ========================================================================= */
/* PERSISTANCE DES PROFILS & CACHE IPTV                                      */
/* ========================================================================= */

fn get_iptv_base_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/noos".to_string());
    let dir = PathBuf::from(home).join(".config/noos-htpc/iptv");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn get_iptv_cache_dir() -> PathBuf {
    let dir = get_iptv_base_dir().join("cache");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn load_internal_profiles() -> Vec<IptvProfileInternal> {
    let file_path = get_iptv_base_dir().join("profiles.enc");
    if !file_path.exists() {
        return Vec::new();
    }

    if let Ok(encrypted_bytes) = fs::read(&file_path) {
        if let Ok(decrypted) = decrypt_profiles_payload(&encrypted_bytes) {
            if let Ok(profiles) = serde_json::from_slice::<Vec<IptvProfileInternal>>(&decrypted) {
                return profiles;
            }
        }
    }
    Vec::new()
}

fn save_internal_profiles(profiles: &[IptvProfileInternal]) -> Result<(), String> {
    let json = serde_json::to_vec_pretty(profiles).map_err(|e| e.to_string())?;
    let encrypted = encrypt_profiles_payload(&json)?;
    let file_path = get_iptv_base_dir().join("profiles.enc");
    fs::write(&file_path, encrypted).map_err(|e| format!("Impossible d'enregistrer les profils : {}", e))?;
    Ok(())
}

fn clean_server_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        format!("http://{}", trimmed)
    } else {
        trimmed.to_string()
    }
}

/* ========================================================================= */
/* COMMANDES TAURI EXPORTÉES                                                 */
/* ========================================================================= */

#[tauri::command]
pub async fn iptv_get_saved_profiles() -> Result<Vec<IptvProfilePublic>, String> {
    tokio::task::spawn_blocking(move || {
        let internal = load_internal_profiles();
        let mut public: Vec<IptvProfilePublic> = internal.into_iter().map(|p| IptvProfilePublic {
            id: p.id,
            name: p.name,
            server_url: p.server_url,
            username: p.username,
            last_used: p.last_used,
        }).collect();
        public.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        Ok(public)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_delete_profile(profile_id: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let mut profiles = load_internal_profiles();
        let initial_len = profiles.len();
        profiles.retain(|p| p.id != profile_id);

        if profiles.len() != initial_len {
            save_internal_profiles(&profiles)?;
            // Supprimer le cache associé
            let cache_prefix = format!("profile_{}", profile_id);
            if let Ok(entries) = fs::read_dir(get_iptv_cache_dir()) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(&cache_prefix) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_get_cache_summary(profile_id: String) -> Result<IptvCacheSummary, String> {
    tokio::task::spawn_blocking(move || {
        let meta_file = get_iptv_cache_dir().join(format!("profile_{}_meta.json", profile_id));
        if meta_file.exists() {
            if let Ok(data) = fs::read_to_string(&meta_file) {
                if let Ok(summary) = serde_json::from_str::<IptvCacheSummary>(&data) {
                    return Ok(summary);
                }
            }
        }
        Ok(IptvCacheSummary {
            has_cache: false,
            profile_id,
            ..Default::default()
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_login_and_sync(
    app: AppHandle,
    profile_id: Option<String>,
    new_creds: Option<IptvCredentials>,
    save_profile: bool,
) -> Result<IptvSyncResult, String> {
    let emit_progress = move |step: u8, total: u8, name: &str, percent: u8, details: &str| {
        let p = IptvSyncProgress {
            step,
            total_steps: total,
            step_name: name.to_string(),
            percent,
            details: details.to_string(),
        };
        let _ = app.emit("iptv_sync_progress", p);
    };

    tokio::task::spawn_blocking(move || {
        // 1. Résolution des identifiants (profil existant chiffré ou nouveaux identifiants)
        let (p_id, p_name, server_url, username, password) = if let Some(id) = profile_id {
            let profiles = load_internal_profiles();
            if let Some(found) = profiles.into_iter().find(|p| p.id == id) {
                (found.id, found.name, found.server_url, found.username, found.password)
            } else {
                return Err("Profil IPTV introuvable sur ce système.".to_string());
            }
        } else if let Some(creds) = new_creds {
            let clean_url = clean_server_url(&creds.server_url);
            let time_val = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let hash = sha256_digest(format!("{}:{}:{}", clean_url, creds.username, time_val).as_bytes());
            let gen_id: String = hash.iter().take(6).map(|b| format!("{:02x}", b)).collect();
            let final_name = if creds.name.trim().is_empty() {
                format!("Compte {}", creds.username)
            } else {
                creds.name.clone()
            };
            (gen_id, final_name, clean_url, creds.username, creds.password)
        } else {
            return Err("Identifiants ou profil requis pour la connexion IPTV.".to_string());
        };

        let is_demo = server_url.contains("demo") || username.to_lowercase() == "demo";

        let now_sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Étape 1 : Authentification Xtream Codes (player_api.php)
        emit_progress(1, 5, "Authentification Xtream Codes", 15, "Vérification du compte auprès du fournisseur IPTV...");

        let (status, exp_date, max_connections) = if is_demo {
            std::thread::sleep(std::time::Duration::from_millis(500));
            ("Active".to_string(), Some("2030-12-31".to_string()), Some("5".to_string()))
        } else {
            let auth_url = format!(
                "{}/player_api.php?username={}&password={}",
                server_url,
                urlencoding(&username),
                urlencoding(&password)
            );

            let auth_output = Command::new("curl")
                .args([
                    "-s", "-L", "-k",
                    "--max-time", "15",
                    "-A", "IPTVSmarters/1.0",
                    "-H", "Accept: application/json",
                    &auth_url,
                ])
                .output();

            let auth_res = match auth_output {
                Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
                Ok(o) => return Err(format!("Échec de connexion au serveur (code {})", o.status)),
                Err(e) => return Err(format!("Erreur réseau curl : {}", e)),
            };

            let parsed_auth: serde_json::Value = serde_json::from_str(&auth_res)
                .map_err(|_| "Réponse du serveur IPTV invalide (vérifiez l'URL de votre serveur).".to_string())?;

            let user_info = parsed_auth.get("user_info").ok_or_else(|| {
                "Identifiants IPTV refusés par le serveur. Vérifiez votre nom d'utilisateur et mot de passe.".to_string()
            })?;

            let auth_flag = user_info.get("auth").and_then(|v| v.as_i64()).unwrap_or(0);
            let status_str = user_info.get("status").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();

            if auth_flag != 1 && status_str.to_lowercase() != "active" {
                let msg = user_info.get("message").and_then(|v| v.as_str()).unwrap_or("Compte inactif ou expiré.");
                return Err(format!("Échec d'authentification IPTV : {}", msg));
            }

            let exp_date = user_info.get("exp_date").and_then(|v| v.as_str()).map(|s| s.to_string());
            let max_connections = user_info.get("max_connections").and_then(|v| v.as_str()).map(|s| s.to_string());
            (status_str, exp_date, max_connections)
        };

        // Si demandé, sauvegarde ou mise à jour chiffrée du profil
        if save_profile {
            let mut profiles = load_internal_profiles();
            if let Some(pos) = profiles.iter().position(|p| p.id == p_id) {
                profiles[pos].name = p_name.clone();
                profiles[pos].server_url = server_url.clone();
                profiles[pos].username = username.clone();
                profiles[pos].password = password.clone();
                profiles[pos].last_used = now_sec;
            } else {
                profiles.push(IptvProfileInternal {
                    id: p_id.clone(),
                    name: if p_name.trim().is_empty() { format!("Compte {}", username) } else { p_name.clone() },
                    server_url: server_url.clone(),
                    username: username.clone(),
                    password: password.clone(),
                    last_used: now_sec,
                });
            }
            let _ = save_internal_profiles(&profiles);
        }

        let cache_dir = get_iptv_cache_dir();

        // Helper pour télécharger ou générer les données de streaming
        let download_json = |action: &str, target_path: &Path| -> Result<usize, String> {
            if is_demo {
                std::thread::sleep(std::time::Duration::from_millis(400));
                let (count, demo_json) = match action {
                    "get_live_categories" | "get_vod_categories" | "get_series_categories" => {
                        (3, r#"[{"category_id":"1","category_name":"Généraliste"},{"category_id":"2","category_name":"Sport & Cinéma"},{"category_id":"3","category_name":"Documentaires"}]"#)
                    }
                    "get_live_streams" => {
                        (1250, r#"[{"num":1,"name":"Noos Cinema 4K","stream_type":"live","stream_id":101,"stream_icon":"","epg_channel_id":"","category_id":"2"}]"#)
                    }
                    "get_vod_streams" => {
                        (3800, r#"[{"num":1,"name":"Film Démo 4K HDR","stream_type":"movie","stream_id":201,"stream_icon":"","rating":"8.5","category_id":"2"}]"#)
                    }
                    "get_series" => {
                        (420, r#"[{"num":1,"name":"Série Démo S01","series_id":301,"cover":"","rating":"9.1","category_id":"1"}]"#)
                    }
                    _ => (0, "[]"),
                };
                let _ = fs::write(target_path, demo_json);
                return Ok(count);
            }

            let url = format!(
                "{}/player_api.php?username={}&password={}&action={}",
                server_url,
                urlencoding(&username),
                urlencoding(&password),
                action
            );
            let status = Command::new("curl")
                .args([
                    "-s", "-L", "-k",
                    "--max-time", "45",
                    "-A", "IPTVSmarters/1.0",
                    "-H", "Accept: application/json",
                    "-o", target_path.to_str().unwrap(),
                    &url,
                ])
                .status();

            if status.is_err() || !status.unwrap().success() {
                return Err(format!("Erreur lors du téléchargement de '{}'", action));
            }

            // Décompte rapide du nombre d'éléments
            if let Ok(data) = fs::read_to_string(target_path) {
                if let Ok(arr) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(vec) = arr.as_array() {
                        return Ok(vec.len());
                    }
                }
            }
            Ok(0)
        };

        // Étape 2 : Chaînes TV en Direct & Catégories
        emit_progress(2, 5, "Chaînes Direct (Live TV)", 35, "Récupération des flux en direct et des catégories TV...");
        let live_cat_file = cache_dir.join(format!("profile_{}_live_categories.json", p_id));
        let _ = download_json("get_live_categories", &live_cat_file);

        let live_streams_file = cache_dir.join(format!("profile_{}_live_streams.json", p_id));
        let channels_count = download_json("get_live_streams", &live_streams_file).unwrap_or(0);

        // Étape 3 : Catalogue Films (VOD)
        emit_progress(3, 5, "Catalogue Films (VOD)", 60, "Mise en cache des affiches et titres de films...");
        let vod_cat_file = cache_dir.join(format!("profile_{}_vod_categories.json", p_id));
        let _ = download_json("get_vod_categories", &vod_cat_file);

        let vod_streams_file = cache_dir.join(format!("profile_{}_vod_streams.json", p_id));
        let movies_count = download_json("get_vod_streams", &vod_streams_file).unwrap_or(0);

        // Étape 4 : Catalogue Séries
        emit_progress(4, 5, "Catalogue Séries", 85, "Indexation des séries et saisons complètes...");
        let series_cat_file = cache_dir.join(format!("profile_{}_series_categories.json", p_id));
        let _ = download_json("get_series_categories", &series_cat_file);

        let series_file = cache_dir.join(format!("profile_{}_series.json", p_id));
        let series_count = download_json("get_series", &series_file).unwrap_or(0);

        // Étape 5 : Enregistrement des métadonnées du cache
        emit_progress(5, 5, "Synchronisation terminée", 100, "Le catalogue Noos IPTV est prêt.");

        let summary = IptvCacheSummary {
            has_cache: true,
            profile_id: p_id.clone(),
            last_synced: now_sec,
            channels_count,
            movies_count,
            series_count,
            status: status.to_string(),
            exp_date: exp_date.clone(),
        };

        let meta_file = cache_dir.join(format!("profile_{}_meta.json", p_id));
        if let Ok(meta_json) = serde_json::to_string_pretty(&summary) {
            let _ = fs::write(&meta_file, meta_json);
        }

        Ok(IptvSyncResult {
            success: true,
            profile_id: p_id,
            profile_name: p_name,
            message: format!(
                "Synchronisation réussie : {} chaînes, {} films et {} séries prêts.",
                channels_count, movies_count, series_count
            ),
            channels_count,
            movies_count,
            series_count,
            exp_date,
            max_connections,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

fn urlencoding(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"{\"server\":\"http://provider.tv:8080\",\"username\":\"noos_user\",\"password\":\"ultra_secret_pass_1234\"}";
        let encrypted = encrypt_profiles_payload(plaintext).expect("Chiffrement OK");
        assert_ne!(plaintext.as_slice(), encrypted.as_slice());
        assert_eq!(&encrypted[..8], MAGIC_HEADER);

        let decrypted = decrypt_profiles_payload(&encrypted).expect("Déchiffrement OK");
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }
}
