<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import Sidebar from "./components/Sidebar.vue";
import { Home, Settings, LayoutGridIcon, Store, Plus, Bolt, Square, ChevronRight } from "@lucide/vue";
import { useTaskStore } from "./stores/taskStore";
import { navigateToInstance } from "./stores/navigation";
import logo from "./assets/logos/logo.png";
import SettingsInterface from "./components/settings_interface.vue";
import NewMciRoot from "./components/view/new_mci/root_interface.vue";
import HomePage from "./components/view/HomePage.vue";
import LibraryPage from "./components/view/rootpages/versionroot.vue";
import ResourcesCenter from "./components/view/ResourcesCenter.vue";
import AccountInterface from "./components/accinterface.vue";
import OnboardingWindow from "./components/view/onboarding/OnboardingWindow.vue";
import CrashShell from "./components/view/window/crush_shell.vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

const appWindow = getCurrentWindow();
const isOobeWindow = appWindow.label === "oobe";
const isCrashShellWindow = appWindow.label === "crash-shell";

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
const taskDropdownOpen = ref(false);
const taskTab = ref<"tasks" | "running">("tasks");

const { tasks } = useTaskStore();
const runningTasks = computed(() => tasks.value.filter(t => t.status === "running"))

function goToInstance(inst: { name: string; version: string; version_type: string; loader?: { type: "fabric" | "forge" | "neoforge" | "quilt"; version: string }; icon?: string }) {
  taskDropdownOpen.value = false;
  navigateToInstance(inst);
  onNavChange("library");
}

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

  document.addEventListener("click", (e) => {
    const target = e.target as HTMLElement;
    if (!target.closest(".task-btn-wrap")) {
      taskDropdownOpen.value = false;
    }
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
  <CrashShell v-else-if="isCrashShellWindow" />
  <template v-else>
    <div class="titlebar" :class="{ 'is-win': !isMac && !isLinux, 'is-linux': isLinux, 'is-mac': isMac }" @mousedown="handleTitlebarMouseDown">
      <div class="logo-wrap" :class="{ 'is-mac': isMac }">
        <img class="logo" :src="logo" alt="" />
        <span class="logo-text">Firefiles Launcher</span>
      </div>
      <span class="title"></span>
      <div class="task-btn-wrap">
        <button class="task-btn" @click.stop="taskDropdownOpen = !taskDropdownOpen">
          <Bolt :size="16" />
          <span class="task-btn-label">{{ tasks.length > 0 ? tasks.length + ' 个任务进行中' : '还没有任务啊' }}</span>
        </button>
        <div v-if="taskDropdownOpen" class="task-dropdown">
          <div class="task-dropdown-tabs">
            <button class="task-dropdown-tab" :class="{ active: taskTab === 'tasks' }" @click="taskTab = 'tasks'">
              下载任务
            </button>
            <button class="task-dropdown-tab" :class="{ active: taskTab === 'running' }" @click="taskTab = 'running'">
              运行中
            </button>
          </div>
          <div v-if="taskTab === 'tasks'">
            <div v-if="tasks.length === 0" class="task-empty">暂无任务</div>
            <div v-for="task in tasks" :key="task.id" class="task-item">
              <div class="task-item-header">
                <span class="task-item-title">{{ task.title }}</span>
                <span class="task-item-type">{{ task.type === 'launch' ? '启动' : '安装' }}</span>
              </div>
              <span class="task-item-label">{{ task.label }}</span>
              <div class="task-item-bar">
                <div class="task-item-fill" :style="{ width: (task.progress * 100) + '%' }"></div>
              </div>
            </div>
          </div>
          <div v-if="taskTab === 'running'">
            <div v-if="runningTasks.length === 0" class="task-empty">没有运行中的游戏</div>
            <div v-for="task in runningTasks" :key="task.id" class="task-item">
              <div class="task-item-header">
                <span class="task-item-title">{{ task.title }}</span>
              </div>
              <div class="task-item-actions">
                <button class="task-item-action-btn stop" @click="invoke('stop_game')" title="停止">
                  <Square :size="14" />
                </button>
                <button class="task-item-action-btn" @click="goToInstance({ name: task.title, version: '', version_type: '' })" title="跳转">
                  <ChevronRight :size="14" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div v-if="!isMac" class="win-controls">
        <button class="win-btn minimize" @click="minimize">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <line x1="1.5" y1="6" x2="10.5" y2="6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          </svg>
        </button>
        <button class="win-btn maximize" @click="toggleMaximize">
          <svg v-if="!isMaximized" width="12" height="12" viewBox="0 0 12 12">
            <rect x="1.5" y="1.5" width="9" height="9" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
          </svg>
          <svg v-else width="12" height="12" viewBox="0 0 12 12">
            <rect x="2" y="4" width="7" height="7" rx="0.5" fill="none" stroke="currentColor" stroke-width="1"/>
            <rect x="4" y="2" width="7" height="7" rx="0.5" fill="none" stroke="currentColor" stroke-width="1"/>
          </svg>
        </button>
        <button class="win-btn close" @click="closeWindow">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
            <line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
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
  padding-left: 70px;
  border-radius: 0 16px 0 0;
  position: relative;
}

.titlebar.is-mac::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  width: 70px;
  height: 100%;
  -webkit-app-region: no-drag;
  z-index: 1;
}

