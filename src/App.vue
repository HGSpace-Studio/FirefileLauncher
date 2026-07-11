<script setup lang="ts">
import { ref, computed, provide, onMounted } from "vue";
import { useTaskStore } from "./stores/taskStore";
import { navigateToInstance } from "./stores/navigation";
import { currentInstanceName, currentLaunchFn, currentStopFn } from "./stores/instanceLaunch";
import { openedInstances } from "./stores/openedInstances";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Home, Store, LayoutGridIcon, Plus, Settings, Bolt, Square, ChevronRight, Play, LoaderCircle, Gamepad2 } from "@lucide/vue";
import logo from "./assets/logos/logo.png";
import steve from "./assets/imgs/skins/avator/steve.png";
import alex from "./assets/imgs/skins/avator/alex.png";

import NewMciRoot from "./components/view/new_mci/root_interface.vue";
import HomePage from "./components/view/HomePage.vue";
import LibraryPage from "./components/view/rootpages/versionroot.vue";
import ResourcesCenter from "./components/view/ResourcesCenter.vue";
import AccountInterface from "./components/accinterface.vue";
import SettingsInterface from "./components/settings_interface.vue";
import OnboardingWindow from "./components/view/onboarding/OnboardingWindow.vue";
import CrashShell from "./components/view/window/crush_shell.vue";
const app = getCurrentWindow();
const isOobe = app.label === "oobe";
const isCrash = app.label === "crash-shell";

const isMac = navigator.userAgent.toLowerCase().includes("mac");
const isLinux = navigator.userAgent.toLowerCase().includes("linux");
const maxed = ref(false);
const nav = ref("home");
const showSettings = ref(false);
const showNewInst = ref(false);
const showingInstance = ref(false);
provide('showingInstance', showingInstance);
const goBackLib = ref(0);
provide('goBackLib', goBackLib);
const taskOpen = ref(false);
const userName = ref("");
const userType = ref("");

const avatar = computed(() => {
  const imgs = [steve, alex];
  const i = userName.value ? userName.value.charCodeAt(0) % 2 : 0;
  return imgs[i];
});

const userLabel = computed(() => {
  const m: Record<string, string> = { offline: "离线账号", microsoft: "微软账户" };
  return m[userType.value] || "";
});

const navItems = [
  { id: "home", icon: Home },
  { id: "resourcescenter", icon: Store },
  { id: "library", icon: LayoutGridIcon },
  { id: "add-instance", icon: Plus },
  { id: "settings", icon: Settings },
];
const taskTab = ref<"tasks" | "running">("tasks");
const { tasks } = useTaskStore();
const running = computed(() => tasks.value.filter(t => t.status === "running"));
const dockTask = computed(() => {
  if (!currentInstanceName.value) return null;
  return tasks.value.find(t => t.id === "launch:" + currentInstanceName.value) || null;
});

function onDockLaunch() {
  if (dockTask.value?.status === "running" && currentStopFn.value) {
    currentStopFn.value();
  } else if (currentLaunchFn.value) {
    currentLaunchFn.value();
  }
}

function onNav(id: string) {
  if (id === "settings") { showSettings.value = true; return; }
  if (id === "add-instance") { showNewInst.value = true; return; }
  if (id === "library" && showingInstance.value) {
    goBackLib.value++;
    return;
  }
  nav.value = id;
}

function goInst(inst: any) {
  taskOpen.value = false;
  nav.value = 'library';
  navigateToInstance(inst);
}

async function loadAccount() {
  try {
    const a = await invoke<{ name: string; account_type: string; uuid: string }>("get_current_account");
    userName.value = a.name;
    userType.value = a.account_type;
  } catch {}
}

onMounted(async () => {
  document.addEventListener("contextmenu", e => e.preventDefault());
  maxed.value = await app.isMaximized();
  listen("tauri://resize", async () => { maxed.value = await app.isMaximized(); });
  loadAccount();
  listen("account-refresh", loadAccount);
  window.addEventListener("account-changed", loadAccount);
  document.addEventListener("click", e => {
    if (!(e.target as HTMLElement).closest(".task-wrap")) taskOpen.value = false;
  });
});
</script>

