import { requestClient } from '#/api/request';

/**
 * 获取系统记录列表
 */
export const getSystemRecordListApi = async (params: any) => {
  return requestClient.get('/api/system/record/list', { params });
};

/**
 * 获取系统记录详情
 */
export const getSystemRecordInfoApi = async (id: number) => {
  return requestClient.get(`/api/system/record/${id}`);
};

/**
 * 删除系统记录
 */
export const deleteSystemRecordApi = async (id: number) => {
  return requestClient.delete(`/api/system/record/${id}`);
};

export const getRecordListApi = getSystemRecordListApi;
export const deleteRecordApi = deleteSystemRecordApi;
