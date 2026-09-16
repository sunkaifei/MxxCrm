/**
 * 单位换算（与后端 §17.7 严格对齐）。
 *
 * 基准：CSS 规范 1in = 96px，1in = 25.4mm，1in = 72pt。
 * 设计器内部一律以 **mm** 为唯一真值单位，仅在渲染/量测时换算。
 */

export const MM_PER_INCH = 25.4;
export const PX_PER_INCH = 96;
export const PT_PER_INCH = 72;

/** mm → CSS px（@96dpi） */
export const mmToPx = (mm: number): number => (mm / MM_PER_INCH) * PX_PER_INCH;

/** CSS px → mm */
export const pxToMm = (px: number): number => (px / PX_PER_INCH) * MM_PER_INCH;

/** mm → pt（Typst 内部单位） */
export const mmToPt = (mm: number): number => (mm / MM_PER_INCH) * PT_PER_INCH;

/** pt → mm */
export const ptToMm = (pt: number): number => (pt / PT_PER_INCH) * MM_PER_INCH;

/** 纸张预设（mm） */
export const PAPER_SIZES: Record<string, { height: number; width: number }> = {
  A3: { height: 420, width: 297 },
  A4: { height: 297, width: 210 },
  A5: { height: 210, width: 148 },
  letter: { height: 279.4, width: 215.9 },
};

/** 取纸张尺寸（考虑横竖向） */
export function paperSize(
  size: string,
  orientation: 'landscape' | 'portrait',
): { height: number; width: number } {
  const base = PAPER_SIZES[size] ?? PAPER_SIZES.A4!;
  return orientation === 'landscape'
    ? { height: base.width, width: base.height }
    : { height: base.height, width: base.width };
}

/** 保留 n 位小数（规避浮点噪声，写回 layout_json 前统一处理） */
export const round = (v: number, digits = 2): number => {
  const f = 10 ** digits;
  return Math.round(v * f) / f;
};

/** 数值 + 单位（mm）显示 */
export const fmtMm = (v: number | undefined, digits = 2): string =>
  `${round(v ?? 0, digits)}mm`;
