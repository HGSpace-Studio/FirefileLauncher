<script setup lang="ts">
import { User, Plus, X, Ellipsis, Trash2, Pencil, Palette } from "@lucide/vue";
import steveAvatar from "../assets/imgs/skins/avator/steve.png";
import alexAvatar from "../assets/imgs/skins/avator/alex.png";

import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface AccountEntry {
  name: string;
  type: string;
  uuid: string;
}

const accounts = ref<AccountEntry[]>([]);
const showDialog = ref(false);
const dialogTab = ref(0);
const dialogTabs = ["离线", "微软账户", "第三方登录"];
const offlineNameInput = ref("");
const avatars = [steveAvatar, alexAvatar];
const currentAvatar = ref(avatars[0]);

function getAvatar(name: string): string {
  const hash = name ? name.charCodeAt(0) % avatars.length : 0;
  return avatars[hash];
}

const typeLabels: Record<string, string> = {
  offline: "离线账号",
  microsoft: "微软账户",
};

function getTypeLabel(type: string): string {
  return typeLabels[type] || type;
}

async function loadAccounts() {
  try {
    accounts.value = await invoke<AccountEntry[]>("get_accounts");
  } catch {
    accounts.value = [];
  }
}

function onAvatarInput() {
  currentAvatar.value = avatars[Math.floor(Math.random() * avatars.length)];
}

function openNewAccountDialog() {
  dialogTab.value = 0;
  offlineNameInput.value = "";
  currentAvatar.value = avatars[Math.floor(Math.random() * avatars.length)];
  showDialog.value = true;
}

async function confirmOfflineAccount() {
  const name = offlineNameInput.value.trim();
  if (!name) return;
  try {
    accounts.value = await invoke<AccountEntry[]>("add_account", {
      name,
      accountType: "offline",
    });
    window.dispatchEvent(new CustomEvent("account-changed"));
    showDialog.value = false;
  } catch {
    // ignore
  }
}

async function removeAccount(name: string) {
  try {
    accounts.value = await invoke<AccountEntry[]>("remove_account", { name });
    window.dispatchEvent(new CustomEvent("account-changed"));
  } catch {
    // ignore
  }
}

onMounted(loadAccounts);
</script>

<template>
  <div class="acc-page">
    <div class="acc-header">
      <span class="acc-header-sub">账号</span>
      <div class="acc-header-row">
        <span class="acc-header-main">我的账号</span>
        <button class="acc-add-btn" @click="openNewAccountDialog">
          <Plus :size="16" />
          <span>新建账户</span>
        </button>
      </div>
    </div>
    <div class="acc-area">
      <div v-if="accounts.length === 0" class="acc-empty">
        <User :size="56" class="acc-empty-icon" />
        <span class="acc-empty-text">暂无账户信息</span>
      </div>
      <div v-else class="acc-list">
        <div v-for="acc in accounts" :key="acc.name" class="acc-card">
          <img :src="getAvatar(acc.name)" class="acc-card-avatar" />
          <div class="acc-card-body">
            <span class="acc-card-name">{{ acc.name }}</span>
            <span class="acc-card-type">{{ getTypeLabel(acc.type) }}</span>
          </div>
          <div class="acc-card-actions">
            <button class="acc-action-btn" title="编辑">
              <Pencil :size="16" />
            </button>
            <button class="acc-action-btn" title="皮肤">
              <Palette :size="16" />
            </button>
            <button class="acc-action-btn danger" title="删除" @click="removeAccount(acc.name)">
              <Trash2 :size="16" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <Teleport to="body">
      <div v-if="showDialog" class="dialog-overlay" @click.self="showDialog = false">
        <div class="dialog-box">
          <div class="dialog-header">
            <span class="dialog-title">新建账户</span>
            <button class="dialog-close-btn" @click="showDialog = false">
              <X :size="18" />
            </button>
          </div>
          <div class="dialog-tabs">
            <button
              v-for="(tab, i) in dialogTabs"
              :key="i"
              class="dialog-tab"
              :class="{ active: dialogTab === i, disabled: i === 2 || i === 1 }"
              @click="dialogTab = i"
            >{{ tab }}</button>
          </div>
          <div class="dialog-body">
            <div v-if="dialogTab === 0" class="dialog-offline">
              <div class="dialog-avatar-wrap">
                <img :src="currentAvatar" class="dialog-avatar" />
              </div>
              <span class="dialog-offline-label">输入您的玩家名称</span>
              <div class="dialog-input-wrap">
                <input
                  v-model="offlineNameInput"
                  class="dialog-input"
                  placeholder="Steve"
                  maxlength="16"
                  @input="onAvatarInput"
                  @keyup.enter="confirmOfflineAccount"
                />
              </div>
              <button
                class="dialog-confirm-btn"
                :disabled="!offlineNameInput.trim()"
                @click="confirmOfflineAccount"
              >确认</button>
            </div>
            <div v-else-if="dialogTab === 1" class="dialog-placeholder">
              <Ellipsis :size="32" class="placeholder-icon" />
              <span>微软账户登录暂未开放</span>
            </div>
            <div v-else class="dialog-placeholder">
              <Ellipsis :size="32" class="placeholder-icon" />
              <span>第三方登录暂未开放</span>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.acc-page {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px 28px;
  overflow-y: auto;
}

