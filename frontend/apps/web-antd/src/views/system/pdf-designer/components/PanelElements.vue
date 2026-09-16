<script lang="ts" setup>
/**
 * 左面板 · 元素库（设计文档 §8.4）。
 *
 * 拖拽协议：`dataTransfer['application/x-pdf-element'] = {kind:'element', type}`。
 * 与数据源树共用同一 MIME，画布侧统一解析（useDragDrop.readDropPayload）。
 */
import type { PdfElementType } from '#/types/pdf-layout';

import {
  LucideBarcode,
  LucideCircle,
  LucideHash,
  LucideImage,
  LucideMinus,
  LucidePenTool,
  LucideQrCode,
  LucideRepeat,
  LucideSquare,
  LucideStamp,
  LucideTable,
  LucideType,
  LucideVariable,
} from '@vben/icons';

import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';
import { beginLibraryDrag } from '#/composables/pdf-designer/useDragDrop';

const store = useDesignerStore();

interface LibItem {
  group: string;
  icon: any;
  label: string;
  type: PdfElementType;
}

const items: LibItem[] = [
  { group: '文本', type: 'text', label: '静态文本', icon: LucideType },
  { group: '文本', type: 'field', label: '数据字段', icon: LucideVariable },
  { group: '文本', type: 'pageNumber', label: '页码', icon: LucideHash },
  { group: '图形', type: 'line', label: '直线', icon: LucideMinus },
  { group: '图形', type: 'rect', label: '矩形', icon: LucideSquare },
  { group: '图形', type: 'ellipse', label: '圆形', icon: LucideCircle },
  { group: '图片', type: 'image', label: '图片 / Logo', icon: LucideImage },
  { group: '图片', type: 'seal', label: '电子签章', icon: LucideStamp },
  { group: '图片', type: 'signature', label: '手写签名', icon: LucidePenTool },
  { group: '码', type: 'barcode', label: '条形码', icon: LucideBarcode },
  { group: '码', type: 'qrcode', label: '二维码', icon: LucideQrCode },
  { group: '数据', type: 'table', label: '明细表格', icon: LucideTable },
  { group: '数据', type: 'repeater', label: '重复块', icon: LucideRepeat },
];

const groups = [...new Set(items.map((i) => i.group))];

function onDragStart(e: DragEvent, item: LibItem) {
  beginLibraryDrag(e, { kind: 'element', type: item.type });
}

/** 双击 = 落在纸张可视中心 */
function onDblClick(item: LibItem) {
  store.addElement(item.type, 20, 40);
}
</script>

<template>
  <div class="panel-elements">
    <div v-for="g in groups" :key="g" class="lib-group">
      <div class="lib-group-title">{{ g }}</div>
      <div class="lib-grid">
        <div
          v-for="item in items.filter((i) => i.group === g)"
          :key="item.type"
          class="lib-item"
          draggable="true"
          :title="`拖入画布，或双击放到纸张上`"
          @dragstart="onDragStart($event, item)"
          @dblclick="onDblClick(item)"
        >
          <component :is="item.icon" class="lib-item-icon" />
          <span class="lib-item-label">{{ item.label }}</span>
        </div>
      </div>
    </div>
    <div class="lib-tip">
      拖拽到纸张上摆放。双击可放到默认位置。
    </div>
  </div>
</template>

<style scoped>
.panel-elements {
  padding: 8px;
  overflow-y: auto;
}
.lib-group + .lib-group {
  margin-top: 10px;
}
.lib-group-title {
  margin-bottom: 6px;
  font-size: 12px;
  color: #8c8c8c;
}
.lib-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 6px;
}
.lib-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  align-items: center;
  justify-content: center;
  height: 58px;
  cursor: grab;
  background: #fafafa;
  border: 1px solid #f0f0f0;
  border-radius: 6px;
  transition: all 0.15s;
  user-select: none;
}
.lib-item:hover {
  background: #f0f7ff;
  border-color: #91caff;
}
.lib-item:active {
  cursor: grabbing;
}
.lib-item-icon {
  width: 16px;
  height: 16px;
  color: #1677ff;
}
.lib-item-label {
  font-size: 11px;
  color: #595959;
}
.lib-tip {
  margin-top: 12px;
  font-size: 11px;
  line-height: 1.6;
  color: #bfbfbf;
}
</style>
