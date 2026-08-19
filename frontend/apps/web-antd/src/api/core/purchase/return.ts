import { requestClient } from '#/api/request';

export const getPurchaseReturnListApi = async (
  params?: Record<string, any>,
) => {
  return requestClient.get('/api/system/purchase/return/list', { params });
};

export const getPurchaseReturnInfoApi = async (id: number) => {
  return requestClient.get('/api/system/purchase/return/info', {
    params: { id },
  });
};

export const createPurchaseReturnApi = async (param: any) => {
  return requestClient.post('/api/system/purchase/return/save', param);
};

export const updatePurchaseReturnApi = async (param: any) => {
  return requestClient.put('/api/system/purchase/return/update', param);
};

export const deletePurchaseReturnApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/purchase/return/bath_delete', {
    data: { ids },
  });
};
