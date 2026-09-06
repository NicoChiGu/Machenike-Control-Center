export interface TelemetryData {
  cpu_temp: number;
  gpu_temp: number;
  cpu_fan_rpm: number;
  gpu_fan_rpm: number;
  power_mode: number; // 1: 办公, 2: 游戏, 3: 狂暴
  fan_full_speed: boolean;
  aou_charging: boolean;
  wireless_charging: boolean;
  ac_online: boolean;
}

export interface BiosSettings {
  gpu_direct: boolean;
  memory_gear: boolean;
  is_admin: boolean;
}

export interface OsdPayload {
  kind: "caps_lock" | "num_lock" | "mode" | "fan" | "win_lock" | string;
  title: string;
  subtitle: string;
  status: string;
  active: boolean;
  icon: string;
}

export type PerformanceMode = 1 | 2 | 3;

export interface LightEffectOption {
  id: number;
  name: string;
  enName: string;
  icon: string;
  hasOrientation?: boolean;
}

export const LIGHT_EFFECTS: LightEffectOption[] = [
  { id: 0, name: "常亮", enName: "AlwaysOn", icon: "Sun" },
  { id: 1, name: "钟摆", enName: "Clock", icon: "Clock" },
  { id: 2, name: "风行", enName: "Wind", icon: "Wind" },
  { id: 3, name: "海浪", enName: "Wave", icon: "Waves", hasOrientation: true },
  { id: 4, name: "呼吸", enName: "Breath", icon: "Activity" },
  { id: 5, name: "跳动", enName: "Jump", icon: "Zap" },
  { id: 6, name: "彩虹", enName: "Rainbow", icon: "Sparkles" },
  { id: 7, name: "流光", enName: "Flow", icon: "Flame", hasOrientation: true },
];

export interface LightZoneOption {
  id: number;
  name: string;
  enName: string;
}

export const LIGHT_ZONES: LightZoneOption[] = [
  { id: 4, name: "神光同步 (整机)", enName: "GodSync" },
  { id: 0, name: "键盘背光", enName: "Keyboard" },
  { id: 1, name: "左侧灯带", enName: "Left Light" },
  { id: 2, name: "右侧灯带", enName: "Right Light" },
  { id: 3, name: "Logo 信仰灯", enName: "Logo" },
];

export interface RgbColor {
  r: number;
  g: number;
  b: number;
}

export interface ZoneLightConfig {
  enabled: boolean;
  effect: number;
  brightness: number;
  speed: number;
  orientation: number;
  color: RgbColor;
  subZones?: [RgbColor, RgbColor, RgbColor, RgbColor];
}

export interface LightZonesState {
  masterPower: boolean;
  selectedZone: number;
  zones: Record<number, ZoneLightConfig>;
}
