<script lang="ts" setup>
/**
 * 画布工作区（设计文档 §34：有限粘贴板 + 居中固定纸张 + 受限视口 + 单页母版）。
 *
 * ⚠️ 本组件是「无限画布」决议的落地实现（§34.2）：
 * 纸张**固定居中、不可移动、不可增删，恰好一张**；四周是**有限粘贴板**（外扩 30mm），
 * 不是 Figma/Miro 式无限画布。理由：坐标是纸张绝对 mm、分页由编译期决定、
 * 纸外元素设计态可见而出图不可见会破坏一致性闭环。
 *
 * 编辑态只有**一张纸**（页型母版）；多页结果由「分页预览」呈现（§34.6）。
 */
import type { PdfElementType } from '#/types/pdf-layout';

import { computed, onMounted, ref, watch } from 'vue';

import { mmToPx } from '#/composables/pdf-designer/useUnits';
import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';
import { useDragDrop } from '#/composables/pdf-designer/useDragDrop';
import { drawRuler } from '#/composables/pdf-designer/useRulers';
import { useViewport } from '#/composables/pdf-designer/useViewport';

import ElementRenderer from './ElementRenderer.vue';

/** 粘贴板外扩（mm）——相当于 InDesign 的有限出血台 */
const BLEED_MM = 30;

const store = useDesignerStore();
const scroller = ref<HTMLElement>();
const rulerH = ref<HTMLCanvasElement>();
const rulerV = ref<HTMLCanvasElement>();
const cursorMm = ref<null | { x: number; y: number }>(null);
const marquee = ref<null | { h: number; w: number; x: number; y: number }>(null);

const viewport = useViewport(scroller);
const dnd = useDragDrop(scroller, BLEED_MM);

// ---------------------------------------------------------------------------
// 几何
// ---------------------------------------------------------------------------
const boardMm = computed(() => ({
  w: store.pageWidth + BLEED_MM * 2,
  h: store.pageHeight + BLEED_MM * 2,
}));
const boardPx = computed(() => ({
  w: mmToPx(boardMm.value.w),
  h: mmToPx(boardMm.value.h),
}));
const scaledPx = computed(() => ({
  w: boardPx.value.w * store.zoom,
  h: boardPx.value.h * store.zoom,
}));

const marginGuideStyle = computed(() => {
  const m = store.layout.page.margin;
  return {
    left: `${mmToPx(m.left)}px`,
    top: `${mmToPx(m.top)}px`,
    width: `${mmToPx(store.pageWidth - m.left - m.right)}px`,
    height: `${mmToPx(store.pageHeight - m.top - m.bottom)}px`,
  };
});

const backgroundStyle = computed(() => {
  const bg = store.layout.page.background;
  if (bg.type === 'image' && bg.value && bg.previewOnly) {
    return { background: `repeating-linear-gradient(45deg, #f5f5f5 0 6px, #fafafa 6px 12px)` };
  }
  if (bg.type === 'color') return { background: bg.value ?? '#fff' };
  return {};
});

// ---------------------------------------------------------------------------
// 选中框
// ---------------------------------------------------------------------------
const single = computed(() => store.singleSelected);
const selBox = computed(() => {
  if (!single.value) return null;
  const e = single.value;
  return {
    left: `${mmToPx(e.x)}px`,
    top: `${mmToPx(e.y)}px`,
    width: `${mmToPx(e.w)}px`,
    height: `${mmToPx(e.h)}px`,
  };
});

const multiBox = computed(() => {
  if (store.selection.length <= 1) return null;
  const els = store.selectedElements;
  if (els.length === 0) return null;
  const minX = Math.min(...els.map((e) => e.x));
  const minY = Math.min(...els.map((e) => e.y));
  const maxX = Math.max(...els.map((e) => e.x + e.w));
  const maxY = Math.max(...els.map((e) => e.y + e.h));
  return {
    left: `${mmToPx(minX)}px`,
    top: `${mmToPx(minY)}px`,
    width: `${mmToPx(maxX - minX)}px`,
    height: `${mmToPx(maxY - minY)}px`,
  };
});

