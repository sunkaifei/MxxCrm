<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideEye } from '@vben/icons';
import { Button, Tag, Modal, Descriptions } from 'ant-design-vue';
import { orderApi } from '#/api';

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
      label: '订单状态',
      componentProps: {
        options: [
          { label: '待付款', value: 0 },
          { label: '待发货', value: 1 },
          { label: '已发货', value: 2 },
          { label: '已签收', value: 3 },
          { label: '已完成', value: 4 },
          { label: '已取消', value: 5 },
          { label: '退款中', value: 6 },
          { label: '已退款', value: 7 },
        ],
        placeholder: '请选择状态',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'orderNo',
      label: '订单编号',
      componentProps: {
        placeholder: '请输入订单编号',
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
    {
      component: 'RangePicker',
      fieldName: 'dateRange',
      label: '下单时间',
      componentProps: {
        allowClear: true,
        placeholder: ['开始时间', '结束时间'],
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
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await orderApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status,
          orderNo: formValues.orderNo,
          shopId: formValues.shopId,
          startTime: formValues.dateRange?.[0]?.format('YYYY-MM-DD HH:mm:ss'),
          endTime: formValues.dateRange?.[1]?.format('YYYY-MM-DD HH:mm:ss'),
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
      title: '订单编号',
      field: 'orderNo',
      width: 180,
    },
    {
      title: '店铺名称',
      field: 'shopName',
      width: 150,
    },
    {
      title: '商品数量',
      field: 'goodsCount',
      width: 100,
    },
    {
      title: '订单金额',
      field: 'totalAmount',
      slots: { default: 'totalAmount' },
      width: 120,
    },
    {
      title: '运费',
      field: 'freightAmount',
      slots: { default: 'freightAmount' },
      width: 100,
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
      title: '收货人',
      field: 'receiverName',
      width: 100,
    },
    {
      title: '收货电话',
      field: 'receiverPhone',
      width: 120,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 100,
    },
    {
      title: '下单时间',
      field: 'createTime',
      width: 160,
    },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 120,
    },
  ],
};

const [Grid] = useVbenVxeGrid({ gridOptions, formOptions });

function getStatusText(status: number) {
  const map: Record<number, string> = {
    0: '待付款',
    1: '待发货',
    2: '已发货',
    3: '已签收',
    4: '已完成',
    5: '已取消',
    6: '退款中',
    7: '已退款',
  };
  return map[status] || '未知';
}

function getStatusColor(status: number) {
  const map: Record<number, string> = {
    0: 'warning',
    1: 'processing',
    2: 'blue',
    3: 'info',
    4: 'success',
    5: 'default',
    6: 'orange',
    7: 'error',
  };
  return map[status] || 'default';
}

async function viewDetail(row: any) {
  detailData.value = await orderApi.detail(row.id);
  detailModalVisible.value = true;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="订单管理">
      <template #totalAmount="{ row }">
        <span class="text-red-500 font-medium"
          >¥{{ (row.totalAmount / 100).toFixed(2) }}</span
        >
      </template>

      <template #freightAmount="{ row }">
        <span>¥{{ (row.freightAmount / 100).toFixed(2) }}</span>
      </template>

      <template #commissionAmount="{ row }">
        <span>¥{{ (row.commissionAmount / 100).toFixed(2) }}</span>
      </template>

      <template #settlementAmount="{ row }">
        <span>¥{{ (row.settlementAmount / 100).toFixed(2) }}</span>
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
      </template>
    </Grid>

    <Modal v-model:open="detailModalVisible" title="订单详情" width="800">
      <Descriptions :column="2" bordered>
        <Descriptions.Item label="订单编号">{{
          detailData?.orderNo || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="状态">
          <Tag :color="getStatusColor(detailData?.status)">{{
            getStatusText(detailData?.status)
          }}</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="店铺名称">{{
          detailData?.shopName || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="商品数量"
          >{{ detailData?.goodsCount || 0 }}件</Descriptions.Item
        >
        <Descriptions.Item label="订单金额"
          >¥{{
            ((detailData?.totalAmount || 0) / 100).toFixed(2)
          }}</Descriptions.Item
        >
        <Descriptions.Item label="运费"
          >¥{{
            ((detailData?.freightAmount || 0) / 100).toFixed(2)
          }}</Descriptions.Item
        >
        <Descriptions.Item label="平台佣金"
          >¥{{
            ((detailData?.commissionAmount || 0) / 100).toFixed(2)
          }}</Descriptions.Item
        >
        <Descriptions.Item label="结算金额"
          >¥{{
            ((detailData?.settlementAmount || 0) / 100).toFixed(2)
          }}</Descriptions.Item
        >
        <Descriptions.Item label="收货人" :span="2"
          >{{ detailData?.receiverName }}
          {{ detailData?.receiverPhone }}</Descriptions.Item
        >
        <Descriptions.Item label="收货地址" :span="2">{{
          detailData?.receiverAddress || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="支付时间">{{
          detailData?.payTime || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="发货时间">{{
          detailData?.deliveryTime || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="签收时间">{{
          detailData?.receiveTime || '-'
        }}</Descriptions.Item>
        <Descriptions.Item
          label="取消原因"
          :span="2"
          v-if="detailData?.cancelReason"
        >
          <span class="text-red-500">{{ detailData.cancelReason }}</span>
        </Descriptions.Item>
      </Descriptions>
    </Modal>
  </Page>
</template>
