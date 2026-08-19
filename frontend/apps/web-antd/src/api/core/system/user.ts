import { requestClient } from '#/api/request';

/**
 * 获取用户下拉选项列表
 */
export const getAdminOptionsApi = async () => {
  return requestClient.get('/api/system/admin/options');
};

/**
 * 获取用户列表
 */
export const getUserListApi = async (params?: any) => {
  return requestClient.get('/api/system/admin/list', { params });
};

/**
 * 获取用户详情
 */
export const getUserDetailApi = async (id: number | string) => {
  return requestClient.get(`/api/system/admin/detail/${id}`);
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
export const updateUserApi = async (param: any) => {
  return requestClient.put('/api/system/admin/update', param);
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

/**
 * 踢用户下线（强制该用户所有设备重新登录）
 */
export const kickOfflineApi = async (id: number) => {
  return requestClient.post(`/api/system/admin/kick-offline/${id}`);
};

/**
 * 审核注册用户（1=通过 0=拒绝）
 */
export const auditUserApi = async (id: number, auditStatus: number) => {
  return requestClient.put(`/api/system/admin/audit/${id}`, { auditStatus });
};

/**
 * 获取员工列表列显示配置
 */
export const getColumnsConfigApi = async () => {
  return requestClient.get('/api/system/admin/columns_config');
};

/**
 * 保存员工列表列显示配置
 */
export const saveColumnsConfigApi = async (config: Record<string, string[]>) => {
  return requestClient.put('/api/system/admin/columns_config', config);
};
