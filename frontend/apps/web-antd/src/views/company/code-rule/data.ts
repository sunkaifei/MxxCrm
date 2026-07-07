import { $t } from '#/locales';
import type { SegmentConfig } from '#/api';

/**
 * 段位类型选项
 */
export const segmentTypeOptions = [
  { label: '公司简称', value: 'company' },
  { label: '业务类型', value: 'biz_type' },
  { label: '年份', value: 'year' },
  { label: '部门编码', value: 'dept' },
  { label: '流水号', value: 'seq' },
  { label: '版本号', value: 'version' },
  { label: '固定文本', value: 'fixed' },
  { label: '日期', value: 'date' },
];

/**
 * 年份格式选项
 */
export const yearFormatOptions = [
  { label: '4位 (2026)', value: 'yyyy' },
  { label: '2位 (26)', value: 'yy' },
];

/**
 * 年份来源选项
 */
export const yearSourceOptions = [
  { label: '当前年份', value: 'current' },
  { label: '业务日期年份', value: 'business_date' },
  { label: '记录创建时间年份', value: 'create_time' },
];

/**
 * 日期格式选项
 */
export const dateFormatOptions = [
  { label: '年月 (202607)', value: 'yyyyMM' },
  { label: '年月日 (20260704)', value: 'yyyyMMdd' },
];

/**
 * 段位类型中文名映射
 */
export const segmentTypeLabelMap: Record<string, string> = {
  company: '公司简称',
  biz_type: '业务类型',
  year: '年份',
  dept: '部门编码',
  seq: '流水号',
  version: '版本号',
  fixed: '固定文本',
  date: '日期',
};

/**
 * 流水号位数选项
 */
export const seqLengthOptions = [
  { label: '3位 (001-999)', value: 3 },
  { label: '4位 (0001-9999)', value: 4 },
  { label: '5位 (00001-99999)', value: 5 },
  { label: '6位 (000001-999999)', value: 6 },
];

/**
 * 分隔符选项
 */
export const separatorOptions = [
  { label: '- (默认)', value: '-' },
  { label: '_ (下划线)', value: '_' },
  { label: '. (点号)', value: '.' },
  { label: '/ (斜杠)', value: '/' },
  { label: ' (无)', value: '' },
];

/**
 * 判断段位类型是否需要 value 输入框
 */
export function segTypeNeedValue(type: string): boolean {
  return type === 'fixed' || type === 'biz_type';
}

/**
 * 判断段位类型是否需要 format 选项
 */
export function segTypeNeedFormat(type: string): boolean {
  return type === 'year' || type === 'date';
}

/**
 * 判断段位类型是否需要 source 选项
 */
export function segTypeNeedSource(type: string): boolean {
  return type === 'year';
}

/**
 * 判断段位类型是否需要 length 选项
 */
export function segTypeNeedLength(type: string): boolean {
  return type === 'seq';
}

/**
 * 生成新段位的默认值
 */
export function createDefaultSegment(type: string, sort: number): SegmentConfig {
  const seg: SegmentConfig = { type: type as SegmentConfig['type'], sort };
  if (type === 'year') {
    seg.format = 'yyyy';
    seg.source = 'current';
  } else if (type === 'date') {
    seg.format = 'yyyyMMdd';
  } else if (type === 'seq') {
    seg.length = 4;
  } else if (type === 'version') {
    seg.value = 'V1';
  }
  return seg;
}

/**
 * 段位类型是否可以重复添加（除固定文本外，其他段位类型不可重复）
 */
export function canAddSegmentType(type: string, segments: SegmentConfig[]): boolean {
  if (type === 'fixed') return true;
  return !segments.some((s) => s.type === type);
}

/**
 * 把段位数组渲染成预览字符串（仅用于显示，实际预览走后端 API）
 */
export function segmentsToPreviewText(
  segments: SegmentConfig[],
  companyAbbr = 'XYH',
  bizType = 'KH',
  deptCode = 'XS',
  separator = '-',
): string {
  const sorted = [...segments].sort((a, b) => a.sort - b.sort);
  const parts: string[] = [];
  for (const seg of sorted) {
    let part = '';
    switch (seg.type) {
      case 'company': part = companyAbbr; break;
      case 'biz_type': part = bizType; break;
      case 'year': part = '2026'; break;
      case 'dept': part = deptCode; break;
      case 'seq': part = '0'.repeat(seg.length ?? 4) + '1'; part = part.slice(-Math.max(1, seg.length ?? 4)); break;
      case 'version': part = seg.value ?? 'V1'; break;
      case 'fixed': part = seg.value ?? ''; break;
      case 'date': part = '20260704'; break;
    }
    if (part) parts.push(part);
  }
  return parts.join(separator);
}
