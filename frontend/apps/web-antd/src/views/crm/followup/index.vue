<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideEye } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getFollowupListApi } from '#/api';
import { $t } from '#/locales';
import FollowupDetail from './detail.vue';

// 跟进方式映射
const activityLabelMap: Record<number, string> = {
  1: '电话', 2: '拜访', 3: '邮件', 4: '会议',
  5: 'WhatsApp', 6: '微信', 7: '其他',
};

// 详情抽屉
const detailVisible = ref(false);
const detailId = ref<number | null>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('跟进记录ID不存在'); return; }
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
      fieldName: 'customerName',
      label: '客户',
      componentProps: { placeholder: '输入客户名称', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'activityType',
      label: '跟进方式',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '电话', value: 1 },
          { label: '拜访', value: 2 },
          { label: '邮件', value: 3 },
          { label: '会议', value: 4 },
          { label: 'WhatsApp', value: 5 },
          { label: '微信', value: 6 },
          { label: '其他', value: 7 },
        ],
      },
    },
    {
      component: 'Input',
      fieldName: 'subject',
      label: '主题',
      componentProps: { placeholder: '输入跟进主题', allowClear: true },
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
        return await getFollowupListApi({
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
    { title: '主题', field: 'subject', minWidth: 180, slots: { default: 'subject' } },
    {
      title: '跟进方式', field: 'activityType', width: 90,
      cellRender: {
        name: 'Tag',
        options: [
          { value: 1, label: '电话', color: 'blue' },
          { value: 2, label: '拜访', color: 'cyan' },
          { value: 3, label: '邮件', color: 'purple' },
          { value: 4, label: '会议', color: 'orange' },
          { value: 5, label: 'WhatsApp', color: 'lime' },
          { value: 6, label: '微信', color: 'lime' },
          { value: 7, label: '其他', color: 'default' },
        ],
      },
    },
    { title: '客户', field: 'customerName', width: 150 },
    { title: '跟进内容', field: 'content', minWidth: 200, slots: { default: 'content' } },
    {
      title: '跟进时间', field: 'followTime', slots: { default: 'followTimeSlot' }, width: 160,
    },
    {
      title: '下次跟进', field: 'nextFollowDate', width: 120,
      formatter: ({ cellValue }: any) => cellValue || '-',
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
    <Grid :table-title="$t('page.crm.followup.title')">
      <template #followTimeSlot="{ row }">
        {{ formatDateTime(row.followTime) }}
      </template>

      <template #subject="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.subject || '未命名跟进' }}</a>
      </template>

      <template #content="{ row }">
        <span class="text-gray-600">{{ row.content?.length > 50 ? row.content.slice(0, 50) + '...' : row.content }}</span>
      </template>

      <template #action="{ row }">
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" />
      </template>
    </Grid>

    <Drawer v-model:open="detailVisible" :width="860" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="跟进记录详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <FollowupDetail v-if="detailId" :id="detailId" />
    </Drawer>
  </Page>
</template>