import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getStockLogListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/inventory/log', { params });
};