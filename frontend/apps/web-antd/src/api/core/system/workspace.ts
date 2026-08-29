import { requestClient } from '#/api/request';

/**
 * 多工作台（方案 5.2 二期接口：GET list 仅需登录 / POST save 需 system:workspace:save）
 */

/** 当前用户可见工作台（M3：index.vue 切换器 + 设计器工作台下拉共用） */
export const getWorkspaceListApi = async () => {
  return requestClient.get('/api/system/workspace/list');
};

/** 保存工作台（id 空新增否则更新） */
export const saveWorkspaceApi = async (param: {
  icon?: string;
  id?: number;
  isDefault?: number;
  sort?: number;
  status?: number;
  workspaceCode?: string;
  workspaceName?: string;
}) => {
  return requestClient.post('/api/system/workspace/save', param);
};
