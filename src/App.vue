<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import Sidebar from "./components/Sidebar.vue";
import Dashboard from "./components/Dashboard.vue";
import ModeSelector from "./components/ModeSelector.vue";
import LightingStudio from "./components/LightingStudio.vue";
import QuickSwitches from "./components/QuickSwitches.vue";
import { hardwareApi } from "./services/tauri";
import type { TelemetryData } from "./types/hardware";
import type { UnlistenFn } from "@tauri-apps/api/event";

const activeTab = ref("dashboard");

const telemetry = ref<TelemetryData>({
  cpu_temp: 50,
  gpu_temp: 45,
  cpu_fan_rpm: 2200,
  gpu_fan_rpm: 2000,
  power_mode: 2,
  fan_full_speed: false,
  aou_charging: false,
  wireless_charging: false,
  ac_online: true,
});

const isRefreshing = ref(false);
const toastMessage = ref("");
const toastVisible = ref(false);

const showToast = (msg: string) => {
  toastMessage.value = msg;
  toastVisible.value = true;
  setTimeout(() => {
    toastVisible.value = false;
  }, 2500);
};

const REFRESH_INTERVAL_MS = 3000;
let unlistenTelemetry: UnlistenFn | null = null;
let unlistenVisibility: UnlistenFn | null = null;
let pollingTimer: ReturnType<typeof setTimeout> | null = null;
let isFetchingTelemetry = false;
let isPollingActive = true;

const refreshTelemetry = async () => {
  try {
    const data = await hardwareApi.getTelemetry();
    if (data) {
      telemetry.value = data;
    }
  } catch (err) {
    console.warn("Telemetry refresh failed:", err);
  }
};

const clearScheduledPoll = () => {
  if (pollingTimer !== null) {
    clearTimeout(pollingTimer);
    pollingTimer = null;
  }
};

const scheduleNextPoll = (delay = REFRESH_INTERVAL_MS) => {
  clearScheduledPoll();
  if (!isPollingActive) return;
  pollingTimer = setTimeout(async () => {
    await executePollCycle();
  }, delay);
};

const executePollCycle = async () => {
  if (!isPollingActive || isFetchingTelemetry) return;
  isFetchingTelemetry = true;
  try {
    await refreshTelemetry();
  } finally {
    isFetchingTelemetry = false;
    if (isPollingActive) {
      scheduleNextPoll(REFRESH_INTERVAL_MS);
    }
  }
};

const pausePolling = () => {
  if (!isPollingActive) return;
  isPollingActive = false;
  clearScheduledPoll();
};

const resumePolling = async () => {
  if (isPollingActive) return;
  isPollingActive = true;
  clearScheduledPoll();
  // 唤醒时立即执行一次拉取，并在完成后自动排期下一次 3 秒轮询
  await executePollCycle();
};

const handleVisibilityChange = () => {
  if (document.hidden) {
    pausePolling();
  } else {
    resumePolling();
  }
};

const handleManualRefresh = async () => {
  if (isRefreshing.value || isFetchingTelemetry) return;
  clearScheduledPoll();
  isRefreshing.value = true;
  isFetchingTelemetry = true;
  try {
    await refreshTelemetry();
    showToast("硬件遥测数据已刷新");
  } catch (err) {
    showToast(`刷新失败: ${err}`);
  } finally {
    isRefreshing.value = false;
    isFetchingTelemetry = false;
    if (isPollingActive) {
      scheduleNextPoll(REFRESH_INTERVAL_MS);
    }
  }
};

onMounted(async () => {
  // 首次挂载立即读取一次遥测
  isFetchingTelemetry = true;
  try {
    await refreshTelemetry();
  } finally {
    isFetchingTelemetry = false;
    if (isPollingActive) {
      scheduleNextPoll(REFRESH_INTERVAL_MS);
    }
  }

  // 监听后端可能的遥测广播
  try {
    unlistenTelemetry = await hardwareApi.onTelemetryUpdate((data) => {
      telemetry.value = data;
    });
  } catch (err) {
    console.warn("Failed to listen for telemetry:", err);
  }

  // 监听 Tauri 后端广播的窗口显隐事件（托盘隐藏/唤醒）
  try {
    unlistenVisibility = await hardwareApi.onWindowVisibilityChange((visible) => {
      if (visible) {
        resumePolling();
      } else {
        pausePolling();
      }
    });
  } catch (err) {
    console.warn("Failed to listen for window visibility changes:", err);
  }

  // 监听浏览器 DOM 可见性状态变化（最小化/隐藏）
  document.addEventListener("visibilitychange", handleVisibilityChange);
});

