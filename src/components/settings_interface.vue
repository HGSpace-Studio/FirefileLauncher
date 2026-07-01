<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import { getSystemLocale } from "../i18n";
import { invoke } from "@tauri-apps/api/core";
import { Settings, Languages, Palette, Info, GitPullRequestArrow, X, Coffee, LoaderCircle, ChevronDown } from "@lucide/vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import logo from "../assets/logos/logo.png";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { t, locale } = useI18n();

const currentLang = ref("system");
const currentTheme = ref(
  document.documentElement.getAttribute("data-theme") || "system"
);

watch(currentLang, (val) => {
  locale.value = val === "system" ? getSystemLocale() : val;
});

watch(currentTheme, (val) => {
  if (val === "system") {
    document.documentElement.removeAttribute("data-theme");
  } else {
    document.documentElement.setAttribute("data-theme", val);
  }
});

const activeSetting = ref("appearance");

const DEFAULT_FONT = "__system_default__";
const availableFonts = ref<string[]>([DEFAULT_FONT]);

function getCurrentFont(): string {
  const raw = document.documentElement.style.getPropertyValue("--app-font").replace(/^"|"$/g, "").trim();
  return raw || DEFAULT_FONT;
}

const selectedFont = ref(getCurrentFont());
const fontDropdownOpen = ref(false);
const fontSelectRef = ref<HTMLElement | null>(null);

const fontChanged = computed(() => selectedFont.value !== DEFAULT_FONT);

const FALLBACK_FONTS = [
  "HarmonyOS Sans", "SF Pro Display", "SF Pro Text",
  "Helvetica Neue", "Helvetica", "Arial", "Segoe UI",
  "Roboto", "Ubuntu", "Noto Sans", "DejaVu Sans",
  "Liberation Sans", "Tahoma", "Verdana", "Trebuchet MS",
  "Times New Roman", "Georgia", "Palatino", "Garamond",
  "Courier New", "Menlo", "Monaco", "Consolas", "Lucida Console",
  "Apple Color Emoji", "Segoe UI Emoji"
];

async function loadFonts() {
  try {
    const fonts: { family: string }[] | undefined = await (window as any).queryLocalFonts?.();
    if (fonts) {
      const families = [...new Set(fonts.map((f: any) => f.family))];
      availableFonts.value = [DEFAULT_FONT, ...families.sort()];
      return;
    }
  } catch { /* fallback */ }
  availableFonts.value = [DEFAULT_FONT, ...FALLBACK_FONTS];
}

function applyFont(font: string) {
  if (font === DEFAULT_FONT) {
    document.documentElement.style.removeProperty("--app-font");
  } else {
    document.documentElement.style.setProperty("--app-font", `"${font}"`);
  }
}

watch(selectedFont, (val) => applyFont(val));

function resetFont() {
  selectedFont.value = DEFAULT_FONT;
}

function getFontStyle(f: string) {
  return f !== DEFAULT_FONT ? { fontFamily: `"${f}"` } : {};
}

interface JavaInstall {
  path: string;
  version: string;
}

const javaInstalls = ref<JavaInstall[]>([]);
const javaLoading = ref(false);
const javaError = ref("");
const javaExpanded = ref(true);

interface SystemMemory {
  total_mb: number;
  used_mb: number;
}

const sysMem = ref<SystemMemory>({ total_mb: 0, used_mb: 0 });
const autoMem = ref(true);

const usedPercent = computed(() => {
  if (sysMem.value.total_mb === 0) return 0;
  return Math.round((sysMem.value.used_mb / sysMem.value.total_mb) * 100);
});

const freePercent = computed(() => {
  return Math.max(0, 100 - usedPercent.value);
});

function cleanVersion(raw: string): string {
  const m = raw.match(/"([^"]+)"/);
  return m ? m[1] : raw;
}

async function fetchJavaVersions() {
  javaLoading.value = true;
  javaError.value = "";
  try {
    javaInstalls.value = await invoke<JavaInstall[]>("get_java_versions");
  } catch (e: any) {
    javaError.value = String(e);
  } finally {
    javaLoading.value = false;
  }
}

function onDocMouseDown(e: MouseEvent) {
  if (fontDropdownOpen.value && fontSelectRef.value && !fontSelectRef.value.contains(e.target as Node)) {
    fontDropdownOpen.value = false;
  }
}

