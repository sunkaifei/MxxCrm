import { requestClient } from '#/api/request';

export const getCustomerTypeStatsApi = async (params?: { year?: number; month?: number }) => {
  return requestClient.get('/api/system/statistics/customer/type', { params });
};

export const getCustomerSourceStatsApi = async (params?: { year?: number; month?: number }) => {
  return requestClient.get('/api/system/statistics/customer/source', { params });
};

export const getCustomerIndustryStatsApi = async (params?: { year?: number; month?: number }) => {
  return requestClient.get('/api/system/statistics/customer/industry', { params });
};

export const getCustomerFunnelApi = async (params?: { year?: number; month?: number }) => {
  return requestClient.get('/api/system/statistics/customer/funnel', { params });
};