<template>
  <div class="titlebar">
    <!-- Drag Region Layer -->
    <div class="drag-region" data-tauri-drag-region></div>
    
    <div class="titlebar-content">
      <div class="app-logo">
        <div class="logo-icon">CM</div>
        <span class="app-name">CurlMaster</span>
      </div>

      <div class="environment-selector">
        <select 
          v-model="activeEnvId" 
          @change="switchEnvironment"
          class="env-select"
        >
          <option :value="null">No Environment</option>
          <option v-for="env in environments" :key="env.id" :value="env.id">
            {{ env.name }}
          </option>
        </select>
        <button class="env-settings-btn" @click="openEnvSettings" title="Manage Environments">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 15a3 3 0 100-6 3 3 0 000 6z"/>
            <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/>
          </svg>
        </button>
      </div>
      
      <div class="titlebar-actions">
        <button class="titlebar-btn sync-btn" @click="$emit('openSync')" :title="syncLoggedIn ? 'Sync &amp; Account' : 'Sign in to sync'">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/>
          </svg>
        </button>
        <button class="titlebar-btn history-btn" @click="$emit('toggleHistory')" title="Toggle History">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </button>
        
        <div class="divider"></div>

        <button class="titlebar-btn" @click="minimizeWindow" title="Minimize">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="0" y="5" width="12" height="2" fill="currentColor"/>
          </svg>
        </button>
        <button class="titlebar-btn" @click="maximizeWindow" :title="isMaximized ? 'Restore' : 'Maximize'">
          <svg v-if="!isMaximized" width="12" height="12" viewBox="0 0 12 12">
            <rect x="1.5" y="1.5" width="9" height="9" stroke="currentColor" stroke-width="1.2" fill="none"/>
          </svg>
          <svg v-else width="12" height="12" viewBox="0 0 12 12">
            <rect x="3.5" y="1.5" width="7" height="7" stroke="currentColor" stroke-width="1.2" fill="none"/>
            <path d="M1.5 10.5h7v-7" stroke="currentColor" stroke-width="1.2" fill="none"/>
            <path d="M1.5 3.5v7" stroke="currentColor" stroke-width="1.2" fill="none"/>
          </svg>
        </button>
        <button class="titlebar-btn close-btn" @click="closeWindow" title="Close">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path d="M1 1 L11 11 M11 1 L1 11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';

const appWindow = getCurrentWindow();

const props = defineProps<{
  workspaceId: string;
  syncLoggedIn?: boolean;
}>();

const emit = defineEmits<{
  openEnvSettings: [];
  envChanged: [];
  toggleHistory: [];
  openSync: [];
}>();

const environments = ref<any[]>([]);
const activeEnvId = ref<string | null>(null);
const isMaximized = ref(false);

const fetchEnvironments = async () => {
  try {
    const wId = await invoke('ensure_workspace', { workspaceId: props.workspaceId });
    // props.workspaceId is managed by parent
    
    const envs = await invoke('get_environments', { workspaceId: props.workspaceId }) as any[];
    environments.value = envs;
    
    const active = envs.find(e => e.is_active);
    activeEnvId.value = active ? active.id : null;
  } catch (error) {
    console.error('Failed to fetch environments:', error);
  }
};

watch(() => props.workspaceId, () => {
    fetchEnvironments();
});

const checkMaximizedState = async () => {
    try {
        isMaximized.value = await appWindow.isMaximized();
    } catch (e) {
        console.error('Failed to check maximized state:', e);
    }
};

let unlistenResize: (() => void) | null = null;

onMounted(async () => {
  fetchEnvironments();
  await checkMaximizedState();
  // Listen for window resize events to update the icon
  // In Tauri v2, the event name is 'tauri://resize' or we can poll/listen to window changes
  // But appWindow.onResized is the correct API.
  unlistenResize = await appWindow.onResized(() => {
    checkMaximizedState();
  });
});

onUnmounted(() => {
    if (unlistenResize) {
        unlistenResize();
    }
});

const switchEnvironment = async () => {
  try {
    await invoke('set_active_env', { 
      id: activeEnvId.value, 
      workspaceId: props.workspaceId 
    });
    emit('envChanged');
  } catch (error) {
    console.error('Failed to switch environment:', error);
  }
};

const openEnvSettings = () => {
  emit('openEnvSettings');
};

// Window controls
const minimizeWindow = async () => await appWindow.minimize();
const maximizeWindow = async () => {
    await appWindow.toggleMaximize();
    // Small delay to ensure state update catches the change
    setTimeout(checkMaximizedState, 100);
};
const closeWindow = async () => await appWindow.close();

// Expose refresh method
defineExpose({
  refresh: fetchEnvironments
});
</script>

<style scoped>
.titlebar {
  height: 40px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  user-select: none;
  position: relative;
  z-index: 1000;
}

/* Dedicated drag region layer */
.drag-region {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 10;
}

.titlebar-content {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  position: relative;
  z-index: 20;
  pointer-events: none; /* Let clicks pass through to drag region */
}

/* Enable interactivity for specific controls */
.environment-selector,
.titlebar-actions {
  pointer-events: auto;
}

.app-logo {
  display: flex;
  align-items: center;
  gap: 10px;
  /* Already pointer-events: none via parent, but being explicit is fine */
}

.logo-icon {
  width: 24px;
  height: 24px;
  background: rgba(255, 255, 255, 0.2);
  border: 2px solid rgba(255, 255, 255, 0.8);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 10px;
  color: white;
  letter-spacing: -0.5px;
}

.app-name {
  font-size: 13px;
  font-weight: 600;
  color: white;
  letter-spacing: 0.3px;
}

.environment-selector {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(0, 0, 0, 0.2);
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.env-select {
  background: transparent;
  border: none;
  color: #ddd;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  padding: 0 4px;
  outline: none;
}

.env-select option {
  background: #1a1a1a;
  color: #fff;
}

.env-settings-btn {
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s;
}

.env-settings-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.titlebar-actions {
  display: flex;
  gap: 8px;
}

.titlebar-btn {
  width: 32px;
  height: 28px;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  border-radius: 4px;
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.titlebar-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.titlebar-btn:active {
  background: rgba(255, 255, 255, 0.15);
  transform: scale(0.95);
}

.titlebar-btn.close-btn:hover {
  background: #e74c3c;
}

.titlebar-btn.close-btn:active {
  background: #c0392b;
}

.sync-btn, .history-btn {
  width: 28px;
  height: 28px;
  margin-right: 4px;
}

.divider {
  width: 1px;
  height: 16px;
  background: rgba(255, 255, 255, 0.2);
  margin: 0 8px;
}
</style>
