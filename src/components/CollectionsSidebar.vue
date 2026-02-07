<template>
  <div :class="['collections-sidebar', { collapsed: isCollapsed }]">
    <!-- Sidebar Header -->
    <div class="sidebar-header">
      <div class="header-content">
        <h2 v-if="!isCollapsed">Collections</h2>
        <button class="collapse-btn" @click="toggleCollapse" :title="isCollapsed ? 'Expand' : 'Collapse'">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path v-if="isCollapsed" d="M6 12l4-4-4-4v8z"/>
            <path v-else d="M10 6l-4 4-4-4h8z"/>
          </svg>
        </button>
      </div>
      <div class="sidebar-actions">
        <button class="refresh-btn" @click="fetchCollections" :class="{ 'rotating': isLoading }" title="Refresh">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 3a5 5 0 104.546 2.914.5.5 0 01.908-.417A6 6 0 118 2v1z"/>
            <path d="M8 4.466V.534a.25.25 0 01.41-.192l2.36 1.966c.12.1.12.284 0 .384L8.41 4.658a.25.25 0 01-.41-.192z"/>
          </svg>
        </button>
        <button v-if="!isCollapsed" class="new-btn" @click="showAddSelection = true" title="New Collection">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
            <path d="M7 0v14M0 7h14" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Sidebar Tabs -->
    <div v-if="!isCollapsed" class="sidebar-tabs">
      <button 
        :class="['tab-btn', { active: activeTab === 'collections' }]" 
        @click="activeTab = 'collections'"
      >
        Collections
      </button>
      <button 
        :class="['tab-btn', { active: activeTab === 'history' }]" 
        @click="activeTab = 'history'"
      >
        History
      </button>
    </div>

    <!-- Sidebar Content -->
    <div v-if="!isCollapsed" class="sidebar-body">
      <!-- Collections View -->
      <div v-show="activeTab === 'collections'" class="sidebar-content">
        <!-- Workspace Selector -->
        <div class="workspace-section">
          <div class="workspace-header">
            <select v-model="localWorkspaceId" class="workspace-select">
              <option v-for="ws in workspaces" :key="ws.id" :value="ws.id">
                {{ ws.name }}
              </option>
            </select>
            <button class="workspace-add-btn" @click="handleCreateWorkspace" title="New Workspace">
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/>
              </svg>
            </button>
            <button class="workspace-edit-btn" @click="handleRenameWorkspace" title="Rename Workspace">
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                <path d="M12.146.146a.5.5 0 0 1 .708 0l3 3a.5.5 0 0 1 0 .708l-10 10a.5.5 0 0 1-.168.11l-5 2a.5.5 0 0 1-.65-.65l2-5a.5.5 0 0 1 .11-.168l10-10zM11.207 2.5 13.5 4.793 14.793 3.5 12.5 1.207 11.207 2.5zm1.586 3L10.5 3.207 4 9.707V10h.5a.5.5 0 0 1 .5.5v.5h.5a.5.5 0 0 1 .5.5v.5h.293l6.5-6.5zm-9.761 5.175-.106.106-1.528 3.821 3.821-1.528.106-.106A.5.5 0 0 1 5 12.5V12h-.5a.5.5 0 0 1-.5-.5V11h-.5a.5.5 0 0 1-.468-.325z"/>
              </svg>
            </button>
            <button v-if="localWorkspaceId !== 'default'" class="workspace-delete-btn" @click="handleDeleteWorkspace" title="Delete Workspace">
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                <path d="M11 1.5v1h3.5a.5.5 0 0 1 0 1h-.538l-.853 10.66A2 2 0 0 1 11.115 16h-6.23a2 2 0 0 1-1.994-1.84L2.038 3.5H1.5a.5.5 0 0 1 0-1H5v-1A1.5 1.5 0 0 1 6.5 0h3A1.5 1.5 0 0 1 11 1.5zm-5 0v1h4v-1a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5z"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Collections Tree -->
        <div class="collections-tree">
          <div v-for="collection in collections" :key="collection.id" class="collection-item">
            <div class="collection-header" @click="toggleCollection(collection.id)">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor" class="expand-icon">
                <path v-if="expandedCollections.includes(collection.id)" d="M2 4l4 4 4-4H2z"/>
                <path v-else d="M4 2l4 4-4 4V2z"/>
              </svg>
              <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor" class="folder-icon">
                <path d="M0 2a1 1 0 011-1h4l1 2h7a1 1 0 011 1v8a1 1 0 01-1 1H1a1 1 0 01-1-1V2z"/>
              </svg>
              <span class="collection-name">{{ collection.name }}</span>
              <div class="collection-actions">
                <button class="action-btn" @click.stop="addFolder(collection.id)" title="Add Folder">
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
                    <path d="M6 0v12M0 6h12" stroke="currentColor" stroke-width="1.5"/>
                  </svg>
                </button>
                <button class="action-btn delete-btn" @click.stop="handleDeleteCollection(collection.id, collection.name)" title="Delete Collection">
                  <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M11 1.5v1h3.5a.5.5 0 0 1 0 1h-.538l-.853 10.66A2 2 0 0 1 11.115 16h-6.23a2 2 0 0 1-1.994-1.84L2.038 3.5H1.5a.5.5 0 0 1 0-1H5v-1A1.5 1.5 0 0 1 6.5 0h3A1.5 1.5 0 0 1 11 1.5zm-5 0v1h4v-1a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5z"/>
                  </svg>
                </button>
              </div>
            </div>

            <!-- Folders & Requests -->
            <div v-if="expandedCollections.includes(collection.id)" class="collection-content">
              <!-- Folders -->
              <div v-for="folder in collection.folders" :key="folder.id" class="folder-item">
                <div class="folder-header" @click="toggleFolder(folder.id)">
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor" class="expand-icon">
                    <path v-if="expandedFolders.includes(folder.id)" d="M2 4l4 4 4-4H2z"/>
                    <path v-else d="M4 2l4 4-4 4V2z"/>
                  </svg>
                  <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor" class="folder-icon">
                    <path d="M0 2a1 1 0 011-1h4l1 2h7a1 1 0 011 1v8a1 1 0 01-1 1H1a1 1 0 01-1-1V2z"/>
                  </svg>
                  <span class="folder-name">{{ folder.name }}</span>
                  <div class="folder-actions">
                    <button class="action-btn delete-btn" @click.stop="handleDeleteFolder(folder.id, folder.name)" title="Delete Folder">
                      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                        <path d="M11 1.5v1h3.5a.5.5 0 0 1 0 1h-.538l-.853 10.66A2 2 0 0 1 11.115 16h-6.23a2 2 0 0 1-1.994-1.84L2.038 3.5H1.5a.5.5 0 0 1 0-1H5v-1A1.5 1.5 0 0 1 6.5 0h3A1.5 1.5 0 0 1 11 1.5zm-5 0v1h4v-1a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5z"/>
                      </svg>
                    </button>
                  </div>
                </div>

                <!-- Requests in Folder -->
                <div v-if="expandedFolders.includes(folder.id)" class="folder-content">
                  <div 
                    v-for="request in folder.requests" 
                    :key="request.id"
                    :class="['request-item', { active: activeRequestId === request.id }]"
                    @click="loadRequest(request)"
                  >
                    <div class="request-info">
                      <span :class="['method-badge', request.method.toLowerCase()]">{{ request.method }}</span>
                      <span class="request-name">{{ request.name }}</span>
                    </div>
                    <button class="delete-request-btn" @click.stop="handleDeleteRequest(request.id, request.name)" title="Delete Request">
                      <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
                        <path d="M11 1.5v1h3.5a.5.5 0 0 1 0 1h-.538l-.853 10.66A2 2 0 0 1 11.115 16h-6.23a2 2 0 0 1-1.994-1.84L2.038 3.5H1.5a.5.5 0 0 1 0-1H5v-1A1.5 1.5 0 0 1 6.5 0h3A1.5 1.5 0 0 1 11 1.5zm-5 0v1h4v-1a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5z"/>
                      </svg>
                    </button>
                  </div>
                </div>
              </div>

              <!-- Root-level Requests in Collection -->
              <div 
                v-for="request in collection.requests" 
                :key="request.id"
                :class="['request-item', { active: activeRequestId === request.id }]"
                @click="loadRequest(request)"
              >
                <div class="request-info">
                  <span :class="['method-badge', request.method.toLowerCase()]">{{ request.method }}</span>
                  <span class="request-name">{{ request.name }}</span>
                </div>
                <button class="delete-request-btn" @click.stop="handleDeleteRequest(request.id, request.name)" title="Delete Request">
                  <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M11 1.5v1h3.5a.5.5 0 0 1 0 1h-.538l-.853 10.66A2 2 0 0 1 11.115 16h-6.23a2 2 0 0 1-1.994-1.84L2.038 3.5H1.5a.5.5 0 0 1 0-1H5v-1A1.5 1.5 0 0 1 6.5 0h3A1.5 1.5 0 0 1 11 1.5zm-5 0v1h4v-1a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5z"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>

          <!-- Empty State -->
          <div v-if="collections.length === 0" class="empty-state">
            <p>No collections yet</p>
            <button class="create-first-btn" @click="createFirstCollection">
              Create Collection
            </button>
          </div>
        </div>
      </div>
      
      <!-- History View -->
      <div v-if="activeTab === 'history'" class="sidebar-content history-container">
        <RequestHistory 
          :workspace-id="workspaceId"
          @selectHistory="handleHistorySelected" 
        />
      </div>
    </div>

    <!-- Collapsed State -->
    <div v-else class="collapsed-icons">
      <button class="icon-btn" title="Collections">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
          <path d="M0 3a2 2 0 012-2h6l2 3h8a2 2 0 012 2v10a2 2 0 01-2 2H2a2 2 0 01-2-2V3z"/>
        </svg>
      </button>
    </div>

    <!-- Custom Modal -->
    <Modal 
      :is-open="modal.show"
      :title="modal.title"
      :message="modal.message"
      :input-type="modal.inputType"
      :initial-value="modal.initialValue"
      :placeholder="modal.placeholder"
      :confirm-text="modal.confirmText"
      :variant="modal.variant"
      @confirm="handleModalConfirm"
      @cancel="modal.show = false"
    />
    />

    <!-- Postman Import Modal -->
    <ImportPostmanModal 
      :is-open="showImportModal"
      :workspace-id="workspaceId"
      @close="showImportModal = false"
      @imported="handleImported"
    />

    <!-- Add Collection Type Modal -->
    <AddCollectionTypeModal
      :is-open="showAddSelection"
      @close="showAddSelection = false"
      @select="handleTypeSelect"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, reactive, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import Modal from './Modal.vue';
