import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getInventoryListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/inventory/list', { params });
};
export const getInventoryInfoApi = async (id: number) => {
  return requestClient.get('/api/system/inventory/info', { params: { id } });
};

/** 库存回收站：软删除（仅零库存记录） */
export const recycleDeleteInventoryApi = async (ids: number[]) => {
  // 后端 BathDeleteIdRequest 的 ids 为字符串数组
  return requestClient.delete('/api/system/inventory/recycle', {
    data: { ids: ids.map((id) => String(id)) },
  });
};

/** 库存回收站：恢复 */
export const restoreInventoryApi = async (id: number) => {
  return requestClient.put(`/api/system/inventory/restore/${id}`);
};

/** 库存回收站：彻底删除（仅管理层） */
export const purgeInventoryApi = async (id: number) => {
  return requestClient.delete(`/api/system/inventory/purge/${id}`);
};
