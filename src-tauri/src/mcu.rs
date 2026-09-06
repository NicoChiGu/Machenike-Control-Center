use hidapi::{HidApi, HidDevice};
use std::sync::Mutex;

const TARGET_VID: u16 = 0x1A2C;
const TARGET_PID: u16 = 0x1512;

pub struct McuManager {
    device: Mutex<Option<HidDevice>>,
    light_mask: Mutex<u16>,
}

impl McuManager {
    pub fn new() -> Self {
        let device = Self::connect();
        Self {
            device: Mutex::new(device),
            light_mask: Mutex::new(0x0000), // 0x0000 = all lights enabled (unmasked)
        }
    }

    fn connect() -> Option<HidDevice> {
        let api = match HidApi::new() {
            Ok(api) => api,
            Err(e) => {
                eprintln!("[MCU] Failed to initialize HidApi: {}", e);
                return None;
            }
        };

        // Enumerate devices to specifically find VID 1A2C & PID 1512 interface MI_03 (Lighting control)
        for dev_info in api.device_list() {
            if dev_info.vendor_id() == TARGET_VID && dev_info.product_id() == TARGET_PID {
                let path_str = dev_info.path().to_string_lossy().to_lowercase();
                let is_mi_03 = path_str.contains("mi_03")
                    || dev_info.interface_number() == 3
                    || (dev_info.usage_page() == 0xFF00 && dev_info.usage() == 0x0002);

                if is_mi_03 {
                    match dev_info.open_device(&api) {
                        Ok(dev) => {
                            println!("[MCU] Successfully connected to keyboard MCU (MI_03): {:?}", dev_info.path());
                            return Some(dev);
                        }
                        Err(e) => {
                            eprintln!("[MCU] Failed to open keyboard MCU (MI_03): {}", e);
                        }
                    }
                }
            }
        }

        eprintln!("[MCU] Keyboard MCU (VID: {:04X}, PID: {:04X}, MI_03) not found", TARGET_VID, TARGET_PID);
        None
    }

    fn send_report(&self, cmd: u16, payload: &[u8]) -> Result<(), String> {
        let mut guard = self.device.lock().unwrap();
        if guard.is_none() {
            *guard = Self::connect();
        }
        let dev = match guard.as_ref() {
            Some(d) => d,
            None => return Err("Keyboard MCU HID device (MI_03) not found".into()),
        };

        // 65-byte standard Windows HID buffer: [ReportID = 0, cmd_lo, cmd_hi, payload...]
        let mut packet = [0u8; 65];
        packet[0] = 0x00; // Report ID

        // Exact Little-Endian matching x86 original DLL:
        packet[1] = (cmd & 0xFF) as u8;         // Command LSB (e.g. 0x03 for 0xC003)
        packet[2] = ((cmd >> 8) & 0xFF) as u8; // Command MSB (e.g. 0xC0 for 0xC003)

        let copy_len = payload.len().min(62);
        packet[3..3 + copy_len].copy_from_slice(&payload[..copy_len]);

        // Try device.write (Output Report / WriteFile), fallback to send_feature_report
        let res = match dev.write(&packet) {
            Ok(_) => Ok(()),
            Err(err1) => {
                match dev.send_feature_report(&packet) {
                    Ok(_) => Ok(()),
                    Err(err2) => {
                        Err(format!("HID send failed: write err ({}), feature err ({})", err1, err2))
                    }
                }
            }
        };

        if let Err(ref e) = res {
            eprintln!("[MCU] Send report 0x{:04X} failed: {}. Resetting device handle for retry.", cmd, e);
            *guard = None; // Reset handle on failure so next call re-enumerates
        } else {
            println!("[MCU] Send report 0x{:04X} succeeded (bytes: {:02X} {:02X}, len: {})", cmd, packet[1], packet[2], payload.len());
        }

        res
    }

