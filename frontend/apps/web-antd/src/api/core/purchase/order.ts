import { requestClient } from '#/api/request';

export const getPurchaseOrderListApi = async (params?: PageParams) => {
  return requestClient.get('/api/purchase/order/list', { params });
};
export const getPurchaseOrderInfoApi = async (id: number) => {
  return requestClient.get('/api/purchase/order/info', { params: { id } });
};
export const createPurchaseOrderApi = async (param: any) => {
  return requestClient.post('/api/purchase/order/save', param);
};
export const updatePurchaseOrderApi = async (param: any) => {
  return requestClient.put('/api/purchase/order/update', param);
};
export const deletePurchaseOrderApi = async (ids: number[]) => {
  return requestClient.delete('/api/purchase/order/bath_delete', {
    data: { ids },
  });
};
