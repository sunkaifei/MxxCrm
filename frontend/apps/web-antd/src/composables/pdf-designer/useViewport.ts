/**
 * 画布视口：缩放（指针锚点）、平移、Fit（设计文档 §34.5）。
 *
 * ⚠️ 关键实现点（§34.11 踩坑记录）：
 * 缩放若不做**指针锚点修正**，视图会随缩放"跳飞"——设计器最常见的体验事故。
 * 公式：`scrollLeft' = docX × zoom1 − (clientX − rectLeft)`
 * 其中 `docX` 为缩放前指针处的内容坐标（px）。
 */

import type { Ref } from 'vue';

import { onBeforeUnmount, onMounted, ref } from 'vue';

import { useDesignerStore } from './useDesignerStore';

export function useViewport(scrollerRef: Ref<HTMLElement | undefined>) {
  const store = useDesignerStore();
  const isPanning = ref(false);
  const spaceDown = ref(false);

  let panStart = { x: 0, y: 0, scrollLeft: 0, scrollTop: 0 };

  /** 指针锚点缩放 */
  function zoomAtPointer(nextZoom: number, clientX?: number, clientY?: number) {
    const el = scrollerRef.value;
    const prev = store.zoom;
    if (!el || clientX === undefined || clientY === undefined) {
      store.setZoom(nextZoom);
      return;
    }
    const rect = el.getBoundingClientRect();
    const localX = clientX - rect.left;
    const localY = clientY - rect.top;
    // 指针处的内容坐标（px，未缩放）
    const docX = (el.scrollLeft + localX) / prev;
    const docY = (el.scrollTop + localY) / prev;

    store.setZoom(nextZoom);
    const z1 = store.zoom;
    // rAF 等 DOM 应用 transform 后再校正滚动
    requestAnimationFrame(() => {
      el.scrollLeft = docX * z1 - localX;
      el.scrollTop = docY * z1 - localY;
      store.scrollX = el.scrollLeft;
      store.scrollY = el.scrollTop;
    });
  }

  /**
   * 滚轮缩放（§34.5 按用户要求修订：普通滚轮直接缩放，指针锚点）。
   * Ctrl/⌘+滚轮 同样缩放（兼容触控板捏合）。
   */
  function onWheel(e: WheelEvent) {
    const el = scrollerRef.value;
    if (!el) return;
    e.preventDefault();
    const step = e.deltaY > 0 ? -0.1 : 0.1;
    zoomAtPointer(store.zoom + step, e.clientX, e.clientY);
    store.scrollX = el.scrollLeft;
    store.scrollY = el.scrollTop;
  }

  function onScroll() {
    const el = scrollerRef.value;
    if (!el) return;
    store.scrollX = el.scrollLeft;
    store.scrollY = el.scrollTop;
  }

  /** 空格 / 中键拖拽平移 */
  function startPan(e: MouseEvent) {
    const el = scrollerRef.value;
    if (!el) return;
    const isMiddle = e.button === 1;
    const isSpacePan = e.button === 0 && spaceDown.value;
    if (!isMiddle && !isSpacePan) return;
    e.preventDefault();
    isPanning.value = true;
    panStart = {
      x: e.clientX,
      y: e.clientY,
      scrollLeft: el.scrollLeft,
      scrollTop: el.scrollTop,
    };
    window.addEventListener('mousemove', onPanMove);
    window.addEventListener('mouseup', endPan);
  }

  function onPanMove(e: MouseEvent) {
    const el = scrollerRef.value;
    if (!el || !isPanning.value) return;
    el.scrollLeft = panStart.scrollLeft - (e.clientX - panStart.x);
    el.scrollTop = panStart.scrollTop - (e.clientY - panStart.y);
  }

  function endPan() {
    isPanning.value = false;
    window.removeEventListener('mousemove', onPanMove);
    window.removeEventListener('mouseup', endPan);
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.code === 'Space' && !isInputFocused()) {
      spaceDown.value = true;
    }
  }

  function onKeyUp(e: KeyboardEvent) {
    if (e.code === 'Space') spaceDown.value = false;
  }

  function isInputFocused() {
    const a = document.activeElement;
    if (!a) return false;
    const tag = a.tagName;
    return (
      tag === 'INPUT' ||
      tag === 'TEXTAREA' ||
      tag === 'SELECT' ||
      (a as HTMLElement).isContentEditable
    );
  }

  /** 居中显示当前页 */
  function centerPage() {
    const el = scrollerRef.value;
    if (!el) return;
    const pxW = (store.pageWidth / 25.4) * 96 * store.zoom;
    const pxH = (store.pageHeight / 25.4) * 96 * store.zoom;
    el.scrollLeft = Math.max(0, (pxW - el.clientWidth) / 2 + 200);
    el.scrollTop = Math.max(0, (pxH - el.clientHeight) / 2 + 200);
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
  });
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKeyDown);
    window.removeEventListener('keyup', onKeyUp);
    endPan();
  });

  return {
    isPanning,
    spaceDown,
    onWheel,
    onScroll,
    startPan,
    zoomAtPointer,
    centerPage,
  };
}
