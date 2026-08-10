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
import { convertToProductionOrderApi, deleteProductionPlanApi, getProductionPlanListApi } from '#/api';
import { $t } from '#/locales';

import ProductionPlanDrawer from './drawer.vue';

const accessStore = useAccessStore();

const statusOptions = [
  { label: '待处理', value: 0 },
  { label: '已生成工单', value: 1 },
  { label: '已关闭', value: 2 },
  { label: '已取消', value: 3 },
];

const statusLabelMap: Record<number, string> = {
  0: '待处理',
  1: '已生成工单',
  2: '已关闭',
  3: '已取消',
};

const statusColorMap: Record<number, string> = {
  0: 'orange',
  1: 'green',
  2: 'default',
  3: 'default',
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
        return await getProductionPlanListApi({
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
    { title: '计划完工日', field: 'plannedEndDate', width: 110 },
    { title: '需求量', field: 'demandQuantity', width: 90 },
    { title: '可用量', field: 'availableQuantity', width: 90 },
    { title: '净需求', field: 'netDemand', width: 90 },
    { title: '建议开工日', field: 'suggestedStartDate', width: 110 },
    { title: '建议生产量', field: 'suggestedProductionQty', width: 110 },
    { title: $t('ui.table.status'), field: 'status', width: 110, slots: { default: 'status' } },
    { title: $t('ui.table.createTime'), field: 'createTime', width: 160, slots: { default: 'createTime' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 180 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: ProductionPlanDrawer,
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
    await deleteProductionPlanApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleConvertToOrder(row: any) {
  row.pending = true;
  try {
    await convertToProductionOrderApi(row.id);
    window.$message.success('已成功转生产工单');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.production.plan.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('production:plan:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.production.plan.button.create') }}
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
          v-if="accessStore.hasAccessCode('production:plan:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <template v-if="row.status === 0">
          <Popconfirm
            title="确认转生产工单？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleConvertToOrder(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('production:plan:convert')"
              type="link"
              :icon="h(LucideCheckCircle)"
            />
          </Popconfirm>
        </template>
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: '生产计划' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('production:plan:delete')"
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