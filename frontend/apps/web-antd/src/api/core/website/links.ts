import { requestClient } from '#/api/request';

export interface LinkListParams {
  page?: number;
  pageSize?: number;
  status?: number;
  keyword?: string;
  websiteId?: number;
}

export interface LinkSaveDTO {
  id?: number;
  websiteId?: number;
  linkName?: string;
  linkUrl?: string;
  linkLogo?: string;
  linkType?: number;
  status?: number;
  sort?: number;
}

export interface LinkVO {
  id: number;
  websiteId?: number;
  linkName?: string;
  linkUrl?: string;
  linkLogo?: string;
  linkType?: number;
  status?: number;
  sort?: number;
  createTime?: string;
}

export const linksApi = {
  list: (params: LinkListParams) =>
    requestClient.get('/api/system/links/list', { params }),

  detail: (id: number) =>
    requestClient.get(`/api/system/links/detail/${id}`),

  add: (data: LinkSaveDTO) =>
    requestClient.post('/api/system/links/add', data),

  update: (id: number, data: LinkSaveDTO) =>
    requestClient.put(`/api/system/links/update/${id}`, data),

  delete: (ids: number[]) =>
    requestClient.delete('/api/system/links/batch_delete', { data: { ids } }),
};
