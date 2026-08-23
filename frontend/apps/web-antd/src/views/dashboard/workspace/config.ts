/**
 * 工作台模块权限配置
 *
 * 每个模块声明"任一命中即显示"的权限码集合。
 * 基于 RBAC 权限码（accessCodes）判断，超管（data_scope=1 或 user_type=1）默认全显示。
 * 角色授权变化时，工作台展示自动跟随，无需额外维护岗位映射。
 */
import { useAccessStore } from '@vben/stores';

import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';

/** 工作台模块 id -> 所需权限码（任一命中即显示） */
export const DASHBOARD_MODULE_PERMS: Record<string, string[]> = {
  // 头部统计
  customer: ['crm:customer:list'],
  opportunity: ['crm:opportunity:list'],
  // 待办概览 tab（TodoOverviewCard）
  followUp: ['crm:followup:list'],
  approval: ['system:approval:list', 'system:approval:todo'],
  payment: ['sale:payment:list', 'finance:payment:list'],
  contract: ['crm:contract:list'],
  planApproval: ['statistics:performance-plan:audit'],
  cc: ['system:approval:cc:list'],
};

/** 业务待办类模块：仅"参与业务"的用户可见（超管一律不参与业务） */
const BUSINESS_TODO_MODULES = new Set([
  'followUp',
  'approval',
  'payment',
  'contract',
  'opportunity',
  'planApproval',
]);

export function useDashboardPermission() {
  const accessStore = useAccessStore();
  const { isSuperAdmin, isBizUser } = useSuperAdminGuard();

  /**
   * 判断当前用户是否有权查看某工作台模块
   * 业务待办模块：非业务参与人一律隐藏（管理员不参与业务，不显示业务待办）
   * 其余模块：超管始终可见；普通用户任一权限码命中即可见
   */
  function canShow(moduleId: string): boolean {
    if (BUSINESS_TODO_MODULES.has(moduleId)) {
      if (!isBizUser.value) return false;
      const perms = DASHBOARD_MODULE_PERMS[moduleId];
      if (!perms || perms.length === 0) return true;
      return perms.some((perm) => accessStore.hasAccessCode(perm));
    }
    if (isSuperAdmin.value) return true;
    const perms = DASHBOARD_MODULE_PERMS[moduleId];
    if (!perms || perms.length === 0) return true;
    return perms.some((perm) => accessStore.hasAccessCode(perm));
  }

  /** 按权限过滤待办概览 tabKey 列表 */
  function filterOverviewTabs(tabKeys: string[]): string[] {
    return tabKeys.filter((key) => canShow(key));
  }

  return { canShow, filterOverviewTabs, isBizUser };
}
