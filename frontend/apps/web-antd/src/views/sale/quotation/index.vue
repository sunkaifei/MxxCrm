<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideEye, LucideFilePenLine, LucideSend, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, Modal, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteQuotationApi, convertToOrderApi, getQuotationListApi, submitQuotationApprovalApi } from '#/api';
import { $t } from '#/locales';
import QuotationDetail from './detail.vue';
import QuotationDrawer from './drawer.vue';
import SalesProcessGuide from '../components/SalesProcessGuide.vue';

const accessStore = useAccessStore();

const detailVisible = ref(false);
const detailId = ref(0);

const approvalStatusOptions = [
  { label: '草稿', value: 1 },
  { label: '审批中', value: 2 },
  { label: '已通过', value: 3 },
  { label: '已驳回', value: 4 },
];

const approvalStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'processing',
  3: 'success',
  4: 'error',
};

const approvalStatusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '审批中',
  3: '已通过',
  4: '已驳回',
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
      label: '报价单号/标题',
      componentProps: { placeholder: '请输入报价单号或标题', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'approvalStatus',
      label: '审批状态',
      componentProps: { placeholder: '全部', allowClear: true, options: approvalStatusOptions },
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
        return await getQuotationListApi({ page: page.currentPage, pageSize: page.pageSize, ...formValues });
      },
    },
  },
  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '报价单号', field: 'quotationNo', width: 150, slots: { default: 'quotationNo' } },
    { title: '标题', field: 'title', width: 200 },
    { title: '客户名称', field: 'customerName', width: 140 },
    { title: '报价金额', field: 'grandTotal', width: 140, slots: { default: 'grandTotal' } },
    { title: '审批状态', field: 'approvalStatus', width: 100, slots: { default: 'approvalStatus' } },
    { title: '报价日期', field: 'quotationDate', width: 110 },
    { title: '创建时间', field: 'createTime', width: 160, slots: { default: 'createTime' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 200 },
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

function handleSubmitApproval(row: any) {
  Modal.confirm({
    title: '提交审批',
    content: '确认提交此报价单进行审批？提交后将进入审批流程，审批期间无法修改。',
    okText: '确认提交',
    cancelText: '取消',
    onOk: async () => {
      try {
        await submitQuotationApprovalApi(row.id);
        window.$message.success('已提交审批');
        gridApi.query();
      } catch {
        // error handled by interceptor
      }
    },
  });
}

async function handleConvertToOrder(row: any) {
  row.pending = true;
  try {
    await convertToOrderApi(row.id);
    window.$message.success('已转为订单');
  } finally {
    row.pending = false;
    gridApi.query();
  }
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
    <SalesProcessGuide current-step="quotation" />
    <Grid :table-title="$t('page.sale.quotation.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:quotation:save')"
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
        {{ currencySymbolMap[row.currency] || '¥' }} {{ row.grandTotal?.toLocaleString?.() ?? row.grandTotal }}
      </template>

      <template #approvalStatus="{ row }">
        <Tag :color="approvalStatusColorMap[row.approvalStatus]">
          {{ approvalStatusLabelMap[row.approvalStatus] || '草稿' }}
        </Tag>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" />
        <Button
          v-if="accessStore.hasAccessCode('sale:quotation:edit') && (row.approvalStatus === 1 || row.approvalStatus === 4)"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Button
          v-if="accessStore.hasAccessCode('sale:quotation:update') && (row.approvalStatus === 1 || row.approvalStatus === 4)"
          type="link"
          size="small"
          :icon="h(LucideSend)"
          @click="() => handleSubmitApproval(row)"
        >
          提交审批
        </Button>
        <Popconfirm
          v-if="row.approvalStatus === 3"
          title="确认将此报价单转为订单？"
          ok-text="确认"
          cancel-text="取消"
          @confirm="handleConvertToOrder(row)"
        >
          <Button type="link" size="small">转订单</Button>
        </Popconfirm>
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
