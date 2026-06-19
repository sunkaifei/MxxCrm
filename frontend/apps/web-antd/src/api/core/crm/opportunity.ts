import { requestClient } from '#/api/request';

export const getOpportunityListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/opportunity/list', { params });
};
export const getOpportunityInfoApi = async (id: number) => {
  return requestClient.get('/api/system/opportunity/info', { params: { id } });
};
export const createOpportunityApi = async (param: any) => {
  return requestClient.post('/api/system/opportunity/save', param);
};
export const updateOpportunityApi = async (param: any) => {
  return requestClient.put('/api/system/opportunity/update', param);
};
export const deleteOpportunityApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/opportunity/bath_delete', {
    data: { ids },
  });
};
