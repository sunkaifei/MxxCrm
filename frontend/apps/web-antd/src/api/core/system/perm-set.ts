import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

/**
 * 获取权限集列表
 */
export const getPermSetListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/perm_set/list', { params });
};

export const getPermSetOptionsApi = async () => {
  return requestClient.get('/api/system/perm_set/options');
};

/**
 * 获取权限集详情
 */
export const getPermSetInfoApi = async (id: number) => {
  return requestClient.get(`/api/system/perm_set/detail/${id}`);
};

/**
 * 获取权限集关联的菜单ID列表
 */
export const getPermSetMenuIdsApi = async (permSetId: number) => {
  return requestClient.get(`/api/system/perm_set/${permSetId}/menuIds`);
};

/**
 * 新增权限集
 */
export const createPermSetApi = async (param: any) => {
  return requestClient.post('/api/system/perm_set/save', param);
};

/**
 * 修改权限集
 */
export const updatePermSetApi = async (id: number, param: any) => {
  return requestClient.put(`/api/system/perm_set/update/${id}`, param);
};

/**
 * 删除权限集
 */
export const deletePermSetApi = async (id: number) => {
  return requestClient.delete('/api/system/perm_set/bath_delete', {
    data: { ids: [id] },
  });
};

/**
 * 更新权限集权限（菜单）
 */
export const updatePermSetAuthApi = async (
  id: number,
  data: Record<string, unknown>,
) => {
  return requestClient.put('/api/system/perm_set/assign_perm', {
    permSetId: String(id),
    menuIds: data.authId,
  });
};

/**
 * 复制权限集（P3-1 一键复制：深拷贝菜单授权配置）
 */
export const copyPermSetApi = async (id: number) => {
  return requestClient.post(`/api/system/perm_set/copy/${id}`);
};
