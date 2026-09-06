<script setup lang="ts">
import { computed } from "vue";
import { Cpu, Flame, Wind, Activity, Zap, ShieldCheck } from "lucide-vue-next";
import type { TelemetryData } from "../types/hardware";

const props = defineProps<{
  telemetry: TelemetryData;
}>();

const emit = defineEmits<{
  (e: "toggleFullSpeed"): void;
  (e: "changeMode", mode: number): void;
}>();

const maxFanRpm = 5000;

const cpuFanPercent = computed(() => {
  return Math.min(100, Math.round((props.telemetry.cpu_fan_rpm / maxFanRpm) * 100));
});

const gpuFanPercent = computed(() => {
  return Math.min(100, Math.round((props.telemetry.gpu_fan_rpm / maxFanRpm) * 100));
});

// Calculate SVG circle dashoffset (circumference = 2 * PI * 68 ≈ 427.25)
const circumference = 2 * Math.PI * 68;
const cpuStrokeDashoffset = computed(() => {
  return circumference - (cpuFanPercent.value / 100) * circumference * 0.75;
});
const gpuStrokeDashoffset = computed(() => {
  return circumference - (gpuFanPercent.value / 100) * circumference * 0.75;
});

const getTempColor = (temp: number) => {
  if (temp < 55) return "from-emerald-500 to-teal-400 text-emerald-400";
  if (temp < 75) return "from-amber-500 to-orange-400 text-amber-400";
  return "from-rose-500 to-red-600 text-rose-500";
};
</script>

