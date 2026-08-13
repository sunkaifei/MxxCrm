<script lang="ts" setup>
/**
 * 出库单详情抽屉（含审批流程 + 审批日志 + 审批引擎增强操作）
 *
 * 布局分区：
 *   A. 文档头部（单号 / 状态 / 类型 / 关键信息）
 *   B. 审核进度条（草稿 → 待审核 → 已审核 → 已完成）
 *   C. 汇总 + 基本信息 + 出库明细
 *   D. 底部操作栏（根据用户角色和单据状态动态渲染）
 *
 * 角色判定逻辑：
 *   - 提交人（detail.submittedBy === 当前用户ID）：草稿态查看后提交审批 / 审核中撤回、抄送
 *   - 审批人（持 product:outbound:audit 权限且在当前节点候选审批人池）：审核通过 / 驳回 / 抄送 / 加签 / 转办 / 委派 / 退回
 *   - 查看者：仅查看详情和流程
 */
import { computed, ref, watch } from 'vue';

import { useAccessStore, useUserStore } from '@vben/stores';

import {
  Button,
  Divider,
  Drawer,
  Empty,
  Form,
  FormItem,
  Input,
  Modal,
  Select,
  Spin,
  Table,
  Tag,
  Textarea,
  Tooltip,
  message,
} from 'ant-design-vue';

import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
import { $t } from '#/locales';
import {
  auditOutboundApi,
  getOutboundInfoApi,
  rejectOutboundApi,
  submitOutboundApi,
  withdrawOutboundApi,
} from '#/api/core/product/outbound';
import {
  addCcApprovalApi,
  addSignApprovalApi,
  delegateApprovalApi,
  rejectToApprovalApi,
  transferApprovalApi,
} from '#/api/core/system/approval';
import { searchUsersApi } from '#/api/core/message/chat';

const props = defineProps<{
  visible: boolean;
  outboundId?: number | null;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'refresh'): void;
}>();

const accessStore = useAccessStore();
const userStore = useUserStore();
const { isSuperAdmin } = useSuperAdminGuard();
const loading = ref(false);
const detail = ref<any>(null);
const isFullscreen = ref(false);
const currentUserId = Number(userStore.userInfo?.userId || userStore.userInfo?.id || 0);

// ===== 审批操作状态 =====
const actionLoading = ref(false);
const rejectModalVisible = ref(false);
const rejectReason = ref('');
const approveModalVisible = ref(false);
const approveComment = ref('');

// 审批实例（来自后端 get_detail 返回的 instance 字段）
const instance = computed(() => detail.value?.instance || null);

// ===== 类型 & 状态映射 =====
const typeMap: Record<string, { label: string; color: string; icon: string }> = {
  sale: { label: $t('page.product.outbound.type.sale'), color: 'blue', icon: '📤' },
  material: { label: $t('page.product.outbound.type.material'), color: 'cyan', icon: '🔧' },
  shortage: { label: $t('page.product.outbound.type.shortage'), color: 'orange', icon: '📉' },
  scrap: { label: $t('page.product.outbound.type.scrap'), color: 'red', icon: '🗑️' },
  freeze: { label: $t('page.product.outbound.type.freeze'), color: 'purple', icon: '❄️' },
  other: { label: $t('page.product.outbound.type.other'), color: 'default', icon: '📦' },
};

const statusMap: Record<number, { label: string; color: string; step: number }> = {
  0: { label: $t('page.product.outbound.status.0'), color: 'default', step: 0 },
  1: { label: $t('page.product.outbound.status.1'), color: 'processing', step: 1 },
  2: { label: $t('page.product.outbound.status.2'), color: 'warning', step: 2 },
  3: { label: $t('page.product.outbound.status.3'), color: 'success', step: 3 },
  4: { label: $t('page.product.outbound.status.4'), color: 'error', step: 1 },
};

function getType(val?: string) {
  return typeMap[val || ''] || { label: val || '-', color: 'default', icon: '📦' };
}

function getStatus(val?: number) {
  return statusMap[val ?? -1] || { label: '-', color: 'default', step: 0 };
}

const drawerWidth = computed(() => (isFullscreen.value ? '100%' : '75%'));

// 审核进度条步骤
const auditSteps = computed(() => {
  const s = getStatus(detail.value?.status).step;
  return [
    { key: 'draft', label: $t('page.product.outbound.status.0'), done: s >= 0, active: s === 0 },
    { key: 'pending', label: $t('page.product.outbound.status.1'), done: s >= 1, active: s === 1 },
    { key: 'approved', label: $t('page.product.outbound.status.2'), done: s >= 2, active: s === 2 },
    { key: 'done', label: $t('page.product.outbound.status.3'), done: s >= 3, active: s === 3 },
  ];
});

