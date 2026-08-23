import { requestClient } from '#/api/request';

/**
 * 工作台卡片配置中心
 * 集中控制各页面统计/概览卡片对哪些角色可见
 */

/** 卡片分页列表（含已分配角色ID） */
export const getDashboardCardListApi = async (params?: any) => {
  return requestClient.get('/api/system/dashboard/card/list', { params });
};

/** 新增卡片 */
export const createDashboardCardApi = async (param: any) => {
  return requestClient.post('/api/system/dashboard/card/save', param);
};

/** 更新卡片 */
export const updateDashboardCardApi = async (param: any) => {
  return requestClient.put('/api/system/dashboard/card/update', param);
};

/** 删除卡片 */
export const deleteDashboardCardApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/dashboard/card/bath_delete', {
    data: { ids },
  });
};

/** 分配卡片可见角色 */
export const assignDashboardCardRolesApi = async (param: {
  cardId: number;
  roleIds: number[];
}) => {
  return requestClient.put('/api/system/dashboard/card/assign_roles', param);
};

/** 当前用户可见卡片（前端页面动态渲染统计/概览卡片） */
export const getVisibleDashboardCardsApi = async () => {
  return requestClient.get('/api/system/dashboard/card/visible');
};
