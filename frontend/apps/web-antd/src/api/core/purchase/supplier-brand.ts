import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getSupplierBrandListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/purchase/supplier/brand/list', {
    params,
  });
};
export const createSupplierBrandApi = async (param: any) => {
  return requestClient.post('/api/system/purchase/supplier/brand/save', param);
};
export const deleteSupplierBrandApi = async (ids: number[]) => {
  return requestClient.delete(
    '/api/system/purchase/supplier/brand/bath_delete',
    {
      data: { ids },
    },
  );
};

// ========== 查询关联 ==========

export const getBrandsBySupplierApi = async (params: {
  supplierId: number;
}) => {
  return requestClient.get(
    '/api/system/purchase/supplier/brand/list_by_supplier',
    { params: { id: params.supplierId } },
  );
};
export const getSuppliersByBrandApi = async (params: { brandId: number }) => {
  return requestClient.get(
    '/api/system/purchase/supplier/brand/list_by_brand',
    { params },
  );
};
