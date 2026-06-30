<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deletePaymentApi, getPaymentListApi } from '#/api';
import { $t } from '#/locales';
import PaymentDrawer from './drawer.vue';
import SalesProcessGuide from '../components/SalesProcessGuide.vue';

const accessStore = useAccessStore();

const paymentMethodMap: Record<number, { label: string; color: string }> = {
  1: { label: '银行转账', color: 'blue' },
  2: { label: '支付宝', color: 'cyan' },
  3: { label: '微信支付', color: 'green' },
  4: { label: '现金', color: 'orange' },
  5: { label: '支票', color: 'purple' },
  6: { label: '其他', color: 'default' },
};

const statusMap: Record<number, { label: string; color: string }> = {
  1: { label: '待确认', color: 'orange' },
  2: { label: '已确认', color: 'green' },
  3: { label: '已驳回', color: 'red' },
  4: { label: '已取消', color: 'default' },
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'paymentNo',
      label: '回款编号',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'orderNo',
      label: '订单编号',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '回款状态',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '待确认', value: 1 },
          { label: '已确认', value: 2 },
          { label: '已驳回', value: 3 },
          { label: '已取消', value: 4 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'paymentMethod',
      label: '支付方式',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '银行转账', value: 1 },
          { label: '支付宝', value: 2 },
          { label: '微信支付', value: 3 },
          { label: '现金', value: 4 },
          { label: '支票', value: 5 },
          { label: '其他', value: 6 },
        ],
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
        return await getPaymentListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          paymentNo: formValues.paymentNo,
          orderNo: formValues.orderNo,
          status: formValues.status,
          paymentMethod: formValues.paymentMethod,
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
      title: '回款编号',
      field: 'paymentNo',
      width: 180,
    },
    {
      title: '订单编号',
      field: 'orderNo',
      width: 160,
      slots: { default: 'orderNo' },
    },
    {
      title: '客户名称',
      field: 'customerName',
      width: 140,
    },
    {
      title: '回款金额',
      field: 'amount',
      width: 120,
      align: 'right',
      slots: { default: 'amount' },
    },
    {
      title: '支付方式',
      field: 'paymentMethod',
      width: 100,
      slots: { default: 'paymentMethod' },
    },
    {
      title: '回款状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: '到账日期',
      field: 'paymentDate',
      width: 120,
    },
    {
      title: '登记时间',
      field: 'createTime',
      width: 160,
      slots: { default: 'createTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 120,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: PaymentDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

async function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deletePaymentApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleCreate() {
  openDrawer(true);
}
</script>

<template>
  <Page auto-content-height>
    <SalesProcessGuide current-step="payment" />
    <Grid :table-title="$t('page.sale.payment.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:payment:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.sale.payment.button.create') }}
        </Button>
      </template>

      <template #orderNo="{ row }">
        <a v-if="row.orderNo" class="text-blue-500 hover:underline" @click="() => $router.push(`/sale/order?orderNo=${row.orderNo}`)">
          {{ row.orderNo }}
        </a>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #amount="{ row }">
        <span class="font-medium text-blue-600">
          ¥{{ Number(row.amount || 0).toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
        </span>
      </template>

      <template #paymentMethod="{ row }">
        <Tag v-if="row.paymentMethod && paymentMethodMap[row.paymentMethod]" :color="paymentMethodMap[row.paymentMethod].color">
          {{ paymentMethodMap[row.paymentMethod].label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status && statusMap[row.status]" :color="statusMap[row.status].color">
          {{ statusMap[row.status].label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('sale:payment:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.sale.payment.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('sale:payment:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
