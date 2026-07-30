<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideEye } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import { Button, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getCcListApi, markCcReadApi } from '#/api';
import { $t } from '#/locales';

const businessTypeMap: Record<string, { label: string; color: string }> = {
  contract: { label: '合同', color: 'geekblue' },
  expense: { label: '报销', color: 'volcano' },
  invoice: { label: '发票', color: 'purple' },
  leave: { label: '请假', color: 'orange' },
  order: { label: '订单', color: 'cyan' },
  payment: { label: '回款', color: 'gold' },
  purchase: { label: '采购', color: 'magenta' },
  quotation: { label: '报价单', color: 'blue' },
  refund: { label: '退款', color: 'red' },
  visit: { label: '外勤', color: 'lime' },
};

const instanceStatusList: Record<number, { label: string; color: string }> = {
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
  5: { label: '已撤回', color: 'default' },
  6: { label: '待修改', color: 'orange' },
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'businessTitle',
      label: '业务标题',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'isRead',
      label: '阅读状态',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: [
          { label: '未读', value: 0 },
          { label: '已读', value: 1 },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  cellConfig: {
    isHover: true,
  },
  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      field: 'businessTitle',
      minWidth: 200,
      title: '业务标题',
    },
    {
      field: 'businessType',
      slots: { default: 'businessType' },
      title: '业务类型',
      width: 110,
    },
    {
      field: 'submitterName',
      title: '提交人',
      width: 120,
    },
    {
      field: 'ccFromName',
      title: '抄送人',
      width: 120,
    },
    {
      field: 'ccReason',
      minWidth: 180,
      title: '抄送说明',
    },
    {
      field: 'instanceStatus',
      slots: { default: 'instanceStatus' },
      title: '审批状态',
      width: 110,
    },
    {
      field: 'isRead',
      slots: { default: 'isRead' },
      title: '阅读状态',
      width: 100,
    },
    {
      field: 'createTime',
      slots: { default: 'createTime' },
      title: '抄送时间',
      width: 170,
    },
    {
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      title: $t('ui.table.action'),
      width: 120,
    },
  ],
  height: 'auto',
  pagerConfig: {},
  proxyConfig: {
    ajax: {
      query: async ({ page }, formValues) => {
        return await getCcListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
    autoLoad: true,
  },
  stripe: true,
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleMarkRead(row: any) {
  try {
    await markCcReadApi(row.id);
    message.success('已标记为已读');
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="抄送我的">
      <template #businessType="{ row }">
        <Tag :color="businessTypeMap[row.businessType]?.color || 'default'">
          {{ businessTypeMap[row.businessType]?.label || row.businessType }}
        </Tag>
      </template>

      <template #instanceStatus="{ row }">
        <Tag
          :color="instanceStatusList[row.instanceStatus]?.color || 'default'"
        >
          {{ instanceStatusList[row.instanceStatus]?.label || '未知' }}
        </Tag>
      </template>

      <template #isRead="{ row }">
        <Tag :color="row.isRead === 1 ? 'default' : 'error'">
          {{ row.isRead === 1 ? '已读' : '未读' }}
        </Tag>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="row.isRead !== 1"
          :icon="h(LucideEye)"
          size="small"
          type="link"
          @click="handleMarkRead(row)"
        >
          标记已读
        </Button>
        <span v-else class="text-gray-400 text-xs">已读</span>
      </template>
    </Grid>
  </Page>
</template>
