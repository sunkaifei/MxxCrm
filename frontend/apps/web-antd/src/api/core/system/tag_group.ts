import { requestClient } from '#/api/request';

export const getTagGroupListApi = async () => {
  return requestClient.get('/api/system/tag/group/list');
};

export const getTagGroupDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/tag/group/detail/${id}`);
};

export const createTagGroupApi = async (param: any) => {
  return requestClient.post('/api/system/tag/group', param);
};

export const updateTagGroupApi = async (param: any) => {
  return requestClient.put('/api/system/tag/group', param);
};

export const deleteTagGroupApi = async (id: number) => {
  return requestClient.delete(`/api/system/tag/group/${id}`);
};

export const batchDeleteTagGroupApi = async (params: any) => {
  return requestClient.delete('/api/system/tag/group/batch_delete', { data: params });
};