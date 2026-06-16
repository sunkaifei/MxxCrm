import { requestClient } from '#/api/request';

export const getDeptListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/dept/list', { params });
};
export const getDeptInfoApi = async (id: number) => {
  return requestClient.get('/api/system/dept/detail', { params: { id } });
};
export const createDeptApi = async (param: any) => {
  return requestClient.post('/api/system/dept/save', param);
};
export const updateDeptApi = async (param: any) => {
  return requestClient.put('/api/system/dept/update', param);
};
export const deleteDeptApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/dept/bath_delete', {
    data: { ids },
  });
};
