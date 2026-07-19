import { requestClient } from '#/api/request';

export const getFollowupListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/followup/list', { params });
};
export const getFollowupInfoApi = async (id: number) => {
  return requestClient.get('/api/system/followup/info', { params: { id } });
};
export const createFollowupApi = async (param: any) => {
  return requestClient.post('/api/system/followup/save', param);
};
export const updateFollowupApi = async (param: any) => {
  return requestClient.put('/api/system/followup/update', param);
};
export const deleteFollowupApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/followup/bath_delete', {
    data: { ids },
  });
};