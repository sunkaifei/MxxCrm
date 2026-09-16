/**
 * 标尺绘制（Canvas）与吸附计算（设计文档 §8.5、§34.7）。
 *
 * 标尺是本方案中**唯一使用 Canvas 的部分**（§33.4 渲染引擎决议）：
 * 元素与文本一律 DOM，仅标尺这类"纯几何、高频重绘、无需排版引擎"的图层用 Canvas。
 *
 * ⚠️ 吸附一律在 **mm 域**计算（§34.11 红线）：若在屏幕坐标做吸附，
 * 100% 缩放吸 1mm，200% 缩放就变成吸 0.5mm，精度随缩放漂移。
 */

import { mmToPx } from '#/composables/pdf-designer/useUnits';

export interface GuideLine {
  /** 命中的对齐值（mm） */
  at: number;
  kind: 'h' | 'v';
}

interface RulerOpts {
  /** 粘贴板四周外扩（mm），与 CanvasStage 保持一致 */
  bleed: number;
  zoom: number;
  scrollLeft: number;
  scrollTop: number;
}

const BG = '#fafafa';
const LINE = '#d9d9d9';
const TEXT = '#8c8c8c';
const HIGHLIGHT = '#fa541c';

/** 依据缩放挑选"主刻度"间隔（mm），保证数字不重叠 */
function tickStep(zoom: number): number {
  const pxPerMm = mmToPx(1) * zoom;
  for (const step of [1, 2, 5, 10, 20, 50, 100]) {
    if (step * pxPerMm >= 46) return step;
  }
  return 100;
}

export function drawRuler(
  canvas: HTMLCanvasElement | undefined,
  axis: 'h' | 'v',
  lengthPx: number,
  opts: RulerOpts,
  paperOffsetMm: number,
  cursorMm: number | null,
) {
  if (!canvas) return;
  const dpr = window.devicePixelRatio || 1;
  const thickness = axis === 'h' ? canvas.clientHeight || 20 : canvas.clientWidth || 20;
  const w = axis === 'h' ? lengthPx : thickness;
  const h = axis === 'h' ? thickness : lengthPx;
  if (w <= 0 || h <= 0) return;
  if (canvas.width !== Math.floor(w * dpr) || canvas.height !== Math.floor(h * dpr)) {
    canvas.width = Math.floor(w * dpr);
    canvas.height = Math.floor(h * dpr);
  }
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = BG;
  ctx.fillRect(0, 0, w, h);

  const zoom = opts.zoom;
  const pxPerMm = mmToPx(1) * zoom;
  const scroll = axis === 'h' ? opts.scrollLeft : opts.scrollTop;
  // 内容坐标（px，未缩放）→ 视口坐标：doc*zoom - scroll
  const step = tickStep(zoom);
  const startMm = Math.floor((scroll / zoom / mmToPx(1) - paperOffsetMm) / step) * step;

  ctx.font = '10px system-ui, sans-serif';
  ctx.textBaseline = 'top';
  ctx.strokeStyle = LINE;
  ctx.fillStyle = TEXT;
  ctx.lineWidth = 1;

  const endMm = startMm + (lengthPx / pxPerMm) + step * 2;
  for (let mm = startMm; mm <= endMm; mm += step) {
    const vp = (mm + paperOffsetMm) * pxPerMm - scroll;
    if (vp < -2 || vp > (axis === 'h' ? w : h) + 2) continue;
    const isPaperEdge = Math.abs(mm) < 0.001;
    ctx.strokeStyle = isPaperEdge ? HIGHLIGHT : LINE;
    ctx.beginPath();
    if (axis === 'h') {
      ctx.moveTo(Math.round(vp) + 0.5, h);
      ctx.lineTo(Math.round(vp) + 0.5, isPaperEdge ? 2 : h - 6);
    } else {
      ctx.moveTo(w, Math.round(vp) + 0.5);
      ctx.lineTo(isPaperEdge ? 2 : w - 6, Math.round(vp) + 0.5);
    }
    ctx.stroke();
    ctx.fillStyle = isPaperEdge ? HIGHLIGHT : TEXT;
    if (axis === 'h') {
      ctx.fillText(String(mm), Math.round(vp) + 3, 2);
    } else {
      ctx.save();
      ctx.translate(2, Math.round(vp) + 3);
      ctx.rotate(Math.PI / 2);
      ctx.textBaseline = 'top';
      ctx.fillText(String(mm), 0, -10);
      ctx.restore();
    }
  }

  // 指针位置指示线
  if (cursorMm !== null && Number.isFinite(cursorMm)) {
    const vp = (cursorMm + paperOffsetMm) * pxPerMm - scroll;
    ctx.strokeStyle = HIGHLIGHT;
    ctx.beginPath();
    if (axis === 'h') {
      ctx.moveTo(Math.round(vp) + 0.5, 0);
      ctx.lineTo(Math.round(vp) + 0.5, h);
    } else {
      ctx.moveTo(0, Math.round(vp) + 0.5);
      ctx.lineTo(w, Math.round(vp) + 0.5);
    }
    ctx.stroke();
  }
}

