import { requestClient } from '#/api/request';

/**
 * 用户管理列表查询参数
 */
export interface UserManageListQuery {
  page?: number;
  pageSize?: number;
  username?: string;
  nickname?: string;
  mobile?: string;
  status?: number;
}

/**
 * 用户管理列表项
 */
export interface UserManageInfo {
  id: string;
  username: string;
  nickname: string;
  avatar: string;
  email: string;
  mobile: string;
  lastLoginTime: string;
  status: string;
  motto: string;
  createTime: string;
}

/**
 * 新增用户参数
 */
export interface UserManageCreateReq {
  username: string;
  nickname: string;
  password?: string;
  email?: string;
  mobile?: string;
  avatar?: string;
  motto?: string;
}

/**
 * 更新用户参数
 */
export interface UserManageUpdateReq {
  id: string;
  username?: string;
  nickname?: string;
  password?: string;
  email?: string;
  mobile?: string;
  avatar?: string;
  motto?: string;
  status?: string;
}

/**
 * 获取用户列表
 */
export async function getUserManageListApi(params: UserManageListQuery) {
  const data = await requestClient.get('/api/system/admin/user/list', {
    params,
  });
  return {
    items: data.list ?? [],
    total: data.total ?? 0,
  };
}

/**
 * 获取用户详情
 */
export async function getUserManageDetailApi(id: string) {
  return requestClient.get(`/api/system/admin/user/detail/${id}`);
}

/**
 * 新增用户
 */
export async function createUserManageApi(data: UserManageCreateReq) {
  return requestClient.post('/api/system/admin/user/add', data);
}

/**
 * 更新用户
 */
export async function updateUserManageApi(
  id: string,
  data: UserManageUpdateReq,
) {
  return requestClient.put(`/api/system/admin/user/update/${id}`, data);
}

/**
 * 更新用户状态
 */
export async function updateUserManageStatusApi(data: {
  id: string;
  status: string;
}) {
  return requestClient.put('/api/system/admin/user/update-status', data);
}

/**
 * 批量删除用户
 */
export async function batchDeleteUserManageApi(ids: string[]) {
  return requestClient.delete('/api/system/admin/user/batch_delete', {
    data: { ids },
  });
}
