import { reactive, watch } from "vue";
import type { LightZonesState, RgbColor } from "../types/hardware";
import { hardwareApi } from "./tauri";

const STORAGE_KEY = "machenike_lighting_config_v1";

const DEFAULT_SUBZONES: [RgbColor, RgbColor, RgbColor, RgbColor] = [
  { r: 255, g: 107, b: 0 },
  { r: 255, g: 186, b: 0 },
  { r: 0, g: 230, b: 255 },
  { r: 170, g: 0, b: 255 },
];

export const DEFAULT_LIGHTING_STATE: LightZonesState = {
  masterPower: true,
  selectedZone: 4, // 默认神光同步
  zones: {
    4: {
      enabled: true,
      effect: 0,
      brightness: 128,
      speed: 3,
      orientation: 0,
      color: { r: 255, g: 107, b: 0 },
    },
    0: {
      enabled: true,
      effect: 0,
      brightness: 128,
      speed: 3,
      orientation: 0,
      color: { r: 255, g: 107, b: 0 },
      subZones: [...DEFAULT_SUBZONES],
    },
    1: {
      enabled: true,
      effect: 0,
      brightness: 128,
      speed: 3,
      orientation: 0,
      color: { r: 0, g: 230, b: 255 },
    },
    2: {
      enabled: true,
      effect: 0,
      brightness: 128,
      speed: 3,
      orientation: 0,
      color: { r: 0, g: 230, b: 255 },
    },
    3: {
      enabled: true,
      effect: 0,
      brightness: 128,
      speed: 3,
      orientation: 0,
      color: { r: 255, g: 107, b: 0 },
    },
  },
};

function loadState(): LightZonesState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return JSON.parse(JSON.stringify(DEFAULT_LIGHTING_STATE));
    const parsed = JSON.parse(raw);
    return {
      masterPower: parsed.masterPower ?? true,
      selectedZone: parsed.selectedZone ?? 4,
      zones: {
        4: { ...DEFAULT_LIGHTING_STATE.zones[4], ...(parsed.zones?.[4] || {}) },
        0: {
          ...DEFAULT_LIGHTING_STATE.zones[0],
          ...(parsed.zones?.[0] || {}),
          subZones: parsed.zones?.[0]?.subZones || [...DEFAULT_SUBZONES],
        },
        1: { ...DEFAULT_LIGHTING_STATE.zones[1], ...(parsed.zones?.[1] || {}) },
        2: { ...DEFAULT_LIGHTING_STATE.zones[2], ...(parsed.zones?.[2] || {}) },
        3: { ...DEFAULT_LIGHTING_STATE.zones[3], ...(parsed.zones?.[3] || {}) },
      },
    };
  } catch (err) {
    console.error("Failed to parse saved lighting config, falling back to default:", err);
    return JSON.parse(JSON.stringify(DEFAULT_LIGHTING_STATE));
  }
}

export const lightingState = reactive<LightZonesState>(loadState());

// 监听状态变动并自动保存至 localStorage
watch(
  lightingState,
  (newVal) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(newVal));
    } catch (err) {
      console.warn("Failed to persist lighting state to localStorage:", err);
    }
  },
  { deep: true }
);

/**
 * 切换全局总开关 (0xC001 which_light=4)
 */
export async function toggleMasterPower(): Promise<boolean> {
  lightingState.masterPower = !lightingState.masterPower;
  await hardwareApi.setLightState(4, lightingState.masterPower);
  return lightingState.masterPower;
}

/**
 * 切换某个区域的独立硬件开关 (0xC001 which_light=0..3)
 */
export async function toggleZonePower(zoneId: number): Promise<boolean> {
  const zone = lightingState.zones[zoneId];
  if (!zone) return false;
  zone.enabled = !zone.enabled;
  await hardwareApi.setLightState(zoneId, zone.enabled);

  // 如果重新开启，重新下发该区域的 RGB 与灯效参数以确保亮起
  if (zone.enabled) {
    await applyZoneToHardware(zoneId);
  }
  return zone.enabled;
}

/**
 * 将指定区域的当前配置下发至硬件 (0xC003 / 0xC00B / 0xC00A)
 */
export async function applyZoneToHardware(
  zoneId: number,
  options?: { is4Zone?: boolean }
): Promise<void> {
  const zone = lightingState.zones[zoneId];
  if (!zone) return;

  // 1. 若为键盘 (Zone 0) 且启用了 4 分区独立模式
  if (zoneId === 0 && options?.is4Zone && zone.subZones) {
    const br: [number, number, number, number] = [
      zone.brightness,
      zone.brightness,
      zone.brightness,
      zone.brightness,
    ];
    const sp: [number, number, number, number] = [
      zone.speed,
      zone.speed,
      zone.speed,
      zone.speed,
    ];
    const col: [
      [number, number, number],
      [number, number, number],
      [number, number, number],
      [number, number, number]
    ] = [
      [zone.subZones[0].r, zone.subZones[0].g, zone.subZones[0].b],
      [zone.subZones[1].r, zone.subZones[1].g, zone.subZones[1].b],
      [zone.subZones[2].r, zone.subZones[2].g, zone.subZones[2].b],
      [zone.subZones[3].r, zone.subZones[3].g, zone.subZones[3].b],
    ];

    await hardwareApi.setFourAreas(br, sp, col);
    await hardwareApi.setFourAreaEffects([
      zone.effect,
      zone.effect,
      zone.effect,
      zone.effect,
    ]);
    return;
  }

  // 2. 正常单区域 / 神光同步下发 (0xC003)
  await hardwareApi.setLightRgb(
    zoneId,
    zone.effect,
    zone.brightness,
    zone.speed,
    zone.orientation,
    zone.color.r,
    zone.color.g,
    zone.color.b
  );

  // 若为键盘单色模式，保底同步 4 分区色彩避免分区残留
  if (zoneId === 0 && zone.subZones) {
    const br: [number, number, number, number] = [
      zone.brightness,
      zone.brightness,
      zone.brightness,
      zone.brightness,
    ];
    const sp: [number, number, number, number] = [
      zone.speed,
      zone.speed,
      zone.speed,
      zone.speed,
    ];
    const col: [
      [number, number, number],
      [number, number, number],
      [number, number, number],
      [number, number, number]
    ] = [
      [zone.color.r, zone.color.g, zone.color.b],
      [zone.color.r, zone.color.g, zone.color.b],
      [zone.color.r, zone.color.g, zone.color.b],
      [zone.color.r, zone.color.g, zone.color.b],
    ];
    await hardwareApi.setFourAreas(br, sp, col);
  }
}
