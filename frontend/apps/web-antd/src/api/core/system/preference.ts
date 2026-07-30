import { requestClient } from '#/api/request';

// ============ 用户偏好设置 API ============

/** 快捷导航项 */
export interface QuickNavItem {
  menuId: number;
  sort: number;
}

/** 获取当前用户快捷导航配置 */
export const getQuickNavPreferenceApi = async () =>
  requestClient.get('/api/system/preference/quick-nav');

/** 保存当前用户快捷导航配置 */
export const saveQuickNavPreferenceApi = async (data: QuickNavItem[]) =>
  requestClient.put('/api/system/preference/quick-nav', data);

/** 获取销售简易模式开关 */
export const getSaleSimpleModeApi = async () =>
  requestClient.get<boolean>('/api/system/preference/sale-mode');

/** 保存销售简易模式开关 */
export const saveSaleSimpleModeApi = async (enabled: boolean) =>
  requestClient.put('/api/system/preference/sale-mode', enabled);
