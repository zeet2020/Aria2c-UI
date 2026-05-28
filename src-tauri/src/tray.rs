//! System tray — Rust port of TrayManager.js (core subset).
//! Menu items fire `application:*` commands via the `command` event, which the
//! renderer's Ipc.vue forwards to its command manager. Window/quit are handled here.

use serde_json::json;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

/// Show + focus the main window (used by tray click and several menu items).
fn show_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

/// Forward an application command to the renderer (matches ipcRenderer 'command').
fn send_command(app: &AppHandle, command: &str) {
    let _ = app.emit("command", json!({ "command": command, "args": [] }));
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let new_task = MenuItem::with_id(app, "new-task", "New Task", true, None::<&str>)?;
    let new_bt = MenuItem::with_id(app, "new-bt-task", "New BitTorrent Task", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show Aria2 UI", true, None::<&str>)?;
    let task_list = MenuItem::with_id(app, "task-list", "Task List", true, None::<&str>)?;
    let prefs = MenuItem::with_id(app, "preferences", "Preferences", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Aria2 UI", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &new_task, &new_bt, &sep1,
            &show, &sep2,
            &task_list, &prefs, &sep3,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Aria2 UI")
        .title("Aria2 UI")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "show" => show_main(app),
            "new-task" => {
                show_main(app);
                send_command(app, "application:new-task");
            }
            "new-bt-task" => {
                show_main(app);
                send_command(app, "application:new-bt-task");
            }
            "task-list" => {
                show_main(app);
                send_command(app, "application:task-list");
            }
            "preferences" => {
                show_main(app);
                send_command(app, "application:preferences");
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Left click toggles/shows the window.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
