import { requestClient } from '#/api/request';

export const getTagListApi = async (params?: any) => {
  return requestClient.get('/api/system/tag/list', { params });
};

export const getTagDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/tag/detail/${id}`);
};

export const createTagApi = async (param: any) => {
  return requestClient.post('/api/system/tag/add', param);
};

export const updateTagApi = async (param: any) => {
  return requestClient.put('/api/system/tag/update', param);
};

export const deleteTagApi = async (id: number) => {
  return requestClient.delete(`/api/system/tag/delete/${id}`);
};

export const batchDeleteTagApi = async (params: any) => {
  return requestClient.delete('/api/system/tag/batch_delete', { data: params });
};

export const updateTagStatusApi = async (params: { id: number; status: number }) => {
  return requestClient.put('/api/system/tag/status', params);
};

export const getTagStatisticsApi = async () => {
  return requestClient.get('/api/system/tag/statistics');
};

export const getAllTagsApi = async () => {
  return requestClient.get('/api/system/tag/all');
};

export const getTagsByGroupApi = async (groupId: number) => {
  return requestClient.get(`/api/system/tag/group/${groupId}`);
};

export const moveTagsToGroupApi = async (param: any) => {
  return requestClient.post('/api/system/tag/move-to-group', param);
};

export const tagSuggestApi = async (keyword: string) => {
  return requestClient.get('/api/system/tag/suggest', { params: { keyword } });
};

export const addTagsToEntityApi = async (param: any) => {
  return requestClient.post('/api/system/tag/entity/add', param);
};

export const removeTagsFromEntityApi = async (param: any) => {
  return requestClient.post('/api/system/tag/entity/remove', param);
};

export const getTagsByEntityApi = async (entityType: string, entityId: number) => {
  return requestClient.get(`/api/system/tag/entity/${entityType}/${entityId}`);
};

export const batchTagEntityApi = async (param: any) => {
  return requestClient.post('/api/system/tag/entity/batch', param);
};