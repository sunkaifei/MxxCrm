import { requestClient } from '#/api/request';

export const getPurchaseReportSummaryApi = async (params?: { startDate?: string; endDate?: string }) => {
  return requestClient.get('/api/system/purchase/report/summary', { params });
};

export const getPurchaseReportBySupplierApi = async (params?: { startDate?: string; endDate?: string }) => {
  return requestClient.get('/api/system/purchase/report/by_supplier', { params });
};

export const getPurchaseReportByProductApi = async (params?: { startDate?: string; endDate?: string }) => {
  return requestClient.get('/api/system/purchase/report/by_product', { params });
};

export const getPurchaseReportByDepartmentApi = async (params?: { startDate?: string; endDate?: string }) => {
  return requestClient.get('/api/system/purchase/report/by_department', { params });
};

export const getPurchaseReportByBrandApi = async (params?: { startDate?: string; endDate?: string }) => {
  return requestClient.get('/api/system/purchase/report/by_brand', { params });
};