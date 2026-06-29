<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import { getSystemLocale } from "../i18n";
import { Settings, Languages, Palette, X } from "@lucide/vue";

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

function onDocMouseDown(e: MouseEvent) {
  if (fontDropdownOpen.value && fontSelectRef.value && !fontSelectRef.value.contains(e.target as Node)) {
    fontDropdownOpen.value = false;
  }
}

onMounted(() => {
  loadFonts();
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
        </nav>
        <div class="settings-content">
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
                  <option value="en_us">{{ t("app.lang.en_us") }}</option>
                </select>
                <span class="select-arrow">
                  <svg width="10" height="10" viewBox="0 0 10 10">
                    <path d="M2 3.5l3 3 3-3" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </span>
              </div>
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
</style>
