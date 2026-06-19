import { requestClient } from '#/api/request';

export const getCategoryListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/category/list', { params });
};
export const getCategoryInfoApi = async (id: number) => {
  return requestClient.get('/api/system/category/info', { params: { id } });
};
export const createCategoryApi = async (param: any) => {
  return requestClient.post('/api/system/category/save', param);
};
export const updateCategoryApi = async (param: any) => {
  return requestClient.put('/api/system/category/update', param);
};
export const deleteCategoryApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/category/bath_delete', {
    data: { ids },
  });
};