onMounted(() => {
  loadFonts();
  fetchJavaVersions();
  invoke<SystemMemory>("get_system_memory").then((m) => { sysMem.value = m; }).catch(() => {});
  document.addEventListener("mousedown", onDocMouseDown);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", onDocMouseDown);
});
</script>

<template>
  <div class="settings-overlay" @click.self="emit('close')">
    <div class="settings-window">
      <div class="settings-header">
        <div class="settings-header-left">
          <span class="settings-icon-wrap">
            <Settings :size="16" class="settings-icon" />
          </span>
          <span class="settings-title">{{ t("app.mainwindow.settings.title") }}</span>
        </div>
        <button class="settings-close" @click="emit('close')">
          <X :size="18" />
        </button>
      </div>
      <div class="settings-divider"></div>
      <div class="settings-body">
        <nav class="settings-nav">
          <button
            class="settings-nav-item"
            :class="{ active: activeSetting === 'java-runtime' }"
            @click="activeSetting = 'java-runtime'"
          >
            <Coffee :size="18" />
            <span>{{ t("app.mainwindow.settings.java-runtime") }}</span>
          </button>
          <div class="settings-nav-divider"></div>
          <button
            class="settings-nav-item"
            :class="{ active: activeSetting === 'appearance' }"
            @click="activeSetting = 'appearance'"
          >
            <Palette :size="18" />
            <span>{{ t("app.mainwindow.settings.appearance") }}</span>
          </button>
          <button
            class="settings-nav-item"
            :class="{ active: activeSetting === 'language' }"
            @click="activeSetting = 'language'"
          >
            <Languages :size="18" />
            <span>{{ t("app.mainwindow.settings.language") }}</span>
          </button>
          <button
            class="settings-nav-item"
            :class="{ active: activeSetting === 'about' }"
            @click="activeSetting = 'about'"
          >
            <Info :size="18" />
            <span>{{ t("app.mainwindow.settings.about") }}</span>
          </button>
        </nav>
        <div class="settings-content">
          <div v-if="activeSetting === 'java-runtime'" class="setting-panel">
            <div class="expander-group">
              <div class="expander-header" @click="javaExpanded = !javaExpanded">
                <div class="setting-card-info">
                  <label class="setting-label">Java 运行环境</label>
                  <span class="setting-desc">检测到的 Java 安装</span>
                </div>
                <div class="expander-right">
                  <button class="refresh-btn" @click.stop="fetchJavaVersions" :disabled="javaLoading">
                    <LoaderCircle v-if="javaLoading" :size="14" class="spinner" />
                    <span v-else>刷新</span>
                  </button>
                  <ChevronDown :size="18" class="expander-chevron" :class="{ expanded: javaExpanded }" />
                </div>
              </div>
              <Transition name="expander">
                <div v-if="javaExpanded" class="expander-body">
                  <div v-if="javaError" class="java-error">{{ javaError }}</div>
                  <div v-if="javaInstalls.length === 0 && !javaLoading && !javaError" class="java-empty">
                    未检测到 Java 安装
                  </div>
                  <div v-for="(jv, idx) in javaInstalls" :key="jv.path" class="java-item" :class="{ last: idx === javaInstalls.length - 1 }">
                    <div class="java-item-divider"></div>
                    <span class="java-item-version">{{ cleanVersion(jv.version) }}</span>
                    <span class="java-item-path">{{ jv.path }}</span>
                  </div>
                </div>
              </Transition>
            </div>
            <div class="expander-group">
              <div class="expander-header">
                <div class="setting-card-info">
                  <label class="setting-label">游戏运行分配</label>
                  <span class="setting-desc">设置 Minecraft 实例可运行的运存容量</span>
                </div>
              </div>
              <div class="expander-body">
                  <div class="mem-item">
                    <div class="mem-item-row">
                      <div class="mem-item-info">
                        <span class="mem-item-label">自动分配</span>
                        <span class="mem-item-desc">根据计算机已用运行内存情况分配</span>
                      </div>
                      <label class="switch">
                        <input type="checkbox" v-model="autoMem" />
                        <span class="switch-slider"></span>
                      </label>
                    </div>
                  </div>
                  <div class="mem-item">
                    <div class="mem-slider">
                      <div class="mem-slider-bar">
                        <div class="mem-slider-segment mem-slider-used" :style="{ width: usedPercent + '%' }"></div>
                        <div class="mem-slider-segment mem-slider-free" :style="{ width: freePercent + '%' }"></div>
                      </div>
                    </div>
                    <span class="mem-status">您的计算机一共有 {{ sysMem.total_mb }} MB，已经用了 {{ sysMem.used_mb }} MB</span>
                  </div>
                </div>
            </div>
          </div>
          <div v-if="activeSetting === 'appearance'" class="setting-panel">
            <div class="setting-card">
              <div class="setting-card-info">
                <label class="setting-label">{{ t("app.mainwindow.settings.appearance.theme") }}</label>
                <span class="setting-desc">{{ t("app.mainwindow.settings.appearance.theme.desc") }}</span>
              </div>
              <div class="custom-select">
                <select v-model="currentTheme">
                  <option value="system">{{ t("app.mainwindow.settings.appearance.theme.system") }}</option>
                  <option value="light">{{ t("app.mainwindow.settings.appearance.theme.light") }}</option>
                  <option value="dark">{{ t("app.mainwindow.settings.appearance.theme.dark") }}</option>
                </select>
                <span class="select-arrow">
                  <svg width="10" height="10" viewBox="0 0 10 10">
                    <path d="M2 3.5l3 3 3-3" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </span>
              </div>
            </div>
            <div class="setting-card">
              <div class="setting-card-info">
                <label class="setting-label">{{ t("app.mainwindow.settings.appearance.font") }}</label>
                <span class="setting-desc">{{ t("app.mainwindow.settings.appearance.font.desc") }}</span>
              </div>
              <div class="font-select-group">
                <div class="custom-select" ref="fontSelectRef">
                  <button class="font-select-trigger" @click.stop="fontDropdownOpen = !fontDropdownOpen">
                    <span class="font-select-text">{{ selectedFont === DEFAULT_FONT ? t("app.mainwindow.settings.appearance.font.default") : selectedFont }}</span>
                  </button>
                  <span class="select-arrow">
                    <svg width="10" height="10" viewBox="0 0 10 10">
                      <path d="M2 3.5l3 3 3-3" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </span>
                  <div v-if="fontDropdownOpen" class="font-dropdown">
                    <button
                      v-for="f in availableFonts"
                      :key="f"
                      class="font-option"
                      :class="{ active: f === selectedFont }"
                      :style="getFontStyle(f)"
                      @click="selectedFont = f; fontDropdownOpen = false"
                    >{{ f === DEFAULT_FONT ? t("app.mainwindow.settings.appearance.font.default") : f }}</button>
                  </div>
                </div>
                <button v-if="fontChanged" class="reset-btn" @click="resetFont">{{ t("app.mainwindow.settings.appearance.font.reset") }}</button>
              </div>
            </div>
          </div>
          <div v-if="activeSetting === 'language'" class="setting-panel">
            <div class="setting-card">
              <div class="setting-card-info">
                <label class="setting-label">{{ t("app.mainwindow.settings.language") }}</label>
                <span class="setting-desc">{{ t("app.mainwindow.settings.language.desc") }}</span>
              </div>
              <div class="custom-select">
                <select v-model="currentLang">
                  <option value="system">{{ t("app.lang.system") }}</option>
                  <option value="zh_cn">{{ t("app.lang.zh_cn") }}</option>
                  <option value="ko_kr">{{ t("app.lang.ko_kr") }}</option>
                  <option value="en_us">{{ t("app.lang.en_us") }}</option>
                  <option value="fr">{{ t("app.lang.fr") }}</option>
                  <option value="ru">{{ t("app.lang.ru") }}</option>
                  <option value="vi">{{ t("app.lang.vi") }}</option>
                </select>
                <span class="select-arrow">
                  <svg width="10" height="10" viewBox="0 0 10 10">
                    <path d="M2 3.5l3 3 3-3" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </span>
              </div>
            </div>
            <p class="language-notice">{{ t("app.mainwindow.settings.languageNotice") }}</p>
          </div>
          <div v-if="activeSetting === 'about'" class="setting-panel about-panel">
            <div class="setting-card about-card">
              <div class="about-info">
                <img :src="logo" class="about-logo" alt="logo" />
                <div class="about-meta">
                  <span class="about-name">{{ t("app.mainwindow.settings.about.name") }}</span>
                  <span class="about-version">{{ t("app.mainwindow.settings.about.version") }}</span>
                  
                </div>
              </div>
              <button class="about-link-btn" @click="openUrl('https://github.com/HGSpace-Studio/FirefileLauncher')">
                <GitPullRequestArrow :size="16" />
                <span>GitHub</span>
              </button>
            </div>
            <div class="setting-card about-desc-card">
              <span class="about-desc-text">{{ t("app.mainwindow.settings.about.desc") }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-overlay {
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

.settings-window {
  display: flex;
  flex-direction: column;
  width: 680px;
  height: 500px;
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

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 0 -24px 16px -24px;
  padding: 0 24px;
  color: var(--title-color);
}

.settings-header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.settings-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #0078d4;
  flex-shrink: 0;
}

.settings-icon {
  color: #fff;
}

.settings-title {
  font-size: 14px;
  font-weight: 600;
}

.settings-close {
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

.settings-close:hover {
  background: rgba(128, 128, 128, 0.15);
}

.settings-divider {
  height: 1px;
  background: rgba(128, 128, 128, 0.2);
  margin: 0 -24px 20px -24px;
}

.settings-body {
  display: flex;
  gap: 24px;
  flex: 1;
  min-height: 0;
}

.settings-nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 180px;
  border-right: 1px solid rgba(128, 128, 128, 0.2);
  padding-right: 16px;
  box-sizing: border-box;
}

.settings-nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  font-family: inherit;
  color: var(--title-color);
  text-align: left;
  transition: background 0.15s;
}

