import type {
  PdfAssetVO,
  PdfDocType,
  PdfEditLockInfo,
  PdfFieldGroup,
  PdfLayoutJson,
  PdfPreviewResult,
  PdfValidateBindingsResult,
} from '#/types/pdf-layout';

import { requestClient } from '#/api/request';

// ============================================================================
// 元数据
// ============================================================================

/** 支持的单据类型（含 designable 与合规提示） */
export const getPdfDocTypesApi = async () =>
  requestClient.get<PdfDocType[]>('/api/system/pdf-designer/doc-types');

/** 格式化器清单 */
export const getPdfFormattersApi = async () =>
  requestClient.get<any[]>('/api/system/pdf-designer/formatters');

/** 样本预设集（§32.6） */
export const getPdfPresetsApi = async () =>
  requestClient.get<any[]>('/api/system/pdf-designer/presets');

/** 字段树（按 docType） */
export const getPdfFieldTreeApi = async (docType: string) =>
  requestClient.get<PdfFieldGroup[]>('/api/system/pdf-designer/field-tree', {
    params: { docType },
  });

/** 样例数据（设计态默认视图） */
export const getPdfSampleDataApi = async (params: {
  docType: string;
  itemRows?: number;
  preset?: string;
  templateId?: number;
}) =>
  requestClient.get<Record<string, any>>('/api/system/pdf-designer/sample-data', {
    params,
  });

/** 真实数据（上线前验收；需数据权限） */
export const getPdfRealDataApi = async (params: {
  docId: number;
  docType: string;
}) =>
  requestClient.get<Record<string, any>>('/api/system/pdf-designer/real-data', {
    params,
  });

/** 绑定健康度检查（§32.11） */
export const validatePdfBindingsApi = async (data: {
  docType: string;
  layoutJson: PdfLayoutJson;
}) =>
  requestClient.post<PdfValidateBindingsResult>(
    '/api/system/pdf-designer/validate-bindings',
    data,
  );

// ============================================================================
// 模板读写
// ============================================================================

/** 模板详情（含 layoutJson / version / 素材清单） */
export const getPdfDesignerTemplateApi = async (id: number) =>
  requestClient.get<{
    basePdfId?: number;
    docType?: string;
    engine?: string;
    id: string;
    layoutJson?: PdfLayoutJson;
    name?: string;
    previewUrl?: string;
    remark?: string;
    status?: number;
    templateCode?: string;
    updateTime?: string;
    version: number;
  }>('/api/system/pdf-designer/template-info', { params: { id } });

/** 保存 layout_json */
export const savePdfLayoutApi = async (data: {
  basedOnTemplateId?: number;
  docType?: string;
  force?: boolean;
  id?: number;
  layoutJson: PdfLayoutJson;
  name?: string;
  remark?: string;
  templateCode?: string;
  version?: number;
}) => requestClient.post('/api/system/pdf-designer/save-layout', data);

/** 精确预览 / 分页预览（§25.3、§34.6） */
export const pdfPrecisePreviewApi = async (data: {
  dataMode?: string;
  docId?: number;
  docType?: string;
  format?: string;
  itemRows?: number;
  layoutJson?: PdfLayoutJson;
  pages?: number | string;
  preset?: string;
  templateId?: number;
}) =>
  requestClient.post<PdfPreviewResult>(
    '/api/system/pdf-designer/precise-preview',
    data,
  );

/** 取分页预览某页 SVG 的绝对地址（供 <img> / iframe 直接引用） */
export const pdfPreviewSvgUrl = (token: string, page: number) =>
  `/api/system/pdf-designer/preview-svg?token=${encodeURIComponent(
    token,
  )}&page=${page}`;

/** 版本列表 */
export const getPdfVersionsApi = async (templateId: number) =>
  requestClient.get<any[]>('/api/system/pdf-designer/versions', {
    params: { templateId },
  });

/** 回滚版本 */
export const rollbackPdfVersionApi = async (data: {
  templateId: number;
  version: number;
}) => requestClient.post('/api/system/pdf-designer/rollback', data);

