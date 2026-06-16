import { requestClient } from '#/api/request';

export const getFileListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/attachment/file/list', { params });
};
export const getFileInfoApi = async (id: number) => {
  return requestClient.get('/api/system/attachment/file/info', {
    params: { id },
  });
};
export const createFileApi = async (param: any) => {
  return requestClient.post('/api/system/attachment/file/add', param);
};
export const updateFileApi = async (id: number, param: any) => {
  return requestClient.put(`/api/system/attachment/file/update/${id}`, param);
};
export const deleteFileApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/attachment/file/delete', {
    data: { ids },
  });
};
