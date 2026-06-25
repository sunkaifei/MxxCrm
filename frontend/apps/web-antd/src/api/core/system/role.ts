import { requestClient } from '#/api/request';

/**
 * 获取角色列表
 */
export const getRoleListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/role/list', { params });
};

export const getRoleOptionsApi = async () => {
  return requestClient.get('/api/system/role/options');
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

/**
 * 获取角色关联的部门ID列表（数据权限-自定义）
 */
export const getRoleDeptIdsApi = async (roleId: number) => {
  return requestClient.get(`/api/system/role/${roleId}/deptIds`);
};

/**
 * 更新角色数据权限部门
 */
export const updateRoleDeptApi = async (
  id: number,
  deptIds: (string | number)[],
) => {
  return requestClient.put('/api/system/role/assign_data_scope', {
    roleId: String(id),
    deptIds: deptIds.map(String),
  });
};
