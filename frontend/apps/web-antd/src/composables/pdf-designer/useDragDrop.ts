/**
 * 拖拽与缩放（设计文档 §8.6、§8.7）。
 *
 * 统一处理三类交互：
 * 1. 从元素库 / 数据源树拖入画布（HTML5 DnD）；
 * 2. 画布内移动元素（含吸附对齐、多选整体位移）；
 * 3. 八向控制点缩放（含 `Shift` 等比、`Alt` 以中心缩放）。
 *
 * 所有几何计算在 **mm 域**；仅在读取鼠标位移时把屏幕 px 换算成 mm。
 */

import type { Ref } from 'vue';

import { ref } from 'vue';

import { pxToMm, round } from '#/composables/pdf-designer/useUnits';

import { clampToBleed, computeSnap } from './useRulers';
import { useDesignerStore } from './useDesignerStore';

export const DRAG_MIME = 'application/x-pdf-element';

/**
 * 拖拽载荷写入（元素库 / 数据源树 / 素材库共用）。
 *
 * 单独导出为纯函数，左面板无需实例化 useDragDrop 即可发起拖拽。
 */
export function beginLibraryDrag(e: DragEvent, payload: Record<string, any>) {
  if (!e.dataTransfer) return;
  e.dataTransfer.effectAllowed = 'copy';
  e.dataTransfer.setData(DRAG_MIME, JSON.stringify(payload));
  // 兼容：部分浏览器在 dragover 阶段读不到自定义 MIME
  e.dataTransfer.setData('text/plain', payload.type ?? payload.fieldPath ?? '');
}

