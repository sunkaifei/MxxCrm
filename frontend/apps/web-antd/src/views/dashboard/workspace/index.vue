<script lang="ts" setup>
import type { WorkbenchQuickNavItem } from '@vben/common-ui';

import type { QuickNavItem } from '#/api';

import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import { WorkbenchHeader, WorkbenchQuickNav } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { openWindow } from '@vben/utils';

import { Button, Empty, message, Popover, Select } from 'ant-design-vue';

import {
  getCustomerListApi,
  getMenusRouterApi,
  getMyProfileApi,
  getOpportunityListApi,
  getQuickNavPreferenceApi,
  getSaleSimpleModeApi,
  getTodaySummaryApi,
} from '#/api';
import {
  getUserLayoutApi,
  saveUserLayoutApi,
} from '#/api/core/system/dashboard-card';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
import { $t } from '#/locales';

import ContractApprovalDrawer from '../../crm/contract/approval-drawer.vue';
// 审批流抽屉组件（复用业务模块现有组件，工作台内嵌打开）
import OrderApprovalDrawer from '../../sale/order/approval-drawer.vue';
import QuickNavSettingsModal from '../components/QuickNavSettingsModal.vue';
import TodoOverviewCard from '../components/TodoOverviewCard.vue';
import WorkLogCard from '../components/WorkLogCard.vue';
import { getCanvasCardDef } from './canvas';
import OnboardingCard from './components/OnboardingCard.vue';
import SmartTodoCard from './components/SmartTodoCard.vue';
import WeekLoadCard from './components/WeekLoadCard.vue';
import {
  useDashboardPermission,
  useWorkspaceCards,
  WORKSPACE_CARD_CODES,
} from './config';

const router = useRouter();
const userStore = useUserStore();
const accessStore = useAccessStore();

// ===== 工作台模块权限控制 =====
const { canShow, filterOverviewTabs, isBizUser } = useDashboardPermission();

// ===== 注册卡可见性（方案 3.2/4.1：第一层卡片注册表，cardModeEnabled=false 回旧渲染） =====
const {
  cardModeEnabled,
  visibleCodes,
  cardsLoaded,
  hasCard,
  loadWorkspaceCards,
  workspaceList,
  currentWorkspaceCode,
  workspacesLoaded,
  loadWorkspaces,
  switchWorkspace,
} = useWorkspaceCards();
const { isSuperAdmin } = useSuperAdminGuard();

// 入职审批状态（audit_status=1 已通过 → 引导卡消失；NULL/0 → 显示，覆盖 HR 建号未审批）
const myAuditStatus = ref<null | number>(null);
async function loadMyAuditStatus() {
  try {
    const res: any = await getMyProfileApi();
    const p = res?.data ?? res ?? {};
    myAuditStatus.value = p.auditStatus ?? p.audit_status ?? null;
  } catch {
    myAuditStatus.value = null;
  }
}

// 引导卡显示条件（方案 4.4）：hasCard + userType != 1 + audit_status != 1（含 NULL）
const currentUserType = computed(() => {
  const info: any = userStore.userInfo || {};
  return Number(info.userType ?? info.user_type ?? 0);
});
const showOnboardingCard = computed(
  () =>
    cardModeEnabled.value &&
    hasCard(WORKSPACE_CARD_CODES.onboarding) &&
    currentUserType.value !== 1 &&
    Number(myAuditStatus.value) !== 1,
);

// 空态提示（方案 4.5-1）：注册卡全不可见（visibleCodes=[]，接口失败降级为 null 时不提示）；
// 非超管仅提示，超管附"去配置"入口
const showEmptyTip = computed(
  () =>
    cardsLoaded.value &&
    cardModeEnabled.value &&
    visibleCodes.value !== null &&
    visibleCodes.value.length === 0,
);

// 引导卡"查看待办总览"：定位工作台待办概览卡（ref 定位，卡片被隐藏时回退审批待办页）
function handleViewTodos() {
  if (showOverviewCard.value && hasCard(WORKSPACE_CARD_CODES.todoOverview)) {
    const card: any = getCardRef(WORKSPACE_CARD_CODES.todoOverview);
    if (card?.$el) {
      card.$el.scrollIntoView({ behavior: 'smooth', block: 'start' });
      return;
    }
  }
  router.push('/system/approval/todo').catch(() => {});
}
// 空态提示"去配置"：跳转工作台设计器（超管入口）
function goConfigure() {
  router.push('/system/dashboard-designer').catch(() => {});
}
// 待办概览卡可见 tab（按权限过滤）
const visibleOverviewTabs = computed(() =>
  filterOverviewTabs([
    'followUp',
    'approval',
    'payment',
    'contract',
    'opportunity',
    'planApproval',
    'cc',
  ]),
);
// 待办概览整卡：无任何 tab 权限则隐藏
const showOverviewCard = computed(() => visibleOverviewTabs.value.length > 0);
// 智能待办区可见性（画布过滤用；SmartTodoCard 内部同样自判）
const showSmartTodo = computed(
  () =>
    canShow('approval') ||
    canShow('followUp') ||
    canShow('payment') ||
    canShow('planApproval'),
);
// 欢迎语（含岗位/部门个性化）
const welcomeText = computed(() => {
  const info: any = userStore.userInfo || {};
  const name = info.realName || info.nickname || '同事';
  const tags = [
    ...(Array.isArray(info.deptNames) ? info.deptNames : []),
    ...(Array.isArray(info.postNames) ? info.postNames : []),
  ]
    .filter((t: any) => typeof t === 'string' && t.length > 0)
    .slice(0, 2);
  return tags.length > 0
    ? `早安，${name}（${tags.join(' · ')}）`
    : `早安，${name}`;
});

