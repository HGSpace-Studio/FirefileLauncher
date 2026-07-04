<script setup lang="ts">
import { ref, computed, onUnmounted } from "vue";
import { ArrowLeft, Gamepad2, Play, Square, LoaderCircle } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const props = defineProps<{
  instance: InstanceData;
}>();

const emit = defineEmits<{
  (e: "back"): void;
}>();

export interface InstanceData {
  name: string;
  version: string;
  versionType: string;
  loader?: {
    type: "fabric" | "forge" | "neoforge" | "quilt";
    version: string;
  };
  icon?: string;
}

const loaderLabel = computed(() => {
  if (!props.instance.loader) return "原版";
  const map: Record<string, string> = {
    fabric: "Fabric",
    forge: "Forge",
    neoforge: "NeoForge",
    quilt: "Quilt",
  };
  return `${map[props.instance.loader.type] || props.instance.loader.type} ${props.instance.loader.version}`;
});

type LaunchState = "idle" | "launching" | "running" | "exited";
const state = ref<LaunchState>("idle");
const launchLabel = ref("启动游戏");
const launchProgress = ref(0);
let unlistenProgress: (() => void) | null = null;
let unlistenOutput: (() => void) | null = null;
let unlistenExit: (() => void) | null = null;
let unlistenError: (() => void) | null = null;
let unlistenReady: (() => void) | null = null;

async function launchGame() {
  if (state.value !== "idle") return;
  state.value = "launching";
  launchLabel.value = "准备启动...";
  launchProgress.value = 0;

  unlistenProgress = await listen<{ kind: string; downloaded: number; total: number }>("minecraft-progress", (event) => {
    const { kind, downloaded, total } = event.payload;
    launchLabel.value = kind ? `正在下载 ${kind}...` : "启动中...";
    if (total > 0) {
      launchProgress.value = downloaded / total;
    }
  });

  unlistenOutput = await listen<{ line: string }>("minecraft-output", () => {
    // game output, can be logged
  });

  unlistenReady = await listen("minecraft-ready", () => {
    state.value = "running";
    launchLabel.value = "停止";
  });

  unlistenError = await listen<{ message: string }>("minecraft-error", (event) => {
    launchLabel.value = "启动失败: " + event.payload.message;
    state.value = "idle";
    cleanupListeners();
  });

  unlistenExit = await listen<{ code: number }>("minecraft-exit", () => {
    state.value = "exited";
    launchLabel.value = "游戏已退出";
    launchProgress.value = 1;
    cleanupListeners();
    setTimeout(() => {
      if (state.value === "exited") {
        state.value = "idle";
        launchLabel.value = "启动游戏";
        launchProgress.value = 0;
      }
    }, 3000);
  });

  try {
    const acc = await invoke<{ name: string; account_type: string; uuid: string }>("get_current_account");
    const settings = await invoke<{ account_name: string; java_path: string }>("get_oobe_settings");
    const mcDir = await invoke<string>("get_minecraft_dir_string");

    await invoke("launch_minecraft", {
      args: {
        version: props.instance.version,
        username: acc.name || settings.account_name || "Player",
        game_dir: mcDir,
        min_mem: "1024M",
        max_mem: "2048M",
        loader_type: props.instance.loader?.type || null,
        loader_build: props.instance.loader?.version || null,
        instance: props.instance.name,
        download_only: false,
      },
    });
  } catch (e: any) {
    launchLabel.value = "启动失败: " + (e?.toString() || "未知错误");
    state.value = "idle";
    cleanupListeners();
  }
}

async function stopGame() {
  try {
    await invoke("stop_game");
  } catch (e: any) {
    // ignore
  }
}

function cleanupListeners() {
  if (unlistenProgress) { unlistenProgress(); unlistenProgress = null; }
  if (unlistenOutput) { unlistenOutput(); unlistenOutput = null; }
  if (unlistenError) { unlistenError(); unlistenError = null; }
  if (unlistenExit) { unlistenExit(); unlistenExit = null; }
  if (unlistenReady) { unlistenReady(); unlistenReady = null; }
}

