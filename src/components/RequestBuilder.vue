<template>
  <div class="request-builder">
    <div class="request-header">
      <div class="method-url-section">
        <select v-model="method" class="method-select">
          <option value="GET">GET</option>
          <option value="POST">POST</option>
          <option value="PUT">PUT</option>
          <option value="PATCH">PATCH</option>
          <option value="DELETE">DELETE</option>
          <option value="HEAD">HEAD</option>
          <option value="OPTIONS">OPTIONS</option>
        </select>
        
        <input 
          ref="urlInputRef"
          v-model="url" 
          type="text" 
          class="url-input" 
          placeholder="Enter request URL (e.g., https://api.example.com/users)"
          @keydown.ctrl.enter="sendRequest"
          @keydown.ctrl.s.prevent="saveCurrentRequest"
          @mousemove="handleUrlMouseMove"
          @mouseleave="hideVariableTooltip"
        />
        
        <button 
          class="import-curl-btn" 
          @click="showImportModal = true"
          title="Import from cURL"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M4 17l6-6-6-6M12 19h8"/>
          </svg>
          import
        </button>

        <button 
          class="save-btn" 
          @click="saveCurrentRequest"
          title="Save request (Ctrl+S)"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
            <path d="M0 1a1 1 0 011-1h9l3 3v9a1 1 0 01-1 1H1a1 1 0 01-1-1V1z"/>
            <path d="M3 0v4h6V0M4 8h6v6H4z" fill="#1a1a1a"/>
          </svg>
        </button>
        
        <button 
          v-if="!isSending"
          class="send-btn" 
          @click="sendRequest" 
          :disabled="!url"
        >
          Send
        </button>
        
        <button 
          v-else
          class="cancel-btn" 
          @click="cancelRequest"
          title="Cancel request (Ctrl+.)"
        >
          <span class="loading-spinner"></span>
          Cancel
        </button>
      </div>
    </div>

    <div class="request-tabs">
      <button 
        v-for="tab in tabs" 
        :key="tab.id"
        :class="['tab-btn', { active: activeTab === tab.id }]"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>

    <div class="request-body">
      <!-- Headers Tab -->
      <div v-if="activeTab === 'headers'" class="tab-content">
        <div class="kv-list">
          <div v-for="(header, index) in headers" :key="index" class="kv-row">
            <input type="checkbox" v-model="header.enabled" class="kv-checkbox" />
            <input v-model="header.key" placeholder="Header name" class="kv-input" @mousemove="handleKVMouseMove" @mouseleave="hideVariableTooltip" />
            <input v-model="header.value" placeholder="Value" class="kv-input" @mousemove="handleKVMouseMove" @mouseleave="hideVariableTooltip" />
            <button @click="removeHeader(index)" class="remove-btn">×</button>
          </div>
          <button @click="addHeader" class="add-btn">+ Add Header</button>
        </div>
      </div>

      <!-- Params Tab -->
      <div v-if="activeTab === 'params'" class="tab-content">
        <div class="kv-list">
          <div v-for="(param, index) in params" :key="index" class="kv-row">
            <input type="checkbox" v-model="param.enabled" class="kv-checkbox" />
            <input v-model="param.key" placeholder="Parameter name" class="kv-input" @mousemove="handleKVMouseMove" @mouseleave="hideVariableTooltip" />
            <input v-model="param.value" placeholder="Value" class="kv-input" @mousemove="handleKVMouseMove" @mouseleave="hideVariableTooltip" />
            <button @click="removeParam(index)" class="remove-btn">×</button>
          </div>
          <button @click="addParam" class="add-btn">+ Add Parameter</button>
        </div>
      </div>

      <!-- Body Tab -->
      <div v-if="activeTab === 'body'" class="tab-content">
        <div class="body-type-selector">
          <button 
            v-for="type in bodyTypes" 
            :key="type.id"
            :class="['body-type-btn', { active: bodyType === type.id }]"
            @click="bodyType = type.id"
          >
            {{ type.label }}
          </button>
        </div>

        <div class="body-editor">
          <textarea 
            v-if="bodyType === 'raw' || bodyType === 'json'"
            v-model="bodyContent"
            class="body-textarea"
            @mousemove="handleKVMouseMove"
            @mouseleave="hideVariableTooltip"
            :placeholder="getBodyPlaceholder"
          ></textarea>
          
          <div v-else-if="bodyType === 'none'" class="empty-state">
            No body for this request
          </div>
        </div>
      </div>

      <!-- Auth Tab -->
      <div v-if="activeTab === 'auth'" class="tab-content">
        <div class="auth-selector">
          <select v-model="authType" class="auth-select">
            <option value="none">No Auth</option>
            <option value="basic">Basic Auth</option>
            <option value="bearer">Bearer Token</option>
          </select>
        </div>

        <div v-if="authType === 'basic'" class="auth-fields">
          <input v-model="authUsername" placeholder="Username" class="auth-input" />
          <input v-model="authPassword" type="password" placeholder="Password" class="auth-input" />
        </div>

        <div v-if="authType === 'bearer'" class="auth-fields">
          <input v-model="authToken" placeholder="Token" class="auth-input" />
        </div>
      </div>
    </div>

    <!-- Save Request Modal -->
    <SaveRequestModal
      :is-open="showSaveModal"
      :collections="collections"
      @close="showSaveModal = false"
      @save="handleSaveRequest"
      @create-collection="handleCreateCollection"
      @create-folder="handleCreateFolder"
    />

    <!-- Import cURL Modal -->
    <ImportCurlModal
      :is-open="showImportModal"
      @confirm="handleImportCurl"
      @cancel="showImportModal = false"
    />

    <!-- Variable Tooltip -->
    <VariableTooltip
      :variable-name="hoveredVariable"
      :current-value="hoveredVariableValue"
      :position="tooltipPosition"
      :visible="tooltipVisible"
      :environment-id="activeEnvironmentId"
      :workspace-id="props.workspaceId"
      @close="closeVariableTooltip"
      @updated="handleVariableUpdated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SaveRequestModal from './SaveRequestModal.vue';
