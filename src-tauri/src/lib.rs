use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

fn confirm_and_quit(app: &AppHandle) {
    let handle = app.clone();
    app.dialog()
        .message("是否要退出 Sharelist？")
        .title("Sharelist")
        .buttons(MessageDialogButtons::OkCancelCustom("退出".to_string(), "取消".to_string()))
        .show(move |confirmed| {
            if confirmed {
                handle.exit(0);
            }
        });
}

#[tauri::command]
fn get_preference(app: AppHandle) -> Result<serde_json::Value, String> {
    let path = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let file = path.join("preferences.json");
    if file.exists() {
        let data = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| e.to_string())
    } else {
        Ok(serde_json::json!({
            "starup_hidden": false,
            "shortcut": "S"
        }))
    }
}

#[tauri::command]
fn set_preference(app: AppHandle, pref: serde_json::Value) -> Result<(), String> {
    let path = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let file = path.join("preferences.json");
    let data = serde_json::to_string_pretty(&pref).map_err(|e| e.to_string())?;
    std::fs::write(&file, data).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let toggle = MenuItem::with_id(app, "toggle", "显示/隐藏", true, Some("Cmd+Shift+S"))?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, Some("Cmd+Q"))?;
            let menu = Menu::with_items(app, &[&toggle, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Sharelist")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        confirm_and_quit(app);
                    }
                    "toggle" => {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                window.hide().ok();
                            } else {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                window.hide().ok();
                            } else {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                confirm_and_quit(&window.app_handle());
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![get_preference, set_preference])
        .run(tauri::generate_context!())
        .expect("error while running sharelist");
}
