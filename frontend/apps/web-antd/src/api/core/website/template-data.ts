import { requestClient } from '#/api/request';

export interface TemplateDataVO {
  id: number;
  templateId: number;
  modelId: number;
  typeId: number;
  name: string;
  temptext: string;
  sort: number;
  status: number;
  createTime: string;
}

export interface TemplateDataListQuery {
  keywords?: string;
  templateId?: number;
  modelId?: number;
  typeId?: number;
  status?: number;
  page?: number;
  pageSize?: number;
}

export interface TemplateDataSaveDTO {
  templateId?: number;
  modelId?: number;
  typeId?: number;
  name?: string;
  temptext?: string;
  sort?: number;
  status?: number;
}

export const getTemplateDataListApi = async (params?: TemplateDataListQuery) => {
  return requestClient.get('/api/system/template/data/list', { params });
};

export const getTemplateDataDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/template/data/detail/${id}`);
};

export const addTemplateDataApi = async (data: TemplateDataSaveDTO) => {
  return requestClient.post('/api/system/template/data/add', data);
};

export const updateTemplateDataApi = async (id: number, data: TemplateDataSaveDTO) => {
  return requestClient.put(`/api/system/template/data/update/${id}`, data);
};

export const deleteTemplateDataApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/template/data/batch_delete', {
    data: { ids },
  });
};

/**
 * TPL-6: 模板预览
 * 不保存模板内容，直接渲染返回 HTML
 * @param data 模板内容字符串 + 模板类型
 * @returns 渲染后的 HTML 字符串
 */
export const previewTemplateDataApi = async (data: {
  temptext: string;
  typeId?: number;
}): Promise<string> => {
  return requestClient.post('/api/system/template/data/preview', data, {
    responseType: 'text',
    headers: { Accept: 'text/html' },
  });
};
