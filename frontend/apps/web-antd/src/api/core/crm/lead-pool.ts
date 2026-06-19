import { requestClient } from '#/api/request';

export const getLeadPoolListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/lead-pool/list', { params });
};
export const getLeadPoolInfoApi = async (id: number) => {
  return requestClient.get('/api/system/lead-pool/info', { params: { id } });
};
export const deleteLeadPoolApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/lead-pool/bath_delete', {
    data: { ids },
  });
};
