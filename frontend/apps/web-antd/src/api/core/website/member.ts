import { requestClient } from '#/api/request';

export interface MemberListParams {
  page?: number;
  pageSize?: number;
  keywords?: string;
  status?: number;
  memberLevel?: number;
}

export interface MemberVO {
  id: number;
  username?: string;
  realName?: string;
  phone?: string;
  email?: string;
  avatar?: string;
  gender?: number;
  status?: number;
  memberLevel?: number;
  totalPoints?: number;
  totalSpent?: number;
  orderCount?: number;
  lastLoginTime?: string;
  registerSource?: string;
  remark?: string;
  createTime?: string;
}

export interface MemberSaveDTO {
  id?: number;
  username?: string;
  password?: string;
  realName?: string;
  phone?: string;
  email?: string;
  avatar?: string;
  gender?: number;
  status?: number;
  memberLevel?: number;
  remark?: string;
}

export const memberApi = {
  list: (params: MemberListParams) =>
    requestClient.get('/api/system/website_user/list', { params }),

  detail: (id: number) =>
    requestClient.get(`/api/system/website_user/detail/${id}`),

  add: (data: MemberSaveDTO) =>
    requestClient.post('/api/system/website_user/create', data),

  update: (id: number, data: MemberSaveDTO) =>
    requestClient.put(`/api/system/website_user/update/${id}`, data),

  batchDelete: (ids: number[]) =>
    requestClient.delete('/api/system/website_user/batch_delete', { data: { ids } }),
};