import ImportCurlModal from './ImportCurlModal.vue';
import VariableTooltip from './VariableTooltip.vue';
import { parseCurl } from '../utils/curlParser';
import { useVariableTooltip } from '../composables/useVariableTooltip';

const props = defineProps<{
  workspaceId: string
}>();

// Emit event to send response to parent
const emit = defineEmits<{
  (e: 'responseReceived', response: any): void;
  (e: 'requestError', error: any): void;
  (e: 'requestSaved', data: { id: string, name: string }): void;
  (e: 'change', data: any): void;
}>();

const isHydrating = ref(false);

// Variable tooltip composable
const {
  tooltipVisible,
  tooltipPosition,
  hoveredVariable,
  hoveredVariableValue,
  handleInputMouseMove,
  closeTooltip
} = useVariableTooltip();

const urlInputRef = ref<HTMLInputElement | null>(null);
const activeVariables = ref<Record<string, string>>({});
const activeEnvironmentId = ref<string>('');

const fetchActiveVariables = async () => {
  try {
    const vars = await invoke<Record<string, string>>('get_active_variables', { 
      workspaceId: props.workspaceId 
    });
    activeVariables.value = vars;
    
    const envs = await invoke<any[]>('get_environments', { 
      workspaceId: props.workspaceId 
    });
    const active = envs.find(e => e.is_active);
    if (active) {
      activeEnvironmentId.value = active.id;
    }
  } catch (error) {
    console.error('Failed to fetch active variables:', error);
  }
};

const handleUrlMouseMove = (event: MouseEvent) => {
  if (urlInputRef.value) {
    handleInputMouseMove(event, urlInputRef.value, activeVariables.value);
  }
};

const handleKVMouseMove = (event: MouseEvent) => {
  const target = event.target as HTMLInputElement | HTMLTextAreaElement;
  if (target) {
    handleInputMouseMove(event, target, activeVariables.value);
  }
};

const hideVariableTooltip = () => {
  // useVariableTooltip handles delay
};

const closeVariableTooltip = () => {
  closeTooltip();
};

const handleVariableUpdated = () => {
  fetchActiveVariables();
};

onMounted(() => {
  fetchCollections();
  fetchActiveVariables();
});

watch(() => props.workspaceId, () => {
  fetchCollections();
  fetchActiveVariables();
});

