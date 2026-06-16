import { requestClient } from '#/api/request';

export const getPurchaseItemListApi = async (params?: PageParams) => {
  return requestClient.get('/api/purchase/item/list', { params });
};
export const getPurchaseItemInfoApi = async (id: number) => {
  return requestClient.get('/api/purchase/item/info', { params: { id } });
};
