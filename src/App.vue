<template>
  <div class="app">
    <Titlebar 
      ref="titlebarRef"
      :workspace-id="activeWorkspaceId"
      @open-env-settings="showEnvModal = true"
      @env-changed="handleEnvChanged"
      @toggle-history="toggleHistory"
    />
    
    <div class="main-content">
      <!-- Outer: Vertical split (Sidebar LEFT | Work area RIGHT) -->
      <Splitpanes class="default-theme">
        <!-- Sidebar (LEFT) -->
        <Pane :size="20" :min-size="10" :max-size="40">
          <CollectionsSidebar 
            ref="sidebarRef"
            v-model:workspace-id="activeWorkspaceId"
            @request-selected="handleRequestSelected"
            @history-selected="handleHistorySelected"
          />
        </Pane>
        
        <!-- Main work area (RIGHT) -->
        <Pane>
          <!-- Inner: Horizontal split (Request TOP | Response BOTTOM) -->
          <Splitpanes horizontal class="default-theme">
            <!-- Request Panel (TOP) -->
            <Pane :size="50" :min-size="20">
              <!-- Empty State: No Tabs -->
              <div v-if="tabs.length === 0" class="empty-state">
                <div class="empty-state-content">
                  <div class="empty-icon">
                    <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                      <rect x="3" y="3" width="18" height="18" rx="2"/>
                      <line x1="12" y1="8" x2="12" y2="16"/>
                      <line x1="8" y1="12" x2="16" y2="12"/>
                    </svg>
                  </div>
                  <h2>No requests open</h2>
                  <p>Create a new request to get started</p>
                  <button class="action-btn" @click="createNewTab">
                    New Request
                  </button>
                </div>
              </div>

              <!-- Active Request UI -->
              <template v-else>
                <RequestTabs 
                   :tabs="tabs" 
                   :active-tab-id="activeTabId"
                   @switch-tab="switchTab"
                   @close-tab="closeTab"
                   @new-tab="createNewTab"
                />
                <RequestBuilder 
                  ref="requestBuilderRef"
                  :workspace-id="activeWorkspaceId"
                  @response-received="handleResponse"
                  @request-error="handleError"
                  @request-saved="handleRequestUpdated"
                  @change="handleRequestChange"
                />
              </template>
            </Pane>
            
            <!-- Response Panel (BOTTOM) -->
            <Pane :size="50" :min-size="20">
              <ResponseViewer ref="responseViewerRef" />
            </Pane>
          </Splitpanes>
        </Pane>
      </Splitpanes>
    </div>

    <EnvironmentModal 
      :is-open="showEnvModal"
      :workspace-id="activeWorkspaceId"
      @close="showEnvModal = false"
      @updated="handleEnvUpdated"
      @data-cleared="handleDataCleared"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import { Splitpanes, Pane } from 'splitpanes';
import 'splitpanes/dist/splitpanes.css';

import Titlebar from './components/Titlebar.vue';
import CollectionsSidebar from './components/CollectionsSidebar.vue';
import RequestBuilder from './components/RequestBuilder.vue';
import ResponseViewer from './components/ResponseViewer.vue';
import EnvironmentModal from './components/EnvironmentModal.vue';
import RequestTabs from "./components/RequestTabs.vue";

const titlebarRef = ref<InstanceType<typeof Titlebar>>();
const requestBuilderRef = ref<InstanceType<typeof RequestBuilder> | null>(null);
const responseViewerRef = ref<InstanceType<typeof ResponseViewer>>();
const sidebarRef = ref<InstanceType<typeof CollectionsSidebar> | null>(null);

const activeWorkspaceId = ref('default');
const showEnvModal = ref(false);

const workspaces = ref<any[]>([]);

// Tab Management
interface Tab {
  id: string; // Internal tab ID
  requestId: string | null; // DB ID if saved
  name: string;
  method: string;
  isDirty: boolean;
  data: any; // Full request data
}

const tabs = ref<Tab[]>([]);
const activeTabId = ref<string | null>(null);

const createNewTab = () => {
  const newTab: Tab = {
    id: crypto.randomUUID(),
    requestId: null,
    name: 'New Request',
    method: 'GET',
    isDirty: false,
    data: {
      method: 'GET',
      url: '',
      headers: [],
      params: [],
      body: { type: 'none', content: '' },
      auth: { type: 'none' }
    }
  };
  tabs.value.push(newTab);
  switchTab(newTab.id);
};

const switchTab = async (tabId: string) => {
  console.log('Switching to tab:', tabId);
  activeTabId.value = tabId;
  
  // Wait for next tick to ensure component is mounted
  await nextTick();
  
  const tab = tabs.value.find(t => t.id === tabId);
  if (tab) {
      if (requestBuilderRef.value) {
        console.log('Loading request into builder:', tab.name);
        const requestData = {
            ...tab.data,
            _isCollection: !!tab.requestId,
            id: tab.requestId,
            name: tab.name
        };
        requestBuilderRef.value.loadRequest(requestData);
      } else {
        console.error('requestBuilderRef is null when switching tab');
      }
  } else {
      console.error('Tab not found:', tabId);
  }
};

