<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getSystemLocale } from "./i18n";
import Sidebar from "./components/Sidebar.vue";
import { Home, Settings, Library } from "@lucide/vue";
import logo from "./assets/logos/logo.png";

const { locale } = useI18n();

const currentLang = ref("system");

watch(currentLang, (val) => {
  locale.value = val === "system" ? getSystemLocale() : val;
});

const isMac = ref(false);
const isLinux = ref(false);
const isMaximized = ref(false);
let appWindow: any = null;

const navItems = [
  { id: "home", label: "首页", icon: Home },
  { id: "library", label: "库", icon: Library },
  { id: "settings", label: "设置", icon: Settings },
];

const activeNav = ref("home");

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
    <span class="title">
      <select class="lang-select" v-model="currentLang">
        <option value="system">{{ $t("app.lang.system") }}</option>
        <option value="zh_cn">{{ $t("app.lang.zh_cn") }}</option>
        <option value="en_us">{{ $t("app.lang.en_us") }}</option>
      </select>
    </span>
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
    <Sidebar :navItems="navItems" v-model:activeId="activeNav" />
    <main class="content">
      <p>{{ activeNav }}</p>
    </main>
  </div>
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
  font-family: "HarmonyOS Sans", -apple-system, BlinkMacSystemFont, "Segoe UI",
    Roboto, Helvetica, Arial, sans-serif;
  overflow: hidden;
  height: 100vh;
}

.titlebar {
  display: flex;
  align-items: center;
  height: 38px;
  user-select: none;
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
  padding-right: 16px;
}

.title {
  flex: 1;
  text-align: center;
  font-size: 13px;
  font-weight: 600;
  opacity: 0.85;
  padding-right: 70px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.lang-select {
  appearance: none;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 12px;
  font-family: inherit;
  color: inherit;
  cursor: pointer;
  outline: none;
  transition: border-color 0.15s;
}

.lang-select:hover {
  border-color: rgba(128, 128, 128, 0.3);
}

.lang-select:focus {
  border-color: rgba(128, 128, 128, 0.5);
}

.lang-select option {
  background: var(--panel-bg);
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
  color: inherit;
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
}

.content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.content p {
  font-size: 14px;
  opacity: 0.6;
}

@media (prefers-color-scheme: light) {
  :root {
    --panel-bg: #ececec;
    --title-color: #1d1d1f;
  }
  body {
    background: #ececec;
  }
  .titlebar {
    background: var(--panel-bg);
  }
  .title {
    color: var(--title-color);
  }
  .viewport {
    background: #f6f6f6;
  }
}

@media (prefers-color-scheme: dark) {
  :root {
    --panel-bg: #2d2d2d;
    --title-color: #f5f5f7;
  }
  body {
    background: #2d2d2d;
  }
  .titlebar {
    background: var(--panel-bg);
  }
  .title {
    color: var(--title-color);
  }
  .viewport {
    background: #1c1c1e;
  }
  .win-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .win-btn.close:hover {
    background: #e81123;
    color: #fff;
  }
}
</style>
