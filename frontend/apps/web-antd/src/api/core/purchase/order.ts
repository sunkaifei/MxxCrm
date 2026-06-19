import { requestClient } from '#/api/request';

export const getPurchaseOrderListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/purchase/order/list', { params });
};
export const getPurchaseOrderInfoApi = async (id: number) => {
  return requestClient.get('/api/system/purchase/order/info', { params: { id } });
};
export const createPurchaseOrderApi = async (param: any) => {
  return requestClient.post('/api/system/purchase/order/save', param);
};
export const updatePurchaseOrderApi = async (param: any) => {
  return requestClient.put('/api/system/purchase/order/update', param);
};
export const deletePurchaseOrderApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/purchase/order/bath_delete', {
    data: { ids },
  });
};
