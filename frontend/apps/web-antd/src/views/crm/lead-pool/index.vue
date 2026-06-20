<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideEye } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getLeadPoolListApi } from '#/api';
import { $t } from '#/locales';
import LeadDetail from '../lead/detail.vue';

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
  if (!id) { message.error('线索ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }

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
  rowConfig: { isHover: true },
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getLeadPoolListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: 'valid',
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
      cellRender: {
        name: 'Tag',
        options: [
          { value: 'valid', label: '有效', color: 'green' },
        ],
      },
    },
    { title: '行业', field: 'industry', width: 90 },
    { title: '国家', field: 'country', width: 80 },
    { title: '负责人', field: 'assignee', width: 90 },
    { title: '创建人', field: 'createdBy', width: 90 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 80,
    },
  ],
};

const [Grid] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.crm.leadPool.title')">
      <template #createdAt="{ row }">{{ formatDateTime(row.createdAt) }}</template>

      <template #companyName="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.companyName }}</a>
      </template>

      <template #action="{ row }">
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" />
      </template>
    </Grid>

    <Drawer v-model:open="detailVisible" :width="960" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="线索详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <LeadDetail v-if="detailId" :id="detailId" />
    </Drawer>
  </Page>
</template>