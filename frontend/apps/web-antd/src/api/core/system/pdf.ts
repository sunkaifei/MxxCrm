import { requestClient } from '#/api/request';

// PDF生成请求
export interface PdfGenerateRequest {
  docType: string; // quotation/order/contract
  docId: number | string;
  templateId?: number | string;
}

// PDF生成结果
export interface PdfGenerateResult {
  recordId: string;
  fileUrl: string;
  filePath: string;
  fileSize: number;
}

// PDF记录VO
export interface PdfRecordVO {
  id: string;
  docType: string;
  docId: string;
  docNo?: string;
  templateId: string;
  templateName?: string;
  fileName: string;
  filePath: string;
  fileUrl?: string;
  fileSize?: number;
  pageCount?: number;
  triggerType: string;
  status: number;
  errorMsg?: string;
  createTime?: string;
}

// PDF记录列表查询
export interface PdfRecordListQuery {
  docType: string;
  docId: number | string;
  page?: number;
  pageSize?: number;
}

// PDF全局记录列表查询（管理后台用）
export interface PdfRecordAllQuery {
  page?: number;
  pageSize?: number;
  docType?: string;
  docNo?: string;
  triggerType?: string;
}

// PDF记录分页结果
export interface PdfRecordPageResult {
  list: PdfRecordVO[];
  total: number;
}

// API 函数
export const generatePdfApi = async (data: PdfGenerateRequest) =>
  requestClient.post('/api/system/pdf/generate', data);

export const previewPdfApi = async (params: {
  docId: number | string;
  docType: string;
  templateId?: number | string;
}) => requestClient.get('/api/system/pdf/preview', { params });

export const downloadPdfApi = async (recordId: number | string) =>
  requestClient.get('/api/system/pdf/download', {
    params: { id: recordId },
    responseType: 'blob',
    responseReturn: 'body',
  } as any);

// 模板演示PDF（带鉴权，返回blob）
export const demoPdfApi = async (templateId: number | string) =>
  requestClient.get('/api/system/pdf/demo', {
    params: { id: templateId },
    responseType: 'blob',
    responseReturn: 'body',
  } as any);

export const getPdfRecordListApi = async (params: PdfRecordListQuery) =>
  requestClient.get('/api/system/pdf/record-list', { params });

// 全局PDF记录列表（管理后台用）
export const getPdfRecordAllApi = async (params: PdfRecordAllQuery) =>
  requestClient.get('/api/system/pdf/record-all', { params });

// PDF下载日志查询
export interface PdfDownloadLogQuery {
  page?: number;
  pageSize?: number;
  docType?: string;
  docNo?: string;
}

export const getPdfDownloadLogApi = async (params: PdfDownloadLogQuery) =>
  requestClient.get('/api/system/pdf/download-log', { params });
