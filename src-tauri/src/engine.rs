//! aria2 engine lifecycle — Rust port of `src/main/core/Engine.js`.
//!
//! aria2c ships as a Tauri sidecar (externalBin). We spawn it with the bundled
//! config, a persistent session file, and the RPC overrides the app relies on.

use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

use crate::config;

/// Holds the running aria2 child so we can stop/restart it.
#[derive(Default)]
pub struct EngineState {
    pub child: Mutex<Option<CommandChild>>,
}

/// Resolve the session file path under the app data dir (download.session).
fn session_path(app: &AppHandle) -> std::path::PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("resolve app data dir");
    std::fs::create_dir_all(&dir).ok();
    dir.join("download.session")
}

/// Resolve the bundled aria2.conf (packaged as a resource).
fn conf_path(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path()
        .resolve("aria2.conf", tauri::path::BaseDirectory::Resource)
        .ok()
}

/// Build aria2 start arguments — mirrors Engine.getStartArgs().
fn build_args(app: &AppHandle) -> Vec<String> {
    let session = session_path(app);
    let mut args: Vec<String> = Vec::new();

    if let Some(conf) = conf_path(app) {
        args.push(format!("--conf-path={}", conf.display()));
    }
    args.push(format!("--save-session={}", session.display()));
    if session.exists() {
        args.push(format!("--input-file={}", session.display()));
    }

    // RPC overrides the app depends on (aria2.conf enables rpc but not the port).
    // Use persisted system config so a saved port/dir is honoured on launch.
    let sys = config::system_config(app);
    args.push(format!(
        "--rpc-listen-port={}",
        sys["rpc-listen-port"].as_u64().unwrap_or(config::ENGINE_RPC_PORT as u64)
    ));
    // aria2 rejects an empty --rpc-secret; only pass it when set.
    let secret = sys["rpc-secret"].as_str().unwrap_or("");
    if !secret.is_empty() {
        args.push(format!("--rpc-secret={}", secret));
    }

    // Default download directory.
    if let Some(dl) = app.path().download_dir().ok() {
        args.push(format!("--dir={}", dl.display()));
    }

    // Tie aria2's lifetime to ours: it auto-stops when this process exits,
    // even on a hard kill — prevents orphaned engines holding the RPC port.
    args.push(format!("--stop-with-process={}", std::process::id()));

    args
}

/// Start the aria2 sidecar. No-op if already running.
pub fn start(app: &AppHandle) -> Result<(), String> {
    let state: State<EngineState> = app.state();
    {
        let guard = state.child.lock().unwrap();
        if guard.is_some() {
            return Ok(());
        }
    }

    let args = build_args(app);
    log::info!("[AUI] starting aria2 sidecar: {:?}", args);

    let sidecar = app
        .shell()
        .sidecar("aria2c")
        .map_err(|e| format!("sidecar resolve failed: {e}"))?
        .args(args);

    let (mut rx, child) = sidecar
        .spawn()
        .map_err(|e| format!("aria2 spawn failed: {e}"))?;

    *state.child.lock().unwrap() = Some(child);

    // Drain stdout/stderr for logging (dev visibility).
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    log::debug!("[aria2] {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Stderr(line) => {
                    log::warn!("[aria2] {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Terminated(payload) => {
                    log::info!("[aria2] terminated: {:?}", payload);
                }
                _ => {}
            }
        }
    });

    Ok(())
}

/// Stop the aria2 sidecar if running.
pub fn stop(app: &AppHandle) {
    // Take the child out under a scoped lock so the MutexGuard is dropped
    // before we kill (avoids holding the borrow across the call).
    let child = {
        let state: State<EngineState> = app.state();
        let mut guard = state.child.lock().unwrap();
        guard.take()
    };
    if let Some(child) = child {
        let _ = child.kill();
    }
}
