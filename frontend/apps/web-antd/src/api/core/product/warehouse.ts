import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

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
  return requestClient.delete('/api/system/warehouse/batch_delete', {
    data: { ids },
  });
};

/** 回收站恢复（负责人/管理层） */
export const restoreWarehouseApi = async (id: number) => {
  return requestClient.put(`/api/system/warehouse/restore/${id}`);
};

/** 彻底删除（仅管理层/超管，回收站内） */
export const purgeWarehouseApi = async (id: number) => {
  return requestClient.delete(`/api/system/warehouse/purge/${id}`);
};
