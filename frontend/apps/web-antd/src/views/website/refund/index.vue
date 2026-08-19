<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import {
  LucideCheckCircle,
  LucideEye,
  LucideReceipt,
  LucideTrash2,
} from '@vben/icons';

import {
  Button,
  message,
  Modal,
  Radio,
  RadioGroup,
  Select,
  Tag,
  Textarea,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { refundApi } from '#/api/core/website/refund';

defineOptions({ name: 'WebsiteRefund' });

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '待审核', value: 0 },
          { label: '已通过', value: 1 },
          { label: '已拒绝', value: 2 },
          { label: '已退款', value: 3 },
          { label: '已取消', value: 4 },
        ],
        placeholder: '请选择状态',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'refundNo',
      label: '退款单号',
      componentProps: {
        placeholder: '请输入退款单号',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'orderNo',
      label: '订单号',
      componentProps: {
        placeholder: '请输入订单号',
        allowClear: true,
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
  cellConfig: {},
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await refundApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status ?? undefined,
          refundNo: formValues.refundNo || undefined,
          orderNo: formValues.orderNo || undefined,
        });
      },
      delete: async ({ body }) => {
        await refundApi.batchDelete(body.removeRecords);
      },
    },
  },
  columns: [
    { title: '序号', type: 'seq', width: 70 },
    { title: '退款单号', field: 'refundNo', width: 200 },
    { title: '订单号', field: 'orderNo', width: 200 },
    { title: '用户ID', field: 'userId', width: 90 },
    {
      title: '退款类型',
      field: 'refundType',
      width: 100,
      slots: { default: 'refundType' },
    },
    { title: '退款金额', field: 'refundAmount', width: 110 },
    {
      title: '状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: '退款原因',
      field: 'refundReason',
      minWidth: 200,
      slots: { default: 'refundReason' },
    },
    { title: '申请时间', field: 'createTime', width: 170 },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 280,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 详情弹窗
const detailVisible = ref(false);
const detailData = ref<any>({});

function handleView(row: any) {
  detailData.value = row;
  detailVisible.value = true;
}

// 审核弹窗
const handleVisible = ref(false);
const handleRow = ref<any>({});
const handleForm = ref({
  action: 0,
  refundWay: 1,
  handleRemark: '',
});

function openHandleModal(row: any) {
  handleRow.value = row;
  handleForm.value = {
    action: 0,
    refundWay: 1,
    handleRemark: '',
  };
  handleVisible.value = true;
}

async function handleAudit() {
  if (!handleForm.value.handleRemark) {
    message.warning('请输入审核备注');
    return;
  }
  try {
    await refundApi.handle(handleRow.value.id, handleForm.value);
    message.success('审核成功');
    handleVisible.value = false;
    gridApi.query();
  } catch {
    message.error('审核失败');
  }
}

// 标记已退款
function handleMarkRefunded(row: any) {
  Modal.confirm({
    title: '确认操作',
    content: `确定要将退款单"${row.refundNo}"标记为已退款吗？`,
    onOk: async () => {
      try {
        await refundApi.markRefunded(row.id);
        message.success('操作成功');
        gridApi.query();
      } catch {
        message.error('操作失败');
      }
    },
  });
}

// 删除
async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除退款单"${row.refundNo}"吗？`,
    okType: 'danger',
    onOk: async () => {
      try {
        await refundApi.batchDelete([row.id]);
        message.success('删除成功');
        gridApi.query();
      } catch {
        message.error('删除失败');
      }
    },
  });
}

function truncate(text: string | undefined, len = 40): string {
  if (!text) return '—';
  return text.length > len ? `${text.slice(0, len)}...` : text;
}

function formatAmount(val: any): string {
  if (val === null || val === undefined || val === '') return '—';
  return `¥${Number(val).toFixed(2)}`;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="退款管理">
      <template #refundType="{ row }">
        <Tag v-if="row.refundType === 1" color="blue">仅退款</Tag>
        <Tag v-else-if="row.refundType === 2" color="purple">退货退款</Tag>
        <span v-else>—</span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 0" color="orange">待审核</Tag>
        <Tag v-else-if="row.status === 1" color="blue">已通过</Tag>
        <Tag v-else-if="row.status === 2" color="red">已拒绝</Tag>
        <Tag v-else-if="row.status === 3" color="success">已退款</Tag>
        <Tag v-else-if="row.status === 4" color="default">已取消</Tag>
        <Tag v-else color="default">未知</Tag>
      </template>

      <template #refundReason="{ row }">
        <span :title="row.refundReason">{{ truncate(row.refundReason) }}</span>
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideEye)"
          @click="() => handleView(row)"
        >
          详情
        </Button>
        <Button
          v-if="row.status === 0"
          type="primary"
          link
          :icon="h(LucideCheckCircle)"
          @click="() => openHandleModal(row)"
        >
          审核
        </Button>
        <Button
          v-if="row.status === 1"
          type="primary"
          link
          :icon="h(LucideReceipt)"
          @click="() => handleMarkRefunded(row)"
        >
          标记已退款
        </Button>
        <Button
          type="primary"
          link
          danger
          :icon="h(LucideTrash2)"
          @click="() => handleDelete(row)"
        >
          删除
        </Button>
      </template>
    </Grid>

    <!-- 详情弹窗 -->
    <Modal
      v-model:open="detailVisible"
      title="退款详情"
      width="640px"
      :footer="null"
    >
      <div class="space-y-3">
        <div class="flex">
          <span class="w-24 text-gray-500">退款单号：</span>
          <span>{{ detailData.refundNo || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">订单号：</span>
          <span>{{ detailData.orderNo || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">用户ID：</span>
          <span>{{ detailData.userId || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">退款类型：</span>
          <span>
            {{
              detailData.refundType === 1
                ? '仅退款'
                : detailData.refundType === 2
                  ? '退货退款'
                  : '—'
            }}
          </span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">退款金额：</span>
          <span class="font-medium text-red-500">
            {{ formatAmount(detailData.refundAmount) }}
          </span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">状态：</span>
          <Tag v-if="detailData.status === 0" color="orange">待审核</Tag>
          <Tag v-else-if="detailData.status === 1" color="blue">已通过</Tag>
          <Tag v-else-if="detailData.status === 2" color="red">已拒绝</Tag>
          <Tag v-else-if="detailData.status === 3" color="success">已退款</Tag>
          <Tag v-else-if="detailData.status === 4" color="default">已取消</Tag>
          <span v-else>—</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">退款方式：</span>
          <span>
            {{
              detailData.refundWay === 1
                ? '原路退回'
                : detailData.refundWay === 2
                  ? '余额'
                  : '—'
            }}
          </span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">申请时间：</span>
          <span>{{ detailData.createTime || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">处理时间：</span>
          <span>{{ detailData.handleTime || '—' }}</span>
        </div>
        <div>
          <span class="w-24 text-gray-500 inline-block">退款原因：</span>
          <div class="mt-2 rounded bg-gray-50 p-3 text-sm leading-relaxed">
            {{ detailData.refundReason || '—' }}
          </div>
        </div>
        <div>
          <span class="w-24 text-gray-500 inline-block">审核备注：</span>
          <div class="mt-2 rounded bg-gray-50 p-3 text-sm leading-relaxed">
            {{ detailData.handleRemark || '—' }}
          </div>
        </div>
      </div>
    </Modal>

    <!-- 审核弹窗 -->
    <Modal
      v-model:open="handleVisible"
      title="退款审核"
      width="520px"
      @ok="handleAudit"
    >
      <div class="space-y-4 py-2">
        <div class="flex items-center gap-3">
          <span class="w-24">退款单号：</span>
          <span>{{ handleRow.refundNo || '—' }}</span>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-24">订单号：</span>
          <span>{{ handleRow.orderNo || '—' }}</span>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-24">退款金额：</span>
          <span class="font-medium text-red-500">
            {{ formatAmount(handleRow.refundAmount) }}
          </span>
        </div>
        <div class="flex items-start gap-3">
          <span class="w-24 shrink-0">审核结果：</span>
          <RadioGroup v-model:value="handleForm.action" class="flex flex-col">
            <Radio :value="0">通过</Radio>
            <Radio :value="1">拒绝</Radio>
          </RadioGroup>
        </div>
        <div v-if="handleForm.action === 0" class="flex items-center gap-3">
          <span class="w-24">退款方式：</span>
          <Select
            v-model:value="handleForm.refundWay"
            style="flex: 1"
            :options="[
              { label: '原路退回', value: 1 },
              { label: '余额', value: 2 },
            ]"
          />
        </div>
        <div class="flex items-start gap-3">
          <span class="w-24 shrink-0">
            <span class="text-red-500">*</span> 审核备注：
          </span>
          <Textarea
            v-model:value="handleForm.handleRemark"
            placeholder="请输入审核备注"
            :rows="3"
            style="flex: 1"
          />
        </div>
      </div>
    </Modal>
  </Page>
</template>
