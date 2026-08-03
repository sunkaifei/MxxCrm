import { requestClient } from '#/api/request';

export interface ArticleTagListParams {
  page?: number;
  pageSize?: number;
  keywords?: string;
  status?: number;
}

export interface ArticleTagVO {
  id: number;
  name: string;
  slug?: string;
  color?: string;
  sort: number;
  status: number;
  articleCount?: number;
  createTime?: string;
}

export interface ArticleTagSaveDTO {
  id?: number;
  name?: string;
  slug?: string;
  color?: string;
  sort?: number;
  status?: number;
}

export const articleTagApi = {
  /** 分页列表 */
  list: (params: ArticleTagListParams) =>
    requestClient.get('/api/system/article/tag/list', { params }),

  /** 详情 */
  detail: (id: number) =>
    requestClient.get(`/api/system/article/tag/detail/${id}`),

  /** 新增 */
  add: (data: ArticleTagSaveDTO) =>
    requestClient.post('/api/system/article/tag/add', data),

  /** 修改 */
  update: (id: number, data: ArticleTagSaveDTO) =>
    requestClient.put(`/api/system/article/tag/update/${id}`, data),

  /** 删除 */
  delete: (ids: number[]) =>
    requestClient.delete('/api/system/article/tag/batch_delete', { data: { ids } }),

  /** 获取所有启用的标签（用于下拉选择） */
  all: () =>
    requestClient.get('/api/system/article/tag/all'),
};