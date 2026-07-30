<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, createVNode, h, onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { CircleCheckBig, CircleX, LucideEye } from '@vben/icons';
import { useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Input,
  Modal,
  Select,
  Tag,
  Timeline,
  message,
} from 'ant-design-vue';
import { useRoute, useRouter } from 'vue-router';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  addCcApprovalApi,
  addSignApprovalApi,
  cancelApprovalApi,
  delegateApprovalApi,
  getApprovalDetailApi,
  getApprovalListApi,
  processApprovalApi,
  rejectToApprovalApi,
  transferApprovalApi,
} from '#/api';
import { searchUsersApi } from '#/api/core/message/chat';
import { $t } from '#/locales';

const userStore = useUserStore();
const route = useRoute();
const router = useRouter();

const businessTypeMap: Record<string, { label: string; color: string }> = {
  contract: { label: '合同', color: 'geekblue' },
  expense: { label: '报销', color: 'volcano' },
  invoice: { label: '发票', color: 'purple' },
  leave: { label: '请假', color: 'orange' },
  order: { label: '订单', color: 'cyan' },
  payment: { label: '回款', color: 'gold' },
  purchase: { label: '采购', color: 'magenta' },
  quotation: { label: '报价单', color: 'blue' },
  refund: { label: '退款', color: 'red' },
  visit: { label: '外勤', color: 'lime' },
};

// 实例状态：1=待审批,2=审批中,3=已通过,4=已驳回,5=已撤回,6=待修改
const approvalStatusList: Record<number, { label: string; color: string }> = {
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
  5: { label: '已撤回', color: 'default' },
  6: { label: '待修改', color: 'orange' },
};

// 日志动作：1=通过,2=驳回,3=转办,4=委派,5=加签,6=退回,7=取消,8=抄送
const logActionText: Record<number, string> = {
  1: '审批通过',
  2: '驳回',
  3: '转办',
  4: '委派',
  5: '加签',
  6: '退回',
  7: '取消',
  8: '抄送',
};

const logActionColor: Record<number, string> = {
  1: 'green',
  2: 'red',
  3: 'blue',
  4: 'blue',
  5: 'orange',
  6: 'orange',
  7: 'gray',
  8: 'cyan',
};

const currentUserId = computed(() => userStore.userInfo?.userId);

const detailVisible = ref(false);
const detailData = ref<any>(null);
const commentRef = ref('');

// ============ 增强功能弹窗状态 ============
const modalState = ref<{
  type:
    | 'addCc'
    | 'addSign'
    | 'cancel'
    | 'delegate'
    | 'rejectTo'
    | 'transfer'
    | null;
  row: any;
}>({ type: null, row: null });

// 表单字段
const targetUserId = ref<null | number>(null);
const targetUserName = ref('');
const targetUserIds = ref<number[]>([]);
const addSignType = ref<1 | 2 | 3>(2); // 1=前加签,2=后加签,3=并加签
const rejectToNodeKey = ref<'' | null>(''); // '' 表示退回到发起人
const commentText = ref('');
const cancelReason = ref('');
const ccReason = ref('');

