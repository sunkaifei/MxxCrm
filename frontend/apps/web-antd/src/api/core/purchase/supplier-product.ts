import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getSupplierProductListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/purchase/supplier_product/list', {
    params,
  });
};
export const createSupplierProductApi = async (param: any) => {
  return requestClient.post(
    '/api/system/purchase/supplier_product/save',
    param,
  );
};
export const updateSupplierProductApi = async (param: any) => {
  return requestClient.put(
    '/api/system/purchase/supplier_product/update',
    param,
  );
};
export const deleteSupplierProductApi = async (ids: number[]) => {
  return requestClient.delete(
    '/api/system/purchase/supplier_product/bath_delete',
    {
      data: { ids },
    },
  );
};
