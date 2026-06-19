import { requestClient } from '#/api/request';

export const getProductListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/product/product/list', { params });
};
export const getProductInfoApi = async (id: number) => {
  return requestClient.get('/api/system/product/product/info', { params: { id } });
};
export const createProductApi = async (param: any) => {
  return requestClient.post('/api/system/product/product/save', param);
};
export const updateProductApi = async (param: any) => {
  return requestClient.put('/api/system/product/product/update', param);
};
export const deleteProductApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/product/product/batchDelete', {
    data: { ids },
  });
};
