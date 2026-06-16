import { requestClient } from '#/api/request';

export const getOpportunityListApi = async (params?: PageParams) => {
  return requestClient.get('/api/crm/opportunity/list', { params });
};
export const getOpportunityInfoApi = async (id: number) => {
  return requestClient.get('/api/crm/opportunity/info', { params: { id } });
};
export const createOpportunityApi = async (param: any) => {
  return requestClient.post('/api/crm/opportunity/save', param);
};
export const updateOpportunityApi = async (param: any) => {
  return requestClient.put('/api/crm/opportunity/update', param);
};
export const deleteOpportunityApi = async (ids: number[]) => {
  return requestClient.delete('/api/crm/opportunity/bath_delete', {
    data: { ids },
  });
};
