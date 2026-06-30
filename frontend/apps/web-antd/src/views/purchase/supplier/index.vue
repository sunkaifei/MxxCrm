<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deleteSupplierApi, getSupplierListApi } from '#/api';
import { $t } from '#/locales';

import SupplierDrawer from './drawer.vue';

const accessStore = useAccessStore();

const statusOptions = [
  { label: '正常', value: 1, color: 'green' },
  { label: '停用', value: 2, color: 'red' },
  { label: '待审核', value: 3, color: 'orange' },
  { label: '黑名单', value: 4, color: 'default' },
];

const statusLabelMap: Record<number, string> = {
  1: '正常',
  2: '停用',
  3: '待审核',
  4: '黑名单',
};

const statusColorMap: Record<number, string> = {
  1: 'green',
  2: 'red',
  3: 'orange',
  4: 'default',
};

const levelOptions = [
  { label: '战略供应商', value: 1 },
  { label: '核心供应商', value: 2 },
  { label: '普通供应商', value: 3 },
  { label: '备选供应商', value: 4 },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '供应商名称/联系人',
      componentProps: { placeholder: '请输入供应商名称或联系人', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: { placeholder: '全部', allowClear: true, options: statusOptions },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true },
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getSupplierListApi({
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
    { title: '供应商编号', field: 'supplierNo', width: 140 },
    { title: '公司名称', field: 'companyName', width: 200 },
    { title: '简称', field: 'shortName', width: 120 },
    { title: '联系人', field: 'contactName', width: 100 },
    { title: '联系电话', field: 'contactPhone', width: 130 },
    { title: '邮箱', field: 'contactEmail', width: 180 },
    { title: '国家/地区', field: 'country', width: 100 },
    { title: '等级', field: 'level', width: 120, slots: { default: 'level' } },
    { title: '状态', field: 'status', width: 90, slots: { default: 'status' } },
    { title: '创建时间', field: 'createTime', width: 160, slots: { default: 'createTime' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 120 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: SupplierDrawer,
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
    await deleteSupplierApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleBatchDelete() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    window.$message.warning('请选择要删除的供应商');
    return;
  }
  const ids = records.map((r: any) => r.id);
  await deleteSupplierApi(ids);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.purchase.supplier.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('purchase:supplier:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新建供应商
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('purchase:supplier:delete')"
          class="mr-2"
          @click="handleBatchDelete"
        >
          批量删除
        </Button>
      </template>

      <template #level="{ row }">
        <Tag v-if="row.level === 1" color="red">战略供应商</Tag>
        <Tag v-else-if="row.level === 2" color="orange">核心供应商</Tag>
        <Tag v-else-if="row.level === 3" color="blue">普通供应商</Tag>
        <Tag v-else-if="row.level === 4" color="default">备选供应商</Tag>
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
          v-if="accessStore.hasAccessCode('purchase:supplier:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: '供应商' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('purchase:supplier:delete')"
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
