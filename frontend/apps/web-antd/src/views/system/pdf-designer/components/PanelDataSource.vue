<script lang="ts" setup>
/**
 * 左面板 · 数据源树（设计文档 §8.4、§32）。
 *
 * 显示内容随数据视图变化（§32.13 交叉矩阵）：
 * - 样例模式：显示典型样例值（不显示真实数据，避免误读）
 * - 占位符/轮廓：显示 `{{field}}`，用于查漏绑定
 * - 真实模式：显示当前演示单据的真实值
 *
 * 拖拽 → 画布生成 `field` 元素并以 `fieldPath` 绑定（字段树拖拽绑定）。
 */
import type { PdfFieldGroup } from '#/types/pdf-layout';

import { computed, ref } from 'vue';

import { LucideLoader, LucideSearch } from '@vben/icons';

import { Input, Tooltip } from 'ant-design-vue';

import { beginLibraryDrag } from '#/composables/pdf-designer/useDragDrop';
import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';

const props = defineProps<{ loading?: boolean; tree: PdfFieldGroup[] }>();

const store = useDesignerStore();
const keyword = ref('');
const expanded = ref<Record<string, boolean>>({});

const filtered = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  if (!kw) return props.tree;
  return props.tree
    .map((g) => ({
      ...g,
      children: g.children.filter(
        (f) =>
          f.fieldName.toLowerCase().includes(kw) ||
          f.fieldPath.toLowerCase().includes(kw),
      ),
    }))
    .filter((g) => g.children.length > 0);
});

function isOpen(code: string) {
  return expanded.value[code] !== false;
}

function toggle(code: string) {
  expanded.value[code] = !isOpen(code);
}

/** 设计态展示值（§32.13） */
function displayValue(f: any): string {
  if (store.dataMode === 'placeholder' || store.dataMode === 'outline') {
    return `{{${f.fieldPath}}}`;
  }
  // 样例模式取 sample；真实模式从上下文里取
  if (store.dataMode === 'real') {
    const parts = String(f.fieldPath).split('.');
    let cur: any = store.dataContext;
    for (const p of parts) {
      if (cur === null || cur === undefined) break;
      cur = cur[p];
    }
    return cur === null || cur === undefined || cur === ''
      ? '—'
      : String(cur);
  }
  return f.sample ?? '—';
}

function onDragStart(e: DragEvent, f: any, group: PdfFieldGroup) {
  beginLibraryDrag(e, {
    kind: 'field',
    fieldPath: f.fieldPath,
    fieldName: f.fieldName,
    dataType: f.dataType,
    groupCode: group.groupCode,
  });
}
</script>

<template>
  <div class="panel-ds">
    <div class="ds-search">
      <Input v-model:value="keyword" allow-clear size="small" placeholder="搜索字段">
        <template #prefix>
          <LucideSearch class="ds-search-icon" />
        </template>
      </Input>
    </div>

    <div v-if="props.loading" class="ds-loading">
      <LucideLoader class="ds-spin" />
      <span>加载字段树…</span>
    </div>

    <div v-else class="ds-body">
      <div v-for="g in filtered" :key="g.groupCode" class="ds-group">
        <div class="ds-group-head" @click="toggle(g.groupCode)">
          <span class="ds-caret" :class="{ 'is-open': isOpen(g.groupCode) }">▸</span>
          <span>{{ g.groupName }}</span>
          <span class="ds-count">{{ g.children.length }}</span>
        </div>
        <div v-show="isOpen(g.groupCode)" class="ds-fields">
          <Tooltip
            v-for="f in g.children"
            :key="f.fieldPath"
            :title="`${f.fieldPath} · ${f.dataType}`"
            placement="right"
          >
            <div
              class="ds-field"
              draggable="true"
              @dragstart="onDragStart($event, f, g)"
            >
              <span class="ds-field-name">{{ f.fieldName }}</span>
              <span
                class="ds-field-sample"
                :class="{
                  'is-php': store.dataMode === 'placeholder' || store.dataMode === 'outline',
                }"
              >
                {{ displayValue(f) }}
              </span>
            </div>
          </Tooltip>
        </div>
      </div>
      <div v-if="filtered.length === 0" class="ds-empty">无匹配字段</div>
    </div>
  </div>
</template>

<style scoped>
.panel-ds {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}
.ds-search {
  padding: 8px;
  border-bottom: 1px solid #f0f0f0;
}
.ds-search-icon {
  width: 13px;
  height: 13px;
  color: #bfbfbf;
}
.ds-loading {
  display: flex;
  gap: 6px;
  align-items: center;
  justify-content: center;
  padding: 24px;
  font-size: 12px;
  color: #8c8c8c;
}
.ds-spin {
  width: 14px;
  height: 14px;
  animation: ds-rotate 1s linear infinite;
}
@keyframes ds-rotate {
  to {
    transform: rotate(360deg);
  }
}
.ds-body {
  flex: 1;
  padding: 6px;
  overflow-y: auto;
}
.ds-group + .ds-group {
  margin-top: 4px;
}
.ds-group-head {
  display: flex;
  gap: 4px;
  align-items: center;
  padding: 4px 6px;
  font-size: 12px;
  font-weight: 500;
  color: #595959;
  cursor: pointer;
  border-radius: 4px;
  user-select: none;
}
.ds-group-head:hover {
  background: #f5f5f5;
}
.ds-caret {
  display: inline-block;
  font-size: 10px;
  transition: transform 0.15s;
}
.ds-caret.is-open {
  transform: rotate(90deg);
}
.ds-count {
  margin-left: auto;
  font-size: 11px;
  color: #bfbfbf;
}
.ds-field {
  display: flex;
  gap: 6px;
  align-items: center;
  justify-content: space-between;
  padding: 3px 6px 3px 18px;
  font-size: 12px;
  cursor: grab;
  border-radius: 4px;
}
.ds-field:hover {
  background: #f0f7ff;
}
.ds-field:active {
  cursor: grabbing;
}
.ds-field-name {
  color: #262626;
  white-space: nowrap;
}
.ds-field-sample {
  max-width: 50%;
  overflow: hidden;
  font-size: 11px;
  color: #bfbfbf;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ds-field-sample.is-php {
  color: #1677ff;
}
.ds-empty {
  padding: 24px;
  font-size: 12px;
  color: #bfbfbf;
  text-align: center;
}
</style>
