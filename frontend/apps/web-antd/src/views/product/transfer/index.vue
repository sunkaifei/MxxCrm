<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideArrowRightLeft, LucideEye, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteTransferApi,
  getTransferListApi,
  transferInboundApi,
  transferOutboundApi,
} from '#/api/core/product/transfer';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import TransferDrawer from './drawer.vue';

const accessStore = useAccessStore();

// 调拨状态选项
const statusOptions = [
  { label: $t('page.product.inventory.transfer.status.0'), value: 0 },
  { label: $t('page.product.inventory.transfer.status.1'), value: 1 },
  { label: $t('page.product.inventory.transfer.status.2'), value: 2 },
  { label: $t('page.product.inventory.transfer.status.3'), value: 3 },
  { label: $t('page.product.inventory.transfer.status.4'), value: 4 },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'transferNo',
      label: $t('page.product.inventory.transfer.field.transferNo'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.product.inventory.transfer.field.status'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        options: statusOptions,
        allowClear: true,
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'createTimeRange',
      label: $t('page.product.inventory.transfer.field.createTime'),
      componentProps: {
        placeholder: [
          $t('ui.placeholder.startDate'),
          $t('ui.placeholder.endDate'),
        ],
        allowClear: true,
        valueFormat: 'YYYY-MM-DD',
        range: true,
      },
    },
  ],
};

// 调拨状态标签映射
function getTransferStatusTag(status: number) {
  const map: Record<number, { color: string; label: string }> = {
    0: {
      label: $t('page.product.inventory.transfer.status.0'),
      color: 'default',
    },
    1: {
      label: $t('page.product.inventory.transfer.status.1'),
      color: 'processing',
    },
    2: {
      label: $t('page.product.inventory.transfer.status.2'),
      color: 'warning',
    },
    3: {
      label: $t('page.product.inventory.transfer.status.3'),
      color: 'success',
    },
    4: {
      label: $t('page.product.inventory.transfer.status.4'),
      color: 'error',
    },
  };
  return map[status] || { label: $t('ui.unknown'), color: 'default' };
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
        return await getTransferListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          transferNo: formValues.transferNo,
          status: formValues.status,
          createTimeStart: formValues.createTimeRange?.[0],
          createTimeEnd: formValues.createTimeRange?.[1],
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
      title: $t('page.product.inventory.transfer.field.transferNo'),
      field: 'transferNo',
      width: 140,
    },
    {
      title: $t('page.product.inventory.transfer.field.sourceWarehouse'),
      field: 'sourceWarehouse',
      minWidth: 120,
    },
    {
      title: $t('page.product.inventory.transfer.field.targetWarehouse'),
      field: 'targetWarehouse',
      minWidth: 120,
    },
    {
      title: $t('page.product.inventory.transfer.field.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('page.product.inventory.transfer.field.totalQuantity'),
      field: 'totalQuantity',
      width: 100,
    },
    {
      title: $t('page.product.inventory.transfer.field.createdBy'),
      field: 'createdByName',
      width: 100,
    },
    {
      title: $t('page.product.inventory.transfer.field.createTime'),
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
  connectedComponent: TransferDrawer,
  onOpenChange(isOpen: boolean) {
    if (!isOpen) {
      const data = drawerApi.getData();
      if (data?.needRefresh) {
        gridApi.query();
      }
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

async function handleViewDetail(row: any) {
  window.$message.info(
    `${$t('page.product.inventory.transfer.action.viewDetail')} #${row.transferNo}`,
  );
}

async function handleOutbound(row: any) {
  row.pending = true;
  try {
    await transferOutboundApi(row.id);
    window.$message.success(
      $t('page.product.inventory.transfer.action.outboundSuccess'),
    );
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleInbound(row: any) {
  row.pending = true;
  try {
    await transferInboundApi(row.id);
    window.$message.success(
      $t('page.product.inventory.transfer.action.inboundSuccess'),
    );
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteTransferApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="transfer" />
    <Grid :table-title="$t('page.product.inventory.transfer.title')">
      <template #toolbar-actions>
        <Button
          v-if="accessStore.hasAccessCode('product:transfer:create')"
          type="primary"
          @click="openDrawer(true)"
        >
          {{ $t('page.product.inventory.transfer.create') }}
        </Button>
      </template>

      <template #status="{ row }">
        <Tag :color="getTransferStatusTag(row.status).color">
          {{ getTransferStatusTag(row.status).label }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:transfer:view')"
          type="link"
          :icon="h(LucideEye)"
          @click="() => handleViewDetail(row)"
        >
          {{ $t('page.product.inventory.transfer.action.viewDetail') }}
        </Button>
        <Button
          v-if="
            accessStore.hasAccessCode('product:transfer:outbound') &&
            row.status === 0
          "
          type="link"
          :icon="h(LucideArrowRightLeft)"
          @click="() => handleOutbound(row)"
        >
          {{ $t('page.product.inventory.transfer.action.outbound') }}
        </Button>
        <Button
          v-if="
            accessStore.hasAccessCode('product:transfer:inbound') &&
            row.status === 2
          "
          type="link"
          :icon="h(LucideArrowRightLeft)"
          @click="() => handleInbound(row)"
        >
          {{ $t('page.product.inventory.transfer.action.inbound') }}
        </Button>
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.product.inventory.transfer.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('product:transfer:delete')"
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
