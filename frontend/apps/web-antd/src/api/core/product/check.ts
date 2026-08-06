import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getCheckListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/inventory/check/list', { params });
};

export const getCheckInfoApi = async (id: number) => {
  return requestClient.get('/api/system/inventory/check/info', { params: { id } });
};

export const createCheckApi = async (data: any) => {
  return requestClient.post('/api/system/inventory/check/save', data);
};

export const updateCheckApi = async (data: any) => {
  const { id, ...rest } = data;
  return requestClient.put(`/api/system/inventory/check/update/${id}`, rest);
};

export const deleteCheckApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/inventory/check/batch_delete', {
    data: { ids },
  });
};

export const auditCheckApi = async (id: number) => {
  return requestClient.post('/api/system/inventory/check/audit', { id });
};