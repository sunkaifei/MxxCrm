import { requestClient } from '#/api/request';

export const getInventoryListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/inventory/list', { params });
};
export const getInventoryInfoApi = async (id: number) => {
  return requestClient.get('/api/system/inventory/info', { params: { id } });
};
