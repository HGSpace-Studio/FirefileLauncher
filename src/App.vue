<script setup lang="ts">
import { onMounted, ref } from "vue";
import Sidebar from "./components/Sidebar.vue";
import { Home, Settings, LayoutGridIcon, Store, Plus } from "@lucide/vue";
import logo from "./assets/logos/logo.png";
import SettingsInterface from "./components/settings_interface.vue";
import NewMciRoot from "./components/view/new_mci/root_interface.vue";
import HomePage from "./components/view/HomePage.vue";
import LibraryPage from "./components/view/rootpages/versionroot.vue";
import ResourcesCenter from "./components/view/ResourcesCenter.vue";
import AccountInterface from "./components/accinterface.vue";
import OnboardingWindow from "./components/view/onboarding/OnboardingWindow.vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

const appWindow = getCurrentWindow();
const isOobeWindow = appWindow.label === "oobe";

const isMac = ref(false);
const isLinux = ref(false);
const isMaximized = ref(false);

const mainItems = [
  { id: "home", label: "首页", icon: Home },
  
  { id: "resourcescenter", label: "资源中心", icon: Store },
  { id: "library", label: "库", icon: LayoutGridIcon },
  { id: "add-instance", label: "新建实例", icon: Plus, highlight: true },
];

const footerItem = { id: "settings", label: "设置", icon: Settings };

const activeNav = ref("home");

const showSettings = ref(false);
const showNewInstance = ref(false);

const userName = ref("");
const userType = ref("");

async function loadAccount() {
  try {
    const acc = await invoke<{ name: string; account_type: string; uuid: string }>("get_current_account");
    userName.value = acc.name;
    userType.value = acc.account_type;
  } catch {
    // ignore
  }
}

function onNavChange(id: string) {
  if (id === "settings") {
    showSettings.value = true;
    return;
  }
  if (id === "add-instance") {
    showNewInstance.value = true;
    return;
  }
  activeNav.value = id;
}

onMounted(async () => {
  document.addEventListener("contextmenu", (e) => e.preventDefault());
  const ua = navigator.userAgent.toLowerCase();
  isMac.value = ua.includes("mac");
  isLinux.value = ua.includes("linux");

  isMaximized.value = await appWindow.isMaximized();
  listen("tauri://resize", () => {
    appWindow?.isMaximized().then((v: boolean) => (isMaximized.value = v));
  });

  loadAccount();

  listen("account-refresh", () => {
    loadAccount();
  });

  window.addEventListener("account-changed", loadAccount);
});

function minimize() {
  appWindow?.minimize();
}

function toggleMaximize() {
  appWindow?.toggleMaximize().then(() =>
    appWindow?.isMaximized().then((v: boolean) => (isMaximized.value = v))
  );
}

function closeWindow() {
  appWindow?.close();
}

function handleTitlebarMouseDown(e: MouseEvent) {
  // 如果点击的是按钮或交互元素，不触发拖拽
  const target = e.target as HTMLElement;
  if (
    target.closest("button") ||
    target.closest("input") ||
    target.closest("select") ||
    target.closest("a")
  ) {
    return;
  }
  // Windows: 由 CSS -webkit-app-region: drag 处理
  // macOS: 由 titleBarStyle 原生处理
  // Linux (WebKitGTK 不支持 -webkit-app-region): 使用 Tauri 手动拖拽 API
  if (isLinux.value) {
    appWindow?.startDragging();
  }
}
</script>
 <template>
  <OnboardingWindow v-if="isOobeWindow" />
  <template v-else>
    <div class="titlebar" :class="{ 'is-win': !isMac && !isLinux, 'is-linux': isLinux }" @mousedown="handleTitlebarMouseDown">
      <div v-if="isMac" class="traffic-light-area"></div>
      <div class="logo-wrap" :class="{ 'is-mac': isMac }">
        <img class="logo" :src="logo" alt="" />
        <span class="logo-text">Firefiles Launcher</span>
      </div>
      <span class="title"></span>
      <div v-if="!isMac" class="win-controls">
        <button class="win-btn" @click="minimize">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="1" y="5.5" width="10" height="1" fill="currentColor" />
          </svg>
        </button>
        <button class="win-btn" @click="toggleMaximize">
          <svg v-if="!isMaximized" width="12" height="12" viewBox="0 0 12 12">
            <rect x="1.5" y="1.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.1" />
          </svg>
          <svg v-else width="12" height="12" viewBox="0 0 12 12">
            <rect x="2.5" y="4.5" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1" />
            <rect x="3.5" y="2.5" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1" />
          </svg>
        </button>
        <button class="win-btn close" @click="closeWindow">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" stroke-width="1.2" />
            <line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" stroke-width="1.2" />
          </svg>
        </button>
      </div>
    </div>
    <div class="viewport">
      <Sidebar
        :mainItems="mainItems"
        :footerItem="footerItem"
        :activeId="activeNav"
        :userName="userName"
        :userType="userType"
        @update:active-id="onNavChange"
      />
      <main class="content">
        <HomePage v-if="activeNav === 'home'" />
        <LibraryPage v-else-if="activeNav === 'library'" @open-new-instance="showNewInstance = true" />
        <ResourcesCenter v-else-if="activeNav === 'resourcescenter'" />
        <AccountInterface v-else-if="activeNav === 'account'" />
      </main>
      <SettingsInterface v-if="showSettings" @close="showSettings = false" />
      <NewMciRoot v-if="showNewInstance" @close="showNewInstance = false" @navigate="(id: string) => { showNewInstance = false; activeNav = id }" />
    </div>
    <SettingsInterface v-if="showSettings" @close="showSettings = false" />
    <NewMciRoot v-if="showNewInstance" @close="showNewInstance = false" @navigate="(id: string) => { showNewInstance = false; activeNav = id }" />
  </template>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html {
  border-radius: 16px;
  overflow: hidden;
}

