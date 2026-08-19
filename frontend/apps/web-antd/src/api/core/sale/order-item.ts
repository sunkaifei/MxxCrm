import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getOrderItemListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/sale/order-item/list', { params });
};
export const getOrderItemInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/order-item/info', {
    params: { id },
  });
};
