<script lang="ts" setup>
/**
 * 入库单详情抽屉（含审批流程图 + 审批日志 + 审批引擎增强操作）
 *
 * 布局分区：
 *   A. 文档头部（单号 / 状态 / 类型 / 关键信息）
 *   B. 审批流程图（纵向时间线，节点状态可视化 + 抄送人列表）
 *   C. 汇总 + 基本信息 + 入库明细（可折叠）
 *   D. 底部操作栏（根据用户角色和单据状态动态渲染）
 *
 * 角色判定逻辑：
 *   - 提交人（detail.submittedBy === 当前用户ID）：草稿态查看后提交审批 / 审核中撤回、抄送
 *   - 审批人（持 product:inbound:audit 权限且在当前节点候选审批人池）：审核通过 / 驳回 / 抄送 / 加签 / 转办 / 委派 / 退回
 *   - 查看者：仅查看详情和流程
 */
import { computed, ref, watch } from 'vue';

import { useAccessStore, useUserStore } from '@vben/stores';

import {
  Button,
  Collapse,
  CollapsePanel,
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
  auditInboundApi,
  getInboundInfoApi,
  rejectInboundApi,
  submitInboundApi,
  withdrawInboundApi,
} from '#/api/core/product/inbound';
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
  inboundId?: number | null;
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
  purchase: { label: $t('page.product.inbound.type.purchase'), color: 'blue', icon: 'truck' },
  return: { label: $t('page.product.inbound.type.return'), color: 'orange', icon: 'corner-up-left' },
  surplus: { label: $t('page.product.inbound.type.surplus'), color: 'green', icon: 'trending-up' },
  initial: { label: $t('page.product.inbound.type.initial'), color: 'cyan', icon: 'flag' },
  other: { label: $t('page.product.inbound.type.other'), color: 'default', icon: 'package' },
};

const statusMap: Record<number, { label: string; color: string; phase: number }> = {
  0: { label: $t('page.product.inbound.status.0'), color: 'default', phase: 0 },
  1: { label: $t('page.product.inbound.status.1'), color: 'processing', phase: 1 },
  2: { label: $t('page.product.inbound.status.2'), color: 'warning', phase: 2 },
  3: { label: $t('page.product.inbound.status.3'), color: 'success', phase: 3 },
  4: { label: $t('page.product.inbound.status.4'), color: 'error', phase: 0 },
};

function getType(val?: string) {
  return typeMap[val || ''] || { label: val || '-', color: 'default', icon: 'package' };
}

function getStatus(val?: number) {
  return statusMap[val ?? -1] || { label: '-', color: 'default', phase: 0 };
}

const drawerWidth = computed(() => (isFullscreen.value ? '100%' : '75%'));

// ===== 角色判定 =====
// 提交人：提交审核的人
const isSelfSubmitted = computed(() => {
  return detail.value?.submittedBy === currentUserId;
});

const canAudit = computed(() => {
  return accessStore.hasAccessCode('product:inbound:audit');
});

