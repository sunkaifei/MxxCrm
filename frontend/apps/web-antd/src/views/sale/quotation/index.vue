<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideEye, LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, Popconfirm, Select, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteQuotationApi, getQuotationListApi } from '#/api';
import { $t } from '#/locales';
import QuotationDetail from './detail.vue';
import QuotationDrawer from './drawer.vue';

const accessStore = useAccessStore();

const detailVisible = ref(false);
const detailId = ref(0);

const statusOptions = [
  { label: '草稿', value: 'draft' },
  { label: '已发送', value: 'sent' },
  { label: '已接受', value: 'accepted' },
  { label: '已拒绝', value: 'rejected' },
  { label: '已过期', value: 'expired' },
];

const statusColorMap: Record<string, string> = {
  draft: 'default',
  sent: 'blue',
  accepted: 'green',
  rejected: 'red',
  expired: 'orange',
  revised: 'purple',
};

const statusLabelMap: Record<string, string> = {
  draft: '草稿',
  sent: '已发送',
  accepted: '已接受',
  rejected: '已拒绝',
  expired: '已过期',
  revised: '已修订',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '报价单号/标题',
      componentProps: { placeholder: '请输入报价单号或标题', allowClear: true },
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
  rowConfig: { isHover: true },
  stripe: true,
  checkboxConfig: { checkMethod: () => true },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getQuotationListApi({ page: page.currentPage, pageSize: page.pageSize, ...formValues });
      },
    },
  },
  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '报价单号', field: 'quotationNo', width: 150, slots: { default: 'quotationNo' } },
    { title: '标题', field: 'title', width: 200 },
    { title: '客户ID', field: 'customerId', width: 100 },
    { title: '合计金额', field: 'grandTotal', width: 140, slots: { default: 'grandTotal' } },
    { title: '币种', field: 'currency', width: 60 },
    { title: '状态', field: 'status', width: 90, slots: { default: 'status' } },
    { title: '有效期', field: 'validUntil', width: 110 },
    { title: '负责人', field: 'assignedTo', width: 80 },
    { title: '创建时间', field: 'createdAt', width: 160, slots: { default: 'createdAt' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 140 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: QuotationDrawer,
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
    await deleteQuotationApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleBatchDelete() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    window.$message.warning('请选择要删除的报价单');
    return;
  }
  const ids = records.map((r: any) => r.id);
  await deleteQuotationApi(ids);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}

function openDetail(row: any) {
  detailId.value = row.id;
  detailVisible.value = true;
}

function handleDetailEdit(id: string) {
  detailVisible.value = false;
  const record = gridApi.grid.getTableData().fullData.find((r: any) => String(r.id) === id);
  if (record) openDrawer(false, record);
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.sale.quotation.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:quotation:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新建报价单
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:quotation:delete')"
          class="mr-2"
          @click="handleBatchDelete"
        >
          批量删除
        </Button>
      </template>

      <template #quotationNo="{ row }">
        <a class="text-blue-600 cursor-pointer" @click="openDetail(row)">{{ row.quotationNo }}</a>
      </template>

      <template #grandTotal="{ row }">
        {{ row.currency }} {{ row.grandTotal?.toLocaleString?.() ?? row.grandTotal }}
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
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" />
        <Button
          v-if="accessStore.hasAccessCode('sale:quotation:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: '报价单' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('sale:quotation:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />
    <Drawer
      v-model:open="detailVisible"
      placement="right"
      :width="960"
      title="报价单详情"
      :destroy-on-close="true"
      :body-style="{ padding: 0 }"
    >
      <QuotationDetail v-if="detailVisible" :id="detailId" @edit="handleDetailEdit" />
    </Drawer>
  </Page>
</template>