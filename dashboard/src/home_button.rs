use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use crate::commands::trigger_home_action;

const EV_KEY: u16 = 0x01;
const BTN_MODE: u16 = 0x13c;      // 316: Bouton Home / Xbox / Guide / PS / 8BitDo
const KEY_HOMEPAGE: u16 = 172;     // 172: Touche Home / Multimédia
const KEY_HOME: u16 = 102;         // 102: Touche Home clavier standard

pub fn start_home_button_listener(app: AppHandle) {
    thread::spawn(move || {
        let watched_devices = Arc::new(Mutex::new(std::collections::HashSet::<PathBuf>::new()));

        loop {
            if let Ok(entries) = std::fs::read_dir("/dev/input") {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with("event") {
                        let path = entry.path();
                        let mut set = watched_devices.lock().unwrap();
                        if !set.contains(&path) {
                            set.insert(path.clone());
                            let app_clone = app.clone();
                            let path_clone = path.clone();
                            let set_clone = Arc::clone(&watched_devices);

                            thread::spawn(move || {
                                listen_device(path_clone.clone(), app_clone);
                                if let Ok(mut set) = set_clone.lock() {
                                    set.remove(&path_clone);
                                }
                            });
                        }
                    }
                }
            }
            thread::sleep(Duration::from_secs(2));
        }
    });
}

fn listen_device(path: PathBuf, app: AppHandle) {
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let mut buf = [0u8; 24];
    while file.read_exact(&mut buf).is_ok() {
        // En décodant la structure input_event x86_64 standard:
        // octets 16..18: type (u16)
        // octets 18..20: code (u16)
        // octets 20..24: value (i32)
        let type_ = u16::from_ne_bytes([buf[16], buf[17]]);
        let code = u16::from_ne_bytes([buf[18], buf[19]]);
        let value = i32::from_ne_bytes([buf[20], buf[21], buf[22], buf[23]]);

        if type_ == EV_KEY && value == 1 {
            if code == BTN_MODE || code == KEY_HOMEPAGE || code == KEY_HOME {
                tracing::info!("Bouton HOME pressé sur {:?} (code {})", path, code);
                trigger_home_action(&app);
            }
        }
    }
}