<template>
  <div class="h-full flex flex-col gap-6 p-6 overflow-y-auto">
    <!-- Top Hardware Telemetry Gauges -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- CPU Fan Card -->
      <div class="glass-panel rounded-2xl p-6 relative overflow-hidden border border-white/5 flex items-center justify-between shadow-xl">
        <div class="space-y-4">
          <div class="flex items-center gap-2.5">
            <div class="p-2 rounded-xl bg-orange-500/10 text-orange-400 border border-orange-500/20">
              <Wind class="w-5 h-5 animate-spin" :style="{ animationDuration: `${Math.max(0.4, 3000 / (telemetry.cpu_fan_rpm || 1000))}s` }" />
            </div>
            <div>
              <h3 class="text-base font-bold text-white tracking-wide">CPU 散流风扇</h3>
              <p class="text-xs text-gray-400">实时转速监测与温控</p>
            </div>
          </div>

          <div>
            <div class="text-4xl font-extrabold tracking-tight text-white font-mono flex items-baseline gap-1">
              {{ telemetry.cpu_fan_rpm }}
              <span class="text-xs font-normal text-gray-400">RPM</span>
            </div>
            <div class="text-xs text-gray-400 mt-1 flex items-center gap-2">
              <span>负载负荷率:</span>
              <span class="font-mono text-orange-400 font-semibold">{{ cpuFanPercent }}%</span>
            </div>
          </div>

          <div class="flex items-center gap-3 text-xs text-gray-300">
            <span class="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-white/5 border border-white/10">
              <Flame class="w-3.5 h-3.5 text-orange-400" />
              温度: <strong class="font-mono text-white">{{ telemetry.cpu_temp }}°C</strong>
            </span>
            <span class="px-2 py-0.5 rounded text-[11px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              正常运转
            </span>
          </div>
        </div>

        <!-- Tachometer Radial Gauge -->
        <div class="relative w-40 h-40 flex items-center justify-center shrink-0">
          <svg class="w-full h-full -rotate-135 transform" viewBox="0 0 160 160">
            <!-- Background Arc -->
            <circle
              cx="80"
              cy="80"
              r="68"
              class="stroke-[#21262d]"
              stroke-width="12"
              fill="transparent"
              :stroke-dasharray="circumference"
              :stroke-dashoffset="circumference * 0.25"
              stroke-linecap="round"
            />
            <!-- Active Value Arc -->
            <circle
              cx="80"
              cy="80"
              r="68"
              class="stroke-orange-500 transition-all duration-500 ease-out"
              stroke-width="12"
              fill="transparent"
              :stroke-dasharray="circumference"
              :stroke-dashoffset="cpuStrokeDashoffset"
              stroke-linecap="round"
            />
          </svg>
          <div class="absolute flex flex-col items-center justify-center pointer-events-none text-center">
            <span class="text-2xl font-black text-white font-mono">{{ cpuFanPercent }}%</span>
            <span class="text-[10px] text-gray-400 uppercase tracking-wider">Fan Rate</span>
          </div>
        </div>
      </div>

      <!-- GPU Fan Card -->
      <div class="glass-panel rounded-2xl p-6 relative overflow-hidden border border-white/5 flex items-center justify-between shadow-xl">
        <div class="space-y-4">
          <div class="flex items-center gap-2.5">
            <div class="p-2 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
              <Wind class="w-5 h-5 animate-spin" :style="{ animationDuration: `${Math.max(0.4, 3000 / (telemetry.gpu_fan_rpm || 1000))}s` }" />
            </div>
            <div>
              <h3 class="text-base font-bold text-white tracking-wide">GPU 独立显卡风扇</h3>
              <p class="text-xs text-gray-400">独立供电温控监测</p>
            </div>
          </div>

          <div>
            <div class="text-4xl font-extrabold tracking-tight text-white font-mono flex items-baseline gap-1">
              {{ telemetry.gpu_fan_rpm }}
              <span class="text-xs font-normal text-gray-400">RPM</span>
            </div>
            <div class="text-xs text-gray-400 mt-1 flex items-center gap-2">
              <span>负载负荷率:</span>
              <span class="font-mono text-cyan-400 font-semibold">{{ gpuFanPercent }}%</span>
            </div>
          </div>

          <div class="flex items-center gap-3 text-xs text-gray-300">
            <span class="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-white/5 border border-white/10">
              <Flame class="w-3.5 h-3.5 text-cyan-400" />
              温度: <strong class="font-mono text-white">{{ telemetry.gpu_temp }}°C</strong>
            </span>
            <span class="px-2 py-0.5 rounded text-[11px] bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
              温控良好
            </span>
          </div>
        </div>

        <!-- Tachometer Radial Gauge -->
        <div class="relative w-40 h-40 flex items-center justify-center shrink-0">
          <svg class="w-full h-full -rotate-135 transform" viewBox="0 0 160 160">
            <!-- Background Arc -->
            <circle
              cx="80"
              cy="80"
              r="68"
              class="stroke-[#21262d]"
              stroke-width="12"
              fill="transparent"
              :stroke-dasharray="circumference"
              :stroke-dashoffset="circumference * 0.25"
              stroke-linecap="round"
            />
            <!-- Active Value Arc -->
            <circle
              cx="80"
              cy="80"
              r="68"
              class="stroke-cyan-500 transition-all duration-500 ease-out"
              stroke-width="12"
              fill="transparent"
              :stroke-dasharray="circumference"
              :stroke-dashoffset="gpuStrokeDashoffset"
              stroke-linecap="round"
            />
          </svg>
          <div class="absolute flex flex-col items-center justify-center pointer-events-none text-center">
            <span class="text-2xl font-black text-white font-mono">{{ gpuFanPercent }}%</span>
            <span class="text-[10px] text-gray-400 uppercase tracking-wider">GPU Fan</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick Mode Switcher & Cooling Override Bar -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
      <!-- Fan Full Speed (强冷) Banner -->
      <button
        @click="emit('toggleFullSpeed')"
        class="glass-card p-4 rounded-xl border transition-all duration-200 flex items-center justify-between text-left group hover:border-orange-500/40"
        :class="[telemetry.fan_full_speed ? 'border-orange-500/80 bg-orange-500/20 glow-orange' : 'border-white/5']"
      >
        <div class="flex items-center gap-3">
          <div
            class="w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-300"
            :class="[telemetry.fan_full_speed ? 'bg-orange-500 text-black shadow-lg shadow-orange-500/50' : 'bg-white/5 text-gray-400 group-hover:text-orange-400']"
          >
            <Zap class="w-5 h-5" :class="{ 'animate-bounce': telemetry.fan_full_speed }" />
          </div>
          <div>
            <div class="text-sm font-bold text-white">风扇全速 (强冷)</div>
            <div class="text-xs text-gray-400">{{ telemetry.fan_full_speed ? "已开启极致散热" : "EC 自动智能温控" }}</div>
          </div>
        </div>
        <div class="text-xs px-2 py-1 rounded font-mono font-bold" :class="telemetry.fan_full_speed ? 'bg-orange-500 text-black' : 'bg-white/10 text-gray-400'">
          {{ telemetry.fan_full_speed ? "ON" : "OFF" }}
        </div>
      </button>

      <!-- Office Mode -->
      <button
        @click="emit('changeMode', 1)"
        class="glass-card p-4 rounded-xl border transition-all duration-200 flex items-center justify-between text-left group hover:border-emerald-500/40"
        :class="[telemetry.power_mode === 1 ? 'border-emerald-500/80 bg-emerald-500/10' : 'border-white/5']"
      >
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg flex items-center justify-center bg-emerald-500/10 text-emerald-400">
            <ShieldCheck class="w-5 h-5" />
          </div>
          <div>
            <div class="text-sm font-bold text-white">办公模式</div>
            <div class="text-xs text-gray-400">静音 / 低能耗</div>
          </div>
        </div>
        <span v-if="telemetry.power_mode === 1" class="w-2 h-2 rounded-full bg-emerald-400"></span>
      </button>

      <!-- Gaming Mode -->
      <button
        @click="emit('changeMode', 2)"
        class="glass-card p-4 rounded-xl border transition-all duration-200 flex items-center justify-between text-left group hover:border-amber-500/40"
        :class="[telemetry.power_mode === 2 ? 'border-amber-500/80 bg-amber-500/10' : 'border-white/5']"
      >
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg flex items-center justify-center bg-amber-500/10 text-amber-400">
            <Activity class="w-5 h-5" />
          </div>
          <div>
            <div class="text-sm font-bold text-white">游戏模式</div>
            <div class="text-xs text-gray-400">性能与发热均衡</div>
          </div>
        </div>
        <span v-if="telemetry.power_mode === 2" class="w-2 h-2 rounded-full bg-amber-400"></span>
      </button>

      <!-- Turbo Mode -->
      <button
        @click="emit('changeMode', 3)"
        class="glass-card p-4 rounded-xl border transition-all duration-200 flex items-center justify-between text-left group hover:border-orange-500/40"
        :class="[telemetry.power_mode === 3 ? 'border-orange-500/80 bg-orange-500/20' : 'border-white/5']"
      >
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg flex items-center justify-center bg-orange-500/10 text-orange-500">
            <Flame class="w-5 h-5" />
          </div>
          <div>
            <div class="text-sm font-bold text-white">狂暴模式</div>
            <div class="text-xs text-gray-400">满功耗释放性能</div>
          </div>
        </div>
        <span v-if="telemetry.power_mode === 3" class="w-2 h-2 rounded-full bg-orange-500"></span>
      </button>
    </div>

    <!-- Temperature Thermal Bars Details -->
    <div class="glass-panel rounded-2xl p-6 border border-white/5 space-y-4">
      <h3 class="text-sm font-semibold text-gray-300 uppercase tracking-wider flex items-center gap-2">
        <Cpu class="w-4 h-4 text-orange-400" />
        硬件核心热敏状态
      </h3>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
        <!-- CPU Temp Bar -->
        <div class="space-y-2">
          <div class="flex justify-between text-xs">
            <span class="text-gray-400">处理器 (CPU)</span>
            <span class="font-mono font-bold text-white">{{ telemetry.cpu_temp }} °C</span>
          </div>
          <div class="h-2.5 w-full bg-white/5 rounded-full overflow-hidden p-0.5 border border-white/5">
            <div
              class="h-full rounded-full transition-all duration-300 bg-gradient-to-r"
              :class="getTempColor(telemetry.cpu_temp)"
              :style="{ width: `${Math.min(100, Math.max(10, telemetry.cpu_temp))}%` }"
            ></div>
          </div>
        </div>

        <!-- GPU Temp Bar -->
        <div class="space-y-2">
          <div class="flex justify-between text-xs">
            <span class="text-gray-400">图形显卡 (GPU)</span>
            <span class="font-mono font-bold text-white">{{ telemetry.gpu_temp }} °C</span>
          </div>
          <div class="h-2.5 w-full bg-white/5 rounded-full overflow-hidden p-0.5 border border-white/5">
            <div
              class="h-full rounded-full transition-all duration-300 bg-gradient-to-r"
              :class="getTempColor(telemetry.gpu_temp)"
              :style="{ width: `${Math.min(100, Math.max(10, telemetry.gpu_temp))}%` }"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
