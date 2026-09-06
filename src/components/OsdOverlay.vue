<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { hardwareApi } from "../services/tauri";
import type { OsdPayload } from "../types/hardware";
import type { UnlistenFn } from "@tauri-apps/api/event";

const visible = ref(false);
const payload = ref<OsdPayload>({
  kind: "caps_lock",
  title: "大写锁定",
  subtitle: "CAPS LOCK",
  status: "已开启",
  active: true,
  icon: "caps",
});

let unlisten: UnlistenFn | null = null;
let hideTimeout: any = null;
let fadeTimeout: any = null;
const progressKey = ref(0);

const themeClass = computed(() => {
  const p = payload.value;
  if (p.kind === "caps_lock") {
    return p.active
      ? {
          border: "border-cyan-500/50 shadow-cyan-500/25",
          iconBg: "bg-cyan-500/20 text-cyan-400 border border-cyan-500/30",
          tagBg: "bg-cyan-500/20 text-cyan-300 border-cyan-500/30",
          bar: "bg-gradient-to-r from-cyan-500 to-blue-500",
        }
      : {
          border: "border-gray-700/60 shadow-black/40",
          iconBg: "bg-gray-800 text-gray-400 border border-gray-700",
          tagBg: "bg-gray-800/80 text-gray-400 border-gray-700",
          bar: "bg-gray-600",
        };
  }
  if (p.kind === "num_lock") {
    return p.active
      ? {
          border: "border-emerald-500/50 shadow-emerald-500/25",
          iconBg: "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30",
          tagBg: "bg-emerald-500/20 text-emerald-300 border-emerald-500/30",
          bar: "bg-gradient-to-r from-emerald-500 to-teal-400",
        }
      : {
          border: "border-gray-700/60 shadow-black/40",
          iconBg: "bg-gray-800 text-gray-400 border border-gray-700",
          tagBg: "bg-gray-800/80 text-gray-400 border-gray-700",
          bar: "bg-gray-600",
        };
  }
  if (p.kind === "mode") {
    if (p.icon === "office") {
      return {
        border: "border-emerald-500/50 shadow-emerald-500/25",
        iconBg: "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30",
        tagBg: "bg-emerald-500/20 text-emerald-300 border-emerald-500/30",
        bar: "bg-gradient-to-r from-emerald-400 to-green-500",
      };
    }
    if (p.icon === "gaming") {
      return {
        border: "border-indigo-500/50 shadow-indigo-500/25",
        iconBg: "bg-indigo-500/20 text-indigo-400 border border-indigo-500/30",
        tagBg: "bg-indigo-500/20 text-indigo-300 border-indigo-500/30",
        bar: "bg-gradient-to-r from-indigo-500 to-purple-500",
      };
    }
    return {
      border: "border-orange-500/60 shadow-orange-500/30",
      iconBg: "bg-orange-500/20 text-orange-400 border border-orange-500/30",
      tagBg: "bg-orange-500/20 text-orange-300 border-orange-500/30",
      bar: "bg-gradient-to-r from-orange-500 via-amber-500 to-red-500",
    };
  }
  if (p.kind === "fan") {
    return p.active
      ? {
          border: "border-sky-400/60 shadow-sky-400/30",
          iconBg: "bg-sky-500/20 text-sky-400 border border-sky-400/40",
          tagBg: "bg-sky-500/20 text-sky-300 border-sky-400/40",
          bar: "bg-gradient-to-r from-sky-400 to-blue-600",
        }
      : {
          border: "border-gray-700/60 shadow-black/40",
          iconBg: "bg-gray-800 text-gray-400 border border-gray-700",
          tagBg: "bg-gray-800/80 text-gray-400 border-gray-700",
          bar: "bg-gray-600",
        };
  }
  if (p.kind === "win_lock") {
    return p.active
      ? {
          border: "border-amber-500/50 shadow-amber-500/25",
          iconBg: "bg-amber-500/20 text-amber-400 border border-amber-500/30",
          tagBg: "bg-amber-500/20 text-amber-300 border-amber-500/30",
          bar: "bg-gradient-to-r from-amber-400 to-yellow-500",
        }
      : {
          border: "border-gray-700/60 shadow-black/40",
          iconBg: "bg-gray-800 text-gray-400 border border-gray-700",
          tagBg: "bg-gray-800/80 text-gray-400 border-gray-700",
          bar: "bg-gray-600",
        };
  }

  return {
    border: "border-white/10 shadow-black/40",
    iconBg: "bg-gray-800 text-gray-300 border border-gray-700",
    tagBg: "bg-gray-800/80 text-gray-300 border-gray-700",
    bar: "bg-orange-500",
  };
});