/** 读取拖拽载荷（画布侧使用） */
export function readDragPayload(e: DragEvent): null | Record<string, any> {
  const raw = e.dataTransfer?.getData(DRAG_MIME);
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export type ResizeHandle = 'e' | 'n' | 'ne' | 'nw' | 's' | 'se' | 'sw' | 'w';

export function useDragDrop(
  stageRef: Ref<HTMLElement | undefined>,
  /** 粘贴板外扩（mm） */
  bleed: number,
) {
  const store = useDesignerStore();
  const guides = ref<{ at: number; kind: 'h' | 'v' }[]>([]);
  const dragging = ref(false);
  const resizing = ref<null | ResizeHandle>(null);

  let startState: {
    boxes: Array<{ h: number; w: number; x: number; y: number; id: string }>;
    clientX: number;
    clientY: number;
    handle?: ResizeHandle;
  } | null = null;

  // ---------------------------------------------------------------------------
  // 屏幕坐标 → 页面 mm 坐标
  // ---------------------------------------------------------------------------

  /** 相对纸张左上角的 mm 坐标（粘贴板视为负偏移区） */
  function clientToPageMm(clientX: number, clientY: number) {
    const stage = stageRef.value;
    if (!stage) return { x: 0, y: 0 };
    const paper = stage.querySelector<HTMLElement>('[data-pdf-paper]');
    if (!paper) return { x: 0, y: 0 };
    const rect = paper.getBoundingClientRect();
    return {
      x: pxToMm(clientX - rect.left) / store.zoom,
      y: pxToMm(clientY - rect.top) / store.zoom,
    };
  }

  // ---------------------------------------------------------------------------
  // 从元素库拖入
  // ---------------------------------------------------------------------------

  function beginLibraryDragImpl(e: DragEvent, payload: Record<string, any>) {
    beginLibraryDrag(e, payload);
  }

  function readDropPayload(e: DragEvent): null | Record<string, any> {
    return readDragPayload(e);
  }

  /** 放置：返回落点 mm 坐标（左上角对齐鼠标） */
  function dropPoint(e: DragEvent) {
    const p = clientToPageMm(e.clientX, e.clientY);
    return { x: round(p.x), y: round(p.y) };
  }

  // ---------------------------------------------------------------------------
  // 移动 / 缩放
  // ---------------------------------------------------------------------------

  function startMove(e: MouseEvent) {
    if (e.button !== 0) return;
    const ids = store.selection;
    if (ids.length === 0) return;
    store.commit('移动元素');
    dragging.value = true;
    startState = {
      boxes: store.elements
        .filter((el) => ids.includes(el.id))
        .map((el) => ({ id: el.id, x: el.x, y: el.y, w: el.w, h: el.h })),
      clientX: e.clientX,
      clientY: e.clientY,
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', end);
  }

  function startResize(e: MouseEvent, handle: ResizeHandle) {
    if (e.button !== 0) return;
    const el = store.singleSelected;
    if (!el) return;
    e.stopPropagation();
    store.commit('缩放元素');
    resizing.value = handle;
    startState = {
      boxes: [{ id: el.id, x: el.x, y: el.y, w: el.w, h: el.h }],
      clientX: e.clientX,
      clientY: e.clientY,
      handle,
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', end);
  }

  function onMove(e: MouseEvent) {
    if (!startState) return;
    const dx = pxToMm(e.clientX - startState.clientX) / store.zoom;
    const dy = pxToMm(e.clientY - startState.clientY) / store.zoom;

    if (resizing.value) {
      const b = startState.boxes[0]!;
      const h = resizing.value;
      let { x, y, w } = b;
      let height = b.h;
      if (h.includes('e')) w = b.w + dx;
      if (h.includes('s')) height = b.h + dy;
      if (h.includes('w')) {
        w = b.w - dx;
        x = b.x + dx;
      }
      if (h.includes('n')) {
        height = b.h - dy;
        y = b.y + dy;
      }
      // Shift 等比；Alt 以中心缩放
      if (e.shiftKey && b.w > 0 && b.h > 0) {
        const ratio = b.h / b.w;
        height = w * ratio;
        if (h.includes('n')) y = b.y + (b.h - height);
      }
      if (e.altKey) {
        const cx = b.x + b.w / 2;
        const cy = b.y + b.h / 2;
        x = cx - w / 2;
        y = cy - height / 2;
      }
      const min = 2;
      const patch = {
        x: round(x),
        y: round(y),
        w: round(Math.max(min, w)),
        h: round(Math.max(min, height)),
      };
      store.patchElementLive(b.id, patch);
      return;
    }

    // 移动：以主选中元素参与吸附，其余元素跟随同等位移
    const primary = startState.boxes[0];
    if (!primary) return;
    const others = store.elements
      .filter((el) => !startState!.boxes.some((b) => b.id === el.id))
      .map((el) => ({ x: el.x, y: el.y, w: el.w, h: el.h }));

    let nx = primary.x + dx;
    let ny = primary.y + dy;
    let offsetX = dx;
    let offsetY = dy;

    if (store.snap) {
      const snapped = computeSnap({
        box: { x: nx, y: ny, w: primary.w, h: primary.h },
        threshold: store.snapMm,
        others,
        page: {
          width: store.pageWidth,
          height: store.pageHeight,
          margin: store.layout.page.margin,
        },
      });
      guides.value = snapped.guides;
      offsetX = snapped.x - primary.x;
      offsetY = snapped.y - primary.y;
      nx = snapped.x;
      ny = snapped.y;
    } else {
      guides.value = [];
    }

    for (const b of startState.boxes) {
      const clamped = clampToBleed(
        { x: b.x + offsetX, y: b.y + offsetY, w: b.w, h: b.h },
        store.pageWidth,
        store.pageHeight,
        bleed,
      );
      store.patchElementLive(b.id, { x: round(clamped.x), y: round(clamped.y) });
    }
  }

  function end() {
    dragging.value = false;
    resizing.value = null;
    startState = null;
    guides.value = [];
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', end);
  }

  // ---------------------------------------------------------------------------
  // 键盘微调
  // ---------------------------------------------------------------------------

  function nudge(dx: number, dy: number) {
    if (store.selection.length === 0) return;
    store.commit('微调位置');
    for (const id of store.selection) {
      const el = store.elements.find((e) => e.id === id);
      if (!el) continue;
      store.patchElementLive(id, {
        x: round(el.x + dx),
        y: round(el.y + dy),
      });
    }
  }

  return {
    guides,
    dragging,
    resizing,
    beginLibraryDrag: beginLibraryDragImpl,
    readDropPayload,
    dropPoint,
    clientToPageMm,
    startMove,
    startResize,
    nudge,
  };
}
