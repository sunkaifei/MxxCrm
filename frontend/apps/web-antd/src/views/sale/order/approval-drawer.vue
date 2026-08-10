<script lang="ts" setup>
import { computed, h, ref, watch } from 'vue';

import { useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Avatar,
  Button,
  Descriptions,
  DescriptionsItem,
  Drawer,
  Input,
  Modal,
  Select,
  Spin,
  TabPane,
  Table,
  Tabs,
  Tag,
  message,
} from 'ant-design-vue';

import {
  addCcApprovalApi,
  addSignApprovalApi,
  approveOrderApi,
  cancelApprovalApi,
  delegateApprovalApi,
  getOrderApprovalDetailApi,
  rejectOrderApi,
  rejectToApprovalApi,
  transferApprovalApi,
} from '#/api';
import { searchUsersApi } from '#/api/core/message/chat';

const STAMP_APPROVED = '/images/approval-approved.svg';
const STAMP_PENDING = '/images/approval-pending.svg';
const STAMP_REJECTED = '/images/approval-rejected.svg';

const props = defineProps<{
  visible: boolean;
  orderId: number | null;
}>();

const userStore = useUserStore();
const currentUserId = computed(() => userStore.userInfo?.userId);

const emit = defineEmits<{
  'update:visible': [val: boolean];
  success: [];
}>();

const loading = ref(false);
const detail = ref<any>(null);
const comment = ref('');
const actionLoading = ref(false);
const activeTab = ref('detail');

// 本地控制 Drawer 开关：父组件打开时同步打开
const drawerOpen = ref(false);
watch(
  () => props.visible,
  (val) => {
    drawerOpen.value = val;
  },
  { immediate: true },
);

// Drawer 状态变化（含点击 X / 遮罩 / ESC / 底部按钮）统一在此同步父组件
watch(drawerOpen, (val) => {
  if (!val) {
    emit('update:visible', false);
  }
});

function handleClose() {
  drawerOpen.value = false;
}

const instance = computed(() => detail.value?.instance);
const order = computed(() => detail.value);

// 审批实例状态映射
const instanceStatusMap: Record<number, { label: string; color: string; stamp: string }> = {
  1: { label: '审批中', color: 'processing', stamp: STAMP_PENDING },
  2: { label: '审批中', color: 'processing', stamp: STAMP_PENDING },
  3: { label: '审批通过', color: 'success', stamp: STAMP_APPROVED },
  4: { label: '已驳回', color: 'error', stamp: STAMP_REJECTED },
};

// 审批操作文本
const actionTextMap: Record<number, { label: string; color: string }> = {
  1: { label: '审批通过', color: 'success' },
  2: { label: '驳回', color: 'error' },
};

// 节点状态
const nodeStatusMap: Record<number, { label: string; color: string; bgClass: string; borderClass: string; textClass: string }> = {
  0: { label: '未到达', color: 'default', bgClass: 'bg-gray-50', borderClass: 'border-gray-200', textClass: 'text-gray-400' },
  1: { label: '审批中', color: 'processing', bgClass: 'bg-blue-50', borderClass: 'border-blue-400 border-2', textClass: 'text-blue-600' },
  2: { label: '已通过', color: 'success', bgClass: 'bg-green-50', borderClass: 'border-green-200', textClass: 'text-green-700' },
  3: { label: '已驳回', color: 'error', bgClass: 'bg-red-50', borderClass: 'border-red-200', textClass: 'text-red-700' },
  4: { label: '已完成', color: 'success', bgClass: 'bg-green-50', borderClass: 'border-green-300', textClass: 'text-green-700' },
};

// 节点类型名称
const _nodeTypeMap: Record<number, string> = {
  1: '发起人',
  2: '审批人',
  3: '条件分支',
  4: '结束',
};

// 审批模式映射
const approveModeMap: Record<number, { label: string; color: string }> = {
  1: { label: '或签', color: 'blue' },
  2: { label: '会签', color: 'purple' },
  3: { label: '依次审批', color: 'orange' },
};

