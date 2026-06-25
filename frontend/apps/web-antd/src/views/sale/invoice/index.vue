<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideEye, LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteInvoiceApi, getInvoiceListApi } from '#/api';
import { $t } from '#/locales';

const accessStore = useAccessStore();

const typeOptions = [
  { label: 'PI (形式发票)', value: 'PI' },
  { label: 'CI (商业发票)', value: 'CI' },
  { label: 'VAT (增值税发票)', value: 'VAT' },
];

const statusOptions = [
  { label: '草稿', value: 'draft' },
  { label: '已开票', value: 'issued' },
  { label: '已付款', value: 'paid' },
  { label: '已取消', value: 'cancelled' },
];

const statusColorMap: Record<string, string> = {
  draft: 'default',
  issued: 'blue',
  paid: 'green',
  cancelled: 'red',
};

const statusLabelMap: Record<string, string> = {
  draft: '草稿',
  issued: '已开票',
  paid: '已付款',
  cancelled: '已取消',
};

const typeLabelMap: Record<string, string> = {
  PI: 'PI (形式发票)',
  CI: 'CI (商业发票)',
  VAT: 'VAT (增值税发票)',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '发票号',
      componentProps: { placeholder: '请输入发票号', allowClear: true },
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
    { title: '类型', field: 'invoiceType', width: 100, slots: { default: 'invoiceType' } },
    { title: '订单ID', field: 'orderId', width: 90 },
    { title: '客户ID', field: 'customerId', width: 90 },
    { title: '金额', field: 'amount', width: 130, slots: { default: 'amount' } },
    { title: '币种', field: 'currency', width: 60 },
    { title: '状态', field: 'status', width: 80, slots: { default: 'status' } },
    { title: '开票日期', field: 'issuedAt', width: 110 },
    { title: '到期日', field: 'dueDate', width: 110 },
    { title: '创建时间', field: 'createdAt', width: 160, slots: { default: 'createdAt' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 120 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

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
    <Grid :table-title="$t('page.sale.invoice.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:invoice:create')"
          type="primary"
          class="mr-2"
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
        <Tag>{{ typeLabelMap[row.invoiceType] || row.invoiceType }}</Tag>
      </template>

      <template #amount="{ row }">
        {{ row.currency }} {{ row.amount?.toLocaleString?.() ?? row.amount }}
      </template>

      <template #status="{ row }">
        <Tag :color="statusColorMap[row.status]">
          {{ statusLabelMap[row.status] || row.status }}
        </Tag>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt) }}
      </template>

      <template #action="{ row }">
        <Button type="link" :icon="h(LucideEye)" />
        <Button
          v-if="accessStore.hasAccessCode('sale:invoice:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
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
  </Page>
</template>