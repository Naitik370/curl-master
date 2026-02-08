<template>
  <div v-if="isOpen" class="modal-overlay" @click="handleOverlayClick">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>Manage Environments</h2>
        <button class="close-btn" @click="close">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M2 2l12 12M14 2L2 14" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <div class="modal-body">
        <div class="env-layout">
          <!-- Sidebar: Environment List -->
          <div class="env-sidebar">
            <button 
              class="add-env-btn" 
              @click="createNewEnv"
            >
              + New Environment
            </button>
            <div class="env-list">
              <div 
                v-for="env in environments" 
                :key="env.id"
                :class="['env-list-item', { active: selectedEnvId === env.id }]"
                @click="selectEnv(env.id)"
              >
                <span>{{ env.name }}</span>
              </div>
            </div>
          </div>

          <!-- Main: Variable Editor -->
          <div class="env-main">
            <div v-if="selectedEnvId" class="variable-editor">
              <div class="editor-header">
                <input 
                  v-model="editingEnvName" 
                  class="env-name-input"
                  placeholder="Environment Name"
                  @blur="updateEnvName"
                />
              </div>

              <div class="variables-table">
                <div class="table-header">
                  <div class="col-key">Variable</div>
                  <div class="col-value">Value</div>
                  <div class="col-secret">Secret</div>
                  <div class="col-actions"></div>
                </div>
                
                <div class="table-body">
                  <div 
                    v-for="(v, index) in variables" 
                    :key="v.id || index"
                    class="variable-row"
                  >
                    <input v-model="v.key" placeholder="Key" class="row-input" />
                    <input 
                      v-model="v.value" 
                      :type="v.is_secret ? 'password' : 'text'" 
                      placeholder="Value" 
                      class="row-input"
                    />
                    <div class="col-secret">
                      <input type="checkbox" v-model="v.is_secret" />
                    </div>
                    <div class="col-actions">
                      <button class="delete-btn" @click="removeVariable(index)">×</button>
                    </div>
                  </div>
                </div>
              </div>

              <button class="add-var-btn" @click="addVariable">+ Add Variable</button>
              
              <div class="editor-footer">
                <button class="save-btn" @click="saveVariables">Save Changes</button>
              </div>
            </div>
            <div v-else class="empty-selection">
              <p>Select an environment to manage variables</p>
            </div>
          </div>
        </div>

        <!-- Clear all data -->
        <div class="clear-data-section">
          <button type="button" class="clear-data-btn" @click="confirmClearAllData">
            Clear all data
          </button>
          <p class="clear-data-hint">Remove all workspaces, collections, history, and environments. Settings reset to defaults.</p>
        </div>
      </div>
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
  </div>
</template>