<template>
  <OnboardingWindow v-if="isOobe" />
  <CrashShell v-else-if="isCrash" />
  <div v-else class="root">
    <header class="bar" :class="{ mac: isMac }" @mousedown="(e: MouseEvent) => { const t = e.target as HTMLElement; if (isLinux && !t.closest('button,input,select,a')) app.startDragging(); }">
      <div class="bar-left">
        <img class="barlogo" :src="logo" />
        <span class="bartitle" v-if="!isMac">Firefiles Launcher</span>
      </div>
      <div class="bar-center"></div>
      <div class="bar-right">
        <div class="task-wrap">
          <button class="taskbtn" @click.stop="taskOpen = !taskOpen">
            <Bolt :size="16" />
            <span class="tasklbl">{{ tasks.length > 0 ? tasks.length + ' 个任务进行中' : '还没有任务啊' }}</span>
          </button>
          <div v-if="taskOpen" class="taskdrop">
            <div class="tasktabs">
              <button class="tasktab" :class="{ on: taskTab === 'tasks' }" @click="taskTab = 'tasks'">下载任务</button>
              <button class="tasktab" :class="{ on: taskTab === 'running' }" @click="taskTab = 'running'">运行中</button>
            </div>
            <div v-if="taskTab === 'tasks'">
              <div v-if="!tasks.length" class="taskempty">暂无任务</div>
              <div v-for="t in tasks" :key="t.id" class="titem">
                <div class="tih">
                  <span class="titl">{{ t.title }}</span>
                  <span class="titype">{{ t.type === 'launch' ? '启动' : '安装' }}</span>
                </div>
                <span class="tlabel">{{ t.label }}</span>
                <div class="tibar"><div class="tifill" :style="{ width: (t.progress * 100) + '%' }"></div></div>
              </div>
            </div>
            <div v-if="taskTab === 'running'">
              <div v-if="!running.length" class="taskempty">没有运行中的游戏</div>
              <div v-for="t in running" :key="t.id" class="titem">
                <div class="tih"><span class="titl">{{ t.title }}</span></div>
                <div class="tiactions">
                  <button class="tiaction stop" @click="invoke('stop_game')"><Square :size="14" /></button>
                  <button class="tiaction" @click="goInst({ name: t.title, version: '', version_type: '' })"><ChevronRight :size="14" /></button>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div v-if="!isMac" class="winctrl">
          <button class="wbtn" @click="app.minimize()">
            <svg width="12" height="12" viewBox="0 0 12 12"><line x1="1.5" y1="6" x2="10.5" y2="6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
          </button>
          <button class="wbtn" @click="app.toggleMaximize().then(() => app.isMaximized().then(v => maxed = v))">
            <svg v-if="!maxed" width="12" height="12" viewBox="0 0 12 12"><rect x="1.5" y="1.5" width="9" height="9" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/></svg>
            <svg v-else width="12" height="12" viewBox="0 0 12 12"><rect x="2" y="4" width="7" height="7" rx="0.5" fill="none" stroke="currentColor" stroke-width="1"/><rect x="4" y="2" width="7" height="7" rx="0.5" fill="none" stroke="currentColor" stroke-width="1"/></svg>
          </button>
          <button class="wbtn close" @click="app.close()">
            <svg width="12" height="12" viewBox="0 0 12 12"><line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
          </button>
        </div>
      </div>
    </header>
    <main class="main">
      <HomePage v-show="nav === 'home'" />
      <LibraryPage v-show="nav === 'library'" @open-new-instance="showNewInst = true" />
      <ResourcesCenter v-show="nav === 'resourcescenter'" />
      <AccountInterface v-show="nav === 'account'" />
    </main>
    <NewMciRoot v-if="showNewInst" @close="showNewInst = false" @navigate="(id: string) => { showNewInst = false; nav = id }" />
    <div class="dock-wrap">
      <nav class="dock">
        <button v-if="userName" class="daccount" @click="onNav('account')">
          <img :src="avatar" class="davatar" />
          <div class="daccinfo">
            <span class="daccname">{{ userName }}</span>
            <span class="dacctype">{{ userLabel }}</span>
          </div>
        </button>
        <div class="dsep"></div>
        <button
          v-for="item in navItems.slice(0, 3)"
          :key="item.id"
          class="ditem"
          :class="{ on: nav === item.id && !(item.id === 'library' && showingInstance) }"
          @click="onNav(item.id)"
        >
          <component :is="item.icon" :size="21" />
        </button>
        <div class="dsep"></div>
        <button
          v-for="inst in openedInstances"
          :key="inst.name"
          class="ditem inst-icon"
          :class="{ on: nav === 'library' && currentInstanceName === inst.name }"
          @click="goInst(inst)"
        >
          <Gamepad2 :size="18" />
        </button>
        <button
          v-for="item in navItems.slice(3)"
          :key="item.id"
          class="ditem"
          :class="{ on: nav === item.id }"
          @click="onNav(item.id)"
        >
          <component :is="item.icon" :size="21" />
        </button>
      </nav>
      <button v-if="nav === 'library' && showingInstance" class="dlaunch" :class="{ running: dockTask?.status === 'running' }" @click="onDockLaunch">
        <LoaderCircle v-if="dockTask?.status === 'launching'" :size="18" class="spin" />
        <Square v-else-if="dockTask?.status === 'running'" :size="18" />
        <Play v-else :size="18" />
        <span>{{ dockTask?.status === 'running' ? '运行中' : dockTask?.status === 'launching' ? '启动中...' : '启动该实例' }}</span>
      </button>
    </div>
    <SettingsInterface v-if="showSettings" @close="showSettings = false" />
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
html { border-radius: 16px; overflow: hidden; }
@font-face {
  font-family: "HarmonyOS Sans";
  src: url("./assets/fonts/HarmonyOS_Sans_Regular.ttf") format("truetype");
  font-weight: 400;
  font-style: normal;
}
body {
  font-family: var(--app-font, "HarmonyOS Sans"), -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  overflow: hidden; height: 100vh; background: #1a1a1e;
}
.root {
  height: 100vh; display: flex; flex-direction: column; overflow: hidden; position: relative;
}