onUnmounted(() => {
  pausePolling();
  document.removeEventListener("visibilitychange", handleVisibilityChange);
  if (unlistenTelemetry) {
    unlistenTelemetry();
    unlistenTelemetry = null;
  }
  if (unlistenVisibility) {
    unlistenVisibility();
    unlistenVisibility = null;
  }
});

const handleModeChange = async (mode: number) => {
  const modeNames: Record<number, string> = {
    1: "办公模式",
    2: "游戏模式",
    3: "狂暴模式",
  };
  try {
    await hardwareApi.setPerformanceMode(mode);
    telemetry.value.power_mode = mode;
    showToast(`已成功切换至 ${modeNames[mode] || "新模式"}`);
  } catch (err) {
    console.error("Failed to change mode:", err);
    showToast(`模式切换失败: ${err}`);
  }
};

const handleToggleFullSpeed = async () => {
  const newState = !telemetry.value.fan_full_speed;
  try {
    await hardwareApi.setFanFullSpeed(newState);
    telemetry.value.fan_full_speed = newState;
    showToast(newState ? "风扇全速强冷已开启！" : "已恢复智能温控转速");
  } catch (err) {
    console.error("Failed to toggle fan full speed:", err);
    showToast(`切换风扇全速失败: ${err}`);
  }
};
</script>

<template>
  <div class="flex flex-row h-screen w-screen overflow-hidden bg-[#0a0d14] text-gray-200">
    <!-- Left Sidebar Navigation -->
    <Sidebar
      :activeTab="activeTab"
      @update:activeTab="activeTab = $event"
      :powerMode="telemetry.power_mode"
      :cpuTemp="telemetry.cpu_temp"
      :gpuTemp="telemetry.gpu_temp"
      :isRefreshing="isRefreshing"
      @refresh="handleManualRefresh"
    />

    <!-- Main Content Area -->
    <main class="flex-1 h-screen overflow-hidden relative flex flex-col bg-[#0b0e14]">
      <Transition name="fade" mode="out-in">
        <KeepAlive>
          <Dashboard
            v-if="activeTab === 'dashboard'"
            :telemetry="telemetry"
            @changeMode="handleModeChange"
            @toggleFullSpeed="handleToggleFullSpeed"
          />
          <ModeSelector
            v-else-if="activeTab === 'modes'"
            :currentMode="telemetry.power_mode"
            :fanFullSpeed="telemetry.fan_full_speed"
            @selectMode="handleModeChange"
            @toggleFullSpeed="handleToggleFullSpeed"
          />
          <LightingStudio
            v-else-if="activeTab === 'lighting'"
            @toast="showToast"
          />
          <QuickSwitches
            v-else-if="activeTab === 'settings'"
            :fanFullSpeed="telemetry.fan_full_speed"
            :aouCharging="telemetry.aou_charging"
            :wirelessCharging="telemetry.wireless_charging"
            :acOnline="telemetry.ac_online"
            @toggleFullSpeed="handleToggleFullSpeed"
            @toast="showToast"
          />
        </KeepAlive>
      </Transition>
    </main>

    <!-- Toast Floating Alert -->
    <Transition name="toast">
      <div
        v-if="toastVisible"
        class="fixed bottom-6 right-6 z-50 px-4 py-2.5 rounded-xl bg-orange-500 text-black font-bold text-xs shadow-2xl shadow-orange-500/50 flex items-center gap-2 border border-orange-400"
      >
        <span class="w-2 h-2 rounded-full bg-black animate-ping"></span>
        {{ toastMessage }}
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(12px) scale(0.95);
}
</style>
