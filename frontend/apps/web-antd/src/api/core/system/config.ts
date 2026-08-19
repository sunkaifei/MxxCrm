import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getConfigListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/config/list', { params });
};
export const getConfigInfoApi = async (id: number) => {
  return requestClient.get(`/api/system/config/detail/${id}`);
};
export const createConfigApi = async (param: any) => {
  return requestClient.post('/api/system/config/add', param);
};
export const updateConfigApi = async (id: number, param: any) => {
  return requestClient.put(`/api/system/config/update/${id}`, param);
};
export const deleteConfigApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/config/batch_delete', {
    data: { ids },
  });
};
