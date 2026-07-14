<script setup lang="ts">
import { User, Plus, X, Ellipsis, Trash2, Pencil, Palette, LoaderCircle } from "@lucide/vue";
import steveAvatar from "../assets/imgs/skins/avator/steve.png";
import alexAvatar from "../assets/imgs/skins/avator/alex.png";

import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl as openExternalUrl } from "@tauri-apps/plugin-opener";

interface AccountEntry {
  name: string;
  type: string;
  uuid: string;
  ms_refresh_token?: string | null;
  mc_token?: string | null;
  xuid?: string | null;
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
      uuid: null,
      msRefreshToken: null,
      mcToken: null,
      xuid: null,
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

// ── Microsoft Auth ──
const MS_CLIENT_ID = "1cabeaef-70e5-4834-8aeb-85ff3671c46d"
interface DeviceCodeResponse {
  user_code: string
  device_code: string
  verification_uri: string
  interval: number
}
interface MicrosoftAccount {
  name: string
  uuid: string
  xuid: string
  mc_token: string
  ms_refresh_token: string
}

type PollResult =
  | { status: "pending" }
  | { status: "complete" } & MicrosoftAccount
  | { status: "error"; reason: string }

const msDeviceCode = ref<DeviceCodeResponse | null>(null)
const msLoading = ref(false)
const msPollTimer = ref<ReturnType<typeof setTimeout> | null>(null)
const msPollProgress = ref(0)
const msError = ref("")

async function startMsLogin() {
  msError.value = ""
  msLoading.value = true
  msDeviceCode.value = null
  try {
    const code = await invoke<DeviceCodeResponse>("start_microsoft_auth", { clientId: MS_CLIENT_ID })
    msDeviceCode.value = code
    msPollProgress.value = 0
    pollMsAuth(code)
  } catch (e: any) {
    msError.value = "启动 Microsoft 登录失败: " + (e?.toString() || "未知错误")
  } finally {
    msLoading.value = false
  }
}

async function pollMsAuth(code: DeviceCodeResponse) {
  try {
    const result = await invoke<PollResult>("poll_microsoft_auth", {
      clientId: MS_CLIENT_ID,
      deviceCode: code.device_code,
    })
    if (result.status === "complete") {
      const account = result as MicrosoftAccount & { status: "complete" }
      await invoke<AccountEntry[]>("add_account", {
        name: account.name,
        accountType: "microsoft",
        uuid: account.uuid,
        msRefreshToken: account.ms_refresh_token,
        mcToken: account.mc_token,
        xuid: account.xuid,
      })
      window.dispatchEvent(new CustomEvent("account-changed"))
      accounts.value = await invoke<AccountEntry[]>("get_accounts")
      showDialog.value = false
      msDeviceCode.value = null
      return
    } else if (result.status === "pending") {
      // continue below to poll again
    } else {
      alert("Microsoft 登录出错: " + (result as any).reason)
      return
    }
  } catch (e: any) {
    console.error("poll invoke failed:", e)
    return
  }

  // poll again after interval
  const progress = Math.min(msPollProgress.value + 3, 95)
  msPollProgress.value = progress
  const delay = (code.interval || 5) * 1000
  msPollTimer.value = setTimeout(() => pollMsAuth(code), delay)
}

function cancelMsLogin() {
  if (msPollTimer.value) {
    clearTimeout(msPollTimer.value)
    msPollTimer.value = null
  }
  msDeviceCode.value = null
  msPollProgress.value = 0
}

function openMsUrl(url: string) {
  openExternalUrl(url)
}

onMounted(loadAccounts)

onUnmounted(() => {
  if (msPollTimer.value) clearTimeout(msPollTimer.value)
})
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
              :class="{ active: dialogTab === i, disabled: i === 2 }"
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
            <div v-else-if="dialogTab === 1" class="dialog-microsoft">
              <div v-if="!msDeviceCode" class="ms-start">
                <span class="ms-desc">使用 Microsoft 账户登录以获取您的 Minecraft Java 角色</span>
                <button class="ms-login-btn" :disabled="msLoading" @click="startMsLogin">
                  <LoaderCircle v-if="msLoading" :size="16" class="spin" />
                  <span>{{ msLoading ? '请稍候...' : '登录微软账户' }}</span>
                </button>
                <span v-if="msError" class="ms-error">{{ msError }}</span>
              </div>
              <div v-else class="ms-code">
                <span class="ms-desc">请在浏览器中打开以下链接并输入代码</span>
                <a class="ms-url" :href="msDeviceCode.verification_uri" target="_blank" @click.prevent="openMsUrl(msDeviceCode.verification_uri)">{{ msDeviceCode.verification_uri }}</a>
                <div class="ms-code-box">{{ msDeviceCode.user_code }}</div>
                <span class="ms-hint">等待验证中...</span>
                <div class="ms-tbar"><div class="ms-tfill" :style="{ width: msPollProgress + '%' }"></div></div>
                <button class="ms-cancel-btn" @click="cancelMsLogin">取消</button>
              </div>
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
  padding: 24px 0 80px;
  overflow-y: auto;
}

.acc-page > * {
  padding-left: 28px;
  padding-right: 28px;
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

.dialog-microsoft {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 16px 0;
}

.ms-start {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  width: 100%;
}

.ms-desc {
  font-size: 13px;
  color: var(--title-color);
  opacity: 0.7;
  text-align: center;
  line-height: 1.5;
}

.ms-login-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 10px;
  border: none;
  border-radius: 8px;
  background: #0078d4;
  color: #fff;
  font-size: 14px;
  font-weight: 500;
  font-family: inherit;
  cursor: pointer;
  transition: opacity 0.15s;
}

.ms-login-btn:hover:not(:disabled) { opacity: 0.85; }
.ms-login-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.ms-error { font-size: 12px; color: #e74c3c; text-align: center; line-height: 1.4; }

.ms-code {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.ms-url {
  font-size: 14px;
  font-weight: 600;
  color: #0078d4;
  text-decoration: none;
  cursor: pointer;
}

.ms-url:hover { text-decoration: underline; }

.ms-code-box {
  font-size: 28px;
  font-weight: 700;
  letter-spacing: 6px;
  padding: 12px 24px;
  border-radius: 10px;
  background: rgba(128,128,128,0.08);
  color: var(--title-color);
  font-family: monospace;
}

.ms-hint {
  font-size: 12px;
  color: var(--title-color);
  opacity: 0.45;
}

.ms-tbar {
  width: 100%;
  height: 3px;
  border-radius: 2px;
  background: rgba(128,128,128,0.12);
  overflow: hidden;
}

.ms-tfill {
  height: 100%;
  border-radius: 2px;
  background: #0078d4;
  transition: width 0.5s ease;
}

.ms-cancel-btn {
  padding: 6px 16px;
  border: 1px solid rgba(128,128,128,0.25);
  border-radius: 6px;
  background: transparent;
  color: var(--title-color);
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
  opacity: 0.6;
  transition: opacity 0.15s;
}

.ms-cancel-btn:hover { opacity: 1; }

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.spin {
  animation: spin 1s linear infinite;
}
</style>
