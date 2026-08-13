import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getInboundListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/inbound/list', { params });
};

export const getInboundInfoApi = async (id: number) => {
  return requestClient.get('/api/system/inbound/info', { params: { id } });
};

export const createInboundApi = async (data: any) => {
  return requestClient.post('/api/system/inbound/save', data);
};

export const updateInboundApi = async (data: any) => {
  const { id, ...rest } = data;
  return requestClient.put('/api/system/inbound/update', rest, { params: { id } });
};

export const deleteInboundApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/inbound/batch_delete', {
    data: { ids },
  });
};

export const auditInboundApi = async (id: number) => {
  return requestClient.post('/api/system/inbound/audit', { id });
};

export const rejectInboundApi = async (id: number) => {
  return requestClient.post('/api/system/inbound/reject', { id });
};