// ===== 角色判定 =====
// 提交人：提交审核的人
const isSelfSubmitted = computed(() => {
  return detail.value?.submittedBy === currentUserId;
});

const canAudit = computed(() => {
  return accessStore.hasAccessCode('product:outbound:audit');
});

// 制单人权限：可提交审批 / 撤回自己的单据
const canSubmit = computed(() => {
  return accessStore.hasAccessCode('product:outbound:update');
});

// 当前用户是否在当前节点候选审批人池中（审批引擎判定）
const isCandidateApprover = computed(() => {
  const inst = instance.value;
  if (!inst) return false;
  const candidates: number[] = inst.candidateApprovers || [];
  if (candidates.length > 0) {
    return candidates.includes(currentUserId);
  }
  return inst.currentApproverId === currentUserId;
});

// 可审核：持审核权限 + 非本人提交（超管豁免）+ 在当前节点候选池中
const canApprove = computed(() => {
  if (detail.value?.status !== 1) return false;
  if (!canAudit.value) return false;
  if (isSelfSubmitted.value && !isSuperAdmin.value) return false;
  return isCandidateApprover.value;
});

// 抄送：提交人或当前审批人均可（审核中）
const canCc = computed(() => {
  return detail.value?.status === 1 && (isSelfSubmitted.value || isCandidateApprover.value);
});

// 转办 / 委派 / 加签 / 退回：仅当前审批人
const canTransfer = computed(() => detail.value?.status === 1 && isCandidateApprover.value);
const canDelegate = computed(() => detail.value?.status === 1 && isCandidateApprover.value);
const canAddSign = computed(() => detail.value?.status === 1 && isCandidateApprover.value);
const canRejectTo = computed(() => detail.value?.status === 1 && isCandidateApprover.value);

// 撤回：仅单据提交人本人（审批人/超管不显示）
const canWithdraw = computed(() => {
  return detail.value?.status === 1 && canSubmit.value && isSelfSubmitted.value;
});

// 当前用户可执行的操作集合
const availableActions = computed<string[]>(() => {
  if (!detail.value) return [];
  const status = detail.value.status;
  const actions: string[] = [];

  // 草稿态：制单人（update 权限）查看内容后提交审批
  if (status === 0 && canSubmit.value) {
    actions.push('submit');
  }
  // 审核中：审批引擎增强操作
  if (status === 1) {
    if (canApprove.value) {
      actions.push('approve');
      actions.push('reject');
    }
    if (canWithdraw.value) {
      actions.push('withdraw');
    }
    if (canCc.value) actions.push('cc');
    if (canTransfer.value) actions.push('transfer');
    if (canDelegate.value) actions.push('delegate');
    if (canAddSign.value) actions.push('addSign');
    if (canRejectTo.value) actions.push('rejectTo');
  }
  return actions;
});

