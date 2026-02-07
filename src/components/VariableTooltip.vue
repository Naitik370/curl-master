<template>
  <div 
    v-if="isVisible" 
    class="variable-tooltip"
    :style="tooltipStyle"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <div class="tooltip-header">
      <span class="variable-name">{{ variableName }}</span>
      <button class="close-btn" @click="close">×</button>
    </div>
    
    <div class="tooltip-body">
      <div class="current-value">
        <label>Current Value:</label>
        <div class="value-display">{{ currentValue || '(not set)' }}</div>
      </div>
      
      <div class="edit-section">
        <label>Edit Value:</label>
        <input 
          v-model="editValue" 
          class="value-input"
          placeholder="Enter new value"
          @keydown.enter="saveValue"
          @keydown.esc="close"
        />
        <button class="save-btn" @click="saveValue">Save</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  variableName: string;
  currentValue: string;
  position: { x: number; y: number };
  visible: boolean;
  environmentId: string;
  workspaceId: string;
}>();

const emit = defineEmits<{
  close: [];
  updated: [{ key: string; value: string }];
}>();

const isVisible = computed(() => props.visible);
const editValue = ref('');
let hoverTimeout: number | null = null;

watch(() => props.visible, (visible) => {
  if (visible) {
    editValue.value = props.currentValue || '';
  }
});

const tooltipStyle = computed(() => ({
  left: `${props.position.x}px`,
  top: `${props.position.y + 25}px`,
}));

const handleMouseEnter = () => {
  if (hoverTimeout) {
    clearTimeout(hoverTimeout);
    hoverTimeout = null;
  }
};

const handleMouseLeave = () => {
  hoverTimeout = window.setTimeout(() => {
    close();
  }, 300);
};

const saveValue = async () => {
  try {
    // Get all existing variables
    const variables = await invoke<Array<{ id: string; key: string; value: string; is_secret: boolean }>>(
      'get_variables',
      { environmentId: props.environmentId }
    );
    
    // Update or add the variable
    const existingVar = variables.find(v => v.key === props.variableName);
    
    await invoke('save_variables', {
      environmentId: props.environmentId,
      variables: variables.map(v => 
        v.key === props.variableName 
          ? { ...v, value: editValue.value }
          : v
      ).concat(
        existingVar ? [] : [{
          id: crypto.randomUUID(),
          key: props.variableName,
          value: editValue.value,
          is_secret: false
        }]
      )
    });
    
    emit('updated', { key: props.variableName, value: editValue.value });
    close();
  } catch (error) {
    console.error('Failed to save variable:', error);
  }
};

const close = () => {
  emit('close');
};
</script>

<style scoped>
.variable-tooltip {
  position: fixed;
  z-index: 10000;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  min-width: 280px;
  max-width: 400px;
  padding: 0;
  animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-5px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.tooltip-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  border-bottom: 1px solid #3a3a3a;
  background: #252525;
  border-radius: 8px 8px 0 0;
}

.variable-name {
  font-size: 12px;
  font-weight: 600;
  color: #667eea;
  font-family: 'Courier New', monospace;
}

.close-btn {
  background: none;
  border: none;
  color: #888;
  font-size: 20px;
  cursor: pointer;
  padding: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #3a3a3a;
  color: #fff;
}

.tooltip-body {
  padding: 12px;
}

.current-value {
  margin-bottom: 12px;
}

.current-value label,
.edit-section label {
  display: block;
  font-size: 11px;
  color: #999;
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.value-display {
  background: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  padding: 8px 10px;
  font-size: 13px;
  color: #ddd;
  font-family: 'Courier New', monospace;
  word-break: break-all;
}

.edit-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.value-input {
  background: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  padding: 8px 10px;
  font-size: 13px;
  color: #fff;
  font-family: 'Courier New', monospace;
  transition: border-color 0.2s;
}

.value-input:hover {
  border-color: #4a4a4a;
}

.value-input:focus {
  outline: none;
  border-color: #667eea;
}

.value-input::placeholder {
  color: #555;
}

.save-btn {
  background: #667eea;
  border: none;
  border-radius: 4px;
  padding: 8px 16px;
  font-size: 12px;
  font-weight: 600;
  color: #fff;
  cursor: pointer;
  transition: background 0.2s;
}

.save-btn:hover {
  background: #7c8ff0;
}

.save-btn:active {
  background: #5a6fd8;
}
</style>