.settings-nav-item:hover {
  background: rgba(128, 128, 128, 0.1);
}

.settings-nav-item.active {
  background: var(--sidebar-active);
  color: var(--sidebar-active-color);
}

.settings-nav-divider {
  height: 1px;
  background: rgba(128, 128, 128, 0.2);
  margin: 6px 12px;
}

.settings-content {
  flex: 1;
}

.setting-panel {
  display: flex;
  flex-direction: column;
  gap: 9px;
}

.setting-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--panel-bg);
  border-radius: 10px;
}

.setting-card-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.setting-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--title-color);
  opacity: 0.8;
}

.setting-desc {
  font-size: 11px;
  opacity: 0.5;
  color: var(--title-color);
}

.custom-select {
  position: relative;
  width: 200px;
}

.custom-select select {
  appearance: none;
  width: 100%;
  padding: 8px 32px 8px 12px;
  font-size: 13px;
  font-family: inherit;
  color: var(--title-color);
  background: var(--panel-bg);
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  cursor: pointer;
  outline: none;
  transition: border-color 0.15s;
}

.custom-select select:hover {
  border-color: rgba(128, 128, 128, 0.4);
}

.custom-select select:focus {
  border-color: #0078d4;
}

.custom-select select option {
  background: var(--panel-bg);
  color: var(--title-color);
}