// ===== 内嵌审批抽屉 =====
const approvalDrawerOrderId = ref<null | number>(null);
const approvalDrawerContractId = ref<null | number>(null);
const orderApprovalVisible = ref(false);
const contractApprovalVisible = ref(false);
const approvalCurrentUserId = computed(() =>
  userStore.userInfo?.userId ? Number(userStore.userInfo.userId) : undefined,
);

// 处理查看审批流详情事件：在工作台内嵌打开抽屉
function handleViewApproval(payload: {
  businessId: number;
  businessType: string;
  instanceId: number;
}) {
  const { businessType, businessId } = payload;
  switch (businessType) {
    case 'contract': {
      approvalDrawerContractId.value = businessId;
      contractApprovalVisible.value = true;
      break;
    }
    case 'order': {
      approvalDrawerOrderId.value = businessId;
      orderApprovalVisible.value = true;
      break;
    }
    default: {
      // 报价单/回款/发票等其他业务暂无独立审批抽屉，跳转审批待办页
      router
        .push(`/system/approval/todo?instanceId=${payload.instanceId}`)
        .catch(() => {});
      break;
    }
  }
}

// 审批抽屉操作成功后刷新相关卡片（卡片自加载，经 expose reload 触发）
function handleApprovalSuccess() {
  getCardRef(WORKSPACE_CARD_CODES.smartTodo)?.reload?.();
  getCardRef(WORKSPACE_CARD_CODES.weekLoad)?.reload?.();
  workLogRefreshKey.value++;
}

// ===== 工作日志刷新 key =====
const workLogRefreshKey = ref(0);

// ===== 卡片实例引用（画布/回退两种模式统一经 bindCardRef 收集，供 reload/定位） =====
const cardRefMap: Record<string, any> = {};
function bindCardRef(el: any, code: string) {
  if (el) {
    cardRefMap[code] = el;
  } else {
    delete cardRefMap[code];
  }
}
function getCardRef(code: string): any {
  return cardRefMap[code];
}

// ===== 快捷导航 =====
const quickNavItems = ref<WorkbenchQuickNavItem[]>([]);
const navSettingsVisible = ref(false);
// 销售简易模式开关
const saleSimpleMode = ref(false);
// 标准模式默认快捷导航（按销售流程排序：客户→商机→报价单→订单→合同→回款）
const defaultQuickNavStandard: WorkbenchQuickNavItem[] = [
  {
    color: '#1890ff',
    icon: 'lucide:users',
    title: $t('page.crm.customer.title'),
    url: '/crm/customer',
  },
  {
    color: '#52c41a',
    icon: 'lucide:target',
    title: $t('page.crm.opportunity.title'),
    url: '/sale/opportunity',
  },
  {
    color: '#722ed1',
    icon: 'lucide:file-text',
    title: $t('page.sale.quotation.title'),
    url: '/sale/quotation',
  },
  {
    color: '#faad14',
    icon: 'lucide:shopping-cart',
    title: $t('page.sale.order.title'),
    url: '/sale/order',
  },
  {
    color: '#13c2c2',
    icon: 'lucide:file-text',
    title: $t('page.crm.contract.title'),
    url: '/sale/contract',
  },
  {
    color: '#eb2f96',
    icon: 'lucide:wallet',
    title: $t('page.sale.payment.title'),
    url: '/sale/payment',
  },
];