const notifyChange = () => {
  if (isHydrating.value) return;
  
  const data = {
    method: method.value,
    url: url.value,
    headers: headers.value,
    params: params.value,
    body: { type: bodyType.value, content: bodyContent.value },
    auth: { type: authType.value, username: authUsername.value, password: authPassword.value, token: authToken.value }
  };
  emit('change', data);
};

const method = ref('GET');
const url = ref('https://jsonplaceholder.typicode.com/todos/1');
const isSending = ref(false);
const currentRequestId = ref<string | null>(null);
const showSaveModal = ref(false);
const showImportModal = ref(false);

// Database-backed collections for the save modal
const collections = ref<any[]>([]);

const fetchCollections = async () => {
  try {
    await invoke('ensure_workspace', { workspaceId: props.workspaceId });
    // no need to set props.workspaceId as it's managed by parent
    const result = await invoke('get_collections_with_folders', { workspaceId: props.workspaceId });
    collections.value = result as any[];
  } catch (error) {
    console.error('Failed to fetch collections for save modal:', error);
  }
};

const activeTab = ref('params');

const tabs = [
  { id: 'params', label: 'Params' },
  { id: 'headers', label: 'Headers' },
  { id: 'body', label: 'Body' },
  { id: 'auth', label: 'Auth' },
];

// Headers
const headers = ref([{ key: '', value: '', enabled: true }]);
const addHeader = () => headers.value.push({ key: '', value: '', enabled: true });
const removeHeader = (index: number) => headers.value.splice(index, 1);

// Params
const params = ref([{ key: '', value: '', enabled: true }]);
const addParam = () => params.value.push({ key: '', value: '', enabled: true });
const removeParam = (index: number) => params.value.splice(index, 1);

// Body
const bodyType = ref('none');
const bodyTypes = [
  { id: 'none', label: 'None' },
  { id: 'json', label: 'JSON' },
  { id: 'raw', label: 'Raw' },
];
const bodyContent = ref('');

// Auth
const authType = ref('none');
const authUsername = ref('');
const authPassword = ref('');
const authToken = ref('');

const getBodyPlaceholder = computed(() => {
  if (bodyType.value === 'json') {
    return '{\n  "key": "value"\n}';
  }
  return 'Enter request body...';
});

const sendRequest = async () => {
  if (!url.value || isSending.value) return;
  
  isSending.value = true;
  currentRequestId.value = null;
  
  try {
    // Build request body
    let requestBody: any = { type: 'None' };
    
    if (bodyType.value === 'json') {
      try {
        const jsonValue = JSON.parse(bodyContent.value || '{}');
        requestBody = { type: 'Json', value: jsonValue };
      } catch (e) {
        console.error('Invalid JSON:', e);
        requestBody = { type: 'Raw', mime: 'application/json', content: bodyContent.value };
      }
    } else if (bodyType.value === 'raw') {
      requestBody = { type: 'Raw', mime: 'text/plain', content: bodyContent.value };
    }

    // Build auth config
    let authConfig = null;
    if (authType.value === 'basic') {
      authConfig = { type: 'Basic', username: authUsername.value, password: authPassword.value };
    } else if (authType.value === 'bearer') {
      authConfig = { type: 'Bearer', token: authToken.value };
    }

    // Build request config
    const config = {
      method: method.value,
      url: url.value,
      headers: headers.value.filter(h => h.key && h.enabled),
      params: params.value.filter(p => p.key && p.enabled),
      body: requestBody,
      auth: authConfig,
      timeout_ms: 30000,
      ignore_tls: false,
      follow_redirects: true,
    };

    console.log('Sending request:', config);

    // Call Tauri backend
    const result = await invoke('send_request', { 
      config, 
      workspaceId: props.workspaceId 
    });
    
    // Extract request ID from result
    if (result && typeof result === 'object') {
      if ('request_id' in result) {
        currentRequestId.value = (result as any).request_id;
      }
    }
    
    console.log('Response received:', result);
    
    // Emit response to parent
    emit('responseReceived', result);
    
  } catch (error) {
    console.error('Request failed:', error);
    emit('requestError', error);
  } finally {
    isSending.value = false;
    currentRequestId.value = null;
  }
};

