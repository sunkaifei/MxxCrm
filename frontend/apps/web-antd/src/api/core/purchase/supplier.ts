import { requestClient } from '#/api/request';

export const getSupplierListApi = async (params?: PageParams) => {
  return requestClient.get('/api/purchase/supplier/list', { params });
};
export const getSupplierInfoApi = async (id: number) => {
  return requestClient.get('/api/purchase/supplier/info', { params: { id } });
};
export const createSupplierApi = async (param: any) => {
  return requestClient.post('/api/purchase/supplier/save', param);
};
export const updateSupplierApi = async (param: any) => {
  return requestClient.put('/api/purchase/supplier/update', param);
};
export const deleteSupplierApi = async (ids: number[]) => {
  return requestClient.delete('/api/purchase/supplier/bath_delete', {
    data: { ids },
  });
};
