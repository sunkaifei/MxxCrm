<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';

import { Button, Modal, Tag, message } from 'ant-design-vue';
import { LucideFilePenLine, Power, RefreshCw, Star, StarOff, Trash2 } from 'lucide-vue-next';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteCommissionRuleApi,
  getCommissionRuleListApi,
  getCommissionRuleDetailApi,
  setCommissionDefaultApi,
  toggleCommissionRuleApi,
} from '#/api/core/finance';
import { $t } from '#/locales';

import CommissionRuleDrawer from './drawer.vue';
import { PageUsageGuide } from '#/components/PageUsageGuide';

const guideStepCount = 5;
const drawerVisible = ref(false);
const drawerData = ref<any>(null);

const triggerConditionMap: Record<number, { label: string; color: string }> = {
  1: { label: $t('page.finance.commissionRule.triggerCondition.contractSign'), color: 'blue' },
  2: { label: $t('page.finance.commissionRule.triggerCondition.paymentReceived'), color: 'green' },
  3: { label: $t('page.finance.commissionRule.triggerCondition.orderComplete'), color: 'orange' },
  4: { label: $t('page.finance.commissionRule.triggerCondition.invoiceIssued'), color: 'purple' },
};

const ruleTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: $t('page.finance.commissionRule.ruleType.personal'), color: 'blue' },
  2: { label: $t('page.finance.commissionRule.ruleType.team'), color: 'green' },
  3: { label: $t('page.finance.commissionRule.ruleType.manager'), color: 'orange' },
  4: { label: $t('page.finance.commissionRule.ruleType.director'), color: 'purple' },
  5: { label: $t('page.finance.commissionRule.ruleType.leader'), color: 'cyan' },
};

const applyScopeMap: Record<number, string> = {
  1: $t('page.finance.commissionRule.applyScope.department'),
  2: $t('page.finance.commissionRule.applyScope.company'),
  3: $t('page.finance.commissionRule.applyScope.post'),
  4: $t('page.finance.commissionRule.applyScope.employee'),
};

const ruleTypeOptions = [
  { value: 1, label: $t('page.finance.commissionRule.ruleType.personal') },
  { value: 2, label: $t('page.finance.commissionRule.ruleType.team') },
  { value: 3, label: $t('page.finance.commissionRule.ruleType.manager') },
  { value: 4, label: $t('page.finance.commissionRule.ruleType.director') },
  { value: 5, label: $t('page.finance.commissionRule.ruleType.leader') },
];

