<script lang="ts" setup>
import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Modal, Popconfirm, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  deleteShipmentApi,
  getShipmentListApi,
  signShipmentApi,
} from '#/api';
import { $t } from '#/locales';
import ShipmentDrawer from './drawer.vue';
import SalesProcessGuide from '../components/SalesProcessGuide.vue';

const accessStore = useAccessStore();

// 发货状态映射
const statusMap: Record<number, { label: string; color: string }> = {
  1: { label: '待发货', color: 'default' },
  2: { label: '已发货', color: 'processing' },
  3: { label: '已签收', color: 'green' },
  4: { label: '已取消', color: 'red' },
};

// 配送方式映射
const shippingMethodMap: Record<number, { label: string; color: string }> = {
  1: { label: '快递', color: 'blue' },
  2: { label: '物流', color: 'cyan' },
  3: { label: '自提', color: 'orange' },
  4: { label: '送货上门', color: 'green' },
  5: { label: '其他', color: 'default' },
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
      componentProps: {
        placeholder: '发货单号/物流单号/收货人',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '发货状态',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '待发货', value: 1 },
          { label: '已发货', value: 2 },
          { label: '已签收', value: 3 },
          { label: '已取消', value: 4 },
        ],
      },
    },
    {
      component: 'RangePicker',
      fieldName: 'dateRange',
      label: '发货日期',
      componentProps: {
        valueFormat: 'YYYY-MM-DD',
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
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
          status: formValues.status,
        };
        if (formValues.dateRange && formValues.dateRange.length === 2) {
          params.startDate = formValues.dateRange[0];
          params.endDate = formValues.dateRange[1];
        }
        return await getShipmentListApi(params);
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
      title: '发货单号',
      field: 'shipmentNo',
      width: 180,
    },
    {
      title: '订单ID',
      field: 'orderId',
      width: 120,
    },
    {
      title: '发货日期',
      field: 'shipmentDate',
      width: 120,
    },
    {
      title: '物流公司',
      field: 'logisticsCompany',
      width: 120,
    },
    {
      title: '物流单号',
      field: 'trackingNo',
      width: 160,
    },
    {
      title: '配送方式',
      field: 'shippingMethod',
      width: 100,
      slots: { default: 'shippingMethod' },
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
      title: '发货数量',
      field: 'totalQuantity',
      width: 100,
      align: 'right',
    },
    {
      title: '发货状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
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
      width: 200,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: ShipmentDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(row?: any) {
  drawerApi.setData({ row });
  drawerApi.open();
}

function handleCreate(row?: any) {
  openDrawer(row);
}

function handleSign(row: any) {
  Modal.confirm({
    title: '签收确认',
    content: `确定要确认签收发货单「${row.shipmentNo || ''}」吗？`,
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await signShipmentApi(row.id);
        message.success('签收成功');
        gridApi.query();
      } catch {
        message.error('签收失败');
      }
    },
  });
}

async function handleDelete(row: any) {
  try {
    await deleteShipmentApi(row.id);
    message.success($t('ui.notification.delete_success'));
    gridApi.query();
  } catch {
    // 错误由全局拦截器处理
  }
}
</script>

<template>
  <Page auto-content-height>
    <SalesProcessGuide current-step="shipment" />
    <Grid :table-title="$t('page.sale.shipment.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:order:edit')"
          type="primary"
          class="mr-2"
          @click="() => handleCreate()"
        >
          {{ $t('page.sale.shipment.button.create') }}
        </Button>
      </template>

      <template #shippingMethod="{ row }">
        <Tag
          v-if="row.shippingMethod && shippingMethodMap[row.shippingMethod]"
          :color="shippingMethodMap[row.shippingMethod]?.color"
        >
          {{ shippingMethodMap[row.shippingMethod]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status && statusMap[row.status]" :color="statusMap[row.status]?.color">
          {{ statusMap[row.status]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('sale:order:edit') && row.status === 2"
          type="link"
          size="small"
          @click="() => handleSign(row)"
        >
          {{ $t('page.sale.shipment.button.sign') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:order:edit')"
          type="link"
          size="small"
          :icon="h(LucideFilePenLine)"
          @click="() => handleCreate(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.sale.shipment.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('sale:order:edit')"
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
