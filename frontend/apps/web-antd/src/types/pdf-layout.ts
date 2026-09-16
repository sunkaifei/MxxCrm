/**
 * 可视化 PDF 模板设计器 —— 类型定义
 *
 * 与后端 `pdf_layout_schema.rs` / 设计文档 §2（layout_json v1）严格对齐。
 * 单位约定：x/y/w/h 一律为 **纸张绝对坐标（mm）**，与编译期 `#place(dx, dy)` 一致。
 */

/** 元素类型（13 类，§2.4） */
export type PdfElementType =
  | 'barcode'
  | 'ellipse'
  | 'field'
  | 'image'
  | 'line'
  | 'pageNumber'
  | 'qrcode'
  | 'rect'
  | 'repeater'
  | 'seal'
  | 'signature'
  | 'table'
  | 'text';

/** 数据视图（§32.3） */
export type PdfDataMode = 'outline' | 'placeholder' | 'real' | 'sample';

/** 样本预设集（§32.6） */
export type PdfSamplePreset =
  | 'empty'
  | 'long'
  | 'longlist'
  | 'mixed'
  | 'typical'
  | 'unicode'
  | 'zero';

export interface PdfMargin {
  bottom: number;
  left: number;
  right: number;
  top: number;
}

export interface PdfStyles {
  align: 'center' | 'justify' | 'left' | 'right';
  bold: boolean;
  borderColor?: string;
  borderWidth: number;
  color: string;
  fontFamily?: null | string;
  fontSize: number;
  italic: boolean;
  lineHeight: number;
  padding: number;
  underline: boolean;
  valign: 'bottom' | 'middle' | 'top';
}

export interface PdfBackground {
  fit: 'contain' | 'cover' | 'fill';
  opacity: number;
  /** 区别于运行时：预印纸底图绝不随正式出图打印（§19.3） */
  previewOnly: boolean;
  type: 'color' | 'image' | 'none';
  value?: null | string;
}

export interface PdfWatermark {
  image?: null | string;
  opacity: number;
  rotate: number;
  /** 每联都印 / 仅首联 / 仅尾联（§16.3） */
  scope?: 'all' | 'first' | 'last';
  text?: null | string;
}

export interface PdfSeamSeal {
  assetId?: null | number;
  enabled: boolean;
  /** 骑缝章横向位置：跨页时逐页偏移 */
  position?: 'left' | 'right';
  /** 上下偏移 mm */
  offset?: number;
  size: number;
}

export interface PdfPage {
  background: PdfBackground;
  height: number;
  margin: PdfMargin;
  orientation: 'landscape' | 'portrait';
  size: 'A4' | 'A5' | 'custom';
  watermark: PdfWatermark;
  width: number;
}

/** 条件显示（§16.4） */
export interface PdfPrintIf {
  field: string;
  op: 'contains' | 'eq' | 'gt' | 'gte' | 'lt' | 'lte' | 'ne' | 'notEmpty';
  value?: boolean | number | string;
}

/** 国际化文案（§21.2） */
export interface PdfI18nText {
  en?: string;
  zh?: string;
}

export interface PdfTableColumn {
  align: 'center' | 'left' | 'right';
  bind: string;
  format?: null | string;
  title: string | PdfI18nText;
  /** 列宽：数字 = mm；'auto' = 自适应 */
  total?: 'avg' | 'count' | 'max' | 'min' | 'sum' | null;
  width: number | 'auto';
}

export interface PdfElementProps {
  [key: string]: any;
  /** 表格：数据源（items / items2 ...） */
  dataSource?: string;
  columns?: PdfTableColumn[];
  /** 表格：每页固定行数（0 = 不限制，按 y 自动分页） */
  rowsPerPage?: number;
  headerHeight?: number;
  headerRepeat?: boolean;
  rowHeight?: number;
  zebra?: boolean;
  emptyText?: string;
  showIndex?: boolean;
}