/** 复制模板 */
export const duplicatePdfTemplateApi = async (data: {
  id: number;
  name?: string;
}) => requestClient.post('/api/system/pdf-designer/duplicate', data);

/** 设为默认（独立权限；纸外元素需二次确认） */
export const setDefaultPdfTemplateApi = async (data: {
  confirmOutOfPaper?: boolean;
  id: number;
}) => requestClient.put('/api/system/pdf-designer/set-default', data);

/** 上传 PDF 反设计（底图模式） */
export const parsePdfToLayoutApi = async (data: {
  assetId: number;
  docType: string;
  mode?: string;
  pageHeightMm?: number;
  pageWidthMm?: number;
}) => requestClient.post('/api/system/pdf-designer/parse-pdf', data);

// ============================================================================
// 编辑期软锁（§33.1）
// ============================================================================

export const acquirePdfLockApi = async (templateId: number) =>
  requestClient.post<PdfEditLockInfo>('/api/system/pdf-designer/lock/acquire', {
    templateId,
  });

export const heartbeatPdfLockApi = async (templateId: number) =>
  requestClient.post<PdfEditLockInfo>(
    '/api/system/pdf-designer/lock/heartbeat',
    { templateId },
  );

export const releasePdfLockApi = async (templateId: number) =>
  requestClient.post('/api/system/pdf-designer/lock/release', { templateId });

export const getPdfLockApi = async (templateId: number) =>
  requestClient.get<PdfEditLockInfo>('/api/system/pdf-designer/lock/current', {
    params: { templateId },
  });

// ============================================================================
// 模板包（§33.2）
// ============================================================================

/** 导出模板包（浏览器直接下载） */
export const pdfBundleExportUrl = (id: number) =>
  `/api/system/pdf-designer/bundle/export?id=${id}`;

/** 导入模板包 */
export const importPdfBundleApi = async (file: File) => {
  const form = new FormData();
  form.append('file', file);
  // multipart 走原生 fetch：requestClient 会按 msgpack 解包，不适用于二进制上传
  const { useAccessStore } = await import('@vben/stores');
  const token = useAccessStore().accessToken ?? '';
  const resp = await fetch('/api/system/pdf-designer/bundle/import', {
    body: form,
    headers: token ? { Authorization: `Bearer ${token}` } : {},
    method: 'POST',
  });
  return resp.json();
};

// ============================================================================
// 素材
// ============================================================================

export const getPdfAssetListApi = async (params?: {
  category?: string;
  name?: string;
  page?: number;
  pageSize?: number;
}) => requestClient.get('/api/system/pdf-asset/list', { params });

/**
 * 上传素材。
 *
 * ⚠️ 后端返回 **JSON**（`JsonResp`）而非 msgpack，必须走原生 fetch
 * （与 `api/core/product/category.ts` 的 uploadCategoryImageApi 同源问题）。
 */
export const uploadPdfAssetApi = async (file: File, category: string, name?: string) => {
  const form = new FormData();
  form.append('file', file);
  form.append('category', category);
  if (name) form.append('name', name);
  const { useAccessStore } = await import('@vben/stores');
  const token = useAccessStore().accessToken ?? '';
  const resp = await fetch('/api/system/pdf-asset/upload', {
    body: form,
    headers: token ? { Authorization: `Bearer ${token}` } : {},
    method: 'POST',
  });
  const json = await resp.json();
  if (json.code !== 200) {
    throw new Error(json.msg || '素材上传失败');
  }
  return json.data as PdfAssetVO;
};

export const savePdfAssetApi = async (data: {
  category: string;
  filePath?: string;
  fileSize?: number;
  fileUrl: string;
  heightPx?: number;
  id?: number;
  name: string;
  sort?: number;
  status?: number;
  widthPx?: number;
}) => requestClient.post('/api/system/pdf-asset/save', data);

export const deletePdfAssetApi = async (ids: number[]) =>
  requestClient.post('/api/system/pdf-asset/delete', { ids });