import RequestHistory from './RequestHistory.vue';
import ImportPostmanModal from './ImportPostmanModal.vue';
import AddCollectionTypeModal from './AddCollectionTypeModal.vue';

interface Request {
  id: string;
  name: string;
  method: string;
  url: string;
  folder_id: string;
}

interface Folder {
  id: string;
  name: string;
  requests: Request[];
}

interface Collection {
  id: string;
  name: string;
  folders: Folder[];
  requests: Request[];
}

interface Workspace {
  id: string;
  name: string;
}

const props = defineProps<{
  workspaceId: string
}>();

const emit = defineEmits<{
  'request-selected': [request: any],
  'history-selected': [item: any],
  'update:workspaceId': [id: string]
}>();

const activeTab = ref<'collections' | 'history'>('collections');
const showImportModal = ref(false);
const showAddSelection = ref(false);

const handleTypeSelect = (type: 'blank' | 'import') => {
  showAddSelection.value = false;
  if (type === 'blank') {
    showNewMenu();
  } else {
    showImportModal.value = true;
  }
};

const handleImported = async (collectionId: string) => {
  await fetchCollections();
  if (!expandedCollections.value.includes(collectionId)) {
    expandedCollections.value.push(collectionId);
  }
};

const isCollapsed = ref(false);
// currentWorkspace is now managed by parent via v-model:workspace-id
const localWorkspaceId = computed({
  get: () => props.workspaceId,
  set: (val) => emit('update:workspaceId', val)
});

