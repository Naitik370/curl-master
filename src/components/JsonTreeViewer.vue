<template>
  <div class="json-tree-viewer">
    <div class="tree-toolbar">
      <button class="tree-action-btn" @click="expandAll" title="Expand All">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1 1v14h14V1H1zm1 1h12v12H2V2zm2 3v1h8V5H4zm0 3v1h8V8H4zm0 3v1h5v-1H4z"/>
        </svg>
        Expand All
      </button>
      <button class="tree-action-btn" @click="collapseAll" title="Collapse All">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1 1v14h14V1H1zm1 1h12v12H2V2zm2 3v1h8V5H4z"/>
        </svg>
        Collapse All
      </button>
      <button class="tree-action-btn" @click="copyPath" title="Copy Path" v-if="selectedPath">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M4 1.5H3a2 2 0 00-2 2V14a2 2 0 002 2h10a2 2 0 002-2V3.5a2 2 0 00-2-2h-1v1h1a1 1 0 011 1V14a1 1 0 01-1 1H3a1 1 0 01-1-1V3.5a1 1 0 011-1h1v-1z"/>
          <path d="M9.5 1a.5.5 0 01.5.5v1a.5.5 0 01-.5.5h-3a.5.5 0 01-.5-.5v-1a.5.5 0 01.5-.5h3z"/>
        </svg>
        {{ selectedPath }}
      </button>
      <span class="tree-stats">{{ nodeCount }} nodes</span>
    </div>
    <div class="tree-content">
      <JsonNode 
        :data="parsedData" 
        :path="'$'" 
        :depth="0"
        :expandedPaths="expandedPaths"
        @toggle="toggleNode"
        @select-path="handleSelectPath"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import JsonNode from './JsonNode.vue';

const props = defineProps<{
  data: string;
}>();

const expandedPaths = ref<Set<string>>(new Set());
const selectedPath = ref('');

const parsedData = computed(() => {
  try {
    return JSON.parse(props.data);
  } catch {
    return null;
  }
});

const nodeCount = computed(() => {
  return countNodes(parsedData.value);
});

function countNodes(obj: any): number {
  if (obj === null || obj === undefined) return 1;
  if (typeof obj !== 'object') return 1;
  let count = 1;
  if (Array.isArray(obj)) {
    for (const item of obj) count += countNodes(item);
  } else {
    for (const key of Object.keys(obj)) count += countNodes(obj[key]);
  }
  return count;
}

// Auto-expand first 2 levels on data change
watch(parsedData, () => {
  expandedPaths.value = new Set();
  autoExpand(parsedData.value, '$', 0, 2);
}, { immediate: true });

function autoExpand(obj: any, path: string, depth: number, maxDepth: number) {
  if (depth >= maxDepth) return;
  if (obj === null || typeof obj !== 'object') return;
  expandedPaths.value.add(path);
  if (Array.isArray(obj)) {
    obj.forEach((_, i) => autoExpand(obj[i], `${path}[${i}]`, depth + 1, maxDepth));
  } else {
    Object.keys(obj).forEach(key => autoExpand(obj[key], `${path}.${key}`, depth + 1, maxDepth));
  }
}

function toggleNode(path: string) {
  if (expandedPaths.value.has(path)) {
    expandedPaths.value.delete(path);
  } else {
    expandedPaths.value.add(path);
  }
  // Force reactivity
  expandedPaths.value = new Set(expandedPaths.value);
}

function expandAll() {
  expandedPaths.value = new Set();
  autoExpand(parsedData.value, '$', 0, 100);
  expandedPaths.value = new Set(expandedPaths.value);
}

function collapseAll() {
  expandedPaths.value = new Set();
}

function handleSelectPath(path: string) {
  selectedPath.value = path;
}

function copyPath() {
  if (selectedPath.value) {
    navigator.clipboard.writeText(selectedPath.value);
  }
}
</script>

<style scoped>
.json-tree-viewer {
  display: flex;
  flex-direction: column;
  height: 100%;
  font-family: 'Fira Code', 'Cascadia Code', 'Consolas', monospace;
  font-size: 13px;
}

.tree-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-bottom: 1px solid #2a2a2a;
  background: #1a1a1a;
  flex-shrink: 0;
}

.tree-action-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: #252525;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  color: #999;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s;
  font-family: inherit;
}

.tree-action-btn:hover {
  border-color: #667eea;
  color: #667eea;
  background: rgba(102, 126, 234, 0.08);
}

.tree-stats {
  margin-left: auto;
  font-size: 11px;
  color: #555;
}

.tree-content {
  flex: 1;
  overflow: auto;
  padding: 12px 16px;
  background: #1e1e1e;
}
</style>