/* titlebar */
.bar {
  display: flex; align-items: center; height: 38px; flex-shrink: 0;
  background: transparent; -webkit-app-region: drag; user-select: none; z-index: 10;
}
.bar.mac { padding-left: 70px; }
.bar-left {
  display: flex; align-items: center; gap: 8px; padding-left: 14px;
  -webkit-app-region: no-drag;
}
.bar.mac .bar-left { padding-left: 0; }
.barlogo { height: 20px; width: auto; }
.bartitle { font-size: 13px; font-weight: 600; color: var(--title-color); opacity: 0.85; white-space: nowrap; }
.bar.mac .bartitle { display: none; }
.bar-center { flex: 1; }
.bar-right { display: flex; align-items: center; gap: 4px; -webkit-app-region: no-drag; }

/* background */
.root::before {
  content: '';
  position: fixed;
  inset: -3px;
  z-index: -1;
  background: url('./assets/imgs/background/default1.png') center / cover no-repeat;
  filter: blur(5px);
}

/* main */
.main {
  flex: 1; display: flex; overflow: hidden; margin: 0 0 0 12px; min-height: 0;
}

/* task btn */
.task-wrap { position: relative; display: flex; align-items: center; height: 100%; }
.taskbtn {
  display: flex; align-items: center; gap: 6px; padding: 0 10px; height: 32px;
  border: none; border-radius: 8px; background: rgba(128,128,128,0.08); color: var(--title-color);
  opacity: 0.7; cursor: pointer; transition: background 0.15s, opacity 0.15s; white-space: nowrap;
}
.taskbtn:hover { background: rgba(128,128,128,0.18); opacity: 1; }
.tasklbl { font-size: 12px; line-height: 1; }

