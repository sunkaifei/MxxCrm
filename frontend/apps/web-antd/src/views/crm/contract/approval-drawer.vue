<script lang="ts" setup>
import { computed, h, ref, watch } from 'vue';

import { formatDateTime } from '@vben/utils';

import {
  Avatar,
  Button,
  Descriptions,
  DescriptionsItem,
  Drawer,
  Input,
  Spin,
  TabPane,
  Table,
  Tabs,
  Tag,
} from 'ant-design-vue';

import {
  approveContractApi,
  getContractApprovalDetailApi,
  rejectContractApi,
} from '#/api';

const STAMP_APPROVED = '/images/approval-approved.svg';
const STAMP_PENDING = '/images/approval-pending.svg';
const STAMP_REJECTED = '/images/approval-rejected.svg';

const props = defineProps<{
  visible: boolean;
  contractId: number | null;
  currentUserId?: number;
}>();

const emit = defineEmits<{
  'update:visible': [val: boolean];
  success: [];
}>();

const loading = ref(false);
const detail = ref<any>(null);
const comment = ref('');
const actionLoading = ref(false);
const activeTab = ref('detail');

const instance = computed(() => detail.value?.instance);
const contract = computed(() => detail.value);

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
const nodeTypeMap: Record<number, string> = {
  1: '发起人',
  2: '审批人',
  3: '条件分支',
  4: '结束',
};

// 是否可审批（当前用户是审批人且状态为进行中）
const canApprove = computed(() => {
  if (!instance.value) return false;
  if (instance.value.status !== 1 && instance.value.status !== 2) return false;
  if (!props.currentUserId) return false;
  return instance.value.currentApproverId === props.currentUserId;
});

// 是否是发起人
const isSubmitter = computed(() => {
  if (!instance.value || !props.currentUserId) return false;
  return instance.value.submitterId === props.currentUserId;
});

// 是否只是查看者（既不是发起人也不是当前审批人）
const isViewer = computed(() => {
  return !canApprove.value && !isSubmitter.value;
});

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
  if (!props.contractId) return;
  loading.value = true;
  try {
    detail.value = await getContractApprovalDetailApi(props.contractId);
  } finally {
    loading.value = false;
  }
}

// 审批通过
async function handleApprove() {
  if (!props.contractId) return;
  actionLoading.value = true;
  try {
    await approveContractApi(props.contractId, comment.value || undefined);
    emit('success');
    emit('update:visible', false);
  } finally {
    actionLoading.value = false;
  }
}

// 驳回
async function handleReject() {
  if (!props.contractId) return;
  actionLoading.value = true;
  try {
    await rejectContractApi(props.contractId, comment.value || undefined);
    emit('success');
    emit('update:visible', false);
  } finally {
    actionLoading.value = false;
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
    if (val && props.contractId) {
      comment.value = '';
      activeTab.value = 'detail';
      loadDetail();
    }
  },
);
</script>