// 是否可审批（当前用户在候选审批人池中且状态为进行中）
const canApprove = computed(() => {
  if (!instance.value) return false;
  if (instance.value.status !== 1 && instance.value.status !== 2) return false;
  const uid = currentUserId.value;
  if (!uid) return false;
  // userId 可能是 string，candidateApprovers 是 number[]，统一转 number 比较
  const uidNum = Number(uid);
  const candidates = instance.value.candidateApprovers || [];
  if (candidates.length > 0) {
    return candidates.includes(uidNum);
  }
  return instance.value.currentApproverId === uidNum;
});

// 当前用户是否为发起人
const isSubmitter = computed(() => {
  if (!instance.value || !currentUserId.value) return false;
  return Number(instance.value.submitterId) === Number(currentUserId.value);
});

// 是否当前节点候选审批人（含转办/委派后扩展的池，与 canApprove 判定保持一致）
const isCandidateApprover = computed(() => {
  if (!instance.value || !currentUserId.value) return false;
  const uidNum = Number(currentUserId.value);
  const candidates = instance.value.candidateApprovers || [];
  if (candidates.length > 0) {
    return candidates.includes(uidNum);
  }
  return instance.value.currentApproverId === uidNum;
});

// 实例是否处于可操作状态（待审批/审批中）
const isActionable = computed(() => {
  if (!instance.value) return false;
  return instance.value.status === 1 || instance.value.status === 2;
});

// 撤销审批：仅发起人 + 实例待处理（1/2）
const canCancel = computed(() => isSubmitter.value && isActionable.value);

// 退回审批：候选审批人 + 实例待处理
const canRejectTo = computed(() => isCandidateApprover.value && isActionable.value);

// 转办：候选审批人 + 实例待处理
const canTransfer = computed(() => isCandidateApprover.value && isActionable.value);

// 委派：候选审批人 + 实例待处理
const canDelegate = computed(() => isCandidateApprover.value && isActionable.value);

// 加签：候选审批人 + 实例待处理
const canAddSign = computed(() => isCandidateApprover.value && isActionable.value);

// 抄送：发起人 或 当前候选审批人 + 实例可操作
const canCc = computed(() => {
  if (!isActionable.value) return false;
  return isSubmitter.value || isCandidateApprover.value;
});

// 底部操作栏是否展示：存在任意可用操作时展示
const showActionBar = computed(
  () =>
    canApprove.value ||
    canCancel.value ||
    canRejectTo.value ||
    canTransfer.value ||
    canDelegate.value ||
    canAddSign.value ||
    canCc.value,
);

// 审批实例ID
const instanceId = computed(() => instance.value?.id);

// 流程节点（按node_order排序，排除条件分支节点，用于流程图和流转记录）
const flowNodesOrdered = computed(() => {
  if (!instance.value?.flowNodes) return [];
  return [...instance.value.flowNodes]
    .filter((n: any) => n.nodeType !== 3)
    .sort((a: any, b: any) => a.nodeOrder - b.nodeOrder);
});

// 审批流转记录表格数据（结合logs和nodes）
const flowRecordColumns = [
  { title: '审批节点', dataIndex: 'nodeName', key: 'nodeName', width: 120 },
  { title: '审批人', dataIndex: 'approverName', key: 'approverName', width: 100 },
  { title: '开始时间', dataIndex: 'startTime', key: 'startTime', width: 170 },
  { title: '结束时间', dataIndex: 'endTime', key: 'endTime', width: 170 },
  { title: '审批状态', dataIndex: 'statusTag', key: 'statusTag', width: 100 },
  { title: '审批建议', dataIndex: 'comment', key: 'comment' },
  { title: '耗时', dataIndex: 'duration', key: 'duration', width: 100 },
];

