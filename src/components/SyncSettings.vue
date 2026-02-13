<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-container">
      <div class="modal-header">
        <h3>GitHub Sync Settings</h3>
        <button class="close-btn" @click="close">&times;</button>
      </div>
      
      <div class="modal-body">
        <div class="form-group">
          <label>GitHub Repository</label>
          <input 
            v-model="settings.repo" 
            type="text" 
            placeholder="e.g. username/repo-name"
            class="form-input"
          />
          <span class="help-text">The repository where your collections will be stored.</span>
        </div>

        <div class="form-group">
          <label>Branch</label>
          <input 
            v-model="settings.branch" 
            type="text" 
            placeholder="main"
            class="form-input"
          />
        </div>

        <div class="form-group">
          <label>File Path</label>
          <input 
            v-model="settings.path" 
            type="text" 
            placeholder="curlmaster.json"
            class="form-input"
          />
        </div>

        <div class="form-group">
          <label>Personal Access Token</label>
          <div class="password-input">
            <input 
              v-model="token" 
              :type="showToken ? 'text' : 'password'" 
              placeholder="ghp_xxxxxxxxxxxx"
              class="form-input"
            />
            <button @click="showToken = !showToken" class="toggle-token">
              {{ showToken ? 'Hide' : 'Show' }}
            </button>
          </div>
          <span class="help-text">Requires 'repo' scope. Stored securely in Windows Credential Manager.</span>
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>
      </div>

      <div class="modal-footer">
        <button class="cancel-btn" @click="close">Cancel</button>
        <button 
          class="save-btn" 
          :disabled="isLoading"
          @click="saveSettings"
        >
          {{ isLoading ? 'Saving...' : 'Save & Sync' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  isOpen: boolean,
  workspaceId: string
}>();

const emit = defineEmits(['close', 'saved']);

const isLoading = ref(false);
const showToken = ref(false);
const token = ref('');
const error = ref('');

const settings = reactive({
  repo: '',
  branch: 'main',
  path: 'curlmaster.json'
});

onMounted(async () => {
    // Load existing settings from DB/Store
    try {
        const dbSettings = await invoke('get_settings') as Record<string, string>;
        if (dbSettings.github_repo) settings.repo = dbSettings.github_repo;
        if (dbSettings.github_branch) settings.branch = dbSettings.github_branch;
        if (dbSettings.github_path) settings.path = dbSettings.github_path;
        
        // Try to fetch token from secure storage
        const savedToken = await invoke('get_github_token') as string;
        if (savedToken) token.value = savedToken;
    } catch (e) {
        console.error('Failed to load GitHub settings:', e);
    }
});

const saveSettings = async () => {
  if (!settings.repo || !token.value) {
    error.value = 'Repository and Token are required.';
    return;
  }

  isLoading.value = true;
  error.value = '';

  try {
    // 1. Save settings to DB
    await invoke('update_setting', { key: 'github_repo', value: settings.repo });
    await invoke('update_setting', { key: 'github_branch', value: settings.branch });
    await invoke('update_setting', { key: 'github_path', value: settings.path });

    // 2. Save token to secure storage
    await invoke('save_github_token', { token: token.value });

    // 3. Initial Sync
    await invoke('sync_to_github', { 
        workspaceId: props.workspaceId,
        repo: settings.repo,
        path: settings.path,
        branch: settings.branch
    });

    emit('saved');
    close();
  } catch (e: any) {
    error.value = `Failed to save or sync: ${e}`;
  } finally {
    isLoading.value = false;
  }
};

const close = () => {
  emit('close');
};
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-container {
  background: #1e1e1e;
  width: 100%;
  max-width: 500px;
  border-radius: 8px;
  border: 1px solid #333;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.modal-header {
  padding: 16px 20px;
  border-bottom: 1px solid #333;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h3 {
  margin: 0;
  color: #fff;
  font-size: 18px;
}

.close-btn {
  background: none;
  border: none;
  color: #888;
  font-size: 24px;
  cursor: pointer;
}

.modal-body {
  padding: 20px;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  color: #aaa;
  margin-bottom: 8px;
  font-size: 13px;
}

.form-input {
  width: 100%;
  background: #2a2a2a;
  border: 1px solid #333;
  color: #fff;
  padding: 10px 12px;
  border-radius: 4px;
  font-size: 14px;
}

.password-input {
  display: flex;
  gap: 8px;
}

.toggle-token {
  background: #333;
  border: none;
  color: #ccc;
  padding: 0 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.help-text {
  display: block;
  margin-top: 6px;
  color: #666;
  font-size: 11px;
}

.error-message {
  color: #ff4d4d;
  background: rgba(255, 77, 77, 0.1);
  padding: 10px;
  border-radius: 4px;
  font-size: 13px;
  margin-top: 10px;
}

.modal-footer {
  padding: 16px 20px;
  border-top: 1px solid #333;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.cancel-btn {
  background: transparent;
  border: 1px solid #444;
  color: #ccc;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
}

.save-btn {
  background: #667eea;
  border: none;
  color: #fff;
  padding: 10px 20px;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
