import { requestClient } from '#/api/request';

export const getPurchaseReportSummaryApi = async (params?: {
  endDate?: string;
  startDate?: string;
}) => {
  return requestClient.get('/api/system/purchase/report/summary', { params });
};

export const getPurchaseReportBySupplierApi = async (params?: {
  endDate?: string;
  startDate?: string;
}) => {
  return requestClient.get('/api/system/purchase/report/by_supplier', {
    params,
  });
};

export const getPurchaseReportByProductApi = async (params?: {
  endDate?: string;
  startDate?: string;
}) => {
  return requestClient.get('/api/system/purchase/report/by_product', {
    params,
  });
};

export const getPurchaseReportByDepartmentApi = async (params?: {
  endDate?: string;
  startDate?: string;
}) => {
  return requestClient.get('/api/system/purchase/report/by_department', {
    params,
  });
};

export const getPurchaseReportByBrandApi = async (params?: {
  endDate?: string;
  startDate?: string;
}) => {
  return requestClient.get('/api/system/purchase/report/by_brand', { params });
};