// 简易模式默认快捷导航（按销售流程排序：线索→客户→商机→报价单→订单→合同）
const defaultQuickNavSimple: WorkbenchQuickNavItem[] = [
  {
    color: '#1890ff',
    icon: 'lucide:contact',
    title: $t('page.crm.lead.title'),
    url: '/crm/lead',
  },
  {
    color: '#52c41a',
    icon: 'lucide:users',
    title: $t('page.crm.customer.title'),
    url: '/crm/customer',
  },
  {
    color: '#722ed1',
    icon: 'lucide:target',
    title: $t('page.crm.opportunity.title'),
    url: '/sale/opportunity',
  },
  {
    color: '#faad14',
    icon: 'lucide:file-text',
    title: $t('page.sale.quotation.title'),
    url: '/sale/quotation',
  },
  {
    color: '#13c2c2',
    icon: 'lucide:shopping-cart',
    title: $t('page.sale.order.title'),
    url: '/sale/order',
  },
  {
    color: '#eb2f96',
    icon: 'lucide:file-text',
    title: $t('page.crm.contract.title'),
    url: '/sale/contract',
  },
];

// 根据简易模式返回默认导航
function getDefaultQuickNavItems(): WorkbenchQuickNavItem[] {
  return saleSimpleMode.value ? defaultQuickNavSimple : defaultQuickNavStandard;
}

// 展平菜单树，只保留有 path 的叶子节点
function flattenMenus(menus: any[]): any[] {
  const result: any[] = [];
  const traverse = (list: any[]) => {
    if (!Array.isArray(list)) return;
    for (const menu of list) {
      if (!menu) continue;
      const children = menu.children || [];
      if (menu.path && children.length === 0) {
        result.push(menu);
      }
      if (children.length > 0) {
        traverse(children);
      }
    }
  };
  traverse(menus);
  return result;
}

async function loadQuickNav() {
  try {
    // 并行加载简易模式开关和快捷导航配置
    const [simpleMode, prefResp]: any = await Promise.all([
      getSaleSimpleModeApi().catch(() => false),
      getQuickNavPreferenceApi().catch(() => null),
    ]);
    saleSimpleMode.value = !!simpleMode;

    const savedPref: QuickNavItem[] = Array.isArray(prefResp)
      ? prefResp
      : prefResp?.items || [];

    if (!savedPref || savedPref.length === 0) {
      // 无自定义配置，使用简易模式/标准模式默认导航
      quickNavItems.value = getDefaultQuickNavItems();
      return;
    }

    const menuResp: any = await getMenusRouterApi({});
    const allMenus = flattenMenus(menuResp?.items || menuResp || []);

    // 按 sort 顺序匹配前 6 个
    const sortedPref = savedPref.toSorted(
      (a, b) => (a.sort ?? 0) - (b.sort ?? 0),
    );
    const items: WorkbenchQuickNavItem[] = [];
    for (const pref of sortedPref) {
      if (items.length >= 6) break;
      const menu = allMenus.find((m) => m.id === pref.menuId);
      if (!menu) continue;
      const meta = menu.meta || {};
      const rawTitle = meta.title || menu.name || menu.path;
      const title =
        typeof rawTitle === 'string' && rawTitle.startsWith('page.')
          ? $t(rawTitle)
          : rawTitle;
      items.push({
        color: '#1890ff',
        icon: meta.icon || 'lucide:menu',
        title: typeof title === 'string' ? title : String(title || ''),
        url: menu.path,
      });
    }
    quickNavItems.value = items.length > 0 ? items : getDefaultQuickNavItems();
  } catch {
    quickNavItems.value = getDefaultQuickNavItems();
  }
}

// ===== 待办概览卡片点击 =====
const overviewRouteMap: Record<string, string> = {
  approval: '/system/approval/todo',
  followUp: '/crm/customer',
  // 待办回款与回款计划同源(contract_payment_plan)，跳回款计划列表而非回款登记列表
  payment: '/sale/payment-plan',
  contract: '/sale/contract',
  opportunity: '/sale/opportunity',
  planApproval: '/dashboard/performance',
};

function handleOverviewClick(tabKey: string) {
  // 跳转到对应业务列表页
  const path = overviewRouteMap[tabKey];
  if (path) {
    router.push(path).catch(() => {
      // 跳转失败忽略
    });
  }
}

// ===== 快捷导航点击跳转 =====
function navTo(nav: WorkbenchQuickNavItem) {
  if (nav.url?.startsWith('http')) {
    openWindow(nav.url);
    return;
  }
  if (nav.url?.startsWith('/')) {
    router.push(nav.url).catch((error) => {
      console.error('Navigation failed:', error);
    });
  } else {
    console.warn(`Unknown URL for navigation item: ${nav.title} -> ${nav.url}`);
  }
}

// ===== WorkbenchHeader 动态数据 =====
// 今日已处理数（来自 mxx_work_log 持久化，由后端聚合接口返回）
const todoProcessed = ref(0);
// 今日待办总数（= 已处理数 + 剩余待办数，由后端聚合接口返回）
const todoTotal = ref(0);
const customerCount = ref<null | number>(null);
const opportunityCount = ref<null | number>(null);
// 智能待办总数（由 SmartTodoCard 经 count-change 事件上抛）
const todoCount = ref(0);

