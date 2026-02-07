<template>
  <div class="request-history">
    <div class="history-header">
      <h3>Request History</h3>
      <button class="clear-btn" @click="clearHistory" title="Clear History">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M11 1.5v1h3.5a.5.5 0 0 1 0 1h-.538l-.853 10.66A2 2 0 0 1 11.115 16h-6.23a2 2 0 0 1-1.994-1.84L2.038 3.5H1.5a.5.5 0 0 1 0-1H5v-1A1.5 1.5 0 0 1 6.5 0h3A1.5 1.5 0 0 1 11 1.5zm-5 0v1h4v-1a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5z"/>
        </svg>
      </button>
    </div>
    
    <div v-if="historyItems.length === 0" class="empty-history">
      <p>No requests sent yet</p>
    </div>
    
    <div v-else class="history-list">
      <div 
        v-for="item in historyItems" 
        :key="item.id" 
        class="history-item"
      >
        <div class="history-content" @click="$emit('selectHistory', item)">
          <div class="history-meta">
            <span :class="['method-badge', item.method.toLowerCase()]">{{ item.method }}</span>
            <span class="status-code" :class="getStatusClass(item.response_status)">{{ item.response_status }}</span>
          </div>
          <div class="history-url">{{ item.url }}</div>
          <div class="history-time">{{ formatTime(item.created_at) }}</div>
        </div>
        <button class="delete-history-btn" @click.stop="deleteHistoryItem(item.id)" title="Delete">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
            <path d="M11 1.5v1h3.5a.5.5 0 0 1 0 1h-.538l-.853 10.66A2 2 0 0 1 11.115 16h-6.23a2 2 0 0 1-1.994-1.84L2.038 3.5H1.5a.5.5 0 0 1 0-1H5v-1A1.5 1.5 0 0 1 6.5 0h3A1.5 1.5 0 0 1 11 1.5zm-5 0v1h4v-1a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface HistoryItem {
  id: string;
  method: string;
  url: string;
  response_status: number;
  created_at: number;
  // Included for re-hydration logic in parent
  request_headers: string;
  request_params: string;
  request_body: string | null;
  // ... other fields matching backend
}

const props = defineProps<{
  workspaceId: string
}>();

const emit = defineEmits<{
  selectHistory: [item: HistoryItem]
}>();

const historyItems = ref<HistoryItem[]>([]);

const fetchHistory = async () => {
  try {
    const result = await invoke('get_history', { 
        workspaceId: props.workspaceId,
        limit: 50, 
        offset: 0 
    }) as HistoryItem[];
    historyItems.value = result;
  } catch (error) {
    console.error('Failed to fetch history:', error);
  }
};

watch(() => props.workspaceId, () => {
    fetchHistory();
});

const clearHistory = async () => {
  try {
    await invoke('clear_history', { workspaceId: props.workspaceId });
    historyItems.value = [];
  } catch (error) {
    console.error('Failed to clear history:', error);
  }
};

const deleteHistoryItem = async (id: string) => {
  try {
    await invoke('delete_history_entry', { historyId: id });
    // Remove from local array
    historyItems.value = historyItems.value.filter(item => item.id !== id);
  } catch (error) {
    console.error('Failed to delete history item:', error);
  }
};

const getStatusClass = (status: number) => {
  if (status >= 200 && status < 300) return 'status-success';
  if (status >= 300 && status < 400) return 'status-redirect';
  if (status >= 400 && status < 500) return 'status-client-error';
  return 'status-server-error';
};

const formatTime = (ts: number) => {
  return new Date(ts).toLocaleTimeString();
};

onMounted(() => {
  fetchHistory();
});

defineExpose({
  addEntry: () => {
    // Refresh history to get the latest entry with correct ID from DB
    // Alternatively, accept the entry if we want to be optimistic
    fetchHistory();
  },
  refresh: fetchHistory
});
</script>

<style scoped>
.request-history {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #1a1a1a;
  border-left: 1px solid #2a2a2a;
}

.history-header {
  padding: 12px 16px;
  border-bottom: 1px solid #2a2a2a;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.history-header h3 {
  margin: 0;
  font-size: 14px;
  color: #fff;
  font-weight: 600;
}

.clear-btn {
  background: transparent;
  border: none;
  color: #666;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.clear-btn:hover {
  color: #ff4d4d;
  background: rgba(255, 77, 77, 0.1);
}

.history-list {
  flex: 1;
  overflow-y: auto;
}


.history-item {
  padding: 12px 16px;
  border-bottom: 1px solid #2a2a2a;
  transition: background 0.2s;
  display: flex;
  align-items: center;
  gap: 8px;
}

.history-item:hover {
  background: #252525;
}

.history-content {
  flex: 1;
  cursor: pointer;
}

.delete-history-btn {
  background: transparent;
  border: none;
  color: #666;
  cursor: pointer;
  padding: 6px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: all 0.2s;
}

.history-item:hover .delete-history-btn {
  opacity: 1;
}

.delete-history-btn:hover {
  color: #ff4d4d;
  background: rgba(255, 77, 77, 0.1);
}


.history-meta {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
}

.method-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  text-transform: uppercase;
}

.method-badge.get { color: #61affe; background: rgba(97, 175, 254, 0.1); }
.method-badge.post { color: #49cc90; background: rgba(73, 204, 144, 0.1); }
.method-badge.put { color: #fca130; background: rgba(252, 161, 48, 0.1); }
.method-badge.delete { color: #f93e3e; background: rgba(249, 62, 62, 0.1); }

.status-code {
  font-size: 11px;
  font-weight: 600;
}

.status-success { color: #49cc90; }
.status-redirect { color: #fca130; }
.status-client-error { color: #f93e3e; }
.status-server-error { color: #d63031; }

.history-url {
  font-size: 12px;
  color: #ddd;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.history-time {
  font-size: 10px;
  color: #666;
  text-align: right;
}

.empty-history {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #555;
  font-size: 13px;
}
</style>
