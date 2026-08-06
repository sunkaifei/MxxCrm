<script lang="ts" setup>
import { h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { requestClient } from '#/api/request';
import { auditOutboundApi, deleteOutboundApi, getOutboundListApi, rejectOutboundApi } from '#/api/core/product/outbound';
import { getWarehouseListApi } from '#/api/core/product/warehouse';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import OutboundDrawer from './drawer.vue';

const accessStore = useAccessStore();

const warehouseOptions = ref<{ label: string; value: number }[]>([]);

async function loadWarehouseOptions() {
  try {
    const resp = await getWarehouseListApi({ page: 1, pageSize: 999 });
    const list = resp?.data ?? resp ?? [];
    warehouseOptions.value = (Array.isArray(list) ? list : []).map((w: any) => ({
      label: w.warehouseName ?? w.name ?? w.label,
      value: Number(w.id ?? w.value),
    }));
  } catch (e) {
    console.error('[出库] 加载仓库选项失败:', e);
  }
}

onMounted(() => {
  loadWarehouseOptions();
});

const outboundTypeOptions = [
  { label: $t('page.product.outbound.type.sale'), value: 'sale' },
  { label: $t('page.product.outbound.type.material'), value: 'material' },
  { label: $t('page.product.outbound.type.shortage'), value: 'shortage' },
  { label: $t('page.product.outbound.type.scrap'), value: 'scrap' },
  { label: $t('page.product.outbound.type.freeze'), value: 'freeze' },
  { label: $t('page.product.outbound.type.other'), value: 'other' },
];

const statusOptions = [
  { label: $t('page.product.outbound.status.0'), value: 0 },
  { label: $t('page.product.outbound.status.1'), value: 1 },
  { label: $t('page.product.outbound.status.2'), value: 2 },
  { label: $t('page.product.outbound.status.3'), value: 3 },
  { label: $t('page.product.outbound.status.4'), value: 4 },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'outboundNo',
      label: $t('page.product.outbound.field.outboundNo'),
      componentProps: {
        placeholder: $t('page.product.outbound.placeholder.outboundNo'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'outboundType',
      label: $t('page.product.outbound.field.outboundType'),
      componentProps: {
        placeholder: $t('page.product.outbound.placeholder.outboundType'),
        options: outboundTypeOptions,
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'warehouseId',
      label: $t('page.product.outbound.field.warehouse'),
      componentProps: {
        placeholder: $t('page.product.outbound.placeholder.warehouse'),
        options: warehouseOptions,
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          (option?.label ?? '').toLowerCase().includes(input.toLowerCase()),
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.product.outbound.field.status'),
      componentProps: {
        placeholder: $t('page.product.outbound.placeholder.status'),
        options: statusOptions,
        allowClear: true,
      },
    },
  ],
};

function getOutboundTypeTag(type: string) {
  const map: Record<string, { label: string; color: string }> = {
    sale: { label: $t('page.product.outbound.type.sale'), color: 'blue' },
    material: { label: $t('page.product.outbound.type.material'), color: 'cyan' },
    shortage: { label: $t('page.product.outbound.type.shortage'), color: 'orange' },
    scrap: { label: $t('page.product.outbound.type.scrap'), color: 'red' },
    freeze: { label: $t('page.product.outbound.type.freeze'), color: 'purple' },
    other: { label: $t('page.product.outbound.type.other'), color: 'default' },
  };
  return map[type] || { label: $t('page.product.outbound.type.unknown'), color: 'default' };
}

function getStatusTag(status: number) {
  const map: Record<number, { label: string; color: string }> = {
    0: { label: $t('page.product.outbound.status.0'), color: 'default' },
    1: { label: $t('page.product.outbound.status.1'), color: 'processing' },
    2: { label: $t('page.product.outbound.status.2'), color: 'warning' },
    3: { label: $t('page.product.outbound.status.3'), color: 'success' },
    4: { label: $t('page.product.outbound.status.4'), color: 'error' },
  };
  return map[status] || { label: $t('page.product.outbound.status.unknown'), color: 'default' };
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
        return await getOutboundListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          outboundNo: formValues.outboundNo,
          outboundType: formValues.outboundType,
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
      title: $t('page.product.outbound.field.outboundNo'),
      field: 'outboundNo',
      minWidth: 140,
    },
    {
      title: $t('page.product.outbound.field.outboundType'),
      field: 'outboundType',
      width: 110,
      slots: { default: 'outboundType' },
    },
    {
      title: $t('page.product.outbound.field.warehouse'),
      field: 'warehouseName',
      minWidth: 120,
    },
    {
      title: $t('page.product.outbound.field.status'),
      field: 'status',
      width: 90,
      slots: { default: 'status' },
    },
    {
      title: $t('page.product.outbound.field.totalQuantity'),
      field: 'totalQuantity',
      width: 100,
    },
    {
      title: $t('page.product.outbound.field.totalAmount'),
      field: 'totalAmount',
      width: 110,
    },
    {
      title: $t('page.product.outbound.field.createdBy'),
      field: 'createdByName',
      width: 100,
    },
    {
      title: $t('page.product.outbound.field.createTime'),
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
  connectedComponent: OutboundDrawer,
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
    await deleteOutboundApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleSubmitAudit(row: any) {
  row.pending = true;
  try {
    await requestClient.put(`/api/system/outbound/submit/${row.id}`);
    window.$message.success($t('page.product.outbound.message.submitSuccess'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleAudit(row: any) {
  row.pending = true;
  try {
    await auditOutboundApi(row.id);
    window.$message.success($t('page.product.outbound.message.auditSuccess'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleReject(row: any) {
  row.pending = true;
  try {
    await rejectOutboundApi(row.id);
    window.$message.success($t('page.product.outbound.message.rejectSuccess'));
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
    <InventoryProcessGuide current-step="outbound" />
    <Grid :table-title="$t('page.product.outbound.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:outbound:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.outbound.button.create') }}
        </Button>
      </template>

      <template #outboundType="{ row }">
        <Tag :color="getOutboundTypeTag(row.outboundType).color">
          {{ getOutboundTypeTag(row.outboundType).label }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="getStatusTag(row.status).color">
          {{ getStatusTag(row.status).label }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="row.status === 0 && accessStore.hasAccessCode('product:outbound:audit')"
          type="link"
          @click="() => handleSubmitAudit(row)"
        >
          {{ $t('page.product.outbound.action.submitAudit') }}
        </Button>
        <Button
          v-if="row.status === 1 && accessStore.hasAccessCode('product:outbound:audit')"
          type="link"
          @click="() => handleAudit(row)"
        >
          {{ $t('page.product.outbound.action.audit') }}
        </Button>
        <Button
          v-if="row.status === 1 && accessStore.hasAccessCode('product:outbound:audit')"
          type="link"
          danger
          @click="() => handleReject(row)"
        >
          {{ $t('page.product.outbound.action.reject') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('product:outbound:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.product.outbound.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('product:outbound:delete')"
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
