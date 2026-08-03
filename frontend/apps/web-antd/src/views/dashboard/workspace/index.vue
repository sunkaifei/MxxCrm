<script lang="ts" setup>
import type { WorkbenchQuickNavItem } from '@vben/common-ui';

import { computed, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import { WorkbenchHeader, WorkbenchQuickNav } from '@vben/common-ui';
import { preferences } from '@vben/preferences';
import { useUserStore } from '@vben/stores';
import { openWindow } from '@vben/utils';

import { Card, Empty, Spin, Tag } from 'ant-design-vue';

import {
  getCustomerListApi,
  getMenusRouterApi,
  getOpportunityListApi,
  getQuickNavPreferenceApi,
  getSaleSimpleModeApi,
  getTodaySummaryApi,
  getTodoApprovalListApi,
  getTodoFollowUpListApi,
  getTodoPaymentListApi,
  getWeekWorkloadApi,
} from '#/api';
import { getPlanListApi } from '#/api/core/statistics';
import type { QuickNavItem } from '#/api';
import { $t } from '#/locales';

import TodoOverviewCard from '../components/TodoOverviewCard.vue';
import WorkLogCard from '../components/WorkLogCard.vue';
import QuickNavSettingsModal from '../components/QuickNavSettingsModal.vue';
import QuickProcessModal from '../components/QuickProcessModal.vue';
// 审批流抽屉组件（复用业务模块现有组件，工作台内嵌打开）
import OrderApprovalDrawer from '../../sale/order/approval-drawer.vue';
import ContractApprovalDrawer from '../../crm/contract/approval-drawer.vue';

const router = useRouter();
const userStore = useUserStore();

// ===== 内嵌审批抽屉 =====
const approvalDrawerOrderId = ref<number | null>(null);
const approvalDrawerContractId = ref<number | null>(null);
const orderApprovalVisible = ref(false);
const contractApprovalVisible = ref(false);
const approvalCurrentUserId = computed(() => userStore.userInfo?.userId ? Number(userStore.userInfo.userId) : undefined);

// 处理 QuickProcessModal 的查看审批流详情事件：在工作台内嵌打开抽屉
function handleViewApproval(payload: {
  businessType: string;
  businessId: number;
  instanceId: number;
}) {
  const { businessType, businessId } = payload;
  switch (businessType) {
    case 'order': {
      approvalDrawerOrderId.value = businessId;
      orderApprovalVisible.value = true;
      break;
    }
    case 'contract': {
      approvalDrawerContractId.value = businessId;
      contractApprovalVisible.value = true;
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

// 审批抽屉操作成功后刷新待办列表
function handleApprovalSuccess() {
  loadSmartTodos();
  workLogRefreshKey.value++;
}

// ===== 工作日志刷新 key =====
const workLogRefreshKey = ref(0);

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
  return saleSimpleMode.value
    ? defaultQuickNavSimple
    : defaultQuickNavStandard;
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
    const sortedPref = [...savedPref].sort(
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
    quickNavItems.value =
      items.length > 0 ? items : getDefaultQuickNavItems();
  } catch {
    quickNavItems.value = getDefaultQuickNavItems();
  }
}

// ===== 智能待办 =====
interface SmartTodoItem {
  id: number;
  type: 'approval' | 'followUp' | 'payment' | 'planApproval';
  title: string;
  meta: string;
  color: string;
  badge?: number;
  raw: any;
  /** 已处理标记（当天保留显示，带删除线） */
  done?: boolean;
  processedAt?: string;
}

const todoLoading = ref(false);
const todoItems = ref<SmartTodoItem[]>([]);
const todoTotalCount = ref(0);
const quickProcessVisible = ref(false);
const currentTodoItem = ref<any>(null);
// 当前点击的待办项（用于处理完后标记已处理）
const currentClickedTodo = ref<SmartTodoItem | null>(null);

// 今日已处理待办缓存（跨天自动清空，当天保留显示删除线）
const processedToday = ref<SmartTodoItem[]>([]);

// ===== 今日已处理待办缓存（localStorage，跨天自动清空） =====
const PROCESSED_TODAY_KEY = computed(
  () => `todo_processed_${userStore.userInfo?.userId || 'guest'}`,
);

function getTodayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

function loadProcessedToday(): SmartTodoItem[] {
  try {
    const raw = localStorage.getItem(PROCESSED_TODAY_KEY.value);
    if (!raw) return [];
    const cache = JSON.parse(raw);
    // 跨天清空：日期不匹配则清除前一天已处理记录
    if (cache.date !== getTodayStr()) {
      localStorage.removeItem(PROCESSED_TODAY_KEY.value);
      return [];
    }
    return (cache.items || []).map((p: any) => ({ ...p, done: true }));
  } catch {
    return [];
  }
}

function saveProcessedToday(items: SmartTodoItem[]) {
  const compact = items.map((p) => ({
    id: p.id,
    type: p.type,
    title: p.title,
    meta: p.meta,
    color: p.color,
    processedAt: p.processedAt,
  }));
  localStorage.setItem(
    PROCESSED_TODAY_KEY.value,
    JSON.stringify({ date: getTodayStr(), items: compact }),
  );
}

function markAsProcessed(item: SmartTodoItem) {
  const exists = processedToday.value.some(
    (p) => p.type === item.type && p.id === item.id,
  );
  if (!exists) {
    processedToday.value.push({
      ...item,
      done: true,
      processedAt: new Date().toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
      }),
    });
    saveProcessedToday(processedToday.value);
  }
}

// ===== WorkbenchHeader 动态数据 =====
// 今日已处理数（来自 mxx_work_log 持久化，由后端聚合接口返回）
const todoProcessed = ref(0);
// 今日待办总数（= 已处理数 + 剩余待办数，由后端聚合接口返回）
const todoTotal = ref(0);
const customerCount = ref(0);
const opportunityCount = ref(0);

const todoCount = computed(() => todoTotalCount.value);

// 审批业务类型中文映射
const businessTypeMap: Record<string, string> = {
  order: '订单',
  quotation: '报价单',
  contract: '合同',
  payment: '回款',
  invoice: '发票',
  opportunity: '商机',
  customer: '客户',
};

async function loadSmartTodos() {
  todoLoading.value = true;
  try {
    const [approvalResp, followUpResp, paymentResp, planResp]: any[] =
      await Promise.all([
        getTodoApprovalListApi({ pageNum: 1, pageSize: 2 }).catch(
          () => ({ items: [], total: 0 }),
        ),
        getTodoFollowUpListApi({
          pageNum: 1,
          pageSize: 2,
          rangeType: 'overdue',
        }).catch(() => ({ items: [], total: 0 })),
        getTodoPaymentListApi({ pageNum: 1, pageSize: 2, days: 7 }).catch(
          () => ({ items: [], total: 0 }),
        ),
        getPlanListApi({
          pendingMyApproval: true,
          year: new Date().getFullYear(),
        }).catch(() => []),
      ]);

    const items: SmartTodoItem[] = [];

    // 审批待办
    const approvalItems = approvalResp?.items || [];
    approvalItems.forEach((item: any) => {
      const bizName = businessTypeMap[item.businessType] || '业务';
      const submitter = item.submitterName || '某人';
      const bizTitle = item.businessTitle || '';
      items.push({
        id: item.id,
        type: 'approval',
        title: bizTitle || `${bizName}审批`,
        meta: `由 ${submitter} 发起的${bizName} ${bizTitle} 审批流程，请尽快审核`,
        color: '#1890ff',
        raw: item,
      });
    });

    // 跟进待办
    const followUpItems = followUpResp?.items || [];
    followUpItems.forEach((item: any) => {
      const overdueDays = item.overdueDays || 0;
      const itemTypeText = item.itemType === 'lead' ? '线索' : '客户';
      items.push({
        id: item.id,
        type: 'followUp',
        title: item.name || `${itemTypeText}跟进`,
        meta: `该${itemTypeText}已逾期 ${overdueDays} 天未跟进，请尽快联系`,
        color: '#ff4d4f',
        raw: item,
      });
    });

    // 待回款
    const paymentItems = paymentResp?.items || [];
    paymentItems.forEach((item: any) => {
      const planAmount = item.planAmount || 0;
      const remainingDays = item.remainingDays ?? 0;
      const contractTitle = item.contractTitle || '';
      const stageName = item.stageName || '回款阶段';
      const timeDesc =
        remainingDays < 0
          ? `已逾期 ${Math.abs(remainingDays)} 天`
          : remainingDays === 0
            ? '今日到期'
            : `还有 ${remainingDays} 天到期`;
      items.push({
        id: item.id,
        type: 'payment',
        title: `${stageName} - ${contractTitle || '回款提醒'}`,
        meta: `计划回款 ¥${planAmount}，${timeDesc}，请尽快跟进回款`,
        color: '#13c2c2',
        raw: item,
      });
    });

    // 计划待审批（上级主管可见）
    const planItems = Array.isArray(planResp) ? planResp : planResp?.data || [];
    planItems.forEach((item: any) => {
      const empName = item.employeeName || '员工';
      const totalContract = Number(item.totalContractTarget || 0);
      const amtText =
        totalContract >= 10000
          ? `${(totalContract / 10000).toFixed(1)}万`
          : `${totalContract}`;
      items.push({
        id: item.id,
        type: 'planApproval',
        title: `${empName} ${item.year}年销售计划`,
        meta: `合同目标 ¥${amtText}，第${item.approvalLevel || 1}级/共${item.totalLevels || 1}级审批，请尽快审核`,
        color: '#722ed1',
        raw: item,
      });
    });

    // 汇总总数
    todoTotalCount.value =
      (approvalResp?.total || 0) +
      (followUpResp?.total || 0) +
      (paymentResp?.total || 0) +
      planItems.length;

    // 未处理项最多 5 条
    const pendingItems = items.slice(0, 5);
    // 今日已处理项：排除仍出现在未处理列表中的（防重复），最多追加 3 条
    const pendingKeys = new Set(
      pendingItems.map((i) => `${i.type}-${i.id}`),
    );
    const doneItems = processedToday.value
      .filter((p) => !pendingKeys.has(`${p.type}-${p.id}`))
      .slice(0, 3);
    todoItems.value = [...pendingItems, ...doneItems];
  } catch {
    todoItems.value = [];
    todoTotalCount.value = 0;
  } finally {
    todoLoading.value = false;
  }
}

function handleTodoClick(item: SmartTodoItem) {
  // 已处理项点击不触发操作
  if (item.done) return;
  // 计划待审批：跳转业绩页处理（在业绩页待审批抽屉中完成审批）
  if (item.type === 'planApproval') {
    router.push('/dashboard/performance').catch(() => {});
    return;
  }
  currentClickedTodo.value = item;
  const raw = item.raw || {};
  currentTodoItem.value = {
    ...raw,
    type: item.type,
    // 审批类型：raw.businessId 是业务ID（如订单ID），raw.id 是审批实例ID，不能覆盖
    // 其他类型：raw.id 即为业务ID，作为 businessId 传给快速处理弹窗
    businessId:
      item.type === 'approval' ? raw.businessId : raw.id,
    businessTitle: item.title,
  };
  quickProcessVisible.value = true;
}

function handleProcessed() {
  // 标记当前处理的待办为已处理（当天保留显示删除线，跨天自动清空）
  if (currentClickedTodo.value) {
    markAsProcessed(currentClickedTodo.value);
    currentClickedTodo.value = null;
  }
  loadSmartTodos();
  loadTodaySummary();
  workLogRefreshKey.value++;
}

// ===== 待办概览卡片点击 =====
const overviewRouteMap: Record<string, string> = {
  approval: '/system/approval/todo',
  followUp: '/crm/customer',
  payment: '/sale/payment',
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

// ===== 本周工作负载 =====
const weekLoading = ref(false);
const weekWorkload = ref<Array<{ day: string; count: number }>>([]);

const weekMaxCount = computed(() => {
  return Math.max(1, ...weekWorkload.value.map((w) => w.count || 0));
});

async function loadWeekWorkload() {
  weekLoading.value = true;
  try {
    const res: any = await getWeekWorkloadApi();
    if (Array.isArray(res)) {
      weekWorkload.value = res;
    } else if (Array.isArray(res?.items)) {
      weekWorkload.value = res.items;
    } else {
      weekWorkload.value = [];
    }
  } catch {
    weekWorkload.value = [];
  } finally {
    weekLoading.value = false;
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

// ===== WorkbenchHeader 动态数据加载 =====
// 加载客户总数
async function loadCustomerCount() {
  try {
    const res: any = await getCustomerListApi({ pageNum: 1, pageSize: 1 });
    customerCount.value = res?.total || 0;
  } catch {
    customerCount.value = 0;
  }
}

// 加载商机总数（自己的商机数量）
async function loadOpportunityCount() {
  try {
    const res: any = await getOpportunityListApi({ pageNum: 1, pageSize: 1 });
    opportunityCount.value = res?.total || 0;
  } catch {
    opportunityCount.value = 0;
  }
}

// 加载今日待办汇总（已处理数 + 总数 + 完成率，来自后端聚合接口）
// 已处理数来自 mxx_work_log 持久化，剩余数实时查询，总数 = 已处理 + 剩余
async function loadTodaySummary() {
  try {
    const res: any = await getTodaySummaryApi();
    todoProcessed.value = res?.todoProcessed || 0;
    todoTotal.value = res?.todoTotal || 0;
  } catch {
    todoProcessed.value = 0;
    todoTotal.value = 0;
  }
}

onMounted(() => {
  // 初始化今日已处理待办缓存（跨天自动清空）
  processedToday.value = loadProcessedToday();
  loadQuickNav();
  loadSmartTodos();
  loadWeekWorkload();
  loadCustomerCount();
  loadOpportunityCount();
  loadTodaySummary();
});
</script>

<template>
  <div class="p-5">
    <WorkbenchHeader
      :avatar="userStore.userInfo?.avatar || preferences.app.defaultAvatar"
      :customer-count="customerCount"
      :opportunity-count="opportunityCount"
      :todo-processed="todoProcessed"
      :todo-total="todoTotal"
    >
      <template #title>
        早安, {{ userStore.userInfo?.realName }}, 开始您一天的工作吧！
      </template>
      <template #description>
        {{ $t('page.dashboard.todoList') }} {{ todoCount }} 项
      </template>
    </WorkbenchHeader>

    <div class="mt-5 flex flex-col gap-5 lg:flex-row">
      <!-- 左列 3/5 -->
      <div class="flex w-full flex-col gap-5 lg:w-3/5">
        <TodoOverviewCard @click-card="handleOverviewClick" />
        <WorkLogCard :refresh-key="workLogRefreshKey" />
      </div>
      <!-- 右列 2/5 -->
      <div class="flex w-full flex-col gap-5 lg:w-2/5">
        <!-- 快捷导航 + 设置图标 -->
        <div class="relative">
          <WorkbenchQuickNav
            :items="quickNavItems"
            :title="$t('page.dashboard.quickNav')"
            @click="navTo"
          />
          <button
            class="settings-btn absolute right-3 top-3 inline-flex size-7 items-center justify-center rounded text-gray-400 opacity-0 transition hover:bg-gray-100 hover:text-gray-600"
            type="button"
            :title="$t('page.dashboard.settings')"
            @click="navSettingsVisible = true"
          >
            <svg
              class="size-4"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
              />
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
            </svg>
          </button>
        </div>

        <!-- 智能待办 -->
        <Card>
          <template #title>
            <div class="flex items-center gap-2">
              <span
                class="inline-block size-2 rounded-full bg-red-500"
                aria-hidden="true"
              ></span>
              <span>{{ $t('page.dashboard.todoList') }}</span>
            </div>
          </template>
          <Spin :spinning="todoLoading">
            <div v-if="todoItems.length > 0" class="todo-list">
              <div
                v-for="item in todoItems"
                :key="`${item.type}-${item.id}`"
                class="todo-item flex cursor-pointer items-start gap-3 py-3 transition hover:bg-gray-50"
                :class="{ 'opacity-60': item.done }"
                @click="handleTodoClick(item)"
              >
                <span
                  class="mt-1.5 inline-block size-2 shrink-0 rounded-full"
                  :style="{ background: item.color }"
                  aria-hidden="true"
                ></span>
                <div class="min-w-0 flex-1">
                  <div
                    class="truncate text-sm font-medium text-gray-800"
                    :class="{ 'line-through text-gray-400': item.done }"
                  >
                    {{ item.title }}
                  </div>
                  <div
                    class="mt-0.5 truncate text-xs text-gray-500"
                    :class="{ 'line-through': item.done }"
                  >
                    {{ item.meta }}
                  </div>
                </div>
                <Tag v-if="item.done" color="default" class="ml-2 shrink-0">
                  已处理
                </Tag>
                <Tag v-else-if="item.badge" color="red" class="ml-2 shrink-0">
                  {{ item.badge }}
                </Tag>
              </div>
            </div>
            <Empty
              v-else
              :image="Empty.PRESENTED_IMAGE_SIMPLE"
              description="暂无待办"
              class="py-8"
            />
          </Spin>
        </Card>

        <!-- 本周工作负载 -->
        <Card>
          <template #title>
            <div class="flex items-center gap-2">
              <span
                class="inline-block size-2 rounded-full bg-blue-500"
                aria-hidden="true"
              ></span>
              <span>{{ $t('page.dashboard.weekWorkload') }}</span>
            </div>
          </template>
          <Spin :spinning="weekLoading">
            <div
              v-if="weekWorkload.length > 0"
              class="week-chart flex h-40 items-end justify-between gap-2 px-2"
            >
              <div
                v-for="(w, idx) in weekWorkload"
                :key="idx"
                class="flex flex-1 flex-col items-center gap-1"
              >
                <div class="text-xs text-gray-500">{{ w.count || 0 }}</div>
                <div
                  class="w-full rounded-t transition-all duration-300"
                  :style="{
                    height: `${Math.max(
                      4,
                      ((w.count || 0) / weekMaxCount) * 110,
                    )}px`,
                    background: 'linear-gradient(180deg, #1890ff, #69c0ff)',
                  }"
                ></div>
                <div class="text-xs text-gray-600">{{ w.day }}</div>
              </div>
            </div>
            <Empty
              v-else
              :image="Empty.PRESENTED_IMAGE_SIMPLE"
              description="暂无数据"
              class="py-8"
            />
          </Spin>
        </Card>
      </div>
    </div>

    <!-- 快速处理弹窗 -->
    <QuickProcessModal
      v-model:visible="quickProcessVisible"
      :todo-item="currentTodoItem"
      @processed="handleProcessed"
      @view-approval="handleViewApproval"
    />
    <!-- 快捷导航设置弹窗 -->
    <QuickNavSettingsModal
      v-model:visible="navSettingsVisible"
      @saved="loadQuickNav"
    />

    <!-- 内嵌审批流抽屉：复用业务模块组件，避免跳转 -->
    <!-- 始终挂载（visible 初始 false 不显示），确保 watch visible 能正常触发 loadDetail -->
    <OrderApprovalDrawer
      v-model:visible="orderApprovalVisible"
      :order-id="approvalDrawerOrderId"
      :current-user-id="approvalCurrentUserId"
      @success="handleApprovalSuccess"
    />
    <ContractApprovalDrawer
      v-model:visible="contractApprovalVisible"
      :contract-id="approvalDrawerContractId"
      :current-user-id="approvalCurrentUserId"
      @success="handleApprovalSuccess"
    />
  </div>
</template>

<style scoped>
.todo-list {
  max-height: 360px;
  overflow-y: auto;
}

.todo-item + .todo-item {
  border-top: 1px dashed #f0f0f0;
}

/* 设置按钮：父级 hover 时显示 */
.relative:hover .settings-btn {
  opacity: 1;
}
</style>
