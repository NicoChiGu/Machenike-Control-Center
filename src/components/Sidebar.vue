<script setup lang="ts">
import { Gauge, Cpu, Sparkles, Sliders, RefreshCw, Flame, Activity } from "lucide-vue-next";

withDefaults(
  defineProps<{
    activeTab: string;
    powerMode: number;
    cpuTemp: number;
    gpuTemp: number;
    isRefreshing?: boolean;
  }>(),
  {
    isRefreshing: false,
  }
);

const emit = defineEmits<{
  (e: "update:activeTab", tab: string): void;
  (e: "refresh"): void;
}>();

const tabs = [
  { id: "dashboard", label: "仪表监控", icon: Gauge, desc: "风扇与硬件实时图表" },
  { id: "modes", label: "性能模式", icon: Cpu, desc: "办公 / 游戏 / 狂暴切换" },
  { id: "lighting", label: "灯光工坊", icon: Sparkles, desc: "4 分区 RGB 键盘灯效" },
  { id: "settings", label: "快捷设置", icon: Sliders, desc: "Win 锁、强冷与供电" },
];

const getModeName = (mode: number) => {
  switch (mode) {
    case 1:
      return "办公模式";
    case 2:
      return "游戏模式";
    case 3:
      return "狂暴模式";
    default:
      return "标准模式";
  }
};

const getModeColor = (mode: number) => {
  switch (mode) {
    case 1:
      return "text-emerald-400 border-emerald-500/30 bg-emerald-500/10";
    case 2:
      return "text-amber-400 border-amber-500/30 bg-amber-500/10";
    case 3:
      return "text-orange-500 border-orange-500/40 bg-orange-500/20 shadow-[0_0_12px_rgba(249,115,22,0.3)]";
    default:
      return "text-gray-400 border-gray-700 bg-gray-800/30";
  }
};
</script>

<template>
  <aside
    class="w-64 h-full glass-panel border-r border-[#30363d]/80 flex flex-col justify-between select-none shrink-0 bg-[#0d1117]/95 z-20"
  >
    <!-- Top: Brand Header -->
    <div class="p-5 border-b border-white/5">
      <div class="flex items-center gap-3">
        <div
          class="w-10 h-10 rounded-xl bg-gradient-to-br from-orange-500 to-amber-600 flex items-center justify-center font-black text-black tracking-tighter text-xl shadow-lg shadow-orange-500/30 shrink-0"
        >
          M
        </div>
        <div class="min-w-0">
          <div class="flex items-center gap-1.5">
            <span class="font-extrabold tracking-wider text-base text-white truncate">MACHENIKE</span>
            <span
              class="text-[9px] font-semibold uppercase px-1.5 py-0.5 rounded bg-orange-500/20 text-orange-400 border border-orange-500/30 shrink-0"
            >
              L16W
            </span>
          </div>
          <div class="text-[11px] text-gray-400 truncate">控制中心 (Next-Gen)</div>
        </div>
      </div>
    </div>

    <!-- Middle: Vertical Navigation Menu -->
    <nav class="flex-1 py-5 px-3 space-y-2 overflow-y-auto">
      <button
        v-for="t in tabs"
        :key="t.id"
        @click="emit('update:activeTab', t.id)"
        class="w-full flex items-center gap-3.5 px-3.5 py-3 rounded-xl font-medium text-sm transition-all duration-200 group relative text-left"
        :class="[
          activeTab === t.id
            ? 'bg-gradient-to-r from-orange-500/20 via-orange-500/10 to-transparent text-white border border-orange-500/30 shadow-lg shadow-orange-500/5'
            : 'text-gray-400 hover:text-gray-100 hover:bg-white/[0.05] border border-transparent',
        ]"
      >
        <!-- Active Left Indicator Bar -->
        <span
          v-if="activeTab === t.id"
          class="absolute left-0 top-2.5 bottom-2.5 w-1 rounded-r-full bg-orange-500 shadow-[0_0_10px_#f97316]"
        ></span>

        <div
          class="p-2 rounded-lg transition-colors duration-200 shrink-0"
          :class="[
            activeTab === t.id
              ? 'bg-orange-500 text-black shadow-md shadow-orange-500/30'
              : 'bg-white/5 text-gray-400 group-hover:text-white group-hover:bg-white/10',
          ]"
        >
          <component :is="t.icon" class="w-4 h-4" />
        </div>

        <div class="min-w-0">
          <div class="text-sm font-semibold truncate">{{ t.label }}</div>
          <div class="text-[10px] text-gray-500 group-hover:text-gray-400 transition-colors truncate">
            {{ t.desc }}
          </div>
        </div>
      </button>
    </nav>

    <!-- Bottom: Hardware Quick Summary & Manual Refresh -->
    <div class="p-3 border-t border-white/5 space-y-3">
      <!-- Hardware Status Card -->
      <div class="p-3.5 rounded-xl bg-black/40 border border-white/5 space-y-3">
        <!-- Power Mode Line -->
        <div class="flex items-center justify-between">
          <span class="text-[11px] text-gray-400 flex items-center gap-1">
            <Activity class="w-3.5 h-3.5 text-gray-400" />
            当前状态
          </span>
          <span
            class="text-[11px] px-2.5 py-0.5 rounded-full border font-medium flex items-center gap-1.5"
            :class="getModeColor(powerMode)"
          >
            <span class="w-1.5 h-1.5 rounded-full animate-pulse bg-current"></span>
            {{ getModeName(powerMode) }}
          </span>
        </div>

        <!-- CPU / GPU Temperatures -->
        <div class="grid grid-cols-2 gap-2">
          <div class="px-2.5 py-1.5 rounded-lg bg-white/[0.03] border border-white/5">
            <div class="text-[10px] text-gray-400 flex items-center gap-1">
              <Flame class="w-3 h-3 text-orange-400" />
              CPU 温度
            </div>
            <div class="text-sm font-mono font-bold text-orange-400 mt-0.5">
              {{ cpuTemp }} <span class="text-[10px] font-normal text-gray-400">°C</span>
            </div>
          </div>
          <div class="px-2.5 py-1.5 rounded-lg bg-white/[0.03] border border-white/5">
            <div class="text-[10px] text-gray-400 flex items-center gap-1">
              <Flame class="w-3 h-3 text-cyan-400" />
              GPU 温度
            </div>
            <div class="text-sm font-mono font-bold text-cyan-400 mt-0.5">
              {{ gpuTemp }} <span class="text-[10px] font-normal text-gray-400">°C</span>
            </div>
          </div>
        </div>

        <!-- Manual Refresh Button (Works alongside 3s anti-stacking auto polling) -->
        <button
          @click="emit('refresh')"
          :disabled="isRefreshing"
          class="w-full flex items-center justify-center gap-2 py-2 px-3 rounded-lg bg-white/5 hover:bg-orange-500/15 text-gray-300 hover:text-orange-400 border border-white/10 hover:border-orange-500/30 transition-all duration-200 text-xs font-medium cursor-pointer active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <RefreshCw
            class="w-3.5 h-3.5"
            :class="{ 'animate-spin text-orange-400': isRefreshing }"
          />
          <span>{{ isRefreshing ? "正在刷新..." : "刷新状态" }}</span>
        </button>
      </div>

      <!-- Footer Info -->
      <div class="text-[10px] text-gray-500 text-center tracking-wider font-mono">
        Machenike L16W · v1.0.0
      </div>
    </div>
  </aside>
</template>