.select-arrow {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
  color: var(--title-color);
  opacity: 0.6;
}

.font-select-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.font-select-trigger {
  width: 100%;
  padding: 8px 32px 8px 12px;
  font-size: 13px;
  font-family: inherit;
  color: var(--title-color);
  background: var(--panel-bg);
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  cursor: pointer;
  outline: none;
  transition: border-color 0.15s;
  text-align: left;
}

.font-select-trigger:hover {
  border-color: rgba(128, 128, 128, 0.4);
}

.font-select-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: block;
}

.font-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  max-height: 200px;
  overflow-y: auto;
  background: var(--panel-bg);
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18);
  z-index: 60;
}

.font-option {
  display: block;
  width: 100%;
  padding: 7px 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  text-align: left;
  color: var(--title-color);
  transition: background 0.1s;
}

.font-option:hover {
  background: rgba(128, 128, 128, 0.12);
}

.font-option.active {
  background: var(--sidebar-active);
  color: var(--sidebar-active-color);
}

.reset-btn {
  flex-shrink: 0;
  padding: 8px 12px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  font-family: inherit;
  color: var(--sidebar-active-color);
  white-space: nowrap;
  transition: background 0.15s;
}

.reset-btn:hover {
  background: var(--sidebar-active);
}

.language-notice {
  font-size: 11px;
  line-height: 1.5;
  color: var(--title-color);
  opacity: 0.5;
  padding: 0 4px;
  margin: 0;
}

