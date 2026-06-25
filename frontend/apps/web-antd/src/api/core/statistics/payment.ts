import { requestClient } from '#/api/request';

export const getPaymentCompletionApi = async (params?: { year?: number; month?: number }) => {
  return requestClient.get('/api/system/statistics/payment/completion', { params });
};

export const getPaymentMonthlyTrendApi = async (params?: { year?: number }) => {
  return requestClient.get('/api/system/statistics/payment/monthly-trend', { params });
};

export const getPaymentStatusAnalysisApi = async (params?: { year?: number; month?: number }) => {
  return requestClient.get('/api/system/statistics/payment/status-analysis', { params });
};

export const getPaymentRankingApi = async (params?: { year?: number; month?: number; order_by?: string; limit?: number }) => {
  return requestClient.get('/api/system/statistics/payment/ranking', { params });
};