onUnmounted(cleanupListeners);
</script>

<template>
  <div class="detail-page">
    <div class="detail-header">
      <button class="detail-back" @click="emit('back')">
        <ArrowLeft :size="20" />
      </button>
      <div class="detail-icon-area">
        <img v-if="instance.icon" :src="instance.icon" class="detail-icon" />
        <Gamepad2 v-else :size="40" class="detail-icon-placeholder" />
      </div>
      <div class="detail-meta">
        <span class="detail-loader">{{ loaderLabel }}</span>
        <span class="detail-version">Minecraft {{ instance.version }}</span>
      </div>
    </div>
    <div class="detail-content">
    </div>
    <div class="detail-footer">
      <button
        v-if="state === 'launching'"
        class="launch-btn launching"
      >
        <LoaderCircle :size="18" class="spin" />
        <span>{{ launchLabel }}</span>
        <div class="launch-bar">
          <div class="launch-bar-fill" :style="{ width: (launchProgress * 100) + '%' }"></div>
        </div>
      </button>
      <button
        v-else-if="state === 'running'"
        class="launch-btn running"
        @click="stopGame"
      >
        <Square :size="18" />
        <span>{{ launchLabel }}</span>
      </button>
      <button
        v-else-if="state === 'exited'"
        class="launch-btn exited"
      >
        <span>{{ launchLabel }}</span>
      </button>
      <button
        v-else
        class="launch-btn ready"
        @click="launchGame"
      >
        <Play :size="18" />
        <span>{{ launchLabel }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.detail-page {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px 28px;
  overflow-y: auto;
}

.detail-header {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-shrink: 0;
}

.detail-back {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--title-color);
  cursor: pointer;
  transition: background 0.15s;
  flex-shrink: 0;
}

.detail-back:hover {
  background: rgba(128, 128, 128, 0.15);
}

.detail-icon-area {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  background: var(--panel-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
}

.detail-icon {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.detail-icon-placeholder {
  opacity: 0.45;
  color: var(--title-color);
}

.detail-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.detail-loader {
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.45;
  line-height: 1;
}

.detail-version {
  font-size: 18px;
  font-weight: 600;
  color: var(--title-color);
  line-height: 1.2;
}

.detail-content {
  flex: 1;
  margin-top: 20px;
  min-height: 0;
}

.detail-footer {
  display: flex;
  justify-content: flex-end;
  padding-top: 16px;
  flex-shrink: 0;
}

.launch-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 24px;
  border: none;
  border-radius: 100px;
  background: rgba(0, 120, 212, 0.15);
  color: #0078d4;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.3s ease, color 0.3s ease, opacity 0.3s ease;
  font-family: inherit;
  position: relative;
  overflow: hidden;
}

.launch-btn:hover {
  background: rgba(0, 120, 212, 0.25);
}

.launch-btn.ready {
  background: #0078d4;
  color: #fff;
}

.launch-btn.ready:hover {
  background: #1a8ae8;
}

.launch-btn.running {
  background: #d43a3a;
  color: #fff;
}

.launch-btn.running:hover {
  background: #e05555;
}

.launch-btn.exited {
  opacity: 0.6;
  cursor: default;
}

.launch-btn.launching {
  background: rgba(0, 120, 212, 0.1);
  color: #0078d4;
  cursor: default;
  flex-direction: column;
  gap: 6px;
  padding: 8px 24px 10px;
  min-width: 200px;
}

.launch-btn.launching .spin {
  animation: spin 1s linear infinite;
}

.launch-bar {
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: rgba(0, 120, 212, 0.15);
  overflow: hidden;
}

.launch-bar-fill {
  height: 100%;
  border-radius: 2px;
  background: #0078d4;
  transition: width 0.4s ease;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
