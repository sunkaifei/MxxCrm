<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { createVNode, h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { CircleCheckBig, CircleX, LucideEye } from '@vben/icons';
import { useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Input, Modal, Tag, Timeline, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  getApprovalDetailApi,
  getApprovalListApi,
  processApprovalApi,
} from '#/api';
import { $t } from '#/locales';

const userStore = useUserStore();

const businessTypeMap: Record<string, { label: string; color: string }> = {
  contract: { label: '合同', color: 'blue' },
  order: { label: '订单', color: 'cyan' },
  purchase: { label: '采购', color: 'purple' },
  payment: { label: '付款', color: 'gold' },
  expense: { label: '报销', color: 'magenta' },
  leave: { label: '请假', color: 'orange' },
};

const approvalStatusList: Record<number, { label: string; color: string }> = {
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
};

const logActionText: Record<number, string> = {
  1: '提交',
  2: '审批通过',
  3: '驳回',
};

const detailVisible = ref(false);
const detailData = ref<any>(null);
const commentRef = ref('');

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
          { value: 1, label: '待审批' },
          { value: 2, label: '审批中' },
          { value: 3, label: '已通过' },
          { value: 4, label: '已驳回' },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  pagerConfig: {},
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getApprovalListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: '业务标题',
      field: 'businessTitle',
      minWidth: 180,
    },
    {
      title: '业务类型',
      field: 'businessType',
      width: 110,
      slots: { default: 'businessType' },
    },
    {
      title: '提交人',
      field: 'submitterName',
      width: 120,
    },
    {
      title: '提交时间',
      field: 'submittedAt',
      width: 170,
      slots: { default: 'submittedAt' },
    },
    {
      title: '状态',
      field: 'status',
      width: 110,
      slots: { default: 'status' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 240,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function canProcess(row: any) {
  return row.status === 1 || row.status === 2;
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
    title: action === 'approve' ? '审批通过' : '驳回审批',
    content: () =>
      createVNode('div', { style: 'padding-top:8px' }, [
        createVNode(
          'div',
          { style: 'margin-bottom:8px;color:#666' },
          action === 'approve'
            ? '请确认审批通过该申请'
            : '请填写驳回原因',
        ),
        createVNode(Input.TextArea, {
          value: commentRef.value,
          'onUpdate:value': (v: string) => {
            commentRef.value = v;
          },
          rows: 4,
          placeholder:
            action === 'approve' ? '可选填写审批意见' : '请填写驳回原因',
        }),
      ]),
    okText: '确认',
    cancelText: '取消',
    okButtonProps: action === 'reject' ? { danger: true } : {},
    async onOk() {
      await processApprovalApi({
        instanceId: row.id,
        action,
        approverId: userInfo?.userId,
        approverName: userInfo?.realName || userInfo?.username,
        comment: commentRef.value,
      });
      message.success(action === 'approve' ? '已审批通过' : '已驳回');
      gridApi.query();
    },
  });
}
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
        <Button
          type="link"
          :icon="h(LucideEye)"
          @click="openDetail(row)"
        >
          查看详情
        </Button>
        <Button
          v-if="canProcess(row)"
          type="link"
          :icon="h(CircleCheckBig)"
          @click="openProcess(row, 'approve')"
        >
          通过
        </Button>
        <Button
          v-if="canProcess(row)"
          type="link"
          danger
          :icon="h(CircleX)"
          @click="openProcess(row, 'reject')"
        >
          驳回
        </Button>
      </template>
    </Grid>

    <Modal
      v-model:visible="detailVisible"
      :title="detailData?.businessTitle || '审批详情'"
      width="720px"
      :footer="null"
      destroy-on-close
    >
      <div v-if="detailData" class="space-y-4">
        <div class="flex justify-between items-center">
          <span class="text-gray-500">业务类型：</span>
          <Tag :color="businessTypeMap[detailData.businessType]?.color || 'default'">
            {{ businessTypeMap[detailData.businessType]?.label || detailData.businessType }}
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
          <Tag :color="approvalStatusList[detailData.status]?.color || 'default'">
            {{ approvalStatusList[detailData.status]?.label || '未知' }}
          </Tag>
        </div>

        <div v-if="detailData.approvalLogs && detailData.approvalLogs.length > 0">
          <h4 class="text-lg font-semibold mb-4">审批记录</h4>
          <Timeline>
            <Timeline.Item
              v-for="log in detailData.approvalLogs"
              :key="log.id"
              :color="log.action === 2 ? 'green' : log.action === 3 ? 'red' : 'blue'"
            >
              <div class="font-medium">
                {{ log.nodeName || logActionText[log.action] || '审批' }}
              </div>
              <div class="text-gray-500 text-sm">
                {{ log.approverName || log.operatorName }} ·
                {{ logActionText[log.action] || '--' }} ·
                {{ formatDateTime(log.createTime || log.create_at) }}
              </div>
              <div v-if="log.comment || log.reason" class="text-gray-600 text-sm mt-1">
                {{ log.comment || log.reason }}
              </div>
            </Timeline.Item>
          </Timeline>
        </div>
        <div v-else class="text-gray-400">暂无审批记录</div>
      </div>
    </Modal>
  </Page>
</template>
