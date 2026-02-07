<template>
  <div class="tabs-container">
    <div 
      v-for="tab in tabs" 
      :key="tab.id" 
      class="tab-item"
      :class="{ active: activeTabId === tab.id, dirty: tab.isDirty }"
      @click="$emit('switch-tab', tab.id)"
    >
      <span class="method" :class="tab.method">{{ tab.method }}</span>
      <span class="name">{{ tab.name || 'New Request' }}</span>
      <span v-if="tab.isDirty" class="dirty-indicator"></span>
      <button class="close-btn" @click.stop="$emit('close-tab', tab.id)">×</button>
    </div>
    <div class="new-tab-btn" @click="$emit('new-tab')">+</div>
  </div>
</template>


<script setup lang="ts">
interface Tab {
  id: string;
  name: string;
  method: string;
  isDirty: boolean;
}

defineProps<{
  tabs: Tab[];
  activeTabId: string | null;
}>();

defineEmits<{
  'switch-tab': [id: string];
  'close-tab': [id: string];
  'new-tab': [];
}>();
</script>

<style scoped>
.tabs-container {
  display: flex;
  background: #111;
  border-bottom: 1px solid #2a2a2a;
  overflow-x: auto;
  height: 40px;
  align-items: center;
  padding: 0 4px;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: 32px;
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-bottom: none;
  font-size: 12px;
  cursor: pointer;
  color: #888;
  border-radius: 6px 6px 0 0;
  margin-right: 2px;
  min-width: 120px;
  max-width: 200px;
  user-select: none;
  position: relative;
}

.tab-item:hover {
  background: #222;
}

.tab-item.active {
  background: #2a2a2a;
  color: #fff;
  border-color: #3a3a3a;
  border-bottom: 1px solid #2a2a2a; /* Match container border color? No, usually hides it. */
  border-bottom-color: #2a2a2a; /* Wait, if container has border-bottom, active tab should probably cover it? */
  margin-bottom: -1px; /* Overlap border */
  z-index: 1;
}

.method {
  font-weight: 600;
  font-size: 10px;
}

.method.GET { color: #667eea; }
.method.POST { color: #48bb78; }
.method.PUT { color: #ed8936; }
.method.DELETE { color: #f56565; }
.method.PATCH { color: #ecc94b; }

.name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dirty-indicator {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #ed8936;
}

.close-btn {
  background: transparent;
  border: none;
  color: #666;
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  padding: 0 2px;
  border-radius: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s;
}

.tab-item:hover .close-btn {
  opacity: 1;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.new-tab-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #888;
  font-size: 18px;
  border-radius: 4px;
}

.new-tab-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #fff;
}
</style>