// ===== 明细表格列 =====
const itemColumns = computed(() => [
  { title: '#', key: 'seq', width: 45, customRender: ({ index }: any) => index + 1 },
  { title: $t('page.product.outbound.field.itemProductCode'), dataIndex: 'productCode', width: 120, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.outbound.field.itemProductName'), dataIndex: 'productName', ellipsis: true, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.outbound.field.spec'), dataIndex: 'spec', width: 100, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.outbound.field.unit'), dataIndex: 'unit', width: 70, customRender: ({ value }: any) => value || '-' },
  {
    title: $t('page.product.outbound.field.itemQuantity'),
    dataIndex: 'quantity',
    width: 90,
    customRender: ({ value }: any) => value ?? '-',
  },
  { title: $t('page.product.outbound.field.batchNo'), dataIndex: 'batchNo', width: 110, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.outbound.field.remark'), dataIndex: 'remark', width: 120, ellipsis: true, customRender: ({ value }: any) => value || '-' },
]);

// ===== 数据加载 =====
async function loadDetail(id: number) {
  loading.value = true;
  try {
    const res: any = await getOutboundInfoApi(id);
    const raw = res?.data ?? res;
    // 后端返回 { detail: {...}, items: [...], instance: {...} }，扁平化到顶层
    if (raw?.detail) {
      detail.value = { ...raw.detail, items: raw.items ?? [], instance: raw.instance ?? null };
    } else {
      detail.value = raw;
    }
  } catch {
    detail.value = null;
  } finally {
    loading.value = false;
  }
}

async function reload() {
  if (props.outboundId) {
    await loadDetail(props.outboundId);
    emit('refresh');
  }
}

// ===== 基础审批操作 =====
// 提交审批弹窗（抄送人可选）
const submitModalVisible = ref(false);
const submitCcUserIds = ref<number[]>([]);
const submitCcReason = ref('');

function openSubmitModal() {
  submitCcUserIds.value = [];
  submitCcReason.value = '';
  userOptions.value = [];
  submitModalVisible.value = true;
}

async function handleSubmit() {
  actionLoading.value = true;
  try {
    await submitOutboundApi(
      detail.value.id,
      submitCcUserIds.value,
      submitCcReason.value || undefined,
    );
    message.success('已提交审批');
    submitModalVisible.value = false;
    await reload();
  } catch {
    // 错误由全局拦截器处理
  } finally {
    actionLoading.value = false;
  }
}

function openApproveModal() {
  approveComment.value = '';
  approveModalVisible.value = true;
}

async function handleApprove() {
  actionLoading.value = true;
  try {
    await auditOutboundApi(detail.value.id, approveComment.value || undefined);
    message.success('审核通过');
    approveModalVisible.value = false;
    await reload();
  } catch {
    // 错误由全局拦截器处理
  } finally {
    actionLoading.value = false;
  }
}

function openRejectModal() {
  rejectReason.value = '';
  rejectModalVisible.value = true;
}

async function handleRejectConfirm() {
  if (!rejectReason.value.trim()) {
    message.warning('请填写驳回原因');
    return;
  }
  actionLoading.value = true;
  try {
    await rejectOutboundApi(detail.value.id, rejectReason.value || undefined);
    message.success('已驳回');
    rejectModalVisible.value = false;
    await reload();
  } catch {
    // 错误由全局拦截器处理
  } finally {
    actionLoading.value = false;
  }
}

async function handleWithdraw() {
  Modal.confirm({
    title: '撤回审批',
    content: '确定要撤回该出库单的审批申请吗？撤回后单据将退回草稿状态。',
    okText: '确定撤回',
    cancelText: '取消',
    async onOk() {
      actionLoading.value = true;
      try {
        await withdrawOutboundApi(detail.value.id);
        message.success('已撤回');
        await reload();
      } catch {
        // 错误由全局拦截器处理
      } finally {
        actionLoading.value = false;
      }
    },
  });
}

// ============ 审批增强功能弹窗（抄送/加签/转办/委派/退回） ============
const modalState = ref<{
  type: 'addCc' | 'addSign' | 'delegate' | 'rejectTo' | 'transfer' | null;
}>({ type: null });

const targetUserId = ref<number | undefined>(undefined);
const targetUserName = ref('');
const targetUserIds = ref<number[]>([]);
const addSignType = ref<1 | 2 | 3>(2); // 1=前加签,2=后加签,3=并加签
const rejectToNodeKey = ref<string | undefined>(''); // '' 表示退回到发起人
const commentText = ref('');
const ccReason = ref('');

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
      const res: any = await searchUsersApi({ keyword, page: 1, pageSize: 20 });
      const list: any[] = res?.list || res || [];
      userOptions.value = list.map((u: any) => ({
        label:
          u.nickname ||
          u.nickName ||
          u.realName ||
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

function resetModalForm() {
  targetUserId.value = undefined;
  targetUserName.value = '';
  targetUserIds.value = [];
  addSignType.value = 2;
  rejectToNodeKey.value = '';
  commentText.value = '';
  ccReason.value = '';
  userOptions.value = [];
}

function openModal(type: 'addCc' | 'addSign' | 'delegate' | 'rejectTo' | 'transfer') {
  resetModalForm();
  modalState.value = { type };
}

function closeModal() {
  modalState.value = { type: null };
}

const modalVisible = computed({
  get: () => modalState.value.type !== null,
  set: (v: boolean) => {
    if (!v) closeModal();
  },
});

const modalTitle = computed(() => {
  const map: Record<string, string> = {
    addCc: '添加抄送',
    addSign: '加签',
    delegate: '委派审批人',
    rejectTo: '退回审批',
    transfer: '转办审批',
  };
  return modalState.value.type ? map[modalState.value.type] : '';
});

// 退回节点选项：基于审批实例中的 flowNodes（审批类型节点）
const rejectNodeOptions = computed(() => {
  if (!instance.value) return [];
  const nodes: any[] = instance.value.flowNodes || [];
  return [
    { label: '退回到发起人（修改后重新提交）', value: '' },
    ...nodes
      .filter((n) => n.nodeType === 2)
      .map((n) => ({
        label: `退回到节点：${n.nodeName}`,
        value: n.nodeKey,
      })),
  ];
});

// 增强操作提交（抄送/加签/转办/委派/退回）
async function handleModalSubmit() {
  const type = modalState.value.type;
  const instanceId = instance.value?.id;
  if (!type || !instanceId) return;
  try {
    switch (type) {
      case 'rejectTo': {
        await rejectToApprovalApi({
          instanceId,
          rejectToNodeKey:
            rejectToNodeKey.value === ''
              ? undefined
              : rejectToNodeKey.value || undefined,
          comment: commentText.value || undefined,
        });
        message.success('已退回');
        break;
      }
      case 'transfer': {
        if (!targetUserId.value) {
          message.warning('请选择转办目标用户');
          return;
        }
        await transferApprovalApi({
          instanceId,
          targetUserId: targetUserId.value,
          targetUserName: targetUserName.value || undefined,
          comment: commentText.value || undefined,
        });
        message.success('已转办');
        break;
      }
      case 'delegate': {
        if (!targetUserId.value) {
          message.warning('请选择被委派人');
          return;
        }
        await delegateApprovalApi({
          instanceId,
          targetUserId: targetUserId.value,
          targetUserName: targetUserName.value || undefined,
          comment: commentText.value || undefined,
        });
        message.success('已委派');
        break;
      }
      case 'addSign': {
        if (!targetUserIds.value.length) {
          message.warning('请选择加签用户');
          return;
        }
        await addSignApprovalApi({
          instanceId,
          addSignType: addSignType.value,
          targetUserIds: targetUserIds.value,
          comment: commentText.value || undefined,
        });
        message.success('已加签');
        break;
      }
      case 'addCc': {
        if (!targetUserIds.value.length) {
          message.warning('请选择抄送用户');
          return;
        }
        await addCcApprovalApi({
          instanceId,
          userIds: targetUserIds.value,
          ccReason: ccReason.value || undefined,
        });
        message.success('已添加抄送');
        break;
      }
    }
    closeModal();
    await reload();
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  }
}

watch(
  () => [props.visible, props.outboundId] as const,
  ([v, id]) => {
    if (v && id) {
      loadDetail(id);
    }
    if (!v) {
      detail.value = null;
      isFullscreen.value = false;
      rejectModalVisible.value = false;
      approveModalVisible.value = false;
      closeModal();
    }
  },
);
</script>

<template>
  <Drawer
    :open="visible"
    :width="drawerWidth"
    placement="right"
    :title="$t('page.product.outbound.detail')"
    :body-style="{ padding: '0', display: 'flex', flexDirection: 'column', height: '100%' }"
    @close="emit('update:visible', false)"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? $t('page.product.outbound.drawer.restore') : $t('page.product.outbound.drawer.fullscreen')">
        <Button type="text" size="small" @click="isFullscreen = !isFullscreen">
          <svg v-if="!isFullscreen" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </Button>
      </Tooltip>
    </template>

    <!-- ===== 可滚动内容区 ===== -->
    <div class="detail-scroll-area">
      <Spin :spinning="loading">
        <div v-if="detail" class="outbound-detail">
          <!-- ===== 头部卡片：单号 + 状态 + 类型 ===== -->
          <div class="detail-header">
            <div class="header-left">
              <div class="header-icon" :style="{ background: `hsl(var(--primary) / 0.1)` }">
                <span class="text-xl">{{ getType(detail.outboundType).icon }}</span>
              </div>
              <div class="header-info">
                <div class="header-title-row">
                  <h2 class="header-title">{{ detail.outboundNo || '-' }}</h2>
                  <Tag :color="getStatus(detail.status).color" class="header-status-tag">
                    {{ getStatus(detail.status).label }}
                  </Tag>
                </div>
                <div class="header-meta">
                  <Tag :color="getType(detail.outboundType).color" class="header-type-tag">
                    {{ getType(detail.outboundType).label }}
                  </Tag>
                  <span class="header-meta-item">{{ $t('page.product.outbound.field.warehouse') }}：{{ detail.warehouseName || '-' }}</span>
                  <span class="header-meta-item">{{ $t('page.product.outbound.field.createTime') }}：{{ detail.createTime || '-' }}</span>
                  <span v-if="detail.submittedByName || detail.createdByName" class="header-meta-item">提交人：{{ detail.submittedByName || detail.createdByName }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- ===== 审核进度条 ===== -->
          <div class="audit-progress">
            <div
              v-for="(step, i) in auditSteps"
              :key="step.key"
              class="progress-step"
              :class="{
                'step-done': step.done && !step.active,
                'step-active': step.active,
                'step-pending': !step.done,
              }"
            >
              <div class="progress-dot">
                <svg v-if="step.done && !step.active" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
                <span v-else-if="step.active" class="progress-pulse"></span>
                <span v-else class="progress-num">{{ i + 1 }}</span>
              </div>
              <span class="progress-label">{{ step.label }}</span>
              <div v-if="i < auditSteps.length - 1" class="progress-bar" :class="{ 'bar-filled': step.done }"></div>
            </div>
          </div>

          <!-- ===== 汇总数据卡片 ===== -->
          <div class="summary-cards">
            <div class="summary-card">
              <div class="summary-label">{{ $t('page.product.outbound.field.totalQuantity') }}</div>
              <div class="summary-value">{{ detail.totalQuantity ?? '-' }}</div>
            </div>
            <div class="summary-card summary-card--primary">
              <div class="summary-label">{{ $t('page.product.outbound.field.totalAmount') }}</div>
              <div class="summary-value">¥{{ Number(detail.totalAmount ?? 0).toFixed(2) }}</div>
            </div>
            <div class="summary-card">
              <div class="summary-label">{{ $t('page.product.outbound.field.items') }}</div>
              <div class="summary-value">{{ detail.items?.length ?? 0 }}</div>
            </div>
          </div>

          <Divider style="margin: 16px 0 12px" />

          <!-- ===== 基本信息 ===== -->
          <div class="section-title">{{ $t('page.product.outbound.drawer.basicInfo') }}</div>
          <div class="info-grid">
            <div class="info-item">
              <span class="info-label">{{ $t('page.product.outbound.field.outboundNo') }}</span>
              <span class="info-value">{{ detail.outboundNo || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ $t('page.product.outbound.field.outboundType') }}</span>
              <span class="info-value">{{ getType(detail.outboundType).label }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ $t('page.product.outbound.field.warehouse') }}</span>
              <span class="info-value">{{ detail.warehouseName || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ $t('page.product.outbound.field.status') }}</span>
              <span class="info-value">
                <Tag :color="getStatus(detail.status).color">{{ getStatus(detail.status).label }}</Tag>
              </span>
            </div>
            <div v-if="detail.sourceOrderNo" class="info-item">
              <span class="info-label">{{ $t('page.product.outbound.field.sourceOrderNo') }}</span>
              <span class="info-value">{{ detail.sourceOrderNo }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ $t('page.product.outbound.field.createdBy') }}</span>
              <span class="info-value">{{ detail.createdByName || '-' }}</span>
            </div>
            <div v-if="detail.submittedByName" class="info-item">
              <span class="info-label">提交人</span>
              <span class="info-value">{{ detail.submittedByName }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ $t('page.product.outbound.field.createTime') }}</span>
              <span class="info-value">{{ detail.createTime || '-' }}</span>
            </div>
            <div v-if="detail.auditByName" class="info-item">
              <span class="info-label">{{ $t('page.product.outbound.field.auditedBy') }}</span>
              <span class="info-value">{{ detail.auditByName }}</span>
            </div>
            <div v-if="detail.auditTime" class="info-item">
              <span class="info-label">{{ $t('page.product.outbound.field.auditTime') }}</span>
              <span class="info-value">{{ detail.auditTime }}</span>
            </div>
          </div>

          <!-- ===== 备注 ===== -->
          <div v-if="detail.remark" class="remark-box">
            <span class="info-label">{{ $t('page.product.outbound.field.remark') }}</span>
            <p class="remark-text">{{ detail.remark }}</p>
          </div>

          <!-- ===== 出库明细表 ===== -->
          <div class="section-title" style="margin-top: 20px">{{ $t('page.product.outbound.field.items') }}</div>
          <Table
            :columns="itemColumns"
            :data-source="detail.items || []"
            :pagination="false"
            size="small"
            :scroll="{ x: 800 }"
            row-key="id"
            class="items-table"
            bordered
          >
            <template #emptyText>
              <Empty :description="$t('page.product.outbound.message.noItems')" />
            </template>
          </Table>
        </div>

        <Empty v-else-if="!loading" :description="$t('page.product.outbound.message.noData')" />
      </Spin>
    </div>

    <!-- ===== D. 底部操作栏（角色感知） ===== -->
    <div v-if="detail && availableActions.length > 0" class="action-footer">
      <div class="action-footer-left">
        <span class="action-hint">
          <template v-if="detail.status === 0 && canSubmit">草稿状态，确认内容后提交审批</template>
          <template v-else-if="detail.status === 1 && canApprove">待审核，可审核通过 / 驳回 / 加签 / 抄送 / 转办 / 委派 / 退回</template>
          <template v-else-if="detail.status === 1 && isSelfSubmitted">您提交的单据，等待库管审核；可撤回或抄送他人</template>
          <template v-else-if="detail.status === 3">已完成</template>
          <template v-else-if="detail.status === 4">已驳回，可修改后重新提交</template>
        </span>
      </div>
      <div class="action-footer-right">
        <!-- 提交人：提交审批（弹窗内可选择抄送人） -->
        <Button
          v-if="availableActions.includes('submit')"
          type="primary"
          :loading="actionLoading"
          @click="openSubmitModal"
        >
          提交审批
        </Button>
        <!-- 提交人：撤回 -->
        <Button
          v-if="availableActions.includes('withdraw')"
          :loading="actionLoading"
          @click="handleWithdraw"
        >
          撤回
        </Button>
        <!-- 审批增强操作 -->
        <Button
          v-if="availableActions.includes('cc')"
          :loading="actionLoading"
          @click="openModal('addCc')"
        >
          抄送
        </Button>
        <Button
          v-if="availableActions.includes('transfer')"
          :loading="actionLoading"
          @click="openModal('transfer')"
        >
          转办
        </Button>
        <Button
          v-if="availableActions.includes('delegate')"
          :loading="actionLoading"
          @click="openModal('delegate')"
        >
          委派
        </Button>
        <Button
          v-if="availableActions.includes('addSign')"
          :loading="actionLoading"
          @click="openModal('addSign')"
        >
          加签
        </Button>
        <Button
          v-if="availableActions.includes('rejectTo')"
          :loading="actionLoading"
          @click="openModal('rejectTo')"
        >
          退回
        </Button>
        <!-- 审批人：通过 -->
        <Button
          v-if="availableActions.includes('approve')"
          type="primary"
          :loading="actionLoading"
          @click="openApproveModal"
        >
          审核通过
        </Button>
        <!-- 审批人：驳回 -->
        <Button
          v-if="availableActions.includes('reject')"
          danger
          :loading="actionLoading"
          @click="openRejectModal"
        >
          驳回
        </Button>
      </div>
    </div>

    <!-- 审核通过弹窗 -->
    <Modal
      v-model:open="approveModalVisible"
      title="审核通过"
      :confirm-loading="actionLoading"
      ok-text="确认通过"
      cancel-text="取消"
      @ok="handleApprove"
    >
      <Form layout="vertical" class="pt-4">
        <FormItem label="审批意见">
          <Input.TextArea
            v-model:value="approveComment"
            :rows="4"
            placeholder="请输入审批意见（可选）"
            :maxlength="500"
            show-count
          />
        </FormItem>
      </Form>
    </Modal>

    <!-- 驳回弹窗 -->
    <Modal
      v-model:open="rejectModalVisible"
      title="驳回出库单"
      :confirm-loading="actionLoading"
      ok-text="确认驳回"
      cancel-text="取消"
      @ok="handleRejectConfirm"
    >
      <Form layout="vertical" class="pt-4">
        <FormItem label="驳回原因" required>
          <Textarea
            v-model:value="rejectReason"
            :rows="4"
            placeholder="请输入驳回原因，将通知提交人"
            :maxlength="500"
            show-count
          />
        </FormItem>
      </Form>
    </Modal>

    <!-- 提交审批弹窗（抄送人可选） -->
    <Modal
      v-model:open="submitModalVisible"
      title="提交审批"
      width="520px"
      destroy-on-close
      :confirm-loading="actionLoading"
      @ok="handleSubmit"
      @cancel="submitCcUserIds = []; submitCcReason = ''"
    >
      <div class="space-y-4 py-2">
        <div>
          <div class="mb-2 text-sm text-gray-600">抄送给（可选）：</div>
          <Select
            v-model:value="submitCcUserIds"
            mode="multiple"
            placeholder="搜索并选择抄送人"
            style="width: 100%"
            :options="userOptions"
            :filter-option="false"
            :loading="userSearching"
            @search="handleUserSearch"
          />
        </div>
        <div>
          <div class="mb-2 text-sm text-gray-600">抄送说明（可选）：</div>
          <Textarea
            v-model:value="submitCcReason"
            :rows="2"
            placeholder="请填写抄送说明"
          />
        </div>
      </div>
    </Modal>

    <!-- 增强功能统一弹窗（抄送/加签/转办/委派/退回） -->
    <Modal
      v-model:visible="modalVisible"
      :title="modalTitle"
      destroy-on-close
      width="520px"
      @cancel="closeModal"
    >
      <div class="space-y-4 py-2">
        <!-- 退回目标节点选择 -->
        <div v-if="modalState.type === 'rejectTo'">
          <div class="mb-2 text-sm text-gray-600">退回到：</div>
          <Select
            v-model:value="rejectToNodeKey"
            :options="rejectNodeOptions"
            placeholder="请选择退回目标"
            style="width: 100%"
          />
          <div class="mt-1 text-xs text-gray-400">
            默认退回到发起人，发起人修改后可重新提交
          </div>
        </div>

        <!-- 转办 / 委派：单选用户 -->
        <div v-if="modalState.type === 'transfer' || modalState.type === 'delegate'">
          <div class="mb-2 text-sm text-gray-600">
            {{ modalState.type === 'transfer' ? '转办给：' : '委派给：' }}
          </div>
          <Select
            v-model:value="targetUserId"
            :filter-option="false"
            :loading="userSearching"
            :options="userOptions"
            allow-clear
            show-search
            placeholder="输入姓名/用户名搜索"
            style="width: 100%"
            @search="handleUserSearch"
            @change="(v: any) => {
              const opt = userOptions.find((o) => o.value === v);
              targetUserName = opt?.label || '';
            }"
          />
          <div class="mt-1 text-xs text-gray-400">
            <template v-if="modalState.type === 'transfer'">
              转办后责任转移，原审批人不再参与此节点审批
            </template>
            <template v-else>
              委派后责任仍归原审批人，被委派人处理后转回
            </template>
          </div>
        </div>

        <!-- 加签：类型选择 + 多选用户 -->
        <div v-if="modalState.type === 'addSign'">
          <div class="mb-2 text-sm text-gray-600">加签类型：</div>
          <Select
            v-model:value="addSignType"
            :options="[
              { label: '前加签（先由加签人审批，再由我审批）', value: 1 },
              { label: '后加签（我通过后，再由加签人审批）', value: 2 },
              { label: '并加签（与我同时审批，并行处理）', value: 3 },
            ]"
            style="width: 100%"
          />
          <div class="mt-3 mb-2 text-sm text-gray-600">加签用户：</div>
          <Select
            v-model:value="targetUserIds"
            :filter-option="false"
            :loading="userSearching"
            :options="userOptions"
            allow-clear
            mode="multiple"
            placeholder="输入姓名/用户名搜索（可多选）"
            style="width: 100%"
            @search="handleUserSearch"
          />
        </div>

        <!-- 抄送：多选用户 -->
        <div v-if="modalState.type === 'addCc'">
          <div class="mb-2 text-sm text-gray-600">抄送给：</div>
          <Select
            v-model:value="targetUserIds"
            :filter-option="false"
            :loading="userSearching"
            :options="userOptions"
            allow-clear
            mode="multiple"
            placeholder="输入姓名/用户名搜索（可多选）"
            style="width: 100%"
            @search="handleUserSearch"
          />
        </div>

        <!-- 退回 / 转办 / 委派 / 加签：审批意见 -->
        <div v-if="modalState.type && ['addSign', 'delegate', 'rejectTo', 'transfer'].includes(modalState.type)">
          <div class="mb-2 text-sm text-gray-600">审批意见：</div>
          <Input.TextArea
            v-model:value="commentText"
            :rows="3"
            placeholder="请填写说明（可选）"
          />
        </div>

        <!-- 抄送说明 -->
        <div v-if="modalState.type === 'addCc'">
          <div class="mb-2 text-sm text-gray-600">抄送说明：</div>
          <Input.TextArea
            v-model:value="ccReason"
            :rows="3"
            placeholder="请填写抄送说明（可选）"
          />
        </div>
      </div>

      <template #footer>
        <Button @click="closeModal">取消</Button>
        <Button type="primary" @click="handleModalSubmit">确认</Button>
      </template>
    </Modal>
  </Drawer>
