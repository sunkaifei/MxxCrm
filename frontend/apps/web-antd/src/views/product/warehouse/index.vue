<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deleteWarehouseApi, getWarehouseListApi } from '#/api';
import { $t } from '#/locales';

import WarehouseDrawer from './drawer.vue';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'warehouseName',
      label: '仓库名称',
      componentProps: {
        placeholder: '请输入仓库名称',
        allowClear: true,
      },
    },
  ],
};

function getWarehouseTypeTag(type: number) {
  const map: Record<number, { label: string; color: string }> = {
    1: { label: '原材料仓', color: 'blue' },
    2: { label: '成品仓', color: 'green' },
    3: { label: '半成品仓', color: 'orange' },
    4: { label: '退货仓', color: 'red' },
    5: { label: '中转仓', color: 'purple' },
  };
  return map[type] || { label: '未知', color: 'default' };
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
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getWarehouseListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          warehouseName: formValues.warehouseName,
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
      title: '仓库编码',
      field: 'code',
      width: 110,
    },
    {
      title: '仓库名称',
      field: 'warehouseName',
      minWidth: 140,
    },
    {
      title: '仓库类型',
      field: 'warehouseType',
      width: 110,
      slots: { default: 'warehouseType' },
    },
    {
      title: '所在区域',
      field: 'region',
      width: 110,
    },
    {
      title: '面积(㎡)',
      field: 'areaSqm',
      width: 90,
    },
    {
      title: '负责人',
      field: 'contactPerson',
      width: 100,
    },
    {
      title: '联系电话',
      field: 'contactPhone',
      width: 130,
    },
    {
      title: '地址',
      field: 'address',
      minWidth: 160,
      showOverflow: 'tooltip',
    },
    {
      title: $t('ui.table.status'),
      field: 'isActive',
      width: 80,
      slots: { default: 'status' },
    },
    {
      title: '创建时间',
      field: 'createTime',
      width: 160,
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
  connectedComponent: WarehouseDrawer,
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
    await deleteWarehouseApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
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
    <Grid :table-title="$t('page.product.warehouse.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:warehouse:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.warehouse.button.create') }}
        </Button>
      </template>

      <template #warehouseType="{ row }">
        <Tag :color="getWarehouseTypeTag(row.warehouseType).color">
          {{ getWarehouseTypeTag(row.warehouseType).label }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="row.isActive !== false ? 'green' : 'red'">
          {{ row.isActive !== false ? '启用' : '停用' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:warehouse:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.product.warehouse.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('product:warehouse:delete')"
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