const expandedCollections = ref<string[]>([]);
const expandedFolders = ref<string[]>([]);
const activeRequestId = ref<string | null>(null);
const collections = ref<Collection[]>([]);
const workspaces = ref<Workspace[]>([]);
const isLoading = ref(false);

watch(() => props.workspaceId, () => {
  fetchCollections();
});

const fetchWorkspaces = async () => {
  try {
    const result = await invoke('get_workspaces') as Workspace[];
    workspaces.value = result;
    // If no workspaces, ensure at least default exists
    if (result.length === 0) {
      await invoke('ensure_workspace', { workspaceId: 'default' });
      await fetchWorkspaces();
    }
  } catch (error) {
    console.error('Failed to fetch workspaces:', error);
  }
};

const fetchCollections = async () => {
  isLoading.value = true;
  try {
    console.log('Fetching collections for workspace:', props.workspaceId);
    // Ensure workspace exists
    const wId = await invoke('ensure_workspace', { workspaceId: props.workspaceId });
    console.log('Workspace ensured:', wId);
    
    // Fetch collections
    const result = await invoke('get_collections_with_folders', { workspaceId: props.workspaceId });
    collections.value = result as Collection[];
  } catch (error) {
    console.error('Failed to fetch collections:', error);
  } finally {
    isLoading.value = false;
  }
};

