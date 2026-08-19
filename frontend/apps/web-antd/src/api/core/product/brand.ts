import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getBrandListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/product/brand/list', { params });
};
export const getBrandInfoApi = async (id: number) => {
  return requestClient.get('/api/system/product/brand/info', {
    params: { id },
  });
};
export const createBrandApi = async (param: any) => {
  return requestClient.post('/api/system/product/brand/save', param);
};
export const updateBrandApi = async (param: any) => {
  return requestClient.put('/api/system/product/brand/update', param);
};
export const deleteBrandApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/product/brand/batch_delete', {
    data: { ids },
  });
};

// ========== 下拉选择 ==========

export const getAllBrandsApi = async () => {
  return requestClient.get('/api/system/product/brand/all');
};
