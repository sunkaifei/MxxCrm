<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideEye, LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteInvoiceApi, getInvoiceListApi } from '#/api';
import { $t } from '#/locales';
import InvoiceDrawer from './drawer.vue';
import SalesProcessGuide from '../components/SalesProcessGuide.vue';

const accessStore = useAccessStore();

const typeOptions = [
  { label: '增值税专用发票', value: 1 },
  { label: '增值税普通发票', value: 2 },
  { label: '形式发票(PI)', value: 3 },
  { label: '商业发票(CI)', value: 4 },
];

const statusOptions = [
  { label: '草稿', value: 1 },
  { label: '已开票', value: 2 },
  { label: '已作废', value: 3 },
  { label: '已红冲', value: 4 },
];

const statusColorMap: Record<number, string> = {
  1: 'default',
  2: 'green',
  3: 'red',
  4: 'orange',
};

const statusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '已开票',
  3: '已作废',
  4: '已红冲',
};

const typeColorMap: Record<number, string> = {
  1: 'blue',
  2: 'cyan',
  3: 'orange',
  4: 'purple',
};

const typeLabelMap: Record<number, string> = {
  1: '增值税专用发票',
  2: '增值税普通发票',
  3: '形式发票(PI)',
  4: '商业发票(CI)',
};

const currencySymbolMap: Record<number, string> = {
  1: '¥',
  2: '$',
  3: '€',
  4: '£',
  5: '¥',
  6: 'HK$',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: { placeholder: '发票号/客户/标题', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'invoiceType',
      label: '类型',
      componentProps: { placeholder: '全部', allowClear: true, options: typeOptions },
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
  checkboxConfig: { checkMethod: () => true },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getInvoiceListApi({ page: page.currentPage, pageSize: page.pageSize, ...formValues });
      },
    },
  },
  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '发票号', field: 'invoiceNo', width: 160 },
    { title: '发票标题', field: 'title', minWidth: 150 },
    { title: '类型', field: 'invoiceType', width: 140, slots: { default: 'invoiceType' } },
    { title: '客户名称', field: 'customerName', minWidth: 120 },
    { title: '金额', field: 'amount', width: 130, slots: { default: 'amount' } },
    { title: '状态', field: 'status', width: 90, slots: { default: 'status' } },
    { title: '开票日期', field: 'invoiceDate', width: 110 },
    { title: '到期日', field: 'dueDate', width: 110 },
    { title: '创建时间', field: 'createTime', width: 160, slots: { default: 'createTime' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 150 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: InvoiceDrawer,
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
function handleView(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteInvoiceApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleBatchDelete() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    window.$message.warning('请选择要删除的发票');
    return;
  }
  const ids = records.map((r: any) => r.id);
  await deleteInvoiceApi(ids);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <SalesProcessGuide current-step="invoice" />
    <Grid :table-title="$t('page.sale.invoice.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:invoice:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新建发票
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:invoice:delete')"
          class="mr-2"
          @click="handleBatchDelete"
        >
          批量删除
        </Button>
      </template>

      <template #invoiceType="{ row }">
        <Tag :color="typeColorMap[row.invoiceType]">
          {{ typeLabelMap[row.invoiceType] || row.invoiceType }}
        </Tag>
      </template>

      <template #amount="{ row }">
        {{ currencySymbolMap[row.currency] || '¥' }} {{ row.amount?.toLocaleString?.() ?? row.amount }}
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
        <Button type="link" :icon="h(LucideEye)" @click="handleView(row)" />
        <Button
          v-if="accessStore.hasAccessCode('sale:invoice:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="handleEdit(row)"
        />
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: '发票' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('sale:invoice:delete')"
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
