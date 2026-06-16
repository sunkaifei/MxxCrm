import { requestClient } from '#/api/request';

export const getInventoryListApi = async (params?: PageParams) => {
  return requestClient.get('/api/product/inventory/list', { params });
};
export const getInventoryInfoApi = async (id: number) => {
  return requestClient.get('/api/product/inventory/info', { params: { id } });
};