onMounted(async () => {
  await fetchWorkspaces();
  await fetchCollections();
});

const toggleCollapse = () => {
  isCollapsed.value = !isCollapsed.value;
};

const toggleCollection = (id: string) => {
  const index = expandedCollections.value.indexOf(id);
  if (index > -1) {
    expandedCollections.value.splice(index, 1);
  } else {
    expandedCollections.value.push(id);
  }
};

const toggleFolder = (id: string) => {
  const index = expandedFolders.value.indexOf(id);
  if (index > -1) {
    expandedFolders.value.splice(index, 1);
  } else {
    expandedFolders.value.push(id);
  }
};

const loadRequest = (request: any) => {
  activeRequestId.value = request.id;
  emit('request-selected', request);
};

// Modal System
const modal = reactive({
  show: false,
  title: '',
  message: '',
  inputType: '',
  initialValue: '',
  placeholder: '',
  confirmText: 'Confirm',
  variant: 'primary',
  action: null as Function | null
});

const openModal = (config: any) => {
  modal.title = config.title || '';
  modal.message = config.message || '';
  modal.inputType = config.inputType || '';
  modal.initialValue = config.initialValue || '';
  modal.placeholder = config.placeholder || '';
  modal.confirmText = config.confirmText || 'Confirm';
  modal.variant = config.variant || 'primary';
  modal.action = config.action || null;
  modal.show = true;
};

const handleModalConfirm = async (value: string) => {
  if (modal.action) {
    await modal.action(value);
  }
  modal.show = false;
};

const showNewMenu = () => {
  openModal({
    title: 'New Collection',
    message: 'Enter a name for your new collection.',
    inputType: 'text',
    placeholder: 'Collection Name',
    confirmText: 'Create',
    action: async (name: string) => {
      if (!name.trim()) return;
      try {
        const id = await invoke('create_collection', { 
          name: name.trim(), 
          workspaceId: props.workspaceId 
        }) as string;
        await fetchCollections();
        if (!expandedCollections.value.includes(id)) {
          expandedCollections.value.push(id);
        }
      } catch (error) {
        console.error('Failed to create collection:', error);
      }
    }
  });
};

const addFolder = (collectionId: string) => {
  openModal({
    title: 'New Folder',
    message: 'Enter a name for the folder.',
    inputType: 'text',
    placeholder: 'Folder Name',
    confirmText: 'Create',
    action: async (name: string) => {
      if (!name.trim()) return;
      try {
        await invoke('create_folder', { name: name.trim(), collectionId });
        await fetchCollections();
      } catch (error) {
        console.error('Failed to create folder:', error);
      }
    }
  });
};

const createFirstCollection = () => {
  showAddSelection.value = true;
};

const handleDeleteWorkspace = () => {
  if (props.workspaceId === 'default') return;
  const ws = workspaces.value.find(w => w.id === props.workspaceId);
  
  openModal({
    title: 'Delete Workspace',
    message: `Are you sure you want to delete "${ws?.name}"? All collections and requests inside will be lost forever.`,
    variant: 'danger',
    confirmText: 'Delete Workspace',
    action: async () => {
      try {
        await invoke('delete_workspace', { workspaceId: props.workspaceId });
        await fetchWorkspaces();
        // Since we are changing props indirectly via parent update, we should emit new value
        // But the parent manages it.
        // Actually, deleting the current workspace means we must switch to 'default'.
        // So we emit the update.
        emit('update:workspaceId', 'default');
      } catch (error) {
        console.error('Failed to delete workspace:', error);
      }
    }
  });
};

