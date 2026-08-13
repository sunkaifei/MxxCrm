import type { StatsTimeParams } from './contract';

import { requestClient } from '#/api/request';

export const getEmployeeCustomerCountApi = async (params?: StatsTimeParams & { department_id?: number }) => {
  return requestClient.get('/api/system/statistics/employee/customer-count', { params });
};

export const getEmployeeFollowUpApi = async (params?: StatsTimeParams & { department_id?: number }) => {
  return requestClient.get('/api/system/statistics/employee/follow-up', { params });
};

export const getEmployeeConversionApi = async (params?: StatsTimeParams & { department_id?: number }) => {
  return requestClient.get('/api/system/statistics/employee/conversion', { params });
};
