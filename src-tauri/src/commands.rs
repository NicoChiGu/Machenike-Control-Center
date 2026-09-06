use std::sync::Arc;
use tauri::{AppHandle, State};
use crate::ec::{EcManager, TelemetryData};
use crate::mcu::McuManager;
use crate::osd::{hide_osd, trigger_osd, OsdPayload};

pub struct AppState {
    pub ec: Arc<EcManager>,
    pub mcu: Arc<McuManager>,
}

#[tauri::command]
pub fn get_telemetry(state: State<'_, AppState>) -> Result<TelemetryData, String> {
    state.ec.get_all_telemetry()
}

#[tauri::command]
pub fn set_performance_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: u8,
) -> Result<(), String> {
    let res = state.ec.set_thermal_mode(mode);
    if res.is_ok() {
        let (title, status, icon) = match mode {
            1 => ("办公模式", "低功耗 静音节能", "office"),
            2 => ("游戏模式", "均衡调教 高能释放", "gaming"),
            3 => ("狂暴模式", "满血激进 极限性能", "turbo"),
            _ => ("性能模式", "模式切换完成", "mode"),
        };
        trigger_osd(
            &app,
            OsdPayload {
                kind: "mode".into(),
                title: title.into(),
                subtitle: "PERFORMANCE MODE".into(),
                status: status.into(),
                active: true,
                icon: icon.into(),
            },
        );
    }
    res
}

#[tauri::command]
pub fn set_fan_full_speed(
    app: AppHandle,
    state: State<'_, AppState>,
    enable: bool,
) -> Result<(), String> {
    let res = state.ec.set_fan_full_speed(enable);
    if res.is_ok() {
        trigger_osd(
            &app,
            OsdPayload {
                kind: "fan".into(),
                title: if enable { "风扇全速强冷" } else { "智能温控转速" }.into(),
                subtitle: "COOLER BOOST".into(),
                status: if enable { "双风扇全速 极速降温" } else { "自适应动态温控" }.into(),
                active: enable,
                icon: "fan".into(),
            },
        );
    }
    res
}

#[tauri::command]
pub fn set_light_rgb(
    state: State<'_, AppState>,
    which_light: u8,
    effect: u8,
    brightness: u8,
    speed: u8,
    orientation: u8,
    r: u8,
    g: u8,
    b: u8,
) -> Result<(), String> {
    state.mcu.set_light_rgb(which_light, effect, brightness, speed, orientation, r, g, b)
}

#[tauri::command]
pub fn set_four_areas(
    state: State<'_, AppState>,
    brightness: [u8; 4],
    speed: [u8; 4],
    colors: [[u8; 3]; 4],
) -> Result<(), String> {
    state.mcu.set_four_areas(brightness, speed, colors)
}

#[tauri::command]
pub fn set_four_area_effects(state: State<'_, AppState>, effects: [u8; 4]) -> Result<(), String> {
    state.mcu.set_four_area_effects(effects)
}

#[tauri::command]
pub fn set_light_state(state: State<'_, AppState>, which_light: u8, enable: bool) -> Result<(), String> {
    state.mcu.set_light_state(which_light, enable)
}

#[tauri::command]
pub fn set_win_lock(app: AppHandle, state: State<'_, AppState>, lock: bool) -> Result<(), String> {
    let res = state.mcu.set_win_lock(lock);
    if res.is_ok() {
        trigger_osd(
            &app,
            OsdPayload {
                kind: "win_lock".into(),
                title: if lock { "Win 键锁定" } else { "Win 键解锁" }.into(),
                subtitle: "WIN LOCK".into(),
                status: if lock { "防游戏误触 已启用" } else { "按键功能 已恢复" }.into(),
                active: lock,
                icon: "winlock".into(),
            },
        );
    }
    res
}

#[tauri::command]
pub fn set_keyboard_timeout(state: State<'_, AppState>, always_on: bool) -> Result<(), String> {
    let timeout = if always_on { 0 } else { 30 };
    state.mcu.set_keyboard_timeout(timeout)
}

#[tauri::command]
pub fn set_aou_charge(state: State<'_, AppState>, enable: bool) -> Result<(), String> {
    state.ec.set_aou_charging(enable)
}

#[tauri::command]
pub fn set_wireless_charge(state: State<'_, AppState>, enable: bool) -> Result<(), String> {
    state.ec.set_wireless_charging(enable)
}

#[tauri::command]
pub fn get_bios_settings() -> Result<crate::nvram::BiosSettings, String> {
    crate::nvram::query_bios_settings()
}

#[tauri::command]
pub fn set_gpu_direct_mode(enable: bool) -> Result<(), String> {
    crate::nvram::set_gpu_direct_mode(enable)
}

#[tauri::command]
pub fn set_memory_gear_mode(enable: bool) -> Result<(), String> {
    crate::nvram::set_memory_gear_mode(enable)
}

#[tauri::command]
pub fn request_reboot() -> Result<(), String> {
    crate::nvram::reboot_system()
}

#[tauri::command]
pub fn hide_osd_window(app: AppHandle) {
    hide_osd(&app);
}

#[tauri::command]
pub fn trigger_osd_test(app: AppHandle, kind: String) -> Result<(), String> {
    let payload = match kind.as_str() {
        "caps" => OsdPayload {
            kind: "caps_lock".into(),
            title: "大写锁定".into(),
            subtitle: "CAPS LOCK".into(),
            status: "已开启".into(),
            active: true,
            icon: "caps".into(),
        },
        "num" => OsdPayload {
            kind: "num_lock".into(),
            title: "数字小键盘".into(),
            subtitle: "NUM LOCK".into(),
            status: "已开启".into(),
            active: true,
            icon: "num".into(),
        },
        "gaming" => OsdPayload {
            kind: "mode".into(),
            title: "游戏模式".into(),
            subtitle: "PERFORMANCE MODE".into(),
            status: "均衡调教 高能释放".into(),
            active: true,
            icon: "gaming".into(),
        },
        _ => OsdPayload {
            kind: "mode".into(),
            title: "狂暴模式".into(),
            subtitle: "TURBO MODE".into(),
            status: "满血激进 极限性能".into(),
            active: true,
            icon: "turbo".into(),
        },
    };
    trigger_osd(&app, payload);
    Ok(())
}
