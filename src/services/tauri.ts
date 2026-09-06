import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { TelemetryData, OsdPayload } from "../types/hardware";

export const hardwareApi = {
  getTelemetry(): Promise<TelemetryData> {
    return invoke("get_telemetry");
  },
  setPerformanceMode(mode: number): Promise<void> {
    return invoke("set_performance_mode", { mode });
  },
  setFanFullSpeed(enable: boolean): Promise<void> {
    return invoke("set_fan_full_speed", { enable });
  },
  setLightRgb(
    whichLight: number,
    effect: number,
    brightness: number,
    speed: number,
    orientation: number,
    r: number,
    g: number,
    b: number
  ): Promise<void> {
    return invoke("set_light_rgb", {
      whichLight,
      effect,
      brightness,
      speed,
      orientation,
      r,
      g,
      b,
    });
  },
  setFourAreas(
    brightness: [number, number, number, number],
    speed: [number, number, number, number],
    colors: [
      [number, number, number],
      [number, number, number],
      [number, number, number],
      [number, number, number]
    ]
  ): Promise<void> {
    return invoke("set_four_areas", { brightness, speed, colors });
  },
  setFourAreaEffects(effects: [number, number, number, number]): Promise<void> {
    return invoke("set_four_area_effects", { effects });
  },
  setLightState(whichLight: number, enable: boolean): Promise<void> {
    return invoke("set_light_state", { whichLight, enable });
  },
  setWinLock(lock: boolean): Promise<void> {
    return invoke("set_win_lock", { lock });
  },
  setKeyboardTimeout(alwaysOn: boolean): Promise<void> {
    return invoke("set_keyboard_timeout", { alwaysOn });
  },
  setAouCharge(enable: boolean): Promise<void> {
    return invoke("set_aou_charge", { enable });
  },
  setWirelessCharge(enable: boolean): Promise<void> {
    return invoke("set_wireless_charge", { enable });
  },
  getBiosSettings(): Promise<import("../types/hardware").BiosSettings> {
    return invoke("get_bios_settings");
  },
  setGpuDirectMode(enable: boolean): Promise<void> {
    return invoke("set_gpu_direct_mode", { enable });
  },
  setMemoryGearMode(enable: boolean): Promise<void> {
    return invoke("set_memory_gear_mode", { enable });
  },
  requestReboot(): Promise<void> {
    return invoke("request_reboot");
  },
  hideOsd(): Promise<void> {
    return invoke("hide_osd_window");
  },
  triggerOsdTest(kind: string): Promise<void> {
    return invoke("trigger_osd_test", { kind });
  },
  onOsdEvent(callback: (data: OsdPayload) => void): Promise<UnlistenFn> {
    return listen<OsdPayload>("osd-event", (event) => {
      callback(event.payload);
    });
  },
  onTelemetryUpdate(callback: (data: TelemetryData) => void): Promise<UnlistenFn> {
    return listen<TelemetryData>("telemetry-update", (event) => {
      callback(event.payload);
    });
  },
  onWindowVisibilityChange(callback: (visible: boolean) => void): Promise<UnlistenFn> {
    return listen<boolean>("window-visibility-change", (event) => {
      callback(event.payload);
    });
  },
};