</template>

<style scoped>
/* ===== 整体布局 ===== */
.detail-scroll-area {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.outbound-detail {
  padding-bottom: 24px;
}

/* ===== 头部卡片 ===== */
.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 20px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
}

.header-left {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  flex: 1;
  min-width: 0;
}

.header-icon {
  flex-shrink: 0;
  width: 52px;
  height: 52px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.header-info {
  flex: 1;
  min-width: 0;
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}

.header-title {
  font-size: 18px;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin: 0;
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-status-tag {
  flex-shrink: 0;
  font-size: 12px;
  padding: 2px 10px;
  border-radius: 4px;
}

.header-meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px 14px;
}

.header-type-tag {
  flex-shrink: 0;
}

.header-meta-item {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
}

/* ===== 审核进度条 ===== */
.audit-progress {
  display: flex;
  align-items: flex-start;
  margin-top: 20px;
  padding: 0 8px;
}

.progress-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex: 1;
  position: relative;
}

.progress-dot {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  z-index: 1;
  transition: all 0.3s ease;
}

.progress-num {
  color: hsl(var(--muted-foreground));
}

.step-pending .progress-dot {
  background: hsl(var(--muted) / 0.6);
  border: 2px solid hsl(var(--border));
  color: hsl(var(--muted-foreground) / 0.6);
}

