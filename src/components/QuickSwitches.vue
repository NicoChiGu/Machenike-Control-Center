<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import {
  Lock,
  Sun,
  BatteryCharging,
  Zap,
  Shield,
  Monitor,
  Cpu,
  Radio,
  Layers,
  RotateCcw,
  X,
} from "lucide-vue-next";
import { hardwareApi } from "../services/tauri";

const props = defineProps<{
  fanFullSpeed: boolean;
  aouCharging: boolean;
  wirelessCharging?: boolean;
  acOnline?: boolean;
}>();

const emit = defineEmits<{
  (e: "toggleFullSpeed"): void;
  (e: "toast", msg: string): void;
}>();

const testOsd = (kind: string) => {
  hardwareApi.triggerOsdTest(kind).catch(console.error);
};

// Local switch states
const winLocked = ref(false);
const kbAlwaysOn = ref(true);
const aouLocal = ref(props.aouCharging);
const wirelessLocal = ref(props.wirelessCharging ?? false);

// BIOS NVRAM states
const gpuDirect = ref(false);
const memoryGear = ref(false);
const isAdmin = ref(true);
const isLoadingBios = ref(true);

// Reboot confirmation modal
const showRebootModal = ref(false);
const rebootTitle = ref("");
const rebootDescription = ref("");

watch(
  () => props.aouCharging,
  (val) => {
    aouLocal.value = val;
  }
);

watch(
  () => props.wirelessCharging,
  (val) => {
    if (val !== undefined) {
      wirelessLocal.value = val;
    }
  }
);

const loadBiosSettings = async () => {
  isLoadingBios.value = true;
  try {
    const bios = await hardwareApi.getBiosSettings();
    if (bios) {
      gpuDirect.value = bios.gpu_direct;
      memoryGear.value = bios.memory_gear;
      isAdmin.value = bios.is_admin;
    }
  } catch (err) {
    console.warn("Failed to load BIOS settings:", err);
  } finally {
    isLoadingBios.value = false;
  }
};

onMounted(() => {
  loadBiosSettings();
});

const toggleGpuDirect = async () => {
  const target = !gpuDirect.value;
  try {
    await hardwareApi.setGpuDirectMode(target);
    gpuDirect.value = target;
    rebootTitle.value = target ? "已开启「独显直连」" : "已切换为「混合输出」";
    rebootDescription.value =
      "设置已成功写入 UEFI BIOS NVRAM (SaSetup)。硬件 MUX 切线走线将在下次计算机启动时生效，请选择是否立即重启。";
    showRebootModal.value = true;
  } catch (err) {
    console.error("Failed to set GPU Direct:", err);
    emit("toast", `设置独显直连失败 (请确认以管理员权限运行): ${err}`);
  }
};

const toggleMemoryGear = async () => {
  const target = !memoryGear.value;
  try {
    await hardwareApi.setMemoryGearMode(target);
    memoryGear.value = target;
    rebootTitle.value = target
      ? "已开启「内存分频模式 (Gear 2)」"
      : "已关闭「内存分频 (Gear 1 直连)」";
    rebootDescription.value =
      "设置已成功写入 UEFI BIOS NVRAM。内存控制器时钟模式将在下次系统冷启动时由 BIOS 重新训练加载。";
    showRebootModal.value = true;
  } catch (err) {
    console.error("Failed to set Memory Gear:", err);
    emit("toast", `设置内存分频失败 (请确认以管理员权限运行): ${err}`);
  }
};

const toggleWireless = async () => {
  if (props.acOnline === false) {
    emit("toast", "电脑当前由电池供电，无线充电已自动保护锁定（需插入电源适配器）");
    return;
  }
  const target = !wirelessLocal.value;
  wirelessLocal.value = target;
  try {
    await hardwareApi.setWirelessCharge(target);
    emit("toast", target ? "机身无线充电已开启" : "机身无线充电已关闭");
  } catch (err) {
    console.error("Failed to set wireless charge:", err);
    wirelessLocal.value = !target;
    emit("toast", `无线充电切换失败: ${err}`);
  }
};

