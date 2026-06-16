<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideEye, LucideCheckCircle } from '@vben/icons';
import { Button, Tag, Modal, Descriptions, message } from 'ant-design-vue';
import { settlementApi } from '#/api';

const detailModalVisible = ref(false);
const detailData = ref<any>(null);

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'status',
      label: '结算状态',
      componentProps: {
        options: [
          { label: '待结算', value: 0 },
          { label: '已结算', value: 1 },
          { label: '已付款', value: 2 },
        ],
        placeholder: '请选择状态',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'shopName',
      label: '店铺名称',
      componentProps: {
        placeholder: '请输入店铺名称',
        allowClear: true,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  rowConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await settlementApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status,
          shopId: formValues.shopId,
        });
      },
    },
  },

  columns: [
    {
      title: '序号',
      type: 'seq',
      width: 70,
    },
    {
      title: '结算单号',
      field: 'settlementNo',
      width: 180,
    },
    {
      title: '店铺名称',
      field: 'shopName',
      width: 150,
    },
    {
      title: '结算周期',
      field: 'period',
      slots: { default: 'period' },
      width: 180,
    },
    {
      title: '订单数量',
      field: 'orderCount',
      width: 100,
    },
    {
      title: '总交易额',
      field: 'totalAmount',
      slots: { default: 'totalAmount' },
      width: 120,
    },
    {
      title: '佣金',
      field: 'commissionAmount',
      slots: { default: 'commissionAmount' },
      width: 100,
    },
    {
      title: '结算金额',
      field: 'settlementAmount',
      slots: { default: 'settlementAmount' },
      width: 120,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 100,
    },
    {
      title: '付款时间',
      field: 'payTime',
      width: 160,
    },
    {
      title: '创建时间',
      field: 'createTime',
      width: 160,
    },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 200,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function getStatusText(status: number) {
  const map: Record<number, string> = { 0: '待结算', 1: '已结算', 2: '已付款' };
  return map[status] || '未知';
}

function getStatusColor(status: number) {
  const map: Record<number, string> = {
    0: 'warning',
    1: 'processing',
    2: 'success',
  };
  return map[status] || 'default';
}

async function handlePay(row: any) {
  row.pending = true;
  try {
    await settlementApi.pay({ id: row.id });
    message.success('付款成功');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function viewDetail(row: any) {
  detailData.value = await settlementApi.detail(row.id);
  detailModalVisible.value = true;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="结算管理">
      <template #period="{ row }">
        <span>{{ row.periodStart || '-' }} ~ {{ row.periodEnd || '-' }}</span>
      </template>

      <template #totalAmount="{ row }">
        <span>¥{{ (row.totalAmount / 100).toFixed(2) }}</span>
      </template>

      <template #commissionAmount="{ row }">
        <span>¥{{ (row.commissionAmount / 100).toFixed(2) }}</span>
      </template>

      <template #settlementAmount="{ row }">
        <span class="text-green-500 font-medium"
          >¥{{ (row.settlementAmount / 100).toFixed(2) }}</span
        >
      </template>

      <template #status="{ row }">
        <Tag :color="getStatusColor(row.status)">
          {{ getStatusText(row.status) }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideEye)"
          @click="() => viewDetail(row)"
        >
          详情
        </Button>
        <template v-if="row.status === 1">
          <Button
            type="primary"
            link
            :icon="h(LucideCheckCircle)"
            :loading="row.pending"
            @click="() => handlePay(row)"
          >
            确认付款
          </Button>
        </template>
      </template>
    </Grid>

    <Modal v-model:open="detailModalVisible" title="结算详情" width="800">
      <Descriptions :column="2" bordered>
        <Descriptions.Item label="结算单号">{{
          detailData?.settlementNo || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="状态">
          <Tag :color="getStatusColor(detailData?.status)">{{
            getStatusText(detailData?.status)
          }}</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="店铺名称">{{
          detailData?.shopName || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="结算周期"
          >{{ detailData?.periodStart || '-' }} ~
          {{ detailData?.periodEnd || '-' }}</Descriptions.Item
        >
        <Descriptions.Item label="订单数量"
          >{{ detailData?.orderCount || 0 }}单</Descriptions.Item
        >
        <Descriptions.Item label="付款时间">{{
          detailData?.payTime || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="总交易额"
          >¥{{
            ((detailData?.totalAmount || 0) / 100).toFixed(2)
          }}</Descriptions.Item
        >
        <Descriptions.Item label="平台佣金"
          >¥{{
            ((detailData?.commissionAmount || 0) / 100).toFixed(2)
          }}</Descriptions.Item
        >
        <Descriptions.Item label="结算金额" :span="2">
          <span class="text-green-500 text-xl font-medium"
            >¥{{ ((detailData?.settlementAmount || 0) / 100).toFixed(2) }}</span
          >
        </Descriptions.Item>
        <Descriptions.Item label="备注" :span="2" v-if="detailData?.remark">
          {{ detailData.remark }}
        </Descriptions.Item>
      </Descriptions>
    </Modal>
  </Page>
</template>
