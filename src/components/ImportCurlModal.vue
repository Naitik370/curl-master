<template>
  <Transition name="modal">
    <div v-if="isOpen" class="modal-overlay" @click.self="cancel">
      <div class="modal-container">
        <div class="modal-header">
          <div class="title-with-icon">
            <div class="curl-icon">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <path d="M4 17l6-6-6-6M12 19h8"/>
              </svg>
            </div>
            <h3>Import from cURL</h3>
          </div>
          <button class="close-btn" @click="cancel">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        </div>
        
        <div class="modal-body">
          <p class="modal-message">Paste a cURL command below to automatically populate the request builder. We'll handle the method, URL, headers, and body for you.</p>
          <div class="textarea-wrapper">
            <textarea 
              ref="textareaRef"
              v-model="curlCommand" 
              placeholder="curl -X POST https://api.example.com/data -H 'Content-Type: application/json' -d '{ &quot;key&quot;: &quot;value&quot; }'"
              class="modal-textarea"
              spellcheck="false"
            ></textarea>
          </div>
          <div v-if="error" class="error-msg">
            {{ error }}
          </div>
        </div>
        
        <div class="modal-footer">
          <button class="modal-btn cancel" @click="cancel">Cancel</button>
          <button 
            class="modal-btn confirm" 
            :disabled="!curlCommand.trim()"
            @click="confirm"
          >
            Import Request
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'confirm', command: string): void;
  (e: 'cancel'): void;
}>();

const curlCommand = ref('');
const error = ref('');
const textareaRef = ref<HTMLTextAreaElement | null>(null);

watch(() => props.isOpen, (newVal) => {
  if (newVal) {
    curlCommand.value = '';
    error.value = '';
    nextTick(() => {
      textareaRef.value?.focus();
    });
  }
});

const confirm = () => {
  if (curlCommand.value.trim()) {
    if (!curlCommand.value.trim().toLowerCase().startsWith('curl')) {
      error.value = 'Command must start with "curl"';
      return;
    }
    emit('confirm', curlCommand.value.trim());
  }
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
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.modal-container {
  background: #141414;
  border: 1px solid #333;
  border-radius: 16px;
  width: 600px;
  max-width: 90%;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  animation: modal-in 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes modal-in {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(20px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.modal-header {
  padding: 20px 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid #222;
}

.title-with-icon {
  display: flex;
  align-items: center;
  gap: 12px;
}

.curl-icon {
  width: 32px;
  height: 32px;
  background: rgba(102, 126, 234, 0.1);
  border: 1px solid rgba(102, 126, 234, 0.3);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #667eea;
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #fff;
}

.close-btn {
  background: transparent;
  border: none;
  color: #555;
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #222;
  color: #fff;
}

.modal-body {
  padding: 24px;
}

.modal-message {
  color: #888;
  font-size: 14px;
  margin: 0 0 20px 0;
  line-height: 1.6;
}

.textarea-wrapper {
  background: #0d0d0d;
  border: 1px solid #333;
  border-radius: 10px;
  padding: 4px;
  transition: border-color 0.2s;
}

.textarea-wrapper:focus-within {
  border-color: #667eea;
}

.modal-textarea {
  width: 100%;
  height: 200px;
  background: transparent;
  border: none;
  padding: 12px;
  color: #e0e0e0;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.5;
  resize: none;
  outline: none;
}

.error-msg {
  color: #ff4d4d;
  font-size: 12px;
  margin-top: 12px;
  padding: 8px 12px;
  background: rgba(255, 77, 77, 0.1);
  border-radius: 6px;
}

.modal-footer {
  padding: 20px 24px;
  background: #1a1a1a;
  display: flex;
  justify-content: flex-end;
  gap: 16px;
}

.modal-btn {
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.modal-btn.cancel {
  background: transparent;
  color: #777;
}

.modal-btn.cancel:hover {
  background: #2a2a2a;
  color: #fff;
}

.modal-btn.confirm {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}

.modal-btn.confirm:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.3);
}

.modal-btn.confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
