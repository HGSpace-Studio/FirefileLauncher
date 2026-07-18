<script setup lang="ts">
import { ref, computed, provide, onMounted, watch } from "vue";
import { useTaskStore } from "./stores/taskStore";
import { navigateToInstance } from "./stores/navigation";
import { currentInstanceName, currentLaunchFn, currentStopFn } from "./stores/instanceLaunch";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Icon as VIcon } from "@vicons/utils";
import { Home24Regular, StoreMicrosoft24Regular, Grid24Regular, Add24Regular, Settings24Regular, Flash24Regular, Square24Regular, ChevronRight24Regular, Play24Regular, ArrowClockwise24Regular, Games24Regular, MoreHorizontal24Regular } from "@vicons/fluent";
import logo from "./assets/logos/logo.png";
import steve from "./assets/imgs/skins/avator/steve.png";
import alex from "./assets/imgs/skins/avator/alex.png";
import default1Bg from "./assets/imgs/background/default1.png";

import Sidebar from "./components/Sidebar.vue";
import NewMciRoot from "./components/view/new_mci/root_interface.vue";
import HomePage from "./components/view/HomePage.vue";
import LibraryPage from "./components/view/rootpages/versionroot.vue";
import ResourcesCenter from "./components/view/ResourcesCenter.vue";
import AccountInterface from "./components/accinterface.vue";
import SettingsInterface from "./components/settings_interface.vue";
import InstanceSettingsInterface from "./components/InstanceSettingsInterface.vue";
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
const showInstanceSettings = ref(false);
const showAccount = ref(false);
const showNewInst = ref(false);
const showingInstance = ref(false);
provide('showingInstance', showingInstance);
const goBackLib = ref(0);
provide('goBackLib', goBackLib);
const taskOpen = ref(false);
const isFullscreenUI = ref(localStorage.getItem("firefile-ui-layout") !== "sidebar");
watch(showSettings, (val: boolean) => {
  if (!val) isFullscreenUI.value = localStorage.getItem("firefile-ui-layout") !== "sidebar";
});
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
  { id: "home", icon: Home24Regular },
  { id: "resourcescenter", icon: StoreMicrosoft24Regular },
  { id: "library", icon: Grid24Regular },
  { id: "add-instance", icon: Add24Regular },
  { id: "settings", icon: Settings24Regular },
];
const taskTab = ref<"tasks" | "running">("tasks");
const { tasks } = useTaskStore();
const running = computed(() => tasks.value.filter(t => t.status === "running"));
const filteredTasks = computed(() => tasks.value.filter(t => t.status !== "running" && t.status !== "exited"));

interface InstanceEntry {
  name: string;
  version: string;
  version_type: string;
  loader: { type: string; version: string } | null;
  icon: string | null;
  installed: boolean;
}

const instances = ref<InstanceEntry[]>([]);
async function loadInstances() {
  try {
    instances.value = await invoke<InstanceEntry[]>("get_instances_list");
  } catch {
    instances.value = [];
  }
}

const MAX_VISIBLE_INSTANCES = 3;
const visibleInstances = computed(() => instances.value.slice(0, MAX_VISIBLE_INSTANCES));
const overflowInstances = computed(() => instances.value.slice(MAX_VISIBLE_INSTANCES));
const instMenuOpen = ref(false);
const instMoreRef = ref<HTMLElement | null>(null);
function closeInstMenu(e: MouseEvent) {
  if (instMoreRef.value && !instMoreRef.value.contains(e.target as Node)) {
    instMenuOpen.value = false;
  }
}
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

