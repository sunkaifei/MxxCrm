import { requestClient } from '#/api/request';

export const getInvoiceListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/sale/invoice/list', { params });
};
export const getInvoiceInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/invoice/info', { params: { id } });
};
export const createInvoiceApi = async (param: any) => {
  return requestClient.post('/api/system/sale/invoice/save', param);
};
export const updateInvoiceApi = async (param: any) => {
  return requestClient.put('/api/system/sale/invoice/update', param);
};
export const deleteInvoiceApi = async (ids: number[]) => {
  return requestClient.post('/api/system/sale/invoice/batch-delete', { ids });
};