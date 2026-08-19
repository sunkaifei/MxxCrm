import { requestClient } from '#/api/request';

// PDF模板
export const getPdfTemplateListApi = async (params?: any) =>
  requestClient.get('/api/system/pdf-template/list', { params });

export const getPdfTemplateInfoApi = async (id: number) =>
  requestClient.get('/api/system/pdf-template/info', { params: { id } });

export const savePdfTemplateApi = async (data: any) =>
  requestClient.post('/api/system/pdf-template/save', data);

export const updatePdfTemplateApi = async (data: any) =>
  requestClient.put('/api/system/pdf-template/update', data);

export const bathDeletePdfTemplateApi = async (ids: number[]) =>
  requestClient.delete('/api/system/pdf-template/bath_delete', {
    data: { ids },
  });

export const setDefaultPdfTemplateApi = async (id: number) =>
  requestClient.put('/api/system/pdf-template/set_default', { id });

// PDF模板选项（按单据类型获取可用模板下拉选项）
export interface PdfTemplateOptionVO {
  id: number | string;
  name?: string;
  templateCode?: string;
  isDefault?: number;
}

export const getPdfTemplateOptionsApi = async (docType: string) =>
  requestClient.get('/api/system/pdf-template/options', {
    params: { docType },
  });
