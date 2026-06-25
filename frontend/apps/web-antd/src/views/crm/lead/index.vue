<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Drawer, Modal, Tabs, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteLeadApi, getLeadListApi, addLeadToPoolApi } from '#/api';
import { $t } from '#/locales';
import LeadDrawerComp from './drawer.vue';
import LeadDetail from './detail.vue';
import LeadFollowupDrawer from './followup-drawer.vue';

const accessStore = useAccessStore();

const statusTabs = [
  { key: 0, label: '全部' },
  { key: 6, label: '未审查' },
  { key: 7, label: '审查中' },
  { key: 4, label: '无效线索' },
];

const activeTab = ref(0);

const sourceLabelMap: Record<string, string> = {
  website: '官网', exhibition: '展会', social: '社交媒体', referral: '客户转介',
  cold_call: '陌生拜访', customs: '海关数据', email: '邮件营销', alibaba: '阿里国际站',
  amazon: 'Amazon', tiktok: 'TikTok', wechat: '微信', other: '其他',
};

const industryLabelMap: Record<string, string> = {
  retail: '零售', wholesale: '批发', manufacturer: '制造', trade_agent: '贸易代理',
  ecommerce: '电商', wechat_business: '微商', social: '社交电商', other: '其他',
};

const detailVisible = ref(false);
const detailId = ref<number | null>(null);

const followupVisible = ref(false);
const followupId = ref<number | null>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('线索ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }
function handleDetailEdit(lead: any) { closeDetail(); openDrawer(false, lead); }

function openFollowup(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('线索ID不存在'); return; }
  followupId.value = Number(id);
  followupVisible.value = true;
}
function closeFollowup() { followupVisible.value = false; followupId.value = null; }

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
      fieldName: 'source',
      label: '来源',
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
        return await getLeadListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: activeTab.value || undefined,
          ...formValues,
        });
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '公司名称', field: 'companyName', minWidth: 180, slots: { default: 'companyName' } },
    { title: '所属行业', field: 'industry', width: 100, formatter: ({ cellValue }: any) => industryLabelMap[cellValue] || cellValue || '-' },
    { title: '联系人', field: 'contactName', width: 100 },
    {
      title: '状态', field: 'status', width: 90,
      cellRender: {
        name: 'Tag',
        options: [
          { value: 1, label: '新客', color: 'blue' },
          { value: 2, label: '跟进中', color: 'cyan' },
          { value: 3, label: '已成交', color: 'green' },
          { value: 4, label: '无效线索', color: 'default' },
          { value: 5, label: '已回收', color: 'orange' },
          { value: 6, label: '未核查', color: 'blue' },
          { value: 7, label: '核查中', color: 'cyan' },
          { value: 8, label: '有效线索', color: 'green' },
        ],
      },
    },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    { title: '邮箱', field: 'email', width: 160 },
    { title: '手机', field: 'mobile', width: 130 },
    { title: '国家', field: 'country', width: 80 },
    { title: '负责人', field: 'assignee', width: 90 },
    { title: '创建人', field: 'createdByName', width: 90 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createTime' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 280,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
    connectedComponent: LeadDrawerComp,
  onClosed() { if (drawerApi.getData()?.needRefresh) gridApi.query(); },
});

function openDrawer(create: boolean, row?: any) { drawerApi.setData({ create, row }); drawerApi.open(); }
function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try { await deleteLeadApi([row.id]); message.success($t('ui.notification.delete_success')); }
  finally { row.pending = false; gridApi.query(); }
}

async function handleBatchDelete() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) { message.warning('请先选择要删除的线索'); return; }
  Modal.confirm({
    title: '批量删除',
    content: `确定批量删除 ${records.length} 条线索？`,
    onOk: async () => {
      try {
        const ids = records.map((r: any) => r.id);
        await deleteLeadApi(ids);
        message.success(`已删除 ${records.length} 条线索`);
        gridApi.query();
      } catch { /* ignore */ }
    },
  });
}

async function handleFollow(row: any) {
  openFollowup(row);
}

async function handleAddToPool(row: any) {
  Modal.confirm({
    title: '加入线索池',
    content: `确定将线索"${row.companyName}"加入线索池吗？`,
    onOk: async () => {
      try {
        await addLeadToPoolApi(row.id);
        message.success('已加入线索池');
        gridApi.query();
      } catch (e) {
        message.error('操作失败');
      }
    },
  });
}

function handleTabChange(key: number | string) {
  activeTab.value = Number(key);
  gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <div class="mb-4">
      <Tabs v-model:activeKey="activeTab" @change="handleTabChange" type="card">
        <Tabs.TabPane v-for="tab in statusTabs" :key="tab.key" :tab="tab.label" />
      </Tabs>
    </div>

    <Grid :table-title="$t('page.crm.lead.title')">
      <template #toolbar-tools>
        <Button v-if="accessStore.hasAccessCode('crm:lead:create')" type="primary" class="mr-2" @click="handleCreate">
          {{ $t('page.crm.lead.button.create') }}
        </Button>
        <Button @click="handleBatchDelete" class="mr-2" danger ghost>批量删除</Button>
      </template>

      <template #createTime="{ row }">{{ formatDateTime(row.createTime) }}</template>

      <template #companyName="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.companyName }}</a>
      </template>

      <template #action="{ row }">
        <Button type="link" @click="() => handleFollow(row)">跟进</Button>
        <Button v-if="row.status !== 4" type="link" @click="() => handleAddToPool(row)">加入线索池</Button>
        <Button v-if="accessStore.hasAccessCode('crm:lead:edit')" type="link" :icon="h(LucideFilePenLine)" @click="() => handleEdit(row)" title="编辑" />
        <Popconfirm :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.lead.title') })" :ok-text="$t('ui.button.ok')" :cancel-text="$t('ui.button.cancel')" @confirm="handleDelete(row)">
          <Button v-if="accessStore.hasAccessCode('crm:lead:delete')" type="link" danger :icon="h(LucideTrash2)" title="删除" />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />

    <Drawer v-model:open="detailVisible" :width="960" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="线索详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <LeadDetail v-if="detailId" :id="detailId" @edit="handleDetailEdit" />
    </Drawer>

    <Drawer v-model:open="followupVisible" :width="1100" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="跟进线索" :body-style="{ padding: 0, height: '100%' }" @close="closeFollowup">
      <LeadFollowupDrawer v-if="followupId" :id="followupId" />
    </Drawer>
  </Page>
</template>