const handleCreateWorkspace = () => {
  openModal({
    title: 'New Workspace',
    message: 'Workspaces help you organize different projects.',
    inputType: 'text',
    placeholder: 'Workspace Name',
    confirmText: 'Create Workspace',
    action: async (name: string) => {
      if (!name.trim()) return;
      try {
        const id = await invoke('create_workspace', { name: name.trim() }) as string;
        await fetchWorkspaces();
        emit('update:workspaceId', id);
      } catch (error) {
        console.error('Failed to create workspace:', error);
      }
    }
  });
};

const handleRenameWorkspace = () => {
  const currentWs = workspaces.value.find(w => w.id === props.workspaceId);
  openModal({
    title: 'Rename Workspace',
    inputType: 'text',
    initialValue: currentWs?.name,
    confirmText: 'Rename',
    action: async (name: string) => {
      if (!name.trim() || name === currentWs?.name) return;
      try {
        await invoke('rename_workspace', { 
          workspaceId: props.workspaceId, 
          name: name.trim() 
        });
        await fetchWorkspaces();
      } catch (error) {
        console.error('Failed to rename workspace:', error);
      }
    }
  });
};

const handleDeleteCollection = (id: string, name: string) => {
  openModal({
    title: 'Delete Collection',
    message: `Are you sure you want to delete "${name}"?`,
    variant: 'danger',
    confirmText: 'Delete',
    action: async () => {
      try {
        await invoke('delete_collection', { collectionId: id });
        await fetchCollections();
      } catch (error) {
        console.error('Failed to delete collection:', error);
      }
    }
  });
};

const handleDeleteFolder = (id: string, name: string) => {
  openModal({
    title: 'Delete Folder',
    message: `Are you sure you want to delete "${name}"?`,
    variant: 'danger',
    confirmText: 'Delete',
    action: async () => {
      try {
        await invoke('delete_folder', { folderId: id });
        await fetchCollections();
      } catch (error) {
        console.error('Failed to delete folder:', error);
      }
    }
  });
};

const handleDeleteRequest = (id: string, name: string) => {
  openModal({
    title: 'Delete Request',
    message: `Are you sure you want to delete "${name}"?`,
    variant: 'danger',
    confirmText: 'Delete',
    action: async () => {
      try {
        await invoke('delete_request', { requestId: id });
        await fetchCollections();
      } catch (error) {
        console.error('Failed to delete request:', error);
      }
    }
  });
};

const switchTab = (tab: 'collections' | 'history') => {
  activeTab.value = tab;
};

const handleHistorySelected = (item: any) => {
  emit('history-selected', item);
};

// Expose refresh method for other components
defineExpose({
  refresh: fetchCollections,
  switchTab
});
</script>

<style scoped>
.collections-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1a1a1a;
  border-right: 1px solid #2a2a2a;
  width: 100%;
  min-width: 180px;
  overflow: hidden;
}

.collections-sidebar.collapsed {
  min-width: 50px;
}