.step-done .progress-dot {
  background: hsl(142 71% 45%);
  border: 2px solid hsl(142 71% 45%);
}

.step-active .progress-dot {
  background: hsl(var(--primary));
  border: 2px solid hsl(var(--primary));
  box-shadow: 0 0 0 4px hsl(var(--primary) / 0.12);
}

.progress-pulse {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: white;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.7); }
}

.progress-label {
  margin-top: 6px;
  font-size: 11px;
  white-space: nowrap;
}

.step-pending .progress-label {
  color: hsl(var(--muted-foreground) / 0.6);
}

.step-done .progress-label {
  color: hsl(142 71% 35%);
}

.step-active .progress-label {
  color: hsl(var(--primary));
  font-weight: 600;
}

.progress-bar {
  position: absolute;
  top: 14px;
  left: 50%;
  width: 100%;
  height: 2px;
  background: hsl(var(--border));
  z-index: 0;
}

.bar-filled {
  background: hsl(142 71% 45%);
}

/* ===== 汇总卡片 ===== */
.summary-cards {
  display: flex;
  gap: 12px;
  margin-top: 20px;
}

.summary-card {
  flex: 1;
  padding: 14px 16px;
  border-radius: 10px;
  background: hsl(var(--muted) / 0.4);
  border: 1px solid hsl(var(--border));
  text-align: center;
}

