import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getLogListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/logs/list', { params });
};
export const deleteLogApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/logs/bath_delete', {
    data: { ids },
  });
};
