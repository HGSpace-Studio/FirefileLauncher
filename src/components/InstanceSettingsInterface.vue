<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { Icon as VIcon } from "@vicons/utils";
import { Dismiss24Regular, Settings24Regular, Flash24Regular, PuzzlePiece24Regular, WindowConsole20Regular, Options24Regular, Save24Regular, Folder24Regular } from "@vicons/fluent";
import { invoke } from "@tauri-apps/api/core";
import type { InstanceData } from "./view/rootpages/instance_detail.vue";

const props = defineProps<{
  instance: InstanceData;
}>();
const emit = defineEmits<{
  (e: "close"): void;
}>();

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

const settingsTab = ref<"general" | "quicklaunch" | "extensions" | "java" | "other">("general")
const saveMsg = ref("")
let saveTimer: number | null = null

interface JavaInstall {
  path: string
  version: string
}

interface SystemMem {
  totalMb: number
  usedMb: number
}

interface InstanceSettings {
  icon: string | null
  skipLauncher: boolean
  fullscreen: boolean
  autoConnectAddress: string | null
  javaVersion: string | null
  autoMemory: boolean
  minMemory: string
  maxMemory: string
  jvmArgs: string
  gameArgs: string
  downloadConcurrency: number
  verifyConcurrency: number
}

const instSettings = ref<InstanceSettings>({
  icon: null,
  skipLauncher: false,
  fullscreen: false,
  autoConnectAddress: null,
  javaVersion: null,
  autoMemory: true,
  minMemory: "1024M",
  maxMemory: "2048M",
  jvmArgs: "",
  gameArgs: "",
  downloadConcurrency: 10,
  verifyConcurrency: 4,
})

const javaVersions = ref<JavaInstall[]>([])
const systemMem = ref<SystemMem | null>(null)