// 制单人权限：可提交审批 / 撤回自己的单据
const canSubmit = computed(() => {
  return accessStore.hasAccessCode('product:inbound:update');
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

// ===== 审批流程节点（纵向时间线，结合审批实例数据） =====
const flowNodes = computed(() => {
  if (!detail.value) return [];
  const status = detail.value.status;
  const inst = instance.value;

  const submitter = inst?.submitterName || detail.value.submittedByName || detail.value.createdByName || '-';
  const submitTime = inst?.submittedAt || detail.value.submitTime || detail.value.updateTime || '-';
  const auditDesc = inst?.candidateApproverNames?.length
    ? inst.candidateApproverNames.join('、')
    : status >= 3 || status === 4
      ? detail.value.auditByName || '-'
      : '等待审核';

  const nodes: {
    key: string;
    title: string;
    desc: string;
    time: string;
    icon: string;
    state: 'done' | 'current' | 'rejected' | 'pending';
  }[] = [
    {
      key: 'create',
      title: '制单',
      desc: detail.value.createdByName || '-',
      time: detail.value.createTime || '-',
      icon: 'file-plus',
      state: 'done',
    },
    {
      key: 'submit',
      title: '提交审批',
      desc: status >= 1 ? submitter : '等待提交',
      time: status >= 1 ? submitTime : '',
      icon: 'send',
      state: status === 4 ? 'rejected' : status >= 1 ? 'done' : status === 0 ? 'current' : 'pending',
    },
    {
      key: 'audit',
      title: '审核通过',
      desc: auditDesc,
      time: status >= 3 ? detail.value.auditTime || '-' : status === 4 ? detail.value.auditTime || '-' : '',
      icon: 'check-circle',
      state: status === 4 ? 'rejected' : status >= 3 ? 'done' : status === 1 ? 'current' : 'pending',
    },
    {
      key: 'complete',
      title: '入库完成',
      desc: status >= 3 ? '库存已更新' : '',
      time: status >= 3 ? detail.value.updateTime || '-' : '',
      icon: 'package-check',
      state: status >= 3 ? 'done' : 'pending',
    },
  ];

  // 驳回场景：追加驳回节点
  if (status === 4) {
    nodes.push({
      key: 'rejected',
      title: '审核驳回',
      desc: detail.value.auditByName || '-',
      time: detail.value.auditTime || '-',
      icon: 'x-circle',
      state: 'rejected',
    });
  }

  return nodes;
});

// 抄送人列表
const ccUsers = computed(() => {
  const inst = instance.value;
  const users: any[] = inst?.ccUsers || [];
  return users.map((u) => u.userName || `用户${u.userId}`);
});

// ===== 明细表格列 =====
const itemColumns = computed(() => [
  { title: '#', key: 'seq', width: 45, customRender: ({ index }: any) => index + 1 },
  { title: $t('page.product.inbound.field.itemProductCode'), dataIndex: 'productCode', width: 120, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.inbound.field.itemProductName'), dataIndex: 'productName', ellipsis: true, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.inbound.field.spec') || '规格', dataIndex: 'spec', width: 100, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.inbound.field.unit') || '单位', dataIndex: 'unit', width: 70, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.inbound.field.itemQuantity'), dataIndex: 'quantity', width: 90, customRender: ({ value }: any) => value ?? '-' },
  { title: $t('page.product.inbound.field.itemUnitPrice'), dataIndex: 'unitPrice', width: 100, customRender: ({ value }: any) => (value != null ? `¥${Number(value).toFixed(2)}` : '-') },
  { title: $t('page.product.inbound.field.totalPrice') || '金额', dataIndex: 'totalPrice', width: 110, customRender: ({ value }: any) => (value != null ? `¥${Number(value).toFixed(2)}` : '-') },
  { title: $t('page.product.inbound.field.remark'), dataIndex: 'remark', width: 120, ellipsis: true, customRender: ({ value }: any) => value || '-' },
]);