@font-face {
  font-family: "HarmonyOS Sans";
  src: url("./assets/fonts/HarmonyOS_Sans_Regular.ttf") format("truetype");
  font-weight: 400;
  font-style: normal;
}

body {
  font-family: var(--app-font, "HarmonyOS Sans"), -apple-system, BlinkMacSystemFont, "Segoe UI",
    Roboto, Helvetica, Arial, sans-serif;
  overflow: hidden;
  height: 100vh;
  background: var(--panel-bg);
}

.titlebar {
  display: flex;
  align-items: center;
  height: 38px;
  user-select: none;
  background: var(--panel-bg);
  border-radius: 16px 16px 0 0;
  -webkit-app-region: drag;
}

.titlebar.is-mac {
  border-radius: 0 16px 0 0;
}

.traffic-light-area {
  width: 70px;
  min-width: 70px;
  height: 100%;
}

.logo-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.logo-wrap.is-mac {
  margin-left: 19px;
}

.titlebar.is-win .logo-wrap,
.titlebar.is-linux .logo-wrap {
  margin-left: 14px;
}

.logo {
  height: 20px;
  width: auto;
}

.logo-text {
  font-size: 13px;
  font-weight: 600;
  opacity: 0.85;
  white-space: nowrap;
  color: var(--title-color);
}

.titlebar.is-win,
.titlebar.is-linux {
  padding-right: 0px;
}

.title {
  flex: 1;
  text-align: center;
  font-size: 13px;
  font-weight: 600;
  opacity: 0.85;
  color: var(--title-color);
}

.win-controls {
  display: flex;
  height: 100%;
  -webkit-app-region: no-drag;
}

.win-btn {
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--title-color);
  transition: background 0.1s;
  -webkit-app-region: no-drag;
}

.win-btn:hover {
  background: rgba(0, 0, 0, 0.08);
}

.win-btn.close:hover {
  background: #e81123;
  color: #fff;
}

.viewport {
  height: calc(100vh - 38px);
  display: flex;
  overflow: hidden;
  border-radius: 0 0 16px 16px;
}

.content {
  flex: 1;
  display: flex;
  border-radius: 13px 0 0 0;
  overflow: hidden;
  background: var(--content-bg);
}

.nav-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  opacity: 0.6;
}

@media (prefers-color-scheme: light) {
  :root {
    --panel-bg: #ececec;
    --title-color: #1d1d1f;
    --sidebar-color: #1d1d1f;
    --sidebar-hover: rgba(0, 0, 0, 0.08);
    --sidebar-active: rgba(0, 120, 212, 0.15);
    --sidebar-active-color: #0078d4;
    --tooltip-bg: #1d1d1f;
    --tooltip-color: #f5f5f7;
    --content-bg: #f6f6f6;
    --settings-icon-bg: #0078d4;
    --window-border: rgba(0, 0, 0, 0.12);
  }
}

@media (prefers-color-scheme: dark) {
  :root {
    --panel-bg: #2d2d2d;
    --title-color: #f5f5f7;
    --sidebar-color: #e0e0e0;
    --sidebar-hover: rgba(255, 255, 255, 0.08);
    --sidebar-active: rgba(59, 130, 246, 0.2);
    --sidebar-active-color: #60a5fa;
    --tooltip-bg: #e0e0e0;
    --tooltip-color: #1d1d1f;
    --content-bg: #1c1c1e;
    --settings-icon-bg: #60a5fa;
    --window-border: rgba(255, 255, 255, 0.12);
  }
  .win-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .win-btn.close:hover {
    background: #e81123;
    color: #fff;
  }
}

html[data-theme="light"] {
  --panel-bg: #ececec;
  --title-color: #1d1d1f;
  --sidebar-color: #1d1d1f;
  --sidebar-hover: rgba(0, 0, 0, 0.08);
  --sidebar-active: rgba(0, 120, 212, 0.15);
  --sidebar-active-color: #0078d4;
  --tooltip-bg: #1d1d1f;
  --tooltip-color: #f5f5f7;
  --content-bg: #f6f6f6;
  --settings-icon-bg: #0078d4;
  --window-border: rgba(0, 0, 0, 0.12);
}

html[data-theme="dark"] {
  --panel-bg: #2d2d2d;
  --title-color: #f5f5f7;
  --sidebar-color: #e0e0e0;
  --sidebar-hover: rgba(255, 255, 255, 0.08);
  --sidebar-active: rgba(59, 130, 246, 0.2);
  --sidebar-active-color: #60a5fa;
  --tooltip-bg: #e0e0e0;
  --tooltip-color: #1d1d1f;
  --content-bg: #1c1c1e;
  --settings-icon-bg: #60a5fa;
  --window-border: rgba(255, 255, 255, 0.12);
}

html[data-theme="dark"] .win-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

html[data-theme="dark"] .win-btn.close:hover {
  background: #e81123;
  color: #fff;
}
</style>
