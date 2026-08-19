<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import {
  LucideArrowRight,
  LucideCheck,
  LucideCheckCircle,
  LucideFilePenLine,
  LucidePackage,
  LucideTrash2,
  LucideXCircle,
} from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  closeProductionOrderApi,
  completeProductionOrderApi,
  deleteProductionOrderApi,
  getProductionOrderListApi,
  inboundProductionOrderApi,
  releaseProductionOrderApi,
  startProductionOrderApi,
} from '#/api';
import { $t } from '#/locales';

import ProductionOrderDrawer from './drawer.vue';

const accessStore = useAccessStore();

const statusOptions = [
  { label: '草稿', value: 0 },
  { label: '已下达', value: 1 },
  { label: '生产中', value: 2 },
  { label: '已完工', value: 3 },
  { label: '已入库', value: 4 },
  { label: '已关闭', value: 5 },
  { label: '已取消', value: 6 },
];

const statusLabelMap: Record<number, string> = {
  0: '草稿',
  1: '已下达',
  2: '生产中',
  3: '已完工',
  4: '已入库',
  5: '已关闭',
  6: '已取消',
};

const statusColorMap: Record<number, string> = {
  0: 'default',
  1: 'blue',
  2: 'orange',
  3: 'green',
  4: 'green',
  5: 'default',
  6: 'default',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '工单号',
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
        return await getProductionOrderListApi({
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
    { title: '工单号', field: 'orderNo', width: 140 },
    { title: '产品', field: 'productName', width: 160 },
    { title: '计划数量', field: 'plannedQuantity', width: 100 },
    { title: '已完工数', field: 'completedQuantity', width: 100 },
    { title: '计划开工日', field: 'plannedStartDate', width: 110 },
    { title: '计划完工日', field: 'plannedEndDate', width: 110 },
    { title: '实际完工日', field: 'actualEndDate', width: 110 },
    {
      title: $t('ui.table.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 160,
      slots: { default: 'createTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 280,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: ProductionOrderDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleCreate() {
  openDrawer(true);
}
function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteProductionOrderApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleRelease(row: any) {
  row.pending = true;
  try {
    await releaseProductionOrderApi(row.id);
    window.$message.success('已下达');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}

async function handleStart(row: any) {
  row.pending = true;
  try {
    await startProductionOrderApi(row.id);
    window.$message.success('已开工');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}

async function handleComplete(row: any) {
  row.pending = true;
  try {
    await completeProductionOrderApi(row.id);
    window.$message.success('已完工');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}

async function handleWarehouse(row: any) {
  row.pending = true;
  try {
    await inboundProductionOrderApi(row.id);
    window.$message.success('已完工入库');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}

async function handleClose(row: any) {
  row.pending = true;
  try {
    await closeProductionOrderApi(row.id);
    window.$message.success('已关闭');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.production.order.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('production:order:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.production.order.button.create') }}
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
          v-if="accessStore.hasAccessCode('production:order:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <template v-if="row.status === 0">
          <Popconfirm
            title="确认下达？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleRelease(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('production:order:release')"
              type="link"
              :icon="h(LucideCheckCircle)"
            />
          </Popconfirm>
        </template>
        <template v-if="row.status === 1">
          <Popconfirm
            title="确认开工？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleStart(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('production:order:start')"
              type="link"
              :icon="h(LucideArrowRight)"
            />
          </Popconfirm>
        </template>
        <template v-if="row.status === 2">
          <Popconfirm
            title="确认完工？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleComplete(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('production:order:complete')"
              type="link"
              :icon="h(LucideCheck)"
            />
          </Popconfirm>
        </template>
        <template v-if="row.status === 3">
          <Popconfirm
            title="确认完工入库？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleWarehouse(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('production:order:warehouse')"
              type="link"
              :icon="h(LucidePackage)"
            />
          </Popconfirm>
        </template>
        <template
          v-if="row.status === 0 || row.status === 1 || row.status === 2"
        >
          <Popconfirm
            title="确认关闭？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleClose(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('production:order:close')"
              type="link"
              :icon="h(LucideXCircle)"
            />
          </Popconfirm>
        </template>
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: '生产工单' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('production:order:delete')"
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
