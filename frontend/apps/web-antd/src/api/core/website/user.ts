import { requestClient } from '#/api/request';

export interface WebsiteUserListParams {
  page?: number;
  pageSize?: number;
  username?: string;
  phone?: string;
  status?: number;
}

export interface WebsiteUserVO {
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

export interface WebsiteUserSaveDTO {
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

export interface ResetPasswordParams {
  newPassword: string;
}

export interface UpdateStatusParams {
  status: number;
}

export const userApi = {
  list: (params: WebsiteUserListParams) =>
    requestClient.get('/api/system/website_user/list', { params }),

  detail: (id: number) =>
    requestClient.get(`/api/system/website_user/detail/${id}`),

  create: (data: WebsiteUserSaveDTO) =>
    requestClient.post('/api/system/website_user/create', data),

  update: (id: number, data: WebsiteUserSaveDTO) =>
    requestClient.put(`/api/system/website_user/update/${id}`, data),

  resetPassword: (id: number, data: ResetPasswordParams) =>
    requestClient.put(`/api/system/website_user/reset_password/${id}`, data),

  updateStatus: (id: number, status: number) =>
    requestClient.put(`/api/system/website_user/status/${id}`, { status }),

  batchDelete: (ids: number[]) =>
    requestClient.delete('/api/system/website_user/batch_delete', {
      data: { ids },
    }),
};
