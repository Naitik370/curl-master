<template>
  <div class="json-node" :style="{ paddingLeft: depth > 0 ? '20px' : '0' }">
    <!-- Object node -->
    <template v-if="isObject">
      <div class="node-row" :class="{ selected: isSelected }" @click.stop="$emit('select-path', path)">
        <button class="toggle-btn" @click.stop="$emit('toggle', path)">
          <svg :class="['caret', { open: isExpanded }]" width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
            <path d="M3 2l4 3-4 3V2z"/>
          </svg>
        </button>
        <span v-if="label !== undefined" class="node-key">{{ label }}</span>
        <span v-if="label !== undefined" class="colon">:&nbsp;</span>
        <span class="bracket">{</span>
        <span v-if="!isExpanded" class="collapsed-preview">{{ objectPreview }}<span class="bracket">}</span></span>
        <span v-if="isExpanded" class="item-count">{{ Object.keys(data).length }} {{ Object.keys(data).length === 1 ? 'key' : 'keys' }}</span>
      </div>
      <template v-if="isExpanded">
        <JsonNode
          v-for="key in Object.keys(data)"
          :key="key"
          :data="data[key]"
          :label="key"
          :path="`${path}.${key}`"
          :depth="depth + 1"
          :expandedPaths="expandedPaths"
          @toggle="(p: string) => $emit('toggle', p)"
          @select-path="(p: string) => $emit('select-path', p)"
        />
        <div class="bracket-close" :style="{ paddingLeft: '20px' }">}</div>
      </template>
    </template>

    <!-- Array node -->
    <template v-else-if="isArray">
      <div class="node-row" :class="{ selected: isSelected }" @click.stop="$emit('select-path', path)">
        <button class="toggle-btn" @click.stop="$emit('toggle', path)">
          <svg :class="['caret', { open: isExpanded }]" width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
            <path d="M3 2l4 3-4 3V2z"/>
          </svg>
        </button>
        <span v-if="label !== undefined" class="node-key">{{ label }}</span>
        <span v-if="label !== undefined" class="colon">:&nbsp;</span>
        <span class="bracket">[</span>
        <span v-if="!isExpanded" class="collapsed-preview">{{ arrayPreview }}<span class="bracket">]</span></span>
        <span v-if="isExpanded" class="item-count">{{ data.length }} {{ data.length === 1 ? 'item' : 'items' }}</span>
      </div>
      <template v-if="isExpanded">
        <JsonNode
          v-for="(item, index) in data"
          :key="index"
          :data="item"
          :label="String(index)"
          :path="`${path}[${index}]`"
          :depth="depth + 1"
          :expandedPaths="expandedPaths"
          :isArrayItem="true"
          @toggle="(p: string) => $emit('toggle', p)"
          @select-path="(p: string) => $emit('select-path', p)"
        />
        <div class="bracket-close" :style="{ paddingLeft: '20px' }">]</div>
      </template>
    </template>

    <!-- Primitive node -->
    <template v-else>
      <div class="node-row leaf" :class="{ selected: isSelected }" @click.stop="$emit('select-path', path)">
        <span class="leaf-spacer"></span>
        <span v-if="label !== undefined" :class="isArrayItem ? 'node-index' : 'node-key'">{{ label }}</span>
        <span v-if="label !== undefined" class="colon">:&nbsp;</span>
        <span :class="valueClass">{{ displayValue }}</span>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  data: any;
  label?: string;
  path: string;
  depth: number;
  expandedPaths: Set<string>;
  isArrayItem?: boolean;
}>();

defineEmits<{
  (e: 'toggle', path: string): void;
  (e: 'select-path', path: string): void;
}>();

const isObject = computed(() => props.data !== null && typeof props.data === 'object' && !Array.isArray(props.data));
const isArray = computed(() => Array.isArray(props.data));
const isExpanded = computed(() => props.expandedPaths.has(props.path));
const isSelected = computed(() => false); // Could be connected to selectedPath from parent

const valueClass = computed(() => {
  if (props.data === null) return 'value-null';
  switch (typeof props.data) {
    case 'string': return 'value-string';
    case 'number': return 'value-number';
    case 'boolean': return 'value-boolean';
    default: return 'value-other';
  }
});

const displayValue = computed(() => {
  if (props.data === null) return 'null';
  if (typeof props.data === 'string') return `"${props.data}"`;
  return String(props.data);
});

const objectPreview = computed(() => {
  if (!isObject.value) return '';
  const keys = Object.keys(props.data);
  if (keys.length === 0) return ' ';
  const preview = keys.slice(0, 3).map(k => `${k}: …`).join(', ');
  return ` ${preview}${keys.length > 3 ? ', …' : ''} `;
});

const arrayPreview = computed(() => {
  if (!isArray.value) return '';
  if (props.data.length === 0) return ' ';
  return ` ${props.data.length} items `;
});
</script>

<style scoped>
.json-node {
  line-height: 1.8;
}

.node-row {
  display: inline-flex;
  align-items: center;
  cursor: default;
  border-radius: 3px;
  padding: 0 4px;
  transition: background 0.1s;
  width: fit-content;
  min-height: 22px;
}

.node-row:hover {
  background: rgba(102, 126, 234, 0.06);
}

.node-row.selected {
  background: rgba(102, 126, 234, 0.12);
}

.leaf {
  cursor: pointer;
}

.toggle-btn {
  background: none;
  border: none;
  padding: 0 4px 0 0;
  cursor: pointer;
  display: flex;
  align-items: center;
  color: #666;
  transition: color 0.15s;
}

.toggle-btn:hover {
  color: #667eea;
}

.caret {
  transition: transform 0.15s ease;
}

.caret.open {
  transform: rotate(90deg);
}

.leaf-spacer {
  width: 14px;
  display: inline-block;
}

.node-key {
  color: #667eea;
  font-weight: 500;
}

.node-index {
  color: #888;
  font-weight: 400;
}

.colon {
  color: #666;
}

.bracket {
  color: #888;
  font-weight: 600;
}

.bracket-close {
  color: #888;
  font-weight: 600;
}

.collapsed-preview {
  color: #555;
  font-style: italic;
  font-size: 12px;
}

.item-count {
  color: #555;
  font-size: 11px;
  margin-left: 8px;
  font-style: italic;
}

.value-string {
  color: #98c379;
}

.value-number {
  color: #d19a66;
}

.value-boolean {
  color: #56b6c2;
}

.value-null {
  color: #e06c75;
  font-style: italic;
}

.value-other {
  color: #abb2bf;
}
</style>
