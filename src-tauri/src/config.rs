//! Application configuration.
//!
//! Mirrors the Electron `ConfigManager` defaults (system + user). For now this
//! provides the default config the renderer needs at boot (rpc port/secret,
//! locale, preferences). Task 6 layers persistence (tauri-plugin-store) and
//! migration of the existing Electron config on top of these defaults.

use serde_json::{json, Map, Value};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[allow(dead_code)] // used by the renderer RPC client; kept here for parity / Task 6.
pub const ENGINE_RPC_HOST: &str = "127.0.0.1";
pub const ENGINE_RPC_PORT: u32 = 16800;

/// Default system config — passed to aria2 and surfaced to the renderer.
pub fn system_defaults() -> Value {
    json!({
        "all-proxy": "",
        "allow-overwrite": false,
        "auto-file-renaming": true,
        "bt-exclude-tracker": "",
        "bt-force-encryption": false,
        "bt-load-saved-metadata": true,
        "bt-save-metadata": true,
        "bt-tracker": "",
        "continue": true,
        "dht-listen-port": 26701,
        "enable-dht6": true,
        "follow-metalink": true,
        "follow-torrent": true,
        "listen-port": 21301,
        "max-concurrent-downloads": 5,
        "max-connection-per-server": 16,
        "max-download-limit": 0,
        "max-overall-download-limit": 0,
        "max-overall-upload-limit": 0,
        "no-proxy": "",
        "pause-metadata": false,
        "pause": true,
        "rpc-listen-port": ENGINE_RPC_PORT,
        "rpc-secret": "",
        "seed-ratio": 2,
        "seed-time": 2880,
        "split": 16
    })
}

/// Default user config — application preferences.
pub fn user_defaults(locale: &str) -> Value {
    json!({
        "auto-hide-window": false,
        "auto-sync-tracker": true,
        "enable-upnp": true,
        "engine-max-connection-per-server": 16,
        "favorite-directories": [],
        "hide-app-menu": cfg!(any(target_os = "windows", target_os = "linux")),
        "history-directories": [],
        "keep-seeding": false,
        "keep-window-state": false,
        "last-sync-tracker-time": 0,
        "locale": locale,
        "log-level": "warn",
        "new-task-show-downloading": true,
        "no-confirm-before-delete-task": false,
        "open-at-login": false,
        "protocols": { "magnet": true, "thunder": false },
        "proxy": { "enable": false, "server": "", "bypass": "", "scope": [] },
        "resume-all-when-app-launched": false,
        "run-mode": "standard",
        "show-progress-bar": true,
        "task-notification": true,
        "theme": "auto",
        "tracker-source": [],
        "tray-theme": "auto",
        "tray-speedometer": cfg!(target_os = "macos"),
        "window-state": {}
    })
}

/// Merge system + user defaults into the flat object the renderer expects
/// (matches Electron's get-app-config: ...systemConfig, ...userConfig, ...context).
pub fn merged_defaults(locale: &str) -> Value {
    let mut out = system_defaults();
    let user = user_defaults(locale);
    if let (Some(o), Some(u)) = (out.as_object_mut(), user.as_object()) {
        for (k, v) in u {
            o.insert(k.clone(), v.clone());
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Persistence (electron-store-compatible: system.json + user.json in app data)
// ---------------------------------------------------------------------------

fn config_dir(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().expect("resolve app data dir");
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn store_path(app: &AppHandle, name: &str) -> PathBuf {
    config_dir(app).join(format!("{name}.json"))
}

/// Read a stored config file as an object, or empty if missing/invalid.
fn read_store(app: &AppHandle, name: &str) -> Map<String, Value> {
    let path = store_path(app, name);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

fn write_store(app: &AppHandle, name: &str, data: &Map<String, Value>) -> Result<(), String> {
    let path = store_path(app, name);
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

/// Flat merged config for the renderer: defaults <- stored system <- stored user.
pub fn merged_config(app: &AppHandle) -> Value {
    let locale = detect_locale(app);
    let mut out = merged_defaults(&locale);
    let obj = out.as_object_mut().unwrap();
    for (k, v) in read_store(app, "system") {
        obj.insert(k, v);
    }
    for (k, v) in read_store(app, "user") {
        obj.insert(k, v);
    }
    out
}

/// Locale = stored user locale if present, else system locale.
pub fn detect_locale(app: &AppHandle) -> String {
    if let Some(Value::String(l)) = read_store(app, "user").get("locale") {
        if !l.is_empty() {
            return l.clone();
        }
    }
    system_locale()
}

fn system_locale() -> String {
    std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .ok()
        .and_then(|l| l.split('.').next().map(|s| s.replace('_', "-")))
        .unwrap_or_else(|| "en-US".to_string())
}

/// Persist a `{ user, system }` payload (kebab keys) by merging into the stores.
pub fn save_preference(app: &AppHandle, payload: &Value) -> Result<(), String> {
    for name in ["user", "system"] {
        if let Some(incoming) = payload.get(name).and_then(|v| v.as_object()) {
            let mut store = read_store(app, name);
            for (k, v) in incoming {
                store.insert(k.clone(), v.clone());
            }
            write_store(app, name, &store)?;
        }
    }
    Ok(())
}

/// One-time import of the existing Electron Motrix config (~/.config/Motrix on
/// Linux) on first launch, so a user's settings carry over to the Tauri build.
/// No-op if our stores already exist or the Electron files aren't found.
pub fn migrate_from_electron(app: &AppHandle) {
    let already = store_path(app, "user").exists() || store_path(app, "system").exists();
    if already {
        return;
    }
    let Some(src) = electron_config_dir() else { return };
    let mut migrated = false;
    for name in ["system", "user"] {
        let from = src.join(format!("{name}.json"));
        if let Ok(s) = std::fs::read_to_string(&from) {
            if let Ok(v) = serde_json::from_str::<Value>(&s) {
                if let Some(obj) = v.as_object() {
                    let _ = write_store(app, name, obj);
                    migrated = true;
                }
            }
        }
    }
    if migrated {
        log::info!("[AUI] migrated existing Electron config from {:?}", src);
    }
}

/// Electron's userData dir for Motrix per-OS (electron-store location).
fn electron_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .map(|c| c.join("Motrix"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support/Motrix"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("Motrix"))
    }
}

/// Ensure a strong `rpc-secret` exists in the system store; generate one on
/// first launch. Closes the "any local process can drive aria2" risk.
pub fn ensure_rpc_secret(app: &AppHandle) {
    let mut store = read_store(app, "system");
    let needs_gen = store
        .get("rpc-secret")
        .and_then(|v| v.as_str())
        .map(|s| s.is_empty())
        .unwrap_or(true);
    if needs_gen {
        let secret = uuid::Uuid::new_v4().simple().to_string();
        store.insert("rpc-secret".to_string(), Value::String(secret));
        if let Err(e) = write_store(app, "system", &store) {
            log::warn!("[AUI] failed to persist generated rpc-secret: {}", e);
        } else {
            log::info!("[AUI] generated rpc-secret on first launch");
        }
    }
}

/// Stored system config merged over system defaults (for engine start args).
pub fn system_config(app: &AppHandle) -> Value {
    let mut out = system_defaults();
    let obj = out.as_object_mut().unwrap();
    for (k, v) in read_store(app, "system") {
        obj.insert(k, v);
    }
    out
}
