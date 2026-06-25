import { requestClient } from '#/api/request';

export const getContractRankingApi = async (params?: { year?: number; month?: number; order_by?: string; order_type?: string; limit?: number }) => {
  return requestClient.get('/api/system/statistics/contract/ranking', { params });
};

export const getContractTypeDistributionApi = async (params?: { year?: number; month?: number }) => {
  return requestClient.get('/api/system/statistics/contract/type-distribution', { params });
};

export const getContractStatusAnalysisApi = async (params?: { year?: number; month?: number }) => {
  return requestClient.get('/api/system/statistics/contract/status-analysis', { params });
};