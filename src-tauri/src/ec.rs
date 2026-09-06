use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::driver::PortDriver;

const PORT_CMD: u8 = 0x6C;  // 108
const PORT_DATA: u8 = 0x68; // 104

#[repr(C)]
struct SystemPowerStatus {
    ac_line_status: u8,
    battery_flag: u8,
    battery_life_percent: u8,
    system_status_flag: u8,
    battery_life_time: u32,
    battery_full_life_time: u32,
}

extern "system" {
    fn GetSystemPowerStatus(lpSystemPowerStatus: *mut SystemPowerStatus) -> i32;
}

pub fn is_ac_online() -> bool {
    let mut status = SystemPowerStatus {
        ac_line_status: 1,
        battery_flag: 0,
        battery_life_percent: 100,
        system_status_flag: 0,
        battery_life_time: 0,
        battery_full_life_time: 0,
    };
    if unsafe { GetSystemPowerStatus(&mut status) } != 0 {
        status.ac_line_status == 1
    } else {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub cpu_temp: u8,
    pub gpu_temp: u8,
    pub cpu_fan_rpm: u32,
    pub gpu_fan_rpm: u32,
    pub power_mode: u8,        // 1: Office, 2: Gaming, 3: Turbo
    pub fan_full_speed: bool,
    pub aou_charging: bool,
    pub wireless_charging: bool,
    pub ac_online: bool,
}

pub struct EcManager {
    driver: Arc<PortDriver>,
    last_telemetry: Mutex<TelemetryData>,
    wireless_charging_state: Mutex<bool>,
}

impl EcManager {
    pub fn new(driver: Arc<PortDriver>) -> Self {
        Self {
            driver,
            last_telemetry: Mutex::new(TelemetryData {
                cpu_temp: 50,
                gpu_temp: 45,
                cpu_fan_rpm: 2200,
                gpu_fan_rpm: 2000,
                power_mode: 2,
                fan_full_speed: false,
                aou_charging: false,
                wireless_charging: false,
                ac_online: true,
            }),
            wireless_charging_state: Mutex::new(false),
        }
    }

    /// Set performance mode:
    /// 1 = Office (办公)
    /// 2 = Gaming (游戏)
    /// 3 = Turbo (狂暴)
    pub fn set_thermal_mode(&self, mode: u8) -> Result<(), String> {
        if !(1..=3).contains(&mode) {
            return Err("Invalid mode. Must be 1 (Office), 2 (Gaming), or 3 (Turbo)".into());
        }

        self.driver.transaction(|| {
            self.driver.write_port_raw(PORT_CMD, 0xDE)?;
            sleep(Duration::from_millis(10));
            self.driver.write_port_raw(PORT_DATA, mode)?;
            Ok(())
        })
    }

    /// Read current thermal mode from EC (1..3)
    pub fn read_thermal_mode(&self) -> Result<u8, String> {
        self.driver.transaction(|| {
            self.driver.write_port_raw(PORT_CMD, 0xDE)?;
            sleep(Duration::from_millis(10));
            self.driver.write_port_raw(PORT_DATA, 0x11)?;
            sleep(Duration::from_millis(10));
            let mode = self.driver.read_port_raw(PORT_DATA)?;
            if (1..=3).contains(&mode) {
                Ok(mode)
            } else {
                Ok(2)
            }
        })
    }

    /// Enable / disable fan full speed (强冷)
    pub fn set_fan_full_speed(&self, enable: bool) -> Result<(), String> {
        let code = if enable { 14 } else { 15 };
        self.driver.transaction(|| {
            for _ in 0..2 {
                let _ = self.driver.write_port_raw(PORT_CMD, 0xDE);
                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_DATA, code);
                sleep(Duration::from_millis(10));
            }
            Ok(())
        })
    }

    /// Read fan full speed status
    pub fn read_fan_full_speed(&self) -> Result<bool, String> {
        self.driver.transaction(|| {
            self.driver.write_port_raw(PORT_CMD, 0xDE)?;
            sleep(Duration::from_millis(10));
            self.driver.write_port_raw(PORT_DATA, 16)?;
            sleep(Duration::from_millis(10));
            let val = self.driver.read_port_raw(PORT_DATA)?;
            Ok(val == 1)
        })
    }

    /// Read CPU temperature (°C) with range validation and retry
    pub fn read_cpu_temp(&self) -> Option<u8> {
        self.driver.transaction(|| {
            for _ in 0..5 {
                let _ = self.driver.write_port_raw(PORT_CMD, 0xDD);
                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_DATA, 32);
                sleep(Duration::from_millis(10));
                if let Ok(val) = self.driver.read_port_raw(PORT_DATA) {
                    if (20..=115).contains(&val) {
                        return Some(val);
                    }
                }
                sleep(Duration::from_millis(10));
            }
            None
        })
    }

    /// Read GPU temperature (°C) with range validation and retry
    pub fn read_gpu_temp(&self) -> Option<u8> {
        self.driver.transaction(|| {
            for _ in 0..5 {
                let _ = self.driver.write_port_raw(PORT_CMD, 0xDD);
                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_DATA, 35);
                sleep(Duration::from_millis(10));
                if let Ok(val) = self.driver.read_port_raw(PORT_DATA) {
                    if (20..=115).contains(&val) {
                        return Some(val);
                    }
                }
                sleep(Duration::from_millis(10));
            }
            None
        })
    }

    /// Read CPU fan speed (RPM) with range validation
    pub fn read_cpu_fan_speed(&self) -> Option<u32> {
        self.driver.transaction(|| {
            for _ in 0..5 {
                let _ = self.driver.write_port_raw(PORT_CMD, 0xD5);
                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_DATA, 24);
                sleep(Duration::from_millis(10));
                let low = match self.driver.read_port_raw(PORT_DATA) {
                    Ok(l) => l as u32,
                    Err(_) => continue,
                };

                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_CMD, 0xD5);
                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_DATA, 25);
                sleep(Duration::from_millis(10));
                let high = match self.driver.read_port_raw(PORT_DATA) {
                    Ok(h) => h as u32,
                    Err(_) => continue,
                };

                let rpm = (high << 8) | low;
                // Normal laptop fan RPM range: 0 to 6500 RPM
                if rpm <= 6500 {
                    return Some(rpm);
                }
                sleep(Duration::from_millis(10));
            }
            None
        })
    }

    /// Read GPU fan speed (RPM) with range validation
    pub fn read_gpu_fan_speed(&self) -> Option<u32> {
        self.driver.transaction(|| {
            for _ in 0..5 {
                let _ = self.driver.write_port_raw(PORT_CMD, 0xD5);
                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_DATA, 22);
                sleep(Duration::from_millis(10));
                let low = match self.driver.read_port_raw(PORT_DATA) {
                    Ok(l) => l as u32,
                    Err(_) => continue,
                };

                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_CMD, 0xD5);
                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_DATA, 23);
                sleep(Duration::from_millis(10));
                let high = match self.driver.read_port_raw(PORT_DATA) {
                    Ok(h) => h as u32,
                    Err(_) => continue,
                };

                let rpm = (high << 8) | low;
                // Normal laptop fan RPM range: 0 to 6500 RPM
                if rpm <= 6500 {
                    return Some(rpm);
                }
                sleep(Duration::from_millis(10));
            }
            None
        })
    }

    /// Set AOU (Always-On USB / 关机对外充电)
    pub fn set_aou_charging(&self, enable: bool) -> Result<(), String> {
        let code = if enable { 10 } else { 11 };
        self.driver.transaction(|| {
            for _ in 0..2 {
                let _ = self.driver.write_port_raw(PORT_CMD, 0xDE);
                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_DATA, code);
                sleep(Duration::from_millis(10));
            }
            Ok(())
        })
    }

    /// Read AOU status
    pub fn read_aou_status(&self) -> Result<bool, String> {
        self.driver.transaction(|| {
            self.driver.write_port_raw(PORT_CMD, 0xDE)?;
            sleep(Duration::from_millis(10));
            self.driver.write_port_raw(PORT_DATA, 12)?;
            sleep(Duration::from_millis(10));
            let val = self.driver.read_port_raw(PORT_DATA)?;
            Ok(val == 16)
        })
    }

    /// Set Wireless Charging (无线充电)
    /// Protocol reversed from OEM ECManager::NotifyECChangeWirelessCharge
    pub fn set_wireless_charging(&self, enable: bool) -> Result<(), String> {
        let code = if enable { 64 } else { 65 }; // 64 = 0x40 (Enable), 65 = 0x41 (Disable)
        self.driver.transaction(|| {
            for _ in 0..2 {
                let _ = self.driver.write_port_raw(PORT_CMD, 0xDE);
                sleep(Duration::from_millis(10));
                let _ = self.driver.write_port_raw(PORT_DATA, code);
                sleep(Duration::from_millis(10));
            }
            let mut state = self.wireless_charging_state.lock().unwrap();
            *state = enable;
            Ok(())
        })
    }

    /// Fetch all telemetry data with noise smoothing & cache fallback
    pub fn get_all_telemetry(&self) -> Result<TelemetryData, String> {
        let mut cache = self.last_telemetry.lock().unwrap();

        let cpu_temp = self.read_cpu_temp().unwrap_or(cache.cpu_temp);
        let gpu_temp = self.read_gpu_temp().unwrap_or(cache.gpu_temp);
        let cpu_fan_rpm = self.read_cpu_fan_speed().unwrap_or(cache.cpu_fan_rpm);
        let gpu_fan_rpm = self.read_gpu_fan_speed().unwrap_or(cache.gpu_fan_rpm);
        let power_mode = self.read_thermal_mode().unwrap_or(cache.power_mode);
        let fan_full_speed = self.read_fan_full_speed().unwrap_or(cache.fan_full_speed);
        let aou_charging = self.read_aou_status().unwrap_or(cache.aou_charging);
        let wireless_charging = *self.wireless_charging_state.lock().unwrap();
        let ac_online = is_ac_online();

        let data = TelemetryData {
            cpu_temp,
            gpu_temp,
            cpu_fan_rpm,
            gpu_fan_rpm,
            power_mode,
            fan_full_speed,
            aou_charging,
            wireless_charging,
            ac_online,
        };

        *cache = data.clone();
        Ok(data)
    }
}
