<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2, LucideEye } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Drawer, Modal, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteOpportunityApi, getOpportunityListApi } from '#/api';
import { $t } from '#/locales';
import OpportunityDrawer from './drawer.vue';
import OpportunityDetail from './detail.vue';

const accessStore = useAccessStore();

// 来源映射 - 对齐后端 LeadSource 枚举
const sourceLabelMap: Record<string, string> = {
  website: '官网', exhibition: '展会', social: '社交媒体', referral: '客户转介',
  cold_call: '陌生拜访', customs: '海关数据', email: '邮件营销', alibaba: '阿里国际站',
  amazon: 'Amazon', tiktok: 'TikTok', wechat: '微信', other: '其他',
};

// 详情抽屉
const detailVisible = ref(false);
const detailId = ref<number | null>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('商机ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }
function handleDetailEdit(opp: any) { closeDetail(); openDrawer(false, opp); }

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '商机名称',
      componentProps: { placeholder: '输入商机名称/编号搜索', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'stage',
      label: '销售阶段',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '资格审查', value: 'qualification' },
          { label: '需求分析', value: 'needs_analysis' },
          { label: '方案报价', value: 'proposal' },
          { label: '商务谈判', value: 'negotiation' },
          { label: '已成交', value: 'won' },
          { label: '已输单', value: 'lost' },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'source',
      label: '商机来源',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '官网', value: 'website' },
          { label: '展会', value: 'exhibition' },
          { label: '社交媒体', value: 'social' },
          { label: '客户转介', value: 'referral' },
          { label: '陌生拜访', value: 'cold_call' },
          { label: '海关数据', value: 'customs' },
          { label: '邮件营销', value: 'email' },
          { label: '阿里国际站', value: 'alibaba' },
          { label: 'Amazon', value: 'amazon' },
          { label: 'TikTok', value: 'tiktok' },
          { label: '微信', value: 'wechat' },
          { label: '其他', value: 'other' },
        ],
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'customerId',
      label: '客户ID',
      componentProps: { placeholder: '客户ID', allowClear: true, min: 0 },
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
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getOpportunityListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '商机编号', field: 'opportunityNo', width: 140 },
    { title: '商机名称', field: 'title', minWidth: 200, slots: { default: 'title' } },
    {
      title: '客户', field: 'customerName', width: 150,
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: '销售阶段', field: 'stage', width: 110,
      cellRender: {
        name: 'Tag',
        options: [
          { value: 'qualification', label: '资格审查', color: 'blue' },
          { value: 'needs_analysis', label: '需求分析', color: 'cyan' },
          { value: 'proposal', label: '方案报价', color: 'gold' },
          { value: 'negotiation', label: '商务谈判', color: 'orange' },
          { value: 'won', label: '已成交', color: 'green' },
          { value: 'lost', label: '已输单', color: 'red' },
        ],
      },
    },
    {
      title: '金额', field: 'amount', width: 140,
      formatter: ({ cellValue, row }: any) => {
        if (cellValue == null) return '-';
        const currency = row.currency || '';
        return `${currency} ${Number(cellValue).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
      },
    },
    {
      title: '概率', field: 'probability', width: 80, align: 'center',
      formatter: ({ cellValue }: any) => (cellValue == null ? '-' : `${cellValue}%`),
    },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    { title: '预计成交日', field: 'expectedCloseDate', width: 120 },
    { title: '负责人', field: 'assignee', width: 90 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 150,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: OpportunityDrawer,
  onClosed() { if (drawerApi.getData()?.needRefresh) gridApi.query(); },
});

function openDrawer(create: boolean, row?: any) { drawerApi.setData({ create, row }); drawerApi.open(); }
function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try { await deleteOpportunityApi([row.id]); message.success($t('ui.notification.delete_success')); }
  finally { row.pending = false; gridApi.query(); }
}

async function handleBatchDelete() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) { message.warning('请先选择要删除的商机'); return; }
  Modal.confirm({
    title: '批量删除',
    content: `确定批量删除 ${records.length} 个商机？`,
    onOk: async () => {
      try {
        const ids = records.map((r: any) => r.id);
        await deleteOpportunityApi(ids);
        message.success(`已删除 ${records.length} 个商机`);
        gridApi.query();
      } catch { /* ignore */ }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.crm.opportunity.title')">
      <template #toolbar-tools>
        <Button v-if="accessStore.hasAccessCode('crm:opportunity:create')" type="primary" class="mr-2" @click="handleCreate">
          {{ $t('page.crm.opportunity.button.create') }}
        </Button>
        <Button @click="handleBatchDelete" class="mr-2" danger ghost>批量删除</Button>
      </template>

      <template #createdAt="{ row }">{{ formatDateTime(row.createdAt) }}</template>

      <template #title="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.title }}</a>
      </template>

      <template #action="{ row }">
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" />
        <Button v-if="accessStore.hasAccessCode('crm:opportunity:edit')" type="link" :icon="h(LucideFilePenLine)" @click="() => handleEdit(row)" />
        <Popconfirm :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.opportunity.title') })" :ok-text="$t('ui.button.ok')" :cancel-text="$t('ui.button.cancel')" @confirm="handleDelete(row)">
          <Button v-if="accessStore.hasAccessCode('crm:opportunity:delete')" type="link" danger :icon="h(LucideTrash2)" />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />

    <Drawer v-model:open="detailVisible" :width="1000" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="商机详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <OpportunityDetail v-if="detailId" :id="detailId" @edit="handleDetailEdit" />
    </Drawer>
  </Page>
</template>