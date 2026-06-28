<script setup lang="ts">
import { onMounted, ref } from "vue";

const isMac = ref(false);
const isMaximized = ref(false);
let appWindow: any = null;

onMounted(async () => {
  document.addEventListener("contextmenu", (e) => e.preventDefault());

  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const { listen } = await import("@tauri-apps/api/event");
  appWindow = getCurrentWindow();
  isMac.value = navigator.userAgent.toLowerCase().includes("mac");

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
  <div class="titlebar" :class="{ 'is-win': !isMac }" data-tauri-drag-region>
    <div v-if="isMac" class="traffic-light-area"></div>
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

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica,
    Arial, sans-serif;
  overflow: hidden;
  height: 100vh;
}

.titlebar {
  display: flex;
  align-items: center;
  height: 38px;
  user-select: none;
}

.titlebar.is-win {
  padding: 0 16px;
}

.traffic-light-area {
  width: 70px;
  min-width: 70px;
  height: 100%;
}

.title {
  flex: 1;
  text-align: center;
  font-size: 13px;
  font-weight: 600;
  opacity: 0.85;
  padding-right: 70px;
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
  align-items: center;
  justify-content: center;
}

.viewport p {
  font-size: 14px;
  opacity: 0.6;
}

@media (prefers-color-scheme: light) {
  body {
    background: #ececec;
  }
  .titlebar {
    background: #ececec;
  }
  .title {
    color: #1d1d1f;
  }
  .viewport {
    background: #f6f6f6;
  }
}

@media (prefers-color-scheme: dark) {
  body {
    background: #2d2d2d;
  }
  .titlebar {
    background: #2d2d2d;
  }
  .title {
    color: #f5f5f7;
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
