import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getRecycleListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/recycle/list', { params });
};

export const restoreRecycleApi = async (module: string, id: number) => {
  return requestClient.post('/api/system/recycle/restore', { module, id });
};

export const purgeRecycleApi = async (module: string, id: number) => {
  return requestClient.post('/api/system/recycle/purge', { module, id });
};