.summary-card--primary {
  background: hsl(var(--primary) / 0.06);
  border-color: hsl(var(--primary) / 0.2);
}

.summary-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  margin-bottom: 4px;
}

.summary-value {
  font-size: 20px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.summary-card--primary .summary-value {
  color: hsl(var(--primary));
}

/* ===== 信息网格 ===== */
.section-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin-bottom: 10px;
  padding-left: 8px;
  border-left: 3px solid hsl(var(--primary));
  line-height: 1.2;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0;
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  overflow: hidden;
}

.info-item {
  padding: 10px 14px;
  border-bottom: 1px solid hsl(var(--border));
  border-right: 1px solid hsl(var(--border));
  background: hsl(var(--card));
}

.info-item:nth-child(3n) {
  border-right: none;
}

.info-item:nth-last-child(-n+3):nth-child(3n+1),
.info-item:nth-last-child(-n+3):nth-child(3n+1) ~ .info-item {
  border-bottom: none;
}

.info-label {
  display: block;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin-bottom: 3px;
}

.info-value {
  display: block;
  font-size: 13px;
  color: hsl(var(--foreground));
  font-weight: 500;
  word-break: break-all;
}

/* ===== 备注框 ===== */
.remark-box {
  margin-top: 12px;
  padding: 12px 14px;
  background: hsl(var(--muted) / 0.3);
  border-radius: 8px;
  border-left: 3px solid hsl(var(--primary) / 0.3);
}

