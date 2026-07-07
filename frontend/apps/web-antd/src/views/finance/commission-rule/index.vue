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

const drawerVisible = ref(false);
const drawerData = ref<any>(null);

const triggerConditionMap: Record<number, { label: string; color: string }> = {
  1: { label: '合同签订', color: 'blue' },
  2: { label: '回款到账', color: 'green' },
  3: { label: '订单完成', color: 'orange' },
  4: { label: '发票开具', color: 'purple' },
};

const ruleTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: '个人业绩', color: 'blue' },
  2: { label: '团队分成', color: 'green' },
  3: { label: '部门经理', color: 'orange' },
  4: { label: '总监', color: 'purple' },
  5: { label: '团队长', color: 'cyan' },
};

const applyScopeMap: Record<number, string> = {
  1: '指定部门',
  2: '全公司',
  3: '指定岗位',
  4: '指定人员',
};

const ruleTypeOptions = [
  { value: 1, label: '个人业绩' },
  { value: 2, label: '团队分成' },
  { value: 3, label: '部门经理' },
  { value: 4, label: '总监' },
  { value: 5, label: '团队长' },
];

const statusOptions = [
  { value: 1, label: '启用' },
  { value: 0, label: '禁用' },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'ruleName',
      label: '方案名称',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'ruleType',
      label: '方案类型',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: ruleTypeOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'enabled',
      label: '状态',
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
      title: '方案名称',
      field: 'ruleName',
      minWidth: 160,
    },
    {
      title: '方案类型',
      field: 'ruleType',
      width: 120,
      slots: { default: 'ruleType' },
    },
    {
      title: '适用范围',
      field: 'applyScope',
      width: 120,
      slots: { default: 'applyScope' },
    },
    {
      title: '触发条件',
      field: 'triggerCondition',
      width: 120,
      slots: { default: 'triggerCondition' },
    },
    {
      title: '是否默认',
      field: 'isDefault',
      width: 100,
      slots: { default: 'isDefault' },
    },
    {
      title: '状态',
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
    message.success(row.enabled ? '已禁用' : '已启用');
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  } finally {
    row.pending = false;
  }
}

async function handleSetDefault(row: any) {
  try {
    await setCommissionDefaultApi(row.id);
    message.success('已设为默认方案');
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  }
}

async function handleCancelDefault(row: any) {
  Modal.confirm({
    title: '确认取消默认',
    content: `确定要取消「${row.ruleName}」的默认方案状态吗？`,
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await setCommissionDefaultApi(0);
        message.success('已取消默认方案');
        gridApi.query();
      } catch (e: any) {
        message.error(e?.message || '操作失败');
      }
    },
  });
}

async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除提成方案「${row.ruleName}」吗？`,
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        await deleteCommissionRuleApi(row.id);
        message.success('删除成功');
        gridApi.query();
      } catch (e: any) {
        message.error(e?.message || '删除失败');
      }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="提成方案管理">
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" @click="handleCreate">
          新增提成方案
        </Button>
        <Button class="mr-2" :icon="h(RefreshCw)" @click="gridApi.query()">
          刷新
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
          {{ ruleTypeMap[row.ruleType]?.label || '未知' }}
        </Tag>
      </template>

      <template #applyScope="{ row }">
        {{ applyScopeMap[row.applyScope] || row.applyScope || '-' }}
      </template>

      <template #isDefault="{ row }">
        <Tag v-if="row.isDefault" color="gold">默认</Tag>
        <span v-else>-</span>
      </template>

      <template #enabled="{ row }">
        <Tag :color="row.enabled ? 'green' : 'red'">
          {{ row.enabled ? '启用' : '禁用' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="link"
          :icon="h(LucideFilePenLine, { size: 14 })"
          title="编辑"
          @click="handleEdit(row)"
        />
        <Button
          v-if="!row.isDefault"
          type="link"
          :icon="h(Star, { size: 14 })"
          title="设为默认"
          @click="handleSetDefault(row)"
        />
        <Button
          v-else
          type="link"
          :icon="h(StarOff, { size: 14 })"
          title="取消默认"
          @click="handleCancelDefault(row)"
        />
        <Button
          type="link"
          :icon="h(Power, { size: 14 })"
          :loading="row.pending"
          :title="row.enabled ? '禁用' : '启用'"
          @click="handleToggle(row)"
        />
        <Button
          type="link"
          danger
          :icon="h(Trash2, { size: 14 })"
          title="删除"
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
