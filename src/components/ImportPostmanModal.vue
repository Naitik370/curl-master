<template>
  <div v-if="isOpen" class="modal-overlay" @click="handleOverlayClick">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <div class="header-title">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="#FF6C37" class="postman-logo">
            <path d="M12 0C5.373 0 0 5.373 0 12s5.373 12 12 12 12-5.373 12-12S18.627 0 12 0zm0 18.667c-3.682 0-6.667-2.985-6.667-6.667S8.318 5.333 12 5.333 18.667 8.318 18.667 12 15.682 18.667 12 18.667z"/>
            <path d="M12 7.333c-2.577 0-4.667 2.09-4.667 4.667s2.09 4.667 4.667 4.667 4.667-2.09 4.667-4.667-2.09-4.667-4.667-4.667zm3.179 6.221l-1.077-1.077c-.126-.126-.33-.126-.456 0l-.513.513c-.126.126-.126.33 0 .456l1.077 1.077c.126.126.33.126.456 0l.513-.513c.126-.126.126-.33 0-.456zm-.513-5.316l-1.077 1.077c-.126.126-.126.33 0 .456l.513.513c.126.126.33.126.456 0l1.077-1.077c.126-.126.126-.33 0-.456l-.513-.513c-.126-.126-.33-.126-.456 0zm-5.332 0l-.513.513c-.126.126-.126.33 0 .456l1.077 1.077c.126.126.33.126.456 0l.513-.513c.126-.126.126-.33 0-.456l-1.077-1.077c-.126-.126-.33-.126-.456 0zm-1.077 5.316l.513.513c.126.126.33.126.456 0l1.077-1.077c.126-.126.126-.33 0-.456l-.513-.513c-.126-.126-.33-.126-.456 0l-1.077 1.077c-.126.126-.126.33 0 .456z"/>
          </svg>
          <h2>Import Postman Collection</h2>
        </div>
        <button class="close-btn" @click="close">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M2 2l12 12M14 2L2 14" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <div class="modal-body">
        <div 
          v-if="!fileData" 
          class="drop-zone" 
          @dragover.prevent="isDragging = true"
          @dragleave.prevent="isDragging = false"
          @drop.prevent="handleDrop"
          @click="triggerFilePicker"
          :class="{ dragging: isDragging }"
        >
          <div class="drop-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" stroke-linecap="round" stroke-linejoin="round"/>
              <polyline points="17 8 12 3 7 8" stroke-linecap="round" stroke-linejoin="round"/>
              <line x1="12" y1="3" x2="12" y2="15" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <p class="drop-text">Drag and drop your Postman Collection JSON</p>
          <p class="sub-text">or click to browse files</p>
          <input 
            ref="fileInput" 
            type="file" 
            accept=".json" 
            class="hidden-input" 
            @change="handleFileSelect"
          />
        </div>

        <div v-else class="preview-area">
          <div class="file-info">
            <div class="info-row">
              <span class="label">Collection:</span>
              <span class="value">{{ fileData.info.name }}</span>
            </div>
            <div class="info-row">
              <span class="label">Items:</span>
              <span class="value">{{ fileData.item.length }} top-level items</span>
            </div>
          </div>
          
          <div class="actions">
            <button class="btn-secondary" @click="fileData = null">Choose different file</button>
          </div>
        </div>

        <div v-if="error" class="error-msg">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8.982 1.566a1.13 1.13 0 0 0-1.96 0L.165 13.233c-.457.778.091 1.767.98 1.767h13.71c.889 0 1.438-.99.98-1.767L8.982 1.566zM8 5c.535 0 .954.462.9.995l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 5.995A.905.905 0 0 1 8 5zm.002 6a1 1 0 1 1 0 2 1 1 0 0 1 0-2z"/>
          </svg>
          {{ error }}
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" @click="close" :disabled="isImporting">Cancel</button>
        <button 
          class="btn-primary" 
          @click="importCollection"
          :disabled="!fileData || isImporting"
        >
          <span v-if="isImporting" class="loader"></span>
          {{ isImporting ? 'Importing...' : 'Import Collection' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { convertPostmanToCM } from '../utils/postmanConverter';

const props = defineProps<{
  isOpen: boolean;
  workspaceId: string;
}>();

const emit = defineEmits<{
  close: [];
  imported: [collectionId: string];
}>();

const fileInput = ref<HTMLInputElement | null>(null);
const fileData = ref<any>(null);
const isDragging = ref(false);
const isImporting = ref(false);
const error = ref<string | null>(null);

const handleDrop = (e: DragEvent) => {
  isDragging.value = false;
  const file = e.dataTransfer?.files[0];
  if (file) readFile(file);
};

const handleFileSelect = (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (file) readFile(file);
};

const triggerFilePicker = () => {
  fileInput.value?.click();
};

const readFile = (file: File) => {
  error.value = null;
  if (file.type !== 'application/json' && !file.name.endsWith('.json')) {
    error.value = 'Please select a valid JSON file.';
    return;
  }

  const reader = new FileReader();
  reader.onload = (e) => {
    try {
      const json = JSON.parse(e.target?.result as string);
      if (!json.info || !json.item) {
        error.value = 'Invalid Postman Collection format. Please use v2.1 exports.';
        return;
      }
      fileData.value = json;
    } catch (err) {
      error.value = 'Failed to parse JSON file.';
    }
  };
  reader.readAsText(file);
};

const importCollection = async () => {
  if (!fileData.value) return;
  
  isImporting.value = true;
  error.value = null;

  try {
    const cmCollection = convertPostmanToCM(fileData.value);
    cmCollection.workspace_id = props.workspaceId;
    
    console.log('Converted collection for import:', cmCollection);
    
    const collectionId = await invoke<string>('import_collection', { 
      collection: cmCollection 
    });
    
    emit('imported', collectionId);
    close();
  } catch (err: any) {
    console.error('Import failed:', err);
    error.value = `Import failed: ${err.message || err}`;
  } finally {
    isImporting.value = false;
  }
};

const close = () => {
  if (isImporting.value) return;
  fileData.value = null;
  error.value = null;
  emit('close');
};

const handleOverlayClick = () => {
  close();
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

.modal-content {
  background: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 12px;
  width: 95%;
  max-width: 500px;
  max-height: 90vh;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
  animation: slideUp 0.15s ease-out;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

@keyframes slideUp {
  from { transform: translateY(20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.modal-header {
  padding: 24px;
  border-bottom: 1px solid #2a2a2a;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.modal-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #fff;
}

.close-btn {
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
  padding: 8px;
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

.drop-zone {
  border: 2px dashed #4a4a4a;
  border-radius: 8px;
  padding: 40px 20px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
  background: rgba(42, 42, 42, 0.3);
}

.drop-zone:hover, .drop-zone.dragging {
  border-color: #FF6C37;
  background: rgba(255, 108, 55, 0.05);
}

.drop-icon {
  margin-bottom: 16px;
  color: #666;
  transition: color 0.2s;
}

.drop-zone:hover .drop-icon {
  color: #FF6C37;
}

.drop-text {
  font-weight: 500;
  margin-bottom: 4px;
  color: #eee;
}

.sub-text {
  font-size: 13px;
  color: #666;
}

.hidden-input {
  display: none;
}

.preview-area {
  background: #252525;
  border: 1px solid #3a3a3a;
  border-radius: 8px;
  padding: 20px;
}

.file-info {
  margin-bottom: 20px;
}

.info-row {
  display: flex;
  margin-bottom: 8px;
}

.info-row .label {
  color: #888;
  width: 100px;
  font-size: 13px;
}

.info-row .value {
  color: #fff;
  font-weight: 500;
  font-size: 14px;
}

.error-msg {
  margin-top: 16px;
  padding: 12px;
  background: rgba(220, 53, 69, 0.1);
  border: 1px solid rgba(220, 53, 69, 0.2);
  border-radius: 6px;
  color: #ff6b6b;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-footer {
  padding: 20px 24px;
  background: #222;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn-secondary {
  padding: 8px 16px;
  background: transparent;
  border: 1px solid #444;
  border-radius: 6px;
  color: #aaa;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background: #2a2a2a;
  color: #fff;
}

.btn-primary {
  padding: 8px 24px;
  background: #FF6C37;
  border: none;
  border-radius: 6px;
  color: #fff;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-primary:hover:not(:disabled) {
  background: #ff7f52;
  transform: translateY(-1px);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.loader {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  border-top-color: #fff;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
