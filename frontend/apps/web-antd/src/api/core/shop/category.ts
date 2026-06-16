import { requestClient } from '#/api/request';

export interface CategoryListParams {
  page?: number;
  pageSize?: number;
}

export interface CategorySaveDTO {
  id?: number;
  parentId: number;
  name: string;
  icon?: string;
  sortOrder?: number;
  level?: number;
  isShow?: number;
}

export interface CategoryVO {
  id: number;
  parentId: number;
  name: string;
  icon?: string;
  sortOrder: number;
  level: number;
  isShow: number;
  children?: CategoryVO[];
  createTime?: string;
  updateTime?: string;
}

export const categoryApi = {
  tree: () => requestClient.get('/api/system/category/tree'),
  save: (data: CategorySaveDTO) =>
    requestClient.post('/api/system/category/save', data),
  update: (data: CategorySaveDTO) =>
    requestClient.put('/api/system/category/update', data),
  delete: (params: { id: number }) =>
    requestClient.delete('/api/system/category/delete', { params }),
};
