<template>
  <Transition name="modal">
    <div v-if="isOpen" class="modal-overlay" @click.self="cancel">
      <div class="modal-container" :class="{ danger: variant === 'danger' }">
        <div class="modal-header">
          <h3>{{ title }}</h3>
          <button class="close-btn" @click="cancel">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        </div>
        
        <div class="modal-body">
          <p v-if="message" class="modal-message">{{ message }}</p>
          <div v-if="inputType" class="input-wrapper">
            <input 
              ref="inputRef"
              v-model="inputValue" 
              :type="inputType" 
              :placeholder="placeholder"
              class="modal-input"
              @keyup.enter="confirm"
            />
          </div>
        </div>
        
        <div class="modal-footer">
          <button class="modal-btn cancel" @click="cancel">{{ cancelText }}</button>
          <button 
            class="modal-btn confirm" 
            :class="variant"
            @click="confirm"
          >
            {{ confirmText }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

const props = defineProps({
  isOpen: Boolean,
  title: String,
  message: String,
  initialValue: {
    type: String,
    default: ''
  },
  inputType: {
    type: String,
    default: '' // '' for no input, 'text', etc.
  },
  placeholder: String,
  confirmText: {
    type: String,
    default: 'Confirm'
  },
  cancelText: {
    type: String,
    default: 'Cancel'
  },
  variant: {
    type: String,
    default: 'primary' // 'primary', 'danger'
  }
});

const emit = defineEmits(['confirm', 'cancel']);

const inputValue = ref(props.initialValue);
const inputRef = ref<HTMLInputElement | null>(null);

watch(() => props.isOpen, (newVal) => {
  if (newVal) {
    inputValue.value = props.initialValue;
    if (props.inputType) {
      nextTick(() => {
        inputRef.value?.focus();
        inputRef.value?.select();
      });
    }
  }
});

const confirm = () => {
  emit('confirm', inputValue.value);
};

const cancel = () => {
  emit('cancel');
};
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.modal-container {
  background: #1e1e1e;
  border: 1px solid #333;
  border-radius: 12px;
  width: 400px;
  max-width: 90%;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5), 0 10px 10px -5px rgba(0, 0, 0, 0.4);
  overflow: hidden;
  animation: modal-in 0.3s ease-out;
}

.modal-container.danger {
  border-top: 4px solid #ff4d4d;
}

@keyframes modal-in {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.modal-header {
  padding: 16px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid #2a2a2a;
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #fff;
}

.close-btn {
  background: transparent;
  border: none;
  color: #666;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
}

.close-btn:hover {
  background: #2a2a2a;
  color: #fff;
}

.modal-body {
  padding: 20px;
}

.modal-message {
  color: #aaa;
  font-size: 14px;
  margin: 0 0 16px 0;
  line-height: 1.5;
}

.input-wrapper {
  margin-top: 8px;
}

.modal-input {
  width: 100%;
  background: #121212;
  border: 1px solid #333;
  border-radius: 6px;
  padding: 10px 12px;
  color: #fff;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.modal-input:focus {
  border-color: #667eea;
}

.modal-footer {
  padding: 16px 20px;
  background: #252525;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.modal-btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.modal-btn.cancel {
  background: transparent;
  color: #888;
}

.modal-btn.cancel:hover {
  background: #333;
  color: #fff;
}

.modal-btn.confirm.primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}

.modal-btn.confirm.primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.modal-btn.confirm.danger {
  background: #ff4d4d;
  color: #fff;
}

.modal-btn.confirm.danger:hover {
  background: #ff3333;
  box-shadow: 0 4px 12px rgba(255, 77, 77, 0.3);
}

/* Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
