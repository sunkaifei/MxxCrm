import { requestClient } from '#/api/request';

export const getCategoryListApi = async (params?: PageParams) => {
  return requestClient.get('/api/product/category/list', { params });
};
export const getCategoryInfoApi = async (id: number) => {
  return requestClient.get('/api/product/category/info', { params: { id } });
};
export const createCategoryApi = async (param: any) => {
  return requestClient.post('/api/product/category/save', param);
};
export const updateCategoryApi = async (param: any) => {
  return requestClient.put('/api/product/category/update', param);
};
export const deleteCategoryApi = async (ids: number[]) => {
  return requestClient.delete('/api/product/category/bath_delete', {
    data: { ids },
  });
};
