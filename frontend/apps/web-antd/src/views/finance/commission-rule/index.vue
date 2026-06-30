<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';

import { Button, Modal, Tag, message } from 'ant-design-vue';
import { LucideFilePenLine, Power, RefreshCw, Trash2 } from 'lucide-vue-next';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteCommissionRuleApi,
  getCommissionRuleListApi,
  toggleCommissionRuleApi,
} from '#/api/core/finance';
import { $t } from '#/locales';

import CommissionRuleDrawer from './drawer.vue';

const drawerVisible = ref(false);
const drawerData = ref<any>(null);

const triggerTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: '合同签订', color: 'blue' },
  2: { label: '回款到账', color: 'green' },
  3: { label: '订单完成', color: 'orange' },
  4: { label: '发票开具', color: 'purple' },
};

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
      label: '规则名称',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
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
        return await getCommissionRuleListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
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
      title: '规则名称',
      field: 'ruleName',
      minWidth: 160,
    },
    {
      title: '适用部门',
      field: 'deptName',
      minWidth: 120,
    },
    {
      title: '适用岗位',
      field: 'postName',
      minWidth: 120,
    },
    {
      title: '触发条件',
      field: 'triggerType',
      width: 120,
      slots: { default: 'triggerType' },
    },
    {
      title: '生效日期~失效日期',
      field: 'effectiveDate',
      minWidth: 220,
      slots: { default: 'dateRange' },
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
      width: 160,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function handleCreate() {
  drawerData.value = null;
  drawerVisible.value = true;
}

function handleEdit(row: any) {
  drawerData.value = row;
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

async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除提成规则「${row.ruleName}」吗？`,
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
    <Grid table-title="提成规则管理">
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" @click="handleCreate">
          新增提成规则
        </Button>
        <Button class="mr-2" :icon="h(RefreshCw)" @click="gridApi.query()">
          刷新
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #triggerType="{ row }">
        <Tag :color="triggerTypeMap[row.triggerType]?.color || 'default'">
          {{ triggerTypeMap[row.triggerType]?.label || row.triggerType }}
        </Tag>
      </template>

      <template #dateRange="{ row }">
        <span>{{ row.effectiveDate || '-' }}</span>
        <span class="mx-1 text-gray-400">~</span>
        <span>{{ row.expireDate || '长期' }}</span>
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