.remark-box .info-label {
  margin-bottom: 4px;
}

.remark-text {
  font-size: 13px;
  color: hsl(var(--foreground));
  line-height: 1.6;
  margin: 0;
}

/* ===== 明细表格 ===== */
.items-table :deep(.ant-table) {
  border-radius: 8px;
}

.items-table :deep(.ant-table-thead > tr > th) {
  background: hsl(var(--muted) / 0.5);
  font-weight: 600;
  font-size: 12px;
}

.items-table :deep(.ant-table-tbody > tr > td) {
  font-size: 13px;
  padding: 8px 12px;
}

/* ===== D. 底部操作栏 ===== */
.action-footer {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-top: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  box-shadow: 0 -2px 8px hsl(var(--foreground) / 0.04);
}

.action-footer-left {
  flex: 1;
}

.action-hint {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.action-footer-right {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .info-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .info-item:nth-child(3n) {
    border-right: 1px solid hsl(var(--border));
  }

  .info-item:nth-child(2n) {
    border-right: none;
  }

  .summary-cards {
    flex-direction: column;
    gap: 8px;
  }

  .header-meta {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .action-footer {
    flex-direction: column;
    gap: 8px;
    align-items: stretch;
  }

  .action-footer-right {
    justify-content: flex-end;
  }
}
</style>