interface InstanceStats {
  lastPlayDuration: number
  lastPlayTime: string
  totalPlayTime: number
  currentSessionStart: number | null
}
const STORAGE_KEY = 'firefile-instance-stats'
const instanceStats = ref<Record<string, InstanceStats>>({})
function loadStats() {
  try { instanceStats.value = JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}') } catch {}
}
function saveStats() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(instanceStats.value))
}
function ensureStats(name: string): InstanceStats {
  if (!instanceStats.value[name]) {
    instanceStats.value[name] = { lastPlayDuration: 0, lastPlayTime: '', totalPlayTime: 0, currentSessionStart: null }
  }
  return instanceStats.value[name]
}
function fmtDuration(sec: number): string {
  if (sec <= 0) return '0s'
  const h = Math.floor(sec / 3600)
  const m = Math.floor((sec % 3600) / 60)
  const s = sec % 60
  let r = ''
  if (h > 0) r += h + 'h '
  if (m > 0 || h > 0) r += m + 'm '
  r += s + 's'
  return r.trim()
}
function fmtDate(d: Date): string {
  const y = d.getFullYear()
  const mo = String(d.getMonth() + 1).padStart(2, '0')
  const da = String(d.getDate()).padStart(2, '0')
  const h = String(d.getHours()).padStart(2, '0')
  const mi = String(d.getMinutes()).padStart(2, '0')
  return `${y}/${mo}/${da} ${h}:${mi}`
}
const liveTimer = ref(0)
watch(dockTask, (n, o) => {
  const name = currentInstanceName.value
  if (!name) return
  const stats = ensureStats(name)
  if (n?.status === 'launching' && (!o || o.status === 'idle' || o.status === 'exited' || o.status === 'error')) {
    stats.currentSessionStart = Date.now()
    saveStats()
  }
  if (o && (o.status === 'running' || o.status === 'launching') && n && (n.status === 'exited' || n.status === 'error')) {
    if (stats.currentSessionStart) {
      const dur = Math.floor((Date.now() - stats.currentSessionStart) / 1000)
      stats.lastPlayDuration = dur
      stats.lastPlayTime = fmtDate(new Date())
      stats.totalPlayTime += dur
      stats.currentSessionStart = null
      saveStats()
    }
  }
})
watch(currentInstanceName, () => { liveTimer.value = Date.now() })

const currentStats = computed(() => {
  const name = currentInstanceName.value
  if (!name) return null
  return ensureStats(name)
})
const displayLastDuration = computed(() => {
  const s = currentStats.value
  if (!s) return '0s'
  if (s.currentSessionStart) {
    return '正在游玩 ' + fmtDuration(Math.floor((Date.now() - s.currentSessionStart) / 1000))
  }
  return fmtDuration(s.lastPlayDuration)
})
const displayLastTime = computed(() => {
  const s = currentStats.value
  if (!s) return '--'
  if (s.currentSessionStart) return '正在游玩'
  return s.lastPlayTime || '--'
})
const displayTotalTime = computed(() => {
  const s = currentStats.value
  if (!s) return '0s'
  return fmtDuration(s.totalPlayTime)
})

const currentInstForSettings = computed(() => {
  const inst = instances.value.find(i => i.name === currentInstanceName.value)
  if (!inst) return null
  return {
    name: inst.name,
    version: inst.version,
    versionType: inst.version_type,
    loader: inst.loader ? { type: inst.loader.type as "fabric" | "forge" | "neoforge" | "quilt", version: inst.loader.version } : undefined,
    icon: inst.icon || undefined,
  }
})

function onNav(id: string) {
  if (id === "settings") { showSettings.value = true; return; }
  if (id === "account") { showAccount.value = true; return; }
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
  const savedUrl = localStorage.getItem("firefile-bg-url");
  if (savedUrl) {
    document.documentElement.style.setProperty("--bg-image", savedUrl);
  } else {
    document.documentElement.style.setProperty("--bg-image", `url("${default1Bg}")`);
  }
  const savedBlur = Number(localStorage.getItem("firefile-bg-blur")) || 5;
  document.documentElement.style.setProperty("--bg-blur", savedBlur + "px");
  document.addEventListener("contextmenu", e => e.preventDefault());
  maxed.value = await app.isMaximized();
  listen("tauri://resize", async () => { maxed.value = await app.isMaximized(); });
  loadAccount();
  loadInstances();
  loadStats();
  window.setInterval(() => { liveTimer.value = Date.now() }, 1000);
  listen("account-refresh", loadAccount);
  window.addEventListener("account-changed", loadAccount);
  window.addEventListener("instance-installed", loadInstances);
  document.addEventListener("click", e => {
    if (!(e.target as HTMLElement).closest(".task-wrap")) taskOpen.value = false;
  });
  document.addEventListener("click", closeInstMenu);
});
</script>

