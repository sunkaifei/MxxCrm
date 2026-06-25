import { requestClient } from '#/api/request';

export const getPerformanceTargetApi = async (params?: { employee_id?: number; year?: number; month?: number }) => {
  return requestClient.get('/api/system/statistics/performance/target', { params });
};

export const savePerformanceTargetApi = async (param: { targets: any[] }) => {
  return requestClient.post('/api/system/statistics/performance/target/save', param);
};

export const getMonthlyPerformanceApi = async (params?: { year?: number; department_id?: number }) => {
  return requestClient.get('/api/system/statistics/performance/monthly', { params });
};

export const getPerformanceRankingApi = async (params?: { year?: number; month?: number; order_by?: string; department_id?: number }) => {
  return requestClient.get('/api/system/statistics/performance/ranking', { params });
};