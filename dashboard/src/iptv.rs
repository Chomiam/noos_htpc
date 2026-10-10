use std::collections::HashMap;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvCategory {
    pub category_id: String,
    pub category_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvLiveStream {
    pub num: Option<u32>,
    pub name: String,
    pub stream_type: Option<String>,
    pub stream_id: u64,
    pub stream_icon: Option<String>,
    pub epg_channel_id: Option<String>,
    pub category_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvVodStream {
    pub num: Option<u32>,
    pub name: String,
    pub stream_type: Option<String>,
    pub stream_id: u64,
    pub stream_icon: Option<String>,
    pub rating: Option<String>,
    pub year: Option<String>,
    pub category_id: String,
    pub container_extension: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvSeriesItem {
    pub num: Option<u32>,
    pub name: String,
    pub series_id: u64,
    pub cover: Option<String>,
    pub rating: Option<String>,
    pub year: Option<String>,
    pub category_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvCatalogResponse {
    pub categories: Vec<IptvCategory>,
    pub live_streams: Option<Vec<IptvLiveStream>>,
    pub vod_streams: Option<Vec<IptvVodStream>>,
    pub series_streams: Option<Vec<IptvSeriesItem>>,
    pub hidden_category_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvEpgProgram {
    pub id: String,
    pub title: String,
    pub start: String,
    pub stop: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvEpisode {
    pub id: String,
    pub episode_num: u32,
    pub title: String,
    pub container_extension: String,
    pub duration: Option<String>,
    pub plot: Option<String>,
    pub season: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvSeriesDetails {
    pub series_id: u64,
    pub name: String,
    pub cover: Option<String>,
    pub backdrop: Option<String>,
    pub plot: Option<String>,
    pub cast: Option<String>,
    pub director: Option<String>,
    pub genre: Option<String>,
    pub release_date: Option<String>,
    pub rating: Option<String>,
    pub seasons: Vec<u32>,
    pub episodes: HashMap<String, Vec<IptvEpisode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvFiltersData {
    pub live_categories: Vec<IptvCategory>,
    pub vod_categories: Vec<IptvCategory>,
    pub series_categories: Vec<IptvCategory>,
    pub hidden_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvFavorite {
    pub id: String,
    pub item_type: String, // "live", "movie", "series"
    pub stream_id: u64,
    pub name: String,
    pub icon: Option<String>,
    pub category_name: Option<String>,
    pub extra: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptvPlayerSettings {
    pub upscale_profile: String,
    pub deband: bool,
    pub interpolation: bool,
    pub buffer_seconds: u32,
    pub audio_passthrough: bool,
}

impl Default for IptvPlayerSettings {
    fn default() -> Self {
        Self {
            upscale_profile: "fsr-ultra".to_string(),
            deband: true,
            interpolation: false,
            buffer_seconds: 5,
            audio_passthrough: true,
        }
    }
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
                std::thread::sleep(std::time::Duration::from_millis(300));
                let (count, demo_json) = match action {
                    "get_live_categories" => {
                        let c = get_demo_catalog("live").categories;
                        (c.len(), serde_json::to_string(&c).unwrap_or_default())
                    }
                    "get_live_streams" => {
                        let s = get_demo_catalog("live").live_streams.unwrap_or_default();
                        (s.len(), serde_json::to_string(&s).unwrap_or_default())
                    }
                    "get_vod_categories" => {
                        let c = get_demo_catalog("vod").categories;
                        (c.len(), serde_json::to_string(&c).unwrap_or_default())
                    }
                    "get_vod_streams" => {
                        let s = get_demo_catalog("vod").vod_streams.unwrap_or_default();
                        (s.len(), serde_json::to_string(&s).unwrap_or_default())
                    }
                    "get_series_categories" => {
                        let c = get_demo_catalog("series").categories;
                        (c.len(), serde_json::to_string(&c).unwrap_or_default())
                    }
                    "get_series" => {
                        let s = get_demo_catalog("series").series_streams.unwrap_or_default();
                        (s.len(), serde_json::to_string(&s).unwrap_or_default())
                    }
                    _ => (0, "[]".to_string()),
                };
                let _ = fs::write(target_path, &demo_json);
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

fn get_field_as_string(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|val| {
        if let Some(s) = val.as_str() {
            Some(s.to_string())
        } else if let Some(n) = val.as_i64() {
            Some(n.to_string())
        } else if let Some(u) = val.as_u64() {
            Some(u.to_string())
        } else {
            None
        }
    })
}

fn get_field_as_u64(v: &serde_json::Value, key: &str) -> Option<u64> {
    v.get(key).and_then(|val| {
        if let Some(u) = val.as_u64() {
            Some(u)
        } else if let Some(i) = val.as_i64() {
            Some(i as u64)
        } else if let Some(s) = val.as_str() {
            s.parse::<u64>().ok()
        } else {
            None
        }
    })
}

fn get_iptv_settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/noos".to_string());
    let dir = PathBuf::from(home).join(".config/noos-tv");
    let _ = fs::create_dir_all(&dir);
    dir.join("iptv_settings.json")
}

pub fn load_player_settings() -> IptvPlayerSettings {
    let p = get_iptv_settings_path();
    if let Ok(data) = fs::read_to_string(&p) {
        if let Ok(settings) = serde_json::from_str::<IptvPlayerSettings>(&data) {
            return settings;
        }
    }
    IptvPlayerSettings::default()
}

pub fn save_player_settings(settings: &IptvPlayerSettings) -> Result<(), String> {
    let p = get_iptv_settings_path();
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(p, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_hidden_categories(profile_id: &str) -> Vec<String> {
    let path = get_iptv_cache_dir().join(format!("profile_{}_hidden_categories.json", profile_id));
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(ids) = serde_json::from_str::<Vec<String>>(&data) {
            return ids;
        }
    }
    Vec::new()
}

fn save_hidden_categories(profile_id: &str, ids: &[String]) -> Result<(), String> {
    let path = get_iptv_cache_dir().join(format!("profile_{}_hidden_categories.json", profile_id));
    let json = serde_json::to_string_pretty(ids).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_favorites(profile_id: &str) -> Vec<IptvFavorite> {
    let path = get_iptv_cache_dir().join(format!("profile_{}_favorites.json", profile_id));
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(favs) = serde_json::from_str::<Vec<IptvFavorite>>(&data) {
            return favs;
        }
    }
    Vec::new()
}

fn save_favorites(profile_id: &str, favs: &[IptvFavorite]) -> Result<(), String> {
    let path = get_iptv_cache_dir().join(format!("profile_{}_favorites.json", profile_id));
    let json = serde_json::to_string_pretty(favs).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/* Fallbacks de démonstration riches pour tester et naviguer immédiatement */
fn get_demo_catalog(section: &str) -> IptvCatalogResponse {
    match section {
        "live" => {
            let categories = vec![
                IptvCategory { category_id: "1".into(), category_name: "TNT & Généralistes France".into() },
                IptvCategory { category_id: "2".into(), category_name: "Cinéma & Séries".into() },
                IptvCategory { category_id: "3".into(), category_name: "Sport & Événements".into() },
                IptvCategory { category_id: "4".into(), category_name: "Information 24/7".into() },
                IptvCategory { category_id: "5".into(), category_name: "Documentaires & Découverte".into() },
            ];
            let live_streams = vec![
                IptvLiveStream { num: Some(1), name: "TF1 UHD 4K HDR".into(), stream_type: Some("live".into()), stream_id: 101, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/tf1-fr.png".into()), epg_channel_id: Some("TF1.fr".into()), category_id: "1".into() },
                IptvLiveStream { num: Some(2), name: "France 2 UHD 4K".into(), stream_type: Some("live".into()), stream_id: 102, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/france-2-fr.png".into()), epg_channel_id: Some("France2.fr".into()), category_id: "1".into() },
                IptvLiveStream { num: Some(3), name: "Canal+ UHD 4K Live".into(), stream_type: Some("live".into()), stream_id: 103, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/canal-plus-fr.png".into()), epg_channel_id: Some("CanalPlus.fr".into()), category_id: "1".into() },
                IptvLiveStream { num: Some(4), name: "France 3 National HD".into(), stream_type: Some("live".into()), stream_id: 104, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/france-3-fr.png".into()), epg_channel_id: Some("France3.fr".into()), category_id: "1".into() },
                IptvLiveStream { num: Some(5), name: "M6 HDR Ultra HD".into(), stream_type: Some("live".into()), stream_id: 105, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/m6-fr.png".into()), epg_channel_id: Some("M6.fr".into()), category_id: "1".into() },
                IptvLiveStream { num: Some(6), name: "Arte Concert & Culture UHD".into(), stream_type: Some("live".into()), stream_id: 106, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/arte-fr.png".into()), epg_channel_id: Some("Arte.fr".into()), category_id: "1".into() },
                IptvLiveStream { num: Some(7), name: "Canal+ Cinéma 4K".into(), stream_type: Some("live".into()), stream_id: 201, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/canal-plus-cinema-fr.png".into()), epg_channel_id: Some("CanalCinema.fr".into()), category_id: "2".into() },
                IptvLiveStream { num: Some(8), name: "Ciné+ Premier 4K".into(), stream_type: Some("live".into()), stream_id: 202, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/cine-plus-premier-fr.png".into()), epg_channel_id: Some("CinePremier.fr".into()), category_id: "2".into() },
                IptvLiveStream { num: Some(9), name: "beIN Sports 1 4K UHD".into(), stream_type: Some("live".into()), stream_id: 301, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/bein-sports-1-fr.png".into()), epg_channel_id: Some("Bein1.fr".into()), category_id: "3".into() },
                IptvLiveStream { num: Some(10), name: "beIN Sports 2 HD".into(), stream_type: Some("live".into()), stream_id: 302, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/bein-sports-2-fr.png".into()), epg_channel_id: Some("Bein2.fr".into()), category_id: "3".into() },
                IptvLiveStream { num: Some(11), name: "Canal+ Sport 360".into(), stream_type: Some("live".into()), stream_id: 303, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/canal-plus-sport-360-fr.png".into()), epg_channel_id: Some("CanalSport.fr".into()), category_id: "3".into() },
                IptvLiveStream { num: Some(12), name: "franceinfo: 4K Direct".into(), stream_type: Some("live".into()), stream_id: 401, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/franceinfo-fr.png".into()), epg_channel_id: Some("FranceInfo.fr".into()), category_id: "4".into() },
                IptvLiveStream { num: Some(13), name: "National Geographic UHD".into(), stream_type: Some("live".into()), stream_id: 501, stream_icon: Some("https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/national-geographic-fr.png".into()), epg_channel_id: Some("NatGeo.fr".into()), category_id: "5".into() },
            ];
            IptvCatalogResponse {
                categories,
                live_streams: Some(live_streams),
                vod_streams: None,
                series_streams: None,
                hidden_category_ids: Vec::new(),
            }
        }
        "vod" => {
            let categories = vec![
                IptvCategory { category_id: "10".into(), category_name: "Films 4K HDR".into() },
                IptvCategory { category_id: "11".into(), category_name: "Action & Aventure".into() },
                IptvCategory { category_id: "12".into(), category_name: "Science-Fiction".into() },
                IptvCategory { category_id: "13".into(), category_name: "Animation & Famille".into() },
            ];
            let vod_streams = vec![
                IptvVodStream { num: Some(1), name: "Dune : Deuxième Partie".into(), stream_type: Some("movie".into()), stream_id: 1001, stream_icon: Some("https://image.tmdb.org/t/p/w500/1pdfLvkbY9ohJlCjQH2CZjjYVvJ.jpg".into()), rating: Some("8.6".into()), year: Some("2024".into()), category_id: "10".into(), container_extension: Some("mkv".into()) },
                IptvVodStream { num: Some(2), name: "Oppenheimer Ultra HD".into(), stream_type: Some("movie".into()), stream_id: 1002, stream_icon: Some("https://image.tmdb.org/t/p/w500/8Gxv8gSFCU0XGDykEGv7zR1n2ua.jpg".into()), rating: Some("8.9".into()), year: Some("2023".into()), category_id: "10".into(), container_extension: Some("mkv".into()) },
                IptvVodStream { num: Some(3), name: "Avatar : La Voie de l'Eau".into(), stream_type: Some("movie".into()), stream_id: 1003, stream_icon: Some("https://image.tmdb.org/t/p/w500/t6HIqrRAclMCA60NsSmeqe9RmNV.jpg".into()), rating: Some("7.8".into()), year: Some("2022".into()), category_id: "10".into(), container_extension: Some("mkv".into()) },
                IptvVodStream { num: Some(4), name: "Top Gun : Maverick".into(), stream_type: Some("movie".into()), stream_id: 1004, stream_icon: Some("https://image.tmdb.org/t/p/w500/62HCnUTziyWcpDaBO2i1DX17ljH.jpg".into()), rating: Some("8.3".into()), year: Some("2022".into()), category_id: "11".into(), container_extension: Some("mkv".into()) },
                IptvVodStream { num: Some(5), name: "Interstellar 4K HDR".into(), stream_type: Some("movie".into()), stream_id: 1005, stream_icon: Some("https://image.tmdb.org/t/p/w500/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg".into()), rating: Some("8.7".into()), year: Some("2014".into()), category_id: "12".into(), container_extension: Some("mkv".into()) },
                IptvVodStream { num: Some(6), name: "Blade Runner 2049".into(), stream_type: Some("movie".into()), stream_id: 1006, stream_icon: Some("https://image.tmdb.org/t/p/w500/gajva2L0rPYkEWjzgFlBXCAVBE5.jpg".into()), rating: Some("8.0".into()), year: Some("2017".into()), category_id: "12".into(), container_extension: Some("mkv".into()) },
                IptvVodStream { num: Some(7), name: "Spider-Man : Across the Spider-Verse".into(), stream_type: Some("movie".into()), stream_id: 1007, stream_icon: Some("https://image.tmdb.org/t/p/w500/8Vt6mWEReuy4Of61Lnj5Xj704m8.jpg".into()), rating: Some("8.7".into()), year: Some("2023".into()), category_id: "13".into(), container_extension: Some("mkv".into()) },
                IptvVodStream { num: Some(8), name: "Le Comte de Monte-Cristo".into(), stream_type: Some("movie".into()), stream_id: 1008, stream_icon: Some("https://image.tmdb.org/t/p/w500/zw4OLmzFg0P12Q8xWf2G4l2o2t3.jpg".into()), rating: Some("8.2".into()), year: Some("2024".into()), category_id: "11".into(), container_extension: Some("mkv".into()) },
            ];
            IptvCatalogResponse {
                categories,
                live_streams: None,
                vod_streams: Some(vod_streams),
                series_streams: None,
                hidden_category_ids: Vec::new(),
            }
        }
        "series" => {
            let categories = vec![
                IptvCategory { category_id: "20".into(), category_name: "Séries 4K HDR".into() },
                IptvCategory { category_id: "21".into(), category_name: "Drame & Mystère".into() },
                IptvCategory { category_id: "22".into(), category_name: "Science-Fiction & Fantastique".into() },
            ];
            let series_streams = vec![
                IptvSeriesItem { num: Some(1), name: "Fallout".into(), series_id: 2001, cover: Some("https://image.tmdb.org/t/p/w500/AnsZu4h0wYwJ38e7s5pP6G2xQ2z.jpg".into()), rating: Some("8.4".into()), year: Some("2024".into()), category_id: "20".into() },
                IptvSeriesItem { num: Some(2), name: "The Last of Us".into(), series_id: 2002, cover: Some("https://image.tmdb.org/t/p/w500/uKvVjHNqB5VmOrdxqMisSYaq9e3.jpg".into()), rating: Some("8.8".into()), year: Some("2023".into()), category_id: "20".into() },
                IptvSeriesItem { num: Some(3), name: "House of the Dragon".into(), series_id: 2003, cover: Some("https://image.tmdb.org/t/p/w500/1X4h40fcB4WWUmIBK0auT4zRBAV.jpg".into()), rating: Some("8.5".into()), year: Some("2024".into()), category_id: "20".into() },
                IptvSeriesItem { num: Some(4), name: "Shōgun".into(), series_id: 2004, cover: Some("https://image.tmdb.org/t/p/w500/7O4iVfOMQmdCSxhOg1WNzG1AgYT.jpg".into()), rating: Some("8.7".into()), year: Some("2024".into()), category_id: "21".into() },
                IptvSeriesItem { num: Some(5), name: "Severance".into(), series_id: 2005, cover: Some("https://image.tmdb.org/t/p/w500/p1cu0gS84yvQkQ1uN2f3E4z6L1u.jpg".into()), rating: Some("8.7".into()), year: Some("2022".into()), category_id: "22".into() },
                IptvSeriesItem { num: Some(6), name: "Stranger Things".into(), series_id: 2006, cover: Some("https://image.tmdb.org/t/p/w500/49WJfeN0moxb9IPfGn8AIqMGskD.jpg".into()), rating: Some("8.7".into()), year: Some("2022".into()), category_id: "22".into() },
            ];
            IptvCatalogResponse {
                categories,
                live_streams: None,
                vod_streams: None,
                series_streams: Some(series_streams),
                hidden_category_ids: Vec::new(),
            }
        }
        _ => IptvCatalogResponse {
            categories: Vec::new(),
            live_streams: None,
            vod_streams: None,
            series_streams: None,
            hidden_category_ids: Vec::new(),
        },
    }
}

#[tauri::command]
pub async fn iptv_get_catalog(profile_id: String, section: String) -> Result<IptvCatalogResponse, String> {
    tokio::task::spawn_blocking(move || {
        let cache_dir = get_iptv_cache_dir();
        let hidden = load_hidden_categories(&profile_id);

        let cat_file = match section.as_str() {
            "live" => cache_dir.join(format!("profile_{}_live_categories.json", profile_id)),
            "vod" => cache_dir.join(format!("profile_{}_vod_categories.json", profile_id)),
            "series" => cache_dir.join(format!("profile_{}_series_categories.json", profile_id)),
            _ => return Err("Section IPTV inconnue.".to_string()),
        };

        let streams_file = match section.as_str() {
            "live" => cache_dir.join(format!("profile_{}_live_streams.json", profile_id)),
            "vod" => cache_dir.join(format!("profile_{}_vod_streams.json", profile_id)),
            "series" => cache_dir.join(format!("profile_{}_series.json", profile_id)),
            _ => return Err("Section IPTV inconnue.".to_string()),
        };

        let raw_cat = fs::read_to_string(&cat_file).unwrap_or_default();
        let raw_streams = fs::read_to_string(&streams_file).unwrap_or_default();

        if raw_cat.trim().is_empty() || raw_streams.trim().is_empty() {
            let mut demo = get_demo_catalog(&section);
            demo.hidden_category_ids = hidden;
            return Ok(demo);
        }

        let cat_values = serde_json::from_str::<Vec<serde_json::Value>>(&raw_cat).unwrap_or_default();
        let categories: Vec<IptvCategory> = cat_values
            .into_iter()
            .filter_map(|v| {
                let cat_id = get_field_as_string(&v, "category_id")?;
                let cat_name = get_field_as_string(&v, "category_name").unwrap_or_else(|| "Général".into());
                Some(IptvCategory { category_id: cat_id, category_name: cat_name })
            })
            .collect();

        if categories.is_empty() {
            let mut demo = get_demo_catalog(&section);
            demo.hidden_category_ids = hidden;
            return Ok(demo);
        }


        match section.as_str() {
            "live" => {
                let st_values = serde_json::from_str::<Vec<serde_json::Value>>(&raw_streams).unwrap_or_default();
                let live_streams: Vec<IptvLiveStream> = st_values
                    .into_iter()
                    .filter_map(|v| {
                        let stream_id = get_field_as_u64(&v, "stream_id")?;
                        let name = get_field_as_string(&v, "name").unwrap_or_else(|| "Chaîne Direct".into());
                        let category_id = get_field_as_string(&v, "category_id").unwrap_or_else(|| "1".into());
                        let stream_icon = get_field_as_string(&v, "stream_icon");
                        let epg_channel_id = get_field_as_string(&v, "epg_channel_id");
                        let num = get_field_as_u64(&v, "num").map(|n| n as u32);
                        Some(IptvLiveStream {
                            num,
                            name,
                            stream_type: Some("live".into()),
                            stream_id,
                            stream_icon,
                            epg_channel_id,
                            category_id,
                        })
                    })
                    .collect();

                Ok(IptvCatalogResponse {
                    categories,
                    live_streams: Some(live_streams),
                    vod_streams: None,
                    series_streams: None,
                    hidden_category_ids: hidden,
                })
            }
            "vod" => {
                let st_values = serde_json::from_str::<Vec<serde_json::Value>>(&raw_streams).unwrap_or_default();
                let vod_streams: Vec<IptvVodStream> = st_values
                    .into_iter()
                    .filter_map(|v| {
                        let stream_id = get_field_as_u64(&v, "stream_id")?;
                        let name = get_field_as_string(&v, "name").unwrap_or_else(|| "Film".into());
                        let category_id = get_field_as_string(&v, "category_id").unwrap_or_else(|| "1".into());
                        let stream_icon = get_field_as_string(&v, "stream_icon")
                            .or_else(|| get_field_as_string(&v, "cover"))
                            .or_else(|| get_field_as_string(&v, "movie_image"));
                        let rating = get_field_as_string(&v, "rating");
                        let year = get_field_as_string(&v, "year");
                        let container_extension = get_field_as_string(&v, "container_extension").or_else(|| Some("mkv".into()));
                        let num = get_field_as_u64(&v, "num").map(|n| n as u32);
                        Some(IptvVodStream {
                            num,
                            name,
                            stream_type: Some("movie".into()),
                            stream_id,
                            stream_icon,
                            rating,
                            year,
                            category_id,
                            container_extension,
                        })
                    })
                    .collect();

                Ok(IptvCatalogResponse {
                    categories,
                    live_streams: None,
                    vod_streams: Some(vod_streams),
                    series_streams: None,
                    hidden_category_ids: hidden,
                })
            }
            "series" => {
                let st_values = serde_json::from_str::<Vec<serde_json::Value>>(&raw_streams).unwrap_or_default();
                let series_streams: Vec<IptvSeriesItem> = st_values
                    .into_iter()
                    .filter_map(|v| {
                        let series_id = get_field_as_u64(&v, "series_id")?;
                        let name = get_field_as_string(&v, "name").unwrap_or_else(|| "Série".into());
                        let category_id = get_field_as_string(&v, "category_id").unwrap_or_else(|| "1".into());
                        let cover = get_field_as_string(&v, "cover")
                            .or_else(|| get_field_as_string(&v, "stream_icon"))
                            .or_else(|| get_field_as_string(&v, "series_icon"));
                        let rating = get_field_as_string(&v, "rating");
                        let year = get_field_as_string(&v, "year");
                        let num = get_field_as_u64(&v, "num").map(|n| n as u32);
                        Some(IptvSeriesItem {
                            num,
                            name,
                            series_id,
                            cover,
                            rating,
                            year,
                            category_id,
                        })
                    })
                    .collect();

                Ok(IptvCatalogResponse {
                    categories,
                    live_streams: None,
                    vod_streams: None,
                    series_streams: Some(series_streams),
                    hidden_category_ids: hidden,
                })
            }
            _ => Err("Section non gérée.".into()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_get_channel_epg(profile_id: String, stream_id: u64) -> Result<Vec<IptvEpgProgram>, String> {
    tokio::task::spawn_blocking(move || {
        let profiles = load_internal_profiles();
        let found = profiles.into_iter().find(|p| p.id == profile_id);

        if let Some(p) = found {
            if !p.server_url.contains("demo") && p.username.to_lowercase() != "demo" {
                let url = format!(
                    "{}/player_api.php?username={}&password={}&action=get_short_epg&stream_id={}&limit=6",
                    clean_server_url(&p.server_url),
                    urlencoding(&p.username),
                    urlencoding(&p.password),
                    stream_id
                );

                if let Ok(output) = Command::new("curl")
                    .args(["-s", "-L", "-k", "--max-time", "8", "-A", "IPTVSmarters/1.0", &url])
                    .output()
                {
                    if output.status.success() {
                        let text = String::from_utf8_lossy(&output.stdout);
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(listings) = val.get("epg_listings").and_then(|l| l.as_array()) {
                                let mut results = Vec::new();
                                for item in listings {
                                    let title_raw = get_field_as_string(item, "title").unwrap_or_default();
                                    // Décodage base64 si présent
                                    let title = if let Ok(dec) = base64_simple_decode(&title_raw) {
                                        dec
                                    } else {
                                        title_raw
                                    };
                                    let desc_raw = get_field_as_string(item, "descr").or_else(|| get_field_as_string(item, "description")).unwrap_or_default();
                                    let desc = if let Ok(dec) = base64_simple_decode(&desc_raw) {
                                        dec
                                    } else {
                                        desc_raw
                                    };
                                    let start = get_field_as_string(item, "start").unwrap_or_default();
                                    let stop = get_field_as_string(item, "stop").unwrap_or_default();
                                    let id = get_field_as_string(item, "id").unwrap_or_else(|| format!("{}", stream_id));
                                    results.push(IptvEpgProgram { id, title, start, stop, description: desc });
                                }
                                if !results.is_empty() {
                                    return Ok(results);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Guide TV démonstration fluide & réaliste
        let epg_demo = vec![
            IptvEpgProgram {
                id: "epg1".into(),
                title: "Programme en cours • Diffusion Direct 4K".into(),
                start: "20:50".into(),
                stop: "22:45".into(),
                description: "Retrouvez votre émission en direct avec traitement sonore cinéma, sous-titres et qualité 4K Ultra HD.".into(),
            },
            IptvEpgProgram {
                id: "epg2".into(),
                title: "Journal Télévisé & Édition Spéciale".into(),
                start: "22:45".into(),
                stop: "23:30".into(),
                description: "Le tour complet de l'actualité nationale et internationale en direct avec nos envoyés spéciaux.".into(),
            },
            IptvEpgProgram {
                id: "epg3".into(),
                title: "Grand Documentaire Découverte".into(),
                start: "23:30".into(),
                stop: "01:00".into(),
                description: "Une immersion inédite au cœur des espaces naturels les plus spectaculaires de notre planète.".into(),
            },
        ];
        Ok(epg_demo)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn base64_simple_decode(input: &str) -> Result<String, ()> {
    // Si la chaîne n'est pas en base64 valide ou vide, retourne Err
    let clean = input.trim();
    if clean.is_empty() || clean.len() % 4 != 0 {
        return Err(());
    }
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buffer = 0u32;
    let mut bits = 0;
    let mut bytes = Vec::new();

    for &b in clean.as_bytes() {
        if b == b'=' {
            break;
        }
        let pos = chars.find(b as char).ok_or(())? as u32;
        buffer = (buffer << 6) | pos;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    String::from_utf8(bytes).map_err(|_| ())
}

#[tauri::command]
pub async fn iptv_get_series_details(profile_id: String, series_id: u64) -> Result<IptvSeriesDetails, String> {
    tokio::task::spawn_blocking(move || {
        let profiles = load_internal_profiles();
        let found = profiles.into_iter().find(|p| p.id == profile_id);

        if let Some(p) = found {
            if !p.server_url.contains("demo") && p.username.to_lowercase() != "demo" {
                let url = format!(
                    "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
                    clean_server_url(&p.server_url),
                    urlencoding(&p.username),
                    urlencoding(&p.password),
                    series_id
                );

                if let Ok(output) = Command::new("curl")
                    .args(["-s", "-L", "-k", "--max-time", "15", "-A", "IPTVSmarters/1.0", &url])
                    .output()
                {
                    if output.status.success() {
                        let text = String::from_utf8_lossy(&output.stdout);
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            let info = val.get("info").unwrap_or(&serde_json::Value::Null);
                            let name = get_field_as_string(info, "name").unwrap_or_else(|| "Série".into());
                            let cover = get_field_as_string(info, "cover");
                            let backdrop = get_field_as_string(info, "backdrop_path")
                                .and_then(|b| if b.is_empty() { None } else { Some(b) });
                            let plot = get_field_as_string(info, "plot").or_else(|| get_field_as_string(info, "description"));
                            let cast = get_field_as_string(info, "cast");
                            let director = get_field_as_string(info, "director");
                            let genre = get_field_as_string(info, "genre");
                            let release_date = get_field_as_string(info, "releaseDate").or_else(|| get_field_as_string(info, "year"));
                            let rating = get_field_as_string(info, "rating");

                            let mut episodes_map: HashMap<String, Vec<IptvEpisode>> = HashMap::new();
                            let mut seasons_set = std::collections::BTreeSet::new();

                            if let Some(ep_obj) = val.get("episodes").and_then(|e| e.as_object()) {
                                for (season_str, ep_array) in ep_obj {
                                    if let Some(arr) = ep_array.as_array() {
                                        let season_num = season_str.parse::<u32>().unwrap_or(1);
                                        seasons_set.insert(season_num);
                                        let mut eps = Vec::new();
                                        for ep in arr {
                                            let id = get_field_as_string(ep, "id").unwrap_or_default();
                                            let ep_num = get_field_as_u64(ep, "episode_num").unwrap_or(1) as u32;
                                            let title = get_field_as_string(ep, "title").unwrap_or_else(|| format!("Épisode {}", ep_num));
                                            let container_ext = get_field_as_string(ep, "container_extension").unwrap_or_else(|| "mp4".into());
                                            let ep_info = ep.get("info").unwrap_or(&serde_json::Value::Null);
                                            let plot = get_field_as_string(ep_info, "plot").or_else(|| get_field_as_string(ep, "plot"));
                                            let duration = get_field_as_string(ep_info, "duration").or_else(|| get_field_as_string(ep, "duration"));
                                            eps.push(IptvEpisode {
                                                id,
                                                episode_num: ep_num,
                                                title,
                                                container_extension: container_ext,
                                                duration,
                                                plot,
                                                season: season_num,
                                            });
                                        }
                                        episodes_map.insert(season_str.clone(), eps);
                                    }
                                }
                            }

                            let seasons: Vec<u32> = if seasons_set.is_empty() { vec![1] } else { seasons_set.into_iter().collect() };

                            return Ok(IptvSeriesDetails {
                                series_id,
                                name,
                                cover,
                                backdrop,
                                plot,
                                cast,
                                director,
                                genre,
                                release_date,
                                rating,
                                seasons,
                                episodes: episodes_map,
                            });
                        }
                    }
                }
            }
        }

        // Données complètes de démonstration pour séries
        let mut ep_map = HashMap::new();
        let s1_eps = vec![
            IptvEpisode {
                id: "ep_1".into(),
                episode_num: 1,
                title: "Épisode 1 • Les Débuts".into(),
                container_extension: "mp4".into(),
                duration: Some("58 min".into()),
                plot: Some("Un événement inattendu bouleverse l'équilibre établi et force les protagonistes à franchir un nouveau cap.".into()),
                season: 1,
            },
            IptvEpisode {
                id: "ep_2".into(),
                episode_num: 2,
                title: "Épisode 2 • La Traque".into(),
                container_extension: "mp4".into(),
                duration: Some("54 min".into()),
                plot: Some("Alors que les indices s'accumulent, une alliance improbable se forme pour contrer une menace grandissante.".into()),
                season: 1,
            },
            IptvEpisode {
                id: "ep_3".into(),
                episode_num: 3,
                title: "Épisode 3 • Révélations".into(),
                container_extension: "mp4".into(),
                duration: Some("62 min".into()),
                plot: Some("Les vérités enfouies refont surface au pire moment, provoquant une confrontation sous haute tension.".into()),
                season: 1,
            },
        ];
        ep_map.insert("1".into(), s1_eps);

        Ok(IptvSeriesDetails {
            series_id,
            name: "Série Démo 4K".into(),
            cover: Some("https://image.tmdb.org/t/p/w500/AnsZu4h0wYwJ38e7s5pP6G2xQ2z.jpg".into()),
            backdrop: Some("https://image.tmdb.org/t/p/original/4HodYYKEIsGOdinkGi2Ucz6X9i0.jpg".into()),
            plot: Some("Dans un univers post-apocalyptique fascinant et impitoyable, une habitante d'un abri souterrain d'élite est forcée d'explorer le monde extérieur pour sauver les siens.".into()),
            cast: Some("Ella Purnell, Aaron Moten, Walton Goggins".into()),
            director: Some("Jonathan Nolan".into()),
            genre: Some("Science-Fiction • Action • Aventure".into()),
            release_date: Some("2024".into()),
            rating: Some("8.5".into()),
            seasons: vec![1],
            episodes: ep_map,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_get_filters_data(profile_id: String) -> Result<IptvFiltersData, String> {
    tokio::task::spawn_blocking(move || {
        let cache_dir = get_iptv_cache_dir();
        let hidden = load_hidden_categories(&profile_id);

        let read_cats = |fname: &str, default_cats: Vec<IptvCategory>| -> Vec<IptvCategory> {
            let path = cache_dir.join(format!("profile_{}_{}.json", profile_id, fname));
            if let Ok(raw) = fs::read_to_string(path) {
                if let Ok(cat_values) = serde_json::from_str::<Vec<serde_json::Value>>(&raw) {
                    let parsed: Vec<IptvCategory> = cat_values
                        .into_iter()
                        .filter_map(|v| {
                            let cat_id = get_field_as_string(&v, "category_id")?;
                            let cat_name = get_field_as_string(&v, "category_name").unwrap_or_else(|| "Général".into());
                            Some(IptvCategory { category_id: cat_id, category_name: cat_name })
                        })
                        .collect();
                    if !parsed.is_empty() {
                        return parsed;
                    }
                }
            }
            default_cats
        };

        let live = read_cats("live_categories", get_demo_catalog("live").categories);
        let vod = read_cats("vod_categories", get_demo_catalog("vod").categories);
        let series = read_cats("series_categories", get_demo_catalog("series").categories);

        Ok(IptvFiltersData {
            live_categories: live,
            vod_categories: vod,
            series_categories: series,
            hidden_ids: hidden,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_save_hidden_categories(profile_id: String, hidden_ids: Vec<String>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        save_hidden_categories(&profile_id, &hidden_ids)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_get_favorites(profile_id: String) -> Result<Vec<IptvFavorite>, String> {
    tokio::task::spawn_blocking(move || {
        Ok(load_favorites(&profile_id))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_toggle_favorite(profile_id: String, item: IptvFavorite) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let mut favs = load_favorites(&profile_id);
        let pos = favs.iter().position(|f| f.stream_id == item.stream_id && f.item_type == item.item_type);
        let is_now_fav = if let Some(idx) = pos {
            favs.remove(idx);
            false
        } else {
            favs.push(item);
            true
        };
        save_favorites(&profile_id, &favs)?;
        Ok(is_now_fav)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_play_stream(
    profile_id: String,
    item_type: String,
    stream_id: u64,
    extension: Option<String>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let profiles = load_internal_profiles();
        let p = profiles.into_iter().find(|x| x.id == profile_id);

        let (server_url, username, password) = if let Some(found) = p {
            (found.server_url, found.username, found.password)
        } else {
            ("http://demo.noos.tv:8080".to_string(), "demo".to_string(), "demo".to_string())
        };

        let ext = extension.unwrap_or_else(|| {
            if item_type == "live" { "ts".to_string() } else { "mp4".to_string() }
        });

        let stream_url = if server_url.contains("demo") || username == "demo" {
            if item_type == "live" {
                "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8".to_string()
            } else if item_type == "movie" {
                "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4".to_string()
            } else {
                "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/TearsOfSteel.mp4".to_string()
            }
        } else {
            let clean_url = clean_server_url(&server_url);
            match item_type.as_str() {
                "live" => format!("{}/live/{}/{}/{}.{}", clean_url, username, password, stream_id, ext),
                "movie" => format!("{}/movie/{}/{}/{}.{}", clean_url, username, password, stream_id, ext),
                "series" => format!("{}/series/{}/{}/{}.{}", clean_url, username, password, stream_id, ext),
                _ => format!("{}/live/{}/{}/{}.{}", clean_url, username, password, stream_id, ext),
            }
        };

        // Arrêter toute instance MPV existante pour une transition propre
        let _ = Command::new("pkill").args(["-TERM", "-f", "mpv"]).status();

        let settings = load_player_settings();
        let mut mpv_cmd = Command::new("mpv");
        mpv_cmd.args([
            "--fs",
            "--border=no",
            "--osc=no",
            "--osd-bar=no",
            "--vo=gpu-next",
            "--gpu-context=wayland",
            "--hwdec=auto-safe",
            "--force-window=yes",
            "--keep-open=no",
            "--title=Noos IPTV MPV Player",
            &format!("--cache-secs={}", settings.buffer_seconds),
            &stream_url,
        ]);

        if settings.deband {
            mpv_cmd.args(["--deband=yes", "--deband-iterations=4", "--deband-threshold=48"]);
        }
        if settings.interpolation {
            mpv_cmd.args(["--interpolation=yes", "--video-sync=display-resample"]);
        }
        if settings.audio_passthrough {
            mpv_cmd.args([
                "--ao=pipewire,alsa,pulse",
                "--audio-spdif=ac3,dts,dts-hd,eac3,truehd",
                "--audio-channels=auto",
                "--audio-pitch-correction=no",
            ]);
            if !settings.interpolation {
                mpv_cmd.arg("--video-sync=audio");
            }
        }

        let child = mpv_cmd.spawn()
            .map_err(|e| format!("Erreur lors du lancement de MPV : {}", e))?;

        Ok(format!("Lecture IPTV MPV lancée avec succès (PID {})", child.id()))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_get_player_settings() -> Result<IptvPlayerSettings, String> {
    tokio::task::spawn_blocking(move || {
        Ok(load_player_settings())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn iptv_save_player_settings(settings: IptvPlayerSettings) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        save_player_settings(&settings)?;
        // Applique également le profil d'upscale mpv dans ~/.config/mpv/mpv.conf si configuré
        let upscale_profiles = crate::commands::get_all_upscale_profiles("4K (3840x2160)");
        if let Some(prof) = upscale_profiles.into_iter().find(|p| p.id == settings.upscale_profile) {
            crate::commands::apply_mpv_profile(&prof, "4K (3840x2160)");
        }
        Ok(())
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

    #[tokio::test]
    async fn test_iptv_get_catalog_demo() {
        let res = iptv_get_catalog("demo".to_string(), "live".to_string()).await.unwrap();
        println!("Categories: {:?}", res.categories);
        println!("Live streams count: {:?}", res.live_streams.as_ref().map(|s| s.len()));
        assert!(!res.categories.is_empty());
    }

    #[test]
    fn test_parse_vm_cache() {
        let raw_cat = r#"[{"category_id":"1","category_name":"Généraliste"},{"category_id":"2","category_name":"Sport & Cinéma"},{"category_id":"3","category_name":"Documentaires"}]"#;
        let raw_streams = r#"[{"num":1,"name":"Noos Cinema 4K","stream_type":"live","stream_id":101,"stream_icon":"","epg_channel_id":"","category_id":"2"}]"#;

        let cat_values = serde_json::from_str::<Vec<serde_json::Value>>(raw_cat).unwrap();
        let categories: Vec<IptvCategory> = cat_values
            .into_iter()
            .filter_map(|v| {
                let cat_id = get_field_as_string(&v, "category_id")?;
                let cat_name = get_field_as_string(&v, "category_name").unwrap_or_else(|| "Général".into());
                Some(IptvCategory { category_id: cat_id, category_name: cat_name })
            })
            .collect();
        println!("Parsed categories: {:?}", categories);
        assert_eq!(categories.len(), 3);

        let st_values = serde_json::from_str::<Vec<serde_json::Value>>(raw_streams).unwrap();
        let live_streams: Vec<IptvLiveStream> = st_values
            .into_iter()
            .filter_map(|v| {
                let stream_id = get_field_as_u64(&v, "stream_id")?;
                let name = get_field_as_string(&v, "name").unwrap_or_else(|| "Chaîne Direct".into());
                let category_id = get_field_as_string(&v, "category_id").unwrap_or_else(|| "1".into());
                let stream_icon = get_field_as_string(&v, "stream_icon");
                let epg_channel_id = get_field_as_string(&v, "epg_channel_id");
                let num = get_field_as_u64(&v, "num").map(|n| n as u32);
                Some(IptvLiveStream {
                    num,
                    name,
                    stream_type: Some("live".into()),
                    stream_id,
                    stream_icon,
                    epg_channel_id,
                    category_id,
                })
            })
            .collect();
        println!("Parsed live streams: {:?}", live_streams);
        assert_eq!(live_streams.len(), 1);
    }

    #[tokio::test]
    async fn test_iptv_get_catalog_func() {
        let res = iptv_get_catalog("demo".to_string(), "live".to_string()).await;
        println!("Result: {:?}", res);
        assert!(res.is_ok());
        let cat = res.unwrap();
        assert_eq!(cat.categories.len(), 5);
        assert!(cat.live_streams.is_some());
    }
}