// 智能待办处理完成：刷新头部汇总与工作日志（待办列表由卡片内部重载）
function handleSmartProcessed() {
  loadTodaySummary();
  workLogRefreshKey.value++;
}

// 引导卡提交审批成功：刷新审批状态 + 智能待办（计划待审批随审核状态变化）
function handleAuditChange() {
  loadMyAuditStatus();
  getCardRef(WORKSPACE_CARD_CODES.smartTodo)?.reload?.();
}

// 加载客户总数（无权限不请求；失败静默降级，无权限/失败均显示 --）
async function loadCustomerCount() {
  if (!canShow('customer')) {
    customerCount.value = null;
    return;
  }
  try {
    const res: any = await getCustomerListApi(
      { pageNum: 1, pageSize: 1 },
      { silentError: true },
    );
    customerCount.value = res?.total || 0;
  } catch {
    customerCount.value = null;
  }
}

// 加载商机总数（无权限不请求；失败静默降级，无权限/失败均显示 --）
async function loadOpportunityCount() {
  if (!canShow('opportunity')) {
    opportunityCount.value = null;
    return;
  }
  try {
    const res: any = await getOpportunityListApi(
      { pageNum: 1, pageSize: 1 },
      { silentError: true },
    );
    opportunityCount.value = res?.total || 0;
  } catch {
    opportunityCount.value = null;
  }
}

// 加载今日待办汇总（已处理数 + 总数 + 完成率，来自后端聚合接口）
// 已处理数来自 mxx_work_log 持久化，剩余数实时查询，总数 = 已处理 + 剩余
// 不参与业务的账号（含超管）：无业务待办，汇总直接置 0，避免请求
async function loadTodaySummary() {
  if (!isBizUser.value) {
    todoProcessed.value = 0;
    todoTotal.value = 0;
    return;
  }
  try {
    const res: any = await getTodaySummaryApi();
    todoProcessed.value = res?.todoProcessed || 0;
    todoTotal.value = res?.todoTotal || 0;
  } catch {
    todoProcessed.value = 0;
    todoTotal.value = 0;
  }
}

// ===== 画布布局（方案 5.3-M1：gridstack 12 列画布；M3：page_key 即工作台码，切换重载） =====
const pageKey = computed(() => currentWorkspaceCode.value);

interface CanvasLayoutItem {
  cardCode: string;
  h: number;
  hidden: number;
  w: number;
  x: number;
  y: number;
}

interface SaveLayoutPayload {
  cards: CanvasLayoutItem[];
  pageKey: string;
}

const layoutItems = ref<CanvasLayoutItem[]>([]);
const canvasReady = ref(false);
const canvasContainer = ref<HTMLElement>();
let gridStack: any = null;
let rebuildingGrid = false;
let saveTimer: null | ReturnType<typeof setTimeout> = null;
// 已提交未确认（在途或失败）的保存载荷：卸载补交的兜底数据
let pendingSavePayload: null | SaveLayoutPayload = null;

// 卡片内容级可见性（第二层过滤：卡片注册可见 ≠ 内容可见）
function isCardContentVisible(code: string): boolean {
  switch (code) {
    case WORKSPACE_CARD_CODES.onboarding:
      return currentUserType.value !== 1 && Number(myAuditStatus.value) !== 1;
    case WORKSPACE_CARD_CODES.smartTodo:
      return showSmartTodo.value;
    case WORKSPACE_CARD_CODES.todoOverview:
      return showOverviewCard.value;
    case WORKSPACE_CARD_CODES.weekLoad:
      return true;
    default:
      return true;
  }
}

// 卡片标题（隐藏待恢复列表展示用）
const CARD_TITLE_KEYS: Record<string, string> = {
  [WORKSPACE_CARD_CODES.onboarding]: 'page.dashboard.onboarding.cardTitle',
  [WORKSPACE_CARD_CODES.smartTodo]: 'page.dashboard.todoList',
  [WORKSPACE_CARD_CODES.todoOverview]: 'page.dashboard.todoOverview',
  [WORKSPACE_CARD_CODES.weekLoad]: 'page.dashboard.weekWorkload',
  [WORKSPACE_CARD_CODES.calendar]:
    'page.dashboard.workspace.cards.calendar.title',
  // 三期 8 张岗位卡（d31 种子）
  [WORKSPACE_CARD_CODES.announcement]:
    'page.dashboard.workspace.cards.announcement.title',
  [WORKSPACE_CARD_CODES.stockAlert]:
    'page.dashboard.workspace.cards.stockAlert.title',
  [WORKSPACE_CARD_CODES.stockDocTodo]:
    'page.dashboard.workspace.cards.stockDocTodo.title',
  [WORKSPACE_CARD_CODES.purchaseApproval]:
    'page.dashboard.workspace.cards.purchaseApproval.title',
  [WORKSPACE_CARD_CODES.paymentReminder]:
    'page.dashboard.workspace.cards.paymentReminder.title',
  [WORKSPACE_CARD_CODES.payslipStat]:
    'page.dashboard.workspace.cards.payslipStat.title',
  [WORKSPACE_CARD_CODES.hrTodo]: 'page.dashboard.workspace.cards.hrTodo.title',
  [WORKSPACE_CARD_CODES.salesPerformance]:
    'page.dashboard.workspace.cards.salesPerformance.title',
};
function cardTitle(code: string): string {
  const key = CARD_TITLE_KEYS[code];
  return key ? $t(key) : code;
}