// 用户远程搜索
const userOptions = ref<{ label: string; value: number }[]>([]);
const userSearching = ref(false);
let userSearchTimer: any = null;

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'businessTitle',
      label: '业务标题',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: [
          { label: '待审批', value: 1 },
          { label: '审批中', value: 2 },
          { label: '已通过', value: 3 },
          { label: '已驳回', value: 4 },
          { label: '已撤回', value: 5 },
          { label: '待修改', value: 6 },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  cellConfig: {
    isHover: true,
  },
  columns: [
    {
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      title: $t('ui.table.action'),
      width: 420,
    },
    {
      field: 'businessTitle',
      minWidth: 180,
      title: '业务标题',
    },
    {
      field: 'businessType',
      slots: { default: 'businessType' },
      title: '业务类型',
      width: 110,
    },
    {
      field: 'submitterName',
      title: '提交人',
      width: 120,
    },
    {
      field: 'submittedAt',
      slots: { default: 'submittedAt' },
      title: '提交时间',
      width: 170,
    },
    {
      field: 'status',
      slots: { default: 'status' },
      title: '状态',
      width: 110,
    },
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
  ],
  height: 'auto',
  pagerConfig: {},
  proxyConfig: {
    ajax: {
      query: async ({ page }, formValues) => {
        return await getApprovalListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
    autoLoad: true,
  },
  stripe: true,
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// ============ 权限判断 ============
// 是否当前节点候选审批人（含转办/委派后扩展的池）
function isCandidateApprover(row: any) {
  const list: number[] = row.candidateApprovers || [];
  return currentUserId.value != null && list.includes(currentUserId.value);
}

// 实例是否处于可操作状态（待审批/审批中/待修改）
function isActionable(row: any) {
  return row.status === 1 || row.status === 2 || row.status === 6;
}

// 当前用户是否为发起人
function isSubmitter(row: any) {
  return (
    currentUserId.value != null && Number(row.submitterId) === currentUserId.value
  );
}

// 审批人按钮可见：候选池包含自己 + 实例待处理（1/2）
function canProcess(row: any) {
  return isCandidateApprover(row) && (row.status === 1 || row.status === 2);
}

// 审批人增强按钮可见：同 canProcess（退回/转办/委派/加签 仅审批人可操作）
function canApproverAction(row: any) {
  return canProcess(row);
}

// 抄送按钮可见：发起人 或 当前审批人
function canCc(row: any) {
  if (!isActionable(row)) return false;
  if (isSubmitter(row)) return true;
  return isCandidateApprover(row);
}

// 取消（撤回）按钮可见：仅发起人 + 实例待处理（1/2）
function canCancel(row: any) {
  return isSubmitter(row) && (row.status === 1 || row.status === 2);
}

async function openDetail(row: any) {
  try {
    const res: any = await getApprovalDetailApi(row.id);
    detailData.value = res?.data?.data ?? res?.data ?? res ?? null;
    detailVisible.value = true;
  } catch (e: any) {
    message.error(e?.message || '加载详情失败');
  }
}

function openProcess(row: any, action: 'approve' | 'reject') {
  commentRef.value = '';
  const userInfo = userStore.userInfo;
  Modal.confirm({
    cancelText: '取消',
    content: () =>
      createVNode('div', { style: 'padding-top:8px' }, [
        createVNode(
          'div',
          { style: 'margin-bottom:8px;color:#666' },
          action === 'approve' ? '请确认审批通过该申请' : '请填写驳回原因',
        ),
        createVNode(Input.TextArea, {
          'onUpdate:value': (v: string) => {
            commentRef.value = v;
          },
          rows: 4,
          placeholder:
            action === 'approve' ? '可选填写审批意见' : '请填写驳回原因',
          value: commentRef.value,
        }),
      ]),
    okButtonProps: action === 'reject' ? { danger: true } : {},
    okText: '确认',
    async onOk() {
      await processApprovalApi({
        action: action === 'approve' ? 1 : 2,
        approverId: userInfo?.userId,
        approverName: userInfo?.realName || userInfo?.username,
        comment: commentRef.value,
        instanceId: row.id,
      });
      message.success(action === 'approve' ? '已审批通过' : '已驳回');
      gridApi.query();
    },
    title: action === 'approve' ? '审批通过' : '驳回审批',
  });
}

// ============ 用户远程搜索 ============
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
    } catch (e) {
      userOptions.value = [];
    } finally {
      userSearching.value = false;
    }
  }, 300);
}

function resetModalForm() {
  targetUserId.value = null;
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
  row: any,
) {
  resetModalForm();
  modalState.value = { type, row };
}

function closeModal() {
  modalState.value = { type: null, row: null };
}

const modalTitle = computed(() => {
  const map: Record<string, string> = {
    addCc: '添加抄送',
    addSign: '加签',
    cancel: '撤回审批',
    delegate: '委派审批人',
    rejectTo: '退回审批',
    transfer: '转办审批',
  };
  return modalState.value.type ? map[modalState.value.type] : '';
});

