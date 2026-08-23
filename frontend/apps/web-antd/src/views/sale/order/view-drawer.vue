<script lang="ts" setup>
import type { TableColumnsType } from 'ant-design-vue';

import { computed, h, ref, watch } from 'vue';

import { useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Alert,
  Button,
  Card,
  Descriptions,
  DescriptionsItem,
  Drawer,
  Empty,
  Input,
  message,
  Modal,
  Select,
  Skeleton,
  Table,
  Tag,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';

import {
  cancelApprovalApi,
  getOrderApprovalDetailApi,
  getOrderApprovalFlowApi,
  getOrderInfoApi,
  submitOrderApi,
  updateOrderStatusApi,
} from '#/api';
import { searchUsersApi } from '#/api/core/message/chat';

import {
  approvalStatusColorMap,
  approvalStatusLabelMap,
  currencyLabelMap,
  currencySymbolMap,
  formatMoney,
  orderStatusColorMap,
  orderStatusLabelMap,
  paymentMethodLabelMap,
  paymentStatusColorMap,
  paymentStatusLabelMap,
  shippingMethodLabelMap,
} from './constants';

const props = defineProps<{
  orderId: null | number;
  visible: boolean;
}>();

const emit = defineEmits<{
  // 详情抽屉底部操作按钮触发的动作，父组件根据 type 决定调用哪个 handler（避免父组件重复实现）
  action: [type: string, row: any];
  refresh: [];
  'update:visible': [val: boolean];
}>();

const loading = ref(false);
const detail = ref<any>({});
const items = ref<any[]>([]);
const shipments = ref<any[]>([]);
const approvals = ref<any[]>([]);
const actionLoading = ref(false);

const userStore = useUserStore();
const currentUserId = computed(() => userStore.userInfo?.userId);

// 审批实例（撤销/抄送判断用）
const apprInstance = ref<any>(null);

// ===== 审批操作文本映射（与审批引擎 action 约定一致）=====
const approvalActionMap: Record<number, { color: string; label: string }> = {
  0: { label: '提交审批', color: 'processing' },
  1: { label: '审批通过', color: 'success' },
  2: { label: '驳回', color: 'error' },
  3: { label: '转办', color: 'blue' },
  4: { label: '委派', color: 'cyan' },
  5: { label: '加签', color: 'purple' },
  6: { label: '退回', color: 'orange' },
  7: { label: '撤销审批', color: 'warning' },
  8: { label: '抄送', color: 'geekblue' },
};

function approvalActionColor(action: any) {
  const num = Number(action);
  if (num === 1) return 'green';
  if (num === 2) return 'red';
  return 'blue';
}

// ===== 提交审批弹窗（含抄送）=====
const submitModalVisible = ref(false);
const ccUserIds = ref<number[]>([]);
const ccReason = ref('');
const submitLoading = ref(false);
const userOptions = ref<{ label: string; value: number }[]>([]);
const userSearching = ref(false);
let userSearchTimer: any = null;

function handleUserSearch(keyword: string) {
  if (userSearchTimer) clearTimeout(userSearchTimer);
  if (!keyword.trim()) {
    userOptions.value = [];
    return;
  }
  userSearchTimer = setTimeout(async () => {
    userSearching.value = true;
    try {
      const res: any = await searchUsersApi({
        keyword,
        page: 1,
        pageSize: 20,
      });
      const list: any[] = res.list || res || [];
      userOptions.value = list.map((u: any) => ({
        label:
          u.realName ||
          u.nickName ||
          u.userName ||
          u.username ||
          `用户${u.userId || u.id}`,
        value: u.userId || u.id,
      }));
    } catch {
      userOptions.value = [];
    } finally {
      userSearching.value = false;
    }
  }, 300);
}

function openSubmitModal() {
  ccUserIds.value = [];
  ccReason.value = '';
  userOptions.value = [];
  submitModalVisible.value = true;
}

// ===== 撤销审批弹窗 =====
const cancelModalVisible = ref(false);
const cancelReason = ref('');

// 是否可撤销：审批中（1/2）+ 当前用户为发起人
const canCancelApproval = computed(() => {
  if (![1, 2].includes(approvalStatusValue.value)) return false;
  if (!apprInstance.value || !currentUserId.value) return false;
  return Number(apprInstance.value.submitterId) === Number(currentUserId.value);
});

function openCancelModal() {
  cancelReason.value = '';
  cancelModalVisible.value = true;
}

// ===== 审批流预览 =====
const flowDetail = ref<any>(null);
const flowMissing = ref(false);
const approveModeMap: Record<number, string> = {
  1: '或签',
  2: '会签',
  3: '依次审批',
};

// 计算属性
const orderStatusValue = computed(() => Number(detail.value.orderStatus) || 0);
const approvalStatusValue = computed(
  () => Number(detail.value.approvalStatus) || 0,
);
const paymentStatusValue = computed(
  () => Number(detail.value.paymentStatus || detail.value.payStatus) || 0,
);
const currencySymbol = computed(
  () => currencySymbolMap[Number(detail.value.currency)] || '¥',
);
const currencyCode = computed(
  () => currencyLabelMap[Number(detail.value.currency)] || 'CNY',
);

// 金额
const productAmount = computed(() => Number(detail.value.productAmount || 0));
const discountAmount = computed(() => Number(detail.value.discountAmount || 0));
const taxAmount = computed(() => Number(detail.value.taxAmount || 0));
const shippingFee = computed(() => Number(detail.value.shippingFee || 0));
const otherFee = computed(() => Number(detail.value.otherFee || 0));
const totalAmount = computed(() => Number(detail.value.totalAmount || 0));
const paidAmount = computed(() => Number(detail.value.paidAmount || 0));
const unpaidAmount = computed(() =>
  Number(detail.value.unpaidAmount || totalAmount.value),
);

// 状态横幅渐变色
const bannerStyle = computed(() => {
  const status = orderStatusValue.value;
  const palettes: Record<number, string> = {
    1: 'linear-gradient(135deg, hsl(var(--muted) / 0.6), hsl(var(--muted) / 0.2))',
    2: 'linear-gradient(135deg, hsl(212 100% 45% / 0.08), hsl(212 100% 45% / 0.02))',
    3: 'linear-gradient(135deg, hsl(212 100% 45% / 0.10), hsl(212 100% 45% / 0.03))',
    4: 'linear-gradient(135deg, hsl(42 84% 61% / 0.12), hsl(42 84% 61% / 0.03))',
    5: 'linear-gradient(135deg, hsl(187 85% 43% / 0.10), hsl(187 85% 43% / 0.03))',
    6: 'linear-gradient(135deg, hsl(262 83% 58% / 0.10), hsl(262 83% 58% / 0.03))',
    7: 'linear-gradient(135deg, hsl(0 84% 60% / 0.12), hsl(0 84% 60% / 0.03))',
    8: 'linear-gradient(135deg, hsl(187 85% 43% / 0.10), hsl(187 85% 43% / 0.03))',
    9: 'linear-gradient(135deg, hsl(142 71% 45% / 0.12), hsl(142 71% 45% / 0.03))',
    10: 'linear-gradient(135deg, hsl(212 100% 45% / 0.10), hsl(212 100% 45% / 0.03))',
    11: 'linear-gradient(135deg, hsl(0 84% 60% / 0.14), hsl(0 84% 60% / 0.04))',
  };
  return palettes[status] || palettes[1];
});

// 操作权限（与 index.vue 操作栏一致）
const canEdit = computed(() => [0, 4].includes(approvalStatusValue.value));
const canSubmit = computed(() => [0, 4].includes(approvalStatusValue.value));
const canSignContract = computed(
  () => approvalStatusValue.value === 3 && !detail.value.contractId,
);
const canViewContract = computed(
  () => approvalStatusValue.value === 3 && !!detail.value.contractId,
);
const canVoid = computed(
  () =>
    approvalStatusValue.value === 3 &&
    !detail.value.contractId &&
    ![7, 11].includes(orderStatusValue.value),
);
const canStockUp = computed(() => orderStatusValue.value === 3);
const canShip = computed(() => [3, 4, 5].includes(orderStatusValue.value));
const canSign = computed(() => orderStatusValue.value === 6);
const canComplete = computed(() => orderStatusValue.value === 9);

// 产品明细列
const itemColumns: TableColumnsType = [
  {
    title: '#',
    width: 48,
    key: 'seq',
    align: 'center',
    customRender: ({ index }: any) => index + 1,
  },
  { title: '产品信息', key: 'product', width: 220 },
  { title: '规格', dataIndex: 'spec', width: 110, ellipsis: true },
  { title: '单位', dataIndex: 'unit', width: 60, align: 'center' },
  {
    title: '单价',
    key: 'unitPrice',
    width: 100,
    align: 'right',
    customRender: ({ text }: any) =>
      `${currencySymbol.value}${formatMoney(text)}`,
  },
  {
    title: '数量',
    dataIndex: 'quantity',
    width: 70,
    align: 'right',
    customRender: ({ text }: any) => Number(text || 0),
  },
  {
    title: '折扣',
    key: 'discountRate',
    width: 70,
    align: 'right',
    customRender: ({ text }: any) =>
      text && Number(text) < 100 ? `${Number(text)}%` : '-',
  },
  {
    title: '小计',
    key: 'amount',
    width: 110,
    align: 'right',
    customRender: ({ text }: any) =>
      h(
        'span',
        { class: 'font-medium' },
        `${currencySymbol.value}${formatMoney(text)}`,
      ),
  },
];

// ===== 审批流预览：从开始节点沿连线遍历，条件分支按当前金额直接命中，生成有序步骤序列 =====
const flowSteps = computed(() => {
  const flow = flowDetail.value;
  if (!flow) return [];
  const nodes: any[] = flow.nodes || [];
  const edges: any[] = flow.edges || [];
  if (!nodes.length) return [];
  const nodeMap = new Map(nodes.map((n: any) => [n.nodeKey, n]));
  const outMap = new Map<string, any[]>();
  for (const e of edges) {
    if (!outMap.has(e.source)) outMap.set(e.source, []);
    outMap.get(e.source)!.push(e);
  }
  const steps: any[] = [];
  const visited = new Set<string>();
  const start = nodes.find((n: any) => n.nodeType === 1) || nodes[0];
  const stack: { key: string; edge: any | null }[] = [
    { key: start.nodeKey, edge: null },
  ];
  while (stack.length) {
    const { key, edge } = stack.shift()!;
    if (visited.has(key)) continue;
    const node = nodeMap.get(key)!;
    visited.add(key);
    const outs = outMap.get(key) || [];
    if (node.nodeType === 3) {
      // 条件分支：不展示分支节点，按当前订单金额选择命中分支继续
      const matched = outs.find((e: any) => amountMatched(e));
      const chosen = matched || outs[0];
      if (chosen) stack.push({ key: chosen.target, edge: chosen });
      continue;
    }
    steps.push({ node, inEdge: edge, outs });
    if (outs.length) {
      stack.push({ key: outs[0].target, edge: outs[0] });
    }
  }
  return steps;
});

function flowNodeTitle(node: any): string {
  if (node.nodeType === 1) return '发起人';
  if (node.nodeType === 4) return '结束';
  return node.nodeName || (node.nodeType === 3 ? '条件分支' : '审批');
}

function flowNodeSubtitle(node: any): string {
  if (node.nodeType === 1) return '提交人自动通过';
  if (node.nodeType === 3) return '按条件分流';
  if (node.nodeType === 4) return '流程完成';
  const parts: string[] = [];
  if (node.approverType === 6) {
    const level = node.approverId || 1;
    parts.push(level === 1 ? '直属上级' : `上${level}级`);
  } else if (node.approverType === 7) {
    parts.push('部门主管链');
  } else {
    parts.push('指定审批人');
  }
  if (node.approveMode) {
    const modeLabel = approveModeMap[node.approveMode];
    if (modeLabel) parts.push(modeLabel);
  }
  return parts.join(' · ');
}

function flowNodeBoxClass(node: any): string {
  if (node.nodeType === 1) return 'border-green-300 bg-green-50';
  if (node.nodeType === 3) return 'border-amber-300 bg-amber-50';
  if (node.nodeType === 4) return 'border-gray-300 bg-gray-50';
  return 'border-blue-300 bg-blue-50';
}

function flowNodeTitleClass(node: any): string {
  if (node.nodeType === 1) return 'text-green-700';
  if (node.nodeType === 3) return 'text-amber-700';
  if (node.nodeType === 4) return 'text-gray-500';
  return 'text-blue-700';
}

// 简单条件求值：仅评估订单金额 amount 字段
function amountMatched(edge: any): boolean {
  const expr = edge?.conditionExpr;
  const amount = Number(totalAmount.value || 0);
  if (!expr) return true;
  const orParts = expr.split('||').map((s: string) => s.trim());
  return orParts.some((part: string) => {
    const andParts = part.split('&&').map((s: string) => s.trim());
    return andParts.every((single: string) => {
      const m = single.match(/^(\w+)\s*(<=|>=|==|!=|>|<)\s*([\d.]+)$/);
      if (!m || m[1] !== 'amount') return true;
      const [, , op, numStr] = m;
      const num = Number(numStr);
      const v = amount;
      switch (op) {
        case '<':
          return v < num;
        case '<=':
          return v <= num;
        case '>':
          return v > num;
        case '>=':
          return v >= num;
        case '==':
          return v === num;
        case '!=':
          return v !== num;
        default:
          return true;
      }
    });
  });
}

// ===== 提交审批 =====
function handleSubmitApproval() {
  openSubmitModal();
}

async function handleSubmitConfirm() {
  submitLoading.value = true;
  try {
    await submitOrderApi({
      id: Number(detail.value.id),
      ccUserIds: ccUserIds.value.length > 0 ? ccUserIds.value : undefined,
      ccReason: ccReason.value.trim() || undefined,
    });
    message.success('已提交审批');
    submitModalVisible.value = false;
    emit('refresh');
    await fetchDetail();
  } catch (error: any) {
    message.error(error?.message || '提交审批失败');
  } finally {
    submitLoading.value = false;
  }
}

// ===== 撤销审批（仅发起人）=====
async function handleCancelConfirm() {
  if (!cancelReason.value.trim()) {
    message.warning('请填写撤销理由');
    return;
  }
  actionLoading.value = true;
  try {
    await cancelApprovalApi({
      instanceId: Number(apprInstance.value?.id),
      cancelReason: cancelReason.value.trim(),
    });
    message.success('已撤销审批，订单恢复为草稿，可重新编辑');
    cancelModalVisible.value = false;
    emit('refresh');
    await fetchDetail();
  } catch (error: any) {
    message.error(error?.message || '撤销审批失败');
  } finally {
    actionLoading.value = false;
  }
}

// 加载详情
async function fetchDetail() {
  if (!props.orderId) return;
  loading.value = true;
  try {
    const res = await getOrderInfoApi(props.orderId);
    const data = res?.data ?? res ?? {};
    detail.value = data;
    items.value = Array.isArray(data.items) ? data.items : [];
    shipments.value = Array.isArray(data.shipments) ? data.shipments : [];

    // 拉取审批流预览（提交前 / 查看时均展示）
    flowMissing.value = false;
    try {
      const flowRes = (await getOrderApprovalFlowApi()) as any;
      flowDetail.value = flowRes?.data ?? flowRes ?? null;
      if (!flowDetail.value) flowMissing.value = true;
    } catch (e: any) {
      flowDetail.value = null;
      if (/审批流程/.test(e?.message || '')) flowMissing.value = true;
    }

    // 拉取审批记录
    if ([1, 2, 3, 4].includes(Number(data.approvalStatus))) {
      try {
        const apprRes = await getOrderApprovalDetailApi(props.orderId);
        const apprData = (apprRes?.data ?? apprRes) as any;
        const inst = apprData?.instance ?? null;
        apprInstance.value = inst;
        // 审批记录来自审批实例日志（logs），按时间正序
        approvals.value = Array.isArray(inst?.logs) ? inst.logs : [];
      } catch {
        apprInstance.value = null;
        approvals.value = [];
      }
    } else {
      apprInstance.value = null;
      approvals.value = [];
    }
  } finally {
    loading.value = false;
  }
}

// 操作函数：通过 emit action 事件让父组件处理，避免重复实现
function emitAction(type: string) {
  emit('action', type, detail.value);
}

// 直接在本抽屉处理的简单动作：作废
function handleVoid() {
  let remark = '';
  Modal.confirm({
    title: '订单作废',
    content: () =>
      h('div', [
        h('p', { class: 'mb-2' }, '作废后订单将无法恢复，请确认操作。'),
        h('textarea', {
          class: 'w-full rounded border p-2 text-sm outline-none border-solid',
          style:
            'border-color: hsl(var(--border)); background: hsl(var(--background)); color: hsl(var(--foreground));',
          placeholder: '请输入作废原因（必填）',
          rows: 3,
          onInput: (e: any) => {
            remark = e.target.value;
          },
        }),
      ]),
    okText: '确认作废',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      if (!remark.trim()) {
        message.warning('请输入作废原因');
        throw new Error('请输入作废原因');
      }
      actionLoading.value = true;
      try {
        await updateOrderStatusApi({
          id: Number(detail.value.id),
          orderStatus: 11,
          remark,
        });
        message.success('订单已作废');
        await fetchDetail();
        emit('refresh');
      } catch (error: any) {
        message.error(error?.message || '作废失败');
      } finally {
        actionLoading.value = false;
      }
    },
  });
}

