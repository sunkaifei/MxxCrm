import { requestClient } from '#/api/request';

export const getLeadListApi = async (params?: PageParams) => {
  return requestClient.get('/api/crm/lead/list', { params });
};
export const getLeadInfoApi = async (id: number) => {
  return requestClient.get('/api/crm/lead/info', { params: { id } });
};
export const createLeadApi = async (param: any) => {
  return requestClient.post('/api/crm/lead/save', param);
};
export const updateLeadApi = async (param: any) => {
  return requestClient.put('/api/crm/lead/update', param);
};
export const deleteLeadApi = async (ids: number[]) => {
  return requestClient.delete('/api/crm/lead/bath_delete', { data: { ids } });
};
