<template>
  <Transition name="modal">
    <div v-if="isOpen" class="modal-overlay" @click.self="$emit('close')">
      <div class="sync-modal">
        <div class="modal-header">
          <h3>Sync &amp; Collaborate</h3>
          <button class="close-btn" @click="$emit('close')">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        </div>
        <div class="modal-body">
          <p v-if="error" class="error-msg">{{ error }}</p>
          <p v-if="syncConfig?.lastError" class="error-msg sync-failed-msg">
            Sync failed: {{ syncConfig.lastError }}
            <button type="button" class="dismiss-btn" @click="clearError">Dismiss</button>
          </p>
          <template v-if="syncConfig?.hasToken">
            <p class="logged-in-msg">You are signed in. Changes are synced when you save.</p>
            <p v-if="syncConfig?.lastSyncedAt" class="last-synced-msg">Last synced: {{ formatLastSynced(syncConfig.lastSyncedAt) }}</p>
            <button class="modal-btn logout" @click="handleLogout">Sign out</button>
          </template>
          <template v-else>
            <div class="form-group">
              <label>Sync server URL</label>
              <input
                v-model="syncUrl"
                type="url"
                placeholder="https://your-sync-server.com"
                class="modal-input"
              />
            </div>
            <div class="form-group">
              <label>Email</label>
              <input
                v-model="email"
                type="email"
                placeholder="you@example.com"
                class="modal-input"
              />
            </div>
            <div class="form-group">
              <label>Password</label>
              <input
                v-model="password"
                type="password"
                placeholder="••••••••"
                class="modal-input"
                @keyup.enter="submit"
              />
            </div>
            <div class="form-actions">
              <button class="modal-btn secondary" @click="mode = mode === 'login' ? 'register' : 'login'">
                {{ mode === 'login' ? 'Create account' : 'Already have an account?' }}
              </button>
              <button class="modal-btn primary" :disabled="loading" @click="submit">
                {{ loading ? '…' : (mode === 'login' ? 'Sign in' : 'Sign up') }}
              </button>
            </div>
          </template>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  isOpen: boolean;
  syncConfig: { url: string; hasToken: boolean; lastError?: string | null; lastSyncedAt?: number | null } | null;
}>();

const emit = defineEmits<{
  close: [];
  'sync-changed': [];
}>();

const syncUrl = ref('');
const email = ref('');
const password = ref('');
const mode = ref<'login' | 'register'>('login');
const loading = ref(false);
const error = ref('');

watch(() => props.isOpen, (open) => {
  if (open) {
    error.value = '';
    if (props.syncConfig?.url) syncUrl.value = props.syncConfig.url;
  }
});

watch(() => props.syncConfig?.url, (url) => {
  if (url) syncUrl.value = url;
}, { immediate: true });

async function submit() {
  error.value = '';
  if (!syncUrl.value.trim()) {
    error.value = 'Enter sync server URL';
    return;
  }
  if (!email.value.trim() || !password.value) {
    error.value = 'Enter email and password';
    return;
  }
  loading.value = true;
  try {
    if (mode.value === 'login') {
      await invoke('sync_login', {
        syncUrl: syncUrl.value.trim(),
        email: email.value.trim(),
        password: password.value
      });
    } else {
      await invoke('sync_register', {
        syncUrl: syncUrl.value.trim(),
        email: email.value.trim(),
        password: password.value
      });
    }
    emit('sync-changed');
    emit('close');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

async function handleLogout() {
  try {
    await invoke('sync_logout');
    emit('sync-changed');
    emit('close');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

async function clearError() {
  try {
    await invoke('clear_sync_error');
    emit('sync-changed');
  } catch {
    // ignore
  }
}

function formatLastSynced(ts: number): string {
  const d = new Date(ts);
  const now = Date.now();
  const diff = Math.floor((now - ts) / 1000);
  if (diff < 60) return 'just now';
  if (diff < 3600) return `${Math.floor(diff / 60)} min ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} h ago`;
  return d.toLocaleString();
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}
.sync-modal {
  background: #1a1a1a;
  border-radius: 12px;
  border: 1px solid #333;
  min-width: 360px;
  max-width: 90vw;
}
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #333;
}
.modal-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: #fff;
}
.close-btn {
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}
.close-btn:hover {
  color: #fff;
  background: #333;
}
.modal-body {
  padding: 20px;
}
.error-msg {
  color: #e74c3c;
  font-size: 13px;
  margin-bottom: 12px;
}
.sync-failed-msg {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.dismiss-btn {
  background: #333;
  border: none;
  color: #ccc;
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}
.dismiss-btn:hover {
  background: #444;
}
.last-synced-msg {
  color: #888;
  font-size: 12px;
  margin-bottom: 8px;
}
.logged-in-msg {
  color: #aaa;
  font-size: 14px;
  margin-bottom: 16px;
}
.form-group {
  margin-bottom: 14px;
}
.form-group label {
  display: block;
  font-size: 12px;
  color: #888;
  margin-bottom: 4px;
}
.modal-input {
  width: 100%;
  padding: 10px 12px;
  background: #0a0a0a;
  border: 1px solid #333;
  border-radius: 6px;
  color: #fff;
  font-size: 14px;
}
.modal-input:focus {
  outline: none;
  border-color: #667eea;
}
.form-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  margin-top: 18px;
}
.modal-btn {
  padding: 10px 18px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
}
.modal-btn.primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}
.modal-btn.primary:hover:not(:disabled) {
  opacity: 0.9;
}
.modal-btn.primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.modal-btn.secondary {
  background: #333;
  color: #ccc;
}
.modal-btn.secondary:hover {
  background: #444;
}
.modal-btn.logout {
  background: #333;
  color: #ccc;
}
.modal-btn.logout:hover {
  background: #444;
}
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
