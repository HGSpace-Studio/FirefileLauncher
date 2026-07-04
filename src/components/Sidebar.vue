<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { ListSortDescending, ChevronRight } from "@lucide/vue";
import steveAvatar from "../assets/imgs/skins/avator/steve.png";
import alexAvatar from "../assets/imgs/skins/avator/alex.png";

const { t } = useI18n();

const props = defineProps<{
  mainItems: { id: string; label: string; icon: any; highlight?: boolean }[];
  footerItem?: { id: string; label: string; icon: any };
  activeId?: string;
  userName?: string;
  userType?: string;
  userAvatar?: string;
}>();

const emit = defineEmits<{
  (e: "update:activeId", id: string): void;
}>();

const expanded = ref(false);
const avatars = [steveAvatar, alexAvatar];

function toggle() {
  expanded.value = !expanded.value;
}

function getAvatar(): string {
  if (props.userAvatar) return props.userAvatar;
  const hash = props.userName ? props.userName.charCodeAt(0) % avatars.length : 0;
  return avatars[hash];
}

const translatedMainItems = computed(() =>
  props.mainItems.map((item) => ({
    ...item,
    label: t(`app.mainwindow.sidebar.${item.id}`),
  }))
);

const translatedFooterItem = computed(() =>
  props.footerItem
    ? {
        ...props.footerItem,
        label: t(`app.mainwindow.sidebar.${props.footerItem.id}`),
      }
    : null
);

const userTypeLabel = computed(() => {
  if (!props.userType) return "";
  const map: Record<string, string> = {
    offline: "离线账号",
    microsoft: "微软账户",
  };
  return map[props.userType] || props.userType;
});
</script>

<template>
  <aside class="sidebar" :class="{ expanded }">
    <div class="sidebar-inner">
      <button class="toggle-btn" @click="toggle" :title="expanded ? t('app.mainwindow.sidebar.collapse') : t('app.mainwindow.sidebar.expand')">
        <span class="nav-icon">
          <ListSortDescending :size="18" />
        </span>
      </button>

      <button
        v-if="userName"
        class="user-btn"
        :class="{ active: activeId === 'account' }"
        @click="emit('update:activeId', 'account')"
      >
        <img :src="getAvatar()" class="user-avatar" />
        <Transition name="fade">
          <div v-if="expanded" class="user-info">
            <span class="user-name">{{ userName }}</span>
            <span class="user-source">{{ userTypeLabel }}</span>
          </div>
        </Transition>
        <ChevronRight v-if="expanded" :size="14" class="user-arrow" />
      </button>

      <nav class="nav">
        <button
          v-for="item in translatedMainItems"
          :key="item.id"
          class="nav-item"
          :class="{ active: activeId === item.id, highlight: item.highlight }"
          @click="emit('update:activeId', item.id)"
        >
          <span class="nav-icon">
            <component :is="item.icon" :size="20" />
          </span>
          <span class="nav-label">{{ item.label }}</span>
          <span class="nav-tooltip">{{ item.label }}</span>
        </button>
        <div class="nav-divider"></div>
      </nav>

      <span class="section-label">{{ t("app.mainwindow.sidebar.pinned") }}</span>

      <div class="nav-footer" v-if="translatedFooterItem">
        <button
          class="nav-item"
          :class="{ active: activeId === translatedFooterItem.id }"
          @click="emit('update:activeId', translatedFooterItem.id)"
        >
          <span class="nav-icon">
            <component :is="translatedFooterItem.icon" :size="20" />
          </span>
          <span class="nav-label">{{ translatedFooterItem.label }}</span>
          <span class="nav-tooltip">{{ translatedFooterItem.label }}</span>
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  --sidebar-width: 50px;
  --sidebar-expanded: 180px;
  width: var(--sidebar-width);
  display: flex;
  flex-direction: column;
  transition: width 0.25s ease;
  flex-shrink: 0;
  position: relative;
  overflow: visible;
  background: var(--panel-bg);
  border-radius: 0 0 0 16px;
}

.sidebar.expanded {
  width: var(--sidebar-expanded);
}

.sidebar-inner {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 8px 6px;
  gap: 4px;
}

.sidebar:not(.expanded) .sidebar-inner {
  padding: 8px 7px;
}

.user-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 6px;
  border: none;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  color: var(--sidebar-color);
  transition: background 0.15s;
  flex-shrink: 0;
  position: relative;
}

.sidebar:not(.expanded) .user-btn {
  justify-content: center;
  padding: 8px 0;
}

.user-btn:hover {
  background: var(--sidebar-hover);
  border-radius: 100px;
}

.user-btn.active {
  background: var(--sidebar-active);
  color: var(--sidebar-active-color);
  border-radius: 100px;
}

.user-avatar {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  flex-shrink: 0;
  image-rendering: pixelated;
}

.user-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  overflow: hidden;
}

.user-name {
  font-size: 13px;
  font-weight: 600;
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-source {
  font-size: 10px;
  opacity: 0.5;
  line-height: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-arrow {
  margin-left: auto;
  opacity: 0.4;
  flex-shrink: 0;
}

.toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  color: var(--sidebar-color);
  transition: background 0.15s;
  flex-shrink: 0;
}

.sidebar:not(.expanded) .toggle-btn {
  margin-top: 4px;
}

.toggle-btn:hover {
  background: var(--sidebar-hover);
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 8px;
}

.nav-footer {
  margin-top: auto;
  padding-top: 4px;
}

.nav-divider {
  height: 1px;
  background: rgba(128, 128, 128, 0.2);
  margin: 6px 8px 4px;
}

.section-label {
  display: block;
  padding: 0 8px;
  margin-top: 3px;
  font-size: 11px;
  opacity: 0;
  color: var(--sidebar-color);
  transition: opacity 0.2s ease;
  user-select: none;
}

.expanded .section-label {
  opacity: 0.5;
}

.nav-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-item {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 10px;
  width: 100%;
  padding: 8px 8px;
  border: none;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  font-family: inherit;
  transition: background 0.15s;
  white-space: nowrap;
  color: var(--sidebar-color);
  position: relative;
}

.sidebar:not(.expanded) .nav-item {
  justify-content: center;
  padding: 8px 0;
  gap: 0;
}

.sidebar:not(.expanded) .nav-label {
  display: none;
}

.nav-item:hover {
  background: var(--sidebar-hover);
  border-radius: 100px;
}

.nav-item.active {
  background: var(--sidebar-active);
  color: var(--sidebar-active-color);
  border-radius: 100px;
}

.nav-item.highlight {
  background: rgba(0, 120, 212, 0.1);
  color: var(--sidebar-active-color);
  border: 1px solid rgba(0, 120, 212, 0.3);
  border-radius: 50%;
  transition: border-radius 0.25s ease, background 0.15s;
  margin-top: 4px;
  width: 36px;
  height: 36px;
}

.expanded .nav-item.highlight {
  border-radius: 18px;
  width: 100%;
  height: auto;
}

.expanded .nav-item.highlight {
  border-radius: 18px;
}

.nav-item.highlight:hover {
  background: rgba(0, 120, 212, 0.18);
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

.nav-item:hover .nav-tooltip {
  opacity: 1;
}

.nav-label {
  opacity: 0;
  overflow: hidden;
  transition: opacity 0.2s ease;
}

.expanded .nav-label {
  opacity: 0.85;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