// ===== 数据加载 =====
async function loadDetail(id: number) {
  loading.value = true;
  try {
    const res: any = await getInboundInfoApi(id);
    const raw = res?.data ?? res;
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
  if (props.inboundId) {
    await loadDetail(props.inboundId);
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
    await submitInboundApi(
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
    await auditInboundApi(detail.value.id, approveComment.value || undefined);
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
    await rejectInboundApi(detail.value.id, rejectReason.value || undefined);
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
    content: '确定要撤回该入库单的审批申请吗？撤回后单据将退回草稿状态。',
    okText: '确定撤回',
    cancelText: '取消',
    async onOk() {
      actionLoading.value = true;
      try {
        await withdrawInboundApi(detail.value.id);
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
  () => [props.visible, props.inboundId] as const,
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
    :title="$t('page.product.inbound.detail')"
    :body-style="{ padding: '0', display: 'flex', flexDirection: 'column', height: '100%' }"
    @close="emit('update:visible', false)"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? $t('page.product.inbound.drawer.restore') : $t('page.product.inbound.drawer.fullscreen')">
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
        <div v-if="detail" class="inbound-detail">
          <!-- ===== A. 文档头部 ===== -->
          <div class="doc-header">
            <div class="doc-header-bar" :class="`bar-${getStatus(detail.status).color}`"></div>
            <div class="doc-header-body">
              <div class="doc-header-top">
                <h2 class="doc-title">{{ detail.inboundNo || '-' }}</h2>
                <Tag :color="getStatus(detail.status).color" class="doc-status-tag">
                  {{ getStatus(detail.status).label }}
                </Tag>
              </div>
              <div class="doc-header-meta">
                <Tag :color="getType(detail.inboundType).color" class="doc-type-tag">
                  {{ getType(detail.inboundType).label }}
                </Tag>
                <span class="meta-item">{{ $t('page.product.inbound.field.warehouse') }}：{{ detail.warehouseName || '-' }}</span>
                <span class="meta-item">{{ $t('page.product.inbound.field.createTime') }}：{{ detail.createTime || '-' }}</span>
                <span v-if="detail.submittedByName || detail.createdByName" class="meta-item">提交人：{{ detail.submittedByName || detail.createdByName }}</span>
              </div>
            </div>
          </div>

          <!-- ===== B. 审批流程图（纵向时间线） ===== -->
          <div class="flow-section">
            <div class="section-label">审批流程</div>
            <div class="flow-timeline">
              <div
                v-for="(node, idx) in flowNodes"
                :key="node.key"
                class="flow-node"
                :class="`state-${node.state}`"
              >
                <!-- 连线 -->
                <div v-if="idx > 0" class="flow-connector" :class="node.state === 'done' ? 'connector-done' : ''"></div>
                <!-- 图标 -->
                <div class="flow-icon">
                  <svg v-if="node.state === 'done'" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                  <svg v-else-if="node.state === 'rejected'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                  <span v-else-if="node.state === 'current'" class="flow-pulse"></span>
                  <span v-else class="flow-num">{{ idx + 1 }}</span>
                </div>
                <!-- 内容 -->
                <div class="flow-content">
                  <div class="flow-node-title">{{ node.title }}</div>
                  <div v-if="node.desc" class="flow-node-desc">{{ node.desc }}</div>
                  <div v-if="node.time" class="flow-node-time">{{ node.time }}</div>
                </div>
                <!-- 角标 -->
                <div class="flow-badge" :class="`badge-${node.state}`">
                  <template v-if="node.state === 'done'">已完成</template>
                  <template v-else-if="node.state === 'current'">进行中</template>
                  <template v-else-if="node.state === 'rejected'">已驳回</template>
                  <template v-else>待处理</template>
                </div>
              </div>
            </div>

            <!-- 抄送人列表 -->
            <div v-if="ccUsers.length > 0" class="cc-section">
              <div class="cc-title">抄送人</div>
              <div class="cc-tags">
                <Tag v-for="(name, idx) in ccUsers" :key="idx" color="geekblue">{{ name }}</Tag>
              </div>
            </div>
          </div>

          <!-- ===== C. 文档详情（可折叠） ===== -->
          <Collapse :default-active-key="['summary', 'items']" ghost class="doc-collapse">
            <!-- 汇总数据 -->
            <CollapsePanel key="summary" header="单据概览">
              <div class="summary-row">
                <div class="summary-item">
                  <span class="summary-label">{{ $t('page.product.inbound.field.totalQuantity') }}</span>
                  <span class="summary-value">{{ detail.totalQuantity ?? '-' }}</span>
                </div>
                <div class="summary-item summary-item--accent">
                  <span class="summary-label">{{ $t('page.product.inbound.field.totalAmount') }}</span>
                  <span class="summary-value">¥{{ Number(detail.totalAmount ?? 0).toFixed(2) }}</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">{{ $t('page.product.inbound.field.items') }}</span>
                  <span class="summary-value">{{ detail.items?.length ?? 0 }}</span>
                </div>
              </div>
              <div class="info-grid">
                <div class="info-cell">
                  <span class="info-key">{{ $t('page.product.inbound.field.inboundType') }}</span>
                  <span class="info-val">{{ getType(detail.inboundType).label }}</span>
                </div>
                <div class="info-cell">
                  <span class="info-key">{{ $t('page.product.inbound.field.warehouse') }}</span>
                  <span class="info-val">{{ detail.warehouseName || '-' }}</span>
                </div>
                <div class="info-cell">
                  <span class="info-key">制单人</span>
                  <span class="info-val">{{ detail.createdByName || '-' }}</span>
                </div>
                <div v-if="detail.submittedByName" class="info-cell">
                  <span class="info-key">提交人</span>
                  <span class="info-val">{{ detail.submittedByName }}</span>
                </div>
                <div v-if="detail.auditByName" class="info-cell">
                  <span class="info-key">审核人</span>
                  <span class="info-val">{{ detail.auditByName }}</span>
                </div>
                <div v-if="detail.auditTime" class="info-cell">
                  <span class="info-key">审核时间</span>
                  <span class="info-val">{{ detail.auditTime }}</span>
                </div>
                <div v-if="detail.sourceOrderNo" class="info-cell">
                  <span class="info-key">{{ $t('page.product.inbound.field.sourceOrderNo') }}</span>
                  <span class="info-val">{{ detail.sourceOrderNo }}</span>
                </div>
              </div>
              <div v-if="detail.remark" class="remark-block">
                <span class="info-key">{{ $t('page.product.inbound.field.remark') }}</span>
                <p class="remark-text">{{ detail.remark }}</p>
              </div>
            </CollapsePanel>

            <!-- 入库明细 -->
            <CollapsePanel key="items" :header="`${$t('page.product.inbound.field.items')}（${detail.items?.length ?? 0}）`">
              <Table
                :columns="itemColumns"
                :data-source="detail.items || []"
                :pagination="false"
                size="small"
                :scroll="{ x: 800 }"
                row-key="id"
                bordered
              >
                <template #emptyText>
                  <Empty description="暂无明细" />
                </template>
              </Table>
            </CollapsePanel>
          </Collapse>
        </div>

        <Empty v-else-if="!loading" :description="$t('page.product.inbound.message.noData') || '暂无数据'" class="py-20" />
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
      title="驳回入库单"
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

.inbound-detail {
  padding-bottom: 80px;
}

/* ===== A. 文档头部 ===== */
.doc-header {
  display: flex;
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
}

.doc-header-bar {
  width: 4px;
  flex-shrink: 0;
}
.bar-default { background: hsl(var(--muted-foreground) / 0.3); }
.bar-processing { background: hsl(221 83% 53%); }
.bar-warning { background: hsl(38 92% 50%); }
.bar-success { background: hsl(142 71% 45%); }
.bar-error { background: hsl(0 84% 60%); }

.doc-header-body {
  flex: 1;
  padding: 16px 18px;
}

.doc-header-top {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.doc-title {
  font-size: 18px;
  font-weight: 700;
  color: hsl(var(--foreground));
  margin: 0;
  letter-spacing: -0.01em;
}

.doc-status-tag {
  flex-shrink: 0;
  font-size: 12px;
  font-weight: 600;
}

.doc-header-meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px 14px;
}

.doc-type-tag {
  flex-shrink: 0;
  font-size: 11px;
}

.meta-item {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== B. 审批流程图 ===== */
.flow-section {
  margin-top: 20px;
  padding: 16px 18px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
}

.section-label {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin-bottom: 14px;
  padding-left: 8px;
  border-left: 3px solid hsl(var(--primary));
}

.flow-timeline {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.flow-node {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 8px 0;
  position: relative;
}

.flow-connector {
  position: absolute;
  top: -12px;
  left: 15px;
  width: 2px;
  height: 20px;
  background: hsl(var(--border));
}

.connector-done {
  background: hsl(142 71% 45%);
}

.flow-icon {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-size: 12px;
  font-weight: 600;
  z-index: 1;
  transition: all 0.3s ease;
}

.state-done .flow-icon {
  background: hsl(142 71% 45%);
  border: 2px solid hsl(142 71% 45%);
}

.state-current .flow-icon {
  background: hsl(221 83% 53%);
  border: 2px solid hsl(221 83% 53%);
  box-shadow: 0 0 0 4px hsl(221 83% 53% / 0.12);
}

.state-rejected .flow-icon {
  background: hsl(0 84% 60%);
  border: 2px solid hsl(0 84% 60%);
}

.state-pending .flow-icon {
  background: hsl(var(--muted) / 0.6);
  border: 2px solid hsl(var(--border));
  color: hsl(var(--muted-foreground) / 0.5);
}

.flow-pulse {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: white;
  animation: flow-pulse-anim 1.5s ease-in-out infinite;
}

@keyframes flow-pulse-anim {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.4; transform: scale(0.6); }
}

.flow-content {
  flex: 1;
  padding-top: 2px;
}

.flow-node-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
  line-height: 1.4;
}

.state-pending .flow-node-title {
  color: hsl(var(--muted-foreground) / 0.6);
}

.flow-node-desc {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  margin-top: 2px;
}

.flow-node-time {
  font-size: 11px;
  color: hsl(var(--muted-foreground) / 0.7);
  margin-top: 1px;
}

.flow-badge {
  flex-shrink: 0;
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  margin-top: 4px;
}

.badge-done {
  background: hsl(142 71% 45% / 0.1);
  color: hsl(142 71% 35%);
}

.badge-current {
  background: hsl(221 83% 53% / 0.1);
  color: hsl(221 83% 53%);
}

.badge-rejected {
  background: hsl(0 84% 60% / 0.1);
  color: hsl(0 84% 50%);
}

.badge-pending {
  background: hsl(var(--muted) / 0.5);
  color: hsl(var(--muted-foreground) / 0.5);
}

/* 抄送人 */
.cc-section {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px dashed hsl(var(--border));
}

.cc-title {
  font-size: 12px;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  margin-bottom: 8px;
}

.cc-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

/* ===== C. 折叠面板 ===== */
.doc-collapse {
  margin-top: 16px;
}

.doc-collapse :deep(.ant-collapse-header) {
  font-size: 13px;
  font-weight: 600;
  padding: 10px 4px !important;
}

.doc-collapse :deep(.ant-collapse-content-box) {
  padding: 0 4px 12px !important;
}

/* 汇总行 */
.summary-row {
  display: flex;
  gap: 12px;
  margin-bottom: 14px;
}

.summary-item {
  flex: 1;
  padding: 12px 14px;
  border-radius: 8px;
  background: hsl(var(--muted) / 0.4);
  border: 1px solid hsl(var(--border));
  text-align: center;
}

.summary-item--accent {
  background: hsl(var(--primary) / 0.06);
  border-color: hsl(var(--primary) / 0.2);
}

.summary-label {
  display: block;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin-bottom: 4px;
}

.summary-value {
  display: block;
  font-size: 18px;
  font-weight: 700;
  color: hsl(var(--foreground));
}

.summary-item--accent .summary-value {
  color: hsl(var(--primary));
}

/* 信息网格 */
.info-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1px;
  background: hsl(var(--border));
  border-radius: 8px;
  overflow: hidden;
}

.info-cell {
  padding: 8px 12px;
  background: hsl(var(--card));
}

.info-key {
  display: block;
  font-size: 10px;
  color: hsl(var(--muted-foreground));
  margin-bottom: 2px;
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.info-val {
  display: block;
  font-size: 13px;
  color: hsl(var(--foreground));
  font-weight: 500;
}

/* 备注 */
.remark-block {
  margin-top: 12px;
  padding: 10px 12px;
  background: hsl(var(--muted) / 0.3);
  border-radius: 8px;
  border-left: 3px solid hsl(var(--primary) / 0.3);
}

.remark-text {
  font-size: 13px;
  color: hsl(var(--foreground));
  line-height: 1.6;
  margin: 4px 0 0;
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

  .summary-row {
    flex-direction: column;
    gap: 8px;
  }

  .doc-header-meta {
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