const flowRecordData = computed(() => {
  if (!instance.value) return [];
  const records: any[] = [];
  const logs = instance.value.logs || [];
  const submittedAt = instance.value.submittedAt;

  // 发起人节点（自动通过）
  records.push({
    key: 'submitter',
    nodeName: '发起人',
    approverName: instance.value.submitterName || '-',
    startTime: submittedAt ? formatDateTime(submittedAt) : '-',
    endTime: submittedAt ? formatDateTime(submittedAt) : '-',
    statusTag: h(Tag, { color: 'success' }, () => '审批通过'),
    comment: '发起人节点首次自动通过',
    duration: '0秒',
  });

  // 审批节点（从logs中取）
  for (let i = 0; i < logs.length; i++) {
    const log = logs[i];
    const prevTime = i === 0 ? submittedAt : logs[i - 1]?.createTime;
    records.push({
      key: `log-${i}`,
      nodeName: log.nodeName || '审批',
      approverName: log.approverName || '-',
      startTime: prevTime ? formatDateTime(prevTime) : '-',
      endTime: log.createTime ? formatDateTime(log.createTime) : '-',
      statusTag: h(Tag, { color: actionTextMap[log.action]?.color || 'default' }, () => actionTextMap[log.action]?.label || '操作'),
      comment: log.comment || '-',
      duration: log.duration || '-',
    });
  }

  return records;
});

// 加载审批详情
async function loadDetail() {
  if (!props.orderId) return;
  loading.value = true;
  try {
    detail.value = await getOrderApprovalDetailApi(props.orderId);
  } finally {
    loading.value = false;
  }
}

// 审批通过
async function handleApprove() {
  if (!props.orderId) return;
  actionLoading.value = true;
  try {
    await approveOrderApi(props.orderId, comment.value || undefined);
    emit('success');
    emit('update:visible', false);
  } finally {
    actionLoading.value = false;
  }
}

// 驳回
async function handleReject() {
  if (!props.orderId) return;
  actionLoading.value = true;
  try {
    await rejectOrderApi(props.orderId, comment.value || undefined);
    emit('success');
    handleClose();
  } finally {
    actionLoading.value = false;
  }
}

// ============ 高级审批操作弹窗 ============
const modalState = ref<{
  type:
    | 'addCc'
    | 'addSign'
    | 'cancel'
    | 'delegate'
    | 'rejectTo'
    | 'transfer'
    | null;
}>({ type: null });

// 表单字段
const targetUserId = ref<number | undefined>(undefined);
const targetUserName = ref('');
const targetUserIds = ref<number[]>([]);
const addSignType = ref<1 | 2 | 3>(2); // 1=前加签,2=后加签,3=并加签
const rejectToNodeKey = ref<string | undefined>(''); // '' 表示退回到发起人
const commentText = ref('');
const cancelReason = ref('');
const ccReason = ref('');

// 用户远程搜索
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

function resetModalForm() {
  targetUserId.value = undefined;
  targetUserName.value = '';
  targetUserIds.value = [];
  addSignType.value = 2;
  rejectToNodeKey.value = '';
  commentText.value = '';
  cancelReason.value = '';
  ccReason.value = '';
  userOptions.value = [];
}

function openModal(
  type:
    | 'addCc'
    | 'addSign'
    | 'cancel'
    | 'delegate'
    | 'rejectTo'
    | 'transfer',
) {
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
    cancel: '撤销审批',
    delegate: '委派审批人',
    rejectTo: '退回审批',
    transfer: '转办审批',
  };
  return modalState.value.type ? map[modalState.value.type] : '';
});

// 退回节点选项：基于流程节点（已流转过的审批节点 + 退回到发起人）
const rejectNodeOptions = computed(() => {
  if (!instance.value) return [];
  const nodes: any[] = instance.value.flowNodes || [];
  return [
    { label: '退回到发起人（修改后重新提交）', value: '' },
    ...nodes
      .filter((n: any) => n.nodeType === 2)
      .map((n: any) => ({
        label: `退回到节点：${n.nodeName}`,
        value: n.nodeKey,
      })),
  ];
});

