<template>
  <div class="response-viewer">
    <div v-if="error" class="error-state">
      <div class="error-icon">⚠️</div>
      <h3>Request Failed</h3>
      <p class="error-message">{{ error }}</p>
      <button class="clear-btn" @click="clearResponse">Clear</button>
    </div>

    <div v-else-if="!response" class="empty-state">
      <div class="empty-icon">📡</div>
      <h3>No Response Yet</h3>
      <p>Send a request to see the response here</p>
    </div>

    <div v-else class="response-content">
      <div class="response-header">
        <div class="status-section">
          <div :class="['status-badge', getStatusClass(response.status)]">
            {{ response.status }} {{ response.status_text }}
          </div>
          <div class="response-meta">
            <span class="meta-item">
              <span class="meta-label">Time:</span>
              <span class="meta-value">{{ response.time_ms }}ms</span>
            </span>
            <span class="meta-item">
              <span class="meta-label">Size:</span>
              <span class="meta-value">{{ formatSize(response.size_bytes) }}</span>
            </span>
          </div>
        </div>
        
        <div class="header-actions">
          <button class="copy-btn" @click="copyResponse" title="Copy Response">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M4 1.5H3a2 2 0 00-2 2V14a2 2 0 002 2h10a2 2 0 002-2V3.5a2 2 0 00-2-2h-1v1h1a1 1 0 011 1V14a1 1 0 01-1 1H3a1 1 0 01-1-1V3.5a1 1 0 011-1h1v-1z"/>
              <path d="M9.5 1a.5.5 0 01.5.5v1a.5.5 0 01-.5.5h-3a.5.5 0 01-.5-.5v-1a.5.5 0 01.5-.5h3z"/>
            </svg>
            Copy
          </button>
        </div>
      </div>

      <div class="response-tabs">
        <div class="tabs-left">
          <button 
            v-for="tab in responseTabs" 
            :key="tab.id"
            :class="['tab-btn', { active: activeTab === tab.id }]"
            @click="activeTab = tab.id"
          >
            {{ tab.label }}
          </button>
        </div>
        
        <div v-if="activeTab === 'body' && isJson" class="view-options">
          <button 
            :class="['mode-btn', { active: viewMode === 'pretty' }]"
            @click="viewMode = 'pretty'"
          >
            Pretty
          </button>
          <button 
            :class="['mode-btn', { active: viewMode === 'raw' }]"
            @click="viewMode = 'raw'"
          >
            Raw
          </button>
          <button 
            :class="['mode-btn', { active: viewMode === 'tree' }]"
            @click="viewMode = 'tree'"
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor" style="margin-right:3px">
              <path d="M1 1h4v4H1V1zm6 0h4v4H7V1zm-6 6h4v4H1V7zm6 0h4v4H7V7zm6-6h4v4h-4V1zm0 6h4v4h-4V7z"/>
            </svg>
            Tree
          </button>
        </div>
      </div>

      <div class="response-body">
        <!-- Body Tab -->
        <div v-show="activeTab === 'body'" class="tab-content body-tab">
          <!-- Tree View -->
          <div v-if="viewMode === 'tree' && isJson" class="tree-container">
            <JsonTreeViewer :data="getBodyText()" />
          </div>
          <!-- Pretty / Raw View -->
          <div v-else class="code-container">
            <pre :class="['response-pre', `language-${detectedLanguage}`]"><code ref="codeElement">{{ getFormattedBody() }}</code></pre>
          </div>
        </div>

        <!-- Headers Tab -->
        <div v-if="activeTab === 'headers'" class="tab-content">
          <div class="headers-list">
            <div v-for="(header, index) in response.headers" :key="index" class="header-row">
              <span class="header-key">{{ header.key }}</span>
              <span class="header-value">{{ header.value }}</span>
            </div>
          </div>
        </div>

        <!-- Cookies Tab -->
        <div v-if="activeTab === 'cookies'" class="tab-content">
          <div class="empty-state-small">
            No cookies in this response
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue';
import Prism from 'prismjs';
import 'prismjs/components/prism-json';
import 'prismjs/components/prism-javascript';
import 'prismjs/components/prism-markup'; // for HTML/XML
import 'prismjs/themes/prism-tomorrow.css';
import JsonTreeViewer from './JsonTreeViewer.vue';