<template>
  <OnboardingWindow v-if="isOobe" />
  <CrashShell v-else-if="isCrash" />
  <div v-else class="root">
    <header class="bar" :class="{ mac: isMac, 'bar-bordered': !isFullscreenUI }" @mousedown="(e: MouseEvent) => { const t = e.target as HTMLElement; if (isLinux && !t.closest('button,input,select,a')) app.startDragging(); }">
      <div class="bar-left">
        <img class="barlogo" :src="logo" />
        <span class="bartitle" v-if="!isMac">Firefiles Launcher</span>
        <button v-if="!isFullscreenUI" class="bar-account" @click="onNav('account')">
          <img :src="avatar" class="bar-avatar" />
          <div class="bar-accinfo">
            <span class="bar-accname">{{ userName || '未设置' }}</span>
            <span class="bar-acctype">{{ userLabel }}</span>
          </div>
        </button>
      </div>
      <div class="bar-center"></div>
      <div class="bar-right">
        <div class="task-wrap">
          <button class="taskbtn" @click.stop="taskOpen = !taskOpen">
            <VIcon :size="16"><Flash24Regular /></VIcon>
            <span class="tasklbl">{{ tasks.length > 0 ? tasks.length + ' 个任务进行中' : '还没有任务啊' }}</span>
          </button>
          <div v-if="taskOpen" class="taskdrop">
            <div class="tasktabs">
              <button class="tasktab" :class="{ on: taskTab === 'tasks' }" @click="taskTab = 'tasks'">下载任务</button>
              <button class="tasktab" :class="{ on: taskTab === 'running' }" @click="taskTab = 'running'">运行中</button>
            </div>
            <div v-if="taskTab === 'tasks'">
              <div v-if="!filteredTasks.length" class="taskempty">暂无任务</div>
              <div v-for="t in filteredTasks" :key="t.id" class="titem">
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
              <div v-for="t in running" :key="t.id" class="titem ritem">
                <div class="tih">
                  <span class="titl">{{ t.title }}</span>
                  <div class="tiactions">
                    <button class="tiaction stop" @click="invoke('stop_game')"><VIcon :size="14"><Square24Regular /></VIcon></button>
                    <button class="tiaction" @click="goInst({ name: t.title, version: '', version_type: '' })"><VIcon :size="14"><ChevronRight24Regular /></VIcon></button>
                  </div>
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
    <div class="body-area" :class="{ 'with-dock': isFullscreenUI }">
      <Sidebar
        v-if="!isFullscreenUI"
        :nav-items="navItems"
        :visible-instances="visibleInstances"
        :overflow-instances="overflowInstances"
        :active-nav="nav"
        :showing-instance="showingInstance"
        :current-instance-name="currentInstanceName"
        @navigate="onNav"
        @go-inst="goInst"
        @toggle-inst-menu="instMenuOpen = !instMenuOpen"
      />
      <main class="main" :class="{ 'main-bordered': !isFullscreenUI }" :style="{ '--content-bottom-pad': isFullscreenUI ? '80px' : '12px' }">
        <HomePage v-show="nav === 'home'" />
        <LibraryPage v-show="nav === 'library'" @open-new-instance="showNewInst = true" />
        <ResourcesCenter v-show="nav === 'resourcescenter'" />
      </main>
    </div>
    <AccountInterface v-if="showAccount" @close="showAccount = false" />
    <NewMciRoot v-if="showNewInst" @close="showNewInst = false" @navigate="(id: string) => { showNewInst = false; nav = id }" />
    <template v-if="isFullscreenUI">
      <div class="dock-wrap">
        <nav class="dock">
          <button class="daccount" @click="onNav('account')">
            <img :src="avatar" class="davatar" />
            <div class="daccinfo">
              <span class="daccname">{{ userName || '未设置' }}</span>
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
            <VIcon :size="21"><component :is="item.icon" /></VIcon>
          </button>
          <div class="dsep"></div>
          <button
            v-for="inst in visibleInstances"
            :key="inst.name"
            class="ditem inst-icon"
            :class="{ on: nav === 'library' && currentInstanceName === inst.name }"
            @click="goInst(inst)"
          >
            <VIcon :size="18"><Games24Regular /></VIcon>
          </button>
          <button
            v-if="overflowInstances.length > 0"
            ref="instMoreRef"
            class="ditem inst-more"
            :class="{ on: instMenuOpen }"
            @click.stop="instMenuOpen = !instMenuOpen"
          >
            <VIcon :size="18"><MoreHorizontal24Regular /></VIcon>
          </button>
          <Teleport to="body">
            <div v-if="instMenuOpen" class="inst-menu" @click.stop>
              <button
                v-for="inst in overflowInstances"
                :key="inst.name"
                class="inst-menu-item"
                :class="{ on: currentInstanceName === inst.name }"
                @click="goInst(inst); instMenuOpen = false"
              >
                <VIcon :size="16"><Games24Regular /></VIcon>
                <span>{{ inst.name }}</span>
              </button>
            </div>
          </Teleport>
          <button
            v-for="item in navItems.slice(3)"
            :key="item.id"
            class="ditem"
            :class="{ on: nav === item.id }"
            @click="onNav(item.id)"
          >
            <VIcon :size="21"><component :is="item.icon" /></VIcon>
          </button>
        </nav>
        <button v-if="nav === 'library' && showingInstance" class="dlaunch" :class="{ running: dockTask?.status === 'running' }" @click="onDockLaunch">
          <VIcon v-if="dockTask?.status === 'launching'" :size="18"><ArrowClockwise24Regular class="spin" /></VIcon>
          <VIcon v-else-if="dockTask?.status === 'running'" :size="18"><Square24Regular /></VIcon>
          <VIcon v-else :size="18"><Play24Regular /></VIcon>
          <span>{{ dockTask?.status === 'running' ? '运行中' : dockTask?.status === 'launching' ? '启动中...' : '启动该实例' }}</span>
        </button>
      </div>
    </template>
    <template v-if="!isFullscreenUI && nav === 'library' && showingInstance">
      <div class="saction-wrap">
        <button class="slaunch" :class="{ running: dockTask?.status === 'running' }" @click="onDockLaunch">
          <VIcon v-if="dockTask?.status === 'launching'" :size="18"><ArrowClockwise24Regular class="spin" /></VIcon>
          <VIcon v-else-if="dockTask?.status === 'running'" :size="18"><Square24Regular /></VIcon>
          <VIcon v-else :size="18"><Play24Regular /></VIcon>
          <span>{{ dockTask?.status === 'running' ? '运行中' : dockTask?.status === 'launching' ? '启动中...' : '启动该实例' }}</span>
        </button>
        <button class="ssettings" @click="showInstanceSettings = true">
          <VIcon :size="18"><Settings24Regular /></VIcon>
        </button>
        <div class="sdivider"></div>
        <div class="sinfo">
          <div class="sinfo-item">
            <span class="sinfo-label">上次游玩时长</span>
            <span class="sinfo-value">{{ displayLastDuration }}</span>
          </div>
          <div class="sinfo-sep"></div>
          <div class="sinfo-item">
            <span class="sinfo-label">最后游玩时间</span>
            <span class="sinfo-value">{{ displayLastTime }}</span>
          </div>
          <div class="sinfo-sep"></div>
          <div class="sinfo-item">
            <span class="sinfo-label">总游玩时间</span>
            <span class="sinfo-value">{{ displayTotalTime }}</span>
          </div>
        </div>
      </div>
    </template>
    <Teleport to="body">
      <div v-if="!isFullscreenUI && instMenuOpen" class="inst-menu sidebar-inst-menu" @click.stop>
        <button
          v-for="inst in overflowInstances"
          :key="inst.name"
          class="inst-menu-item"
          :class="{ on: currentInstanceName === inst.name }"
          @click="goInst(inst); instMenuOpen = false"
        >
          <VIcon :size="16"><Games24Regular /></VIcon>
          <span>{{ inst.name }}</span>
        </button>
      </div>
    </Teleport>
    <SettingsInterface v-if="showSettings" @close="showSettings = false" />
    <InstanceSettingsInterface
      v-if="showInstanceSettings && currentInstForSettings"
      :instance="currentInstForSettings"
      @close="showInstanceSettings = false"
    />
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
  border-radius: 16px;
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
.barlogo { height: 24px; width: auto; }
.bar.mac .barlogo { margin-left: 8px; }
.bartitle { font-size: 13px; font-weight: 600; color: var(--title-color); opacity: 0.85; white-space: nowrap; }
.bar.mac .bartitle { display: none; }
.bar-account {
  display: flex; align-items: center; gap: 8px; padding: 2px 8px 2px 4px;
  border: none; border-radius: 8px; background: transparent; cursor: pointer;
  color: var(--title-color); transition: background 0.15s;
}
.bar-account:hover { background: var(--sidebar-hover); }
.bar-avatar { width: 24px; height: 24px; border-radius: 6px; image-rendering: pixelated; }
.bar-accinfo { display: flex; flex-direction: column; gap: 1px; text-align: left; }
.bar-accname { font-size: 12px; font-weight: 600; line-height: 1.2; }
.bar-acctype { font-size: 10px; opacity: 0.5; line-height: 1; }
.bar-center { flex: 1; }
.bar-right { display: flex; align-items: center; gap: 4px; -webkit-app-region: no-drag; }
.bar-bordered {
}

