use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

const WH_KEYBOARD_LL: i32 = 13;
const WM_KEYUP: u32 = 0x0101;
const WM_SYSKEYUP: u32 = 0x0105;

const VK_CAPITAL: i32 = 0x14;
const VK_NUMLOCK: i32 = 0x90;

const GWL_EXSTYLE: i32 = -20;
const WS_EX_TRANSPARENT: isize = 0x00000020;
const WS_EX_TOOLWINDOW: isize = 0x00000080;
const WS_EX_NOACTIVATE: isize = 0x08000000;

#[repr(C)]
struct KBDLLHOOKSTRUCT {
    vk_code: u32,
    scan_code: u32,
    flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[repr(C)]
struct MSG {
    hwnd: *mut c_void,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt_x: i32,
    pt_y: i32,
}

type HOOKPROC = unsafe extern "system" fn(code: i32, w_param: usize, l_param: isize) -> isize;

extern "system" {
    fn SetWindowsHookExW(idHook: i32, lpfn: HOOKPROC, hmod: *mut c_void, dwThreadId: u32) -> *mut c_void;
    fn CallNextHookEx(hhk: *mut c_void, nCode: i32, wParam: usize, lParam: isize) -> isize;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: *mut c_void, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn GetKeyState(nVirtKey: i32) -> i16;
    fn GetWindowLongW(hWnd: *mut c_void, nIndex: i32) -> isize;
    fn SetWindowLongW(hWnd: *mut c_void, nIndex: i32, dwNewLong: isize) -> isize;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsdPayload {
    pub kind: String,        // "caps_lock", "num_lock", "mode", "fan", "win_lock"
    pub title: String,       // 标题文本
    pub subtitle: String,    // 副标题/英文
    pub status: String,      // 状态文本
    pub active: bool,        // 是否为激活/开启状态
    pub icon: String,        // 图标类型标识
}

static INIT_HOOK: Once = Once::new();
static mut HOOK_HANDLE: *mut c_void = std::ptr::null_mut();
static mut GLOBAL_APP: Option<AppHandle> = None;
static LAST_CAPS: AtomicBool = AtomicBool::new(false);
static LAST_NUM: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn ll_keyboard_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code >= 0 && (w_param == WM_KEYUP as usize || w_param == WM_SYSKEYUP as usize) {
        let kb = &*(l_param as *const KBDLLHOOKSTRUCT);
        let vk = kb.vk_code as i32;

        if vk == VK_CAPITAL {
            // 在按键释放时确定性翻转状态
            let current = LAST_CAPS.load(Ordering::SeqCst);
            let next_state = !current;
            LAST_CAPS.store(next_state, Ordering::SeqCst);

            let payload = OsdPayload {
                kind: "caps_lock".into(),
                title: "大写锁定".into(),
                subtitle: "CAPS LOCK".into(),
                status: if next_state { "已开启" } else { "已关闭" }.into(),
                active: next_state,
                icon: "caps".into(),
            };

            // 在独立线程安全派发，绝不在 FFI 钩子内部阻塞或发生异常
            std::thread::spawn(move || {
                if let Some(ref app) = unsafe { GLOBAL_APP.as_ref() } {
                    trigger_osd(app, payload);
                }
            });
        } else if vk == VK_NUMLOCK {
            let current = LAST_NUM.load(Ordering::SeqCst);
            let next_state = !current;
            LAST_NUM.store(next_state, Ordering::SeqCst);

            let payload = OsdPayload {
                kind: "num_lock".into(),
                title: "数字小键盘".into(),
                subtitle: "NUM LOCK".into(),
                status: if next_state { "已开启" } else { "已关闭" }.into(),
                active: next_state,
                icon: "num".into(),
            };

            std::thread::spawn(move || {
                if let Some(ref app) = unsafe { GLOBAL_APP.as_ref() } {
                    trigger_osd(app, payload);
                }
            });
        }
    }

    CallNextHookEx(HOOK_HANDLE, n_code, w_param, l_param)
}

/// 设置并校准 OSD 悬浮窗的穿透、置顶与居中底部定位
pub fn setup_osd_window(osd_win: &WebviewWindow) {
    reposition_osd_window(osd_win);

    #[cfg(target_os = "windows")]
    if let Ok(hwnd) = osd_win.hwnd() {
        unsafe {
            let ex = GetWindowLongW(hwnd.0, GWL_EXSTYLE);
            // 增加点击穿透 (WS_EX_TRANSPARENT)、工具窗口 (WS_EX_TOOLWINDOW)、不抢焦点 (WS_EX_NOACTIVATE)
            SetWindowLongW(
                hwnd.0,
                GWL_EXSTYLE,
                ex | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            );
        }
    }
}

/// 将 OSD 窗口居中放置在主屏幕中下方
pub fn reposition_osd_window(osd_win: &WebviewWindow) {
    if let Ok(Some(monitor)) = osd_win.primary_monitor() {
        let screen_size = monitor.size();
        let scale_factor = monitor.scale_factor();
        let screen_w = screen_size.width as f64 / scale_factor;
        let screen_h = screen_size.height as f64 / scale_factor;

        let win_w = 340.0;
        let win_h = 110.0;

        // 居中靠下，距离底部 110px 避开任务栏
        let pos_x = (screen_w - win_w) / 2.0;
        let pos_y = screen_h - win_h - 110.0;

        let _ = osd_win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
            pos_x, pos_y,
        )));
    }
}

static OSD_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 触发 OSD 悬浮窗展示
pub fn trigger_osd(app: &AppHandle, payload: OsdPayload) {
    if let Some(osd_win) = app.get_webview_window("osd") {
        reposition_osd_window(&osd_win);

        // 发送前端事件
        let _ = osd_win.emit("osd-event", &payload);

        // 使用 Tauri 官方线程安全的 show & set_always_on_top
        let _ = osd_win.show();
        let _ = osd_win.set_always_on_top(true);

        // 增加代数 ID，连续触发时自动使旧的倒计时线程失效
        let current_gen = OSD_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
        let app_handle = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(3500));
            // 仅当 generation 未发生改变时才执行隐藏，支持多次触发动态续期
            if OSD_GENERATION.load(Ordering::SeqCst) == current_gen {
                hide_osd(&app_handle);
            }
        });
    }
}

/// 隐藏 OSD 窗口
pub fn hide_osd(app: &AppHandle) {
    if let Some(osd_win) = app.get_webview_window("osd") {
        let _ = osd_win.hide();
    }
}

/// 初始化全局键盘监听钩子
pub fn init_global_keyboard_hook(app: AppHandle) {
    INIT_HOOK.call_once(|| {
        unsafe {
            GLOBAL_APP = Some(app);
            // 获取当前键盘初始状态
            LAST_CAPS.store((GetKeyState(VK_CAPITAL) & 1) != 0, Ordering::SeqCst);
            LAST_NUM.store((GetKeyState(VK_NUMLOCK) & 1) != 0, Ordering::SeqCst);
        }

        // 启动后台专用消息循环线程处理底层钩子
        std::thread::spawn(|| {
            unsafe {
                HOOK_HANDLE = SetWindowsHookExW(WH_KEYBOARD_LL, ll_keyboard_proc, std::ptr::null_mut(), 0);
                if HOOK_HANDLE.is_null() {
                    eprintln!("[OSD] SetWindowsHookExW failed to register global keyboard hook");
                    return;
                }
                println!("[OSD] Global keyboard hook installed successfully");

                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        });
    });
}
