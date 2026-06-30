import { requestClient } from '#/api/request';

export const getSalaryListApi = async (params?: any) => {
  return requestClient.get('/api/system/finance/salary/list', { params });
};

export const getSalaryDetailApi = async (id: number) => {
  return requestClient.get('/api/system/finance/salary/detail', {
    params: { id },
  });
};

export const calculateSalaryApi = async (data: {
  year: number;
  month: number;
}) => {
  return requestClient.post('/api/system/finance/salary/calculate', data);
};

export const updateSalaryApi = async (data: any) => {
  return requestClient.post('/api/system/finance/salary/update', data);
};

export const approveSalaryApi = async (id: number) => {
  return requestClient.post('/api/system/finance/salary/approve', { id });
};

export const batchApproveSalaryApi = async (ids: number[]) => {
  return requestClient.post('/api/system/finance/salary/batch-approve', {
    ids,
  });
};

export const paySalaryApi = async (id: number) => {
  return requestClient.post('/api/system/finance/salary/pay', { id });
};

export const batchPaySalaryApi = async (ids: number[]) => {
  return requestClient.post('/api/system/finance/salary/batch-pay', { ids });
};

export const getSalarySummaryApi = async (params: {
  year: number;
  month: number;
}) => {
  return requestClient.get('/api/system/finance/salary/summary', { params });
};