// 画布动态渲染注入（方案 9.1：注册表保持纯净，index.vue 按 code 注入参数与事件）
function cardProps(code: string): Record<string, any> {
  if (code === WORKSPACE_CARD_CODES.todoOverview) {
    return { visibleTabs: visibleOverviewTabs.value };
  }
  return {};
}

function cardEvents(code: string): Record<string, any> {
  switch (code) {
    case WORKSPACE_CARD_CODES.onboarding: {
      return { onAuditChange: handleAuditChange, onViewTodos: handleViewTodos };
    }
    case WORKSPACE_CARD_CODES.todoOverview: {
      return { onClickCard: handleOverviewClick };
    }
    case WORKSPACE_CARD_CODES.smartTodo: {
      return {
        onCountChange: (n: number) => {
          todoCount.value = n;
        },
        onProcessed: handleSmartProcessed,
        onViewApproval: handleViewApproval,
      };
    }
    default: {
      return {};
    }
  }
}

// 画布可见节点（hidden + 注册表 + 两层可见性过滤）与隐藏待恢复列表
const gridDomItems = computed(() =>
  layoutItems.value.filter(
    (item) =>
      item.hidden !== 1 &&
      hasCard(item.cardCode) &&
      isCardContentVisible(item.cardCode) &&
      !!getCanvasCardDef(item.cardCode),
  ),
);
const hiddenLayoutItems = computed(() =>
  layoutItems.value.filter(
    (item) =>
      item.hidden === 1 &&
      hasCard(item.cardCode) &&
      isCardContentVisible(item.cardCode) &&
      !!getCanvasCardDef(item.cardCode),
  ),
);

// 画布模式判定（回退分支渲染条件：总开关关闭或画布初始化失败，方案 3.4）
const canvasFailed = ref(false);
let canvasInitTried = false;
const useCanvasMode = computed(
  () => cardModeEnabled.value && canvasReady.value && !canvasFailed.value,
);

// 拉取个人布局（后端已合并模板默认值与个人覆盖，仅启用卡片）
async function loadUserLayout(): Promise<CanvasLayoutItem[]> {
  try {
    const list: any = await getUserLayoutApi(pageKey.value);
    return (Array.isArray(list) ? list : []).map((i: any) => ({
      cardCode: String(i?.cardCode || ''),
      h: Math.max(1, Number(i?.h) || 6),
      hidden: Number(i?.hidden) === 1 ? 1 : 0,
      w: Math.min(12, Math.max(1, Number(i?.w) || 12)),
      x: Math.max(0, Number(i?.x) || 0),
      y: Math.max(0, Number(i?.y) || 0),
    }));
  } catch {
    return [];
  }
}

function gridStackOptions() {
  return {
    cellHeight: 80,
    column: 12,
    margin: 8,
    minRow: 1,
  };
}

// 初始化画布：动态加载 gridstack（失败仅降级回退渲染，不影响旧模式）
async function initCanvas() {
  if (canvasInitTried) return;
  canvasInitTried = true;
  layoutItems.value = await loadUserLayout();
  await nextTick();
  if (!canvasContainer.value) return;
  try {
    const { GridStack } = await import('gridstack');
    gridStack = GridStack.init(
      gridStackOptions(),
      canvasContainer.value.querySelector('.grid-stack') as HTMLElement,
    );
    bindGridEvents();
    canvasReady.value = true;
  } catch {
    canvasFailed.value = true;
  }
}

// 拖拽/缩放结束触发 change，防抖 500ms 后持久化（rebuild 期间的 change 忽略）
function bindGridEvents() {
  if (!gridStack) return;
  gridStack.on('change', () => {
    if (rebuildingGrid) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      saveTimer = null;
      void saveLayout();
    }, 500);
  });
}

