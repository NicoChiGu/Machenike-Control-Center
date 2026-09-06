use std::ffi::c_void;
use std::ptr::null_mut;
use serde::{Deserialize, Serialize};

const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
const TOKEN_QUERY: u32 = 0x0008;
const SE_PRIVILEGE_ENABLED: u32 = 0x00000002;

const SA_SETUP_NAME: &str = "SaSetup";
const SA_SETUP_GUID: &str = "{72C5E28C-7783-43A1-8767-FAD73FCCAFFA}";

// Buffer capacity for SaSetup
const NVRAM_BUF_SIZE: usize = 4096;

// Offsets reversed from OEM SysInfo.dll & ShortcutSetup
const OFFSET_GPU_MODE_1: usize = 0xB1; // 177: 1 = Dedicated, 4 = Integrated
const OFFSET_GPU_MODE_2: usize = 0xB7; // 183: 0 = Dedicated, 2 = Integrated
const OFFSET_SAGV_MODE: usize = 497;   // 0x1F1: 0 = Gear1 (Off), 5 = Gear2/SAGV (On)

#[repr(C)]
#[derive(Clone, Copy)]
struct Luid {
    low_part: u32,
    high_part: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct LuidAndAttributes {
    luid: Luid,
    attributes: u32,
}

#[repr(C)]
struct TokenPrivileges {
    privilege_count: u32,
    privileges: [LuidAndAttributes; 1],
}

extern "system" {
    fn GetCurrentProcess() -> isize;

    fn CloseHandle(handle: isize) -> i32;

    fn GetLastError() -> u32;

    fn GetFirmwareEnvironmentVariableW(
        lpName: *const u16,
        lpGuid: *const u16,
        pBuffer: *mut c_void,
        nSize: u32,
    ) -> u32;

    fn SetFirmwareEnvironmentVariableW(
        lpName: *const u16,
        lpGuid: *const u16,
        pBuffer: *const c_void,
        nSize: u32,
    ) -> i32;
}

#[link(name = "advapi32")]
extern "system" {
    fn OpenProcessToken(
        process_handle: isize,
        desired_access: u32,
        token_handle: *mut isize,
    ) -> i32;

    fn LookupPrivilegeValueW(
        lp_system_name: *const u16,
        lp_name: *const u16,
        lp_luid: *mut Luid,
    ) -> i32;

    fn AdjustTokenPrivileges(
        token_handle: isize,
        disable_all_privileges: i32,
        new_state: *const TokenPrivileges,
        buffer_length: u32,
        previous_state: *mut c_void,
        return_length: *mut u32,
    ) -> i32;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiosSettings {
    pub gpu_direct: bool,
    pub memory_gear: bool,
    pub is_admin: bool,
}

/// Enable SeSystemEnvironmentPrivilege required for UEFI NVRAM modification
fn enable_privilege() -> Result<(), String> {
    unsafe {
        let mut token: isize = 0;
        let proc = GetCurrentProcess();
        if OpenProcessToken(proc, TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token) == 0 {
            return Err(format!("OpenProcessToken failed (code {})", GetLastError()));
        }

        let priv_name: Vec<u16> = "SeSystemEnvironmentPrivilege"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut luid = Luid {
            low_part: 0,
            high_part: 0,
        };

        if LookupPrivilegeValueW(null_mut(), priv_name.as_ptr(), &mut luid) == 0 {
            CloseHandle(token);
            return Err(format!("LookupPrivilegeValueW failed (code {})", GetLastError()));
        }

        let tp = TokenPrivileges {
            privilege_count: 1,
            privileges: [LuidAndAttributes {
                luid,
                attributes: SE_PRIVILEGE_ENABLED,
            }],
        };

        let ok = AdjustTokenPrivileges(
            token,
            0,
            &tp,
            std::mem::size_of::<TokenPrivileges>() as u32,
            null_mut(),
            null_mut(),
        );

        let err = GetLastError();
        CloseHandle(token);

        if ok == 0 || err != 0 {
            return Err(format!(
                "AdjustTokenPrivileges failed (error {}). Please run with Administrator privilege.",
                err
            ));
        }

        Ok(())
    }
}

/// Read SaSetup NVRAM buffer
fn read_sa_setup() -> Result<(Vec<u8>, usize), String> {
    enable_privilege()?;

    let name: Vec<u16> = SA_SETUP_NAME.encode_utf16().chain(std::iter::once(0)).collect();
    let guid: Vec<u16> = SA_SETUP_GUID.encode_utf16().chain(std::iter::once(0)).collect();

    let mut buf = vec![0u8; NVRAM_BUF_SIZE];
    let bytes_read = unsafe {
        GetFirmwareEnvironmentVariableW(
            name.as_ptr(),
            guid.as_ptr(),
            buf.as_mut_ptr() as *mut c_void,
            NVRAM_BUF_SIZE as u32,
        )
    };

    if bytes_read == 0 {
        let err = unsafe { GetLastError() };
        return Err(format!("GetFirmwareEnvironmentVariableW failed (error {})", err));
    }

    Ok((buf, bytes_read as usize))
}

/// Write back SaSetup NVRAM buffer
fn write_sa_setup(buf: &[u8], len: usize) -> Result<(), String> {
    enable_privilege()?;

    let name: Vec<u16> = SA_SETUP_NAME.encode_utf16().chain(std::iter::once(0)).collect();
    let guid: Vec<u16> = SA_SETUP_GUID.encode_utf16().chain(std::iter::once(0)).collect();

    let ok = unsafe {
        SetFirmwareEnvironmentVariableW(
            name.as_ptr(),
            guid.as_ptr(),
            buf.as_ptr() as *const c_void,
            len as u32,
        )
    };

    if ok == 0 {
        let err = unsafe { GetLastError() };
        return Err(format!("SetFirmwareEnvironmentVariableW failed (error {})", err));
    }

    Ok(())
}

/// Query current BIOS settings (Discrete GPU & SAGV Memory Gear)
pub fn query_bios_settings() -> Result<BiosSettings, String> {
    match read_sa_setup() {
        Ok((buf, len)) => {
            let gpu_direct = if len > OFFSET_GPU_MODE_1 {
                buf[OFFSET_GPU_MODE_1] == 1
            } else {
                false
            };

            let memory_gear = if len > OFFSET_SAGV_MODE {
                buf[OFFSET_SAGV_MODE] == 5
            } else {
                false
            };

            Ok(BiosSettings {
                gpu_direct,
                memory_gear,
                is_admin: true,
            })
        }
        Err(e) => {
            // If failed due to privileges, return safe fallback with is_admin = false
            eprintln!("[NVRAM] Failed to read BIOS NVRAM: {}", e);
            Ok(BiosSettings {
                gpu_direct: false,
                memory_gear: false,
                is_admin: false,
            })
        }
    }
}

/// Set GPU Direct mode (独显直连 vs 混合输出)
pub fn set_gpu_direct_mode(enable_direct: bool) -> Result<(), String> {
    let (mut buf, len) = read_sa_setup()?;

    if len <= OFFSET_GPU_MODE_2 {
        return Err("SaSetup NVRAM buffer too small".into());
    }

    if enable_direct {
        // Discrete only: offset 0xB1 = 1, 0xB7 = 0
        buf[OFFSET_GPU_MODE_1] = 0x01;
        buf[OFFSET_GPU_MODE_2] = 0x00;
    } else {
        // Integrated/Hybrid: offset 0xB1 = 4, 0xB7 = 2
        buf[OFFSET_GPU_MODE_1] = 0x04;
        buf[OFFSET_GPU_MODE_2] = 0x02;
    }

    write_sa_setup(&buf, len)?;
    println!("[NVRAM] Set GPU direct mode to {} in SaSetup", enable_direct);
    Ok(())
}

/// Set SAGV Memory Gear mode (内存分频模式: 开启分频 5 / 关闭分频 0)
pub fn set_memory_gear_mode(enable_sagv: bool) -> Result<(), String> {
    let (mut buf, len) = read_sa_setup()?;

    if len <= OFFSET_SAGV_MODE {
        return Err("SaSetup NVRAM buffer too small for SAGV offset".into());
    }

    // 5 = Gear 2 (SAGV Enabled / Power Saving), 0 = Gear 1 (SAGV Disabled / Performance)
    buf[OFFSET_SAGV_MODE] = if enable_sagv { 5 } else { 0 };

    write_sa_setup(&buf, len)?;
    println!("[NVRAM] Set SAGV Memory Gear mode to {} in SaSetup", enable_sagv);
    Ok(())
}

/// Trigger system restart to apply UEFI NVRAM changes
pub fn reboot_system() -> Result<(), String> {
    std::process::Command::new("shutdown.exe")
        .args(["-r", "-t", "00"])
        .spawn()
        .map_err(|e| format!("Failed to spawn shutdown.exe: {}", e))?;
    Ok(())
}
