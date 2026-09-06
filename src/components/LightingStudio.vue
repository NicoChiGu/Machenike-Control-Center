<script setup lang="ts">
import { ref, computed } from "vue";
import { LIGHT_EFFECTS, LIGHT_ZONES } from "../types/hardware";
import {
  Sparkles,
  Sun,
  Gauge,
  ArrowLeftRight,
  Check,
  Power,
  Layers,
  RefreshCw,
  Eye,
} from "lucide-vue-next";
import {
  lightingState,
  toggleMasterPower,
  toggleZonePower,
  applyZoneToHardware,
} from "../services/lightingStore";

const emit = defineEmits<{
  (e: "toast", msg: string): void;
}>();

// 当前键盘 4 分区聚焦索引 (仅在键盘背光下有效, null 表示整区同步, 0..3 表示 A/B/C/D)
const selectedSubZone = ref<number | null>(null);
const isApplying = ref(false);

// 当前选中的灯控区域配置 (响应式计算属性)
const currentZone = computed(() => {
  return lightingState.zones[lightingState.selectedZone] || lightingState.zones[4];
});

// 当前正在编辑的色彩 (如果是键盘单选分区，指向该分区色彩；否则指向本区域全局色彩)
const activeColor = computed({
  get() {
    if (
      lightingState.selectedZone === 0 &&
      selectedSubZone.value !== null &&
      currentZone.value.subZones
    ) {
      return currentZone.value.subZones[selectedSubZone.value];
    }
    return currentZone.value.color;
  },
  set(val) {
    if (
      lightingState.selectedZone === 0 &&
      selectedSubZone.value !== null &&
      currentZone.value.subZones
    ) {
      currentZone.value.subZones[selectedSubZone.value] = { ...val };
    } else {
      currentZone.value.color = { ...val };
      // 若为键盘整区调色，同步更新 4 个分区的存储值
      if (lightingState.selectedZone === 0 && currentZone.value.subZones) {
        currentZone.value.subZones = currentZone.value.subZones.map(() => ({ ...val })) as [
          any,
          any,
          any,
          any
        ];
      }
    }
  },
});

const currentZoneName = computed(() => {
  return LIGHT_ZONES.find((z) => z.id === lightingState.selectedZone)?.name || "灯控区域";
});

// 预设色彩选项
const colorPresets = [
  { name: "机械师橙", r: 255, g: 107, b: 0, hex: "#ff6b00" },
  { name: "曜石金", r: 255, g: 186, b: 0, hex: "#ffba00" },
  { name: "极光青", r: 0, g: 230, b: 255, hex: "#00e6ff" },
  { name: "赛博蓝", r: 48, g: 79, b: 254, hex: "#304ffe" },
  { name: "霓虹紫", r: 170, g: 0, b: 255, hex: "#aa00ff" },
  { name: "电竞红", r: 255, g: 23, b: 68, hex: "#ff1744" },
  { name: "纯净白", r: 255, g: 255, b: 255, hex: "#ffffff" },
  { name: "冰川绿", r: 0, g: 230, b: 118, hex: "#00e676" },
];

/**
 * 切换控制区域：读取并展现该区域独立配置，绝不下发覆盖指令
 */
const selectZone = (zoneId: number) => {
  lightingState.selectedZone = zoneId;
  selectedSubZone.value = null; // 切区时重置键盘子分区聚焦
};

/**
 * 键盘 4 分区子分区点击
 */
const selectSubZone = (idx: number) => {
  if (selectedSubZone.value === idx) {
    selectedSubZone.value = null; // 再次点击取消聚焦，回到整区
  } else {
    selectedSubZone.value = idx;
    lightingState.selectedZone = 0; // 确保当前大区域为键盘背光
  }
};

/**
 * 同步 4 分区为同一颜色并下发
 */
const resetSubZones = () => {
  selectedSubZone.value = null;
  if (currentZone.value.subZones) {
    const baseCol = { ...currentZone.value.color };
    currentZone.value.subZones = currentZone.value.subZones.map(() => ({ ...baseCol })) as [
      any,
      any,
      any,
      any
    ];
  }
  applyCurrentZone();
};

/**
 * 全局总开关切换
 */
const handleMasterToggle = async () => {
  try {
    const newState = await toggleMasterPower();
    emit("toast", newState ? "全机灯光与背光已开启" : "全机灯光已关闭");
  } catch (err) {
    emit("toast", "总开关切换失败: " + ((err as Error)?.message || String(err)));
  }
};