<script setup lang="ts">
import { ref, watch, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import Modal from './Modal.vue';

const props = defineProps<{
  isOpen: boolean;
  workspaceId: string;
}>();

const emit = defineEmits<{
  close: []
  updated: []
  dataCleared: []
}>();

const environments = ref<any[]>([]);
const selectedEnvId = ref<string | null>(null);
const variables = ref<any[]>([]);
const editingEnvName = ref('');

const fetchEnvironments = async () => {
  try {
    await invoke('ensure_workspace', { workspaceId: props.workspaceId });
    environments.value = await invoke('get_environments', { workspaceId: props.workspaceId }) as any[];
    selectedEnvId.value = null;
    variables.value = [];
    editingEnvName.value = '';
  } catch (error) {
    console.error('Failed to fetch environments:', error);
  }
};

// Refresh environments when modal opens or workspace changes
watch(() => [props.isOpen, props.workspaceId], ([isOpen]) => {
  if (isOpen) {
    fetchEnvironments();
  }
}, { immediate: true });

const selectEnv = async (id: string) => {
  selectedEnvId.value = id;
  const env = environments.value.find(e => e.id === id);
  editingEnvName.value = env ? env.name : '';
  
  try {
    const vars = await invoke('get_variables', { environmentId: id }) as any[];
    // Ensure at least one empty row
    if (vars.length === 0) {
      variables.value = [{ key: '', value: '', is_secret: false }];
    } else {
      variables.value = vars;
    }
  } catch (error) {
    console.error('Failed to fetch variables:', error);
  }
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

const confirmClearAllData = () => {
  openModal({
    title: 'Clear all data?',
    message: 'This will delete all workspaces, collections, history, and environments. Settings will be reset to defaults. This cannot be undone.',
    inputType: '',
    confirmText: 'Clear all',
    variant: 'danger',
    action: async () => {
      try {
        await invoke('clear_all_data');
        close();
        emit('dataCleared');
      } catch (error) {
        console.error('Failed to clear data:', error);
      }
    }
  });
};

const createNewEnv = () => {
  openModal({
    title: 'New Environment',
    message: 'Environments let you store variables for different setups (e.g., Local, Production).',
    inputType: 'text',
    placeholder: 'Environment Name',
    confirmText: 'Create',
    action: async (name: string) => {
      if (!name.trim()) return;
      try {
        const id = await invoke('create_environment', { name: name.trim(), workspaceId: props.workspaceId });
        await fetchEnvironments();
        selectEnv(id as string);
      } catch (error) {
        console.error('Failed to create environment:', error);
      }
    }
  });
};

const addVariable = () => {
  variables.value.push({ key: '', value: '', is_secret: false });
};

const removeVariable = (index: number) => {
  variables.value.splice(index, 1);
};

const saveVariables = async () => {
  if (!selectedEnvId.value) return;
  
  try {
    // Filter out empty keys
    const varsToSave = variables.value.filter(v => v.key.trim());
    await invoke('save_variables', { 
      environmentId: selectedEnvId.value, 
      vars: varsToSave 
    });
    
    // Refresh variables to get official IDs from DB
    await selectEnv(selectedEnvId.value);
    
    openModal({
      title: 'Success',
      message: 'Variables saved successfully!',
      confirmText: 'OK'
    });
    emit('updated');
  } catch (error) {
    console.error('Failed to save variables:', error);
    openModal({
      title: 'Error',
      message: 'Failed to save variables. Please check the logs.',
      variant: 'danger',
      confirmText: 'Retry'
    });
  }
};

const updateEnvName = async () => {
  if (!selectedEnvId.value || !editingEnvName.value.trim()) return;
  
  // Need to implement update_environment_name in Rust if we want this, 
  // but for now let's just keep it in sync with local environments array
  const env = environments.value.find(e => e.id === selectedEnvId.value);
  if (env) {
    env.name = editingEnvName.value;
    // We'll skip the backend update for now or implement it later
  }
};

const close = () => {
  emit('close');
};

const handleOverlayClick = () => {
  close();
};

watch(() => props.isOpen, (newVal) => {
  if (newVal) {
    fetchEnvironments();
  }
});
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.modal-content {
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 12px;
  width: 90%;
  max-width: 900px;
  height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0,0,0,0.5);
}

.modal-header {
  padding: 20px;
  border-bottom: 1px solid #333;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h2 {
  font-size: 18px;
  margin: 0;
}

.close-btn {
  background: transparent;
  border: none;
  color: #666;
  cursor: pointer;
}

.modal-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.env-layout {
  display: flex;
  flex: 1;
  min-height: 0;
}

.env-sidebar {
  width: 240px;
  border-right: 1px solid #333;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.add-env-btn {
  width: 100%;
  padding: 10px;
  background: #2a2a2a;
  border: 1px solid #444;
  border-radius: 6px;
  color: #fff;
  cursor: pointer;
  transition: all 0.2s;
}

.add-env-btn:hover {
  border-color: #667eea;
  color: #667eea;
}

.env-list {
  flex: 1;
  overflow-y: auto;
}

.env-list-item {
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  color: #aaa;
  margin-bottom: 4px;
}

.env-list-item:hover {
  background: #2a2a2a;
}

.env-list-item.active {
  background: rgba(102, 126, 234, 0.15);
  color: #667eea;
  font-weight: 600;
}

.env-main {
  flex: 1;
  padding: 24px;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.editor-header {
  margin-bottom: 24px;
}

.env-name-input {
  background: transparent;
  border: none;
  border-bottom: 1px solid #333;
  font-size: 24px;
  font-weight: 700;
  color: #fff;
  width: 100%;
  padding: 8px 0;
  outline: none;
}

.env-name-input:focus {
  border-color: #667eea;
}

.variables-table {
  flex: 1;
}

.table-header {
  display: flex;
  padding: 10px 0;
  border-bottom: 1px solid #333;
  color: #666;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
}

.col-key { flex: 1; }
.col-value { flex: 1; }
.col-secret { width: 60px; text-align: center; }
.col-actions { width: 40px; }

.variable-row {
  display: flex;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #222;
  gap: 12px;
}

.row-input {
  flex: 1;
  background: #222;
  border: 1px solid #333;
  border-radius: 4px;
  color: #ddd;
  padding: 6px 12px;
  font-size: 13px;
}

.row-input:focus {
  border-color: #444;
  outline: none;
}

.delete-btn {
  background: transparent;
  border: none;
  color: #555;
  font-size: 18px;
  cursor: pointer;
}

.delete-btn:hover {
  color: #ff6b6b;
}

.add-var-btn {
  margin-top: 16px;
  background: transparent;
  border: 1px dashed #444;
  color: #888;
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
  align-self: flex-start;
}

.add-var-btn:hover {
  border-color: #666;
  color: #aaa;
}

.editor-footer {
  margin-top: 32px;
  display: flex;
  justify-content: flex-end;
}

.save-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  color: white;
  padding: 10px 24px;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
}

.empty-selection {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #555;
}

.clear-data-section {
  padding: 16px 24px;
  border-top: 1px solid #333;
  margin-top: auto;
}

.clear-data-btn {
  background: transparent;
  border: 1px solid #5a2a2a;
  color: #c66;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.clear-data-btn:hover {
  background: rgba(200, 80, 80, 0.1);
  border-color: #c66;
}

.clear-data-hint {
  margin: 8px 0 0;
  font-size: 12px;
  color: #666;
}
</style>
