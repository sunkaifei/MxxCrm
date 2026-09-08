import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getOutboundListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/outbound/list', { params });
};

export const getOutboundInfoApi = async (id: number) => {
  return requestClient.get('/api/system/outbound/info', { params: { id } });
};

export const createOutboundApi = async (data: any) => {
  return requestClient.post('/api/system/outbound/save', data);
};

export const updateOutboundApi = async (data: any) => {
  const { id, ...rest } = data;
  return requestClient.put('/api/system/outbound/update', rest, {
    params: { id },
  });
};

export const deleteOutboundApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/outbound/batch_delete', {
    data: { ids },
  });
};

export const submitOutboundApi = async (
  id: number,
  ccUserIds: number[] = [],
  ccReason?: string,
) => {
  return requestClient.put(`/api/system/outbound/submit/${id}`, {
    ccUserIds,
    ccReason,
  });
};

export const auditOutboundApi = async (id: number, comment?: string) => {
  return requestClient.post('/api/system/outbound/audit', { id, comment });
};

export const rejectOutboundApi = async (id: number, comment?: string) => {
  return requestClient.post('/api/system/outbound/reject', { id, comment });
};

export const withdrawOutboundApi = async (id: number) => {
  return requestClient.post('/api/system/outbound/withdraw', { id });
};

/** 回收站恢复（制单人或管理层） */
export const restoreOutboundApi = async (id: number) => {
  return requestClient.put(`/api/system/outbound/restore/${id}`);
};

/** 回收站彻底删除（仅管理层/超管） */
export const purgeOutboundApi = async (id: number) => {
  return requestClient.delete(`/api/system/outbound/purge/${id}`);
};