export interface PdfElement {
  /** 版式级竖直锚点，用于跨页三段切分（§3.3） */
  band?: 'auto' | 'foot' | 'head' | 'table';
  bind?: { format?: null | string; path?: null | string };
  h: number;
  /** 仅设计态可见（预印纸参考线等） */
  hiddenInOutput?: boolean;
  i18n?: null | PdfI18nText;
  id: string;
  locked?: boolean;
  name: string;
  page?: null | number;
  props?: PdfElementProps;
  printIf?: null | PdfPrintIf;
  rotate?: number;
  style?: Partial<PdfStyles>;
  type: PdfElementType;
  /** 静态文案（type=text） */
  value?: null | string;
  w: number;
  x: number;
  y: number;
  z?: number;
}

export interface PdfLayoutSettings {
  /** ⚠️ 仅设计器画布使用；正式出图 **必须完全忽略**（§32.15 防呆红线） */
  designSample?: {
    itemRows?: number;
    preset?: PdfSamplePreset;
  };
  /** 联次（§16.2） */
  copies?: {
    enabled: boolean;
    labels?: string[];
    mode: 'per_page' | 'per_sheet';
    count: number;
  };
  /** 格线吸附阈值（mm） */
  snapEnabled?: boolean;
  snapThreshold?: number;
  /** 打印偏移校准（P2-6，§19.4）：X 右移 / Y 下移，编译叠加到页边距 */
  printOffset?: {
    x: number;
    y: number;
  };
}

export interface PdfLayoutJson {
  elements: PdfElement[];
  page: PdfPage;
  seamSeal?: PdfSeamSeal;
  settings?: PdfLayoutSettings;
  styles?: Partial<PdfStyles>;
  version: number;
}

/** 字段树节点 */
export interface PdfFieldNode {
  children?: PdfFieldNode[];
  dataType: string;
  fieldName: string;
  fieldPath: string;
  groupCode: string;
  groupName: string;
  id?: string;
  /** 超长样本（用于溢出风险检测，§32.6） */
  sampleLong?: string;
  /** 典型样本 */
  sample?: string;
}

export interface PdfFieldGroup {
  children: PdfFieldNode[];
  groupCode: string;
  groupName: string;
}

export interface PdfBindingIssue {
  code:
    | 'condition_never'
    | 'dangling'
    | 'empty_always'
    | 'invalid_format'
    | 'overflow'
    | 'overlap'
    | 'unbound';
  elementId?: string;
  elementName?: string;
  level: 'error' | 'info' | 'warn';
  message: string;
}

export interface PdfValidateBindingsResult {
  issues: PdfBindingIssue[];
  summary: { error: number; info: number; warn: number };
}

export interface PdfHealthCheck {
  /** 合计未落在末页 */
  grandTotalNotOnLastPage?: boolean;
  pageBreakY?: number;
  /** 最后一页近空白（可能是多余尾页） */
  trailingBlankPage?: boolean;
  warnings?: string[];
}

export interface PdfPreviewResult {
  compileMs: number;
  format: 'svg' | 'url';
  health?: PdfHealthCheck;
  index?: number;
  pageBreakY?: number;
  pageCount: number;
  pages?: Array<{ index: number; url: string }>;
  svg?: string;
  token?: string;
  warnings?: string[];
}

export interface PdfAssetVO {
  category: string;
  createTime?: string;
  filePath?: string;
  fileSize?: number;
  fileUrl: string;
  heightPx?: number;
  id: string;
  md5?: string;
  name: string;
  sort: number;
  status: number;
  widthPx?: number;
}

export interface PdfDocType {
  code: string;
  fieldsCount?: number;
  /** 合规提示（发票等） */
  hint?: string;
  name: string;
  /** 是否支持可视化版式设计（发票为 false，§20.1） */
  designable: boolean;
}

export interface PdfEditLockInfo {
  expireAt?: string;
  granted?: boolean;
  locked?: boolean;
  userId?: string;
  userName?: string;
}
