import { requestClient } from '#/api/request';

export const getLeadListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/lead/list', { params });
};
export const getLeadInfoApi = async (id: number) => {
  return requestClient.get('/api/system/lead/info', { params: { id } });
};
export const createLeadApi = async (param: any) => {
  return requestClient.post('/api/system/lead/save', param);
};
export const updateLeadApi = async (param: any) => {
  return requestClient.put('/api/system/lead/update', param);
};
export const deleteLeadApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/lead/bath_delete', { data: { ids } });
};
export const updateLeadStatusApi = async (id: number, status: string) => {
  return requestClient.put('/api/system/lead/update-status', { id, status });
};
export const addLeadToPoolApi = async (id: number) => {
  return requestClient.put('/api/system/lead/add-to-pool', { id });
};