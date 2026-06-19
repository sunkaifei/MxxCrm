import { requestClient } from '#/api/request';

export const getOrderItemListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/sale/order-item/list', { params });
};
export const getOrderItemInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/order-item/info', { params: { id } });
};
