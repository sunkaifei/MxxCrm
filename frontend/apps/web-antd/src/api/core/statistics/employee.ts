import { requestClient } from '#/api/request';

export const getEmployeeCustomerCountApi = async (params?: { department_id?: number }) => {
  return requestClient.get('/api/system/statistics/employee/customer-count', { params });
};

export const getEmployeeFollowUpApi = async (params?: { year?: number; month?: number; department_id?: number }) => {
  return requestClient.get('/api/system/statistics/employee/follow-up', { params });
};

export const getEmployeeConversionApi = async (params?: { year?: number; month?: number; department_id?: number }) => {
  return requestClient.get('/api/system/statistics/employee/conversion', { params });
};