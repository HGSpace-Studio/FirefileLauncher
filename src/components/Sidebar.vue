<script setup lang="ts">
import { Icon as VIcon } from "@vicons/utils";
import { Games24Regular, MoreHorizontal24Regular } from "@vicons/fluent";

defineProps<{
  navItems: { id: string; icon: any }[];
  visibleInstances: { name: string }[];
  overflowInstances: { name: string }[];
  activeNav: string;
  showingInstance: boolean;
  currentInstanceName: string | null;
}>();

const emit = defineEmits<{
  (e: "navigate", id: string): void;
  (e: "go-inst", inst: any): void;
  (e: "toggle-inst-menu"): void;
}>();
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-inner">
      <button
        v-for="item in navItems.slice(0, 3)"
        :key="item.id"
        class="sidebar-item"
        :class="{ active: activeNav === item.id && !(item.id === 'library' && showingInstance) }"
        @click="emit('navigate', item.id)"
      >
        <VIcon :size="23"><component :is="item.icon" /></VIcon>
        <span class="nav-tooltip">{{ item.id === 'home' ? '首页' : item.id === 'resourcescenter' ? '资源中心' : item.id === 'library' ? '库' : '' }}</span>
      </button>
      <div class="sidebar-divider"></div>
      <button
        v-for="inst in visibleInstances"
        :key="inst.name"
        class="sidebar-item"
        :class="{ active: activeNav === 'library' && currentInstanceName === inst.name }"
        @click="emit('go-inst', inst)"
      >
        <VIcon :size="21"><Games24Regular /></VIcon>
        <span class="nav-tooltip">{{ inst.name }}</span>
      </button>
      <button
        v-if="overflowInstances.length > 0"
        class="sidebar-item"
        @click="emit('toggle-inst-menu')"
      >
        <VIcon :size="21"><MoreHorizontal24Regular /></VIcon>
        <span class="nav-tooltip">更多实例...</span>
      </button>
      <button
        v-for="item in [navItems[3]]"
        :key="item.id"
        class="sidebar-item"
        :class="{ active: activeNav === item.id }"
        @click="emit('navigate', item.id)"
      >
        <VIcon :size="23"><component :is="item.icon" /></VIcon>
        <span class="nav-tooltip">{{ item.id === 'add-instance' ? '创建实例' : '' }}</span>
      </button>
      <div class="sidebar-spacer"></div>
      <button
        v-for="item in navItems.slice(4)"
        :key="item.id"
        class="sidebar-item"
        :class="{ active: activeNav === item.id }"
        @click="emit('navigate', item.id)"
      >
        <VIcon :size="23"><component :is="item.icon" /></VIcon>
        <span class="nav-tooltip">{{ item.id === 'settings' ? '设置' : '' }}</span>
      </button>

    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 50px;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  background: transparent;
  overflow: visible;
}
.sidebar-inner {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 8px 6px;
  gap: 6px;
}
.sidebar-item {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  margin: 0 auto;
  border: none;
  border-radius: 50%;
  background: transparent;
  cursor: pointer;
  color: var(--sidebar-color);
  transition: background 0.15s;
  position: relative;
}
.sidebar-item:hover {
  background: var(--sidebar-hover);
}
.sidebar-item.active {
  background: #00BAAD;
  color: #fff;
}
.nav-tooltip {
  position: absolute;
  left: calc(100% + 8px);
  top: 50%;
  transform: translateY(-50%);
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  white-space: nowrap;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.15s ease;
  z-index: 100;
  background: var(--tooltip-bg);
  color: var(--tooltip-color);
}
.sidebar-item:hover .nav-tooltip {
  opacity: 1;
}
.sidebar-divider {
  height: 1px;
  background: var(--content-border);
  margin: 4px 8px;
}
.section-label {
  font-size: 11px;
  color: var(--sidebar-color);
  opacity: 0.4;
  padding: 4px 10px 2px;
  user-select: none;
  text-align: center;
}
.sidebar-spacer {
  flex: 1;
}
</style>
