<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-container">
      <div class="modal-header">
        <div class="header-title">
          <svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor" class="github-icon">
            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
          </svg>
          <h3>GitHub Sync</h3>
        </div>
        <button class="close-btn" @click="close">&times;</button>
      </div>
      
      <!-- ═══════ SETTINGS VIEW ═══════ -->
      <template v-if="view === 'settings'">
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
            class="pull-btn" 
            :disabled="isLoading || !settings.repo || !token"
            @click="handlePull"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 12l-4-4h2.5V4h3v4H12L8 12z"/>
              <path d="M2 14h12v1H2z"/>
            </svg>
            {{ isLoading && activeAction === 'pull' ? 'Pulling...' : 'Pull' }}
          </button>
          <button 
            class="push-btn" 
            :disabled="isLoading || !settings.repo || !token"
            @click="handlePush"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 4l4 4h-2.5v4h-3V8H4l4-4z"/>
              <path d="M2 1h12v1H2z"/>
            </svg>
            {{ isLoading && activeAction === 'push' ? 'Pushing...' : 'Push' }}
          </button>
        </div>
      </template>

      <!-- ═══════ PULL PREVIEW VIEW ═══════ -->
      <template v-if="view === 'pull-preview'">
        <div class="modal-body">
          <div class="pull-header">
            <div class="pull-source">
              <span class="source-label">From:</span>
              <code>{{ settings.repo }}/{{ settings.path }}</code>
              <span class="branch-badge">{{ settings.branch }}</span>
            </div>
          </div>

          <div v-if="pullData" class="pull-preview">
            <h4>Collections found:</h4>
            <div class="collections-list">
              <div 
                v-for="(col, index) in pullData.collections" 
                :key="index" 
                class="collection-preview"
              >
                <div class="collection-preview-header">
                  <svg width="16" height="16" viewBox="0 0 14 14" fill="#667eea">
                    <path d="M0 2a1 1 0 011-1h4l1 2h7a1 1 0 011 1v8a1 1 0 01-1 1H1a1 1 0 01-1-1V2z"/>
                  </svg>
                  <span class="col-name">{{ col.name }}</span>
                </div>
                <div class="collection-stats">
                  <span class="stat">
                    <svg width="12" height="12" viewBox="0 0 14 14" fill="currentColor">
                      <path d="M0 2a1 1 0 011-1h4l1 2h7a1 1 0 011 1v8a1 1 0 01-1 1H1a1 1 0 01-1-1V2z"/>
                    </svg>
                    {{ col.folders?.length || 0 }} folders
                  </span>
                  <span class="stat">
                    <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                      <path d="M14 1a1 1 0 011 1v12a1 1 0 01-1 1H2a1 1 0 01-1-1V2a1 1 0 011-1h12zM2 0a2 2 0 00-2 2v12a2 2 0 002 2h12a2 2 0 002-2V2a2 2 0 00-2-2H2z"/>
                      <path d="M5.854 4.854a.5.5 0 10-.708-.708l-3.5 3.5a.5.5 0 000 .708l3.5 3.5a.5.5 0 00.708-.708L2.707 8l3.147-3.146zm4.292 0a.5.5 0 01.708-.708l3.5 3.5a.5.5 0 010 .708l-3.5 3.5a.5.5 0 01-.708-.708L13.293 8l-3.147-3.146z"/>
                    </svg>
                    {{ countRequests(col) }} requests
                  </span>
                </div>
              </div>
            </div>

            <div class="warning-box">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8.982 1.566a1.13 1.13 0 0 0-1.96 0L.165 13.233c-.457.778.091 1.767.98 1.767h13.71c.889 0 1.438-.99.98-1.767L8.982 1.566zM8 5c.535 0 .954.462.9.995l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 5.995A.905.905 0 0 1 8 5zm.002 6a1 1 0 1 1 0 2 1 1 0 0 1 0-2z"/>
              </svg>
              <span>This will <strong>add</strong> the collections to your workspace. Existing collections won't be removed.</span>
            </div>
          </div>

          <div v-if="error" class="error-message">
            {{ error }}
          </div>
        </div>

        <div class="modal-footer">
          <button class="cancel-btn" @click="view = 'settings'; error = ''">Back</button>
          <button 
            class="import-btn" 
            :disabled="isLoading || !pullData?.collections?.length"
            @click="importPulledData"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 12l-4-4h2.5V4h3v4H12L8 12z"/>
              <path d="M2 14h12v1H2z"/>
            </svg>
            {{ isLoading ? 'Importing...' : `Import ${pullData?.collections?.length || 0} Collection(s)` }}
          </button>
        </div>
      </template>

      <!-- ═══════ SUCCESS VIEW ═══════ -->
      <template v-if="view === 'success'">
        <div class="modal-body success-body">
          <div class="success-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#48bb78" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
              <polyline points="22 4 12 14.01 9 11.01"/>
            </svg>
          </div>
          <h4 class="success-title">{{ successMessage }}</h4>
          <p class="success-subtitle">{{ successSubtext }}</p>
        </div>
        <div class="modal-footer">
          <button class="push-btn" @click="close">Done</button>
        </div>
      </template>
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

