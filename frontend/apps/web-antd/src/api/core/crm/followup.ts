import { requestClient } from '#/api/request';

export const getFollowupListApi = async (params?: PageParams) => {
  return requestClient.get('/api/crm/followup/list', { params });
};
export const getFollowupInfoApi = async (id: number) => {
  return requestClient.get('/api/crm/followup/info', { params: { id } });
};
export const deleteFollowupApi = async (ids: number[]) => {
  return requestClient.delete('/api/crm/followup/bath_delete', {
    data: { ids },
  });
};