// 组装待保存布局：画布实际节点 + 仍隐藏卡片（hidden=1 原样保留）；h/w 按卡片定义钳制保证所见即所得
function buildSavePayload(): SaveLayoutPayload | null {
  if (!gridStack) return null;
  const saved: any[] = gridStack.save(false) || [];
  const cards: CanvasLayoutItem[] = saved
    .filter((n) => n && n.id && getCanvasCardDef(String(n.id)))
    .map((n) => {
      const def = getCanvasCardDef(String(n.id));
      return {
        cardCode: String(n.id),
        h: Math.min(
          def?.maxH ?? 99,
          Math.max(def?.minH ?? 1, Math.round(Number(n.h) || 1)),
        ),
        hidden: 0,
        w: Math.min(
          def?.maxW ?? 12,
          Math.max(def?.minW ?? 1, Math.round(Number(n.w) || 12)),
        ),
        x: Math.max(0, Math.round(Number(n.x) || 0)),
        y: Math.max(0, Math.round(Number(n.y) || 0)),
      };
    });
  const hiddenCards = layoutItems.value
    .filter((i) => i.hidden === 1 && getCanvasCardDef(i.cardCode))
    .map((i) => ({ ...i }));
  return { cards: [...cards, ...hiddenCards], pageKey: pageKey.value };
}

// 布局持久化（change 防抖 500ms 后调用）
async function saveLayout() {
  const payload = buildSavePayload();
  if (!payload) return;
  try {
    pendingSavePayload = payload;
    await saveUserLayoutApi(payload);
    pendingSavePayload = null;
    for (const c of payload.cards) {
      if (c.hidden === 1) continue;
      const item = layoutItems.value.find((i) => i.cardCode === c.cardCode);
      if (item) Object.assign(item, c);
    }
  } catch {
    message.error($t('page.dashboard.canvas.saveFailed'));
  }
}

