import type { StatsTimeParams } from './contract';

import { requestClient } from '#/api/request';

export const getPaymentCompletionApi = async (params?: StatsTimeParams) => {
  return requestClient.get('/api/system/statistics/payment/completion', { params });
};

export const getPaymentMonthlyTrendApi = async (params?: { year?: number }) => {
  return requestClient.get('/api/system/statistics/payment/monthly-trend', { params });
};

export const getPaymentStatusAnalysisApi = async (params?: StatsTimeParams) => {
  return requestClient.get('/api/system/statistics/payment/status-analysis', { params });
};

export const getPaymentRankingApi = async (params?: StatsTimeParams & { order_by?: string; limit?: number }) => {
  return requestClient.get('/api/system/statistics/payment/ranking', { params });
};
