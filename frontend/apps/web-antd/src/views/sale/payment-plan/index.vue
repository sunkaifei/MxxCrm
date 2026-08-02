<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore, useUserStore } from '@vben/stores';

import { Tabs, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getPaymentPlanPageListApi } from '#/api';
import { $t } from '#/locales';
import { PageUsageGuide } from '#/components/PageUsageGuide';

// 回款计划使用说明步骤数（与 i18n 中 page.sale.paymentPlan.guide.steps 数组对齐）
const guideStepCount = 5;

const accessStore = useAccessStore();
const userStore = useUserStore();

const canViewAll = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 1;
});

const canViewSubordinate = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 2 || dataScope === 3 || dataScope === 4;
});

const allTabList = [
  { key: 'all', label: '全部回款计划' },
  { key: 'my', label: '我的回款计划' },
  { key: 'subordinate', label: '下属回款计划' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter(t => keys.includes(t.key));
});

const activeTab = ref('my');

function handleTabChange(key: string) {
  activeTab.value = key;
  gridApi.query();
}

const statusOptions = [
  { label: '未开始', value: 0 },
  { label: '部分回款', value: 1 },
  { label: '已完成', value: 2 },
  { label: '已逾期', value: 3 },
];

const paymentTypeOptions = [
  { label: '预付款', value: 1 },
  { label: '进度款', value: 2 },
  { label: '到货款', value: 3 },
  { label: '验收款', value: 4 },
  { label: '质保金', value: 5 },
  { label: '尾款', value: 6 },
];

const statusColorMap: Record<number, string> = {
  0: 'default',
  1: 'orange',
  2: 'green',
  3: 'red',
};

const statusLabelMap: Record<number, string> = {
  0: '未开始',
  1: '部分回款',
  2: '已完成',
  3: '已逾期',
};

const paymentTypeColorMap: Record<number, string> = {
  1: 'blue',
  2: 'cyan',
  3: 'orange',
  4: 'purple',
  5: 'geekblue',
  6: 'magenta',
};

const paymentTypeLabelMap: Record<number, string> = {
  1: '预付款',
  2: '进度款',
  3: '到货款',
  4: '验收款',
  5: '质保金',
  6: '尾款',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: { placeholder: '合同编号/期次名称', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: { placeholder: '全部', allowClear: true, options: statusOptions },
    },
    {
      component: 'Input',
      fieldName: 'contractId',
      label: '合同ID',
      componentProps: { placeholder: '合同ID', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'customerId',
      label: '客户ID',
      componentProps: { placeholder: '客户ID', allowClear: true },
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
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
        };
        if (formValues.keywords) params.keywords = formValues.keywords;
        if (formValues.status) params.status = formValues.status;
        if (formValues.contractId) params.contractId = formValues.contractId;
        if (formValues.customerId) params.customerId = formValues.customerId;
        return await getPaymentPlanPageListApi(params);
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '合同编号', field: 'contractNo', width: 160 },
    { title: '客户名称', field: 'customerName', minWidth: 140 },
    { title: '期次名称', field: 'stageName', width: 140 },
    { title: '款项类型', field: 'paymentType', width: 110, slots: { default: 'paymentType' } },
    { title: '计划金额', field: 'planAmount', width: 130, slots: { default: 'planAmount' } },
    { title: '已收金额', field: 'receivedAmount', width: 130, slots: { default: 'receivedAmount' } },
    { title: '未收金额', field: 'unreceivedAmount', width: 130, slots: { default: 'unreceivedAmount' } },
    { title: '计划日期', field: 'planDate', width: 120 },
    { title: '实际日期', field: 'actualDate', width: 120 },
    { title: '状态', field: 'status', width: 100, slots: { default: 'status' } },
    { title: '备注', field: 'remark', minWidth: 150 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.sale.paymentPlan.guide.title')"
      :brief="$t('page.sale.paymentPlan.guide.brief')"
      :expand-text="$t('page.sale.paymentPlan.guide.expand')"
      :collapse-text="$t('page.sale.paymentPlan.guide.collapse')"
    >
      <div
        v-for="i in guideStepCount"
        :key="i"
        class="page-guide-step-item"
      >
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.sale.paymentPlan.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.sale.paymentPlan.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Grid table-title="回款计划列表">
      <template #form-header>
        <Tabs v-model:activeKey="activeTab" class="mb-3" @change="handleTabChange">
          <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        </Tabs>
      </template>

      <template #paymentType="{ row }">
        <Tag v-if="row.paymentType != null" :color="paymentTypeColorMap[row.paymentType]">
          {{ paymentTypeLabelMap[row.paymentType] || row.paymentType }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #planAmount="{ row }">
        <span class="text-right block">
          ¥{{ Number(row.planAmount || 0).toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
        </span>
      </template>

      <template #receivedAmount="{ row }">
        <span class="text-right block">
          ¥{{ Number(row.receivedAmount || 0).toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
        </span>
      </template>

      <template #unreceivedAmount="{ row }">
        <span
          class="text-right block"
          :class="(Number(row.planAmount || 0) - Number(row.receivedAmount || 0)) > 0 ? 'text-orange-600 font-medium' : 'text-gray-400'"
        >
          ¥{{ (Number(row.planAmount || 0) - Number(row.receivedAmount || 0)).toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
        </span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status != null" :color="statusColorMap[row.status]">
          {{ statusLabelMap[row.status] || row.status }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>
    </Grid>
  </Page>
</template>