const cancelRequest = async () => {
  if (!currentRequestId.value) {
    // If we don't have a request ID yet, just reset the state
    isSending.value = false;
    return;
  }

  try {
    console.log('Cancelling request:', currentRequestId.value);
    const cancelled = await invoke('cancel_request', { requestId: currentRequestId.value });
    console.log('Cancel result:', cancelled);
  } catch (error) {
    console.error('Cancel failed:', error);
  } finally {
    isSending.value = false;
    currentRequestId.value = null;
  }
};

const savedRequestId = ref<string | null>(null);
const requestName = ref('');

// Method to load a request from collections or history
const loadRequest = (request: any) => {
  isHydrating.value = true;
  console.log('Hydrating request:', request);
  
  try {
      method.value = request.method || 'GET';
      url.value = request.url || '';
      
      if (request._isCollection) {
        savedRequestId.value = request.id;
        requestName.value = request.name;
      } else if (request._isHistory && request._originalRequestId) {
        savedRequestId.value = request._originalRequestId;
        savedRequestId.value = null;
        requestName.value = '';
      } else {
        savedRequestId.value = null;
        requestName.value = '';
      }

      // Hydrate Headers - handle both string (old) and object (new) formats
      try {
        const savedHeaders = typeof request.headers === 'string' 
          ? JSON.parse(request.headers || '[]')
          : Array.isArray(request.headers) ? request.headers : [];
        headers.value = savedHeaders.length > 0 ? savedHeaders : [{ key: '', value: '', enabled: true }];
      } catch (e) {
        console.error('Failed to parse headers:', e);
        headers.value = [{ key: '', value: '', enabled: true }];
      }

      // Hydrate Params - handle both string (old) and object (new) formats
      try {
        const savedParams = typeof request.params === 'string'
          ? JSON.parse(request.params || '[]')
          : Array.isArray(request.params) ? request.params : [];
        params.value = savedParams.length > 0 ? savedParams : [{ key: '', value: '', enabled: true }];
      } catch (e) {
        console.error('Failed to parse params:', e);
        params.value = [{ key: '', value: '', enabled: true }];
      }

      // Hydrate Body - handle both string (old) and object (new) formats
      try {
        let bodyData;
        if (typeof request.body === 'string') {
            bodyData = JSON.parse(request.body || '{"type":"none","content":""}');
        } else {
            bodyData = request.body && typeof request.body === 'object' ? request.body : { type: 'none', content: '' };
        }
        bodyType.value = bodyData.type || 'none';
        bodyContent.value = bodyData.content || '';
        if (bodyType.value !== 'none') {
            activeTab.value = 'body';
        }
      } catch (e) {
        console.error('Failed to parse body:', e);
        bodyType.value = 'none';
        bodyContent.value = '';
      }

      // Hydrate Auth - handle both string (old) and object (new) formats
      try {
        let authData;
        if (typeof request.auth === 'string') {
            authData = JSON.parse(request.auth || '{"type":"none"}');
        } else {
            authData = request.auth && typeof request.auth === 'object' ? request.auth : { type: 'none' };
        }
        authType.value = authData.type || 'none';
        authUsername.value = authData.username || '';
        authPassword.value = authData.password || '';
        authToken.value = authData.token || '';
      } catch (e) {
        console.error('Failed to parse auth:', e);
        authType.value = 'none';
      }
  } finally {
      // Use setTimeout to ensure watchers have fired (and been ignored) before resetting flag
      setTimeout(() => {
          isHydrating.value = false;
      }, 0);
  }
};

// Save request handlers
const openSaveModal = async () => {
  await fetchCollections();
  showSaveModal.value = true;
};

