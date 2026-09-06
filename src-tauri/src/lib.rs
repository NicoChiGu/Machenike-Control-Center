mod driver;
mod ec;
mod mcu;
pub mod osd;
mod nvram;
mod commands;

use std::sync::Arc;
use driver::PortDriver;
use ec::EcManager;
use mcu::McuManager;
use commands::AppState;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let port_driver = match PortDriver::new() {
        Ok(d) => Arc::new(d),
        Err(e) => {
            eprintln!("[Error] Failed to initialize PortDriver: {}", e);
            panic!("Hardware PortDriver failed: {}", e);
        }
    };

    let ec_manager = Arc::new(EcManager::new(port_driver.clone()));
    let mcu_manager = Arc::new(McuManager::new());

    let app_state = AppState {
        ec: ec_manager.clone(),
        mcu: mcu_manager.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                    let _ = window.emit("window-visibility-change", false);
                }
            }
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            // 启动全局键盘钩子 (Caps Lock / Num Lock)
            osd::init_global_keyboard_hook(app_handle.clone());

            // 初始化 OSD 悬浮窗口的点击穿透与位置
            if let Some(osd_win) = app.get_webview_window("osd") {
                osd::setup_osd_window(&osd_win);
            }

            // 创建系统托盘图标
            let show_i = tauri::menu::MenuItem::with_id(app, "show", "显示控制中心", true, None::<&str>)?;
            let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "退出程序", true, None::<&str>)?;
            let tray_menu = tauri::menu::Menu::with_items(app, &[&show_i, &quit_i])?;

            let mut tray_builder = tauri::tray::TrayIconBuilder::new()
                .tooltip("Machenike L16W Center (机械师智能控制中心)")
                .menu(&tray_menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.unminimize();
                                let _ = win.set_focus();
                                let _ = win.emit("window-visibility-change", true);
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                                let _ = win.emit("window-visibility-change", false);
                            } else {
                                let _ = win.show();
                                let _ = win.unminimize();
                                let _ = win.set_focus();
                                let _ = win.emit("window-visibility-change", true);
                            }
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder.build(app)?;

            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_telemetry,
            commands::set_performance_mode,
            commands::set_fan_full_speed,
            commands::set_light_rgb,
            commands::set_four_areas,
            commands::set_four_area_effects,
            commands::set_light_state,
            commands::set_win_lock,
            commands::set_keyboard_timeout,
            commands::set_aou_charge,
            commands::set_wireless_charge,
            commands::get_bios_settings,
            commands::set_gpu_direct_mode,
            commands::set_memory_gear_mode,
            commands::request_reboot,
            commands::hide_osd_window,
            commands::trigger_osd_test,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
