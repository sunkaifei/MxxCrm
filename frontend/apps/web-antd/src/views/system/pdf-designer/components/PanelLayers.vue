<script lang="ts" setup>
/**
 * 左面板 · 图层（设计文档 §8.4）。
 *
 * 列表顺序与 `layout_json.elements` 相反（后添加 = 更上层，显示在最前）。
 */
import { computed } from 'vue';

import {
  LucideChevronDown,
  LucideChevronUp,
  LucideEye,
  LucideEyeOff,
  LucideLock,
  LucideUnlock,
} from '@vben/icons';

import { Button, Tooltip } from 'ant-design-vue';

import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';

const store = useDesignerStore();

const list = computed(() => [...store.elements].reverse());

const TYPE_LABEL: Record<string, string> = {
  text: '文本',
  field: '字段',
  image: '图片',
  seal: '签章',
  signature: '签名',
  barcode: '条码',
  qrcode: '二维码',
  line: '直线',
  rect: '矩形',
  ellipse: '圆形',
  table: '表格',
  pageNumber: '页码',
  repeater: '重复块',
};

function select(id: string, e: MouseEvent) {
  if (e.shiftKey) {
    store.selection = store.selection.includes(id)
      ? store.selection.filter((i) => i !== id)
      : [...store.selection, id];
  } else {
    store.selection = [id];
  }
}

function toggleLock(id: string) {
  const el = store.elements.find((x) => x.id === id);
  if (el) store.updateElement(id, { locked: !el.locked }, '锁定状态');
}

function toggleHide(id: string) {
  const el = store.elements.find((x) => x.id === id);
  if (el) {
    store.updateElement(id, { hiddenInOutput: !el.hiddenInOutput }, '可见性');
  }
}
</script>

<template>
  <div class="panel-layers">
    <div v-if="list.length === 0" class="layers-empty">
      画布上还没有元素，从「元素库」拖一个进来吧
    </div>
    <div
      v-for="el in list"
      :key="el.id"
      class="layer-item"
      :class="{ 'is-active': store.selection.includes(el.id) }"
      @click="select(el.id, $event)"
    >
      <span class="layer-type">{{ TYPE_LABEL[el.type] ?? el.type }}</span>
      <span class="layer-name" :title="el.name">{{ el.name }}</span>
      <span class="layer-actions" @click.stop>
        <Tooltip :title="el.hiddenInOutput ? '设为输出可见' : '仅设计态可见'">
          <LucideEyeOff
            v-if="el.hiddenInOutput"
            class="layer-act is-off"
            @click="toggleHide(el.id)"
          />
          <LucideEye v-else class="layer-act" @click="toggleHide(el.id)" />
        </Tooltip>
        <Tooltip :title="el.locked ? '解锁' : '锁定'">
          <LucideLock v-if="el.locked" class="layer-act is-off" @click="toggleLock(el.id)" />
          <LucideUnlock v-else class="layer-act" @click="toggleLock(el.id)" />
        </Tooltip>
        <Button
          :disabled="list[0]?.id === el.id"
          size="small"
          type="text"
          @click="store.reorder(el.id, 'top')"
        >
          <LucideChevronUp class="layer-act" />
        </Button>
        <Button
          :disabled="list[list.length - 1]?.id === el.id"
          size="small"
          type="text"
          @click="store.reorder(el.id, 'down')"
        >
          <LucideChevronDown class="layer-act" />
        </Button>
      </span>
    </div>
  </div>
</template>

<style scoped>
.panel-layers {
  height: 100%;
  padding: 6px;
  overflow-y: auto;
}
.layers-empty {
  padding: 24px 12px;
  font-size: 12px;
  line-height: 1.7;
  color: #bfbfbf;
  text-align: center;
}
.layer-item {
  display: flex;
  gap: 6px;
  align-items: center;
  padding: 4px 6px;
  font-size: 12px;
  cursor: pointer;
  border-radius: 4px;
}
.layer-item:hover {
  background: #f5f5f5;
}
.layer-item.is-active {
  background: #e6f4ff;
}
.layer-type {
  flex: none;
  padding: 0 4px;
  font-size: 10px;
  line-height: 15px;
  color: #1677ff;
  background: rgb(22 119 255 / 10%);
  border-radius: 3px;
}
.layer-name {
  flex: 1;
  overflow: hidden;
  color: #262626;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.layer-actions {
  display: none;
  gap: 2px;
  align-items: center;
}
.layer-item:hover .layer-actions {
  display: flex;
}
.layer-act {
  width: 13px;
  height: 13px;
  color: #8c8c8c;
  cursor: pointer;
}
.layer-act.is-off {
  color: #faad14;
}
</style>