<template>
  <Drawer
    :open="visible"
    title="合同审批"
    placement="right"
    width="75%"
    :body-style="{ padding: 0, display: 'flex', flexDirection: 'column', height: '100%' }"
    :header-style="{ borderBottom: '1px solid #f0f0f0', padding: '16px 24px' }"
    destroy-on-close
    @close="emit('update:visible', false)"
  >
    <Spin :spinning="loading" class="approval-spin">
      <div v-if="detail" class="approval-container flex flex-col h-full">
        <!-- ========== 顶部区域：编号、标题、状态、提交人、印章 ========== -->
        <div class="approval-header relative px-6 pt-4 pb-0">
          <!-- 编号 + 打印 -->
          <div class="flex items-center justify-between text-sm text-gray-500 mb-3">
            <span>编号：{{ instance?.id || contractId }}</span>
          </div>

          <!-- 标题 + 状态标签 + 印章 -->
          <div class="flex items-start justify-between">
            <div>
              <div class="flex items-center gap-3">
                <h2 class="text-2xl font-bold text-gray-900 m-0">合同审批</h2>
                <Tag
                  v-if="instance"
                  :color="instanceStatusMap[instance.status]?.color"
                  class="text-sm px-3 py-0.5"
                >
                  {{ instanceStatusMap[instance.status]?.label }}
                </Tag>
                <Tag v-else color="default" class="text-sm px-3 py-0.5">草稿</Tag>
              </div>
              <!-- 提交人信息 -->
              <div v-if="instance" class="flex items-center gap-2 mt-3 text-gray-500">
                <Avatar :size="36" class="bg-blue-500 flex items-center justify-center text-white text-sm font-medium">
                  {{ getFirstChar(instance.submitterName) }}
                </Avatar>
                <span class="text-base text-gray-700 font-medium">{{ instance.submitterName }}</span>
                <span class="text-sm">{{ formatDateTime(instance.submittedAt) }} 提交</span>
              </div>
            </div>
            <!-- 印章 -->
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
            <!-- 左侧：合同基本信息 -->
            <div class="flex-1 min-w-0">
              <!-- 合同标题/编号卡片 -->
              <div class="border-b border-gray-100 pb-4 mb-4">
                <div class="flex items-center gap-3 mb-1">
                  <div class="w-10 h-10 rounded-lg bg-blue-50 flex items-center justify-center">
                    <svg class="w-6 h-6 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                  </div>
                  <span class="text-xl font-semibold text-gray-800">{{ contract.title || '-' }}</span>
                </div>
              </div>

              <!-- 基本信息 -->
              <h4 class="text-base font-semibold text-gray-700 mb-3">基本信息</h4>
              <Descriptions :column="3" size="small" class="contract-info-desc">
                <DescriptionsItem label="合同编号">{{ contract.contractNo || '-' }}</DescriptionsItem>
                <DescriptionsItem label="合同名称">{{ contract.title || '-' }}</DescriptionsItem>
                <DescriptionsItem label="客户">{{ contract.customerName || '-' }}</DescriptionsItem>
                <DescriptionsItem label="合同类型">{{ contract.contractType || '-' }}</DescriptionsItem>
                <DescriptionsItem label="合同金额">{{ contract.amount?.toString() || '-' }}</DescriptionsItem>
                <DescriptionsItem label="总金额">{{ contract.totalAmount?.toString() || '-' }}</DescriptionsItem>
              </Descriptions>
            </div>

            <!-- 右侧：审批流程时间线（竖向进度） -->
            <div class="w-64 flex-shrink-0 border-l border-gray-100 pl-6">
              <div class="relative">
                <!-- 发起人节点 -->
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

                <!-- 已审批的节点（从logs中） -->
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
                        v-if="idx < (instance?.logs?.length || 0) - 1 || (instance?.status === 1 || instance?.status === 2)"
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

                <!-- 当前待审批节点（脉冲动画） -->
                <div v-if="instance && (instance.status === 1 || instance.status === 2)" class="timeline-item flex items-start gap-3 pb-6">
                  <div class="relative z-10 flex flex-col items-center">
                    <div class="w-10 h-10 rounded-full bg-blue-500 flex items-center justify-center ring-4 ring-blue-100 animate-pulse">
                      <svg class="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                      </svg>
                    </div>
                  </div>
                  <div class="flex-1 pt-1">
                    <div class="flex items-center gap-2">
                      <span class="font-semibold text-blue-600">审批人</span>
                      <Tag color="processing" class="m-0">待审批</Tag>
                    </div>
                    <div class="text-sm text-blue-500 mt-0.5">{{ instance.currentApproverName || '未分配' }}</div>
                  </div>
                </div>

                <!-- 结束节点（已通过时显示） -->
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

                <!-- 驳回结束节点 -->
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
                <!-- 提交/发起人节点 -->
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

                <!-- 审批流程节点（从flowNodes中取审批类型节点） -->
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

                <!-- 结束节点 -->
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

              <!-- 条件标签说明 -->
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
        <div v-if="canApprove" class="border-t border-gray-200 px-6 py-4 bg-white">
          <div class="mb-3">
            <Input.TextArea
              v-model:value="comment"
              :rows="2"
              placeholder="请输入审批意见（驳回时必填）"
            />
          </div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <!-- 高级操作按钮（视觉参考，功能后续扩展） -->
              <Button class="text-gray-500" disabled>抄送</Button>
              <Button class="text-gray-500" disabled>转办</Button>
              <Button class="text-gray-500" disabled>委派</Button>
              <Button class="text-gray-500" disabled>加签</Button>
              <Button class="text-gray-500" disabled>退回</Button>
              <Button class="text-gray-500" @click="emit('update:visible', false)">取消</Button>
            </div>
            <div class="flex items-center gap-3">
              <Button
                danger
                :loading="actionLoading"
                size="large"
                @click="handleReject"
              >
                <template #icon><svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" /></svg></template>
                拒绝
              </Button>
              <Button
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

        <!-- 非审批人：只显示关闭按钮 -->
        <div v-else class="border-t border-gray-200 px-6 py-3 bg-gray-50 flex justify-end">
          <Button @click="emit('update:visible', false)">关闭</Button>
        </div>
      </div>
    </Spin>
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
