import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getProductionOrderListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/production/order/list', { params });
};
export const getProductionOrderInfoApi = async (id: number) => {
  return requestClient.get('/api/system/production/order/info', {
    params: { id },
  });
};
export const createProductionOrderApi = async (param: any) => {
  return requestClient.post('/api/system/production/order/save', param);
};
export const updateProductionOrderApi = async (param: any) => {
  return requestClient.put('/api/system/production/order/update', param);
};
export const deleteProductionOrderApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/production/order/bath_delete', {
    data: { ids },
  });
};

// ========== 状态流转 ==========

export const releaseProductionOrderApi = async (id: number) => {
  return requestClient.post('/api/system/production/order/release', { id });
};
export const startProductionOrderApi = async (id: number) => {
  return requestClient.post('/api/system/production/order/start', { id });
};
export const completeProductionOrderApi = async (id: number) => {
  return requestClient.post('/api/system/production/order/complete', { id });
};
export const inboundProductionOrderApi = async (id: number) => {
  return requestClient.post('/api/system/production/order/inbound', { id });
};
export const closeProductionOrderApi = async (id: number) => {
  return requestClient.post('/api/system/production/order/close', { id });
};
