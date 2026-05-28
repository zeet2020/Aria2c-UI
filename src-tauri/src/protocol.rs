//! Deep-link / protocol handling — Rust port of ProtocolManager.js.
//! Resource URLs (magnet/thunder/http/https/ftp) -> new-task.
//! mo://<host>?<query> -> mapped application command with the query as args.

use serde_json::{json, Map, Value};
use tauri::{AppHandle, Emitter, Manager};

/// mo:// host -> application command (from configs/protocol.js).
fn mo_command(host: &str) -> Option<&'static str> {
    Some(match host {
        "task-list" => "application:task-list",
        "new-task" => "application:new-task",
        "new-bt-task" => "application:new-bt-task",
        "pause-all-task" => "application:pause-all-task",
        "resume-all-task" => "application:resume-all-task",
        "reveal-in-folder" => "application:reveal-in-folder",
        "preferences" => "application:preferences",
        "about" => "application:about",
        _ => return None,
    })
}

fn show_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

/// Entry point: route a single incoming URL.
pub fn handle_url(app: &AppHandle, url: &str) {
    log::info!("[AUI] protocol url: {}", url);
    let lower = url.to_lowercase();
    show_main(app);

    if lower.starts_with("magnet:")
        || lower.starts_with("thunder:")
        || lower.starts_with("http:")
        || lower.starts_with("https:")
        || lower.starts_with("ftp:")
    {
        let _ = app.emit(
            "command",
            json!({ "command": "application:new-task", "args": [{ "type": "uri", "uri": url }] }),
        );
        return;
    }

    if lower.starts_with("mo:") || lower.starts_with("motrix:") {
        handle_mo(app, url);
    }
}

/// Parse mo://<host>?<k=v&...> and dispatch the mapped command with query args.
fn handle_mo(app: &AppHandle, url: &str) {
    // Strip scheme.
    let rest = url
        .splitn(2, "://")
        .nth(1)
        .unwrap_or("")
        .trim_start_matches('/');
    let (host, query) = match rest.split_once('?') {
        Some((h, q)) => (h.trim_end_matches('/'), q),
        None => (rest.trim_end_matches('/'), ""),
    };

    let Some(command) = mo_command(host) else {
        log::warn!("[AUI] unknown mo:// host: {}", host);
        return;
    };

    let mut args = Map::new();
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        args.insert(
            k.to_string(),
            Value::String(urldecode(v)),
        );
    }

    let _ = app.emit("command", json!({ "command": command, "args": [Value::Object(args)] }));
}

/// Minimal percent-decoding for query values.
fn urldecode(s: &str) -> String {
    let bytes = s.replace('+', " ");
    let bytes = bytes.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
