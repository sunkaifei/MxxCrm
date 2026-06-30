import { requestClient } from '#/api/request';

// 注：函数名加 Attachment 前缀，避免与 product/category 的同名导出在 #/api 聚合时产生
// "conflicting star exports" 错误（该错误会导致整个 #/api 模块加载失败、菜单导航不可用）。

export const getAttachmentCategoryTreeApi = async (params?: any) => {
  return requestClient.get('/api/system/attachment/category/tree', { params });
};

export const getAttachmentCategoryListApi = async (params?: any) => {
  return requestClient.get('/api/system/attachment/category/list', { params });
};

export const getAttachmentCategoryDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/attachment/category/detail/${id}`);
};

export const createAttachmentCategoryApi = async (data: any) => {
  return requestClient.post('/api/system/attachment/category/add', data);
};

export const updateAttachmentCategoryApi = async (id: number, data: any) => {
  return requestClient.put(`/api/system/attachment/category/update/${id}`, data);
};

export const deleteAttachmentCategoryApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/attachment/category/batch_delete', {
    data: { ids },
  });
};