const handleTrigger = (data: OsdPayload) => {
  // 清理现有所有隐藏与淡出定时器，确保完全重置
  if (hideTimeout) {
    clearTimeout(hideTimeout);
    hideTimeout = null;
  }
  if (fadeTimeout) {
    clearTimeout(fadeTimeout);
    fadeTimeout = null;
  }

  payload.value = data;
  visible.value = true;
  progressKey.value += 1;

  // 重新开始完整的 3000ms 倒计时，连续触发时完全续期
  hideTimeout = setTimeout(() => {
    visible.value = false;
    fadeTimeout = setTimeout(() => {
      hardwareApi.hideOsd().catch(() => {});
      fadeTimeout = null;
    }, 250);
    hideTimeout = null;
  }, 3000);
};

onMounted(async () => {
  try {
    unlisten = await hardwareApi.onOsdEvent((data) => {
      handleTrigger(data);
    });
  } catch (err) {
    console.error("[OSD] Failed to listen for osd-event:", err);
  }
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  if (hideTimeout) {
    clearTimeout(hideTimeout);
  }
});
</script>

<template>
  <div class="w-screen h-screen overflow-hidden flex items-center justify-center select-none bg-transparent pointer-events-none">
    <Transition name="osd">
      <div
        v-if="visible"
        class="relative w-[320px] h-[96px] rounded-2xl bg-[#0b0e14]/90 backdrop-blur-2xl border shadow-2xl p-3 flex flex-col justify-between overflow-hidden transition-all duration-300"
        :class="themeClass.border"
      >
        <!-- Card Main Content -->
        <div class="flex items-center gap-3.5">
          <!-- Icon Badge -->
          <div
            class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0 shadow-inner transition-all duration-300"
            :class="themeClass.iconBg"
          >
            <!-- Caps Lock Icon -->
            <svg
              v-if="payload.kind === 'caps_lock'"
              class="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                v-if="payload.active"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
              />
              <path
                v-else
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z"
              />
            </svg>

            <!-- Num Lock Icon -->
            <svg
              v-else-if="payload.kind === 'num_lock'"
              class="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M7 20l4-16m2 16l4-16M6 9h14M4 15h14"
              />
            </svg>

            <!-- Mode Office Icon -->
            <svg
              v-else-if="payload.icon === 'office'"
              class="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"
              />
            </svg>

            <!-- Mode Gaming Icon -->
            <svg
              v-else-if="payload.icon === 'gaming'"
              class="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 10V3L4 14h7v7l9-11h-7z"
              />
            </svg>

            <!-- Mode Turbo Icon -->
            <svg
              v-else-if="payload.icon === 'turbo'"
              class="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M17.657 18.657A8 8 0 016.343 7.343S7 9 9 10c0-2 .5-5 2.986-7C14 5 16.09 5.777 17.656 7.343A7.975 7.975 0 0120 13a7.975 7.975 0 01-2.343 5.657z"
              />
            </svg>

            <!-- Fan Icon -->
            <svg
              v-else-if="payload.kind === 'fan'"
              class="w-6 h-6 animate-spin"
              style="animation-duration: 3s;"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1v-2.5M6 18l2-1m-2 1l2 1m-2 1v-2.5M18 18l-2-1m2 1l-2 1m2 1v-2.5"
              />
            </svg>

            <!-- Win Lock Icon -->
            <svg
              v-else-if="payload.kind === 'win_lock'"
              class="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
              />
            </svg>

            <!-- Fallback Icon -->
            <svg
              v-else
              class="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>

          <!-- Text Details -->
          <div class="flex-1 min-w-0 flex flex-col justify-center">
            <div class="flex items-center justify-between gap-2">
              <div class="text-[15px] font-black tracking-wide text-white truncate">
                {{ payload.title }}
              </div>
              <span
                class="px-2 py-0.5 rounded-full text-[10px] font-bold tracking-wider uppercase shrink-0 border"
                :class="themeClass.tagBg"
              >
                {{ payload.status }}
              </span>
            </div>
            <div class="flex items-center gap-1.5 mt-0.5">
              <span class="text-[11px] font-semibold text-gray-400 tracking-wider uppercase truncate">
                {{ payload.subtitle }}
              </span>
              <span class="w-1 h-1 rounded-full bg-gray-500"></span>
              <span class="text-[10px] text-gray-500 font-mono tracking-tighter">
                MACHENIKE
              </span>
            </div>
          </div>
        </div>

        <!-- Animated Bottom Progress Indicator (3 seconds shrink) -->
        <div class="w-full h-[3px] bg-white/5 rounded-full overflow-hidden mt-2">
          <div
            :key="progressKey"
            class="h-full rounded-full origin-left animate-shrink"
            :class="themeClass.bar"
          ></div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* 动画：3秒平滑进度条倒计时 */
@keyframes shrinkProgress {
  from {
    width: 100%;
  }
  to {
    width: 0%;
  }
}

.animate-shrink {
  animation: shrinkProgress 3s linear forwards;
}

/* OSD 卡片弹入淡出动画 */
.osd-enter-active {
  transition: all 0.22s cubic-bezier(0.16, 1, 0.3, 1);
}

.osd-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 1, 1);
}

.osd-enter-from {
  opacity: 0;
  transform: translateY(16px) scale(0.92);
}

.osd-leave-to {
  opacity: 0;
  transform: translateY(8px) scale(0.96);
}
</style>
