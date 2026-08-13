import type { StatsTimeParams } from './contract';

import { requestClient } from '#/api/request';

export const getCustomerTypeStatsApi = async (params?: StatsTimeParams) => {
  return requestClient.get('/api/system/statistics/customer/type', { params });
};

export const getCustomerSourceStatsApi = async (params?: StatsTimeParams) => {
  return requestClient.get('/api/system/statistics/customer/source', { params });
};

export const getCustomerIndustryStatsApi = async (params?: StatsTimeParams) => {
  return requestClient.get('/api/system/statistics/customer/industry', { params });
};

export const getCustomerFunnelApi = async (params?: StatsTimeParams) => {
  return requestClient.get('/api/system/statistics/customer/funnel', { params });
};
