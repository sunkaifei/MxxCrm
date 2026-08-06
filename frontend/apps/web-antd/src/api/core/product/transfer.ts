import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getTransferListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/inventory/transfer/list', { params });
};

export const getTransferInfoApi = async (id: number) => {
  return requestClient.get('/api/system/inventory/transfer/info', { params: { id } });
};

export const createTransferApi = async (data: any) => {
  return requestClient.post('/api/system/inventory/transfer/save', data);
};

export const transferOutboundApi = async (id: number) => {
  return requestClient.post('/api/system/inventory/transfer/outbound', { params: { id } });
};

export const transferInboundApi = async (id: number) => {
  return requestClient.post('/api/system/inventory/transfer/inbound', { params: { id } });
};

export const deleteTransferApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/inventory/transfer/batch_delete', {
    data: { ids },
  });
};