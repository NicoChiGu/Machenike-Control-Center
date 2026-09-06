<script setup lang="ts">
import { ShieldCheck, Gamepad2, Flame, Zap, CheckCircle2 } from "lucide-vue-next";

defineProps<{
  currentMode: number;
  fanFullSpeed: boolean;
}>();

const emit = defineEmits<{
  (e: "selectMode", mode: number): void;
  (e: "toggleFullSpeed"): void;
}>();

const modes = [
  {
    id: 1,
    name: "办公模式 (Office)",
    desc: "针对文档办公、代码编写与影音播放优化。降低 CPU 瞬时功耗上限，风扇静音平缓，减少发热与风噪。",
    color: "emerald",
    icon: ShieldCheck,
    badges: ["静音低噪", "能效优先", "长续航"],
    specs: {
      powerLimit: "低功耗输出",
      noise: "极低 (约 25-32 dB)",
      fanCurve: "静音舒缓策略",
    }
  },
  {
    id: 2,
    name: "游戏模式 (Gaming)",
    desc: "系统出厂均衡模式。动态平衡 CPU/GPU 负载与温控风扇转速，适合绝大多数日常网络游戏与多任务处理。",
    color: "amber",
    icon: Gamepad2,
    badges: ["动态功耗", "性能均衡", "官方推荐"],
    specs: {
      powerLimit: "标准功耗释放",
      noise: "适中 (约 38-45 dB)",
      fanCurve: "动态智能响应",
    }
  },
  {
    id: 3,
    name: "狂暴模式 (Turbo)",
    desc: "极限性能释放模式。解锁处理器与独显的最大功耗墙（PL2/TGP），温控风扇积极介入，专为大型 3A 游戏与重载渲染设计。",
    color: "orange",
    icon: Flame,
    badges: ["满血功耗", "极限性能", "极致释放"],
    specs: {
      powerLimit: "最高释放功率",
      noise: "高散热输出",
      fanCurve: "高转速激进策略",
    }
  },
];
</script>

<template>
  <div class="h-full flex flex-col gap-6 p-6 overflow-y-auto">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-extrabold text-white tracking-wide">系统性能与功耗模式</h2>
        <p class="text-xs text-gray-400 mt-1">切换嵌入式控制器 (EC) 电源配置文件与硬件温控策略</p>
      </div>

      <!-- Quick Full Speed Button -->
      <button
        @click="emit('toggleFullSpeed')"
        class="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold border transition-all duration-200"
        :class="[
          fanFullSpeed
            ? 'bg-orange-500 text-black border-orange-400 shadow-lg shadow-orange-500/40'
            : 'bg-white/5 text-gray-300 border-white/10 hover:border-orange-500/40'
        ]"
      >
        <Zap class="w-4 h-4" />
        <span>{{ fanFullSpeed ? "强冷全速已开启" : "开启强冷 (一键全速)" }}</span>
      </button>
    </div>

    <!-- Mode Cards Grid -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 flex-1">
      <div
        v-for="m in modes"
        :key="m.id"
        @click="emit('selectMode', m.id)"
        class="glass-panel rounded-2xl p-6 border transition-all duration-300 cursor-pointer flex flex-col justify-between group hover:scale-[1.015]"
        :class="[
          currentMode === m.id
            ? m.color === 'emerald'
              ? 'border-emerald-500/80 bg-emerald-500/10 shadow-lg shadow-emerald-500/10'
              : m.color === 'amber'
              ? 'border-amber-500/80 bg-amber-500/10 shadow-lg shadow-amber-500/10'
              : 'border-orange-500/80 bg-orange-500/15 shadow-xl shadow-orange-500/20 glow-orange'
            : 'border-white/5 hover:border-white/20'
        ]"
      >
        <div class="space-y-4">
          <!-- Card Header -->
          <div class="flex items-center justify-between">
            <div
              class="w-12 h-12 rounded-xl flex items-center justify-center transition-all duration-200"
              :class="[
                currentMode === m.id
                  ? m.color === 'emerald' ? 'bg-emerald-500 text-black' : m.color === 'amber' ? 'bg-amber-500 text-black' : 'bg-orange-500 text-black shadow-lg shadow-orange-500/40'
                  : 'bg-white/5 text-gray-400 group-hover:text-white'
              ]"
            >
              <component :is="m.icon" class="w-6 h-6" />
            </div>

            <div v-if="currentMode === m.id" class="flex items-center gap-1 text-xs font-bold" :class="m.color === 'emerald' ? 'text-emerald-400' : m.color === 'amber' ? 'text-amber-400' : 'text-orange-400'">
              <CheckCircle2 class="w-4 h-4" />
              当前启用
            </div>
          </div>

          <!-- Title & Description -->
          <div>
            <h3 class="text-lg font-extrabold text-white">{{ m.name }}</h3>
            <p class="text-xs text-gray-400 mt-2 leading-relaxed">{{ m.desc }}</p>
          </div>

          <!-- Feature Badges -->
          <div class="flex flex-wrap gap-1.5 pt-1">
            <span
              v-for="b in m.badges"
              :key="b"
              class="text-[10px] font-medium px-2 py-0.5 rounded-full border"
              :class="[
                currentMode === m.id
                  ? 'bg-white/10 text-white border-white/20'
                  : 'bg-white/5 text-gray-400 border-white/5'
              ]"
            >
              {{ b }}
            </span>
          </div>
        </div>

        <!-- Specifications Breakdown -->
        <div class="pt-6 border-t border-white/5 space-y-2 text-xs">
          <div class="flex justify-between text-gray-400">
            <span>功率策略</span>
            <span class="text-white font-medium">{{ m.specs.powerLimit }}</span>
          </div>
          <div class="flex justify-between text-gray-400">
            <span>噪声水平</span>
            <span class="text-white font-medium">{{ m.specs.noise }}</span>
          </div>
          <div class="flex justify-between text-gray-400">
            <span>转速逻辑</span>
            <span class="text-white font-medium">{{ m.specs.fanCurve }}</span>
          </div>

          <button
            class="w-full mt-4 py-2.5 rounded-xl font-bold text-xs uppercase tracking-wider transition-all duration-200"
            :class="[
              currentMode === m.id
                ? 'bg-white text-black shadow-md'
                : 'bg-white/5 text-gray-300 hover:bg-white/10 hover:text-white'
            ]"
          >
            {{ currentMode === m.id ? "已选定" : "点击切换" }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