const saveCurrentRequest = async () => {
    if (savedRequestId.value && requestName.value) {
        // Update existing request
        try {
            const requestData = {
                id: savedRequestId.value,
                name: requestName.value,
                method: method.value,
                url: url.value,
                headers: JSON.stringify(headers.value.filter(h => h.key && h.enabled)),
                params: JSON.stringify(params.value.filter(p => p.key && p.enabled)),
                body: JSON.stringify({
                    type: bodyType.value,
                    content: bodyContent.value
                }),
                auth: JSON.stringify({
                    type: authType.value,
                    username: authUsername.value,
                    password: authPassword.value,
                    token: authToken.value
                })
            };

            await invoke('update_request', requestData);
            console.log('Request updated successfully');
            emit('requestSaved', { id: savedRequestId.value as string, name: requestName.value }); // Refresh sidebar
        } catch (error) {
            console.error('Failed to update request:', error);
            // Fallback to modal if fails?
            openSaveModal();
        }
    } else {
        // New request or ambiguous state
        openSaveModal();
    }
};

const handleSaveRequest = async (data: any) => {
  try {
    const requestData = {
      name: data.name,
      method: method.value,
      url: url.value,
      collection_id: data.collectionId,
      folder_id: data.folderId,
      headers: JSON.stringify(headers.value.filter(h => h.key && h.enabled)),
      params: JSON.stringify(params.value.filter(p => p.key && p.enabled)),
      body: JSON.stringify({
        type: bodyType.value,
        content: bodyContent.value
      }),
      auth: JSON.stringify({
        type: authType.value,
        username: authUsername.value,
        password: authPassword.value,
        token: authToken.value
      })
    };

    const invokeData = {
      name: requestData.name,
      method: requestData.method,
      url: requestData.url,
      collectionId: data.collectionId,
      folderId: data.folderId,
      headers: requestData.headers,
      params: requestData.params,
      body: requestData.body,
      auth: requestData.auth
    };

    const newId = await invoke('save_request', invokeData);
    console.log('Request saved successfully');
    
    // Update local state to treat as saved
    savedRequestId.value = newId as string;
    requestName.value = requestData.name;
    
    emit('requestSaved', { id: newId as string, name: requestData.name });
  } catch (error) {
    console.error('Failed to save request:', error);
  }
};

const handleCreateCollection = async (name: string) => {
  try {
    await invoke('create_collection', { name, workspaceId: props.workspaceId });
    await fetchCollections();
  } catch (error) {
    console.error('Failed to create collection:', error);
  }
};

const handleCreateFolder = async (data: { collectionId: string; name: string }) => {
  try {
    await invoke('create_folder', { name: data.name, collectionId: data.collectionId });
    await fetchCollections();
  } catch (error) {
    console.error('Failed to create folder:', error);
  }
};

const handleImportCurl = (command: string) => {
  try {
    const parsed = parseCurl(command);
    
    method.value = parsed.method;
    url.value = parsed.url;
    
    if (parsed.headers.length > 0) {
      headers.value = parsed.headers;
    } else {
      headers.value = [{ key: '', value: '', enabled: true }];
    }
    
    if (parsed.params.length > 0) {
      params.value = parsed.params;
    } else {
      params.value = [{ key: '', value: '', enabled: true }];
    }
    
    if (parsed.body) {
      bodyType.value = parsed.bodyType;
      bodyContent.value = parsed.body;
      activeTab.value = 'body';
    } else {
      bodyType.value = 'none';
      bodyContent.value = '';
    }
    
    showImportModal.value = false;
  } catch (error) {
    console.error('Failed to import cURL:', error);
  }
};

// Watchers for change detection
watch([method, url, bodyType, bodyContent, authType, authUsername, authPassword, authToken], notifyChange);
watch(headers, notifyChange, { deep: true });
watch(params, notifyChange, { deep: true });

// Expose methods for parent component
defineExpose({
  loadRequest,
  fetchActiveVariables
});
</script>

<style scoped>
.request-builder {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1a1a1a;
}

.request-header {
  padding: 16px;
  border-bottom: 1px solid #2a2a2a;
}

.method-url-section {
  display: flex;
  gap: 8px;
}

.method-select {
  width: 120px;
  height: 38px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #fff;
  padding: 0 12px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.method-select:hover {
  border-color: #667eea;
}

.method-select:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.url-input {
  flex: 1;
  height: 38px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #fff;
  padding: 0 16px;
  font-size: 13px;
  transition: all 0.2s;
}

.url-input:hover {
  border-color: #4a4a4a;
}

.url-input:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.url-input::placeholder {
  color: #666;
}

.import-curl-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 16px;
  height: 38px;
  background: rgba(102, 126, 234, 0.1);
  border: 1px solid rgba(102, 126, 234, 0.3);
  border-radius: 6px;
  color: #667eea;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.import-curl-btn:hover {
  background: rgba(102, 126, 234, 0.15);
  border-color: #667eea;
  transform: translateY(-1px);
}

