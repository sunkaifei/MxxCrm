/**
 * 可视化 PDF 模板设计器 —— 状态与命令栈。
 *
 * 设计依据：设计文档 §8（前端 UI 规格）、§26（工程结构）、§34（画布工作区模型）。
 *
 * 核心约定：
 * - 元素坐标一律 **纸张绝对 mm**（x/y/w/h），不做屏幕坐标存储。
 * - 撤销/重做采用**快照命令栈**（元素 ≤ 200，深拷贝成本远小于增量补丁的复杂度风险）。
 * - 吸附计算一律在 **mm 域**进行（§34.7 红线：屏幕域吸附会随缩放漂移）。
 * - `layout.settings.designSample` 仅驱动画布，**不参与出图**（§32.15 防呆红线）。
 */

import type {
  PdfDataMode,
  PdfElement,
  PdfElementType,
  PdfLayoutJson,
  PdfSamplePreset,
} from '#/types/pdf-layout';

import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

import { paperSize, round } from '#/composables/pdf-designer/useUnits';

/** 元素类型默认尺寸与文案（拖入画布时的初始值，§2.4） */
export const ELEMENT_PRESETS: Record<
  PdfElementType,
  { h: number; icon: string; label: string; w: number }
> = {
  text: { h: 8, icon: 'type', label: '静态文本', w: 60 },
  field: { h: 6, icon: 'variable', label: '数据字段', w: 45 },
  image: { h: 20, icon: 'image', label: '图片', w: 30 },
  seal: { h: 30, icon: 'stamp', label: '电子签章', w: 30 },
  signature: { h: 14, icon: 'pen-tool', label: '手写签名', w: 40 },
  barcode: { h: 12, icon: 'barcode', label: '条码', w: 45 },
  qrcode: { h: 22, icon: 'qr-code', label: '二维码', w: 22 },
  line: { h: 0.5, icon: 'minus', label: '直线', w: 60 },
  rect: { h: 20, icon: 'square', label: '矩形 / 表框', w: 60 },
  ellipse: { h: 20, icon: 'circle', label: '圆形', w: 20 },
  table: { h: 60, icon: 'table', label: '明细表格', w: 186 },
  pageNumber: { h: 6, icon: 'hash', label: '页码', w: 30 },
  repeater: { h: 30, icon: 'repeat', label: '重复块', w: 90 },
};

let seq = 0;
const newId = (type: string) => `${type}_${Date.now().toString(36)}${(seq++).toString(36)}`;