function shortJavaLabel(version: string): string {
  const m = version.match(/"(\d+)\.(\d+)/)
  if (m) {
    if (m[1] === "1") return `Java ${m[2]}`
    return `Java ${m[1]}`
  }
  const m2 = version.match(/(\d+)/)
  return m2 ? `Java ${m2[1]}` : version
}

const totalMemGb = computed(() => {
  if (!systemMem.value) return 0
  return Math.round(systemMem.value.totalMb / 1024)
})

const usedMemGb = computed(() => {
  if (!systemMem.value) return 0
  return Math.round(systemMem.value.usedMb / 1024)
})

const memBarPercent = computed(() => {
  if (!systemMem.value || systemMem.value.totalMb === 0) return 0
  return (systemMem.value.usedMb / systemMem.value.totalMb) * 100
})

onMounted(async () => {
  try {
    const data = await invoke<InstanceSettings>("get_instance_settings", { instanceName: props.instance.name })
    if (data) {
      instSettings.value = { ...instSettings.value, ...data }
    }
  } catch {
    // use defaults
  }
  try {
    javaVersions.value = await invoke<JavaInstall[]>("get_java_versions")
  } catch {
    // ignore
  }
  try {
    systemMem.value = await invoke<SystemMem>("get_system_memory")
  } catch {
    // ignore
  }
})

function onMinMemSlide(e: Event) {
  const v = (e.target as HTMLInputElement).value
  instSettings.value.minMemory = v + "M"
  const max = parseInt(instSettings.value.maxMemory) || 2048
  if (parseInt(v) > max - 256) {
    instSettings.value.maxMemory = (parseInt(v) + 256) + "M"
  }
}

function onMaxMemSlide(e: Event) {
  const v = (e.target as HTMLInputElement).value
  instSettings.value.maxMemory = v + "M"
}

const platformLabel = computed(() => {
  if (navigator.userAgent.includes("Windows")) return "在文件资源管理器中显示"
  if (navigator.userAgent.includes("Mac")) return "在 Finder 中显示"
  return "打开目录"
})

async function openGameFolder() {
  try {
    await invoke("open_instance_game_folder", { instanceName: props.instance.name })
  } catch (e) {
    // ignore
  }
}

async function saveSettings() {
  saveMsg.value = "保存中..."
  try {
    await invoke("save_instance_settings", { instanceName: props.instance.name, settings: instSettings.value })
    saveMsg.value = "已保存"
  } catch {
    saveMsg.value = "保存失败"
  }
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = window.setTimeout(() => { saveMsg.value = "" }, 3000)
}
</script>

<template>
  <div class="isettings-overlay" @click.self="emit('close')">
    <div class="isettings-window">
      <div class="isettings-header">
        <div class="isettings-header-left">
          <span class="isettings-icon-wrap">
            <VIcon :size="16" class="isettings-icon"><Settings24Regular /></VIcon>
          </span>
          <span class="isettings-title">{{ instance.name }} - 设置</span>
        </div>
        <button class="isettings-close" @click="emit('close')">
          <VIcon :size="18"><Dismiss24Regular /></VIcon>
        </button>
      </div>
      <div class="isettings-divider"></div>
      <div class="isettings-body">
        <div class="iset-layout">
          <div class="iset-sidebar">
            <button class="iset-nav-item" :class="{ active: settingsTab === 'general' }" @click="settingsTab = 'general'">
              <VIcon :size="16"><Settings24Regular /></VIcon>
              <span>常规</span>
            </button>
            <button class="iset-nav-item" :class="{ active: settingsTab === 'quicklaunch' }" @click="settingsTab = 'quicklaunch'">
              <VIcon :size="16"><Flash24Regular /></VIcon>
              <span>快速启动</span>
            </button>
            <button class="iset-nav-item" :class="{ active: settingsTab === 'extensions' }" @click="settingsTab = 'extensions'">
              <VIcon :size="16"><PuzzlePiece24Regular /></VIcon>
              <span>可选扩展</span>
            </button>
            <button class="iset-nav-item" :class="{ active: settingsTab === 'java' }" @click="settingsTab = 'java'">
              <VIcon :size="16"><WindowConsole20Regular /></VIcon>
              <span>Java 与运行</span>
            </button>
            <button class="iset-nav-item" :class="{ active: settingsTab === 'other' }" @click="settingsTab = 'other'">
              <VIcon :size="16"><Options24Regular /></VIcon>
              <span>其他</span>
            </button>
            <div class="iset-sidebar-spacer"></div>
            <button class="iset-nav-item" @click="openGameFolder">
              <VIcon :size="16"><Folder24Regular /></VIcon>
              <span>{{ platformLabel }}中显示</span>
            </button>
          </div>
          <div class="iset-content">
            <div class="iset-scroll">
              <div v-if="settingsTab === 'general'" class="iset-section">
                <h3 class="iset-heading">常规</h3>
                <div class="iset-row">
                  <label class="iset-label">实例名称</label>
                  <input v-model="props.instance.name" disabled class="iset-input disabled" />
                </div>
                <div class="iset-row">
                  <label class="iset-label">游戏版本</label>
                  <input :value="props.instance.version" disabled class="iset-input disabled" />
                </div>
                <div class="iset-row">
                  <label class="iset-label">版本类型</label>
                  <input :value="props.instance.versionType" disabled class="iset-input disabled" />
                </div>
              </div>
              <div v-if="settingsTab === 'quicklaunch'" class="iset-section">
                <h3 class="iset-heading">快速启动</h3>
                <div class="iset-row">
                  <label class="iset-label">跳过启动动画</label>
                  <label class="iset-toggle">
                    <input v-model="instSettings.skipLauncher" type="checkbox" />
                    <span class="iset-toggle-slider"></span>
                  </label>
                </div>
                <div class="iset-row">
                  <label class="iset-label">自动连接服务器</label>
                  <input v-model="instSettings.autoConnectAddress" placeholder="例如: example.com:25565" class="iset-input" />
                </div>
                <div class="iset-row">
                  <label class="iset-label">以全屏进入游戏</label>
                  <label class="iset-toggle">
                    <input v-model="instSettings.fullscreen" type="checkbox" />
                    <span class="iset-toggle-slider"></span>
                  </label>
                </div>
              </div>
              <div v-if="settingsTab === 'extensions'" class="iset-section">
                <h3 class="iset-heading">可选扩展</h3>
                <div class="iset-row">
                  <label class="iset-label">加载器</label>
                  <input :value="loaderLabel" disabled class="iset-input disabled" />
                </div>
              </div>
              <div v-if="settingsTab === 'java'" class="iset-section">
                <div class="iset-card">
                  <div class="iset-card-head">
                    <span class="iset-card-title">游戏启动时的 Java 版本</span>
                    <span class="iset-card-desc">选择启动这个实例时使用的 Java 版本</span>
                  </div>
                  <div class="iset-combo-wrap">
                    <select v-model="instSettings.javaVersion" class="iset-combobox">
                      <option :value="null">自动选择</option>
                      <option v-for="jv in javaVersions" :key="jv.path" :value="jv.path">
                        {{ shortJavaLabel(jv.version) }}
                      </option>
                    </select>
                  </div>
                </div>
                <div class="iset-card">
                  <div class="iset-card-head">
                    <span class="iset-card-title">游戏运行内存分配</span>
                    <span class="iset-card-desc">自动设置游戏运行的内存区间</span>
                  </div>
                  <div class="iset-card-controls">
                    <label class="iset-toggle">
                      <input v-model="instSettings.autoMemory" type="checkbox" />
                      <span class="iset-toggle-slider"></span>
                    </label>
                    <span class="iset-toggle-label">{{ instSettings.autoMemory ? '自动分配' : '手动调整' }}</span>
                  </div>
                  <div v-if="instSettings.autoMemory && systemMem" class="mem-section">
                    <div class="mem-bar">
                      <div class="mem-bar-used" :style="{ width: memBarPercent + '%' }"></div>
                    </div>
                    <div class="mem-info">
                      您的电脑一共有 <strong>{{ totalMemGb }} GB</strong> 运行内存，已用了 <strong>{{ usedMemGb }} GB</strong>
                    </div>
                  </div>
                  <div v-else-if="!instSettings.autoMemory && systemMem" class="mem-section">
                    <div class="mem-slider-group">
                      <div class="mem-slider-row">
                        <span class="mem-slider-label">最小内存</span>
                        <div class="mem-slider-track-wrap">
                          <input
                            type="range"
                            class="mem-slider"
                            :min="256"
                            :max="Math.max(256, systemMem.totalMb - 512)"
                            :value="parseInt(instSettings.minMemory) || 1024"
                            @input="onMinMemSlide"
                            step="128"
                          />
                          <span class="mem-slider-val">{{ parseInt(instSettings.minMemory) || 1024 }} MB</span>
                        </div>
                      </div>
                      <div class="mem-slider-row">
                        <span class="mem-slider-label">最大内存</span>
                        <div class="mem-slider-track-wrap">
                          <input
                            type="range"
                            class="mem-slider"
                            :min="Math.max(256, (parseInt(instSettings.minMemory) || 1024) + 256)"
                            :max="Math.max(512, systemMem.totalMb - 256)"
                            :value="parseInt(instSettings.maxMemory) || 2048"
                            @input="onMaxMemSlide"
                            step="128"
                          />
                          <span class="mem-slider-val">{{ parseInt(instSettings.maxMemory) || 2048 }} MB</span>
                        </div>
                      </div>
                    </div>
                    <div class="mem-info">
                      您的电脑一共有 <strong>{{ totalMemGb }} GB</strong> 运行内存
                    </div>
                  </div>
                  <div v-else class="mem-section">
                    <div class="iset-row">
                      <label class="iset-label">最小内存</label>
                      <input v-model="instSettings.minMemory" placeholder="1024M" class="iset-input narrow" />
                    </div>
                    <div class="iset-row">
                      <label class="iset-label">最大内存</label>
                      <input v-model="instSettings.maxMemory" placeholder="2048M" class="iset-input narrow" />
                    </div>
                  </div>
                </div>
              </div>
              <div v-if="settingsTab === 'other'" class="iset-section">
                <h3 class="iset-heading">其他</h3>
                <div class="iset-row">
                  <label class="iset-label">下载线程数</label>
                  <input v-model.number="instSettings.downloadConcurrency" type="number" min="1" max="64" class="iset-input narrow" />
                </div>
                <div class="iset-row">
                  <label class="iset-label">校验线程数</label>
                  <input v-model.number="instSettings.verifyConcurrency" type="number" min="1" max="64" class="iset-input narrow" />
                </div>
              </div>
            </div>
            <div class="iset-footer">
              <button class="iset-save-btn" @click="saveSettings">
                <VIcon :size="15"><Save24Regular /></VIcon>
                <span>保存设置</span>
              </button>
              <span v-if="saveMsg" class="iset-save-msg">{{ saveMsg }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.isettings-overlay {
  position: fixed;
  inset: 0;
  top: 38px;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
  animation: fadeIn 0.2s ease;
}

.isettings-window {
  display: flex;
  flex-direction: column;
  width: 780px;
  height: 580px;
  background: var(--content-bg);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25);
  padding: 16px 24px 20px;
  overflow: visible;
  animation: slideUp 0.25s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(16px) scale(0.97);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.isettings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 0 -24px 16px -24px;
  padding: 0 24px;
  color: var(--title-color);
}

.isettings-header-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.isettings-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #00BAAD;
  flex-shrink: 0;
}