.acc-header {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex-shrink: 0;
}

.acc-header-sub {
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.45;
  line-height: 1;
}

.acc-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.acc-header-main {
  font-size: 18px;
  font-weight: 600;
  color: var(--title-color);
  line-height: 1.2;
}

.acc-add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 16px;
  border: none;
  border-radius: 100px;
  background: var(--sidebar-active-color);
  color: #fff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
  font-family: inherit;
}

.acc-add-btn:hover {
  opacity: 0.85;
}

.acc-area {
  flex: 1;
  margin-top: 20px;
  min-height: 0;
}

.acc-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 100%;
  color: var(--title-color);
  opacity: 0.4;
}

.acc-empty-icon {
  opacity: 0.5;
}

.acc-empty-text {
  font-size: 14px;
}

.acc-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.acc-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 12px;
  background: var(--panel-bg);
  transition: background 0.15s;
}

.acc-card:hover {
  background: rgba(128, 128, 128, 0.08);
}

.acc-card-avatar {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  flex-shrink: 0;
  image-rendering: pixelated;
}

.acc-card-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.acc-card-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--title-color);
  line-height: 1.2;
}

.acc-card-type {
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.45;
  line-height: 1;
}

.acc-card-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.acc-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--title-color);
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
  opacity: 0.45;
}

.acc-action-btn:hover {
  background: rgba(128, 128, 128, 0.12);
  opacity: 0.85;
}

.acc-action-btn.danger:hover {
  background: rgba(231, 76, 60, 0.15);
  color: #e74c3c;
  opacity: 1;
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 200;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog-box {
  background: var(--panel-bg);
  border-radius: 16px;
  width: 420px;
  max-width: 90vw;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px 0;
}

.dialog-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--title-color);
}

.dialog-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--title-color);
  cursor: pointer;
  transition: background 0.15s;
}

.dialog-close-btn:hover {
  background: rgba(128, 128, 128, 0.15);
}

.dialog-tabs {
  display: flex;
  gap: 4px;
  margin: 14px 20px 0;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.15);
}

.dialog-tab {
  flex: 1;
  padding: 8px 0;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--title-color);
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s;
  opacity: 0.6;
}

.dialog-tab:hover:not(.disabled) {
  background: rgba(128, 128, 128, 0.08);
  opacity: 0.8;
}

.dialog-tab.active {
  background: var(--sidebar-active-color);
  color: #fff;
  opacity: 1;
}

.dialog-tab.disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.dialog-body {
  padding: 20px;
}

.dialog-offline {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
}

.dialog-avatar-wrap {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  overflow: hidden;
  background: rgba(128, 128, 128, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.dialog-avatar {
  width: 100%;
  height: 100%;
  object-fit: cover;
  image-rendering: pixelated;
}

.dialog-offline-label {
  font-size: 13px;
  color: var(--title-color);
  opacity: 0.7;
}

.dialog-input-wrap {
  width: 100%;
}

.dialog-input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.1);
  color: var(--title-color);
  font-size: 14px;
  font-family: inherit;
  outline: none;
  text-align: center;
  transition: border-color 0.15s;
}

.dialog-input:focus {
  border-color: var(--sidebar-active-color);
}

.dialog-confirm-btn {
  width: 100%;
  padding: 10px;
  border: none;
  border-radius: 8px;
  background: var(--sidebar-active-color);
  color: #fff;
  font-size: 14px;
  font-weight: 500;
  font-family: inherit;
  cursor: pointer;
  transition: opacity 0.15s;
}

.dialog-confirm-btn:hover:not(:disabled) {
  opacity: 0.85;
}

.dialog-confirm-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.dialog-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 0;
  color: var(--title-color);
  opacity: 0.4;
  font-size: 14px;
}

.placeholder-icon {
  opacity: 0.5;
}
</style>