/**
 * 当前选中区域的独立开关切换 (键盘/左侧/右侧/Logo)
 */
const handleZoneToggle = async () => {
  try {
    const newState = await toggleZonePower(lightingState.selectedZone);
    emit("toast", `${currentZoneName.value} 已${newState ? "开启" : "关闭"}`);
  } catch (err) {
    emit("toast", "区域开关切换失败: " + ((err as Error)?.message || String(err)));
  }
};

/**
 * 点击预设颜色
 */
const applyPresetColor = (p: typeof colorPresets[0]) => {
  activeColor.value = { r: p.r, g: p.g, b: p.b };
  applyCurrentZone();
};

/**
 * 下发当前区域设定至硬件
 */
const applyCurrentZone = async () => {
  if (isApplying.value) return;
  isApplying.value = true;
  try {
    await applyZoneToHardware(lightingState.selectedZone, {
      is4Zone: selectedSubZone.value !== null,
    });
    emit("toast", `${currentZoneName.value} 设置已下发生效`);
  } catch (err) {
    console.error("Failed to apply zone lighting:", err);
    emit("toast", "下发失败: " + ((err as Error)?.message || String(err)));
  } finally {
    isApplying.value = false;
  }
};
</script>

<template>
  <div class="h-full flex flex-col gap-6 p-6 overflow-y-auto">
    <!-- Header: Title & Master Power Toggle -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-extrabold text-white tracking-wide flex items-center gap-2">
          <Sparkles class="w-5 h-5 text-orange-400" />
          RGB 键盘与灯光工坊
        </h2>
        <p class="text-xs text-gray-400 mt-1">控制键盘 4 分区背光、机身左右流光侧灯带与 Logo 信仰灯</p>
      </div>

      <!-- Master Power Switch (全机总开关) -->
      <button
        @click="handleMasterToggle"
        class="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold border transition-all duration-200"
        :class="[
          lightingState.masterPower
            ? 'bg-orange-500 text-black border-orange-400 shadow-md shadow-orange-500/30 hover:brightness-105'
            : 'bg-white/5 text-gray-400 border-white/10 hover:bg-white/10'
        ]"
      >
        <Power class="w-4 h-4" />
        <span>{{ lightingState.masterPower ? "全机灯光已开启" : "全机灯光已关闭" }}</span>
      </button>
    </div>

    <!-- Interactive Lighting Graphic Preview -->
    <div class="glass-panel rounded-2xl p-6 border border-white/5 flex flex-col items-center justify-center space-y-4 shadow-xl">
      <div class="w-full flex items-center justify-between text-xs text-gray-400">
        <span class="uppercase tracking-wider font-semibold flex items-center gap-1.5">
          <Layers class="w-4 h-4 text-orange-400" />
          <span v-if="lightingState.selectedZone === 4">神光同步 · 全机联动光效模型</span>
          <span v-else-if="lightingState.selectedZone === 0">键盘背光 · 4分区独立光效可视化模型</span>
          <span v-else-if="lightingState.selectedZone === 1">机身左侧流光侧灯带可视化</span>
          <span v-else-if="lightingState.selectedZone === 2">机身右侧流光侧灯带可视化</span>
          <span v-else>Logo 信仰背光灯效可视化</span>
        </span>

        <div class="flex items-center gap-2">
          <span v-if="lightingState.selectedZone === 0 && selectedSubZone !== null" class="text-orange-400 font-bold">
            正在独立配置：分区 {{ String.fromCharCode(65 + selectedSubZone) }}
          </span>
          <button
            v-if="lightingState.selectedZone === 0 && selectedSubZone !== null"
            @click="resetSubZones"
            class="flex items-center gap-1 px-2.5 py-1 rounded-md text-[11px] bg-white/10 hover:bg-white/20 text-white transition-colors"
          >
            <RefreshCw class="w-3 h-3" />
            <span>同步为统一颜色</span>
          </button>
        </div>
      </div>

      <!-- Preview Stage Container -->
      <div class="w-full max-w-2xl bg-black/60 p-5 rounded-xl border border-white/10 shadow-2xl relative">
        <!-- 1. Keyboard 4-Zone View (Active for Zone 0 or Zone 4) -->
        <div v-if="lightingState.selectedZone === 0 || lightingState.selectedZone === 4" class="grid grid-cols-4 gap-3 h-24">
          <div
            v-for="(zone, idx) in 4"
            :key="zone"
            @click="selectSubZone(idx)"
            class="rounded-lg flex flex-col items-center justify-center transition-all duration-300 relative border group cursor-pointer"
            :class="[
              selectedSubZone === idx
                ? 'ring-2 ring-orange-400 scale-[1.03] shadow-lg z-10'
                : 'hover:scale-[1.01]'
            ]"
            :style="{
              backgroundColor: `rgba(${
                lightingState.selectedZone === 4
                  ? currentZone.color.r
                  : currentZone.subZones?.[idx]?.r ?? currentZone.color.r
              }, ${
                lightingState.selectedZone === 4
                  ? currentZone.color.g
                  : currentZone.subZones?.[idx]?.g ?? currentZone.color.g
              }, ${
                lightingState.selectedZone === 4
                  ? currentZone.color.b
                  : currentZone.subZones?.[idx]?.b ?? currentZone.color.b
              }, 0.28)`,
              boxShadow: selectedSubZone === idx
                ? `0 0 30px rgba(${currentZone.subZones?.[idx]?.r ?? 255}, ${currentZone.subZones?.[idx]?.g ?? 107}, ${currentZone.subZones?.[idx]?.b ?? 0}, 0.8)`
                : `0 0 20px rgba(${currentZone.subZones?.[idx]?.r ?? 255}, ${currentZone.subZones?.[idx]?.g ?? 107}, ${currentZone.subZones?.[idx]?.b ?? 0}, 0.3)`,
              borderColor: selectedSubZone === idx
                ? '#f97316'
                : `rgba(${currentZone.subZones?.[idx]?.r ?? 255}, ${currentZone.subZones?.[idx]?.g ?? 107}, ${currentZone.subZones?.[idx]?.b ?? 0}, 0.6)`
            }"
          >
            <span class="text-xs font-bold text-white uppercase tracking-wider font-mono">
              分区 {{ String.fromCharCode(65 + idx) }}
            </span>
            <span class="text-[10px] text-gray-300 mt-1">
              {{ idx === 0 ? "QWER / 辅助键" : idx === 1 ? "主字母区" : idx === 2 ? "导航 / 方向键" : "数字小键盘" }}
            </span>
            <span
              v-if="selectedSubZone === idx"
              class="absolute top-1 right-1.5 text-[9px] px-1.5 py-0.5 rounded bg-orange-500 text-black font-extrabold"
            >
              已选
            </span>
          </div>
        </div>

        <!-- 2. Single Zone Accent View (Left light, Right light, Logo) -->
        <div v-else class="h-24 flex items-center justify-center relative overflow-hidden rounded-lg bg-black/40 border border-white/5">
          <div class="flex items-center gap-4 z-10">
            <div
              class="w-12 h-12 rounded-2xl flex items-center justify-center border border-white/20 transition-all duration-300"
              :style="{
                backgroundColor: `rgb(${currentZone.color.r}, ${currentZone.color.g}, ${currentZone.color.b})`,
                boxShadow: `0 0 25px rgba(${currentZone.color.r}, ${currentZone.color.g}, ${currentZone.color.b}, 0.7)`
              }"
            >
              <Eye class="w-6 h-6 text-black/80" />
            </div>
            <div>
              <div class="text-sm font-bold text-white flex items-center gap-2">
                <span>{{ currentZoneName }}</span>
                <span
                  class="text-[10px] px-2 py-0.5 rounded-full font-mono font-bold"
                  :class="currentZone.enabled ? 'bg-green-500/20 text-green-400 border border-green-500/30' : 'bg-red-500/20 text-red-400 border border-red-500/30'"
                >
                  {{ currentZone.enabled ? "通电运行中" : "已断电熄灭" }}
                </span>
              </div>
              <p class="text-xs text-gray-400 mt-0.5">
                当前色彩: RGB({{ currentZone.color.r }}, {{ currentZone.color.g }}, {{ currentZone.color.b }}) · 亮度: {{ Math.round((currentZone.brightness / 255) * 100) }}%
              </p>
            </div>
          </div>
        </div>

        <!-- Glow Accent bar -->
        <div
          class="h-1.5 w-3/4 mx-auto rounded-full mt-4 blur-sm transition-all duration-300"
          :style="{
            backgroundColor: `rgb(${activeColor.r}, ${activeColor.g}, ${activeColor.b})`
          }"
        ></div>
      </div>
    </div>

    <!-- Lighting Controls Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- Zone & Effect Selection -->
      <div class="glass-panel rounded-2xl p-6 border border-white/5 space-y-6">
        <!-- 1. 选择灯控区域 -->
        <div>
          <h3 class="text-sm font-bold text-white mb-3 flex items-center justify-between">
            <span>1. 选择灯控区域</span>
            <span class="text-[11px] text-gray-400 font-normal">点击切换查看对应区域配置</span>
          </h3>
          <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
            <button
              v-for="zone in LIGHT_ZONES"
              :key="zone.id"
              @click="selectZone(zone.id)"
              class="px-3 py-2.5 rounded-xl text-xs font-medium border transition-all duration-150 text-left flex items-center justify-between"
              :class="[
                lightingState.selectedZone === zone.id
                  ? 'bg-orange-500 text-black font-bold border-orange-400 shadow-md shadow-orange-500/20'
                  : 'bg-white/5 text-gray-300 border-white/5 hover:border-white/20'
              ]"
            >
              <span>{{ zone.name }}</span>
              <span
                class="w-2 h-2 rounded-full"
                :class="[
                  lightingState.zones[zone.id]?.enabled
                    ? (lightingState.selectedZone === zone.id ? 'bg-black' : 'bg-green-400 shadow-[0_0_6px_#4ade80]')
                    : 'bg-gray-500'
                ]"
                :title="lightingState.zones[zone.id]?.enabled ? '硬件已开启' : '硬件已关闭'"
              ></span>
            </button>
          </div>
        </div>

        <!-- 2. 当前选中区域的控制区及独立硬件开关 -->
        <div>
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-bold text-white flex items-center gap-2">
              <span>2. 动态灯效调节</span>
              <span class="text-xs font-normal text-orange-400">({{ currentZoneName }})</span>
            </h3>

            <!-- 区域独立开关 (Per-zone Power Toggle) -->
            <button
              @click="handleZoneToggle"
              class="flex items-center gap-1.5 px-3 py-1 rounded-lg text-xs font-bold border transition-all duration-150"
              :class="[
                currentZone.enabled
                  ? 'bg-green-500/20 text-green-300 border-green-500/40 hover:bg-green-500/30'
                  : 'bg-red-500/20 text-red-300 border-red-500/40 hover:bg-red-500/30'
              ]"
            >
              <Power class="w-3.5 h-3.5" />
              <span>{{ currentZone.enabled ? "本区已开" : "本区已关" }}</span>
            </button>
          </div>

          <!-- 动态灯效模式选择网格 -->
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
            <button
              v-for="eff in LIGHT_EFFECTS"
              :key="eff.id"
              @click="currentZone.effect = eff.id; applyCurrentZone()"
              class="px-3 py-2.5 rounded-lg text-xs font-medium border transition-all duration-150 flex flex-col items-center justify-center gap-1.5"
              :class="[
                currentZone.effect === eff.id
                  ? 'bg-gradient-to-br from-orange-500 to-amber-600 text-black font-bold border-orange-400 shadow-md'
                  : 'bg-white/5 text-gray-300 border-white/5 hover:border-white/20 hover:text-white'
              ]"
            >
              <span>{{ eff.name }}</span>
              <span class="text-[10px] opacity-75 font-mono">{{ eff.enName }}</span>
            </button>
          </div>
        </div>

        <!-- Sliders: Brightness & Speed -->
        <div class="space-y-4 pt-2 border-t border-white/5">
          <div>
            <div class="flex justify-between text-xs mb-1.5">
              <span class="text-gray-400 flex items-center gap-1.5">
                <Sun class="w-3.5 h-3.5 text-orange-400" />
                本区背光亮度
              </span>
              <span class="font-mono font-bold text-white">{{ Math.round((currentZone.brightness / 255) * 100) }}%</span>
            </div>
            <input
              type="range"
              min="0"
              max="255"
              v-model.number="currentZone.brightness"
              @change="applyCurrentZone"
              class="w-full accent-orange-500"
            />
          </div>

          <div>
            <div class="flex justify-between text-xs mb-1.5">
              <span class="text-gray-400 flex items-center gap-1.5">
                <Gauge class="w-3.5 h-3.5 text-orange-400" />
                动态流速
              </span>
              <span class="font-mono font-bold text-white">{{ currentZone.speed }} 档</span>
            </div>
            <input
              type="range"
              min="1"
              max="5"
              v-model.number="currentZone.speed"
              @change="applyCurrentZone"
              class="w-full accent-orange-500"
            />
          </div>

          <!-- Orientation Toggle for Wave & Flow -->
          <div class="flex items-center justify-between pt-1">
            <span class="text-xs text-gray-400 flex items-center gap-1.5">
              <ArrowLeftRight class="w-3.5 h-3.5 text-orange-400" />
              流动方向
            </span>
            <button
              @click="currentZone.orientation = currentZone.orientation === 0 ? 1 : 0; applyCurrentZone()"
              class="px-3 py-1 rounded-md text-xs font-mono font-bold bg-white/10 hover:bg-white/20 text-white border border-white/10"
            >
              {{ currentZone.orientation === 0 ? "正向流动 (0)" : "反向流动 (1)" }}
            </button>
          </div>
        </div>
      </div>

      <!-- Color Palette Presets & Custom RGB -->
      <div class="glass-panel rounded-2xl p-6 border border-white/5 flex flex-col justify-between space-y-6">
        <div>
          <h3 class="text-sm font-bold text-white mb-3 flex items-center justify-between">
            <span>3. 专属预设与快速色彩</span>
            <span v-if="lightingState.selectedZone === 0 && selectedSubZone !== null" class="text-xs text-orange-400 font-bold">
              (分区 {{ String.fromCharCode(65 + selectedSubZone) }})
            </span>
          </h3>

          <!-- Color Presets Grid -->
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
            <button
              v-for="p in colorPresets"
              :key="p.name"
              @click="applyPresetColor(p)"
              class="p-3 rounded-xl border border-white/5 flex flex-col items-center justify-center gap-2 group hover:border-white/20 transition-all duration-200"
              :class="{
                'border-orange-500 bg-white/5': activeColor.r === p.r && activeColor.g === p.g && activeColor.b === p.b
              }"
            >
              <div
                class="w-8 h-8 rounded-full border border-white/20 shadow-lg group-hover:scale-110 transition-transform duration-200"
                :style="{ backgroundColor: p.hex, boxShadow: `0 0 12px ${p.hex}80` }"
              ></div>
              <span class="text-[11px] font-medium text-gray-300">{{ p.name }}</span>
            </button>
          </div>

          <!-- Custom RGB Sliders -->
          <div class="mt-6 space-y-3 p-4 rounded-xl bg-black/40 border border-white/5">
            <div class="text-xs font-bold text-gray-300 flex items-center justify-between">
              <span>自定义精确 RGB 数值</span>
              <span class="font-mono text-gray-400">
                RGB({{ activeColor.r }}, {{ activeColor.g }}, {{ activeColor.b }})
              </span>
            </div>
            <div class="grid grid-cols-3 gap-3">
              <div>
                <span class="text-[10px] text-red-400 block mb-1">R: {{ activeColor.r }}</span>
                <input
                  type="range"
                  min="0"
                  max="255"
                  v-model.number="activeColor.r"
                  @change="applyCurrentZone"
                  class="w-full accent-red-500"
                />
              </div>
              <div>
                <span class="text-[10px] text-green-400 block mb-1">G: {{ activeColor.g }}</span>
                <input
                  type="range"
                  min="0"
                  max="255"
                  v-model.number="activeColor.g"
                  @change="applyCurrentZone"
                  class="w-full accent-green-500"
                />
              </div>
              <div>
                <span class="text-[10px] text-blue-400 block mb-1">B: {{ activeColor.b }}</span>
                <input
                  type="range"
                  min="0"
                  max="255"
                  v-model.number="activeColor.b"
                  @change="applyCurrentZone"
                  class="w-full accent-blue-500"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- Manual Re-apply Button -->
        <button
          @click="applyCurrentZone"
          :disabled="isApplying"
          class="w-full py-3 rounded-xl font-bold text-sm bg-gradient-to-r from-orange-500 to-amber-500 text-black shadow-lg shadow-orange-500/25 hover:brightness-110 active:scale-[0.98] transition-all flex items-center justify-center gap-2 disabled:opacity-50"
        >
          <RefreshCw v-if="isApplying" class="w-4 h-4 animate-spin" />
          <Check v-else class="w-4 h-4" />
          <span>{{ isApplying ? "正在下发指令..." : "立即下发当前区域生效" }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
