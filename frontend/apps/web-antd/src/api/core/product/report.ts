import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getStockReportApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/report/stock', { params });
};

export const getInboundSummaryApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/report/inbound_summary', { params });
};

export const getOutboundSummaryApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/report/outbound_summary', { params });
};

export const getTurnoverReportApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/report/turnover', { params });
};