import { requestClient } from '#/api/request';

export const getStockPlanListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/purchase/stock_plan/list', { params });
};
export const getStockPlanInfoApi = async (id: number) => {
  return requestClient.get('/api/system/purchase/stock_plan/info', { params: { id } });
};
export const createStockPlanApi = async (param: any) => {
  return requestClient.post('/api/system/purchase/stock_plan/save', param);
};
export const updateStockPlanApi = async (param: any) => {
  return requestClient.put(`/api/system/purchase/stock_plan/update/${param.id}`, param);
};
export const deleteStockPlanApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/purchase/stock_plan/bath_delete', {
    data: { ids },
  });
};

// ========== 业务操作 ==========

export const convertToRequisitionApi = async (id: number) => {
  return requestClient.put(`/api/system/purchase/stock_plan/generate_pr/${id}`);
};
export const recalculateApi = async (id: number) => {
  return requestClient.put(`/api/system/purchase/stock_plan/recalculate/${id}`);
};

// ========== 预警 ==========

export const getWarningListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/purchase/stock_plan/warning_list', { params });
};
