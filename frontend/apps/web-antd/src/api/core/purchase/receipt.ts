import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getReceiptListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/purchase/receipt/list', { params });
};
export const getReceiptInfoApi = async (id: number) => {
  return requestClient.get('/api/system/purchase/receipt/info', {
    params: { id },
  });
};
export const createReceiptApi = async (param: any) => {
  return requestClient.post('/api/system/purchase/receipt/save', param);
};
export const updateReceiptApi = async (param: any) => {
  return requestClient.post('/api/system/purchase/receipt/save', param);
};
export const deleteReceiptApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/purchase/receipt/bath_delete', {
    data: { ids },
  });
};
export const toInboundApi = async (id: number) => {
  return requestClient.post('/api/system/purchase/receipt/to_inbound', { id });
};