/* background */
.root::before {
  content: '';
  position: absolute;
  inset: -20px;
  z-index: -2;
  background-image: var(--bg-image, none);
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  filter: blur(var(--bg-blur, 5px));
}
.root::after {
  content: '';
  position: absolute;
  inset: 0;
  z-index: -1;
  background: var(--blur-overlay);
  border-radius: 16px;
}

/* body area */
.body-area {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}
.body-area.with-dock {
  padding: 0 0 0 12px;
}

/* main */
.main {
  flex: 1; display: flex; overflow: hidden; min-height: 0;
}
.main-bordered {
  border-top: 1px solid var(--content-border);
  border-left: 1px solid var(--content-border);
  border-radius: 9px 0 0 0;
  box-shadow: inset 1px 1px 4px rgba(0,0,0,0.26);
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
.ritem .tiactions { margin-top: 0; }
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
  background: #00BAAD;
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
  white-space: nowrap;
}

.dlaunch:hover {
  background: #00CFC0;
}

.dlaunch.running {
  background: rgba(212,58,58,0.85);
}

.dlaunch.running:hover {
  background: #d43a3a;
}

.saction-wrap {
  position: fixed;
  top: 50%;
  left: 78px;
  transform: translateY(calc(-50% - 60px));
  display: flex;
  align-items: center;
  gap: 4px;
  z-index: 50;
}

