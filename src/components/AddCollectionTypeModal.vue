<template>
  <Transition name="modal">
    <div v-if="isOpen" class="modal-overlay" @click="close">
      <div class="modal-container" @click.stop>
        <div class="modal-header">
          <h3>Add Collection</h3>
          <button class="close-btn" @click="close">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        </div>
        
        <div class="modal-body">
          <p class="subtitle">How would you like to start your new collection?</p>
          
          <div class="options-grid">
            <div class="option-card" @click="select('blank')">
              <div class="option-icon blank">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M12 5v14M5 12h14" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </div>
              <div class="option-info">
                <h4>Blank Collection</h4>
                <p>Create a fresh collection from scratch</p>
              </div>
              <div class="option-arrow">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9 18l6-6-6-6"/>
                </svg>
              </div>
            </div>

            <div class="option-card" @click="select('import')">
              <div class="option-icon import">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="#FF6C37">
                  <path d="M12 0C5.373 0 0 5.373 0 12s5.373 12 12 12 12-5.373 12-12S18.627 0 12 0zm0 18.667c-3.682 0-6.667-2.985-6.667-6.667S8.318 5.333 12 5.333 18.667 8.318 18.667 12 15.682 18.667 12 18.667z"/>
                  <path d="M12 7.333c-2.577 0-4.667 2.09-4.667 4.667s2.09 4.667 4.667 4.667 4.667-2.09 4.667-4.667-2.09-4.667-4.667-4.667zm3.179 6.221l-1.077-1.077c-.126-.126-.33-.126-.456 0l-.513.513c-.126.126-.126.33 0 .456l1.077 1.077c.126.126.33.126.456 0l.513-.513c.126-.126.126-.33 0-.456zm-.513-5.316l-1.077 1.077c-.126.126-.126.33 0 .456l.513.513c.126.126.33.126.456 0l1.077-1.077c.126-.126.126-.33 0-.456l-.513-.513c-.126-.126-.33-.126-.456 0zm-5.332 0l-.513.513c-.126.126-.126.33 0 .456l1.077 1.077c.126.126.33.126.456 0l.513-.513c.126-.126.126-.33 0-.456l-1.077-1.077c-.126-.126-.33-.126-.456 0zm-1.077 5.316l.513.513c.126.126.33.126.456 0l1.077-1.077c.126-.126.126-.33 0-.456l-.513-.513c-.126-.126-.33-.126-.456 0l-1.077 1.077c-.126.126-.126.33 0 .456z"/>
                </svg>
              </div>
              <div class="option-info">
                <h4>Import Postman</h4>
                <p>Import from a Postman Collection JSON file</p>
              </div>
              <div class="option-arrow">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9 18l6-6-6-6"/>
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  close: [];
  select: [type: 'blank' | 'import'];
}>();

const close = () => {
  emit('close');
};

const select = (type: 'blank' | 'import') => {
  emit('select', type);
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
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 16px;
  width: 500px;
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
  border-bottom: 1px solid #2a2a2a;
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
  color: #666;
  cursor: pointer;
  padding: 8px;
  border-radius: 8px;
  transition: all 0.2s;
  display: flex;
}

.close-btn:hover {
  background: #2a2a2a;
  color: #fff;
}

.modal-body {
  padding: 28px 24px;
}

.subtitle {
  color: #888;
  font-size: 14px;
  margin: 0 0 24px 0;
}

.options-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.option-card {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 20px;
  background: #222;
  border: 1px solid #333;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.option-card:hover {
  background: #2a2a2a;
  border-color: #444;
  transform: translateY(-2px);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
}

.option-card:active {
  transform: translateY(0);
}

.option-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #1a1a1a;
  flex-shrink: 0;
}

.option-icon.blank {
  color: #667eea;
  background: rgba(102, 126, 234, 0.1);
}

.option-icon.import {
  background: rgba(255, 108, 55, 0.1);
}

.option-info {
  flex: 1;
}

.option-info h4 {
  margin: 0 0 4px 0;
  color: #fff;
  font-size: 16px;
  font-weight: 600;
}

.option-info p {
  margin: 0;
  color: #777;
  font-size: 13px;
  line-height: 1.4;
}

.option-arrow {
  color: #444;
  transition: transform 0.2s, color 0.2s;
}

.option-card:hover .option-arrow {
  color: #888;
  transform: translateX(4px);
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