export const useDesignerStore = defineStore('pdfDesigner', () => {
  // ---------------------------------------------------------------------------
  // 基础状态
  // ---------------------------------------------------------------------------

  const templateId = ref<null | number>(null);
  const templateName = ref('');
  const templateCode = ref('');
  const docType = ref('order');
  const version = ref(1);
  /** 是否为系统内置 / 不可设计类型（发票等，§20.1） */
  const designable = ref(true);

  const layout = ref<PdfLayoutJson>(emptyLayout());
  const selection = ref<string[]>([]);
  const dirty = ref(false);
  const saving = ref(false);

  // 视图
  const zoom = ref(1);
  const scrollX = ref(0);
  const scrollY = ref(0);
  const showGrid = ref(true);
  const showRuler = ref(true);
  const showMarginGuide = ref(true);
  const snap = ref(true);
  /** 吸附阈值的**屏幕**像素（换算到 mm 域后使用） */
  const snapPx = ref(6);

  // 数据视图（§32）
  const dataMode = ref<PdfDataMode>('sample');
  const samplePreset = ref<PdfSamplePreset>('typical');
  const itemRows = ref(3);
  const realDocId = ref<null | number>(null);
  const dataContext = ref<Record<string, any>>({});

  // 绑定健康度
  const issues = ref<any[]>([]);

  // ---------------------------------------------------------------------------
  // 派生
  // ---------------------------------------------------------------------------

  const pageWidth = computed(() => layout.value.page.width);
  const pageHeight = computed(() => layout.value.page.height);

  const elements = computed(() => layout.value.elements);

  const selectedElements = computed(() =>
    layout.value.elements.filter((e) => selection.value.includes(e.id)),
  );

  const singleSelected = computed<PdfElement | null>(() =>
    selectedElements.value.length === 1 ? selectedElements.value[0]! : null,
  );

  /** 吸附阈值（mm）——屏幕锚点换算，保证不同缩放下手感一致 */
  const snapMm = computed(() => snapPx.value / (96 / 25.4) / zoom.value);

  // ---------------------------------------------------------------------------
  // 命令栈（快照式）
  // ---------------------------------------------------------------------------

  interface Snapshot {
    elements: PdfElement[];
    label: string;
    selection: string[];
  }

  const undoStack = ref<Snapshot[]>([]);
  const redoStack = ref<Snapshot[]>([]);
  const MAX_HISTORY = 100;

  const cloneDeep = <T>(v: T): T => JSON.parse(JSON.stringify(v)) as T;

  /** 在执行会改变 elements 的操作**之前**调用 */
  function commit(label: string) {
    undoStack.value.push({
      elements: cloneDeep(layout.value.elements),
      selection: [...selection.value],
      label,
    });
    if (undoStack.value.length > MAX_HISTORY) undoStack.value.shift();
    redoStack.value = [];
    dirty.value = true;
  }

  function undo() {
    const s = undoStack.value.pop();
    if (!s) return;
    redoStack.value.push({
      elements: cloneDeep(layout.value.elements),
      selection: [...selection.value],
      label: s.label,
    });
    layout.value.elements = s.elements;
    selection.value = s.selection.filter((id) =>
      s.elements.some((e) => e.id === id),
    );
    dirty.value = true;
  }

  function redo() {
    const s = redoStack.value.pop();
    if (!s) return;
    undoStack.value.push({
      elements: cloneDeep(layout.value.elements),
      selection: [...selection.value],
      label: s.label,
    });
    layout.value.elements = s.elements;
    selection.value = s.selection;
    dirty.value = true;
  }

  const canUndo = computed(() => undoStack.value.length > 0);
  const canRedo = computed(() => redoStack.value.length > 0);

  // ---------------------------------------------------------------------------
  // 元素操作
  // ---------------------------------------------------------------------------

  function addElement(
    type: PdfElementType,
    x: number,
    y: number,
    patch: Partial<PdfElement> = {},
  ): PdfElement {
    const preset = ELEMENT_PRESETS[type] ?? ELEMENT_PRESETS.text;
    commit(`添加${preset.label}`);
    const el: PdfElement = {
      id: newId(type),
      type,
      name: preset.label,
      x: round(x),
      y: round(y),
      w: preset.w,
      h: preset.h,
      props: {},
      style: {},
      ...patch,
    };
    if (type === 'text' && !el.value) el.value = '文本内容';
    if (type === 'table') {
      el.props = {
        dataSource: 'items',
        showIndex: true,
        headerRepeat: true,
        headerHeight: 8,
        rowHeight: 8,
        rowsPerPage: 0,
        emptyText: '无明细记录',
        columns: [
          { title: '商品名称', bind: 'item.product_name', width: 'auto', align: 'left' },
          { title: '数量', bind: 'item.quantity', width: 20, align: 'right', format: 'qty' },
          { title: '单价', bind: 'item.price', width: 28, align: 'right', format: 'money' },
          { title: '金额', bind: 'item.amount', width: 30, align: 'right', format: 'money', total: 'sum' },
        ],
        ...el.props,
      };
    }
    if (type === 'pageNumber') el.value = '第 {{page}} 页 / 共 {{pages}} 页';
    layout.value.elements.push(el);
    selection.value = [el.id];
    return el;
  }

  function updateElement(id: string, patch: Partial<PdfElement>, label = '修改元素') {
    const el = layout.value.elements.find((e) => e.id === id);
    if (!el) return;
    commit(label);
    Object.assign(el, patch);
  }

  /** 拖拽/缩放过程中的高频更新：只在开始拖拽时 commit，过程中直接写 */
  function patchElementLive(id: string, patch: Partial<PdfElement>) {
    const el = layout.value.elements.find((e) => e.id === id);
    if (el) Object.assign(el, patch);
  }

  function removeElements(ids: string[]) {
    if (ids.length === 0) return;
    commit('删除元素');
    layout.value.elements = layout.value.elements.filter((e) => !ids.includes(e.id));
    selection.value = selection.value.filter((id) => !ids.includes(id));
  }

  function duplicateElements(ids: string[]) {
    if (ids.length === 0) return;
    commit('复制元素');
    const copies: PdfElement[] = [];
    for (const id of ids) {
      const el = layout.value.elements.find((e) => e.id === id);
      if (!el) continue;
      copies.push({
        ...cloneDeep(el),
        id: newId(el.type),
        name: `${el.name} 副本`,
        x: round(el.x + 3),
        y: round(el.y + 3),
      });
    }
    layout.value.elements.push(...copies);
    selection.value = copies.map((c) => c.id);
  }

  function reorder(id: string, dir: 'down' | 'top' | 'up') {
    const list = layout.value.elements;
    const idx = list.findIndex((e) => e.id === id);
    if (idx < 0) return;
    commit('调整层级');
    const [el] = list.splice(idx, 1);
    if (!el) return;
    if (dir === 'top') list.push(el);
    else if (dir === 'up') list.splice(Math.max(0, idx - 1), 0, el);
    else list.splice(Math.min(list.length, idx + 1), 0, el);
  }

  // ---------------------------------------------------------------------------
  // 视图
  // ---------------------------------------------------------------------------

  function setZoom(z: number) {
    zoom.value = Math.min(4, Math.max(0.25, round(z, 3)));
  }

  function zoomBy(delta: number) {
    setZoom(zoom.value + delta);
  }

  function fitTo(containerW: number, containerH: number, padding = 60) {
    const zx = (containerW - padding) / ((pageWidth.value / 25.4) * 96);
    const zy = (containerH - padding) / ((pageHeight.value / 25.4) * 96);
    setZoom(Math.min(zx, zy));
  }

  // ---------------------------------------------------------------------------
  // 布局装载
  // ---------------------------------------------------------------------------

  function emptyLayout(): PdfLayoutJson {
    const size = paperSize('A4', 'portrait');
    return {
      version: 1,
      page: {
        size: 'A4',
        orientation: 'portrait',
        width: size.width,
        height: size.height,
        margin: { top: 12, right: 12, bottom: 12, left: 12 },
        background: { type: 'none', fit: 'contain', opacity: 1, previewOnly: false },
        watermark: { text: null, image: null, opacity: 0.08, rotate: 45, scope: 'all' },
      },
      styles: {
        fontFamily: 'Source Han Sans SC',
        fontSize: 10,
        color: '#000000',
        lineHeight: 1.5,
        align: 'left',
        valign: 'top',
      },
      elements: [],
      settings: {
        snapEnabled: true,
        snapThreshold: 1,
        copies: { enabled: false, count: 1, mode: 'per_page', labels: ['存根', '客户', '财务'] },
        designSample: { preset: 'typical', itemRows: 3 },
      },
    };
  }

  function loadLayout(raw: PdfLayoutJson | null | undefined, meta?: {
    code?: string;
    designable?: boolean;
    docType?: string;
    id?: number;
    name?: string;
    version?: number;
  }) {
    layout.value = raw && raw.elements ? cloneDeep(raw) : emptyLayout();
    if (!layout.value.settings) layout.value.settings = emptyLayout().settings;
    if (!layout.value.page.watermark) {
      layout.value.page.watermark = emptyLayout().page.watermark;
    }
    if (meta) {
      templateId.value = meta.id ?? null;
      templateName.value = meta.name ?? '';
      templateCode.value = meta.code ?? '';
      docType.value = meta.docType ?? 'order';
      version.value = meta.version ?? 1;
      if (meta.designable !== undefined) designable.value = meta.designable;
    }
    selection.value = [];
    undoStack.value = [];
    redoStack.value = [];
    dirty.value = false;
    const ds = layout.value.settings?.designSample;
    if (ds?.preset) samplePreset.value = ds.preset;
    if (ds?.itemRows) itemRows.value = ds.itemRows;
  }

  /** 导出给后端的 payload（剥离仅设计态字段之外的运行时杂质，并回写 designSample） */
  function toPayload(): PdfLayoutJson {
    const out = cloneDeep(layout.value);
    out.settings = {
      ...out.settings,
      designSample: { preset: samplePreset.value, itemRows: itemRows.value },
    };
    out.elements = out.elements.map((e) => ({
      ...e,
      x: round(e.x),
      y: round(e.y),
      w: round(e.w),
      h: round(e.h),
    }));
    return out;
  }

  /** 画布上相对 page 的坐标是否在纸外（C/D 档，§34.10） */
  function outOfPaperCount(): number {
    return layout.value.elements.filter(
      (e) =>
        e.x < 0 ||
        e.y < 0 ||
        e.x + e.w > pageWidth.value ||
        e.y + e.h > pageHeight.value * 4,
    ).length;
  }

  return {
    // state
    layout,
    selection,
    dirty,
    saving,
    templateId,
    templateName,
    templateCode,
    docType,
    version,
    designable,
    zoom,
    scrollX,
    scrollY,
    showGrid,
    showRuler,
    showMarginGuide,
    snap,
    snapPx,
    dataMode,
    samplePreset,
    itemRows,
    realDocId,
    dataContext,
    issues,
    // derived
    pageWidth,
    pageHeight,
    elements,
    selectedElements,
    singleSelected,
    snapMm,
    canUndo,
    canRedo,
    // actions
    commit,
    undo,
    redo,
    addElement,
    updateElement,
    patchElementLive,
    removeElements,
    duplicateElements,
    reorder,
    setZoom,
    zoomBy,
    fitTo,
    loadLayout,
    emptyLayout,
    toPayload,
    outOfPaperCount,
    cloneDeep,
  };
});
