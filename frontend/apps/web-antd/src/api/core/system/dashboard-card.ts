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

/** 角色视角预览（管理页增强：选角色查看其可见卡片组合，system:dashboard:list） */
export const getRoleVisibleCardsApi = async (roleId: number) => {
  return requestClient.get('/api/system/dashboard/card/visible_preview', {
    params: { roleId },
  });
};

/** 工作台卡片化总开关状态（workspace_card_enabled，仅需登录） */
export const getDashboardCardStatusApi = async () => {
  return requestClient.get('/api/system/dashboard/card/status');
};

/** 设计器模板布局（方案 5.2：含停用卡片置灰展示，仅需登录） */
export const getCardLayoutApi = async (pageKey: string) => {
  return requestClient.get('/api/system/dashboard/card/layout', {
    params: { pageKey },
  });
};

/** 设计器保存模板布局（system:dashboard:design，整页全量提交） */
export const saveCardLayoutApi = async (param: {
  items: Array<{ cardCode: string; h: number; w: number; x: number; y: number }>;
  pageKey: string;
}) => {
  return requestClient.post('/api/system/dashboard/card/layout/save', param);
};

/** 个人布局（模板布局 + 个人覆盖合并，仅启用卡片；方案 5.4-3 交叉场景记录保留不返回） */
export const getUserLayoutApi = async (pageKey: string) => {
  return requestClient.get('/api/system/dashboard/user/layout', {
    params: { pageKey },
  });
};

/** 保存个人布局（方案 9.4 契约：{ pageKey, cards, reset }；reset=true 清空该页个人覆盖回模板） */
export const saveUserLayoutApi = async (param: {
  cards?: Array<{
    cardCode: string;
    h?: number;
    hidden?: number;
    w?: number;
    x?: number;
    y?: number;
  }>;
  pageKey: string;
  reset?: boolean;
}) => {
  return requestClient.post('/api/system/dashboard/user/layout/save', param);
};