const statusOptions = [
  { value: 1, label: $t('page.finance.common.enabled') },
  { value: 0, label: $t('page.finance.common.disabled') },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'ruleName',
      label: $t('page.finance.commissionRule.column.ruleName'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'ruleType',
      label: $t('page.finance.commissionRule.column.ruleType'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: ruleTypeOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'enabled',
      label: $t('page.finance.common.status'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: statusOptions,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  pagerConfig: {},
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const res = await getCommissionRuleListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
        return res;
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: $t('page.finance.commissionRule.column.ruleName'),
      field: 'ruleName',
      minWidth: 160,
    },
    {
      title: $t('page.finance.commissionRule.column.ruleType'),
      field: 'ruleType',
      width: 120,
      slots: { default: 'ruleType' },
    },
    {
      title: $t('page.finance.commissionRule.column.applyScope'),
      field: 'applyScope',
      width: 120,
      slots: { default: 'applyScope' },
    },
    {
      title: $t('page.finance.commissionRule.column.triggerCondition'),
      field: 'triggerCondition',
      width: 120,
      slots: { default: 'triggerCondition' },
    },
    {
      title: $t('page.finance.commissionRule.column.isDefault'),
      field: 'isDefault',
      width: 100,
      slots: { default: 'isDefault' },
    },
    {
      title: $t('page.finance.common.status'),
      field: 'enabled',
      width: 100,
      slots: { default: 'enabled' },
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 170,
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 220,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function handleCreate() {
  drawerData.value = null;
  drawerVisible.value = true;
}

async function handleEdit(row: any) {
  try {
    const res = await getCommissionRuleDetailApi(row.id);
    drawerData.value = res.data || row;
  } catch (e) {
    drawerData.value = row;
  }
  drawerVisible.value = true;
}

function handleDrawerClose(needRefresh?: boolean) {
  drawerVisible.value = false;
  drawerData.value = null;
  if (needRefresh) {
    gridApi.query();
  }
}

async function handleToggle(row: any) {
  row.pending = true;
  try {
    await toggleCommissionRuleApi(row.id);
    message.success(row.enabled ? $t('page.finance.commissionRule.message.disabled') : $t('page.finance.commissionRule.message.enabled'));
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.failed'));
  } finally {
    row.pending = false;
  }
}

async function handleSetDefault(row: any) {
  try {
    await setCommissionDefaultApi(row.id);
    message.success($t('page.finance.commissionRule.message.setDefaultSuccess'));
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.failed'));
  }
}

async function handleCancelDefault(row: any) {
  Modal.confirm({
    title: $t('page.finance.commissionRule.message.cancelDefaultTitle'),
    content: $t('page.finance.commissionRule.message.cancelDefaultContent', { name: row.ruleName }),
    okText: $t('page.finance.common.confirm'),
    cancelText: $t('page.finance.common.cancel'),
    onOk: async () => {
      try {
        await setCommissionDefaultApi(0);
        message.success($t('page.finance.commissionRule.message.cancelDefaultSuccess'));
        gridApi.query();
      } catch (e: any) {
        message.error(e?.message || $t('page.finance.common.failed'));
      }
    },
  });
}

async function handleDelete(row: any) {
  Modal.confirm({
    title: $t('page.finance.commissionRule.message.deleteTitle'),
    content: $t('page.finance.commissionRule.message.deleteContent', { name: row.ruleName }),
    okText: $t('page.finance.common.delete'),
    okType: 'danger',
    cancelText: $t('page.finance.common.cancel'),
    onOk: async () => {
      try {
        await deleteCommissionRuleApi(row.id);
        message.success($t('page.finance.common.deleteSuccess'));
        gridApi.query();
      } catch (e: any) {
        message.error(e?.message || $t('page.finance.common.deleteFailed'));
      }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.finance.commissionRule.guide.title')"
      :brief="$t('page.finance.commissionRule.guide.brief')"
      :expand-text="$t('page.finance.commissionRule.guide.expand')"
      :collapse-text="$t('page.finance.commissionRule.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.commissionRule.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.commissionRule.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Grid :table-title="$t('page.finance.commissionRule.manageTitle')">
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" @click="handleCreate">
          {{ $t('page.finance.commissionRule.button.createPlan') }}
        </Button>
        <Button class="mr-2" :icon="h(RefreshCw)" @click="gridApi.query()">
          {{ $t('page.finance.common.refresh') }}
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #triggerCondition="{ row }">
        <Tag :color="triggerConditionMap[row.triggerCondition]?.color || 'default'">
          {{ triggerConditionMap[row.triggerCondition]?.label || row.triggerCondition }}
        </Tag>
      </template>

      <template #ruleType="{ row }">
        <Tag :color="ruleTypeMap[row.ruleType]?.color || 'default'">
          {{ ruleTypeMap[row.ruleType]?.label || $t('page.finance.commissionRule.message.unknown') }}
        </Tag>
      </template>

      <template #applyScope="{ row }">
        {{ applyScopeMap[row.applyScope] || row.applyScope || '-' }}
      </template>

      <template #isDefault="{ row }">
        <Tag v-if="row.isDefault" color="gold">{{ $t('page.finance.commissionRule.defaultTag') }}</Tag>
        <span v-else>-</span>
      </template>

      <template #enabled="{ row }">
        <Tag :color="row.enabled ? 'green' : 'red'">
          {{ row.enabled ? $t('page.finance.common.enabled') : $t('page.finance.common.disabled') }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="link"
          :icon="h(LucideFilePenLine, { size: 14 })"
          :title="$t('page.finance.common.edit')"
          @click="handleEdit(row)"
        />
        <Button
          v-if="!row.isDefault"
          type="link"
          :icon="h(Star, { size: 14 })"
          :title="$t('page.finance.commissionRule.button.setDefault')"
          @click="handleSetDefault(row)"
        />
        <Button
          v-else
          type="link"
          :icon="h(StarOff, { size: 14 })"
          :title="$t('page.finance.commissionRule.button.cancelDefault')"
          @click="handleCancelDefault(row)"
        />
        <Button
          type="link"
          :icon="h(Power, { size: 14 })"
          :loading="row.pending"
          :title="row.enabled ? $t('page.finance.common.disabled') : $t('page.finance.common.enabled')"
          @click="handleToggle(row)"
        />
        <Button
          type="link"
          danger
          :icon="h(Trash2, { size: 14 })"
          :title="$t('page.finance.common.delete')"
          @click="handleDelete(row)"
        />
      </template>
    </Grid>

    <CommissionRuleDrawer
      :visible="drawerVisible"
      :data="drawerData"
      @close="handleDrawerClose"
    />
  </Page>
</template>
