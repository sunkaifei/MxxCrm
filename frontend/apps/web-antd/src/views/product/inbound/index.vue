<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { requestClient } from '#/api/request';
import { auditInboundApi, deleteInboundApi, getInboundListApi, rejectInboundApi } from '#/api/core/product/inbound';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import InboundDrawer from './drawer.vue';

const accessStore = useAccessStore();

// 入库类型选项
const inboundTypeOptions = [
  { label: $t('page.product.inbound.type.purchase'), value: 'purchase' },
  { label: $t('page.product.inbound.type.return'), value: 'return' },
  { label: $t('page.product.inbound.type.surplus'), value: 'surplus' },
  { label: $t('page.product.inbound.type.initial'), value: 'initial' },
  { label: $t('page.product.inbound.type.other'), value: 'other' },
];

// 入库状态选项
const statusOptions = [
  { label: $t('page.product.inbound.status.0'), value: 0 },
  { label: $t('page.product.inbound.status.1'), value: 1 },
  { label: $t('page.product.inbound.status.2'), value: 2 },
  { label: $t('page.product.inbound.status.3'), value: 3 },
  { label: $t('page.product.inbound.status.4'), value: 4 },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'inboundNo',
      label: $t('page.product.inbound.field.inboundNo'),
      componentProps: {
        placeholder: $t('page.product.inbound.placeholder.inboundNo'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'inboundType',
      label: $t('page.product.inbound.field.inboundType'),
      componentProps: {
        placeholder: $t('page.product.inbound.placeholder.inboundType'),
        options: inboundTypeOptions,
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'warehouseId',
      label: $t('page.product.inbound.field.warehouse'),
      componentProps: {
        placeholder: $t('page.product.inbound.placeholder.warehouse'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.product.inbound.field.status'),
      componentProps: {
        placeholder: $t('page.product.inbound.placeholder.status'),
        options: statusOptions,
        allowClear: true,
      },
    },
  ],
};

// 入库类型标签映射
function getInboundTypeTag(type: string) {
  const map: Record<string, { label: string; color: string }> = {
    purchase: { label: $t('page.product.inbound.type.purchase'), color: 'blue' },
    return: { label: $t('page.product.inbound.type.return'), color: 'orange' },
    surplus: { label: $t('page.product.inbound.type.surplus'), color: 'green' },
    initial: { label: $t('page.product.inbound.type.initial'), color: 'cyan' },
    other: { label: $t('page.product.inbound.type.other'), color: 'default' },
  };
  return map[type] || { label: $t('page.product.inbound.type.unknown'), color: 'default' };
}

// 入库状态标签映射
function getInboundStatusTag(status: number) {
  const map: Record<number, { label: string; color: string }> = {
    0: { label: $t('page.product.inbound.status.0'), color: 'default' },
    1: { label: $t('page.product.inbound.status.1'), color: 'processing' },
    2: { label: $t('page.product.inbound.status.2'), color: 'warning' },
    3: { label: $t('page.product.inbound.status.3'), color: 'success' },
    4: { label: $t('page.product.inbound.status.4'), color: 'error' },
  };
  return map[status] || { label: $t('page.product.inbound.status.unknown'), color: 'default' };
}

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
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getInboundListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          inboundNo: formValues.inboundNo,
          inboundType: formValues.inboundType,
          warehouseId: formValues.warehouseId,
          status: formValues.status,
        });
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 60,
    },
    {
      title: $t('page.product.inbound.field.inboundNo'),
      field: 'inboundNo',
      width: 140,
    },
    {
      title: $t('page.product.inbound.field.inboundType'),
      field: 'inboundType',
      width: 110,
      slots: { default: 'inboundType' },
    },
    {
      title: $t('page.product.inbound.field.warehouse'),
      field: 'warehouseName',
      minWidth: 120,
    },
    {
      title: $t('page.product.inbound.field.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('page.product.inbound.field.totalQuantity'),
      field: 'totalQuantity',
      width: 100,
    },
    {
      title: $t('page.product.inbound.field.totalAmount'),
      field: 'totalAmount',
      width: 110,
    },
    {
      title: $t('page.product.inbound.field.createdBy'),
      field: 'createdByName',
      width: 100,
    },
    {
      title: $t('page.product.inbound.field.createTime'),
      field: 'createTime',
      width: 160,
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
  connectedComponent: InboundDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({
    create,
    row,
  });
  drawerApi.open();
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteInboundApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleSubmitAudit(row: any) {
  row.pending = true;
  try {
    await requestClient.put(`/api/system/inbound/submit/${row.id}`);
    window.$message.success($t('page.product.inbound.message.submitSuccess'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleAudit(row: any) {
  row.pending = true;
  try {
    await auditInboundApi(row.id);
    window.$message.success($t('page.product.inbound.message.auditSuccess'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleReject(row: any) {
  row.pending = true;
  try {
    await rejectInboundApi(row.id);
    window.$message.success($t('page.product.inbound.message.rejectSuccess'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function handleCreate() {
  openDrawer(true);
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="inbound" />
    <Grid :table-title="$t('page.product.inbound.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:inbound:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.inbound.button.create') }}
        </Button>
      </template>

      <template #inboundType="{ row }">
        <Tag :color="getInboundTypeTag(row.inboundType).color">
          {{ getInboundTypeTag(row.inboundType).label }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="getInboundStatusTag(row.status).color">
          {{ getInboundStatusTag(row.status).label }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="row.status === 0 && accessStore.hasAccessCode('product:inbound:audit')"
          type="link"
          @click="() => handleSubmitAudit(row)"
        >
          {{ $t('page.product.inbound.action.submitAudit') }}
        </Button>
        <Button
          v-if="row.status === 1 && accessStore.hasAccessCode('product:inbound:audit')"
          type="link"
          @click="() => handleAudit(row)"
        >
          {{ $t('page.product.inbound.action.audit') }}
        </Button>
        <Button
          v-if="row.status === 1 && accessStore.hasAccessCode('product:inbound:audit')"
          type="link"
          danger
          @click="() => handleReject(row)"
        >
          {{ $t('page.product.inbound.action.reject') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('product:inbound:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.product.inbound.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('product:inbound:delete')"
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
