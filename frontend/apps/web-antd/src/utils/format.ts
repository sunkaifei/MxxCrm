/**
 * 统一百分比格式化工具函数
 *
 * 数据契约：后端返回的百分比字段已经是「乘过100的数值」（如 90.06 代表 90.06%），
 * 后端已统一 ROUND 到 2 位小数。前端此函数负责：
 *   1) Number / NaN / null / undefined 兜底为 '0.00%'
 *   2) 统一 toFixed(2) 兜底，保证后端没处理到的新字段也能显示 2 位小数
 *   3) 幂等：如果传进来的字符串已经带 %，不会重复拼接
 */

export function formatPercentDisplay(val: any): string {
  if (val === null || val === undefined) return '0.00%';
  let num = Number(val);
  if (Number.isNaN(num)) {
    // 兼容传进来是带%的字符串场景，先去%再解析
    if (typeof val === 'string') {
      const cleaned = val.replaceAll('%', '').trim();
      num = Number(cleaned);
      if (Number.isNaN(num)) return '0.00%';
    } else {
      return '0.00%';
    }
  }
  return `${num.toFixed(2)}%`;
}

/**
 * 只取百分比的数值部分（不带%），用于 a-progress 的 percent 属性等场景
 * 同样保证 2 位小数
 */
export function formatPercentValue(val: any): number {
  if (val === null || val === undefined) return 0;
  const num = Number(val);
  if (Number.isNaN(num)) {
    if (typeof val === 'string') {
      const cleaned = val.replaceAll('%', '').trim();
      const n = Number(cleaned);
      return Number.isNaN(n) ? 0 : Number(n.toFixed(2));
    }
    return 0;
  }
  return Number(num.toFixed(2));
}