// 提交高级操作
async function handleSubmit() {
  if (!instanceId.value) return;
  const iid = instanceId.value;
  try {
    switch (modalState.value.type) {
      case 'cancel': {
        await cancelApprovalApi({
          cancelReason: cancelReason.value || undefined,
          instanceId: iid,
        });
        message.success('已撤销审批');
        break;
      }
      case 'rejectTo': {
        await rejectToApprovalApi({
          comment: commentText.value || undefined,
          instanceId: iid,
          rejectToNodeKey:
            rejectToNodeKey.value === ''
              ? undefined
              : rejectToNodeKey.value || undefined,
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
          comment: commentText.value || undefined,
          instanceId: iid,
          targetUserId: targetUserId.value,
          targetUserName: targetUserName.value || undefined,
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
          comment: commentText.value || undefined,
          instanceId: iid,
          targetUserId: targetUserId.value,
          targetUserName: targetUserName.value || undefined,
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
          addSignType: addSignType.value,
          comment: commentText.value || undefined,
          instanceId: iid,
          targetUserIds: targetUserIds.value,
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
          ccReason: ccReason.value || undefined,
          instanceId: iid,
          userIds: targetUserIds.value,
        });
        message.success('已添加抄送');
        break;
      }
    }
    closeModal();
    emit('success');
    // 刷新审批详情以展示最新状态
    loadDetail();
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  }
}

function getFirstChar(name: string | undefined): string {
  if (!name) return '?';
  return name.charAt(0).toUpperCase();
}

function getStampUrl() {
  if (!instance.value) return '';
  return instanceStatusMap[instance.value.status]?.stamp || STAMP_PENDING;
}

watch(
  () => props.visible,
  (val) => {
    if (val && props.orderId) {
      comment.value = '';
      activeTab.value = 'detail';
      loadDetail();
    }
  },
);
</script>

