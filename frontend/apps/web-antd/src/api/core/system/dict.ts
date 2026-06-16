import { requestClient } from '#/api/request';

/**
 * 获取字典列表
 */
export const getDictListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/dict/list', { params });
};

/**
 * 获取字典详情
 */
export const getDictInfoApi = async (id: number) => {
  return requestClient.get(`/api/system/dict/${id}`);
};

/**
 * 新增字典
 */
export const createDictApi = async (param: any) => {
  return requestClient.post('/api/system/dict/add', param);
};

/**
 * 修改字典
 */
export const updateDictApi = async (id: number, param: any) => {
  return requestClient.put(`/api/system/dict/update/${id}`, param);
};

/**
 * 删除字典
 */
export const deleteDictApi = async (id: number) => {
  return requestClient.delete(`/api/system/dict/delete/${id}`);
};