const HANDLES: Array<{ cls: string; h: any }> = [
  { cls: 'nw', h: 'nw' },
  { cls: 'n', h: 'n' },
  { cls: 'ne', h: 'ne' },
  { cls: 'e', h: 'e' },
  { cls: 'se', h: 'se' },
  { cls: 's', h: 's' },
  { cls: 'sw', h: 'sw' },
  { cls: 'w', h: 'w' },
];

// ---------------------------------------------------------------------------
// 交互
// ---------------------------------------------------------------------------
function onElementMouseDown(e: MouseEvent, el: { id: string }) {
  if (e.button !== 0) return;
  if (e.shiftKey) {
    store.selection = store.selection.includes(el.id)
      ? store.selection.filter((i) => i !== el.id)
      : [...store.selection, el.id];
  } else if (!store.selection.includes(el.id)) {
    store.selection = [el.id];
  }
  dnd.startMove(e);
}

function onPaperMouseDown(e: MouseEvent) {
  if (e.target !== e.currentTarget && !(e.target as HTMLElement).dataset.pdfPaper) {
    return;
  }
  if (e.button !== 0) return;
  store.selection = [];
  // 框选
  const paper = e.currentTarget as HTMLElement;
  const rect = paper.getBoundingClientRect();
  const startX = e.clientX - rect.left;
  const startY = e.clientY - rect.top;
  const onMove = (ev: MouseEvent) => {
    const cx = ev.clientX - rect.left;
    const cy = ev.clientY - rect.top;
    marquee.value = {
      x: Math.min(startX, cx),
      y: Math.min(startY, cy),
      w: Math.abs(cx - startX),
      h: Math.abs(cy - startY),
    };
    // 屏幕 px → 未缩放 px → mm（除以 zoom 后再换算）
    const toMm = (v: number) => v / store.zoom / mmToPx(1);
    const mmX0 = toMm(Math.min(startX, cx));
    const mmY0 = toMm(Math.min(startY, cy));
    const mmX1 = toMm(Math.max(startX, cx));
    const mmY1 = toMm(Math.max(startY, cy));
    store.selection = store.elements
      .filter(
        (el) =>
          el.x < mmX1 && el.x + el.w > mmX0 && el.y < mmY1 && el.y + el.h > mmY0,
      )
      .map((el) => el.id);
  };
  const onUp = () => {
    marquee.value = null;
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
  };
  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
}

function onDrop(e: DragEvent) {
  const payload = dnd.readDropPayload(e);
  if (!payload) return;
  const { x, y } = dnd.dropPoint(e);
  if (payload.kind === 'field') {
    const preset = { h: 6, w: 45 };
    store.addElement('field', x, y, {
      name: payload.fieldName ?? '数据字段',
      w: preset.w,
      h: preset.h,
      bind: { path: payload.fieldPath },
      props: { dataType: payload.dataType },
    });
  } else if (payload.kind === 'asset') {
    store.addElement('image', x, y, {
      name: payload.name ?? '图片',
      props: { assetId: payload.assetId, url: payload.url },
    });
  } else if (payload.type) {
    store.addElement(payload.type as PdfElementType, x, y);
  }
}

function onMouseMoveCapture(e: MouseEvent) {
  const p = dnd.clientToPageMm(e.clientX, e.clientY);
  cursorMm.value = { x: p.x, y: p.y };
}

// ---------------------------------------------------------------------------
// 标尺
// ---------------------------------------------------------------------------
function redrawRulers() {
  const opts = {
    zoom: store.zoom,
    scrollLeft: store.scrollX,
    scrollTop: store.scrollY,
    bleed: BLEED_MM,
  };
  const el = scroller.value;
  if (!el) return;
  drawRuler(rulerH.value, 'h', el.clientWidth, opts, BLEED_MM, cursorMm.value?.x ?? null);
  drawRuler(rulerV.value, 'v', el.clientHeight, opts, BLEED_MM, cursorMm.value?.y ?? null);
}