/* task dropdown */
.taskdrop {
  position: absolute; top: calc(100% + 4px); right: 0; width: 300px; max-height: 360px;
  overflow-y: auto; background: var(--panel-bg); border: 1px solid rgba(128,128,128,0.15);
  border-radius: 10px; box-shadow: 0 8px 24px rgba(0,0,0,0.15); padding: 8px; z-index: 100;
}
.taskempty { text-align: center; color: var(--title-color); opacity: 0.4; font-size: 13px; padding: 24px 0; }
.tasktabs {
  display: flex; gap: 2px; margin-bottom: 8px; padding: 2px;
  background: rgba(128,128,128,0.08); border-radius: 8px;
}
.tasktab {
  flex: 1; display: flex; align-items: center; justify-content: center;
  padding: 6px 12px; border: none; border-radius: 6px; background: transparent;
  color: var(--title-color); opacity: 0.5; font-size: 12px; font-family: inherit;
  cursor: pointer; transition: background 0.15s, opacity 0.15s;
}
.tasktab:hover { opacity: 0.8; }
.tasktab.on { background: var(--panel-bg); opacity: 1; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
.titem {
  display: flex; flex-direction: column; gap: 4px; padding: 10px 12px;
  border-radius: 8px; transition: background 0.15s;
}
.titem:hover { background: rgba(128,128,128,0.06); }
.titem + .titem { margin-top: 4px; }
.tih { display: flex; align-items: center; justify-content: space-between; }
.titl { font-size: 13px; font-weight: 600; color: var(--title-color); line-height: 1.2; }
.titype { font-size: 11px; padding: 1px 6px; border-radius: 4px; background: rgba(128,128,128,0.1); color: var(--title-color); opacity: 0.55; }
.tlabel { font-size: 11px; color: var(--title-color); opacity: 0.5; line-height: 1; }
.tibar { width: 100%; height: 3px; border-radius: 2px; background: rgba(128,128,128,0.12); overflow: hidden; }
.tifill { height: 100%; border-radius: 2px; background: #0078d4; transition: width 0.3s ease; }
.tiactions { display: flex; align-items: center; gap: 4px; margin-top: 4px; }
.tiaction {
  display: flex; align-items: center; justify-content: center; width: 28px; height: 28px;
  border: none; border-radius: 6px; background: transparent; color: var(--title-color);
  opacity: 0.45; cursor: pointer; transition: background 0.15s, opacity 0.15s;
}
.tiaction:hover { background: rgba(128,128,128,0.12); opacity: 0.8; }
.tiaction.stop:hover { background: rgba(212,58,58,0.15); color: #d43a3a; opacity: 1; }

/* floating dock */
.dock-wrap {
  position: fixed;
  bottom: 14px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  z-index: 50;
}

.dock {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 18px;
  background: transparent;
}

.dlaunch {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 18px;
  height: 36px;
  border: none;
  border-radius: 10px;
  background: #0078d4;
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
  white-space: nowrap;
}

.dlaunch:hover {
  background: #0086e6;
}

.dlaunch.running {
  background: rgba(212,58,58,0.85);
}

.dlaunch.running:hover {
  background: #d43a3a;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.daccount {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 8px 4px 4px;
  border: none;
  border-radius: 10px;
  background: transparent;
  cursor: pointer;
  transition: background 0.15s;
}

.daccount:hover {
  background: var(--sidebar-hover);
}

.dsep {
  width: 1px;
  height: 28px;
  background: rgba(128,128,128,0.2);
  flex-shrink: 0;
}

.davatar {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  image-rendering: pixelated;
}

.daccinfo {
  display: flex;
  flex-direction: column;
  gap: 1px;
  text-align: left;
}

.daccname {
  font-size: 13px;
  font-weight: 600;
  color: var(--title-color);
  line-height: 1.2;
}

.dacctype {
  font-size: 10px;
  color: var(--title-color);
  opacity: 0.45;
  line-height: 1;
}

.ditem {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 10px;
  background: transparent;
  color: var(--title-color);
  opacity: 0.5;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
}

.ditem:hover {
  background: var(--sidebar-hover);
  opacity: 0.85;
}

.ditem.on {
  background: #0078d4;
  color: #fff;
  opacity: 1;
}



/* window controls */
.winctrl { display: flex; height: 100%; }
.wbtn {
  width: 46px; height: 100%; border: none; background: transparent;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; color: var(--title-color); transition: background 0.1s, box-shadow 0.1s;
  border-radius: 10px;
}
.wbtn:hover { background: rgba(0,0,0,0.08); box-shadow: 0 0 10px rgba(0,0,0,0.15); }
.wbtn.close:hover { background: #e81123; color: #fff; box-shadow: 0 0 10px rgba(232,17,35,0.5); }

/* scrollbar */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: rgba(128,128,128,0.3);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(128,128,128,0.5);
}

/* themes */
:root {
  --panel-bg: #ececec; --title-color: #1d1d1f; --sidebar-color: #1d1d1f;
  --sidebar-hover: rgba(0,0,0,0.08); --sidebar-active: #c7c7c7;
  --sidebar-active-color: #1d1d1f; --tooltip-bg: #1d1d1f; --tooltip-color: #f5f5f7;
  --content-bg: #f6f6f6; --settings-icon-bg: #0078d4; --window-border: rgba(0,0,0,0.12);
  --content-border: rgba(0,0,0,0.25); --dock-border: #c7c7c7;
}
@media (prefers-color-scheme: dark) {
  :root {
    --panel-bg: #2d2d2d; --title-color: #f5f5f7; --sidebar-color: #e0e0e0;
    --sidebar-hover: rgba(255,255,255,0.08); --sidebar-active: #4a4a4a;
    --sidebar-active-color: #f5f5f7; --tooltip-bg: #e0e0e0; --tooltip-color: #1d1d1f;
    --content-bg: #1c1c1e; --settings-icon-bg: #60a5fa; --window-border: rgba(255,255,255,0.12);
    --content-border: rgba(255,255,255,0.2); --dock-border: #555555;
  }
  .wbtn:hover { background: rgba(255,255,255,0.1); box-shadow: 0 0 10px rgba(0,0,0,0.3); }
  .wbtn.close:hover { background: #e81123; color: #fff; box-shadow: 0 0 10px rgba(232,17,35,0.5); }
}
html[data-theme="light"] {
  --panel-bg: #ececec; --title-color: #1d1d1f; --sidebar-color: #1d1d1f;
  --sidebar-hover: rgba(0,0,0,0.08); --sidebar-active: #c7c7c7;
  --sidebar-active-color: #1d1d1f; --tooltip-bg: #1d1d1f; --tooltip-color: #f5f5f7;
  --content-bg: #f6f6f6; --settings-icon-bg: #0078d4; --window-border: rgba(0,0,0,0.12);
  --content-border: rgba(0,0,0,0.25); --dock-border: #c7c7c7;
}
html[data-theme="dark"] {
  --panel-bg: #2d2d2d; --title-color: #f5f5f7; --sidebar-color: #e0e0e0;
  --sidebar-hover: rgba(255,255,255,0.08); --sidebar-active: #4a4a4a;
  --sidebar-active-color: #f5f5f7; --tooltip-bg: #e0e0e0; --tooltip-color: #1d1d1f;
  --content-bg: #1c1c1e; --settings-icon-bg: #60a5fa; --window-border: rgba(255,255,255,0.12);
  --content-border: rgba(255,255,255,0.2); --dock-border: #555555;
}
html[data-theme="dark"] .wbtn:hover { background: rgba(255,255,255,0.1); box-shadow: 0 0 10px rgba(0,0,0,0.3); }
html[data-theme="dark"] .wbtn.close:hover { background: #e81123; color: #fff; box-shadow: 0 0 10px rgba(232,17,35,0.5); }
</style>