.refresh-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 6px 14px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  font-size: 12px;
  font-family: inherit;
  color: var(--sidebar-active-color);
  transition: background 0.15s;
  flex-shrink: 0;
}

.refresh-btn:hover:not(:disabled) {
  background: var(--sidebar-active);
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.expander-group {
  background: var(--panel-bg);
  border-radius: 10px;
  overflow: hidden;
}

.expander-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  cursor: pointer;
}

.expander-header:active {
  opacity: 0.8;
}

.expander-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.expander-chevron {
  color: var(--title-color);
  opacity: 0.4;
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.expander-chevron.expanded {
  transform: rotate(180deg);
}

.expander-body {
  display: flex;
  flex-direction: column;
}

.expander-enter-active,
.expander-leave-active {
  transition: opacity 0.2s ease;
}

.expander-enter-from,
.expander-leave-to {
  opacity: 0;
}

.java-error {
  font-size: 12px;
  color: #e74c3c;
  padding: 8px 16px;
  opacity: 0.8;
}

.java-empty {
  font-size: 12px;
  color: var(--title-color);
  opacity: 0.45;
  padding: 16px;
  text-align: center;
}

.java-item {
  padding: 6px 16px;
}

.java-item.last {
  border-radius: 0 0 10px 10px;
  padding-bottom: 8px;
}

.java-item-divider {
  height: 1px;
  background: rgba(128, 128, 128, 0.15);
  margin: 0 -16px 6px;
}

.java-item-version {
  display: block;
  font-size: 14px;
  font-weight: 600;
  color: var(--title-color);
  line-height: 1.2;
}

.java-item-path {
  display: block;
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.5;
  line-height: 1;
  word-break: break-all;
  margin-top: 2px;
}

.mem-item {
  padding: 12px 16px;
}

.mem-item-divider {
  height: 1px;
  background: rgba(128, 128, 128, 0.15);
  margin: 0 -16px 6px;
}

.mem-item-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.mem-item-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.mem-item-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--title-color);
  line-height: 1.2;
}

.mem-item-desc {
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.5;
  line-height: 1;
}

.switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
  flex-shrink: 0;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.switch-slider {
  position: absolute;
  inset: 0;
  background: rgba(128, 128, 128, 0.3);
  border-radius: 11px;
  cursor: pointer;
  transition: background 0.2s;
}

.switch-slider::before {
  content: "";
  position: absolute;
  width: 18px;
  height: 18px;
  left: 2px;
  bottom: 2px;
  background: #fff;
  border-radius: 50%;
  transition: transform 0.2s;
}

.switch input:checked + .switch-slider {
  background: #0078d4;
}

.switch input:checked + .switch-slider::before {
  transform: translateX(18px);
}

.mem-slider {
  padding: 0 0 6px;
  margin-top: 3px;
}

.mem-slider-bar {
  display: flex;
  height: 6px;
  border-radius: 3px;
  overflow: hidden;
  background: rgba(128, 128, 128, 0.15);
}

.mem-slider-segment {
  transition: width 0.3s ease;
}

.mem-slider-used {
  background: #0078d4;
}

.mem-slider-free {
  background: rgba(128, 128, 128, 0.2);
}

.mem-status {
  display: block;
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.45;
  line-height: 1;
  padding-bottom: 8px;
}

.about-panel {
  gap: 0;
}

.about-card {
  padding: 16px;
}

.about-info {
  display: flex;
  align-items: center;
  gap: 14px;
}

.about-logo {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  flex-shrink: 0;
}

.about-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.about-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--title-color);
}

.about-version {
  font-size: 12px;
  color: var(--title-color);
  opacity: 0.6;
}

.about-desc {
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.45;
  line-height: 1.4;
}

.about-link-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  font-family: inherit;
  color: var(--title-color);
  opacity: 0.6;
  transition: opacity 0.15s, background 0.15s;
  flex-shrink: 0;
}

.about-link-btn:hover {
  opacity: 1;
  background: rgba(128, 128, 128, 0.12);
}

.about-desc-card {
  margin-top: 7px;
  padding: 14px 16px;
}

.about-desc-text {
  font-size: 12px;
  line-height: 1.6;
  color: var(--title-color);
  opacity: 0.55;
}
</style>
