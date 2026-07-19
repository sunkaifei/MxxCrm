<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2, LucideEye } from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Drawer, Modal, Tabs, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteOpportunityApi, getOpportunityListApi } from '#/api';
import { $t } from '#/locales';
import OpportunityDetail from './detail.vue';
import CustomerDetailDrawer from '../components/CustomerDetailDrawer.vue';
import SalesProcessGuide from '../../sale/components/SalesProcessGuide.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();

// 全部商机 Tab 显示条件：超级管理员 / 系统管理员 / data_scope=全部数据
const canViewAll = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 1;
});

// 下属商机 Tab 显示条件：超级管理员 / 系统管理员 / 数据权限含部门（2/3/4）
const canViewSubordinate = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 2 || dataScope === 3 || dataScope === 4;
});

const activeTab = ref('my');
const allTabList = [
  { key: 'all', label: '全部商机' },
  { key: 'my', label: '我的商机' },
  { key: 'subordinate', label: '下属商机' },
];
const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter(t => keys.includes(t.key));
});
// 当Tab权限变化时，确保当前激活的Tab仍然可见
watch(tabList, (newTabs) => {
  const keys = newTabs.map(t => t.key);
  if (!keys.includes(activeTab.value) && keys.length > 0) {
    activeTab.value = keys[0];
  }
}, { immediate: true });

function handleTabChange(key: string) {
  activeTab.value = key;
  gridApi.query();
}

// 来源映射 - 对齐后端 LeadSource 枚举（数字值）
const sourceLabelMap: Record<string, string> = {
  1: '官网', 2: '展会', 3: '社交媒体', 4: '客户转介',
  5: '陌生拜访', 6: '海关数据', 7: '邮件营销', 8: '阿里国际站',
  9: 'Amazon', 10: 'TikTok', 11: '微信', 12: '其他',
};

// 币种标签映射 - 对齐后端 CurrencyCode 枚举（数字值）
const currencyLabelMap: Record<number, string> = {
  1: 'CNY', 2: 'USD', 3: 'EUR', 4: 'GBP', 5: 'JPY', 6: 'HKD', 7: 'AUD',
};

// 详情抽屉
const detailVisible = ref(false);
const detailId = ref<number | null>(null);
const detailTitle = computed(() => detailId.value ? '商机详情' : '新建商机');

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('商机ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }
// 详情页内已支持内联编辑，edit 事件仅刷新列表
function handleDetailEdit() { gridApi.query(); }

// 客户详情抽屉
const customerDetailVisible = ref(false);
const customerDetailId = ref<number | string | undefined>(undefined);

function openCustomerDetail(row: any) {
  const id = row.customerId ?? row.customer_id;
  if (!id) { message.error('客户ID不存在'); return; }
  customerDetailId.value = Number(id);
  customerDetailVisible.value = true;
}

function handleConverted(_quotationId: number | string) {
  gridApi.query();
}

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
          { label: '初步沟通', value: 1 },
          { label: '需求确认', value: 2 },
          { label: '方案沟通', value: 3 },
          { label: '已报价', value: 4 },
          { label: '成交/丢单', value: 5 },
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
          { label: '官网', value: 1 },
          { label: '展会', value: 2 },
          { label: '社交媒体', value: 3 },
          { label: '客户转介', value: 4 },
          { label: '陌生拜访', value: 5 },
          { label: '海关数据', value: 6 },
          { label: '邮件营销', value: 7 },
          { label: '阿里国际站', value: 8 },
          { label: 'Amazon', value: 9 },
          { label: 'TikTok', value: 10 },
          { label: '微信', value: 11 },
          { label: '其他', value: 12 },
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
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true },
  rowConfig: { height: 'auto' },
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getOpportunityListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
          ...formValues,
        });
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    
    { title: '商机名称', field: 'title', minWidth: 200, align: 'left', headerAlign: 'center', slots: { default: 'title' } },
    { title: '客户', field: 'customerName', width: 150, align: 'left', headerAlign: 'center', slots: { default: 'customerName' } },
    {
      title: '销售阶段', field: 'stage', width: 110,
      formatter: ({ cellValue }: any) => {
        const stageMap: Record<number, string> = { 1: '初步沟通', 2: '需求确认', 3: '方案沟通', 4: '已报价', 5: '成交/丢单' };
        return stageMap[cellValue] ?? '-';
      },
    },
    {
      title: '预算金额', field: 'amount', width: 140,
      formatter: ({ cellValue, row }: any) => {
        if (cellValue == null) return '-';
        const currencyLabel = currencyLabelMap[row.currency] || '';
        return `${currencyLabel} ${Number(cellValue).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
      },
    },
    { title: '报价次数', field: 'quoteCount', width: 90, align: 'center', formatter: ({ cellValue }: any) => cellValue ?? 0 },
    {
      title: '概率', field: 'probability', width: 80, align: 'center',
      formatter: ({ cellValue }: any) => (cellValue == null ? '-' : `${cellValue}%`),
    },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    { title: '预计成交日', field: 'expectedCloseDate', width: 120 },
    { title: '录入人', field: 'createdByName', width: 90 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 150,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function handleCreate() {
  detailId.value = null;
  detailVisible.value = true;
}

function handleCreated(id: number | string) {
  detailId.value = Number(id);
  gridApi.query();
}

// 编辑改为打开详情页（详情页内已有内联编辑表单）
function handleEdit(row: any) { openDetail(row); }

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
  <Page>
    <SalesProcessGuide current-step="opportunity" />
    <Grid :table-title="$t('page.crm.opportunity.title')">
      <template #form-header>
        <Tabs v-model:activeKey="activeTab" class="mb-3" @change="handleTabChange">
          <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Button v-if="accessStore.hasAccessCode('crm:opportunity:save')" type="primary" class="mr-2" @click="handleCreate">
          {{ $t('page.crm.opportunity.button.create') }}
        </Button>
        <Button @click="handleBatchDelete" class="mr-2" danger ghost>批量删除</Button>
      </template>

      <template #createdAt="{ row }">{{ formatDateTime(row.createTime) }}</template>

      <template #title="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.title }}</a>
      </template>

      <template #customerName="{ row }">
        <a v-if="row.customerId" class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openCustomerDetail(row)">{{ row.customerName || '-' }}</a>
        <span v-else>{{ row.customerName || '-' }}</span>
      </template>

      <template #action="{ row }">
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" />
        <Button v-if="accessStore.hasAccessCode('crm:opportunity:update')" type="link" :icon="h(LucideFilePenLine)" @click="() => handleEdit(row)" />
        <Popconfirm :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.opportunity.title') })" :ok-text="$t('ui.button.ok')" :cancel-text="$t('ui.button.cancel')" @confirm="handleDelete(row)">
          <Button v-if="accessStore.hasAccessCode('crm:opportunity:delete')" type="link" danger :icon="h(LucideTrash2)" />
        </Popconfirm>
      </template>
    </Grid>

    <Drawer v-model:open="detailVisible" :width="1200" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" :title="detailTitle" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <OpportunityDetail :id="detailId ?? undefined" @edit="handleDetailEdit" @converted="handleConverted" @created="handleCreated" />
    </Drawer>

    <CustomerDetailDrawer v-model:visible="customerDetailVisible" :id="customerDetailId" />
  </Page>
</template>