<template>
  <Drawer
    v-model:open="drawerOpen"
    title="订单审批"
    placement="right"
    width="75%"
    :closable="false"
    :body-style="{ padding: 0, display: 'flex', flexDirection: 'column', height: '100%' }"
    :header-style="{ borderBottom: '1px solid #f0f0f0', padding: '16px 24px' }"
  >
    <template #extra>
      <Button type="text" size="small" @click="handleClose">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" /></svg>
      </Button>
    </template>
    <Spin :spinning="loading" class="approval-spin">
      <div v-if="detail" class="approval-container flex flex-col h-full">
        <!-- ========== 顶部区域 ========== -->
        <div class="approval-header relative px-6 pt-4 pb-0">
          <div class="flex items-center justify-between text-sm text-gray-500 mb-3">
            <span>编号：{{ order?.orderNo || orderId }}</span>
          </div>

          <div class="flex items-start justify-between">
            <div>
              <div class="flex items-center gap-3">
                <h2 class="text-2xl font-bold text-gray-900 m-0">订单审批</h2>
                <Tag
                  v-if="instance"
                  :color="instanceStatusMap[instance.status]?.color"
                  class="text-sm px-3 py-0.5"
                >
                  {{ instanceStatusMap[instance.status]?.label }}
                </Tag>
                <Tag v-else color="default" class="text-sm px-3 py-0.5">草稿</Tag>
              </div>
              <div v-if="instance" class="flex items-center gap-2 mt-3 text-gray-500">
                <Avatar :size="36" class="bg-blue-500 flex items-center justify-center text-white text-sm font-medium">
                  {{ getFirstChar(instance.submitterName) }}
                </Avatar>
                <span class="text-base text-gray-700 font-medium">{{ instance.submitterName }}</span>
                <span class="text-sm">{{ formatDateTime(instance.submittedAt) }} 提交</span>
              </div>
            </div>
            <div v-if="instance" class="stamp-container -mt-2 -mr-2">
              <img :src="getStampUrl()" class="w-36 h-28 object-contain opacity-70" />
            </div>
          </div>
        </div>

        <!-- ========== Tab 导航 ========== -->
        <div class="px-6 border-b border-gray-200 mt-4">
          <Tabs v-model:activeKey="activeTab" class="approval-tabs">
            <TabPane key="detail" tab="审批详情" />
            <TabPane key="flow" tab="流程图" />
            <TabPane key="record" tab="流转记录" />
          </Tabs>
        </div>

        <!-- ========== Tab 内容区 ========== -->
        <div class="flex-1 overflow-y-auto px-6 py-4">
          <!-- ====== 审批详情 Tab ====== -->
          <div v-if="activeTab === 'detail'" class="flex gap-6">
            <div class="flex-1 min-w-0">
              <div class="border-b border-gray-100 pb-4 mb-4">
                <div class="flex items-center gap-3 mb-1">
                  <div class="w-10 h-10 rounded-lg bg-blue-50 flex items-center justify-center">
                    <svg class="w-6 h-6 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                  </div>
                  <span class="text-xl font-semibold text-gray-800">{{ order?.title || '-' }}</span>
                </div>
              </div>

              <h4 class="text-base font-semibold text-gray-700 mb-3">基本信息</h4>
              <Descriptions :column="3" size="small" class="order-info-desc">
                <DescriptionsItem label="订单编号">{{ order?.orderNo || '-' }}</DescriptionsItem>
                <DescriptionsItem label="订单标题">{{ order?.title || '-' }}</DescriptionsItem>
                <DescriptionsItem label="客户">{{ order?.customerName || '-' }}</DescriptionsItem>
                <DescriptionsItem label="订单金额">{{ order?.totalAmount?.toString() || '-' }}</DescriptionsItem>
              </Descriptions>
            </div>

            <!-- 右侧：审批流程时间线 -->
            <div class="w-64 flex-shrink-0 border-l border-gray-100 pl-6">
              <div class="relative">
                <div v-if="instance" class="timeline-item flex items-start gap-3 pb-6">
                  <div class="relative z-10 flex flex-col items-center">
                    <div class="w-10 h-10 rounded-full bg-blue-500 flex items-center justify-center ring-4 ring-blue-100">
                      <svg class="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                      </svg>
                    </div>
                    <div class="w-0.5 flex-1 bg-gray-200 mt-1 min-h-[20px]"></div>
                  </div>
                  <div class="flex-1 pt-1">
                    <div class="flex items-center gap-2">
                      <span class="font-semibold text-gray-800">发起人</span>
                      <Tag color="green" class="m-0">✓</Tag>
                    </div>
                    <div class="text-sm text-gray-500 mt-0.5">{{ instance.submitterName }}</div>
                    <div class="text-xs text-gray-400 mt-0.5">{{ formatDateTime(instance.submittedAt) }}</div>
                  </div>
                </div>

                <template v-for="(log, idx) in (instance?.logs || [])" :key="idx">
                  <div class="timeline-item flex items-start gap-3 pb-6">
                    <div class="relative z-10 flex flex-col items-center">
                      <div
                        class="w-10 h-10 rounded-full flex items-center justify-center ring-4"
                        :class="log.action === 1 ? 'bg-green-500 ring-green-100' : 'bg-red-500 ring-red-100'"
                      >
                        <svg v-if="log.action === 1" class="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                        </svg>
                        <svg v-else class="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </div>
                      <div
                        v-if="Number(idx) < ((instance?.logs?.length || 0) - 1) || (instance?.status === 1 || instance?.status === 2)"
                        class="w-0.5 flex-1 bg-gray-200 mt-1 min-h-[20px]"
                      ></div>
                    </div>
                    <div class="flex-1 pt-1">
                      <div class="flex items-center gap-2">
                        <span class="font-semibold text-gray-800">{{ log.nodeName || '审批人' }}</span>
                        <Tag :color="actionTextMap[log.action]?.color" class="m-0">
                          {{ actionTextMap[log.action]?.label }}
                        </Tag>
                      </div>
                      <div class="text-sm text-gray-500 mt-0.5">{{ log.approverName }}</div>
                      <div class="text-xs text-gray-400 mt-0.5">{{ formatDateTime(log.createTime) }}</div>
                      <div v-if="log.comment" class="text-sm text-gray-600 mt-1 bg-gray-50 rounded px-2 py-1">
                        {{ log.comment }}
                      </div>
                    </div>
                  </div>
                </template>

                <div v-if="instance && (instance.status === 1 || instance.status === 2)" class="timeline-item flex items-start gap-3 pb-6">
                  <div class="relative z-10 flex flex-col items-center">
                    <div class="w-10 h-10 rounded-full bg-blue-500 flex items-center justify-center ring-4 ring-blue-100 animate-pulse">
                      <svg class="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                      </svg>
                    </div>
                  </div>
                  <div class="flex-1 pt-1">
                    <div class="flex items-center gap-2 flex-wrap">
                      <span class="font-semibold text-blue-600">审批人</span>
                      <Tag color="processing" class="m-0">待审批</Tag>
                      <Tag v-if="instance?.approveMode && approveModeMap[instance.approveMode]" :color="approveModeMap[instance.approveMode]?.color" class="m-0">
                        {{ approveModeMap[instance.approveMode]?.label }}
                      </Tag>
                    </div>
                    <!-- 候选审批人列表 -->
                    <div v-if="instance.candidateApproverNames?.length > 0" class="mt-1.5 flex flex-wrap gap-1.5">
                      <span
                        v-for="(name, idx) in instance.candidateApproverNames"
                        :key="idx"
                        class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs"
                        :class="instance.processedApprovers?.includes(instance.candidateApprovers?.[idx]) ? 'bg-green-100 text-green-700' : 'bg-blue-50 text-blue-600'"
                      >
                        <svg v-if="instance.processedApprovers?.includes(instance.candidateApprovers?.[idx])" class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3"><path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" /></svg>
                        {{ name }}
                      </span>
                    </div>
                    <div v-else class="text-sm text-blue-500 mt-0.5">{{ instance.currentApproverName || '未分配' }}</div>
                    <!-- 进度提示 -->
                    <div v-if="instance.candidateApprovers?.length > 1 && instance.approveMode !== 1" class="text-xs text-gray-400 mt-1">
                      已处理 {{ instance.processedApprovers?.length || 0 }} / {{ instance.candidateApprovers?.length || 0 }} 人
                    </div>
                  </div>
                </div>

                <div v-if="instance && instance.status === 3" class="timeline-item flex items-start gap-3">
                  <div class="relative z-10 flex flex-col items-center">
                    <div class="w-10 h-10 rounded-full bg-green-500 flex items-center justify-center ring-4 ring-green-100">
                      <svg class="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                      </svg>
                    </div>
                  </div>
                  <div class="flex-1 pt-1">
                    <div class="flex items-center gap-2">
                      <span class="font-semibold text-green-700">结束</span>
                      <Tag color="success" class="m-0">审批通过</Tag>
                    </div>
                    <div class="text-xs text-gray-400 mt-0.5">{{ formatDateTime(instance.finishedAt) }}</div>
                  </div>
                </div>

                <div v-if="instance && instance.status === 4" class="timeline-item flex items-start gap-3">
                  <div class="relative z-10 flex flex-col items-center">
                    <div class="w-10 h-10 rounded-full bg-red-500 flex items-center justify-center ring-4 ring-red-100">
                      <svg class="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </div>
                  </div>
                  <div class="flex-1 pt-1">
                    <div class="flex items-center gap-2">
                      <span class="font-semibold text-red-700">结束</span>
                      <Tag color="error" class="m-0">已驳回</Tag>
                    </div>
                    <div class="text-xs text-gray-400 mt-0.5">{{ formatDateTime(instance.finishedAt) }}</div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- ====== 流程图 Tab ====== -->
          <div v-if="activeTab === 'flow'" class="flow-diagram">
            <div class="bg-gray-50 rounded-lg p-8 min-h-[400px] overflow-x-auto">
              <div class="flex items-center justify-center gap-2 flex-wrap">
                <div class="flow-node flex items-center gap-2">
                  <div class="rounded-lg border-2 border-green-300 bg-green-50 px-5 py-3 text-center min-w-[100px]">
                    <div class="flex items-center justify-center gap-1.5">
                      <svg class="w-4 h-4 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M16 7a4 4 0 11-8 0 4 4 0 018 0z" />
                      </svg>
                      <span class="text-sm font-medium text-green-700">发起人</span>
                    </div>
                    <div class="text-xs text-gray-500 mt-1">{{ instance?.submitterName || '-' }}</div>
                  </div>
                  <span class="text-gray-300 text-xl">→</span>
                </div>

                <template v-for="(node, idx) in flowNodesOrdered.filter(n => n.nodeType === 2)" :key="node.nodeKey">
                  <div class="flow-node flex items-center gap-2">
                    <div
                      class="rounded-lg border px-5 py-3 text-center min-w-[120px] transition-all"
                      :class="[
                        nodeStatusMap[node.nodeStatus]?.bgClass,
                        nodeStatusMap[node.nodeStatus]?.borderClass,
                        node.nodeStatus === 1 ? 'animate-pulse' : ''
                      ]"
                    >
                      <div class="flex items-center justify-center gap-1.5">
                        <svg v-if="node.nodeStatus === 2" class="w-4 h-4 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                        </svg>
                        <svg v-else-if="node.nodeStatus === 3" class="w-4 h-4 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                        <svg v-else-if="node.nodeStatus === 1" class="w-4 h-4 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <svg v-else class="w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M16 7a4 4 0 11-8 0 4 4 0 018 0z" />
                        </svg>
                        <span class="text-sm font-medium" :class="nodeStatusMap[node.nodeStatus]?.textClass">
                          {{ node.nodeName }}
                        </span>
                      </div>
                      <div class="text-xs mt-1" :class="node.nodeStatus === 0 ? 'text-gray-400' : 'text-gray-500'">
                        {{ node.approverName || '-' }}
                      </div>
                    </div>
                    <span
                      v-if="idx < flowNodesOrdered.filter(n => n.nodeType === 2).length - 1 || instance?.status !== 3"
                      class="text-gray-300 text-xl"
                    >→</span>
                  </div>
                </template>

                <div class="flow-node">
                  <div
                    class="rounded-lg border px-5 py-3 text-center min-w-[100px]"
                    :class="instance?.status === 3 ? 'border-green-300 bg-green-50' : instance?.status === 4 ? 'border-red-300 bg-red-50' : 'border-gray-200 bg-gray-50'"
                  >
                    <div class="flex items-center justify-center gap-1.5">
                      <svg class="w-4 h-4" :class="instance?.status === 3 ? 'text-green-600' : instance?.status === 4 ? 'text-red-600' : 'text-gray-400'" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                      </svg>
                      <span
                        class="text-sm font-medium"
                        :class="instance?.status === 3 ? 'text-green-700' : instance?.status === 4 ? 'text-red-700' : 'text-gray-400'"
                      >
                        {{ instance?.status === 3 ? '审批通过' : instance?.status === 4 ? '已驳回' : '结束' }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <div v-if="instance?.flowEdges?.some((e: any) => e.conditionExpr)" class="mt-6 flex items-center justify-center gap-4 text-sm text-gray-500">
                <span class="flex items-center gap-1">
                  <span class="inline-block w-3 h-3 rounded bg-green-100 border border-green-300"></span>
                  已通过
                </span>
                <span class="flex items-center gap-1">
                  <span class="inline-block w-3 h-3 rounded bg-blue-100 border-2 border-blue-400 animate-pulse"></span>
                  审批中
                </span>
                <span class="flex items-center gap-1">
                  <span class="inline-block w-3 h-3 rounded bg-red-100 border border-red-300"></span>
                  已驳回
                </span>
                <span class="flex items-center gap-1">
                  <span class="inline-block w-3 h-3 rounded bg-gray-50 border border-gray-200"></span>
                  未到达
                </span>
              </div>
            </div>
          </div>

          <!-- ====== 流转记录 Tab ====== -->
          <div v-if="activeTab === 'record'">
            <Table
              :columns="flowRecordColumns"
              :data-source="flowRecordData"
              :pagination="false"
              size="middle"
              bordered
            />
          </div>
        </div>

        <!-- ========== 底部操作栏 ========== -->
        <div v-if="showActionBar" class="border-t border-gray-200 px-6 py-4 bg-white">
          <div v-if="canApprove" class="mb-3">
            <Input.TextArea
              v-model:value="comment"
              :rows="2"
              placeholder="请输入审批意见（驳回时必填）"
            />
          </div>
          <div class="flex items-center justify-between flex-wrap gap-3">
            <div class="flex items-center gap-3">
              <Button v-if="canCancel" danger ghost @click="openModal('cancel')">
                <template #icon><svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" /></svg></template>
                撤销审批
              </Button>
              <Button class="text-gray-500" @click="emit('update:visible', false)">取消</Button>
            </div>
            <div class="flex items-center gap-2 flex-wrap">
              <Button v-if="canRejectTo" @click="openModal('rejectTo')">退回</Button>
              <Button v-if="canTransfer" @click="openModal('transfer')">转办</Button>
              <Button v-if="canDelegate" @click="openModal('delegate')">委派</Button>
              <Button v-if="canAddSign" @click="openModal('addSign')">加签</Button>
              <Button v-if="canCc" @click="openModal('addCc')">抄送</Button>
              <Button
                v-if="canApprove"
                danger
                :loading="actionLoading"
                size="large"
                @click="handleReject"
              >
                <template #icon><svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" /></svg></template>
                拒绝
              </Button>
              <Button
                v-if="canApprove"
                type="primary"
                :loading="actionLoading"
                size="large"
                class="!bg-green-500 !border-green-500"
                @click="handleApprove"
              >
                <template #icon><svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5"><path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" /></svg></template>
                通过
              </Button>
            </div>
          </div>
        </div>

        <div v-else class="border-t border-gray-200 px-6 py-3 bg-gray-50 flex justify-end">
          <Button @click="handleClose">关闭</Button>
        </div>
      </div>
    </Spin>

    <!-- ========== 高级审批操作弹窗 ========== -->
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
              const opt = userOptions.find(o => o.value === v);
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

        <!-- 撤销：撤回原因 -->
        <div v-if="modalState.type === 'cancel'">
          <div class="mb-2 text-sm text-gray-600">
            撤回原因（仅发起人可撤回进行中的审批）：
          </div>
          <Input.TextArea
            v-model:value="cancelReason"
            :rows="4"
            placeholder="请填写撤回原因"
          />
        </div>

        <!-- 退回 / 转办 / 委派 / 加签：审批意见 -->
        <div
          v-if="
            modalState.type &&
            ['addSign', 'delegate', 'rejectTo', 'transfer'].includes(modalState.type)
          "
        >
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
        <Button type="primary" @click="handleSubmit">确认</Button>
      </template>
    </Modal>
  </Drawer>
</template>

<style scoped>
.approval-container {
  overflow: hidden;
}
.approval-container :deep(.ant-tabs-nav) {
  margin-bottom: 0;
}
.approval-container :deep(.ant-tabs-tab) {
  font-size: 16px;
  padding: 12px 0;
}
.approval-container :deep(.ant-descriptions-item-label) {
  width: 90px;
  color: #6b7280;
}
.approval-container :deep(.ant-descriptions-item-content) {
  color: #1f2937;
}
.stamp-container img {
  transform: rotate(-15deg);
}
.approval-spin {
  height: 100%;
  display: flex;
}
.approval-spin :deep(.ant-spin-container) {
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>