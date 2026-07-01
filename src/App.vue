<script setup lang="ts">
import { onMounted, ref } from "vue";
import Sidebar from "./components/Sidebar.vue";
import { Home, Settings, LayoutGridIcon, Store, Plus } from "@lucide/vue";
import logo from "./assets/logos/logo.png";
import SettingsInterface from "./components/settings_interface.vue";
import NewMciRoot from "./components/view/new_mci/root_interface.vue";
import HomePage from "./components/view/HomePage.vue";

const isMac = ref(false);
const isLinux = ref(false);
const isMaximized = ref(false);
let appWindow: any = null;

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

  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const { listen } = await import("@tauri-apps/api/event");
  appWindow = getCurrentWindow();
  const ua = navigator.userAgent.toLowerCase();
  isMac.value = ua.includes("mac");
  isLinux.value = ua.includes("linux");

  isMaximized.value = await appWindow.isMaximized();
  listen("tauri://resize", () => {
    appWindow?.isMaximized().then((v: boolean) => (isMaximized.value = v));
  });
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
</script>
 <template>
  <div class="titlebar" :class="{ 'is-win': !isMac && !isLinux, 'is-linux': isLinux }" data-tauri-drag-region>
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
    <Sidebar :mainItems="mainItems" :footerItem="footerItem" :activeId="activeNav" @update:active-id="onNavChange" />
    <main class="content">
      <HomePage v-if="activeNav === 'home'" />
      <div v-else-if="activeNav === 'library'" class="nav-placeholder">{{ activeNav }}</div>
      <div v-else-if="activeNav === 'resourcescenter'" class="nav-placeholder">{{ activeNav }}</div>
    </main>
    <SettingsInterface v-if="showSettings" @close="showSettings = false" />
    <NewMciRoot v-if="showNewInstance" @close="showNewInstance = false" />
  </div>
  <SettingsInterface v-if="showSettings" @close="showSettings = false" />
  <NewMciRoot v-if="showNewInstance" @close="showNewInstance = false" />
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html {
  border-radius: 30px;
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
  background: var(--content-bg);
}

.content {
  flex: 1;
  display: flex;
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
}

html[data-theme="dark"] .win-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

html[data-theme="dark"] .win-btn.close:hover {
  background: #e81123;
  color: #fff;
}
</style>