const closeTab = (tabId: string) => {
  const tab = tabs.value.find(t => t.id === tabId);
  if (!tab) return;
  
  if (tab.isDirty) {
    if (!confirm(`Discard unsaved changes in "${tab.name}"?`)) {
        return;
    }
  }
  
  const index = tabs.value.findIndex(t => t.id === tabId);
  tabs.value.splice(index, 1);
  
  if (activeTabId.value === tabId) {
    if (tabs.value.length > 0) {
        const newIndex = Math.min(index, tabs.value.length - 1);
        switchTab(tabs.value[newIndex].id);
    } else {
        activeTabId.value = null;
        // Show empty state instead of creating a new tab
    }
  }
};

const handleRequestChange = (data: any) => {
  const tab = tabs.value.find(t => t.id === activeTabId.value);
  if (tab) {
    tab.data = { ...tab.data, ...data };
    tab.method = data.method;
    if (!tab.isDirty) tab.isDirty = true;
  }
};

const handleRequestUpdated = (data: { id: string, name: string }) => {
    const tab = tabs.value.find(t => t.id === activeTabId.value);
    if (tab) {
        tab.requestId = data.id;
        tab.name = data.name;
        tab.isDirty = false;
        tab.data.id = data.id;
        tab.data.name = data.name;
    }
    refreshSidebar();
};

const toggleHistory = () => {
    sidebarRef.value?.switchTab('history');
};

const handleRequestSelected = (request: any) => {
  console.log('App: Request selected from sidebar', request);
  const existingTab = tabs.value.find(t => t.requestId === request.id);
  if (existingTab) {
      console.log('Switching to existing tab for request', request.id);
      switchTab(existingTab.id);
      return;
  }
  
  console.log('Opening new tab for request', request.name);
  const newTab: Tab = {
      id: crypto.randomUUID(),
      requestId: request.id,
      name: request.name,
      method: request.method,
      isDirty: false,
      data: request
  };
  tabs.value.push(newTab);
  switchTab(newTab.id);
};

const handleHistorySelected = (historyItem: any) => {
    console.log('App: History item selected', historyItem);
    const newTab: Tab = {
        id: crypto.randomUUID(),
        requestId: null,
        name: `${historyItem.method} ${historyItem.url}`.substring(0, 30) || 'History Request',
        method: historyItem.method,
        isDirty: true,
        data: {
            ...historyItem,
            headers: historyItem.request_headers,
            params: historyItem.request_params,
            body: JSON.stringify({
                type: 'raw', 
                content: historyItem.request_body || ''
            }),
            _isHistory: true,
            _originalRequestId: historyItem.request_id
        }
    };
    tabs.value.push(newTab);
    switchTab(newTab.id);
};

const refreshSidebar = () => {
    sidebarRef.value?.refresh();
};

const handleEnvChanged = () => {
  console.log('Environment changed');
  requestBuilderRef.value?.fetchActiveVariables();
};

const handleEnvUpdated = () => {
  titlebarRef.value?.refresh();
};

const handleDataCleared = async () => {
  tabs.value = [];
  activeTabId.value = null;
  await fetchWorkspaces();
  sidebarRef.value?.refresh();
};

const handleResponse = (response: any) => {
  responseViewerRef.value?.handleResponse(response);
  if (response.status === 'Success') {
      sidebarRef.value?.refresh(); 
  }
};

const handleError = (error: any) => {
  responseViewerRef.value?.handleError(error);
};

const fetchWorkspaces = async () => {
  try {
    const result = await invoke('get_workspaces');
    workspaces.value = result as any[];
    if (workspaces.value.length > 0 && !activeWorkspaceId.value) {
      activeWorkspaceId.value = workspaces.value[0].id;
    } else if (workspaces.value.length === 0) {
      const id = await invoke('create_workspace', { name: 'My Workspace' });
      activeWorkspaceId.value = id as string;
      await fetchWorkspaces();
    }
  } catch (error) {
    console.error('Failed to fetch workspaces:', error);
  }
};

onMounted(async () => {
  await fetchWorkspaces();
});
</script>


<style scoped>
.app {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #0a0a0a;
  overflow: hidden;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Empty State */
.empty-state {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #0a0a0a;
}

.empty-state-content {
  text-align: center;
  max-width: 360px;
}

.empty-icon {
  margin-bottom: 24px;
  opacity: 0.4;
}

.empty-state-content h2 {
  font-size: 20px;
  font-weight: 500;
  color: #e0e0e0;
  margin: 0 0 8px 0;
}

.empty-state-content p {
  font-size: 14px;
  color: #666;
  margin: 0 0 24px 0;
}

.action-btn {
  padding: 10px 20px;
  background: #1a1a1a;
  color: #e0e0e0;
  border: 1px solid #333;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: #252525;
  border-color: #444;
}

.action-btn:active {
  background: #1a1a1a;
}

/* Override Splitpanes default theme to match our dark theme */
:deep(.splitpanes.default-theme .splitpanes__pane) {
  background: transparent;
}

:deep(.splitpanes.default-theme .splitpanes__splitter) {
  background: #2a2a2a;
  position: relative;
}

:deep(.splitpanes.default-theme .splitpanes__splitter:hover) {
  background: #667eea;
}

:deep(.splitpanes.default-theme .splitpanes__splitter:before) {
  background: transparent;
}

:deep(.splitpanes.default-theme .splitpanes__splitter:after) {
  background: transparent;
}
</style>