// 直接在本抽屉处理：状态流转（发货/签收/完成/备货）
function handleStatusUpdate(status: number, label: string) {
  Modal.confirm({
    title: `订单${label}`,
    content: `确认将订单状态更新为"${label}"？`,
    onOk: async () => {
      actionLoading.value = true;
      try {
        await updateOrderStatusApi({
          id: Number(detail.value.id),
          orderStatus: status,
        });
        message.success(`已更新为${label}`);
        await fetchDetail();
        emit('refresh');
      } catch (error: any) {
        message.error(error?.message || '操作失败');
      } finally {
        actionLoading.value = false;
      }
    },
  });
}

function close() {
  emit('update:visible', false);
}

// 监听 visible + orderId 变化加载
watch(
  () => [props.visible, props.orderId],
  ([v, id]) => {
    if (v && id) {
      fetchDetail();
    } else if (!v) {
      // 关闭时重置
      detail.value = {};
      items.value = [];
      shipments.value = [];
      approvals.value = [];
      apprInstance.value = null;
      flowDetail.value = null;
      flowMissing.value = false;
    }
  },
  { immediate: true },
);
</script>

<template>
  <Drawer
    :open="visible"
    title="订单详情"
    :width="1200"
    placement="right"
    :destroy-on-close="true"
    :mask-closable="true"
    :body-style="{
      padding: 0,
      overflow: 'auto',
      background: 'hsl(var(--muted) / 30%)',
    }"
    :header-style="{
      padding: '16px 24px',
      borderBottom: '1px solid hsl(var(--border))',
    }"
    @close="close"
  >
    <Skeleton v-if="loading" active :paragraph="{ rows: 8 }" class="p-6" />

    <div v-else class="view-drawer">
      <!-- 状态横幅区 -->
      <section class="view-drawer__banner" :style="{ background: bannerStyle }">
        <div class="view-drawer__banner-main">
          <div class="view-drawer__banner-title-row">
            <h1 class="view-drawer__banner-title">{{ detail.title || '-' }}</h1>
            <div class="view-drawer__banner-tags">
              <Tag
                v-if="orderStatusValue"
                :color="orderStatusColorMap[orderStatusValue]"
                class="!m-0"
              >
                {{ orderStatusLabelMap[orderStatusValue] }}
              </Tag>
              <Tag
                :color="approvalStatusColorMap[approvalStatusValue]"
                class="!m-0"
              >
                审批: {{ approvalStatusLabelMap[approvalStatusValue] }}
              </Tag>
              <Tag
                v-if="paymentStatusValue"
                :color="paymentStatusColorMap[paymentStatusValue]"
                class="!m-0"
              >
                {{ paymentStatusLabelMap[paymentStatusValue] }}
              </Tag>
            </div>
          </div>
          <div class="view-drawer__banner-meta">
            <div class="view-drawer__banner-meta-item">
              <span class="view-drawer__banner-meta-label">订单号</span>
              <span class="view-drawer__banner-meta-value">{{
                detail.orderNo || '-'
              }}</span>
            </div>
            <div class="view-drawer__banner-meta-item">
              <span class="view-drawer__banner-meta-label">客户</span>
              <span class="view-drawer__banner-meta-value">{{
                detail.customerName || '-'
              }}</span>
            </div>
            <div class="view-drawer__banner-meta-item">
              <span class="view-drawer__banner-meta-label">下单日期</span>
              <span class="view-drawer__banner-meta-value">{{
                detail.orderDate || '-'
              }}</span>
            </div>
            <div class="view-drawer__banner-meta-item">
              <span class="view-drawer__banner-meta-label">负责人</span>
              <span class="view-drawer__banner-meta-value">{{
                detail.ownerUserName || '-'
              }}</span>
            </div>
          </div>
        </div>
        <div class="view-drawer__banner-amount">
          <div class="view-drawer__banner-amount-label">订单总额</div>
          <div class="view-drawer__banner-amount-value">
            <span class="view-drawer__banner-amount-symbol">{{
              currencySymbol
            }}</span>
            <span class="view-drawer__banner-amount-number">{{
              formatMoney(totalAmount)
            }}</span>
          </div>
          <div class="view-drawer__banner-amount-currency">
            {{ currencyCode }}
          </div>
        </div>
      </section>

      <!-- KPI 金额指标 -->
      <section class="view-drawer__kpi">
        <div class="view-drawer__kpi-item">
          <div class="view-drawer__kpi-label">商品总额</div>
          <div class="view-drawer__kpi-value">
            {{ currencySymbol }}{{ formatMoney(productAmount) }}
          </div>
        </div>
        <div class="view-drawer__kpi-item">
          <div class="view-drawer__kpi-label">折扣</div>
          <div
            class="view-drawer__kpi-value"
            :class="{ 'view-drawer__kpi-value--minus': discountAmount > 0 }"
          >
            {{ discountAmount > 0 ? '-' : '' }}{{ currencySymbol
            }}{{ formatMoney(discountAmount) }}
          </div>
        </div>
        <div class="view-drawer__kpi-item">
          <div class="view-drawer__kpi-label">税额</div>
          <div class="view-drawer__kpi-value view-drawer__kpi-value--warning">
            {{ currencySymbol }}{{ formatMoney(taxAmount) }}
          </div>
        </div>
        <div class="view-drawer__kpi-item">
          <div class="view-drawer__kpi-label">运费</div>
          <div class="view-drawer__kpi-value">
            {{ currencySymbol }}{{ formatMoney(shippingFee) }}
          </div>
        </div>
        <div class="view-drawer__kpi-item">
          <div class="view-drawer__kpi-label">其他费用</div>
          <div class="view-drawer__kpi-value">
            {{ currencySymbol }}{{ formatMoney(otherFee) }}
          </div>
        </div>
        <div class="view-drawer__kpi-item view-drawer__kpi-item--paid">
          <div class="view-drawer__kpi-label">已收款</div>
          <div class="view-drawer__kpi-value view-drawer__kpi-value--success">
            {{ currencySymbol }}{{ formatMoney(paidAmount) }}
          </div>
        </div>
        <div class="view-drawer__kpi-item view-drawer__kpi-item--unpaid">
          <div class="view-drawer__kpi-label">未收款</div>
          <div
            class="view-drawer__kpi-value"
            :class="{
              'view-drawer__kpi-value--danger': unpaidAmount > 0,
              'view-drawer__kpi-value--success': unpaidAmount <= 0,
            }"
          >
            {{ currencySymbol }}{{ formatMoney(unpaidAmount) }}
          </div>
        </div>
      </section>

      <!-- 主体两栏 -->
      <div class="view-drawer__body">
        <!-- 左侧主信息 -->
        <div class="view-drawer__main">
          <!-- 基本信息 -->
          <Card size="small" class="view-drawer__card">
            <template #title>
              <div class="view-drawer__card-title">
                <span class="view-drawer__card-title-bar"></span>基本信息
              </div>
            </template>
            <Descriptions :column="2" size="small" :colon="false">
              <DescriptionsItem label="订单标题">
                {{ detail.title || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="订单编号">
                {{ detail.orderNo || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="客户名称">
                {{ detail.customerName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="联系人">
                {{ detail.contactName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="下单日期">
                {{ detail.orderDate || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="交货日期">
                {{ detail.deliveryDate || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="币种">
                {{ currencyCode }}
                <span v-if="detail.exchangeRate" class="text-xs opacity-60">
                  (汇率: {{ detail.exchangeRate }})
                </span>
              </DescriptionsItem>
              <DescriptionsItem label="负责人">
                {{ detail.ownerUserName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="创建时间">
                {{ formatDateTime(detail.createTime) }}
              </DescriptionsItem>
              <DescriptionsItem label="更新时间">
                {{ formatDateTime(detail.updateTime) }}
              </DescriptionsItem>
            </Descriptions>
          </Card>

          <!-- 产品明细 -->
          <Card size="small" class="view-drawer__card">
            <template #title>
              <div class="view-drawer__card-title">
                <span class="view-drawer__card-title-bar"></span>产品明细
                <span class="view-drawer__card-title-count">
                  共 {{ items.length }} 项
                </span>
              </div>
            </template>
            <Empty v-if="items.length === 0" description="暂无产品明细" />
            <Table
              v-else
              :columns="itemColumns"
              :data-source="items"
              :pagination="false"
              size="small"
              :scroll="{ x: 900 }"
              row-key="id"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'product'">
                  <div class="view-drawer__product-cell">
                    <div class="view-drawer__product-name">
                      {{ record.productName || '-' }}
                    </div>
                    <div
                      v-if="record.productCode || record.sku"
                      class="view-drawer__product-code"
                    >
                      {{ record.productCode
                      }}<span v-if="record.productCode && record.sku"> · </span>
                      <span v-if="record.sku">SKU: {{ record.sku }}</span>
                    </div>
                  </div>
                </template>
              </template>
            </Table>

            <!-- 金额汇总 -->
            <div v-if="items.length > 0" class="view-drawer__summary">
              <div class="view-drawer__summary-row">
                <span>商品总额</span>
                <span
                  >{{ currencySymbol }}{{ formatMoney(productAmount) }}</span
                >
              </div>
              <div class="view-drawer__summary-row">
                <span>折扣</span>
                <span class="view-drawer__summary-row--minus">
                  -{{ currencySymbol }}{{ formatMoney(discountAmount) }}
                </span>
              </div>
              <div class="view-drawer__summary-row">
                <span>税额</span>
                <span>{{ currencySymbol }}{{ formatMoney(taxAmount) }}</span>
              </div>
              <div class="view-drawer__summary-row">
                <span>运费</span>
                <span>{{ currencySymbol }}{{ formatMoney(shippingFee) }}</span>
              </div>
              <div class="view-drawer__summary-row">
                <span>其他费用</span>
                <span>{{ currencySymbol }}{{ formatMoney(otherFee) }}</span>
              </div>
              <div
                class="view-drawer__summary-row view-drawer__summary-row--total"
              >
                <span>订单总计</span>
                <span>
                  {{ currencySymbol
                  }}<strong>{{ formatMoney(totalAmount) }}</strong>
                </span>
              </div>
            </div>
          </Card>

          <!-- 财务信息 -->
          <Card size="small" class="view-drawer__card">
            <template #title>
              <div class="view-drawer__card-title">
                <span class="view-drawer__card-title-bar"></span>财务信息
              </div>
            </template>
            <div class="view-drawer__finance">
              <div class="view-drawer__finance-block">
                <div class="view-drawer__finance-block-title">
                  付款方（客户）
                </div>
                <Descriptions :column="1" size="small" :colon="false">
                  <DescriptionsItem label="公司名称">
                    {{ detail.buyerCompanyName || '-' }}
                  </DescriptionsItem>
                  <DescriptionsItem label="账户名称">
                    {{ detail.buyerAccountName || '-' }}
                  </DescriptionsItem>
                  <DescriptionsItem label="开户行">
                    {{ detail.buyerBankName || '-' }}
                  </DescriptionsItem>
                  <DescriptionsItem label="账号">
                    {{ detail.buyerAccountNumber || '-' }}
                  </DescriptionsItem>
                </Descriptions>
              </div>
              <div class="view-drawer__finance-block">
                <div class="view-drawer__finance-block-title">
                  收款方（我方）
                </div>
                <Descriptions :column="1" size="small" :colon="false">
                  <DescriptionsItem label="公司名称">
                    {{ detail.sellerCompanyName || '-' }}
                  </DescriptionsItem>
                  <DescriptionsItem label="账户名称">
                    {{ detail.sellerAccountName || '-' }}
                  </DescriptionsItem>
                  <DescriptionsItem label="开户行">
                    {{ detail.sellerBankName || '-' }}
                  </DescriptionsItem>
                  <DescriptionsItem label="账号">
                    {{ detail.sellerAccountNumber || '-' }}
                  </DescriptionsItem>
                </Descriptions>
              </div>
            </div>
            <Descriptions :column="2" size="small" :colon="false" class="mt-3">
              <DescriptionsItem label="支付方式">
                {{ paymentMethodLabelMap[Number(detail.paymentMethod)] || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="支付截止日期">
                {{ detail.paymentDueDate || '-' }}
              </DescriptionsItem>
            </Descriptions>
          </Card>

          <!-- 收发货信息 -->
          <Card size="small" class="view-drawer__card">
            <template #title>
              <div class="view-drawer__card-title">
                <span class="view-drawer__card-title-bar"></span>收发货信息
              </div>
            </template>
            <Descriptions :column="2" size="small" :colon="false">
              <DescriptionsItem label="配送方式">
                {{
                  shippingMethodLabelMap[Number(detail.shippingMethod)] || '-'
                }}
              </DescriptionsItem>
              <DescriptionsItem label="物流单号">
                {{ detail.trackingNo || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="收货人">
                {{ detail.receiverName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="收货电话">
                {{ detail.receiverPhone || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="收货地址" :span="2">
                {{ detail.shippingAddress || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="账单地址" :span="2">
                {{ detail.billingAddress || '-' }}
              </DescriptionsItem>
            </Descriptions>
            <div v-if="shipments.length > 0" class="view-drawer__shipments">
              <div class="view-drawer__shipments-title">
                发货记录 ({{ shipments.length }})
              </div>
              <Timeline>
                <TimelineItem v-for="s in shipments" :key="s.id" color="blue">
                  <div class="view-drawer__shipment-item">
                    <span class="font-medium">{{
                      s.shipmentNo || s.trackingNo || '-'
                    }}</span>
                    <span class="text-xs opacity-60">
                      {{ formatDateTime(s.shippingTime || s.createTime) }}
                    </span>
                  </div>
                  <div v-if="s.trackingNo" class="text-xs opacity-70">
                    物流单号: {{ s.trackingNo }}
                  </div>
                </TimelineItem>
              </Timeline>
            </div>
          </Card>

          <!-- 审批流程 -->
          <Card size="small" class="view-drawer__card">
            <template #title>
              <div class="view-drawer__card-title">
                <span class="view-drawer__card-title-bar"></span>审批流程
                <span
                  v-if="flowDetail"
                  class="view-drawer__card-title-count"
                >
                  （{{ flowDetail.flowName }}）
                </span>
              </div>
            </template>
            <Alert
              v-if="flowMissing"
              type="warning"
              show-icon
              message="未找到启用的订单审批流程"
              description="请联系管理员在系统-审批流管理中对订单审批流程进行配置后再提交。"
            />
            <div v-else-if="flowSteps.length > 0" class="view-drawer__flow">
              <div class="view-drawer__flow-chain">
                <template
                  v-for="(step, idx) in flowSteps"
                  :key="step.node.nodeKey"
                >
                  <!-- 前置箭头 -->
                  <span v-if="idx > 0" class="view-drawer__flow-arrow">→</span>
                  <!-- 节点卡片 -->
                  <div
                    class="view-drawer__flow-node"
                    :class="flowNodeBoxClass(step.node)"
                  >
                    <div
                      class="view-drawer__flow-node-title"
                      :class="flowNodeTitleClass(step.node)"
                    >
                      {{ flowNodeTitle(step.node) }}
                    </div>
                    <div class="view-drawer__flow-node-sub">
                      {{ flowNodeSubtitle(step.node) }}
                    </div>
                  </div>
                </template>
              </div>
            </div>
            <Empty v-else description="暂无审批流程" />
          </Card>

          <!-- 审批进度 -->
          <Card size="small" class="view-drawer__card">
            <template #title>
              <div class="view-drawer__card-title">
                <span class="view-drawer__card-title-bar"></span>审批进度
              </div>
            </template>
            <Empty v-if="approvals.length === 0" description="暂无审批记录" />
            <Timeline v-else>
              <TimelineItem
                v-for="(item, idx) in approvals"
                :key="idx"
                :color="approvalActionColor(item.action)"
              >
                <div class="view-drawer__approval-header">
                  <Tag
                    :color="approvalActionMap[item.action]?.color || 'default'"
                  >
                    {{ approvalActionMap[item.action]?.label || '操作' }}
                  </Tag>
                  <span class="text-xs opacity-60">
                    {{ formatDateTime(item.createTime) }}
                  </span>
                </div>
                <div v-if="item.nodeName" class="mt-1 text-sm">
                  节点：{{ item.nodeName }}
                </div>
                <div class="mt-1 text-sm">
                  操作人：{{ item.approverName || '-' }}
                </div>
                <div v-if="item.comment" class="mt-1 text-sm">
                  意见：{{ item.comment }}
                </div>
              </TimelineItem>
            </Timeline>
          </Card>

          <!-- 备注 -->
          <Card v-if="detail.remark" size="small" class="view-drawer__card">
            <template #title>
              <div class="view-drawer__card-title">
                <span class="view-drawer__card-title-bar"></span>备注
              </div>
            </template>
            <p class="view-drawer__remark">{{ detail.remark }}</p>
          </Card>
        </div>

        <!-- 右侧侧边栏 -->
        <aside class="view-drawer__side">
          <!-- 关联单据 -->
          <Card size="small" class="view-drawer__side-card">
            <template #title>
              <div class="view-drawer__card-title">
                <span class="view-drawer__card-title-bar"></span>关联单据
              </div>
            </template>
            <div class="view-drawer__related">
              <div
                v-if="detail.opportunityId"
                class="view-drawer__related-item view-drawer__related-item--opportunity"
                @click="emitAction('viewOpportunity')"
              >
                <div class="view-drawer__related-icon">商</div>
                <div class="view-drawer__related-info">
                  <div class="view-drawer__related-type">商机</div>
                  <div class="view-drawer__related-name">
                    {{ detail.opportunityName || `#${detail.opportunityId}` }}
                  </div>
                </div>
                <span class="view-drawer__related-arrow">›</span>
              </div>
              <div
                v-if="detail.quotationId"
                class="view-drawer__related-item view-drawer__related-item--quotation"
                @click="emitAction('viewQuotation')"
              >
                <div class="view-drawer__related-icon">报</div>
                <div class="view-drawer__related-info">
                  <div class="view-drawer__related-type">报价单</div>
                  <div class="view-drawer__related-name">
                    {{
                      detail.quotationTitle ||
                      detail.quotationNo ||
                      `#${detail.quotationId}`
                    }}
                  </div>
                </div>
                <span class="view-drawer__related-arrow">›</span>
              </div>
              <div
                v-if="detail.contractId"
                class="view-drawer__related-item view-drawer__related-item--contract"
                @click="emitAction('viewContract')"
              >
                <div class="view-drawer__related-icon">合</div>
                <div class="view-drawer__related-info">
                  <div class="view-drawer__related-type">合同</div>
                  <div class="view-drawer__related-name">
                    {{
                      detail.contractTitle ||
                      detail.contractNo ||
                      `#${detail.contractId}`
                    }}
                  </div>
                </div>
                <span class="view-drawer__related-arrow">›</span>
              </div>
              <div
                v-if="
                  !detail.opportunityId &&
                  !detail.quotationId &&
                  !detail.contractId
                "
                class="view-drawer__related-empty"
              >
                暂无关联单据
              </div>
            </div>
          </Card>

          <!-- 系统信息 -->
          <Card size="small" class="view-drawer__side-card">
            <template #title>
              <div class="view-drawer__card-title">
                <span class="view-drawer__card-title-bar"></span>系统信息
              </div>
            </template>
            <Descriptions :column="1" size="small" :colon="false">
              <DescriptionsItem label="订单ID">
                {{ detail.id || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="创建人">
                {{ detail.createByName || detail.createBy || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="创建时间">
                {{ formatDateTime(detail.createTime) }}
              </DescriptionsItem>
              <DescriptionsItem label="更新人">
                {{ detail.updateByName || detail.updateBy || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="更新时间">
                {{ formatDateTime(detail.updateTime) }}
              </DescriptionsItem>
              <DescriptionsItem label="审批实例">
                <span v-if="detail.instanceId" class="text-xs opacity-70">
                  #{{ detail.instanceId }}
                </span>
                <span v-else>-</span>
              </DescriptionsItem>
            </Descriptions>
          </Card>

          <!-- 状态流转提示 -->
          <Card
            v-if="orderStatusValue === 11"
            size="small"
            class="view-drawer__side-card view-drawer__side-card--alert"
          >
            <div class="view-drawer__alert">
              <div class="view-drawer__alert-icon">⚠</div>
              <div>
                <div class="view-drawer__alert-title">订单已作废</div>
                <div class="view-drawer__alert-desc">
                  此订单已作废，无法继续操作。
                </div>
              </div>
            </div>
          </Card>
        </aside>
      </div>
    </div>

    <!-- 底部操作栏 -->
    <template #footer>
      <div class="view-drawer__footer">
        <div class="view-drawer__footer-left">
          <Button @click="close">关闭</Button>
        </div>
        <div class="view-drawer__footer-actions">
          <Button
            v-if="canEdit"
            type="primary"
            :loading="actionLoading"
            @click="emitAction('edit')"
          >
            编辑
          </Button>
          <Button
            v-if="canSubmit"
            :loading="actionLoading"
            @click="handleSubmitApproval"
          >
            提交审批
          </Button>
          <Button
            v-if="canCancelApproval"
            danger
            :loading="actionLoading"
            @click="openCancelModal"
          >
            撤销审批
          </Button>
          <Button
            v-if="canSignContract"
            type="primary"
            @click="emitAction('signContract')"
          >
            签署合同
          </Button>
          <Button v-if="canViewContract" @click="emitAction('viewContract')">
            查看合同
          </Button>
          <Button
            v-if="canStockUp"
            :loading="actionLoading"
            @click="handleStatusUpdate(4, '备货中')"
          >
            备货
          </Button>
          <Button
            v-if="canShip"
            :loading="actionLoading"
            @click="emitAction('ship')"
          >
            发货
          </Button>
          <Button
            v-if="canSign"
            :loading="actionLoading"
            @click="handleStatusUpdate(9, '已签收')"
          >
            签收
          </Button>
          <Button
            v-if="canComplete"
            :loading="actionLoading"
            @click="handleStatusUpdate(10, '已完成')"
          >
            完成
          </Button>
          <Button
            v-if="canVoid"
            danger
            :loading="actionLoading"
            @click="handleVoid"
          >
            作废
          </Button>
        </div>
      </div>
    </template>
  </Drawer>

  <!-- 提交审批确认（含抄送） -->
  <Modal
    v-model:visible="submitModalVisible"
    title="提交审批"
    :confirm-loading="submitLoading"
    ok-text="确认提交"
    cancel-text="取消"
    @ok="handleSubmitConfirm"
  >
    <div class="py-1">
      <p class="mb-2">提交后订单将进入审批流程，审批通过后方可继续后续操作。</p>
      <p class="mb-3 text-sm">
        <span class="text-gray-500">订单金额：</span>
        <span class="font-semibold"
          >{{ currencyCode }} {{ formatMoney(totalAmount) }}</span
        >
      </p>
      <div class="mb-2 text-sm">抄送人（可选）：</div>
      <Select
        v-model:value="ccUserIds"
        mode="multiple"
        :filter-option="false"
        :loading="userSearching"
        :options="userOptions"
        placeholder="搜索并选择抄送人"
        class="w-full"
        @search="handleUserSearch"
      />
      <div class="mb-1 mt-3 text-sm">抄送说明（可选）：</div>
      <Input.TextArea
        v-model:value="ccReason"
        :rows="2"
        placeholder="请填写抄送说明（可选）"
      />
    </div>
  </Modal>

  <!-- 撤销审批确认 -->
  <Modal
    v-model:visible="cancelModalVisible"
    title="撤销审批"
    :confirm-loading="actionLoading"
    ok-text="确认撤销"
    ok-type="danger"
    cancel-text="取消"
    @ok="handleCancelConfirm"
  >
    <div class="py-1">
      <p class="mb-2 text-sm">
        撤销后订单将恢复为草稿，可重新编辑并再次提交审批。已产生的审批记录将保留。
      </p>
      <div class="mb-1 text-sm">撤销理由（必填）：</div>
      <Input.TextArea
        v-model:value="cancelReason"
        :rows="3"
        placeholder="请填写撤销理由"
      />
    </div>
  </Modal>
</template>

<style scoped>
.view-drawer {
  padding: 16px 24px 24px;
}

/* ===== 状态横幅 ===== */
.view-drawer__banner {
  position: relative;
  display: flex;
  gap: 24px;
  align-items: stretch;
  justify-content: space-between;
  padding: 20px 24px;
  overflow: hidden;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
}

.view-drawer__banner-main {
  flex: 1;
  min-width: 0;
}

.view-drawer__banner-title-row {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
}

.view-drawer__banner-title {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  line-height: 1.3;
  color: hsl(var(--foreground));
  letter-spacing: -0.01em;
}

.view-drawer__banner-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.view-drawer__banner-meta {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.view-drawer__banner-meta-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.view-drawer__banner-meta-label {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.view-drawer__banner-meta-value {
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--foreground));
}

.view-drawer__banner-amount {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: center;
  min-width: 200px;
  padding-left: 24px;
  border-left: 1px solid hsl(var(--border));
}

.view-drawer__banner-amount-label {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.view-drawer__banner-amount-value {
  display: flex;
  gap: 4px;
  align-items: baseline;
  margin-top: 4px;
  color: hsl(var(--primary));
}

.view-drawer__banner-amount-symbol {
  font-size: 16px;
  font-weight: 600;
  opacity: 0.8;
}

.view-drawer__banner-amount-number {
  font-size: 28px;
  font-weight: 800;
  line-height: 1;
  letter-spacing: -0.02em;
}

.view-drawer__banner-amount-currency {
  margin-top: 4px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  letter-spacing: 0.05em;
}

/* ===== KPI 指标 ===== */
.view-drawer__kpi {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 10px;
  margin-top: 14px;
}

.view-drawer__kpi-item {
  padding: 12px 14px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  transition: all 0.2s ease;
}

.view-drawer__kpi-item:hover {
  border-color: hsl(var(--primary) / 40%);
  transform: translateY(-1px);
}

.view-drawer__kpi-item--paid {
  background: hsl(var(--success) / 6%);
  border-color: hsl(var(--success) / 20%);
}

.view-drawer__kpi-item--unpaid {
  background: hsl(var(--warning) / 6%);
  border-color: hsl(var(--warning) / 20%);
}

.view-drawer__kpi-label {
  margin-bottom: 4px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.view-drawer__kpi-value {
  font-size: 15px;
  font-weight: 700;
  color: hsl(var(--foreground));
  letter-spacing: -0.01em;
}

.view-drawer__kpi-value--minus {
  color: hsl(var(--destructive));
}

.view-drawer__kpi-value--warning {
  color: hsl(var(--warning));
}

.view-drawer__kpi-value--success {
  color: hsl(var(--success));
}

.view-drawer__kpi-value--danger {
  color: hsl(var(--destructive));
}

/* ===== 主体两栏 ===== */
.view-drawer__body {
  display: grid;
  grid-template-columns: 1fr 300px;
  gap: 14px;
  margin-top: 14px;
}

.view-drawer__main {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-width: 0;
}

.view-drawer__side {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

/* ===== 卡片 ===== */
.view-drawer__card {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.view-drawer__card-title {
  display: flex;
  gap: 8px;
  align-items: center;
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.view-drawer__card-title-bar {
  display: inline-block;
  width: 3px;
  height: 14px;
  background: hsl(var(--primary));
  border-radius: 2px;
}

.view-drawer__card-title-count {
  margin-left: 4px;
  font-size: 12px;
  font-weight: 400;
  color: hsl(var(--muted-foreground));
}

/* ===== 产品单元格 ===== */
.view-drawer__product-cell {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.view-drawer__product-name {
  font-weight: 500;
  color: hsl(var(--foreground));
}

.view-drawer__product-code {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

/* ===== 金额汇总 ===== */
.view-drawer__summary {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 260px;
  padding: 10px 14px;
  margin-top: 12px;
  margin-left: auto;
  background: hsl(var(--muted) / 40%);
  border-radius: 6px;
}

.view-drawer__summary-row {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
}

.view-drawer__summary-row--minus {
  color: hsl(var(--destructive));
}

.view-drawer__summary-row--total {
  padding-top: 6px;
  margin-top: 6px;
  font-size: 14px;
  color: hsl(var(--foreground));
  border-top: 1px dashed hsl(var(--border));
}

.view-drawer__summary-row--total strong {
  font-size: 15px;
  color: hsl(var(--primary));
}

/* ===== 财务信息 ===== */
.view-drawer__finance {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.view-drawer__finance-block-title {
  padding-bottom: 6px;
  margin-bottom: 8px;
  font-size: 12px;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  text-transform: uppercase;
  letter-spacing: 0.04em;
  border-bottom: 1px solid hsl(var(--border));
}

/* ===== 发货记录 ===== */
.view-drawer__shipments {
  padding-top: 10px;
  margin-top: 12px;
  border-top: 1px solid hsl(var(--border));
}

.view-drawer__shipments-title {
  margin-bottom: 10px;
  font-size: 12px;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
}

.view-drawer__shipment-item {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
}

/* ===== 审批记录 ===== */
.view-drawer__approval-header {
  display: flex;
  gap: 8px;
  align-items: center;
}

/* ===== 审批流程预览 ===== */
.view-drawer__flow {
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-x: auto;
}

.view-drawer__flow-chain {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.view-drawer__flow-arrow {
  font-size: 16px;
  line-height: 1;
  color: hsl(var(--muted-foreground));
}

.view-drawer__flow-node {
  min-width: 104px;
  padding: 8px 12px;
  text-align: center;
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.view-drawer__flow-node-title {
  font-size: 13px;
  font-weight: 600;
}

.view-drawer__flow-node-sub {
  margin-top: 2px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

/* ===== 备注 ===== */
.view-drawer__remark {
  margin: 0;
  font-size: 14px;
  line-height: 1.6;
  color: hsl(var(--foreground) / 85%);
  white-space: pre-wrap;
}

/* ===== 侧边栏：关联单据 ===== */
.view-drawer__related {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.view-drawer__related-item {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 10px 12px;
  cursor: pointer;
  background: hsl(var(--muted) / 30%);
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  transition: all 0.2s ease;
}

.view-drawer__related-item:hover {
  background: hsl(var(--primary) / 6%);
  border-color: hsl(var(--primary) / 30%);
  transform: translateX(2px);
}

.view-drawer__related-icon {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  font-size: 12px;
  font-weight: 700;
  color: white;
  border-radius: 6px;
}

.view-drawer__related-item--opportunity .view-drawer__related-icon {
  background: linear-gradient(135deg, hsl(262deg 83% 58%), hsl(262deg 83% 48%));
}

.view-drawer__related-item--quotation .view-drawer__related-icon {
  background: linear-gradient(
    135deg,
    hsl(212deg 100% 50%),
    hsl(212deg 100% 40%)
  );
}

.view-drawer__related-item--contract .view-drawer__related-icon {
  background: linear-gradient(135deg, hsl(142deg 71% 45%), hsl(142deg 71% 35%));
}

.view-drawer__related-info {
  flex: 1;
  min-width: 0;
}

.view-drawer__related-type {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  letter-spacing: 0.04em;
}

.view-drawer__related-name {
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--foreground));
  white-space: nowrap;
}

.view-drawer__related-arrow {
  font-size: 16px;
  font-weight: 300;
  color: hsl(var(--muted-foreground));
}

.view-drawer__related-empty {
  padding: 16px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  text-align: center;
}

/* ===== 侧边栏：警告卡片 ===== */
.view-drawer__side-card--alert {
  background: hsl(var(--destructive) / 4%);
  border-color: hsl(var(--destructive) / 20%);
}

.view-drawer__alert {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.view-drawer__alert-icon {
  font-size: 20px;
  line-height: 1;
  color: hsl(var(--destructive));
}

.view-drawer__alert-title {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--destructive));
}

.view-drawer__alert-desc {
  margin-top: 2px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== 底部操作栏 ===== */
.view-drawer__footer {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
}

.view-drawer__footer-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}

/* ===== 响应式 ===== */
@media (max-width: 1280px) {
  .view-drawer__kpi {
    grid-template-columns: repeat(4, 1fr);
  }
}

@media (max-width: 1024px) {
  .view-drawer__body {
    grid-template-columns: 1fr;
  }

  .view-drawer__banner {
    flex-direction: column;
  }

  .view-drawer__banner-amount {
    align-items: flex-start;
    padding-top: 12px;
    padding-left: 0;
    border-top: 1px solid hsl(var(--border));
    border-left: none;
  }

  .view-drawer__finance {
    grid-template-columns: 1fr;
    gap: 16px;
  }
}
</style>