interface ResponseData {
  status: number;
  status_text: string;
  time_ms: number;
  size_bytes: number;
  body: any;
  headers: Array<{ key: string; value: string; enabled: boolean }>;
  content_type?: string;
  content_encoding?: string;
}

const response = ref<ResponseData | null>(null);
const error = ref<string | null>(null);
const codeElement = ref<HTMLElement | null>(null);

const activeTab = ref('body');
const viewMode = ref('pretty'); // 'pretty' or 'raw'
const responseTabs = [
  { id: 'body', label: 'Body' },
  { id: 'headers', label: 'Headers' },
  { id: 'cookies', label: 'Cookies' },
];

const isJson = computed(() => {
  if (!response.value) return false;
  const contentType = response.value.headers.find(h => h.key.toLowerCase() === 'content-type')?.value || '';
  return contentType.toLowerCase().includes('application/json');
});

const detectedLanguage = computed(() => {
  if (!response.value) return 'none';
  const contentType = response.value.headers.find(h => h.key.toLowerCase() === 'content-type')?.value || '';
  if (contentType.includes('json')) return 'json';
  if (contentType.includes('html')) return 'markup';
  if (contentType.includes('xml')) return 'markup';
  if (contentType.includes('javascript')) return 'javascript';
  return 'none';
});

const highlight = () => {
  nextTick(() => {
    if (codeElement.value) {
      Prism.highlightElement(codeElement.value);
    }
  });
};

watch([response, activeTab, viewMode], () => {
  if (activeTab.value === 'body' && response.value) {
    highlight();
  }
});

const getStatusClass = (status: number) => {
  if (status >= 200 && status < 300) return 'success';
  if (status >= 300 && status < 400) return 'redirect';
  if (status >= 400 && status < 500) return 'client-error';
  if (status >= 500) return 'server-error';
  return '';
};

const formatSize = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
};

const getFormattedBody = () => {
  const rawText = getBodyText();
  if (!rawText) return '';
  
  if (isJson.value && viewMode.value === 'pretty') {
    try {
      const parsed = JSON.parse(rawText);
      return JSON.stringify(parsed, null, 2);
    } catch (e) {
      return rawText;
    }
  }
  return rawText;
};

const getBodyText = () => {
  if (!response.value) return '';
  
  const body = response.value.body;
  
  // Handle different body types from backend
  if (body.type === 'Text') {
    return body.content;
  } else if (body.type === 'Binary') {
    return `Binary data (hex preview):\n${body.preview_hex}`;
  } else if (body.type === 'Truncated') {
    return `${body.content}\n\n... (Response truncated, original size: ${formatSize(body.original_size)})`;
  }
  
  return '';
};

const copyResponse = () => {
  const text = getFormattedBody();
  if (text) {
    navigator.clipboard.writeText(text);
  }
};

// Method to handle response from parent/backend
const handleResponse = (result: any) => {
  error.value = null;
  activeTab.value = 'body'; // Switch to body by default on new response
  viewMode.value = 'pretty';
  
  if (result.status === 'Success') {
    // Extract response from Success variant
    const resp = result.response;
    response.value = {
      status: resp.status,
      status_text: resp.status_text,
      time_ms: resp.time_ms,
      size_bytes: resp.size_bytes,
      body: resp.body,
      headers: resp.headers,
      content_type: resp.content_type,
      content_encoding: resp.content_encoding,
    };
  } else if (result.status === 'Failed') {
    // Handle error
    error.value = result.error.type || 'Request failed';
    response.value = null;
  }
};

const handleError = (err: any) => {
  error.value = err.toString();
  response.value = null;
};

const clearResponse = () => {
  response.value = null;
  error.value = null;
};

// Expose methods for parent component
defineExpose({
  handleResponse,
  handleError,
  clearResponse,
});
</script>

<style scoped>
.response-viewer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1a1a1a;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: #666;
}

.empty-icon {
  font-size: 64px;
  opacity: 0.5;
}

.empty-state h3 {
  font-size: 18px;
  font-weight: 600;
  color: #888;
  margin: 0;
}

.empty-state p {
  font-size: 14px;
  color: #666;
  margin: 0;
}

