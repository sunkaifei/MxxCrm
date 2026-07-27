import { requestClient } from '#/api/request';

export interface ArticleListParams {
  page?: number;
  pageSize?: number;
  status?: number;
  keyword?: string;
  categoryId?: number;
}

export interface ArticleSaveDTO {
  id?: number;
  categoryId?: number;
  title?: string;
  shortTitle?: string;
  titleImage?: string;
  author?: string;
  shortUrl?: string;
  description?: string;
  content?: string;
  istop?: number;
  isrecommend?: number;
  status?: number;
}

export interface ArticleVO {
  id: string;
  categoryId?: number;
  title?: string;
  shortTitle?: string;
  titleImage?: string;
  author?: string;
  shortUrl?: string;
  description?: string;
  content?: string;
  istop?: number;
  isrecommend?: number;
  status?: number;
  createTime?: string;
}

export const articleApi = {
  list: (params: ArticleListParams) =>
    requestClient.get('/api/system/article/list', { params }),
  detail: (id: number) =>
    requestClient.get(`/api/system/article/detail/${id}`),
  save: (data: ArticleSaveDTO) =>
    requestClient.post('/api/system/article/save', data),
  update: (id: number, data: ArticleSaveDTO) =>
    requestClient.put(`/api/system/article/update/${id}`, data),
  delete: (ids: number[]) =>
    requestClient.delete('/api/system/article/batch_delete', { data: { ids } }),
};
