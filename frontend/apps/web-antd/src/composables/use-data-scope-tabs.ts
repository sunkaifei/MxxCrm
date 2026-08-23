import { computed } from 'vue';

import { useUserStore } from '@vben/stores';

import { useSuperAdminGuard } from './use-super-admin-guard';

/**
 * 数据范围 Tab 控制（全部 / 我的 / 下属）
 *
 * 判定与后端 get_accessible_user_ids 保持一致：
 * - 超级管理员（user_type=1）或系统管理员（data_scope=1）：全部数据
 * - data_scope=2/3/4（部门级）：可见"下属"数据
 * - data_scope=5/未设置：仅本人
 *
 * 说明：超管判定依赖 user_type/data_scope 而非角色 key（角色 key 可能缺失），
 * 与 useSuperAdminGuard 及后端一致，避免超管看不到"全部"Tab。
 */
export function useDataScopeTabs() {
  const userStore = useUserStore();
  const { isSuperAdmin } = useSuperAdminGuard();

  // 归一化 data_scope（超管/系统管理员视为 1=全部数据权限）
  const dataScope = computed<number>(() => {
    if (isSuperAdmin.value) return 1;
    const scope =
      (userStore.userInfo as any)?.dataScope ??
      (userStore.userInfo as any)?.data_scope;
    return typeof scope === 'number' ? scope : 5;
  });

  // "全部" Tab：超级管理员 / 系统管理员 / 全部数据权限
  const canViewAll = computed(() => isSuperAdmin.value);

  // "下属" Tab：超级管理员 / 系统管理员，或部门级权限（2自定义/3本部门/4本部门及以下）
  const canViewSubordinate = computed(() => {
    const scope = dataScope.value;
    return isSuperAdmin.value || scope === 2 || scope === 3 || scope === 4;
  });

  return { canViewAll, canViewSubordinate, dataScope, isSuperAdmin };
}
