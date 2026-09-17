import { getFormLayoutApi } from '#/api/core/system/form-layout';

/** 解析后的布局元数据（mxx_system_form_layout.layout_json） */
export interface ParsedFormLayout {
  version: number;
  tabs: { key: string; title: string }[];
  fields: { key: string; source: string; tab?: string; span: 1 | 2 }[];
  unassignedPolicy: string;
}

/**
 * 取模块布局；无布局/停用/出错返回 null（渲染端回落默认逻辑，保证"现用设计不变"）
 */
export async function fetchFormLayout(
  module: string,
  layoutType: number,
): Promise<ParsedFormLayout | null> {
  try {
    const res: any = await getFormLayoutApi(module, layoutType);
    const json = res?.layoutJson;
    if (!res || !json) return null;
    return {
      version: Number(res.version) || 1,
      tabs: Array.isArray(json.tabs)
        ? json.tabs.map((t: any) => ({ key: String(t.key), title: String(t.title) }))
        : [],
      fields: Array.isArray(json.fields)
        ? json.fields.map((f: any) => ({
            key: String(f.key),
            source: f.source === 'system' ? 'system' : 'field_def',
            tab: f.tab ? String(f.tab) : undefined,
            span: Number(f.span) === 2 ? 2 : 1,
          }))
        : [],
      unassignedPolicy: json.unassignedPolicy === 'hidden' ? 'hidden' : 'append',
    };
  } catch {
    return null;
  }
}

/**
 * 按布局顺序重排字段清单：布局内字段按布局序在前，未编排字段按原相对顺序追加（unassignedPolicy=append 语义）
 */
export function orderFieldsByLayout<T extends { fieldKey: string }>(
  items: T[],
  layout: ParsedFormLayout | null,
): T[] {
  if (!layout || layout.fields.length === 0) return items;
  const orderMap = new Map<string, number>();
  layout.fields.forEach((f, i) => orderMap.set(f.key, i));
  const inLayout = items.filter((i) => orderMap.has(i.fieldKey));
  const rest = items.filter((i) => !orderMap.has(i.fieldKey));
  inLayout.sort(
    (a, b) => orderMap.get(a.fieldKey)! - orderMap.get(b.fieldKey)!,
  );
  return [...inLayout, ...rest];
}

/**
 * 表单项列宽：布局 span=2 → 整行（vben formItemClass col-span-2）；
 * 无布局时沿用现用规则：多行文本整行、其余半行
 */
export function formItemSpan(
  item: { fieldKey: string; fieldType: number },
  layout: ParsedFormLayout | null,
): 'col-span-2' | '' {
  if (layout) {
    const hit = layout.fields.find((f) => f.key === item.fieldKey);
    if (hit) return hit.span === 2 ? 'col-span-2' : '';
  }
  return Number(item.fieldType) === 2 ? 'col-span-2' : '';
}

/**
 * 显示名覆盖：field_def 中该字段的 field_label 即最终显示名
 * （系统字段行由管理员改名后自动全端生效）
 */
export function buildLabelMap(
  schemaItems: { fieldKey: string; fieldLabel: string }[],
): Map<string, string> {
  return new Map(schemaItems.map((i) => [i.fieldKey, i.fieldLabel]));
}

/**
 * 按布局设置抽屉宽度（表单布局 drawerWidth："60%"~"90%"/"100%"=全屏）；
 * 未设置时保持抽屉现用宽度。class 直接替换（与 content-model/drawer.vue 的 setState({class}) 同机制）
 */
export function applyDrawerWidth(
  drawerApi: { setState?: (v: any) => void } | undefined,
  layout: ParsedFormLayout | null,
) {
  if (!drawerApi?.setState || !layout) return;
  const w = (layout as any).drawerWidth as string | undefined;
  if (!w) return;
  const cls = w === '100%' ? 'w-full' : `w-[${w}]`;
  drawerApi.setState({ class: cls });
}
