<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideEye } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, Modal, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getCustomerPoolListApi, claimCustomerApi } from '#/api';
import { $t } from '#/locales';
import CustomerDrawer from '../customer/drawer.vue';
import CustomerDetail from '../customer/detail.vue';

const accessStore = useAccessStore();

const sourceLabelMap: Record<string, string> = {
  website: '官网', exhibition: '展会', social: '社交媒体', referral: '客户转介',
  cold_call: '陌生拜访', customs: '海关数据', email: '邮件营销', alibaba: '阿里国际站',
  amazon: 'Amazon', tiktok: 'TikTok', wechat: '微信', other: '其他',
};

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
      component: 'Input',
      fieldName: 'country',
      label: '国家',
      componentProps: { placeholder: '输入国家', allowClear: true },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  height: 'auto',
  pagerConfig: {},
  cellConfig: { isHover: true },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getCustomerPoolListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
  },

  columns: [
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
    { title: '国家', field: 'country', width: 80 },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    { title: '录入人', field: 'createdByName', width: 90 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 160,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: CustomerDrawer,
  onClosed() { if (drawerApi.getData()?.needRefresh) gridApi.query(); },
});

function openDrawer(create: boolean, row?: any) { drawerApi.setData({ create, row }); drawerApi.open(); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleClaim(row: any) {
  Modal.confirm({
    title: '领取客户',
    content: `确定领取客户"${row.companyName}"吗？领取后将转入您的客户管理。`,
    onOk: async () => {
      try {
        await claimCustomerApi(Number(row.id));
        message.success('领取成功');
        gridApi.query();
      } catch {
        // 错误提示由拦截器处理
      }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="公海客户">
      <template #createdAt="{ row }">{{ formatDateTime(row.createTime) }}</template>

      <template #companyName="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.companyName }}</a>
      </template>

      <template #action="{ row }">
        <Button v-if="accessStore.hasAccessCode('crm:customer:edit')" type="link" @click="() => handleClaim(row)">
          领取
        </Button>
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" />
        <Button v-if="accessStore.hasAccessCode('crm:customer:edit')" type="link" :icon="h(LucideFilePenLine)" @click="() => handleEdit(row)" />
      </template>
    </Grid>
    <FormDrawer />

    <Drawer v-model:open="detailVisible" :width="1000" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="客户详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <CustomerDetail v-if="detailId" :id="detailId" @edit="handleDetailEdit" />
    </Drawer>
  </Page>
</template>
