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
