import { requestClient } from '#/api/request';

export const getPostListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/post/list', { params });
};
export const getPostOptionsApi = async () => {
  return requestClient.get('/api/system/post/options');
};
export const getPostInfoApi = async (id: number) => {
  return requestClient.get('/api/system/post/detail', { params: { id } });
};
export const createPostApi = async (param: any) => {
  return requestClient.post('/api/system/post/save', param);
};
export const updatePostApi = async (param: any) => {
  return requestClient.put('/api/system/post/update', param);
};
export const deletePostApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/post/bath_delete', {
    data: { ids },
  });
};
