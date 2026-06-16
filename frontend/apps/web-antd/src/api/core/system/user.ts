import { requestClient } from '#/api/request';

/**
 * 获取用户列表
 */
export const getUserListApi = async (params?: any) => {
  return requestClient.get('/api/system/admin/list', { params });
};

/**
 * 新增用户信息
 */
export const createUserApi = async (param: any) => {
  return requestClient.post('/api/system/admin/add', param);
};

/**
 * 修改用户信息
 */
export const updateUserApi = async (id: number, param: any) => {
  return requestClient.put(`/api/system/admin/update/${id}`, param);
};

/**
 * 删除用户
 */
export const deleteUserApi = async (id: number) => {
  return requestClient.delete(`/api/system/admin/delete/${id}`);
};

/**
 * 修改我的密码
 */
export const updateMyPasswordApi = async (param: {
  confirmPassword: string;
  newPassword: string;
  oldPassword: string;
}) => {
  return requestClient.put('/api/system/admin/update_my_password', param);
};
