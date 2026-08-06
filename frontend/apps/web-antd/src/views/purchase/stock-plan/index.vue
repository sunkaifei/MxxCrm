<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideCheckCircle, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { convertToRequisitionApi, deleteStockPlanApi, getStockPlanListApi } from '#/api';
import { $t } from '#/locales';

import StockPlanDrawer from './drawer.vue';

const accessStore = useAccessStore();

const statusOptions = [
  { label: '待处理', value: 0 },
  { label: '已生成采购申请', value: 1 },
  { label: '已关闭', value: 2 },
  { label: '已延期', value: 3 },
  { label: '已取消', value: 4 },
];

const statusLabelMap: Record<number, string> = {
  0: '待处理',
  1: '已生成采购申请',
  2: '已关闭',
  3: '已延期',
  4: '已取消',
};

const statusColorMap: Record<number, string> = {
  0: 'orange',
  1: 'green',
  2: 'default',
  3: 'blue',
  4: 'default',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '计划单号',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('ui.table.status'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: statusOptions,
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
        return await getStockPlanListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
          status: formValues.status,
        });
      },
    },
  },

  columns: [
    { type: 'seq', title: $t('ui.table.seq'), width: 60 },
    { title: '计划单号', field: 'planNo', width: 140 },
    { title: '产品', field: 'productName', width: 160 },
    { title: '需求日期', field: 'demandDate', width: 110 },
    { title: '需求量', field: 'demandQuantity', width: 90 },
    { title: '可用量', field: 'availableQuantity', width: 90 },
    { title: '净需求', field: 'netDemand', width: 90 },
    { title: '建议下单日', field: 'suggestedOrderDate', width: 110 },
    { title: '建议采购量', field: 'suggestedPurchaseQty', width: 110 },
    { title: '供应商', field: 'supplierName', width: 160 },
    { title: $t('ui.table.status'), field: 'status', width: 120, slots: { default: 'status' } },
    { title: $t('ui.table.createTime'), field: 'createTime', width: 160, slots: { default: 'createTime' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 180 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: StockPlanDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteStockPlanApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleConvertToRequisition(row: any) {
  row.pending = true;
  try {
    await convertToRequisitionApi(row.id);
    window.$message.success('已成功转采购申请');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.purchase.stockPlan.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('purchase:stockPlan:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.purchase.stockPlan.button.create') }}
        </Button>
      </template>

      <template #status="{ row }">
        <Tag :color="statusColorMap[row.status]">
          {{ statusLabelMap[row.status] || row.status }}
        </Tag>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('purchase:stockPlan:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <template v-if="row.status === 0">
          <Popconfirm
            title="确认转采购申请？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleConvertToRequisition(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('purchase:stockPlan:convert')"
              type="link"
              :icon="h(LucideCheckCircle)"
            />
          </Popconfirm>
        </template>
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: '备货计划' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('purchase:stockPlan:delete')"
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