<script lang="ts" setup>
// 智能待办卡（二期 M1 抽取：原 index.vue 内联实现独立成卡，注册卡 workspace_smart_todo）
// 自包含：内部判定模块权限（无权限整卡不渲染）、内部托管 QuickProcessModal；
// 处理结果经 processed / view-approval 事件上抛（父级负责刷新工作日志与内嵌审批抽屉）
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import { useAccessStore, useUserStore } from '@vben/stores';

import { Button, Card, Empty, Spin, Tag } from 'ant-design-vue';

import {
  getTodoApprovalListApi,
  getTodoFollowUpListApi,
  getTodoPaymentListApi,
} from '#/api';
import { getPlanListApi } from '#/api/core/statistics';

import QuickProcessModal from '../../components/QuickProcessModal.vue';
import { useDashboardPermission } from '../config';

defineOptions({ name: 'WorkspaceSmartTodoCard' });

const emit = defineEmits<{
  (e: 'count-change', count: number): void;
  (e: 'processed'): void;
  (
    e: 'view-approval',
    payload: { businessId: number; businessType: string; instanceId: number },
  ): void;
}>();

const router = useRouter();
const userStore = useUserStore();
const accessStore = useAccessStore();
const { canShow } = useDashboardPermission();

// ===== 快捷新建入口（方案三期 #3：受各自 save 权限码守卫） =====
const canCreateCustomer = computed(() =>
  accessStore.hasAccessCode('crm:customer:save'),
);
const canCreateFollowup = computed(() =>
  accessStore.hasAccessCode('crm:followup:save'),
);
const canCreateOpportunity = computed(() =>
  accessStore.hasAccessCode('crm:opportunity:save'),
);
const showQuickCreate = computed(
  () =>
    canCreateCustomer.value || canCreateFollowup.value ||
    canCreateOpportunity.value,
);

function openQuickCreate(
  type: 'createCustomer' | 'createFollowup' | 'createOpportunity',
) {
  currentTodoItem.value = { type };
  quickProcessVisible.value = true;
}

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
const currentClickedTodo = ref<null | SmartTodoItem>(null);

// 智能待办：任一待办类型有权限则显示
const showSmartTodo = computed(
  () =>
    canShow('approval') ||
    canShow('followUp') ||
    canShow('payment') ||
    canShow('planApproval'),
);

// ===== 今日已处理待办缓存（localStorage，跨天自动清空） =====
const processedToday = ref<SmartTodoItem[]>([]);
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
  // 无任何待办类型权限：直接跳过，避免无效请求
  if (!showSmartTodo.value) {
    todoItems.value = [];
    todoTotalCount.value = 0;
    return;
  }
  todoLoading.value = true;
  try {
    const [approvalResp, followUpResp, paymentResp, planResp]: any[] =
      await Promise.all([
        canShow('approval')
          ? getTodoApprovalListApi({ pageNum: 1, pageSize: 5 }).catch(() => ({
              items: [],
              total: 0,
            }))
          : Promise.resolve({ items: [], total: 0 }),
        canShow('followUp')
          ? getTodoFollowUpListApi({
              pageNum: 1,
              pageSize: 5,
              rangeType: 'overdue',
            }).catch(() => ({ items: [], total: 0 }))
          : Promise.resolve({ items: [], total: 0 }),
        canShow('payment')
          ? getTodoPaymentListApi({ pageNum: 1, pageSize: 5, days: 7 }).catch(
              () => ({ items: [], total: 0 }),
            )
          : Promise.resolve({ items: [], total: 0 }),
        canShow('planApproval')
          ? getPlanListApi({
              pendingMyApproval: true,
              year: new Date().getFullYear(),
            }).catch(() => [])
          : Promise.resolve([]),
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
      let timeDesc: string;
      if (remainingDays < 0) {
        timeDesc = `已逾期 ${Math.abs(remainingDays)} 天`;
      } else if (remainingDays === 0) {
        timeDesc = '今日到期';
      } else {
        timeDesc = `还有 ${remainingDays} 天到期`;
      }
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
        totalContract >= 10_000
          ? `${(totalContract / 10_000).toFixed(1)}万`
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
    const pendingKeys = new Set(pendingItems.map((i) => `${i.type}-${i.id}`));
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
    businessId: item.type === 'approval' ? raw.businessId : raw.id,
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
  emit('processed');
}

watch(todoTotalCount, (count) => {
  emit('count-change', count);
});

onMounted(() => {
  // 初始化今日已处理待办缓存（跨天自动清空）
  processedToday.value = loadProcessedToday();
  loadSmartTodos();
});

defineExpose({ reload: loadSmartTodos });
</script>

<template>
  <Card
    v-if="showSmartTodo"
    class="flex h-full flex-col overflow-hidden"
    :body-style="{ flex: '1 1 0', minHeight: 0, overflowY: 'auto' }"
  >
    <template #title>
      <div class="flex w-full items-center justify-between gap-2">
        <div class="flex min-w-0 items-center gap-2">
          <span
            class="inline-block size-2 rounded-full bg-red-500"
            aria-hidden="true"
          ></span>
          <span>{{ $t('page.dashboard.todoList') }}</span>
        </div>
        <div
          v-if="showQuickCreate"
          class="flex shrink-0 items-center"
          @click.stop
        >
          <Button
            v-if="canCreateCustomer"
            size="small"
            type="link"
            class="px-1.5"
            @click="openQuickCreate('createCustomer')"
          >
            {{ $t('page.dashboard.quickCreateCustomer') }}
          </Button>
          <Button
            v-if="canCreateFollowup"
            size="small"
            type="link"
            class="px-1.5"
            @click="openQuickCreate('createFollowup')"
          >
            {{ $t('page.dashboard.quickCreateFollowUp') }}
          </Button>
          <Button
            v-if="canCreateOpportunity"
            size="small"
            type="link"
            class="px-1.5"
            @click="openQuickCreate('createOpportunity')"
          >
            {{ $t('page.dashboard.quickCreateOpportunity') }}
          </Button>
        </div>
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
    <QuickProcessModal
      v-model:visible="quickProcessVisible"
      :todo-item="currentTodoItem"
      @processed="handleProcessed"
      @view-approval="(p: any) => emit('view-approval', p)"
    />
  </Card>
</template>

<style scoped>
.todo-list {
  max-height: 360px;
  overflow-y: auto;
}

.todo-item + .todo-item {
  border-top: 1px dashed #f0f0f0;
}
</style>
