<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideEye, LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, Modal, Popconfirm, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getLeadPoolListApi, claimLeadApi, deleteLeadPoolApi } from '#/api';
import { $t } from '#/locales';
import LeadDrawerComp from '../lead/drawer.vue';
import LeadDetail from '../lead/detail.vue';

const accessStore = useAccessStore();

const sourceLabelMap: Record<string, string> = {
  website: '官网', exhibition: '展会', social: '社交媒体', referral: '客户转介',
  cold_call: '陌生拜访', customs: '海关数据', email: '邮件营销', alibaba: '阿里国际站',
  amazon: 'Amazon', tiktok: 'TikTok', wechat: '微信', other: '其他',
};

const industryLabelMap: Record<number, string> = {
  1: '零售', 2: '批发', 3: '制造', 4: '贸易代理',
  5: '电商', 6: '微商', 7: '社交电商', 8: '其他',
};

const statusOptions = [
  { value: 1, label: '新客', color: 'blue' },
  { value: 2, label: '跟进中', color: 'cyan' },
  { value: 3, label: '已成交', color: 'green' },
  { value: 4, label: '无效线索', color: 'default' },
  { value: 5, label: '已回收', color: 'orange' },
  { value: 6, label: '未核查', color: 'blue' },
  { value: 7, label: '核查中', color: 'cyan' },
  { value: 8, label: '有效线索', color: 'green' },
];

const detailVisible = ref(false);
const detailId = ref<number | null>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('线索ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }
function handleDetailEdit(lead: any) { closeDetail(); openDrawer(false, lead); }

async function handleClaim(row: any) {
  Modal.confirm({
    title: '领取线索',
    content: `确定领取线索"${row.companyName}"吗？领取后将转为您的客户。`,
    onOk: async () => {
      try {
        await claimLeadApi(row.id);
        message.success('领取成功，已转为客户');
        gridApi.query();
      } catch {
        // 错误提示由 requestClient 拦截器处理，无需重复提示
      }
    },
  });
}

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
    {
      component: 'Input',
      fieldName: 'industry',
      label: '行业',
      componentProps: { placeholder: '输入行业', allowClear: true },
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
        return await getLeadPoolListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: 'pool',
          ...formValues,
        });
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '公司名称', field: 'companyName', minWidth: 180, slots: { default: 'companyName' } },
    { title: '联系人', field: 'contactName', width: 100 },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    {
      title: '状态', field: 'status', width: 90,
      cellRender: { name: 'Tag', options: statusOptions },
    },
    { title: '行业', field: 'industry', width: 90, formatter: ({ cellValue }: any) => industryLabelMap[cellValue] || cellValue || '-' },
    { title: '国家', field: 'country', width: 80 },
    { title: '创建人', field: 'createdByName', width: 90 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 240,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: LeadDrawerComp,
  onClosed() { if (drawerApi.getData()?.needRefresh) gridApi.query(); },
});

function openDrawer(create: boolean, row?: any) { drawerApi.setData({ create, row, fromPool: create }); drawerApi.open(); }
function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteLeadPoolApi([row.id]);
    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.crm.leadPool.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('crm:lead:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.crm.leadPool.button.create') }}
        </Button>
      </template>

      <template #createdAt="{ row }">{{ formatDateTime(row.createTime) }}</template>

      <template #companyName="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.companyName }}</a>
      </template>

      <template #action="{ row }">
        <Button type="link" @click="() => handleClaim(row)">领取</Button>
        <Button
          v-if="accessStore.hasAccessCode('crm:lead:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
          :title="$t('page.crm.leadPool.button.edit')"
        />
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" :title="$t('page.crm.leadPool.button.info')" />
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.leadPool.title') })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('crm:lead-pool:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
            :title="$t('page.crm.leadPool.button.delete')"
          />
        </Popconfirm>
      </template>
    </Grid>

    <FormDrawer />

    <Drawer v-model:open="detailVisible" :width="960" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" :title="$t('page.crm.leadPool.button.info')" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <LeadDetail v-if="detailId" :id="detailId" @edit="handleDetailEdit" />
    </Drawer>
  </Page>
</template>