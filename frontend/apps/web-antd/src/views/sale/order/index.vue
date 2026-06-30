﻿﻿﻿<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideEye, LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Input, Modal, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteOrderApi,
  getOrderListApi,
  updateOrderStatusApi,
} from '#/api';
import { $t } from '#/locales';
import OrderDrawer from './drawer.vue';
import SalesProcessGuide from '../components/SalesProcessGuide.vue';

const accessStore = useAccessStore();

const orderStatusOptions = [
  { label: '草稿', value: 1 },
  { label: '已确认', value: 2 },
  { label: '待发货', value: 3 },
  { label: '部分发货', value: 4 },
  { label: '已发货', value: 5 },
  { label: '已完成', value: 6 },
  { label: '已取消', value: 7 },
  { label: '已关闭', value: 8 },
];

const paymentStatusOptions = [
  { label: '未支付', value: 1 },
  { label: '部分支付', value: 2 },
  { label: '已支付', value: 3 },
  { label: '已退款', value: 4 },
];

const orderStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'blue',
  3: 'cyan',
  4: 'orange',
  5: 'purple',
  6: 'green',
  7: 'red',
  8: 'default',
};

const orderStatusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '已确认',
  3: '待发货',
  4: '部分发货',
  5: '已发货',
  6: '已完成',
  7: '已取消',
  8: '已关闭',
};

const paymentStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'orange',
  3: 'green',
  4: 'red',
};

const paymentStatusLabelMap: Record<number, string> = {
  1: '未支付',
  2: '部分支付',
  3: '已支付',
  4: '已退款',
};

const orderTypeColorMap: Record<number, string> = {
  1: 'blue',
  2: 'red',
};

const orderTypeLabelMap: Record<number, string> = {
  1: '销售订单',
  2: '退货订单',
};

const currencyLabelMap: Record<number, string> = {
  1: 'CNY',
  2: 'USD',
  3: 'EUR',
  4: 'GBP',
  5: 'JPY',
  6: 'HKD',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: { placeholder: '订单号/客户/标题', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'orderStatus',
      label: '订单状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: orderStatusOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'paymentStatus',
      label: '支付状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: paymentStatusOptions,
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'dateRange',
      label: '日期范围',
      componentProps: {
        type: 'daterange',
        placeholder: ['开始日期', '结束日期'],
        style: 'width:100%',
        valueFormat: 'YYYY-MM-DD',
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true },
  stripe: true,
  checkboxConfig: { checkMethod: () => true },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
        };
        if (formValues.keywords) params.keywords = formValues.keywords;
        if (formValues.orderStatus) params.orderStatus = formValues.orderStatus;
        if (formValues.paymentStatus)
          params.paymentStatus = formValues.paymentStatus;
        if (formValues.dateRange && formValues.dateRange.length === 2) {
          params.startDate = formValues.dateRange[0];
          params.endDate = formValues.dateRange[1];
        }
        return await getOrderListApi(params);
      },
    },
  },
  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: '订单号',
      field: 'orderNo',
      width: 160,
      slots: { default: 'orderNo' },
    },
    { title: '订单标题', field: 'title', width: 200 },
    { title: '客户名称', field: 'customerName', width: 180 },
    {
      title: '订单类型',
      field: 'orderType',
      width: 100,
      slots: { default: 'orderType' },
    },
    {
      title: '订单金额',
      field: 'totalAmount',
      width: 140,
      slots: { default: 'totalAmount' },
    },
    {
      title: '订单状态',
      field: 'orderStatus',
      width: 100,
      slots: { default: 'orderStatus' },
    },
    {
      title: '支付状态',
      field: 'paymentStatus',
      width: 100,
      slots: { default: 'paymentStatus' },
    },
    { title: '负责人', field: 'ownerUserName', width: 90 },
    { title: '下单日期', field: 'orderDate', width: 110 },
    {
      title: '创建时间',
      field: 'createTime',
      width: 160,
      slots: { default: 'createTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 220,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: OrderDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleCreate() {
  openDrawer(true);
}
function handleEdit(row: any) {
  openDrawer(false, row);
}
function handleView(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteOrderApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleBatchDelete() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    window.$message.warning('请选择要删除的订单');
    return;
  }
  const ids = records.map((r: any) => r.id);
  await deleteOrderApi(ids);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}

async function handleShip(row: any) {
  let trackingNo = '';
  Modal.confirm({
    title: '发货',
    content: h('div', { style: 'margin-top:16px' }, [
      h('div', { style: 'margin-bottom:8px' }, '请输入物流单号：'),
      h(Input, {
        placeholder: '物流单号',
        value: trackingNo,
        'onUpdate:value': (v: string) => {
          trackingNo = v;
        },
      }),
    ]),
    okText: '确认发货',
    cancelText: '取消',
    onOk: async () => {
      try {
        await updateOrderStatusApi({
          id: row.id,
          orderStatus: 5,
          trackingNo,
        });
        window.$message.success('发货成功');
        gridApi.query();
      } catch {
        window.$message.error('发货失败');
      }
    },
  });
}

async function handleComplete(row: any) {
  Modal.confirm({
    title: '确认完成订单',
    content: '确定要将此订单标记为已完成吗？',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await updateOrderStatusApi({
          id: row.id,
          orderStatus: 6,
        });
        window.$message.success('订单已完成');
        gridApi.query();
      } catch {
        window.$message.error('操作失败');
      }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <SalesProcessGuide current-step="order" />
    <Grid :table-title="''">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:order:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新建订单
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:order:delete')"
          class="mr-2"
          @click="handleBatchDelete"
        >
          批量删除
        </Button>
      </template>

      <template #orderNo="{ row }">
        <a
          v-if="accessStore.hasAccessCode('sale:order:view')"
          class="text-blue-600 cursor-pointer"
          @click="handleView(row)"
        >
          {{ row.orderNo }}
        </a>
        <span v-else>{{ row.orderNo }}</span>
      </template>

      <template #orderType="{ row }">
        <Tag :color="orderTypeColorMap[row.orderType]">
          {{ orderTypeLabelMap[row.orderType] || row.orderType }}
        </Tag>
      </template>

      <template #totalAmount="{ row }">
        {{ currencyLabelMap[row.currency] || 'CNY' }}
        {{ row.totalAmount?.toLocaleString?.() ?? row.totalAmount ?? 0 }}
      </template>

      <template #orderStatus="{ row }">
        <Tag :color="orderStatusColorMap[row.orderStatus]">
          {{ orderStatusLabelMap[row.orderStatus] || row.orderStatus }}
        </Tag>
      </template>

      <template #paymentStatus="{ row }">
        <Tag :color="paymentStatusColorMap[row.paymentStatus]">
          {{ paymentStatusLabelMap[row.paymentStatus] || row.paymentStatus }}
        </Tag>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('sale:order:view')"
          type="link"
          :icon="h(LucideEye)"
          @click="() => handleView(row)"
        />
        <Button
          v-if="accessStore.hasAccessCode('sale:order:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Button
          v-if="
            accessStore.hasAccessCode('sale:order:edit') &&
            (row.orderStatus === 2 || row.orderStatus === 3)
          "
          type="link"
          size="small"
          @click="() => handleShip(row)"
        >
          发货
        </Button>
        <Button
          v-if="
            accessStore.hasAccessCode('sale:order:edit') &&
            row.orderStatus === 5
          "
          type="link"
          size="small"
          @click="() => handleComplete(row)"
        >
          完成
        </Button>
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: '订单' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('sale:order:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />
  </Page>
</template>
