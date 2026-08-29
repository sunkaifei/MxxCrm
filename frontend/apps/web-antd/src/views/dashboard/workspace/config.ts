/**
 * 工作台模块权限配置
 *
 * 双层渲染模型（方案 3.2 三因子 / 3.4 降级回退）：
 * - 第一层（卡片注册表）：mxx_system_dashboard_card 注册卡，GET /dashboard/card/visible
 *   按角色返回可见卡片编码集合；workspace_card_enabled 总开关关闭时回退旧渲染
 * - 第二层（模块权限码）：卡片内部模块按 DASHBOARD_MODULE_PERMS 权限码过滤（三因子第三因子）
 * 降级（方案 3.4）：visible 接口失败按"全部注册卡片可见"处理（禁止全隐藏）；
 * 总开关状态接口失败按开启处理；多角色并集由后端 visible 接口保证去重
 */
import { ref } from 'vue';

import { useAccessStore } from '@vben/stores';

import {
  getDashboardCardStatusApi,
  getVisibleDashboardCardsApi,
} from '#/api/core/system/dashboard-card';
import { getWorkspaceListApi } from '#/api/core/system/workspace';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';

/** 工作台注册卡编码（方案 9.1 注册表：一期 4 张 + 三期 8 张，page_key='default'） */
export const WORKSPACE_CARD_CODES = {
  onboarding: 'workspace_onboarding',
  todoOverview: 'workspace_todo_overview',
  smartTodo: 'workspace_smart_todo',
  weekLoad: 'workspace_week_load',
  // 三期 8 张岗位卡（d31 种子，sort 40-110）
  announcement: 'workspace_announcement',
  stockAlert: 'workspace_stock_alert',
  stockDocTodo: 'workspace_stock_doc_todo',
  purchaseApproval: 'workspace_purchase_approval',
  paymentReminder: 'workspace_payment_reminder',
  payslipStat: 'workspace_payslip_stat',
  hrTodo: 'workspace_hr_todo',
  salesPerformance: 'workspace_sales_performance',
} as const;

export type WorkspaceCardCode =
  (typeof WORKSPACE_CARD_CODES)[keyof typeof WORKSPACE_CARD_CODES];

/** 工作台模块 id -> 所需权限码（任一命中即显示；权限码三处一致，复用后端既有守卫码，方案 9.2 禁止新发明） */
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
  // 三期岗位卡内部模块（公告卡不设权限码，由第一层角色分配控制）
  stockAlert: ['product:alert:list'],
  stockDocTodo: ['product:inbound:list', 'product:outbound:list'],
  purchaseApproval: ['purchase:requisition:list'],
  paymentReminder: ['sale:payment:list', 'finance:payment:list', 'crm:contract:list'],
  payslipStat: ['finance:payslip:list'],
  hrTodo: ['system:onboarding:list', 'system:resign:view'],
  salesPerformance: [
    'crm:customer:list',
    'crm:followup:list',
    'crm:opportunity:list',
    'crm:contract:list',
  ],
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

// ===== 注册卡可见性（模块级单例：同页多组件共享同一份状态） =====
const cardModeEnabled = ref(true);
const visibleCodes = ref<null | string[]>(null);
/** cardCode -> pageKey（方案 5.3-M3：page_key 即工作台码，用于按当前工作台过滤卡片） */
const cardPageKeys = ref<Record<string, string>>({});
const cardsLoaded = ref(false);
const cardsLoading = ref(false);

// ===== 多工作台状态（方案 5.3-M3：list_visible 已按用户过滤 + 最近使用记忆） =====
const WORKSPACE_LAST_KEY = 'workspace:last-used';
const workspaceList = ref<Array<{ label: string; value: string }>>([]);
const currentWorkspaceCode = ref('default');
const workspacesLoaded = ref(false);

async function loadWorkspaces() {
  try {
    const list: any = await getWorkspaceListApi();
    const items = (Array.isArray(list) ? list : list?.data || [])
      .map((w: any) => ({
        label: String(w?.workspaceName || w?.workspaceCode || ''),
        value: String(w?.workspaceCode || ''),
      }))
      .filter((w: any) => w.value.length > 0);
    workspaceList.value = items;
    let last: null | string = null;
    try {
      last = localStorage.getItem(WORKSPACE_LAST_KEY);
    } catch {
      last = null;
    }
    if (last && items.some((w: any) => w.value === last)) {
      currentWorkspaceCode.value = last;
    } else if (
      items.length > 0 &&
      !items.some((w: any) => w.value === currentWorkspaceCode.value)
    ) {
      currentWorkspaceCode.value = items[0]!.value;
    }
  } catch {
    // 接口失败降级：保持 default（后端种子 page_key），切换器因列表为空自动隐藏
  } finally {
    workspacesLoaded.value = true;
  }
}

function switchWorkspace(code: string) {
  if (!code || code === currentWorkspaceCode.value) return;
  currentWorkspaceCode.value = code;
  try {
    localStorage.setItem(WORKSPACE_LAST_KEY, code);
  } catch {
    // localStorage 不可用（隐私模式等）时仅本次会话生效
  }
}

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

/**
 * 工作台注册卡可见性（方案 3.2 / 3.4 / 4.1）
 *
 * - cardModeEnabled=false：总开关关闭 → 工作台回旧渲染，注册卡区整体不渲染
 * - visibleCodes=null：visible 接口失败降级 → hasCard 一律 true（全部注册卡可见）
 * - visibleCodes=[]：加载成功但无可见卡 → 触发空态提示（方案 4.5-1）
 */
export function useWorkspaceCards() {
  async function loadWorkspaceCards() {
    if (cardsLoading.value) return;
    cardsLoading.value = true;
    try {
      try {
        const status: any = await getDashboardCardStatusApi();
        cardModeEnabled.value = status?.workspaceCardEnabled !== false;
      } catch {
        cardModeEnabled.value = true;
      }
      if (!cardModeEnabled.value) {
        visibleCodes.value = null;
        return;
      }
      try {
        const list: any = await getVisibleDashboardCardsApi();
        const codes: string[] = [];
        const pageKeys: Record<string, string> = {};
        for (const c of list || []) {
          const code = c?.cardCode;
          if (typeof code !== 'string' || code.length === 0) continue;
          codes.push(code);
          if (typeof c?.pageKey === 'string' && c.pageKey.length > 0) {
            pageKeys[code] = c.pageKey;
          }
        }
        visibleCodes.value = codes;
        cardPageKeys.value = pageKeys;
      } catch {
        visibleCodes.value = null;
      }
    } finally {
      cardsLoaded.value = true;
      cardsLoading.value = false;
    }
  }

  function hasCard(code: string): boolean {
    if (!cardsLoaded.value) return false;
    if (visibleCodes.value === null) return true;
    if (!visibleCodes.value.includes(code)) return false;
    // 方案 5.3-M3：卡片 page_key 即工作台码，仅渲染当前工作台卡片；
    // 接口未返回 pageKey（旧数据兼容）时不拦截
    const pk = cardPageKeys.value[code];
    return pk === undefined || pk === currentWorkspaceCode.value;
  }

  return {
    cardModeEnabled,
    visibleCodes,
    cardsLoaded,
    cardsLoading,
    hasCard,
    loadWorkspaceCards,
    workspaceList,
    currentWorkspaceCode,
    workspacesLoaded,
    loadWorkspaces,
    switchWorkspace,
  };
}