const toggleWinLock = async () => {
  winLocked.value = !winLocked.value;
  try {
    await hardwareApi.setWinLock(winLocked.value);
  } catch (err) {
    console.error("Failed to toggle WinLock:", err);
  }
};

const toggleKbAlwaysOn = async () => {
  kbAlwaysOn.value = !kbAlwaysOn.value;
  try {
    await hardwareApi.setKeyboardTimeout(kbAlwaysOn.value);
  } catch (err) {
    console.error("Failed to toggle Keyboard Timeout:", err);
  }
};

const toggleAou = async () => {
  aouLocal.value = !aouLocal.value;
  try {
    await hardwareApi.setAouCharge(aouLocal.value);
  } catch (err) {
    console.error("Failed to toggle AOU:", err);
  }
};

const handleConfirmReboot = async () => {
  try {
    await hardwareApi.requestReboot();
  } catch (err) {
    console.error("Reboot request failed:", err);
    emit("toast", `重启触发失败: ${err}`);
  }
};
</script>

<template>
  <div class="h-full flex flex-col gap-6 p-6 overflow-y-auto relative">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-extrabold text-white tracking-wide flex items-center gap-2.5">
          系统与硬件快捷功能
          <span
            v-if="!isAdmin"
            class="text-[10px] px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/20 font-normal"
          >
            受限模式 (需管理员提权读写 BIOS)
          </span>
        </h2>
        <p class="text-xs text-gray-400 mt-1">
          直驱 UEFI BIOS 固件与主板 EC 硬件端口，深度控制独显直连、内存分频、温控与外设供电
        </p>
      </div>
      <button
        @click="loadBiosSettings"
        class="px-3 py-1 rounded-xl bg-white/5 hover:bg-white/10 text-xs text-gray-400 hover:text-white transition-all flex items-center gap-1.5 border border-white/5"
        title="刷新固件状态"
      >
        <RotateCcw class="w-3.5 h-3.5" :class="{ 'animate-spin': isLoadingBios }" />
        <span>刷新</span>
      </button>
    </div>

    <!-- Switches Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- 1. 独显直连 (Discrete GPU Direct) -->
      <div class="glass-panel rounded-2xl p-5 border border-white/5 flex items-center justify-between group hover:border-orange-500/20 transition-all duration-300">
        <div class="flex items-center gap-3.5">
          <div
            class="w-11 h-11 rounded-xl flex items-center justify-center transition-colors duration-200"
            :class="[gpuDirect ? 'bg-orange-500/15 text-orange-400' : 'bg-white/5 text-gray-400']"
          >
            <Cpu class="w-5 h-5" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <span class="text-sm font-bold text-white">独显直连 (MUX Switch)</span>
              <span
                class="text-[10px] px-1.5 py-0.2 rounded font-medium"
                :class="[gpuDirect ? 'bg-orange-500/20 text-orange-300' : 'bg-white/10 text-gray-400']"
              >
                {{ gpuDirect ? '独显直连' : '混合输出' }}
              </span>
            </div>
            <div class="text-xs text-gray-400 mt-0.5">
              绕过核显获得极致游戏帧率与超低延迟；需重启生效
            </div>
          </div>
        </div>
        <button
          @click="toggleGpuDirect"
          class="w-12 h-6 rounded-full transition-colors duration-200 relative p-0.5 border shrink-0"
          :class="[gpuDirect ? 'bg-orange-500 border-orange-400' : 'bg-white/10 border-white/10']"
        >
          <span
            class="w-5 h-5 rounded-full bg-white block shadow transition-transform duration-200"
            :class="{ 'translate-x-6': gpuDirect }"
          ></span>
        </button>
      </div>

      <!-- 2. 内存分频模式 (SAGV Memory Gear) -->
      <div class="glass-panel rounded-2xl p-5 border border-white/5 flex items-center justify-between group hover:border-cyan-500/20 transition-all duration-300">
        <div class="flex items-center gap-3.5">
          <div
            class="w-11 h-11 rounded-xl flex items-center justify-center transition-colors duration-200"
            :class="[memoryGear ? 'bg-cyan-500/15 text-cyan-400' : 'bg-white/5 text-gray-400']"
          >
            <Layers class="w-5 h-5" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <span class="text-sm font-bold text-white">内存分频模式 (SAGV)</span>
              <span
                class="text-[10px] px-1.5 py-0.2 rounded font-medium"
                :class="[memoryGear ? 'bg-cyan-500/20 text-cyan-300' : 'bg-emerald-500/20 text-emerald-300']"
              >
                {{ memoryGear ? 'Gear 2 (节能模式)' : 'Gear 1 (高性能直连)' }}
              </span>
            </div>
            <div class="text-xs text-gray-400 mt-0.5">
              关闭分频获得更强内存读写带宽与极速延迟；需重启生效
            </div>
          </div>
        </div>
        <button
          @click="toggleMemoryGear"
          class="w-12 h-6 rounded-full transition-colors duration-200 relative p-0.5 border shrink-0"
          :class="[memoryGear ? 'bg-cyan-500 border-cyan-400' : 'bg-white/10 border-white/10']"
        >
          <span
            class="w-5 h-5 rounded-full bg-white block shadow transition-transform duration-200"
            :class="{ 'translate-x-6': memoryGear }"
          ></span>
        </button>
      </div>

      <!-- 3. 无线充电 (Wireless Charging) -->
      <div
        class="glass-panel rounded-2xl p-5 border border-white/5 flex items-center justify-between group transition-all duration-300"
        :class="{ 'opacity-60': props.acOnline === false }"
      >
        <div class="flex items-center gap-3.5">
          <div
            class="w-11 h-11 rounded-xl flex items-center justify-center transition-colors duration-200"
            :class="[wirelessLocal ? 'bg-emerald-500/15 text-emerald-400' : 'bg-white/5 text-gray-400']"
          >
            <Radio class="w-5 h-5" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <span class="text-sm font-bold text-white">机身无线充电</span>
              <span
                v-if="props.acOnline === false"
                class="text-[10px] px-1.5 py-0.2 rounded font-medium bg-amber-500/20 text-amber-300"
              >
                仅插电可用
              </span>
              <span
                v-else
                class="text-[10px] px-1.5 py-0.2 rounded font-medium"
                :class="[wirelessLocal ? 'bg-emerald-500/20 text-emerald-300' : 'bg-white/10 text-gray-400']"
              >
                {{ wirelessLocal ? '已开启' : '已关闭' }}
              </span>
            </div>
            <div class="text-xs text-gray-400 mt-0.5">
              开启掌托/特定区域无线快充，仅在连接电源适配器时支持
            </div>
          </div>
        </div>
        <button
          @click="toggleWireless"
          :disabled="props.acOnline === false"
          class="w-12 h-6 rounded-full transition-colors duration-200 relative p-0.5 border shrink-0"
          :class="[
            props.acOnline === false
              ? 'bg-white/5 border-white/5 cursor-not-allowed'
              : wirelessLocal
              ? 'bg-emerald-500 border-emerald-400'
              : 'bg-white/10 border-white/10'
          ]"
        >
          <span
            class="w-5 h-5 rounded-full bg-white block shadow transition-transform duration-200"
            :class="{ 'translate-x-6': wirelessLocal && props.acOnline !== false }"
          ></span>
        </button>
      </div>

      <!-- 4. Win 键锁定 (Win Lock) -->
      <div class="glass-panel rounded-2xl p-5 border border-white/5 flex items-center justify-between">
        <div class="flex items-center gap-3.5">
          <div class="w-11 h-11 rounded-xl flex items-center justify-center bg-white/5 text-gray-300">
            <Lock class="w-5 h-5 text-orange-400" />
          </div>
          <div>
            <div class="text-sm font-bold text-white">Win 键锁定</div>
            <div class="text-xs text-gray-400 mt-0.5">游戏中禁用 Windows 徽标键，避免误触切出桌面</div>
          </div>
        </div>
        <button
          @click="toggleWinLock"
          class="w-12 h-6 rounded-full transition-colors duration-200 relative p-0.5 border shrink-0"
          :class="[winLocked ? 'bg-orange-500 border-orange-400' : 'bg-white/10 border-white/10']"
        >
          <span
            class="w-5 h-5 rounded-full bg-white block shadow transition-transform duration-200"
            :class="{ 'translate-x-6': winLocked }"
          ></span>
        </button>
      </div>

      <!-- 5. 键盘灯常亮 (Keyboard LED Always On) -->
      <div class="glass-panel rounded-2xl p-5 border border-white/5 flex items-center justify-between">
        <div class="flex items-center gap-3.5">
          <div class="w-11 h-11 rounded-xl flex items-center justify-center bg-white/5 text-gray-300">
            <Sun class="w-5 h-5 text-amber-400" />
          </div>
          <div>
            <div class="text-sm font-bold text-white">键盘灯常亮</div>
            <div class="text-xs text-gray-400 mt-0.5">保持键盘背光常亮，不因 30 秒无操作熄灭</div>
          </div>
        </div>
        <button
          @click="toggleKbAlwaysOn"
          class="w-12 h-6 rounded-full transition-colors duration-200 relative p-0.5 border shrink-0"
          :class="[kbAlwaysOn ? 'bg-orange-500 border-orange-400' : 'bg-white/10 border-white/10']"
        >
          <span
            class="w-5 h-5 rounded-full bg-white block shadow transition-transform duration-200"
            :class="{ 'translate-x-6': kbAlwaysOn }"
          ></span>
        </button>
      </div>

      <!-- 6. 风扇全速 (Fan Full Speed) -->
      <div class="glass-panel rounded-2xl p-5 border border-white/5 flex items-center justify-between">
        <div class="flex items-center gap-3.5">
          <div class="w-11 h-11 rounded-xl flex items-center justify-center bg-white/5 text-gray-300">
            <Zap class="w-5 h-5 text-cyan-400" />
          </div>
          <div>
            <div class="text-sm font-bold text-white">风扇全速 (强冷模式)</div>
            <div class="text-xs text-gray-400 mt-0.5">强制 CPU 与 GPU 双风扇以最高 5000+ RPM 运转</div>
          </div>
        </div>
        <button
          @click="emit('toggleFullSpeed')"
          class="w-12 h-6 rounded-full transition-colors duration-200 relative p-0.5 border shrink-0"
          :class="[fanFullSpeed ? 'bg-orange-500 border-orange-400' : 'bg-white/10 border-white/10']"
        >
          <span
            class="w-5 h-5 rounded-full bg-white block shadow transition-transform duration-200"
            :class="{ 'translate-x-6': fanFullSpeed }"
          ></span>
        </button>
      </div>

      <!-- 7. 关机对外充电 (AOU) -->
      <div class="glass-panel rounded-2xl p-5 border border-white/5 flex items-center justify-between md:col-span-2">
        <div class="flex items-center gap-3.5">
          <div class="w-11 h-11 rounded-xl flex items-center justify-center bg-white/5 text-gray-300">
            <BatteryCharging class="w-5 h-5 text-emerald-400" />
          </div>
          <div>
            <div class="text-sm font-bold text-white">关机对外充电 (Always-On USB)</div>
            <div class="text-xs text-gray-400 mt-0.5">休眠或关机状态下，机身右侧特定 USB 接口仍可持续对外供电充电</div>
          </div>
        </div>
        <button
          @click="toggleAou"
          class="w-12 h-6 rounded-full transition-colors duration-200 relative p-0.5 border shrink-0"
          :class="[aouLocal ? 'bg-orange-500 border-orange-400' : 'bg-white/10 border-white/10']"
        >
          <span
            class="w-5 h-5 rounded-full bg-white block shadow transition-transform duration-200"
            :class="{ 'translate-x-6': aouLocal }"
          ></span>
        </button>
      </div>
    </div>

    <!-- OSD Preview & Test Panel -->
    <div class="glass-panel rounded-2xl p-5 border border-white/5 flex flex-col gap-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Monitor class="w-4 h-4 text-cyan-400" />
          <span class="text-sm font-bold text-white">现代化 OSD 悬浮窗联动</span>
        </div>
        <span class="text-xs text-gray-500">屏幕居中靠下 · 3秒自动淡出 · 物理热键全域联动</span>
      </div>
      <div class="flex flex-wrap gap-2 pt-1">
        <button
          @click="testOsd('caps')"
          class="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-cyan-500/20 text-xs text-gray-300 hover:text-cyan-300 border border-white/10 hover:border-cyan-500/30 transition-all duration-150 flex items-center gap-1.5"
        >
          <span>🔒</span> 测试 CapsLock
        </button>
        <button
          @click="testOsd('num')"
          class="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-emerald-500/20 text-xs text-gray-300 hover:text-emerald-300 border border-white/10 hover:border-emerald-500/30 transition-all duration-150 flex items-center gap-1.5"
        >
          <span>🔢</span> 测试 NumLock
        </button>
        <button
          @click="testOsd('gaming')"
          class="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-indigo-500/20 text-xs text-gray-300 hover:text-indigo-300 border border-white/10 hover:border-indigo-500/30 transition-all duration-150 flex items-center gap-1.5"
        >
          <span>⚡</span> 测试游戏模式
        </button>
        <button
          @click="testOsd('turbo')"
          class="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-orange-500/20 text-xs text-gray-300 hover:text-orange-300 border border-white/10 hover:border-orange-500/30 transition-all duration-150 flex items-center gap-1.5"
        >
          <span>🔥</span> 测试狂暴模式
        </button>
      </div>
    </div>

    <!-- System Info & Drivers Footer Card -->
    <div class="mt-auto glass-panel rounded-2xl p-5 border border-white/5 flex items-center justify-between text-xs text-gray-400">
      <div class="flex items-center gap-2">
        <Shield class="w-4 h-4 text-emerald-400" />
        <span>底层驱动状态：<strong class="text-white">WTIOportDrv.sys (ioportdrv 运行中)</strong></span>
      </div>
      <div class="font-mono text-gray-500">
        NVRAM: SaSetup | Port: 0x6C/0x68 | HID: 1A2C:1512
      </div>
    </div>

    <!-- Reboot Confirm Modal Dialog -->
    <Transition name="modal">
      <div
        v-if="showRebootModal"
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-md"
      >
        <div class="glass-panel border border-orange-500/30 rounded-3xl max-w-md w-full p-6 shadow-2xl bg-[#11141c]/95 relative flex flex-col gap-4 animate-in fade-in zoom-in-95 duration-200">
          <button
            @click="showRebootModal = false"
            class="absolute top-5 right-5 text-gray-400 hover:text-white p-1 rounded-lg hover:bg-white/5 transition-colors"
          >
            <X class="w-4 h-4" />
          </button>

          <div class="flex items-center gap-3">
            <div class="w-12 h-12 rounded-2xl bg-orange-500/20 border border-orange-500/30 flex items-center justify-center text-orange-400 shrink-0">
              <RotateCcw class="w-6 h-6" />
            </div>
            <div>
              <h3 class="text-base font-bold text-white">{{ rebootTitle }}</h3>
              <p class="text-xs text-orange-400/90 font-medium mt-0.5">需要重新启动计算机以生效</p>
            </div>
          </div>

          <p class="text-xs text-gray-300 leading-relaxed bg-white/5 p-3.5 rounded-2xl border border-white/5">
            {{ rebootDescription }}
          </p>

          <div class="flex items-center justify-end gap-3 pt-2">
            <button
              @click="showRebootModal = false"
              class="px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 text-xs font-semibold text-gray-300 hover:text-white border border-white/10 transition-all duration-150"
            >
              稍后自行重启
            </button>
            <button
              @click="handleConfirmReboot"
              class="px-5 py-2 rounded-xl bg-gradient-to-r from-orange-500 to-amber-500 hover:from-orange-400 hover:to-amber-400 text-xs font-bold text-black shadow-lg shadow-orange-500/20 transition-all duration-150 flex items-center gap-1.5"
            >
              <RotateCcw class="w-3.5 h-3.5" />
              立即重启
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