// 退回节点选项：基于详情中的 flowNodes（审批过的节点）
const rejectNodeOptions = computed(() => {
  const row = modalState.value.row;
  if (!row || !detailData.value) return [];
  // 若详情中携带审批流节点，则提供"退回到发起人"+"已流转过的节点"
  const nodes: any[] = detailData.value.flowNodes || [];
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

// ============ 提交处理 ============
async function handleSubmit() {
  const { type, row } = modalState.value;
  if (!type || !row) return;
  try {
    switch (type) {
      case 'cancel': {
        await cancelApprovalApi({
          cancelReason: cancelReason.value || undefined,
          instanceId: row.id,
        });
        message.success('已撤回审批');
        break;
      }
      case 'rejectTo': {
        await rejectToApprovalApi({
          comment: commentText.value || undefined,
          instanceId: row.id,
          rejectToNodeKey:
            rejectToNodeKey.value === '' ? undefined : rejectToNodeKey.value || undefined,
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
          instanceId: row.id,
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
          instanceId: row.id,
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
          instanceId: row.id,
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
          instanceId: row.id,
          userIds: targetUserIds.value,
        });
        message.success('已添加抄送');
        break;
      }
    }
    closeModal();
    gridApi.query();
    // 同步刷新已打开的详情
    if (detailVisible.value && detailData.value?.id === row.id) {
      openDetail(row);
    }
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  }
}

// 从工作台待办跳转：通过 query 参数 instanceId 自动打开审批详情 Modal
onMounted(async () => {
  const instanceId = route.query.instanceId;
  if (instanceId) {
    await openDetail({ id: Number(instanceId) });
    // 清除 query 参数，避免刷新后重复打开
    router.replace({ query: {} });
  }
});
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="审批待办">
      <template #businessType="{ row }">
        <Tag :color="businessTypeMap[row.businessType]?.color || 'default'">
          {{ businessTypeMap[row.businessType]?.label || row.businessType }}
        </Tag>
      </template>

      <template #submittedAt="{ row }">
        {{ formatDateTime(row.submittedAt || row.createdAt) }}
      </template>

      <template #status="{ row }">
        <Tag :color="approvalStatusList[row.status]?.color || 'default'">
          {{ approvalStatusList[row.status]?.label || '未知' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <div class="flex flex-wrap gap-1">
          <Button
            :icon="h(LucideEye)"
            size="small"
            type="link"
            @click="openDetail(row)"
          >
            详情
          </Button>
          <Button
            v-if="canProcess(row)"
            :icon="h(CircleCheckBig)"
            size="small"
            type="link"
            @click="openProcess(row, 'approve')"
          >
            通过
          </Button>
          <Button
            v-if="canProcess(row)"
            :icon="h(CircleX)"
            danger
            size="small"
            type="link"
            @click="openProcess(row, 'reject')"
          >
            驳回
          </Button>
          <Button
            v-if="canApproverAction(row)"
            size="small"
            type="link"
            @click="openModal('rejectTo', row)"
          >
            退回
          </Button>
          <Button
            v-if="canApproverAction(row)"
            size="small"
            type="link"
            @click="openModal('transfer', row)"
          >
            转办
          </Button>
          <Button
            v-if="canApproverAction(row)"
            size="small"
            type="link"
            @click="openModal('delegate', row)"
          >
            委派
          </Button>
          <Button
            v-if="canApproverAction(row)"
            size="small"
            type="link"
            @click="openModal('addSign', row)"
          >
            加签
          </Button>
          <Button
            v-if="canCc(row)"
            size="small"
            type="link"
            @click="openModal('addCc', row)"
          >
            抄送
          </Button>
          <Button
            v-if="canCancel(row)"
            danger
            size="small"
            type="link"
            @click="openModal('cancel', row)"
          >
            取消
          </Button>
        </div>
      </template>
    </Grid>

    <!-- 审批详情 Modal -->
    <Modal
      v-model:visible="detailVisible"
      :footer="null"
      :title="detailData?.businessTitle || '审批详情'"
      destroy-on-close
      width="760px"
    >
      <div v-if="detailData" class="space-y-4">
        <div class="flex justify-between items-center">
          <span class="text-gray-500">业务类型：</span>
          <Tag
            :color="businessTypeMap[detailData.businessType]?.color || 'default'"
          >
            {{
              businessTypeMap[detailData.businessType]?.label ||
              detailData.businessType
            }}
          </Tag>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-gray-500">提交人：</span>
          <span>{{ detailData.submitterName }}</span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-gray-500">提交时间：</span>
          <span>{{ formatDateTime(detailData.submittedAt) }}</span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-gray-500">当前状态：</span>
          <Tag
            :color="approvalStatusList[detailData.status]?.color || 'default'"
          >
            {{ approvalStatusList[detailData.status]?.label || '未知' }}
          </Tag>
        </div>

        <!-- 候选审批人列表 -->
        <div
          v-if="
            detailData.candidateApproverNames?.length > 0 &&
            (detailData.status === 1 || detailData.status === 2)
          "
        >
          <div class="flex justify-between items-start mb-2">
            <span class="text-gray-500">候选审批人：</span>
            <div class="flex flex-wrap gap-1.5 justify-end max-w-[70%]">
              <span
                v-for="(name, idx) in detailData.candidateApproverNames"
                :key="idx"
                class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs"
                :class="
                  detailData.processedApprovers?.includes(
                    detailData.candidateApprovers?.[idx],
                  )
                    ? 'bg-green-100 text-green-700'
                    : 'bg-blue-50 text-blue-600'
                "
              >
                <svg
                  v-if="
                    detailData.processedApprovers?.includes(
                      detailData.candidateApprovers?.[idx],
                    )
                  "
                  class="w-3 h-3"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="3"
                  viewBox="0 0 24 24"
                >
                  <path
                    d="M5 13l4 4L19 7"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
                {{ name }}
              </span>
            </div>
          </div>
          <div
            v-if="detailData.candidateApprovers?.length > 1"
            class="text-right text-xs text-gray-400"
          >
            已处理 {{ detailData.processedApprovers?.length || 0 }} /
            {{ detailData.candidateApprovers?.length || 0 }} 人
          </div>
        </div>

        <!-- 详情内快捷操作（仅在状态为待审批/审批中时显示） -->
        <div
          v-if="canProcess(detailData) || canCancel(detailData) || canCc(detailData)"
          class="flex flex-wrap gap-2 pt-2 border-t border-gray-100"
        >
          <Button
            v-if="canProcess(detailData)"
            type="primary"
            size="small"
            @click="openProcess(detailData, 'approve')"
          >
            通过
          </Button>
          <Button
            v-if="canProcess(detailData)"
            danger
            size="small"
            @click="openProcess(detailData, 'reject')"
          >
            驳回
          </Button>
          <Button
            v-if="canApproverAction(detailData)"
            size="small"
            @click="openModal('rejectTo', detailData)"
          >
            退回
          </Button>
          <Button
            v-if="canApproverAction(detailData)"
            size="small"
            @click="openModal('transfer', detailData)"
          >
            转办
          </Button>
          <Button
            v-if="canApproverAction(detailData)"
            size="small"
            @click="openModal('delegate', detailData)"
          >
            委派
          </Button>
          <Button
            v-if="canApproverAction(detailData)"
            size="small"
            @click="openModal('addSign', detailData)"
          >
            加签
          </Button>
          <Button
            v-if="canCc(detailData)"
            size="small"
            @click="openModal('addCc', detailData)"
          >
            抄送
          </Button>
          <Button
            v-if="canCancel(detailData)"
            danger
            size="small"
            @click="openModal('cancel', detailData)"
          >
            取消
          </Button>
        </div>

        <!-- 审批记录 -->
        <div v-if="detailData.approvalLogs && detailData.approvalLogs.length > 0">
          <h4 class="text-lg font-semibold mb-4">审批记录</h4>
          <Timeline>
            <Timeline.Item
              v-for="log in detailData.approvalLogs"
              :key="log.id"
              :color="logActionColor[log.action] || 'blue'"
            >
              <div class="font-medium">
                {{ log.nodeName || logActionText[log.action] || '审批' }}
              </div>
              <div class="text-gray-500 text-sm">
                {{ log.approverName || log.operatorName }} ·
                {{ logActionText[log.action] || '--' }} ·
                {{ formatDateTime(log.createTime || log.create_at) }}
              </div>
              <div
                v-if="log.comment || log.reason"
                class="text-gray-600 text-sm mt-1"
              >
                {{ log.comment || log.reason }}
              </div>
            </Timeline.Item>
          </Timeline>
        </div>
        <div v-else class="text-gray-400">暂无审批记录</div>
      </div>
    </Modal>

    <!-- 增强功能统一弹窗 -->
    <Modal
      v-model:visible="modalState.type"
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

        <!-- 加签：多选用户 + 类型选择 -->
        <div v-if="modalState.type === 'addSign'">
          <div class="mb-2 text-sm text-gray-600">加签类型：</div>
          <Select
            v-model:value="addSignType"
            :options="[
              { label: '前加签（先由加签人审批，再由我审批）', value: 1 },
              { label: '后加签（我通过后，再由加签人审批）', value: 2 },
              { label: '并加签（与同时审批，并行处理）', value: 3 },
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

        <!-- 取消：撤回原因 -->
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

        <!-- 抄送理由 -->
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
  </Page>
</template>
