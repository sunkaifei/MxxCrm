import { requestClient } from '#/api/request';

/**
 * 获取角色列表
 */
export const getRoleListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/role/list', { params });
};

/**
 * 获取角色详情
 */
export const getRoleInfoApi = async (id: number) => {
  return requestClient.get(`/api/system/role/detail/${id}`);
};

/**
 * 获取角色关联的菜单ID列表
 */
export const getRoleMenuIdsApi = async (roleId: number) => {
  return requestClient.get(`/api/system/role/${roleId}/menuIds`);
};

/**
 * 新增角色信息
 */
export const createRoleApi = async (param: any) => {
  return requestClient.post('/api/system/role/save', param);
};

/**
 * 修改角色信息
 */
export const updateRoleApi = async (id: number, param: any) => {
  return requestClient.put(`/api/system/role/update/${id}`, param);
};

/**
 * 删除角色
 */
export const deleteRoleApi = async (id: number) => {
  return requestClient.delete('/api/system/role/bath_delete', {
    data: { ids: [id] },
  });
};

/**
 * 更新角色权限（菜单）
 */
export const updateRoleAuthApi = async (
  id: number,
  data: Record<string, unknown>,
) => {
  return requestClient.put('/api/system/role/assign_perm', {
    roleId: String(id),
    menuIds: data.authId,
  });
};
