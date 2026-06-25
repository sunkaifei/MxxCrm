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
import { deleteCustomerApi, getCustomerListApi } from '#/api';
import { $t } from '#/locales';
import CustomerDrawer from './drawer.vue';
import CustomerDetail from './detail.vue';

const accessStore = useAccessStore();

// 等级颜色映射 - 1:无级别 2:重点客户 3:优质客户 4:普通客户 5:其他
const levelColorMap: Record<string, string> = {
  1: 'default', 2: 'red', 3: 'orange', 4: 'blue', 5: 'green',
};
const levelLabelMap: Record<string, string> = {
  1: '无级别', 2: '重点客户', 3: '优质客户', 4: '普通客户', 5: '其他',
};

// 行业映射 - 对齐后端 IndustryType 枚举
const industryLabelMap: Record<string, string> = {
  retail: '零售', wholesale: '批发', manufacturer: '制造', trade_agent: '贸易代理',
  ecommerce: '电商', wechat_business: '微商', social: '社交电商', other: '其他',
};

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
  if (!id) { message.error('客户ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }
function handleDetailEdit(customer: any) { closeDetail(); openDrawer(false, customer); }

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'companyName',
      label: '公司名称',
      componentProps: { placeholder: '输入公司名称搜索', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'level',
      label: '客户等级',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '无级别', value: 1 },
          { label: '重点客户', value: 2 },
          { label: '优质客户', value: 3 },
          { label: '普通客户', value: 4 },
          { label: '其他', value: 5 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'industry',
      label: '行业',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '零售', value: 'retail' },
          { label: '批发', value: 'wholesale' },
          { label: '制造', value: 'manufacturer' },
          { label: '贸易代理', value: 'trade_agent' },
          { label: '电商', value: 'ecommerce' },
          { label: '微商', value: 'wechat_business' },
          { label: '社交电商', value: 'social' },
          { label: '其他', value: 'other' },
        ],
      },
    },
    {
      component: 'Input',
      fieldName: 'country',
      label: '国家',
      componentProps: { placeholder: '输入国家', allowClear: true },
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
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getCustomerListApi({
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
    { title: '公司名称', field: 'companyName', minWidth: 200, slots: { default: 'companyName' } },
    {
      title: '等级', field: 'level', width: 80,
      cellRender: {
        name: 'Tag',
        options: [
          { value: 1, label: '无级别', color: 'default' },
          { value: 2, label: '重点客户', color: 'red' },
          { value: 3, label: '优质客户', color: 'orange' },
          { value: 4, label: '普通客户', color: 'blue' },
          { value: 5, label: '其他', color: 'green' },
        ],
      },
    },
    {
      title: '行业', field: 'industry', width: 90,
      formatter: ({ cellValue }: any) => industryLabelMap[cellValue] || cellValue || '-',
    },
    { title: '国家', field: 'country', width: 80 },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    {
      title: '联系人', field: 'contactCount', width: 70, align: 'center',
      formatter: ({ cellValue }: any) => cellValue ?? '-',
    },
    {
      title: '商机数', field: 'opportunityCount', width: 70, align: 'center',
      formatter: ({ cellValue }: any) => cellValue ?? '-',
    },
    { title: '负责人', field: 'assignee', width: 90 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 140,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: CustomerDrawer,
  onClosed() { if (drawerApi.getData()?.needRefresh) gridApi.query(); },
});

function openDrawer(create: boolean, row?: any) { drawerApi.setData({ create, row }); drawerApi.open(); }
function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try { await deleteCustomerApi(row.id); message.success($t('ui.notification.delete_success')); }
  finally { row.pending = false; gridApi.query(); }
}

async function handleBatchDelete() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) { message.warning('请先选择要删除的客户'); return; }
  Modal.confirm({
    title: '批量删除',
    content: `确定批量删除 ${records.length} 个客户？`,
    onOk: async () => {
      try {
        await Promise.all(records.map((r: any) => deleteCustomerApi(r.id)));
        message.success(`已删除 ${records.length} 个客户`);
        gridApi.query();
      } catch { /* ignore */ }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.crm.customer.title')">
      <template #toolbar-tools>
        <Button v-if="accessStore.hasAccessCode('crm:customer:create')" type="primary" class="mr-2" @click="handleCreate">
          {{ $t('page.crm.customer.button.create') }}
        </Button>
        <Button @click="handleBatchDelete" class="mr-2" danger ghost>批量删除</Button>
      </template>

      <template #createdAt="{ row }">{{ formatDateTime(row.createdAt) }}</template>

      <template #companyName="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.companyName }}</a>
      </template>

      <template #action="{ row }">
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" />
        <Button v-if="accessStore.hasAccessCode('crm:customer:edit')" type="link" :icon="h(LucideFilePenLine)" @click="() => handleEdit(row)" />
        <Popconfirm :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.customer.title') })" :ok-text="$t('ui.button.ok')" :cancel-text="$t('ui.button.cancel')" @confirm="handleDelete(row)">
          <Button v-if="accessStore.hasAccessCode('crm:customer:delete')" type="link" danger :icon="h(LucideTrash2)" />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />

    <Drawer v-model:open="detailVisible" :width="1000" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="客户详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <CustomerDetail v-if="detailId" :id="detailId" @edit="handleDetailEdit" />
    </Drawer>
  </Page>
</template>