// ============================================================================
// 吸附与对齐辅助线（mm 域）
// ============================================================================

export interface SnapInput {
  /** 拖拽中的元素框（mm） */
  box: { h: number; w: number; x: number; y: number };
  /** 吸附阈值（mm） */
  threshold: number;
  /** 其他元素框（排除自身） */
  others: Array<{ h: number; w: number; x: number; y: number }>;
  /** 纸张几何（mm） */
  page: { height: number; margin: { bottom: number; left: number; right: number; top: number }; width: number };
}

export interface SnapResult {
  guides: GuideLine[];
  x: number;
  y: number;
}

const near = (a: number, b: number, t: number) => Math.abs(a - b) <= t;

/** 计算吸附后的位置 + 对齐辅助线 */
export function computeSnap(input: SnapInput): SnapResult {
  const { box, threshold: t, others, page } = input;
  const vCandidates: number[] = [0, page.width / 2, page.width];
  const hCandidates: number[] = [0, page.height / 2, page.height];
  for (const m of [page.margin.left, page.width - page.margin.right]) vCandidates.push(m);
  for (const m of [page.margin.top, page.height - page.margin.bottom]) hCandidates.push(m);
  for (const o of others) {
    vCandidates.push(o.x, o.x + o.w / 2, o.x + o.w);
    hCandidates.push(o.y, o.y + o.h / 2, o.y + o.h);
  }

  const guides: GuideLine[] = [];
  let x = box.x;
  let y = box.y;

  // 竖直方向：元素左/中/右 依次尝试对齐
  const edgesX = [
    { self: box.x, off: 0 },
    { self: box.x + box.w / 2, off: box.w / 2 },
    { self: box.x + box.w, off: box.w },
  ];
  let bestX: null | { at: number; delta: number } = null;
  for (const c of vCandidates) {
    for (const e of edgesX) {
      if (near(e.self, c, t) && (bestX === null || Math.abs(c - e.self) < Math.abs(bestX.delta))) {
        bestX = { at: c, delta: c - e.self };
      }
    }
  }
  if (bestX) {
    x = box.x + bestX.delta;
    guides.push({ kind: 'v', at: bestX.at });
  }

  const edgesY = [
    { self: box.y, off: 0 },
    { self: box.y + box.h / 2, off: box.h / 2 },
    { self: box.y + box.h, off: box.h },
  ];
  let bestY: null | { at: number; delta: number } = null;
  for (const c of hCandidates) {
    for (const e of edgesY) {
      if (near(e.self, c, t) && (bestY === null || Math.abs(c - e.self) < Math.abs(bestY.delta))) {
        bestY = { at: c, delta: c - e.self };
      }
    }
  }
  if (bestY) {
    y = box.y + bestY.delta;
    guides.push({ kind: 'h', at: bestY.at });
  }

  return { x, y, guides };
}

/** 将元素限制在粘贴板范围内（D 档 clamp，§34.10） */
export function clampToBleed(
  box: { h: number; w: number; x: number; y: number },
  pageW: number,
  pageH: number,
  bleed: number,
) {
  return {
    x: Math.min(pageW + bleed - box.w, Math.max(-bleed, box.x)),
    y: Math.min(pageH + bleed - box.h, Math.max(-bleed, box.y)),
  };
}
