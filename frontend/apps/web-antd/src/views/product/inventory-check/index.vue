<script lang="ts" setup>
import { h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import {
  LucideCheckCircle,
  LucideFilePenLine,
  LucideTrash2,
} from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  auditCheckApi,
  deleteCheckApi,
  getCheckListApi,
  updateCheckApi,
} from '#/api/core/product/check';
import { getWarehouseListApi } from '#/api/core/product/warehouse';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import CheckDrawer from './drawer.vue';

const accessStore = useAccessStore();

const warehouseOptions = ref<{ label: string; value: number }[]>([]);

async function loadWarehouseOptions() {
  try {
    const resp: any = await getWarehouseListApi({ page: 1, pageSize: 999 });
    const list = resp?.data ?? resp ?? [];
    warehouseOptions.value = (Array.isArray(list) ? list : []).map(
      (w: any) => ({
        label: w.warehouseName ?? w.name ?? w.label,
        value: Number(w.id ?? w.value),
      }),
    );
  } catch (e) {
    console.error('[InventoryCheck] 加载仓库选项失败:', e);
  }
}

onMounted(() => {
  loadWarehouseOptions();
});

function getCheckTypeOptions() {
  return [
    { label: $t('page.product.inventory.check.type.1'), value: 1 },
    { label: $t('page.product.inventory.check.type.2'), value: 2 },
    { label: $t('page.product.inventory.check.type.3'), value: 3 },
  ];
}

function getStatusOptions() {
  return [
    { label: $t('page.product.inventory.check.status.0'), value: 0 },
    { label: $t('page.product.inventory.check.status.1'), value: 1 },
    { label: $t('page.product.inventory.check.status.2'), value: 2 },
    { label: $t('page.product.inventory.check.status.3'), value: 3 },
  ];
}

function getCheckTypeTag(type: number) {
  const map: Record<number, { label: string; color: string }> = {
    1: { label: $t('page.product.inventory.check.type.1'), color: 'blue' },
    2: { label: $t('page.product.inventory.check.type.2'), color: 'cyan' },
    3: { label: $t('page.product.inventory.check.type.3'), color: 'purple' },
  };
  return map[type] || { label: $t('ui.unknown'), color: 'default' };
}

function getStatusTag(status: number) {
  const map: Record<number, { label: string; color: string }> = {
    0: { label: $t('page.product.inventory.check.status.0'), color: 'default' },
    1: { label: $t('page.product.inventory.check.status.1'), color: 'processing' },
    2: { label: $t('page.product.inventory.check.status.2'), color: 'success' },
    3: { label: $t('page.product.inventory.check.status.3'), color: 'default' },
  };
  return map[status] || { label: $t('ui.unknown'), color: 'default' };
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'checkNo',
      label: $t('page.product.inventory.check.field.checkNo'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'checkType',
      label: $t('page.product.inventory.check.field.checkType'),
      componentProps: {
        placeholder: $t('ui.placeholder.all'),
        allowClear: true,
        options: getCheckTypeOptions(),
      },
    },
    {
      component: 'Select',
      fieldName: 'warehouseId',
      label: $t('page.product.inventory.check.field.warehouse'),
      componentProps: {
        placeholder: $t('ui.placeholder.all'),
        allowClear: true,
        options: warehouseOptions,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          (option?.label ?? '').toLowerCase().includes(input.toLowerCase()),
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.product.inventory.check.field.status'),
      componentProps: {
        placeholder: $t('ui.placeholder.all'),
        allowClear: true,
        options: getStatusOptions(),
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
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getCheckListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          checkNo: formValues.checkNo,
          checkType: formValues.checkType,
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
      title: $t('page.product.inventory.check.field.checkNo'),
      field: 'checkNo',
      width: 140,
    },
    {
      title: $t('page.product.inventory.check.field.checkType'),
      field: 'checkType',
      width: 110,
      slots: { default: 'checkType' },
    },
    {
      title: $t('page.product.inventory.check.field.warehouse'),
      field: 'warehouseName',
      minWidth: 140,
    },
    {
      title: $t('page.product.inventory.check.field.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('page.product.inventory.check.field.totalItems'),
      field: 'totalItems',
      width: 90,
      align: 'right',
    },
    {
      title: $t('page.product.inventory.check.field.diffItems'),
      field: 'diffItems',
      width: 90,
      align: 'right',
    },
    {
      title: $t('page.product.inventory.check.field.checkBy'),
      field: 'checkByName',
      width: 100,
    },
    {
      title: $t('page.product.inventory.check.field.checkTime'),
      field: 'checkTime',
      width: 160,
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 140,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: CheckDrawer,
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

function handleCreate() {
  openDrawer(true);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteCheckApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleAudit(row: any) {
  row.pending = true;
  try {
    await auditCheckApi(row.id);
    window.$message.success($t('page.product.inventory.check.action.auditSuccess'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="check" />
    <Grid :table-title="$t('page.product.inventory.check.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:check:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.inventory.check.create') }}
        </Button>
      </template>

      <template #checkType="{ row }">
        <Tag :color="getCheckTypeTag(row.checkType).color">
          {{ getCheckTypeTag(row.checkType).label }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="getStatusTag(row.status).color">
          {{ getStatusTag(row.status).label }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:check:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          v-if="row.status === 1 && accessStore.hasAccessCode('product:check:audit')"
          :title="$t('page.product.inventory.check.action.auditConfirm')"
          :ok-text="$t('page.product.inventory.check.action.confirm')"
          :cancel-text="$t('page.product.inventory.check.action.cancel')"
          @confirm="() => handleAudit(row)"
        >
          <Button type="link" :icon="h(LucideCheckCircle)" />
        </Popconfirm>
        <Popconfirm
          v-else-if="accessStore.hasAccessCode('product:check:audit')"
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.product.inventory.check.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button type="link" danger :icon="h(LucideTrash2)" />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
