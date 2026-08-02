import { requestClient } from '#/api/request';

export interface TemplateVarVO {
  id: number;
  varKey: string;
  varLabel?: string;
  varValue?: string;
  varType: number;
  varGroup: string;
  sort: number;
  status: number;
  createTime: string;
}

export interface TemplateVarListQuery {
  keywords?: string;
  varGroup?: string;
  page?: number;
  pageSize?: number;
}

export interface TemplateVarSaveDTO {
  id?: number;
  varKey?: string;
  varLabel?: string;
  varValue?: string;
  varType?: number;
  varGroup?: string;
  sort?: number;
  status?: number;
}

export interface TemplateRevisionVO {
  id: number;
  templateDataId: number;
  temptext: string;
  revisionNote?: string;
  createBy?: number;
  createTime: string;
}

export const getTemplateVarListApi = async (params?: TemplateVarListQuery) => {
  return requestClient.get('/api/system/template/var/list', { params });
};

export const getTemplateVarDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/template/var/detail/${id}`);
};

export const getAllTemplateVarsApi = async () => {
  return requestClient.get('/api/system/template/var/all');
};

export const addTemplateVarApi = async (data: TemplateVarSaveDTO) => {
  return requestClient.post('/api/system/template/var/add', data);
};

export const updateTemplateVarApi = async (
  id: number,
  data: TemplateVarSaveDTO,
) => {
  return requestClient.put(`/api/system/template/var/update/${id}`, data);
};

export const deleteTemplateVarApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/template/var/batch_delete', {
    data: { ids },
  });
};

export const getTemplateRevisionListApi = async (templateDataId: number) => {
  return requestClient.get(`/api/system/template/revision/list/${templateDataId}`);
};

export const getTemplateRevisionDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/template/revision/detail/${id}`);
};
