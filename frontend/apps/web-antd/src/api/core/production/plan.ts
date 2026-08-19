import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getProductionPlanListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/production/plan/list', { params });
};
export const getProductionPlanInfoApi = async (id: number) => {
  return requestClient.get('/api/system/production/plan/info', {
    params: { id },
  });
};
export const createProductionPlanApi = async (param: any) => {
  return requestClient.post('/api/system/production/plan/save', param);
};
export const updateProductionPlanApi = async (param: any) => {
  return requestClient.put('/api/system/production/plan/update', param);
};
export const deleteProductionPlanApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/production/plan/bath_delete', {
    data: { ids },
  });
};

// ========== 业务操作 ==========

export const convertToProductionOrderApi = async (id: number) => {
  return requestClient.post('/api/system/production/plan/generate_mo', { id });
};
