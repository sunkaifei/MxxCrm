import { requestClient } from '#/api/request';

export const getLogListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/logs/list', { params });
};
export const getLogInfoApi = async (id: number) => {
  return requestClient.get('/api/system/logs/detail', { params: { id } });
};
export const deleteLogApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/logs/bath_delete', {
    data: { ids },
  });
};
