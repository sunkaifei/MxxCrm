<script lang="ts" setup>
import { h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { auditPurchaseOrderApi, closePurchaseOrderApi, deletePurchaseOrderApi, getAllBrandsApi, getPurchaseOrderListApi, rejectPurchaseOrderApi } from '#/api';
import { $t } from '#/locales';

import PurchaseOrderDrawer from './drawer.vue';

const accessStore = useAccessStore();

const brandOptions = ref<{ label: string; value: number }[]>([]);

async function loadBrands() {
  try {
    const res = await getAllBrandsApi();
    brandOptions.value = (res || []).map((item: any) => ({
      label: item.name,
      value: item.id,
    }));
  } catch {
    // 忽略品牌加载失败
  }
}

onMounted(() => {
  loadBrands();
});

const statusOptions = [
  { label: $t('page.purchase.po.status.draft'), value: 'draft', color: 'default' },
  { label: $t('page.purchase.po.status.pending_audit'), value: 'pending_audit', color: 'blue' },
  { label: $t('page.purchase.po.status.audited'), value: 'audited', color: 'green' },
  { label: $t('page.purchase.po.status.ordered'), value: 'ordered', color: 'orange' },
  { label: $t('page.purchase.po.status.in_transit'), value: 'in_transit', color: 'purple' },
  { label: $t('page.purchase.po.status.partial_received'), value: 'partial_received', color: 'cyan' },
  { label: $t('page.purchase.po.status.received'), value: 'received', color: 'geekblue' },
  { label: $t('page.purchase.po.status.completed'), value: 'completed', color: 'green' },
  { label: $t('page.purchase.po.status.cancelled'), value: 'cancelled', color: 'red' },
  { label: $t('page.purchase.po.status.rejected'), value: 'rejected', color: 'red' },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: $t('page.purchase.po.form.purchaseNo'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'brandId',
      label: $t('page.purchase.po.form.brand'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: brandOptions,
        showSearch: true,
        filterOption: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('ui.table.status'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: statusOptions.map(s => ({ label: s.label, value: s.value })),
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
        return await getPurchaseOrderListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
          status: formValues.status,
          brandId: formValues.brandId,
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
      title: $t('page.purchase.po.column.purchaseNo'),
      field: 'purchaseNo',
    },
    {
      title: $t('page.purchase.po.column.supplierId'),
      field: 'supplierId',
    },
    {
      title: $t('page.purchase.po.column.amount'),
      field: 'amount',
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      slots: { default: 'status' },
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      slots: { default: 'createdAt' },
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
  connectedComponent: PurchaseOrderDrawer,
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

async function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deletePurchaseOrderApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleCreate() {
  openDrawer(true);
}

async function handleAudit(row: any) {
  row.pending = true;
  try {
    await auditPurchaseOrderApi(row.id);
    window.$message.success($t('page.purchase.po.message.auditSuccess'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleClose(row: any) {
  row.pending = true;
  try {
    await closePurchaseOrderApi(row.id);
    window.$message.success($t('page.purchase.po.message.closeSuccess'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleReject(row: any) {
  row.pending = true;
  try {
    await rejectPurchaseOrderApi(row.id);
    window.$message.success($t('page.purchase.po.message.rejectSuccess'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.purchase.po.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('purchase:po:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.purchase.po.button.create') }}
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #status="{ row }">
        <Tag :color="statusOptions.find(s => s.value === row.status)?.color || 'default'">
          {{ statusOptions.find(s => s.value === row.status)?.label || row.status }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('purchase:order:audit') && (row.status === 'draft' || row.status === 'pending_audit')"
          type="link"
          @click="() => handleAudit(row)"
        >
          {{ row.status === 'draft' ? $t('page.purchase.po.action.submitAudit') : $t('page.purchase.po.action.auditPass') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('purchase:order:audit') && row.status === 'pending_audit'"
          type="link"
          danger
          @click="() => handleReject(row)"
        >
          {{ $t('page.purchase.po.action.reject') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('purchase:order:close') && row.status === 'audited'"
          type="link"
          danger
          @click="() => handleClose(row)"
        >
          {{ $t('page.purchase.po.action.close') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('purchase:order:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.purchase.po.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('purchase:po:delete')"
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