.isettings-icon {
  color: #fff;
}

.isettings-title {
  font-size: 14px;
  font-weight: 600;
}

.isettings-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  color: var(--title-color);
  transition: background 0.15s;
  margin-right: -12px;
}

.isettings-close:hover {
  background: rgba(128, 128, 128, 0.15);
}

.isettings-divider {
  height: 1px;
  background: rgba(128, 128, 128, 0.2);
  margin: 0 -24px 20px -24px;
}

.isettings-body {
  flex: 1;
  min-height: 0;
}

.iset-layout {
  display: flex;
  gap: 24px;
  height: 100%;
}

.iset-sidebar {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 140px;
  flex-shrink: 0;
  padding-top: 4px;
}

.iset-nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--title-color);
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
  opacity: 0.55;
  text-align: left;
}

.iset-nav-item:hover {
  background: rgba(128, 128, 128, 0.1);
  opacity: 0.8;
}

.iset-sidebar-spacer {
  flex: 1;
}

.iset-nav-item.active {
  background: rgba(0, 186, 173, 0.12);
  color: #00BAAD;
  opacity: 1;
}

.iset-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.iset-scroll {
  flex: 1;
  overflow-y: auto;
  padding-right: 8px;
}

.iset-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.iset-heading {
  font-size: 14px;
  font-weight: 600;
  color: var(--title-color);
  margin: 0;
  padding-bottom: 4px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.12);
}