.error-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: #ff6b6b;
}

.error-icon {
  font-size: 64px;
  opacity: 0.8;
}

.error-state h3 {
  font-size: 18px;
  font-weight: 600;
  color: #ff6b6b;
  margin: 0;
}

.error-message {
  font-size: 14px;
  color: #ff6b6b;
  margin: 0;
  max-width: 400px;
  text-align: center;
  word-break: break-word;
}

.clear-btn {
  margin-top: 8px;
  padding: 8px 24px;
  background: rgba(255, 107, 107, 0.1);
  border: 1px solid #ff6b6b;
  border-radius: 6px;
  color: #ff6b6b;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.clear-btn:hover {
  background: rgba(255, 107, 107, 0.2);
}

.response-content {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.response-header {
  padding: 16px;
  border-bottom: 1px solid #2a2a2a;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-section {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-badge {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  border: 1px solid;
}

.status-badge.success {
  background: rgba(46, 213, 115, 0.1);
  border-color: #2ed573;
  color: #2ed573;
}

.status-badge.redirect {
  background: rgba(255, 193, 7, 0.1);
  border-color: #ffc107;
  color: #ffc107;
}

.status-badge.client-error {
  background: rgba(255, 107, 107, 0.1);
  border-color: #ff6b6b;
  color: #ff6b6b;
}

.status-badge.server-error {
  background: rgba(238, 82, 83, 0.1);
  border-color: #ee5253;
  color: #ee5253;
}

.response-meta {
  display: flex;
  gap: 16px;
}

.meta-item {
  font-size: 12px;
  color: #888;
}

.meta-label {
  margin-right: 4px;
}

.meta-value {
  color: #aaa;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.copy-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #888;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.copy-btn:hover {
  border-color: #667eea;
  color: #667eea;
  background: rgba(102, 126, 234, 0.05);
}

.tree-container {
  height: 100%;
  border: 1px solid #2a2a2a;
  border-radius: 8px;
  margin: 16px;
  overflow: hidden;
}

.response-tabs {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid #2a2a2a;
  background: #151515;
}

.tabs-left {
  display: flex;
  gap: 4px;
}

.view-options {
  display: flex;
  background: #1e1e1e;
  padding: 2px;
  border-radius: 6px;
  border: 1px solid #333;
}

.mode-btn {
  padding: 4px 12px;
  font-size: 11px;
  border: none;
  background: transparent;
  color: #666;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  font-weight: 500;
}

.mode-btn:hover {
  color: #888;
}

.mode-btn.active {
  background: #2a2a2a;
  color: #667eea;
  box-shadow: 0 2px 4px rgba(0,0,0,0.2);
}

.tab-btn {
  padding: 12px 20px;
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
  color: #aaa;
}

.tab-btn.active {
  color: #667eea;
  border-bottom-color: #667eea;
}

.response-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.body-tab {
  height: 100%;
}

.code-container {
  height: 100%;
  padding: 16px;
}

.response-pre {
  margin: 0 !important;
  padding: 16px !important;
  background: #1e1e1e !important;
  border: 1px solid #2a2a2a !important;
  border-radius: 8px !important;
  font-family: 'Fira Code', 'Cascadia Code', 'Consolas', monospace !important;
  font-size: 13px !important;
  line-height: 1.6 !important;
  height: 100% !important;
  overflow: auto !important;
}

.response-pre code {
  text-shadow: none !important;
}

/* Prism Overrides for Premium Look */
:deep(.token.property) { color: #667eea; }
:deep(.token.string) { color: #98c379; }
:deep(.token.number) { color: #d19a66; }
:deep(.token.boolean) { color: #56b6c2; }
:deep(.token.operator) { color: #abb2bf; }
:deep(.token.punctuation) { color: #abb2bf; }

.tab-content {
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

.headers-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
}

.header-row {
  display: grid;
  grid-template-columns: 200px 1fr;
  gap: 16px;
  padding: 12px 16px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  font-size: 13px;
}

.header-key {
  color: #667eea;
  font-weight: 600;
  word-break: break-word;
}

.header-value {
  color: #aaa;
  word-break: break-word;
}

.empty-state-small {
  padding: 40px;
  text-align: center;
  color: #666;
  font-size: 14px;
}
</style>