    /// Set RGB for a specific zone or all (godsync) - 0xC003
    /// which_light: 0: Keyboard, 1: Left, 2: Right, 3: Logo, 4: All
    /// effect: 0: AlwaysOn, 1: Clock, 2: Wind, 3: Wave, 4: Breath, 5: Jump, 6: Rainbow, 7: Flow
    pub fn set_light_rgb(
        &self,
        which_light: u8,
        effect: u8,
        brightness: u8,
        speed: u8,
        orientation: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), String> {
        // Ensure lighting power is enabled first (0xC001 OpenLight)
        let _ = self.set_light_state(which_light, true);

        // Exact byte layout from factory mcucontrol.dll:
        // payload[0] = which_light
        // payload[1] = 0x00 (padding)
        // payload[2] = effect | 0x80 (highest bit must be 1!)
        // payload[3] = brightness (written twice)
        // payload[4] = brightness
        // payload[5] = (speed << 4) | (orientation & 0x0F)
        // payload[6] = r
        // payload[7] = g
        // payload[8] = b
        // payload[9] = r (background r)
        // payload[10] = g (background g)
        // payload[11] = b (background b)
        let effect_byte = effect | 0x80;
        let speed_orient = ((speed.clamp(1, 5)) << 4) | (orientation & 0x0F);
        let payload = [
            which_light,
            0x00,
            effect_byte,
            brightness,
            brightness,
            speed_orient,
            r,
            g,
            b,
            r,
            g,
            b,
        ];
        self.send_report(0xC003, &payload)
    }

    /// Set 4-zone colors and speeds - 0xC00B
    /// Layout matching factory mcucontrol.dll: 10 bytes per zone
    pub fn set_four_areas(
        &self,
        brightness: [u8; 4],
        speed: [u8; 4],
        colors: [[u8; 3]; 4],
    ) -> Result<(), String> {
        // Ensure keyboard power is enabled first
        let _ = self.set_light_state(0, true);

        let mut payload = [0u8; 40];
        for i in 0..4 {
            let base = i * 10;
            payload[base] = brightness[i];
            payload[base + 1] = brightness[i];
            payload[base + 2] = speed[i];
            payload[base + 3] = colors[i][0]; // R1
            payload[base + 4] = colors[i][1]; // G1
            payload[base + 5] = colors[i][2]; // B1
            payload[base + 6] = colors[i][0]; // R2
            payload[base + 7] = colors[i][1]; // G2
            payload[base + 8] = colors[i][2]; // B2
            payload[base + 9] = 0;
        }

        self.send_report(0xC00B, &payload)
    }

    /// Set individual effects for 4 zones - 0xC00A (verified from factory binary)
    pub fn set_four_area_effects(&self, effects: [u8; 4]) -> Result<(), String> {
        self.send_report(0xC00A, &effects)
    }

    /// Turn light on or off - 0xC001 (OpenLight / CloseLight)
    /// which_light: 0: Keyboard (0x03), 1: Left (0x0C), 2: Right (0x30), 3: Logo (0xC0), 4: Master/All (0x00FF / 0x0000)
    pub fn set_light_state(&self, which_light: u8, enable: bool) -> Result<(), String> {
        let mut mask_guard = self.light_mask.lock().map_err(|e| e.to_string())?;
        let current_mask = *mask_guard;
        let new_mask = if which_light >= 4 {
            // Master/All toggle: OpenLight sets 0x0000, CloseLight sets 0x00FF
            if enable { 0x0000u16 } else { 0x00FFu16 }
        } else {
            // Zone 0: bits 0..1 (0x03), Zone 1: bits 2..3 (0x0C), Zone 2: bits 4..5 (0x30), Zone 3: bits 6..7 (0xC0)
            let zone_bits: u16 = 0x0003u16 << (which_light * 2);
            if enable {
                current_mask & !zone_bits
            } else {
                current_mask | zone_bits
            }
        };
        *mask_guard = new_mask;

        let low = (new_mask & 0xFF) as u8;
        let high = ((new_mask >> 8) & 0xFF) as u8;
        self.send_report(0xC001, &[low, high])
    }

    /// Lock / Unlock Windows Key - 0xC007
    pub fn set_win_lock(&self, lock: bool) -> Result<(), String> {
        let val = if lock { 1 } else { 0 };
        self.send_report(0xC007, &[val])
    }

    /// Set Keyboard backlight sleep timeout in seconds - 0xC008
    pub fn set_keyboard_timeout(&self, timeout_sec: u16) -> Result<(), String> {
        let low = (timeout_sec & 0xFF) as u8;
        let high = ((timeout_sec >> 8) & 0xFF) as u8;
        self.send_report(0xC008, &[low, high])
    }
}