.sidebar-header {
  padding: 16px;
  border-bottom: 1px solid #2a2a2a;
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 60px;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.header-content h2 {
  font-size: 16px;
  font-weight: 600;
  color: #fff;
  margin: 0;
}

.collapse-btn {
  padding: 6px;
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.collapse-btn:hover {
  background: #2a2a2a;
  color: #aaa;
}

.sidebar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.refresh-btn {
  padding: 6px;
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.refresh-btn:hover {
  background: #2a2a2a;
  color: #aaa;
}

.refresh-btn.rotating svg {
  animation: rotate 1s linear infinite;
}

@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.new-btn {
  padding: 6px;
  background: transparent;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  color: #888;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}


.new-btn:hover {
  border-color: #667eea;
  color: #667eea;
  background: rgba(102, 126, 234, 0.05);
}

.sidebar-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.workspace-section {
  padding: 12px 16px;
  border-bottom: 1px solid #2a2a2a;
}

.workspace-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.workspace-delete-btn, .workspace-add-btn, .workspace-edit-btn {
  padding: 4px;
  background: transparent;
  border: none;
  color: #666;
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.workspace-delete-btn:hover {
  color: #ff4d4d;
  background: rgba(255, 77, 77, 0.1);
}

.workspace-add-btn:hover, .workspace-edit-btn:hover {
  color: #667eea;
  background: rgba(102, 126, 234, 0.1);
}

.workspace-select {
  width: 100%;
  height: 36px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #fff;
  padding: 0 12px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.workspace-select:hover {
  border-color: #4a4a4a;
}

.workspace-select:focus {
  outline: none;
  border-color: #667eea;
}

.collections-tree {
  padding: 8px 0;
}

.collection-item {
  margin: 4px 0;
}

.collection-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.collection-header:hover {
  background: #2a2a2a;
}

.expand-icon {
  color: #666;
  flex-shrink: 0;
}

.folder-icon {
  color: #667eea;
  flex-shrink: 0;
}

.collection-name {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: #ddd;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.collection-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}

.collection-actions, .folder-actions {
  display: flex;
  opacity: 0;
  transition: opacity 0.2s;
}

.collection-header:hover .collection-actions,
.folder-header:hover .folder-actions {
  opacity: 1;
}

.delete-btn:hover {
  color: #ff4d4d !important;
}

.action-btn {
  padding: 4px;
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
  border-radius: 3px;
  transition: all 0.2s;
}

.action-btn:hover {
  background: #3a3a3a;
  color: #aaa;
}

.collection-content {
  padding-left: 8px;
}

.folder-item {
  margin: 4px 0;
}

.folder-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px;
  cursor: pointer;
  transition: all 0.2s;
}

.folder-header:hover {
  background: #2a2a2a;
}

.folder-name {
  flex: 1;
  font-size: 13px;
  color: #ccc;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.folder-content {
  padding-left: 20px;
}

.request-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
  transition: all 0.2s;
  border-left: 2px solid transparent;
}

.request-item:hover {
  background: #2a2a2a;
}

.request-item:hover .delete-request-btn {
  opacity: 1;
}

/* Tabs */
.sidebar-tabs {
  display: flex;
  border-bottom: 1px solid #2a2a2a;
  background: #111;
}

.tab-btn {
  flex: 1;
  padding: 10px;
  background: transparent;
  border: none;
  color: #888;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.tab-btn:hover {
  color: #ccc;
  background: #1a1a1a;
}

.tab-btn.active {
  color: #667eea;
  border-bottom-color: #667eea;
  background: #1a1a1a;
}

.sidebar-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.history-container {
  display: flex;
  flex-direction: column;
}

.request-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  overflow: hidden;
}

.delete-request-btn {
  padding: 4px;
  background: transparent;
  border: none;
  color: #666;
  opacity: 0;
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.delete-request-btn:hover {
  color: #ff4d4d;
  background: rgba(255, 77, 77, 0.1);
}

.request-item.active {
  background: rgba(102, 126, 234, 0.1);
  border-left-color: #667eea;
}

.method-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  flex-shrink: 0;
}

.method-badge.get {
  background: rgba(46, 213, 115, 0.15);
  color: #2ed573;
}

.method-badge.post {
  background: rgba(255, 193, 7, 0.15);
  color: #ffc107;
}

.method-badge.put {
  background: rgba(52, 152, 219, 0.15);
  color: #3498db;
}

.method-badge.patch {
  background: rgba(155, 89, 182, 0.15);
  color: #9b59b6;
}

.method-badge.delete {
  background: rgba(231, 76, 60, 0.15);
  color: #e74c3c;
}

.request-name {
  flex: 1;
  font-size: 13px;
  color: #aaa;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.request-item.active .request-name {
  color: #ddd;
}

.empty-state {
  padding: 40px 20px;
  text-align: center;
  color: #666;
}

.empty-state p {
  font-size: 14px;
  margin: 0 0 16px 0;
}

.create-first-btn {
  padding: 8px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  border-radius: 6px;
  color: white;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.create-first-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.collapsed-icons {
  padding: 16px 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.icon-btn {
  padding: 10px;
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s;
}

.icon-btn:hover {
  background: #2a2a2a;
  color: #667eea;
}
</style>