.iset-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.iset-label {
  font-size: 13px;
  color: var(--title-color);
  opacity: 0.7;
  min-width: 110px;
  padding-top: 6px;
  flex-shrink: 0;
}

.iset-input {
  flex: 1;
  max-width: 320px;
  padding: 6px 10px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 6px;
  background: var(--panel-bg);
  color: var(--title-color);
  font-size: 13px;
  font-family: inherit;
  outline: none;
  transition: border-color 0.15s;
}

.iset-input:focus {
  border-color: var(--title-color);
}

.iset-input.narrow {
  max-width: 100px;
}

.iset-input.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.iset-toggle {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
  flex-shrink: 0;
  margin-top: 4px;
}

.iset-toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.iset-toggle-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background: rgba(128, 128, 128, 0.3);
  border-radius: 20px;
  transition: background 0.2s;
}

.iset-toggle-slider::before {
  content: "";
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  bottom: 2px;
  background: #fff;
  border-radius: 50%;
  transition: transform 0.2s;
}

.iset-toggle input:checked + .iset-toggle-slider {
  background: #00BAAD;
}

.iset-toggle input:checked + .iset-toggle-slider::before {
  transform: translateX(16px);
}

.iset-footer {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-top: 16px;
  flex-shrink: 0;
}

.iset-save-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 16px;
  border: none;
  border-radius: 8px;
  background: #00BAAD;
  color: #fff;
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s;
}

.iset-save-btn:hover {
  background: #00CFC0;
}

.iset-save-msg {
  font-size: 12px;
  color: var(--title-color);
  opacity: 0.5;
}

.iset-card {
  background: var(--panel-bg);
  border: 1px solid rgba(128, 128, 128, 0.12);
  border-radius: 10px;
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.iset-card-head {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.iset-card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--title-color);
}

.iset-card-desc {
  font-size: 12px;
  color: var(--title-color);
  opacity: 0.45;
}

.iset-card-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.iset-toggle-label {
  font-size: 13px;
  color: var(--title-color);
  opacity: 0.7;
}

.iset-combo-wrap {
  max-width: 280px;
}

.iset-combobox {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  background: var(--panel-bg);
  color: var(--title-color);
  font-size: 13px;
  font-family: inherit;
  outline: none;
  cursor: pointer;
  appearance: auto;
  transition: border-color 0.15s;
}

.iset-combobox:focus {
  border-color: var(--title-color);
}

.mem-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mem-bar {
  height: 8px;
  background: rgba(128, 128, 128, 0.15);
  border-radius: 6px;
  overflow: hidden;
}

.mem-bar-used {
  height: 100%;
  background: #00BAAD;
  border-radius: 6px;
  transition: width 0.3s ease;
}

.mem-info {
  font-size: 12px;
  color: var(--title-color);
  opacity: 0.5;
  line-height: 1.4;
}

.mem-info strong {
  font-weight: 600;
}

.mem-slider-group {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.mem-slider-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.mem-slider-label {
  font-size: 12px;
  color: var(--title-color);
  opacity: 0.6;
  min-width: 56px;
  flex-shrink: 0;
}

.mem-slider-track-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 10px;
  max-width: 320px;
}

.mem-slider {
  flex: 1;
  height: 6px;
  appearance: none;
  background: rgba(128, 128, 128, 0.15);
  border-radius: 4px;
  outline: none;
  cursor: pointer;
}

.mem-slider::-webkit-slider-thumb {
  appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #00BAAD;
  border: 2px solid #fff;
  box-shadow: 0 1px 4px rgba(0,0,0,0.2);
  cursor: pointer;
  transition: transform 0.1s;
}

.mem-slider::-webkit-slider-thumb:hover {
  transform: scale(1.15);
}

.mem-slider::-moz-range-thumb {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #00BAAD;
  border: 2px solid #fff;
  box-shadow: 0 1px 4px rgba(0,0,0,0.2);
  cursor: pointer;
}

.mem-slider-val {
  font-size: 12px;
  color: var(--title-color);
  opacity: 0.7;
  min-width: 60px;
  text-align: right;
  font-family: "Menlo", "Monaco", "Courier New", monospace;
}
</style>
