import type { RouteRecordStringComponent } from '@vben/types';

import { requestClient } from '#/api/request';

/**
 * 获取用户所有菜单
 */
export async function getAllMenusApi() {
  const result = await requestClient.get<RouteRecordStringComponent[]>(
    '/api/system/menu/getUserMenus',
  );
  return Array.isArray(result) ? result : [];
}
