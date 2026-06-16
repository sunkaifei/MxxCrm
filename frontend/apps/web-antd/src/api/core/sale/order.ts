import { requestClient } from '#/api/request';

export const getOrderListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/sale/order/list', { params });
};
export const getOrderInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/order/info', { params: { id } });
};
export const createOrderApi = async (param: any) => {
  return requestClient.post('/api/system/sale/order/save', param);
};
export const updateOrderApi = async (param: any) => {
  return requestClient.put('/api/system/sale/order/update', param);
};
export const deleteOrderApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/sale/order/batchDelete', {
    data: { ids },
  });
};