const view = ref<'settings' | 'pull-preview' | 'success'>('settings');
const isLoading = ref(false);
const activeAction = ref<'push' | 'pull' | ''>('');
const showToken = ref(false);
const token = ref('');
const error = ref('');
const pullData = ref<any>(null);
const successMessage = ref('');
const successSubtext = ref('');

const settings = reactive({
  repo: '',
  branch: 'main',
  path: 'curlmaster.json'
});

onMounted(async () => {
    try {
        const dbSettings = await invoke('get_settings') as Record<string, string>;
        if (dbSettings.github_repo) settings.repo = dbSettings.github_repo;
        if (dbSettings.github_branch) settings.branch = dbSettings.github_branch;
        if (dbSettings.github_path) settings.path = dbSettings.github_path;
        
        const savedToken = await invoke('get_github_token') as string;
        if (savedToken) token.value = savedToken;
    } catch (e) {
        console.error('Failed to load GitHub settings:', e);
    }
});

/** Save settings + token to local storage (shared by push & pull). */
const saveSettingsAndToken = async () => {
  await invoke('update_setting', { key: 'github_repo', value: settings.repo });
  await invoke('update_setting', { key: 'github_branch', value: settings.branch });
  await invoke('update_setting', { key: 'github_path', value: settings.path });
  await invoke('save_github_token', { token: token.value });
};

// ── PUSH ──────────────────────────────────────────────────────────────
const handlePush = async () => {
  if (!settings.repo || !token.value) {
    error.value = 'Repository and Token are required.';
    return;
  }

  isLoading.value = true;
  activeAction.value = 'push';
  error.value = '';

  try {
    await saveSettingsAndToken();

    await invoke('sync_to_github', { 
      workspaceId: props.workspaceId,
      repo: settings.repo,
      path: settings.path,
      branch: settings.branch
    });

    successMessage.value = 'Push Successful!';
    successSubtext.value = `Your workspace has been synced to ${settings.repo}/${settings.path}`;
    view.value = 'success';
    emit('saved');
  } catch (e: any) {
    error.value = `Push failed: ${e}`;
  } finally {
    isLoading.value = false;
    activeAction.value = '';
  }
};

// ── PULL ──────────────────────────────────────────────────────────────
const handlePull = async () => {
  if (!settings.repo || !token.value) {
    error.value = 'Repository and Token are required.';
    return;
  }

  isLoading.value = true;
  activeAction.value = 'pull';
  error.value = '';

  try {
    await saveSettingsAndToken();

    const data = await invoke('sync_from_github', { 
      workspaceId: props.workspaceId,
      repo: settings.repo,
      path: settings.path,
      branch: settings.branch
    });

    pullData.value = data;
    view.value = 'pull-preview';
  } catch (e: any) {
    error.value = `Pull failed: ${e}`;
  } finally {
    isLoading.value = false;
    activeAction.value = '';
  }
};

/** Import each pulled collection into the current workspace. */
const importPulledData = async () => {
  if (!pullData.value?.collections?.length) return;

  isLoading.value = true;
  error.value = '';

  try {
    for (const col of pullData.value.collections) {
      // Build the import payload matching the ImportCollection model
      const importPayload = {
        id: null, // Generate new IDs
        name: col.name,
        workspace_id: props.workspaceId,
        folders: (col.folders || []).map((f: any) => ({
          id: null,
          name: f.name,
          requests: (f.requests || []).map(mapRequest),
          folders: [] // Nested folders not supported in current import
        })),
        requests: (col.requests || []).map(mapRequest)
      };

      await invoke('sync_import_collection', { collection: importPayload });
    }

    successMessage.value = 'Pull Successful!';
    successSubtext.value = `Imported ${pullData.value.collections.length} collection(s) from GitHub.`;
    view.value = 'success';
    emit('saved');
  } catch (e: any) {
    error.value = `Import failed: ${e}`;
  } finally {
    isLoading.value = false;
  }
};