.save-btn {
  width: 50px;
  height: 38px;
  background: transparent;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #888;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.save-btn:hover {
  border-color: #667eea;
  color: #667eea;
  background: rgba(102, 126, 234, 0.05);
}

.save-btn svg {
  flex-shrink: 0;
}

.send-btn {
  width: 100px;
  height: 38px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  border-radius: 6px;
  color: white;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.send-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.send-btn:active:not(:disabled) {
  transform: translateY(0);
}

.send-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.cancel-btn {
  width: 120px;
  height: 38px;
  background: linear-gradient(135deg, #ff6b6b 0%, #ee5253 100%);
  border: none;
  border-radius: 6px;
  color: white;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.cancel-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(255, 107, 107, 0.4);
}

.cancel-btn:active {
  transform: translateY(0);
}

.loading-spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.request-tabs {
  display: flex;
  gap: 4px;
  padding: 0 16px;
  border-bottom: 1px solid #2a2a2a;
  background: #151515;
}

.tab-btn {
  padding: 12px 20px;
  background: transparent;
  border: none;
  color: #888;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
  position: relative;
}

.tab-btn:hover {
  color: #aaa;
}

.tab-btn.active {
  color: #667eea;
  border-bottom-color: #667eea;
}

.request-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.tab-content {
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

.kv-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.kv-row {
  display: grid;
  grid-template-columns: auto 1fr 1fr auto;
  gap: 8px;
  align-items: center;
}

.kv-checkbox {
  width: 18px;
  height: 18px;
  cursor: pointer;
  accent-color: #667eea;
}

.kv-input {
  height: 36px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  color: #fff;
  padding: 0 12px;
  font-size: 13px;
  transition: all 0.2s;
}

.kv-input:hover {
  border-color: #4a4a4a;
}

.kv-input:focus {
  outline: none;
  border-color: #667eea;
}

.kv-input::placeholder {
  color: #555;
}

.remove-btn {
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  color: #888;
  font-size: 20px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.remove-btn:hover {
  background: #dc3545;
  color: white;
}

.add-btn {
  padding: 10px 16px;
  background: #2a2a2a;
  border: 1px dashed #4a4a4a;
  border-radius: 4px;
  color: #888;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  align-self: flex-start;
}

.add-btn:hover {
  border-color: #667eea;
  color: #667eea;
  background: rgba(102, 126, 234, 0.05);
}

.body-type-selector {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.body-type-btn {
  padding: 8px 16px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  color: #888;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.body-type-btn:hover {
  border-color: #4a4a4a;
  color: #aaa;
}

.body-type-btn.active {
  background: #667eea;
  border-color: #667eea;
  color: white;
}

.body-editor {
  flex: 1;
}

.body-textarea {
  width: 100%;
  min-height: 300px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #fff;
  padding: 16px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: vertical;
  transition: all 0.2s;
}

.body-textarea:hover {
  border-color: #4a4a4a;
}

.body-textarea:focus {
  outline: none;
  border-color: #667eea;
}

.body-textarea::placeholder {
  color: #555;
}

.empty-state {
  padding: 40px;
  text-align: center;
  color: #666;
  font-size: 14px;
}

.auth-selector {
  margin-bottom: 16px;
}

.auth-select {
  width: 200px;
  height: 38px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #fff;
  padding: 0 12px;
  font-size: 13px;
  cursor: pointer;
}

.auth-select:hover {
  border-color: #4a4a4a;
}

.auth-select:focus {
  outline: none;
  border-color: #667eea;
}

.auth-fields {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 400px;
}

.auth-input {
  height: 38px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #fff;
  padding: 0 16px;
  font-size: 13px;
  transition: all 0.2s;
}

.auth-input:hover {
  border-color: #4a4a4a;
}

.auth-input:focus {
  outline: none;
  border-color: #667eea;
}

.auth-input::placeholder {
  color: #555;
}
</style>