.slaunch {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 18px;
  height: 36px;
  border: none;
  border-radius: 10px;
  background: #00BAAD;
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
  white-space: nowrap;
}

.ssettings {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 10px;
  background: transparent;
  color: var(--title-color);
  opacity: 0.5;
  cursor: pointer;
  transition: opacity 0.15s, background 0.15s;
}

.ssettings:hover {
  background: rgba(128, 128, 128, 0.12);
}

.sdivider {
  width: 1px;
  height: 24px;
  background: rgba(128, 128, 128, 0.2);
  flex-shrink: 0;
  margin: 0 2px;
}

.sinfo {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-left: 12px;
}

.sinfo-item {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.sinfo-label {
  font-size: 10px;
  color: var(--title-color);
  opacity: 0.4;
  line-height: 1;
  white-space: nowrap;
}

.sinfo-value {
  font-size: 16px;
  font-weight: 600;
  color: var(--title-color);
  line-height: 1.2;
  white-space: nowrap;
}

.sinfo-sep {
  width: 1px;
  height: 24px;
  background: rgba(128, 128, 128, 0.15);
  flex-shrink: 0;
}

.slaunch:hover {
  background: #00CFC0;
}
.slaunch.running {
  background: rgba(212,58,58,0.85);
}
.slaunch.running:hover {
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

.inst-more.on {
  background: var(--sidebar-active);
  opacity: 1;
}

.inst-menu {
  position: fixed;
  bottom: 70px;
  left: 50%;
  transform: translateX(-50%);
  min-width: 200px;
  max-height: 280px;
  overflow-y: auto;
  background: var(--panel-bg);
  border: 1px solid rgba(128,128,128,0.15);
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0,0,0,0.15);
  padding: 6px;
  z-index: 200;
}

.inst-menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--title-color);
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s;
  text-align: left;
}

.inst-menu-item:hover {
  background: var(--sidebar-hover);
}

.inst-menu-item.on {
  background: #0078d4;
  color: #fff;
}

.sidebar-inst-menu {
  left: 190px !important;
  bottom: 70px !important;
  transform: none !important;
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
  --blur-overlay: rgba(236, 236, 236, 0.25);
}
@media (prefers-color-scheme: dark) {
  :root {
    --panel-bg: #2d2d2d; --title-color: #f5f5f7; --sidebar-color: #e0e0e0;
    --sidebar-hover: rgba(255,255,255,0.08); --sidebar-active: #4a4a4a;
    --sidebar-active-color: #f5f5f7; --tooltip-bg: #e0e0e0; --tooltip-color: #1d1d1f;
    --content-bg: #1c1c1e; --settings-icon-bg: #60a5fa; --window-border: rgba(255,255,255,0.12);
    --content-border: rgba(255,255,255,0.2); --dock-border: #555555;
    --blur-overlay: rgba(20, 20, 22, 0.45);
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
  --blur-overlay: rgba(236, 236, 236, 0.25);
}
html[data-theme="dark"] {
  --panel-bg: #2d2d2d; --title-color: #f5f5f7; --sidebar-color: #e0e0e0;
  --sidebar-hover: rgba(255,255,255,0.08); --sidebar-active: #4a4a4a;
  --sidebar-active-color: #f5f5f7; --tooltip-bg: #e0e0e0; --tooltip-color: #1d1d1f;
  --content-bg: #1c1c1e; --settings-icon-bg: #60a5fa; --window-border: rgba(255,255,255,0.12);
  --content-border: rgba(255,255,255,0.2); --dock-border: #555555;
  --blur-overlay: rgba(20, 20, 22, 0.45);
}
html[data-theme="dark"] .wbtn:hover { background: rgba(255,255,255,0.1); box-shadow: 0 0 10px rgba(0,0,0,0.3); }
html[data-theme="dark"] .wbtn.close:hover { background: #e81123; color: #fff; box-shadow: 0 0 10px rgba(232,17,35,0.5); }
</style>
