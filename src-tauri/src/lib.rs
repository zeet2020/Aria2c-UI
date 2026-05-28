//! Aria2 UI — Tauri application core (orchestrator).

mod config;
mod engine;
mod protocol;
mod tray;

use serde_json::Value;
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;

use engine::EngineState;

/// Apply the `open-at-login` preference to the OS autostart state.
fn sync_autostart(app: &tauri::AppHandle, enabled: bool) {
    let mgr = app.autolaunch();
    let is_on = mgr.is_enabled().unwrap_or(false);
    if enabled && !is_on {
        let _ = mgr.enable();
    } else if !enabled && is_on {
        let _ = mgr.disable();
    }
}

/// Renderer boot config (Electron's `get-app-config`): defaults overlaid with
/// the persisted system + user config (kebab keys, flat).
#[tauri::command]
fn get_app_config(app: tauri::AppHandle) -> Value {
    config::merged_config(&app)
}

/// Tauri framework version (for the About dialog).
#[tauri::command]
fn get_tauri_version() -> &'static str {
    tauri::VERSION
}

/// Generic command channel (Electron ipcRenderer.send('command', cmd, ...args)).
/// Routes `application:*` commands.
#[tauri::command]
fn dispatch_command(app: tauri::AppHandle, command: String, args: Vec<Value>) {
    log::info!("[AUI] command: {} {:?}", command, args);
    match command.as_str() {
        "application:save-preference" => {
            if let Some(payload) = args.first() {
                if let Err(e) = config::save_preference(&app, payload) {
                    log::error!("[AUI] save-preference failed: {}", e);
                }
                // Apply autostart if the user toggled open-at-login.
                if let Some(v) = payload
                    .get("user")
                    .and_then(|u| u.get("open-at-login"))
                    .and_then(|v| v.as_bool())
                {
                    sync_autostart(&app, v);
                }
            }
        }
        "application:open-external" => {
            if let Some(url) = args.first().and_then(|v| v.as_str()) {
                // Allowlist safe browser schemes. Reject file:, javascript:,
                // vbscript:, data:, etc. — they'd open the OS handler with
                // attacker-controlled content.
                let lower = url.to_lowercase();
                let safe = lower.starts_with("http://")
                    || lower.starts_with("https://")
                    || lower.starts_with("mailto:");
                if safe {
                    let _ = app.opener().open_url(url, None::<&str>);
                } else {
                    log::warn!("[AUI] open-external rejected unsafe URL: {}", url);
                }
            }
        }
        "application:relaunch" => {
            app.restart();
        }
        "application:exit" | "application:quit" => {
            app.exit(0);
        }
        // factory-reset, reset-session, update-tray* -> later.
        other => {
            log::debug!("[AUI] unhandled command (stub): {}", other);
        }
    }
}

/// Generic event channel (Electron ipcRenderer.send('event', name, ...args)).
/// Used for speed/status/progress -> tray & dock updates (future).
#[tauri::command]
fn dispatch_event(name: String, _args: Vec<Value>) {
    log::trace!("[AUI] event: {}", name);
}

/// Allowed roots for `trash_item` — downloads dir, app data dir, configured
/// download dir. Canonicalised so `..` traversal can't escape.
fn trash_allowed_roots(app: &tauri::AppHandle) -> Vec<std::path::PathBuf> {
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(d) = app.path().download_dir() {
        if let Ok(c) = d.canonicalize() { roots.push(c); }
    }
    if let Ok(d) = app.path().app_data_dir() {
        if let Ok(c) = d.canonicalize() { roots.push(c); }
    }
    if let Some(dir) = config::system_config(app)
        .get("dir")
        .and_then(|v| v.as_str())
    {
        if let Ok(c) = std::path::PathBuf::from(dir).canonicalize() {
            roots.push(c);
        }
    }
    roots
}

/// Move a file to the OS trash (Electron shell.trashItem). Rejects paths
/// outside the allowed roots so a compromised renderer can't delete arbitrary
/// files on the host. No-op if missing.
#[tauri::command]
fn trash_item(app: tauri::AppHandle, path: String) -> bool {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return true;
    }
    let target = match p.canonicalize() {
        Ok(t) => t,
        Err(e) => {
            log::warn!("[AUI] trash_item canonicalize failed for {}: {}", path, e);
            return false;
        }
    };
    let roots = trash_allowed_roots(&app);
    if !roots.iter().any(|r| target.starts_with(r)) {
        log::warn!("[AUI] trash_item rejected (outside allowed roots): {}", path);
        return false;
    }
    match trash::delete(&target) {
        Ok(_) => true,
        Err(e) => {
            log::warn!("[AUI] trash_item failed for {}: {}", path, e);
            false
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // WebKitGTK's DMABUF renderer breaks rendering (blank/garbled UI) on many
    // Linux GPU/driver combos and inside portable AppImage runs. Disable it so
    // WebKit falls back to the compatible compositing path. Must be set before
    // any WebKit/GTK init.
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    tauri::Builder::default()
        // single-instance must be registered first; forwards deep-link URLs
        // from a second launch to the running instance.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            for arg in &argv {
                if arg.contains("://") {
                    protocol::handle_url(app, arg);
                }
            }
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_deep_link::init())
        .manage(EngineState::default())
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            get_tauri_version,
            dispatch_command,
            dispatch_event,
            trash_item
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // First-launch: import existing Electron Motrix config if present.
            config::migrate_from_electron(&handle);

            // Make sure aria2 RPC has a strong secret (generated on first run).
            config::ensure_rpc_secret(&handle);

            // Start the aria2 engine before the UI connects over RPC.
            if let Err(e) = engine::start(&handle) {
                log::error!("[AUI] engine start failed: {}", e);
            }

            // System tray (icon + menu).
            if let Err(e) = tray::setup(&handle) {
                log::error!("[AUI] tray setup failed: {}", e);
            }

            // Deep-link protocol: register schemes (best-effort in dev) and
            // route URLs opened while running.
            let _ = app.deep_link().register_all();
            let dl_handle = handle.clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    protocol::handle_url(&dl_handle, url.as_str());
                }
            });

            // Sync autostart with the persisted open-at-login preference.
            let cfg = config::merged_config(&handle);
            let open_at_login = cfg
                .get("open-at-login")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            sync_autostart(&handle, open_at_login);

            // Reveal the main window once setup is done (windows start hidden).
            if let Some(win) = app.get_webview_window("main") {
                // Set the runtime window icon (_NET_WM_ICON on X11) from the
                // bundled icon so installed/packaged runs show it in the task
                // list. Skip under AppImage: the WM resolves the icon via
                // WM_CLASS -> installed .desktop there, so set_icon doesn't
                // help and we leave it to desktop integration. The AppImage
                // runtime exports $APPIMAGE, so its presence flags that case.
                if std::env::var_os("APPIMAGE").is_none() {
                    if let Some(icon) = app.default_window_icon().cloned() {
                        let _ = win.set_icon(icon);
                    }
                }
                let _ = win.show();
                let _ = win.set_focus();
            }

            // Notify the renderer the engine is up (Api.js can (re)connect).
            let _ = handle.emit("engine-ready", ());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            // Closing the main window hides it to the tray instead of quitting.
            tauri::WindowEvent::CloseRequested { api, .. } if window.label() == "main" => {
                let _ = window.hide();
                api.prevent_close();
            }
            tauri::WindowEvent::Destroyed if window.label() == "main" => {
                engine::stop(&window.app_handle());
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running Aria2 UI");
}
