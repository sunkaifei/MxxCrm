import { requestClient } from '#/api/request';

export const getWarehouseListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/warehouse/list', { params });
};
export const getWarehouseInfoApi = async (id: number) => {
  return requestClient.get('/api/system/warehouse/info', { params: { id } });
};
export const createWarehouseApi = async (param: any) => {
  return requestClient.post('/api/system/warehouse/save', param);
};
export const updateWarehouseApi = async (param: any) => {
  return requestClient.put('/api/system/warehouse/update', param);
};
export const deleteWarehouseApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/warehouse/bath_delete', {
    data: { ids },
  });
};