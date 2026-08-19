import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getPurchaseItemListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/purchase/item/list', { params });
};
export const getPurchaseItemInfoApi = async (id: number) => {
  return requestClient.get('/api/system/purchase/item/info', {
    params: { id },
  });
};