// 卸载前补交：防抖窗口或请求在途时刷新/关闭页面，axios 会随页面卸载被取消导致布局丢失；
// 用 keepalive fetch 尽力补交（payload 固化 pageKey，跨工作台补交不会存错页；后端 upsert 幂等）
function flushPendingSave() {
  let payload = pendingSavePayload;
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
    const fresh = buildSavePayload();
    if (fresh) payload = fresh;
  }
  const token = accessStore.accessToken;
  if (!payload || !token) return;
  try {
    void fetch('/api/system/dashboard/user/layout/save', {
      body: JSON.stringify(payload),
      headers: {
        Authorization: `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      keepalive: true,
      method: 'POST',
    }).catch(() => {});
  } catch {
    // 页面卸载场景尽力而为
  }
}

// F5/关闭标签页时 Vue 卸载钩子不保证执行，pagehide 是补交的最后时机
function handlePageHide() {
  flushPendingSave();
}

// 重建画布（复位/恢复卡片后调用）：destroy → 重载布局 → 清 inline 残留 → 重新 init
async function rebuildGrid() {
  if (!canvasContainer.value) return;
  flushPendingSave();
  rebuildingGrid = true;
  try {
    if (gridStack) {
      gridStack.destroy(false);
      gridStack = null;
    }
    layoutItems.value = await loadUserLayout();
    await nextTick();
    if (!canvasContainer.value) return;
    canvasContainer.value
      .querySelectorAll('.grid-stack-item')
      .forEach((el) => (el as HTMLElement).removeAttribute('style'));
    const { GridStack } = await import('gridstack');
    gridStack = GridStack.init(
      gridStackOptions(),
      canvasContainer.value.querySelector('.grid-stack') as HTMLElement,
    );
    bindGridEvents();
  } catch {
    canvasFailed.value = true;
  } finally {
    rebuildingGrid = false;
  }
}

// 隐藏卡片：从画布移除节点（DOM 由 Vue 管理），hidden=1 持久化
function hideCard(code: string) {
  const item = layoutItems.value.find((i) => i.cardCode === code);
  if (!item || item.hidden === 1) return;
  item.hidden = 1;
  rebuildingGrid = true;
  try {
    const el = gridStack?.el?.querySelector(`[gs-id="${code}"]`);
    if (el && gridStack) gridStack.removeWidget(el, false);
  } finally {
    rebuildingGrid = false;
  }
  void saveLayout();
}

async function restoreCard(code: string) {
  const item = layoutItems.value.find((i) => i.cardCode === code);
  if (!item) return;
  item.hidden = 0;
  await rebuildGrid();
  void saveLayout();
}

// 复位：清空个人覆盖（reset=true），后端回退模板默认布局
async function resetLayout() {
  try {
    await saveUserLayoutApi({ pageKey: pageKey.value, reset: true });
    message.success($t('page.dashboard.canvas.resetDone'));
    await rebuildGrid();
  } catch {
    message.error($t('page.dashboard.canvas.saveFailed'));
  }
}

// 注册卡与工作台列表就绪且总开关开启后初始化画布（M3：布局按当前工作台 pageKey 拉取）
watch(
  [cardModeEnabled, cardsLoaded, workspacesLoaded],
  async ([enabled, loaded, wsLoaded]) => {
    if (enabled && loaded && wsLoaded && !canvasInitTried) {
      await nextTick();
      await initCanvas();
    }
  },
  { immediate: true },
);

// 切换工作台（方案 5.3-M3）：记忆最近使用 + 按新 pageKey 重载个人布局
async function handleWorkspaceSwitch(value: any) {
  const code = String(value ?? '');
  flushPendingSave();
  switchWorkspace(code);
  if (useCanvasMode.value) {
    await rebuildGrid();
  }
}

onMounted(() => {
  loadWorkspaceCards();
  loadWorkspaces();
  loadMyAuditStatus();
  loadQuickNav();
  loadCustomerCount();
  loadOpportunityCount();
  loadTodaySummary();
  window.addEventListener('pagehide', handlePageHide);
});

onBeforeUnmount(() => {
  window.removeEventListener('pagehide', handlePageHide);
  // 补交防抖中的待保存布局（原来直接 clearTimeout 会丢弃变更，拖完立刻刷新即丢失）
  flushPendingSave();
  if (gridStack) {
    rebuildingGrid = true;
    gridStack.destroy(false);
    gridStack = null;
  }
});
</script>

<template>
  <div class="workspace-page">
    <WorkbenchHeader
      :avatar="(userStore.userInfo as any)?.avatar || ''"
      :customer-count="customerCount"
      :opportunity-count="opportunityCount"
      :todo-processed="todoProcessed"
      :todo-total="todoTotal"
    >
      <template #title>{{ welcomeText }}</template>
      <template #description>
        <!-- 工作台切换器（方案 5.3-M3）：仅画布模式且存在多个有权限工作台时展示 -->
        <span
          v-if="cardModeEnabled && workspaceList.length > 1"
          class="mt-2 flex items-center gap-2"
        >
          <span class="text-sm">{{ $t('page.dashboard.workspace.switch') }}</span>
          <Select
            :value="currentWorkspaceCode"
            :options="workspaceList"
            class="w-40"
            size="small"
            @change="handleWorkspaceSwitch"
          />
        </span>
      </template>
    </WorkbenchHeader>

    <!-- 空态提示：注册卡全部不可见（方案 4.5-1），超管附去配置入口 -->
    <div v-if="showEmptyTip" class="empty-tip">
      <Empty :description="$t('page.dashboard.workspaceEmptyCards')" />
      <Button v-if="isSuperAdmin" type="primary" @click="goConfigure">
        {{ $t('page.dashboard.workspaceGoConfig') }}
      </Button>
    </div>

    <!-- 画布模式（方案 5.3-M1）：gridstack 12 列，拖拽/缩放/隐藏/复位 -->
    <div v-if="cardModeEnabled && !canvasFailed" class="canvas-wrap">
      <div
        v-if="useCanvasMode && gridDomItems.length > 0"
        class="canvas-toolbar"
      >
        <span class="canvas-tip">
          {{ $t('page.dashboard.canvas.tip') }}
        </span>
        <div class="canvas-actions">
          <Popover
            v-if="hiddenLayoutItems.length > 0"
            placement="bottomRight"
            trigger="click"
          >
            <template #content>
              <div class="restore-list">
                <div
                  v-for="item in hiddenLayoutItems"
                  :key="item.cardCode"
                  class="restore-item"
                  @click="restoreCard(item.cardCode)"
                >
                  <span>{{ cardTitle(item.cardCode) }}</span>
                  <span class="restore-action">
                    {{ $t('page.dashboard.canvas.restore') }}
                  </span>
                </div>
              </div>
            </template>
            <Button size="small">
              {{
                $t('page.dashboard.canvas.hiddenTip', {
                  n: hiddenLayoutItems.length,
                })
              }}
            </Button>
          </Popover>
          <Button size="small" @click="resetLayout">
            {{ $t('page.dashboard.canvas.reset') }}
          </Button>
        </div>
      </div>
      <div
        v-show="useCanvasMode && gridDomItems.length > 0"
        ref="canvasContainer"
        class="workspace-canvas"
      >
        <div class="grid-stack">
          <div
            v-for="item in gridDomItems"
            :key="item.cardCode"
            class="grid-stack-item"
            :gs-h="item.h"
            :gs-id="item.cardCode"
            :gs-max-h="getCanvasCardDef(item.cardCode)?.maxH"
            :gs-max-w="getCanvasCardDef(item.cardCode)?.maxW"
            :gs-min-h="getCanvasCardDef(item.cardCode)?.minH"
            :gs-min-w="getCanvasCardDef(item.cardCode)?.minW"
            :gs-w="item.w"
            :gs-x="item.x"
            :gs-y="item.y"
          >
            <div class="grid-stack-item-content">
              <component
                :is="getCanvasCardDef(item.cardCode)?.component"
                :ref="(el: any) => bindCardRef(el, item.cardCode)"
                v-bind="cardProps(item.cardCode)"
                v-on="cardEvents(item.cardCode)"
              />
              <Button
                class="card-hide-btn"
                size="small"
                type="text"
                @click="hideCard(item.cardCode)"
              >
                <IconifyIcon icon="lucide:x" class="size-3.5" />
              </Button>
            </div>
          </div>
        </div>
      </div>
      <!-- 画布初始化占位（加载个人布局与 gridstack 期间） -->
      <div
        v-if="cardModeEnabled && !canvasReady && !canvasFailed"
        class="canvas-init-placeholder"
      ></div>
    </div>

    <!-- 回退渲染（方案 3.4）：总开关关闭或画布初始化失败时保持两列布局 -->
    <div v-else class="workspace-fallback">
      <div class="fallback-col">
        <OnboardingCard
          v-if="showOnboardingCard"
          :ref="(el: any) => bindCardRef(el, WORKSPACE_CARD_CODES.onboarding)"
          @audit-change="handleAuditChange"
          @view-todos="handleViewTodos"
        />
        <TodoOverviewCard
          :ref="(el: any) => bindCardRef(el, WORKSPACE_CARD_CODES.todoOverview)"
          :visible-tabs="visibleOverviewTabs"
          @click-card="handleOverviewClick"
        />
        <WorkLogCard :refresh-key="workLogRefreshKey" />
      </div>
      <div class="fallback-col fallback-col-side">
        <div class="quick-nav-wrap">
          <WorkbenchQuickNav
            :items="quickNavItems"
            :title="$t('page.dashboard.quickNav')"
            @click="navTo"
          />
          <Popover placement="left" trigger="hover">
            <template #content>
              <span>{{ $t('page.dashboard.customQuickNav') }}</span>
            </template>
            <Button
              class="settings-btn"
              shape="circle"
              size="small"
              type="text"
              @click="navSettingsVisible = true"
            >
              <IconifyIcon icon="lucide:settings" class="size-4" />
            </Button>
          </Popover>
        </div>
        <SmartTodoCard
          :ref="(el: any) => bindCardRef(el, WORKSPACE_CARD_CODES.smartTodo)"
          @count-change="(n: number) => (todoCount = n)"
          @processed="handleSmartProcessed"
          @view-approval="handleViewApproval"
        />
        <WeekLoadCard
          :ref="(el: any) => bindCardRef(el, WORKSPACE_CARD_CODES.weekLoad)"
        />
      </div>
    </div>

    <!-- 内嵌审批抽屉与快捷导航设置 -->
    <OrderApprovalDrawer
      v-model:visible="orderApprovalVisible"
      :order-id="approvalDrawerOrderId"
      @success="handleApprovalSuccess"
    />
    <ContractApprovalDrawer
      v-model:visible="contractApprovalVisible"
      :contract-id="approvalDrawerContractId"
      :current-user-id="approvalCurrentUserId"
      @success="handleApprovalSuccess"
    />
    <QuickNavSettingsModal
      v-model:visible="navSettingsVisible"
      @saved="loadQuickNav"
    />
  </div>
</template>

<style lang="scss">
@import 'gridstack/dist/gridstack.css';
</style>

<style lang="scss" scoped>
.workspace-page {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
}

.empty-tip {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 48px 0;
}

// ===== 画布模式（方案 5.3-M1） =====
.canvas-wrap {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.canvas-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 4px;
}

.canvas-tip {
  color: hsl(var(--foreground) / 60%);
  font-size: 12px;
}

.canvas-actions {
  display: flex;
  gap: 8px;
}

.workspace-canvas {
  min-height: 200px;
}

.grid-stack-item-content {
  position: relative;
  height: 100%;
  overflow: hidden;

  > :first-child {
    height: 100%;
  }
}

.card-hide-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 20;
  opacity: 0;
  transition: opacity 0.2s;
}

.grid-stack-item:hover .card-hide-btn {
  opacity: 1;
}

.restore-list {
  min-width: 160px;
}

.restore-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 4px 0;
  cursor: pointer;

  &:hover .restore-action {
    color: hsl(var(--primary));
  }
}

.restore-action {
  color: hsl(var(--foreground) / 50%);
  font-size: 12px;
}

.canvas-init-placeholder {
  min-height: 120px;
}

// ===== 回退渲染（方案 3.4，旧两列布局） =====
.workspace-fallback {
  display: flex;
  gap: 16px;
}

.fallback-col {
  display: flex;
  flex: 2;
  flex-direction: column;
  gap: 16px;
  min-width: 0;

  &.fallback-col-side {
    flex: 1;
  }
}

.quick-nav-wrap {
  position: relative;

  .settings-btn {
    position: absolute;
    top: 12px;
    right: 12px;
    z-index: 10;
    opacity: 0;
    transition: opacity 0.2s;
  }

  &:hover .settings-btn {
    opacity: 1;
  }
}

@media (max-width: 900px) {
  .workspace-fallback {
    flex-direction: column;
  }
}
</style>