/** Map a raw request from GitHub data to ImportRequest format. */
const mapRequest = (r: any) => ({
  id: null,
  name: r.name || 'Untitled',
  method: r.method || 'GET',
  url: r.url || '',
  headers: typeof r.headers === 'string' ? r.headers : JSON.stringify(r.headers || []),
  params: typeof r.params === 'string' ? r.params : JSON.stringify(r.params || []),
  body: typeof r.body === 'string' ? r.body : JSON.stringify(r.body || { type: 'None' }),
  auth: typeof r.auth === 'string' ? r.auth : JSON.stringify(r.auth || { type: 'None' }),
});

/** Count total requests in a collection (root + inside folders). */
const countRequests = (col: any): number => {
  const rootCount = col.requests?.length || 0;
  const folderCount = (col.folders || []).reduce(
    (sum: number, f: any) => sum + (f.requests?.length || 0), 0
  );
  return rootCount + folderCount;
};

const close = () => {
  view.value = 'settings';
  error.value = '';
  pullData.value = null;
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
  animation: fadeIn 0.15s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.modal-container {
  background: #1e1e1e;
  width: 100%;
  max-width: 520px;
  border-radius: 12px;
  border: 1px solid #333;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
  animation: slideUp 0.15s ease-out;
  overflow: hidden;
}

@keyframes slideUp {
  from { transform: translateY(20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid #333;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.github-icon {
  color: #fff;
}

.modal-header h3 {
  margin: 0;
  color: #fff;
  font-size: 18px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  color: #888;
  font-size: 24px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #2a2a2a;
  color: #fff;
}

.modal-body {
  padding: 24px;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  color: #aaa;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 500;
}

.form-input {
  width: 100%;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  color: #fff;
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s;
}

.form-input:focus {
  outline: none;
  border-color: #667eea;
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
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
}

.toggle-token:hover {
  background: #444;
  color: #fff;
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
  padding: 12px;
  border-radius: 6px;
  font-size: 13px;
  margin-top: 10px;
  border: 1px solid rgba(255, 77, 77, 0.2);
}

.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid #333;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.cancel-btn {
  background: transparent;
  border: 1px solid #444;
  color: #ccc;
  padding: 9px 18px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
}

.cancel-btn:hover {
  background: #2a2a2a;
  color: #fff;
}

.push-btn, .pull-btn, .import-btn {
  border: none;
  color: #fff;
  padding: 9px 20px;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
}

.push-btn {
  background: #667eea;
}

.push-btn:hover:not(:disabled) {
  background: #7b93ff;
  transform: translateY(-1px);
}

.pull-btn {
  background: #48bb78;
}

.pull-btn:hover:not(:disabled) {
  background: #5ccf8b;
  transform: translateY(-1px);
}

.import-btn {
  background: #48bb78;
}

.import-btn:hover:not(:disabled) {
  background: #5ccf8b;
  transform: translateY(-1px);
}

.push-btn:disabled, .pull-btn:disabled, .import-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

/* ── Pull Preview ──────────────────────────────── */
.pull-header {
  margin-bottom: 20px;
}

.pull-source {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #aaa;
}

.pull-source code {
  background: #2a2a2a;
  padding: 4px 8px;
  border-radius: 4px;
  color: #e0e0e0;
  font-size: 12px;
}

.branch-badge {
  background: rgba(102, 126, 234, 0.15);
  color: #667eea;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
}

.pull-preview h4 {
  margin: 0 0 12px 0;
  color: #ddd;
  font-size: 14px;
  font-weight: 500;
}

.collections-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 260px;
  overflow-y: auto;
}

.collection-preview {
  background: #252525;
  border: 1px solid #3a3a3a;
  border-radius: 8px;
  padding: 14px;
  transition: border-color 0.2s;
}

.collection-preview:hover {
  border-color: #4a4a4a;
}

.collection-preview-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.col-name {
  font-weight: 600;
  color: #fff;
  font-size: 14px;
}

.collection-stats {
  display: flex;
  gap: 16px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #888;
}

.warning-box {
  margin-top: 16px;
  padding: 12px;
  background: rgba(255, 193, 7, 0.08);
  border: 1px solid rgba(255, 193, 7, 0.2);
  border-radius: 6px;
  color: #ffc107;
  font-size: 12px;
  display: flex;
  align-items: flex-start;
  gap: 8px;
  line-height: 1.5;
}

.warning-box svg {
  flex-shrink: 0;
  margin-top: 1px;
}

/* ── Success View ──────────────────────────────── */
.success-body {
  text-align: center;
  padding: 40px 24px;
}

.success-icon {
  margin-bottom: 16px;
  animation: popIn 0.4s ease-out;
}

@keyframes popIn {
  from { transform: scale(0.5); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

.success-title {
  margin: 0 0 8px 0;
  color: #48bb78;
  font-size: 20px;
  font-weight: 600;
}

.success-subtitle {
  margin: 0;
  color: #888;
  font-size: 13px;
}
</style>