onMounted(() => {
  redrawRulers();
  viewport.centerPage();
});

watch(
  [() => store.zoom, () => store.scrollX, () => store.scrollY, cursorMm],
  () => redrawRulers(),
);

function onScrollerScroll() {
  viewport.onScroll();
  redrawRulers();
}

defineExpose({
  centerPage: viewport.centerPage,
  fit: (w: number, h: number) => {
    store.fitTo(w, h);
    viewport.centerPage();
  },
  nudge: dnd.nudge,
});
</script>

<template>
  <div class="pdf-stage">
    <!-- 标尺（Canvas：唯一使用 Canvas 的图层，§33.4） -->
    <canvas v-if="store.showRuler" ref="rulerH" class="pdf-ruler pdf-ruler-h" />
    <canvas v-if="store.showRuler" ref="rulerV" class="pdf-ruler pdf-ruler-v" />
    <div v-if="store.showRuler" class="pdf-ruler-corner">mm</div>

    <!-- 视口 -->
    <div
      ref="scroller"
      class="pdf-viewport"
      :class="{ 'is-panning': viewport.isPanning.value }"
      @mousedown="viewport.startPan"
      @mousemove="onMouseMoveCapture"
      @scroll="onScrollerScroll"
      @wheel="viewport.onWheel"
    >
      <div
        class="pdf-scroll-pad"
        :style="{ width: `${scaledPx.w}px`, height: `${scaledPx.h}px` }"
      >
        <div
          class="pdf-board"
          :style="{
            width: `${boardPx.w}px`,
            height: `${boardPx.h}px`,
            transform: `scale(${store.zoom})`,
            transformOrigin: '0 0',
          }"
        >
          <!-- 纸张（固定居中、不可移动，恰好一张） -->
          <div
            class="pdf-paper"
            data-pdf-paper
            :style="{
              left: `${mmToPx(BLEED_MM)}px`,
              top: `${mmToPx(BLEED_MM)}px`,
              width: `${mmToPx(store.pageWidth)}px`,
              height: `${mmToPx(store.pageHeight)}px`,
              ...backgroundStyle,
            }"
            @dragover.prevent
            @drop.prevent="onDrop"
            @mousedown="onPaperMouseDown"
          >
            <!-- 网格 -->
            <div v-if="store.showGrid" class="pdf-grid" />
            <!-- 页边距参考线 -->
            <div v-if="store.showMarginGuide" class="pdf-margin-guide" :style="marginGuideStyle" />
            <!-- 纸外 C 档提示 -->
            <div class="pdf-bleed-hint">纸外区域（出图被裁切）</div>

            <!-- 元素 -->
            <ElementRenderer
              v-for="el in store.elements"
              :key="el.id"
              :ctx="store.dataContext"
              :data-mode="store.dataMode"
              :el="el"
              :selected="store.selection.includes(el.id)"
              @mousedown="onElementMouseDown($event, el)"
              @update="store.selection = [el.id]"
            />

            <!-- 对齐辅助线 -->
            <div
              v-for="(g, i) in dnd.guides.value"
              :key="`g${i}`"
              class="pdf-guide"
              :class="g.kind === 'v' ? 'is-v' : 'is-h'"
              :style="
                g.kind === 'v'
                  ? { left: `${mmToPx(g.at)}px` }
                  : { top: `${mmToPx(g.at)}px` }
              "
            />

            <!-- 单选选中框 + 8 控制点 -->
            <div v-if="selBox" class="pdf-selbox" :style="selBox">
              <span
                v-for="h in HANDLES"
                :key="h.cls"
                class="pdf-handle"
                :class="`is-${h.cls}`"
                @mousedown="dnd.startResize($event, h.h)"
              />
            </div>

            <!-- 多选包围框 -->
            <div v-if="multiBox" class="pdf-selbox is-multi" :style="multiBox" />

            <!-- 框选 -->
            <div
              v-if="marquee"
              class="pdf-marquee"
              :style="{
                left: `${marquee.x}px`,
                top: `${marquee.y}px`,
                width: `${marquee.w}px`,
                height: `${marquee.h}px`,
              }"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pdf-stage {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: #f0f2f5;
}
.pdf-ruler {
  position: absolute;
  z-index: 6;
  background: #fafafa;
}
.pdf-ruler-h {
  top: 0;
  left: 20px;
  width: calc(100% - 20px);
  height: 20px;
  border-bottom: 1px solid #e8e8e8;
}
.pdf-ruler-v {
  top: 20px;
  left: 0;
  width: 20px;
  height: calc(100% - 20px);
  border-right: 1px solid #e8e8e8;
}
.pdf-ruler-corner {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 7;
  width: 20px;
  height: 20px;
  font-size: 9px;
  line-height: 20px;
  color: #8c8c8c;
  text-align: center;
  background: #fafafa;
  border-right: 1px solid #e8e8e8;
  border-bottom: 1px solid #e8e8e8;
}
.pdf-viewport {
  position: absolute;
  inset: 20px 0 0 20px;
  overflow: auto;
}
.pdf-viewport {
  cursor: grab;
}
.pdf-viewport.is-panning {
  cursor: grabbing;
}
/* 中键平移时按住的手感与左键拖拽一致 */
.pdf-viewport.is-panning * {
  cursor: grabbing !important;
  user-select: none;
}
.pdf-scroll-pad {
  position: relative;
  margin: 24px;
}
.pdf-board {
  position: absolute;
  top: 0;
  left: 0;
}
.pdf-paper {
  position: absolute;
  background: #fff;
  box-shadow: 0 2px 8px rgb(0 0 0 / 15%);
}
.pdf-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(to right, rgb(0 0 0 / 4%) 1px, transparent 1px),
    linear-gradient(to bottom, rgb(0 0 0 / 4%) 1px, transparent 1px);
  background-size: 3.7795px 3.7795px;
  pointer-events: none;
}
.pdf-margin-guide {
  position: absolute;
  pointer-events: none;
  border: 1px dashed #d9d9d9;
}
.pdf-bleed-hint {
  position: absolute;
  top: 2px;
  left: 2px;
  font-size: 9px;
  color: #d46b08;
  pointer-events: none;
  opacity: 0.5;
}
.pdf-guide {
  position: absolute;
  pointer-events: none;
  background: #eb2f96;
}
.pdf-guide.is-v {
  top: 0;
  bottom: 0;
  width: 1px;
}
.pdf-guide.is-h {
  right: 0;
  left: 0;
  height: 1px;
}
.pdf-selbox {
  position: absolute;
  pointer-events: none;
  border: 1px solid #1677ff;
}
.pdf-selbox.is-multi {
  border-style: dashed;
  background: rgb(22 119 255 / 4%);
}
.pdf-handle {
  position: absolute;
  width: 7px;
  height: 7px;
  pointer-events: auto;
  background: #fff;
  border: 1px solid #1677ff;
}
.pdf-handle.is-nw {
  top: -4px;
  left: -4px;
  cursor: nwse-resize;
}
.pdf-handle.is-n {
  top: -4px;
  left: calc(50% - 4px);
  cursor: ns-resize;
}
.pdf-handle.is-ne {
  top: -4px;
  right: -4px;
  cursor: nesw-resize;
}
.pdf-handle.is-e {
  top: calc(50% - 4px);
  right: -4px;
  cursor: ew-resize;
}
.pdf-handle.is-se {
  right: -4px;
  bottom: -4px;
  cursor: nwse-resize;
}
.pdf-handle.is-s {
  bottom: -4px;
  left: calc(50% - 4px);
  cursor: ns-resize;
}
.pdf-handle.is-sw {
  bottom: -4px;
  left: -4px;
  cursor: nesw-resize;
}
.pdf-handle.is-w {
  top: calc(50% - 4px);
  left: -4px;
  cursor: ew-resize;
}
.pdf-marquee {
  position: absolute;
  pointer-events: none;
  background: rgb(22 119 255 / 10%);
  border: 1px dashed #1677ff;
}
</style>
