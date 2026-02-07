<template>
  <div v-if="isOpen" class="modal-overlay" @click="handleOverlayClick">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>{{ editMode ? 'Update Request' : 'Save Request' }}</h2>
        <button class="close-btn" @click="close">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M2 2l12 12M14 2L2 14" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <div class="modal-body">
        <!-- Request Name -->
        <div class="form-group">
          <label>Request Name</label>
          <input 
            v-model="requestName" 
            type="text" 
            placeholder="e.g., Get User Profile"
            class="form-input"
            @keydown.enter="save"
          />
        </div>

        <!-- Collection Selection -->
        <div class="form-group">
          <label>Collection</label>
          <div class="select-with-new">
            <select v-model="selectedCollection" class="form-select">
              <option value="">Select Collection</option>
              <option v-for="col in collections" :key="col.id" :value="col.id">
                {{ col.name }}
              </option>
            </select>
            <button class="new-collection-btn" @click="showNewCollection">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
                <path d="M7 0v14M0 7h14" stroke="currentColor" stroke-width="2"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Folder Selection (Optional) -->
        <div v-if="selectedCollection" class="form-group">
          <label>Folder (Optional)</label>
          <div class="select-with-new">
            <select v-model="selectedFolder" class="form-select">
              <option value="">No Folder</option>
              <option v-for="folder in availableFolders" :key="folder.id" :value="folder.id">
                {{ folder.name }}
              </option>
            </select>
            <button class="new-folder-btn" @click="showNewFolder">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
                <path d="M7 0v14M0 7h14" stroke="currentColor" stroke-width="2"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- New Collection Input (Inline) -->
        <div v-if="creatingCollection" class="form-group inline-create">
          <label>New Collection Name</label>
          <div class="inline-input-group">
            <input 
              v-model="newCollectionName" 
              type="text" 
              placeholder="My Collection"
              class="form-input"
              @keydown.enter="createCollection"
              @keydown.esc="cancelNewCollection"
            />
            <button class="confirm-btn" @click="createCollection">✓</button>
            <button class="cancel-btn" @click="cancelNewCollection">×</button>
          </div>
        </div>

        <!-- New Folder Input (Inline) -->
        <div v-if="creatingFolder" class="form-group inline-create">
          <label>New Folder Name</label>
          <div class="inline-input-group">
            <input 
              v-model="newFolderName" 
              type="text" 
              placeholder="My Folder"
              class="form-input"
              @keydown.enter="createFolder"
              @keydown.esc="cancelNewFolder"
            />
            <button class="confirm-btn" @click="createFolder">✓</button>
            <button class="cancel-btn" @click="cancelNewFolder">×</button>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" @click="close">Cancel</button>
        <button 
          class="btn-primary" 
          @click="save"
          :disabled="!canSave"
        >
          {{ editMode ? 'Update' : 'Save' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

interface Collection {
  id: string;
  name: string;
  folders?: Folder[];
}

interface Folder {
  id: string;
  name: string;
}

const props = defineProps<{
  isOpen: boolean;
  editMode?: boolean;
  currentRequest?: any;
  collections: Collection[];
}>();

const emit = defineEmits<{
  close: []
  save: [data: {
    name: string;
    collectionId: string;
    folderId: string | null;
  }]
  createCollection: [name: string]
  createFolder: [data: { collectionId: string; name: string }]
}>();

const requestName = ref('');
const selectedCollection = ref('');
const selectedFolder = ref('');
const creatingCollection = ref(false);
const creatingFolder = ref(false);
const newCollectionName = ref('');
const newFolderName = ref('');

const availableFolders = computed(() => {
  if (!selectedCollection.value) return [];
  const collection = props.collections.find(c => c.id === selectedCollection.value);
  return collection?.folders || [];
});

const canSave = computed(() => {
  return requestName.value.trim() !== '' && selectedCollection.value !== '';
});

const close = () => {
  emit('close');
  resetForm();
};

const save = () => {
  if (!canSave.value) return;
  
  emit('save', {
    name: requestName.value.trim(),
    collectionId: selectedCollection.value,
    folderId: selectedFolder.value || null,
  });
  
  close();
};

const showNewCollection = () => {
  creatingCollection.value = true;
  newCollectionName.value = '';
};

const createCollection = () => {
  if (newCollectionName.value.trim()) {
    emit('createCollection', newCollectionName.value.trim());
    creatingCollection.value = false;
    newCollectionName.value = '';
  }
};

const cancelNewCollection = () => {
  creatingCollection.value = false;
  newCollectionName.value = '';
};

const showNewFolder = () => {
  if (!selectedCollection.value) return;
  creatingFolder.value = true;
  newFolderName.value = '';
};

const createFolder = () => {
  if (newFolderName.value.trim() && selectedCollection.value) {
    emit('createFolder', {
      collectionId: selectedCollection.value,
      name: newFolderName.value.trim(),
    });
    creatingFolder.value = false;
    newFolderName.value = '';
  }
};

const cancelNewFolder = () => {
  creatingFolder.value = false;
  newFolderName.value = '';
};

const resetForm = () => {
  requestName.value = '';
  selectedCollection.value = '';
  selectedFolder.value = '';
  creatingCollection.value = false;
  creatingFolder.value = false;
  newCollectionName.value = '';
  newFolderName.value = '';
};

const handleOverlayClick = () => {
  close();
};

// Initialize form when opened
const initializeForm = () => {
  if (props.currentRequest) {
    requestName.value = props.currentRequest.name || '';
    selectedCollection.value = props.currentRequest.collectionId || '';
    selectedFolder.value = props.currentRequest.folderId || '';
  }
};

// Expose method to initialize
defineExpose({
  initializeForm
});
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
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.modal-content {
  background: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow: auto;
  animation: slideUp 0.3s ease;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

@keyframes slideUp {
  from { 
    opacity: 0;
    transform: translateY(20px);
  }
  to { 
    opacity: 1;
    transform: translateY(0);
  }
}

.modal-header {
  padding: 24px;
  border-bottom: 1px solid #2a2a2a;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #fff;
}

.close-btn {
  padding: 8px;
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #2a2a2a;
  color: #aaa;
}

.modal-body {
  padding: 24px;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 500;
  color: #aaa;
}

.form-input {
  width: 100%;
  height: 40px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #fff;
  padding: 0 16px;
  font-size: 14px;
  transition: all 0.2s;
}

.form-input:hover {
  border-color: #4a4a4a;
}

.form-input:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.form-input::placeholder {
  color: #666;
}

.select-with-new {
  display: flex;
  gap: 8px;
}

.form-select {
  flex: 1;
  height: 40px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #fff;
  padding: 0 16px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.form-select:hover {
  border-color: #4a4a4a;
}

.form-select:focus {
  outline: none;
  border-color: #667eea;
}

.new-collection-btn,
.new-folder-btn {
  width: 40px;
  height: 40px;
  background: transparent;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #888;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.new-collection-btn:hover,
.new-folder-btn:hover {
  border-color: #667eea;
  color: #667eea;
  background: rgba(102, 126, 234, 0.05);
}

.inline-create {
  background: rgba(102, 126, 234, 0.05);
  border: 1px solid rgba(102, 126, 234, 0.2);
  border-radius: 8px;
  padding: 16px;
  animation: slideDown 0.2s ease;
}

@keyframes slideDown {
  from {
    opacity: 0;
    max-height: 0;
  }
  to {
    opacity: 1;
    max-height: 200px;
  }
}

.inline-input-group {
  display: flex;
  gap: 8px;
}

.inline-input-group .form-input {
  flex: 1;
}

.confirm-btn,
.cancel-btn {
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 6px;
  font-size: 18px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.confirm-btn {
  background: #2ed573;
  color: white;
}

.confirm-btn:hover {
  background: #26de81;
  transform: translateY(-1px);
}

.cancel-btn {
  background: #ff6b6b;
  color: white;
}

.cancel-btn:hover {
  background: #ee5253;
  transform: translateY(-1px);
}

.modal-footer {
  padding: 24px;
  border-top: 1px solid #2a2a2a;
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.btn-secondary {
  padding: 10px 24px;
  background: transparent;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #aaa;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover {
  border-color: #4a4a4a;
  background: #2a2a2a;
}

.btn-primary {
  padding: 10px 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  border-radius: 6px;
  color: white;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