.logo-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.titlebar.is-mac .logo-wrap {
  margin-left: 9px;
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
  transition: background 0.1s, box-shadow 0.1s;
  -webkit-app-region: no-drag;
  border-radius: 10px;
}

.win-btn:hover {
  background: rgba(0, 0, 0, 0.08);
  box-shadow: 0 0 10px rgba(0, 0, 0, 0.15);
}

.win-btn.close:hover {
  background: #e81123;
  color: #fff;
  box-shadow: 0 0 10px rgba(232, 17, 35, 0.5);
}

.task-btn-wrap {
  position: relative;
  display: flex;
  align-items: center;
  height: 100%;
  margin-right: 4px;
  -webkit-app-region: no-drag;
}

.titlebar.is-mac .task-btn-wrap {
  margin-right: 12px;
}

.task-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 10px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--title-color);
  opacity: 0.6;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
  white-space: nowrap;
}

.task-btn:hover {
  background: rgba(128, 128, 128, 0.15);
  opacity: 0.9;
}

.task-btn-label {
  font-size: 12px;
  line-height: 1;
}

.task-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  width: 300px;
  max-height: 360px;
  overflow-y: auto;
  background: var(--panel-bg);
  border: 1px solid rgba(128, 128, 128, 0.15);
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  padding: 8px;
  z-index: 100;
}

.task-empty {
  text-align: center;
  color: var(--title-color);
  opacity: 0.4;
  font-size: 13px;
  padding: 24px 0;
}

.task-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border-radius: 8px;
  transition: background 0.15s;
}

.task-item:hover {
  background: rgba(128, 128, 128, 0.06);
}

.task-item + .task-item {
  margin-top: 4px;
}

.task-item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.task-item-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--title-color);
  line-height: 1.2;
}

.task-item-type {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(128, 128, 128, 0.1);
  color: var(--title-color);
  opacity: 0.55;
}

.task-item-label {
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.5;
  line-height: 1;
}

.task-item-bar {
  width: 100%;
  height: 3px;
  border-radius: 2px;
  background: rgba(128, 128, 128, 0.12);
  overflow: hidden;
}

.task-item-fill {
  height: 100%;
  border-radius: 2px;
  background: #0078d4;
  transition: width 0.3s ease;
}

.task-dropdown-tabs {
  display: flex;
  gap: 2px;
  margin-bottom: 8px;
  padding: 2px;
  background: rgba(128, 128, 128, 0.08);
  border-radius: 8px;
}

.task-dropdown-tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--title-color);
  opacity: 0.5;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
}

.task-dropdown-tab:hover {
  opacity: 0.8;
}

.task-dropdown-tab.active {
  background: var(--panel-bg);
  opacity: 1;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.task-item-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 4px;
}

.task-item-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--title-color);
  opacity: 0.45;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
}

.task-item-action-btn:hover {
  background: rgba(128, 128, 128, 0.12);
  opacity: 0.8;
}

.task-item-action-btn.stop:hover {
  background: rgba(212, 58, 58, 0.15);
  color: #d43a3a;
  opacity: 1;
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
  border: 2px solid var(--content-border);
  border-bottom: none;
  border-right: none;
  box-shadow: 0 0 12px rgba(0, 0, 0, 0.08);
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
    --content-border: rgba(0, 0, 0, 0.25);
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
    --content-border: rgba(255, 255, 255, 0.2);
  }
  .win-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.3);
  }
  .win-btn.close:hover {
    background: #e81123;
    color: #fff;
    box-shadow: 0 0 10px rgba(232, 17, 35, 0.5);
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
  --content-border: rgba(0, 0, 0, 0.25);
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
  --content-border: rgba(255, 255, 255, 0.2);
}

html[data-theme="dark"] .win-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  box-shadow: 0 0 10px rgba(0, 0, 0, 0.3);
}

html[data-theme="dark"] .win-btn.close:hover {
  background: #e81123;
  color: #fff;
  box-shadow: 0 0 10px rgba(232, 17, 35, 0.5);
}
</style>
