<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideFilePenLine } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import { Button, Tag, message } from 'ant-design-vue';
import { Power, RefreshCw } from 'lucide-vue-next';
import { useRoute, useRouter } from 'vue-router';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getApprovalFlowListApi, toggleApprovalFlowApi } from '#/api';
import { $t } from '#/locales';

const route = useRoute();
const router = useRouter();

const businessTypeMap: Record<string, { label: string; color: string }> = {
  contract: { label: '合同', color: 'blue' },
  order: { label: '订单', color: 'cyan' },
  purchase: { label: '采购', color: 'purple' },
  payment: { label: '付款', color: 'gold' },
  expense: { label: '报销', color: 'magenta' },
  leave: { label: '请假', color: 'orange' },
};

const businessTypeOptions = Object.entries(businessTypeMap).map(
  ([value, item]) => ({ value, label: item.label }),
);

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'flowName',
      label: '流程名称',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'businessType',
      label: '业务类型',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: businessTypeOptions,
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
        return await getApprovalFlowListApi({
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
      title: '流程名称',
      field: 'flowName',
      minWidth: 160,
    },
    {
      title: '流程编码',
      field: 'flowCode',
      minWidth: 140,
    },
    {
      title: '业务类型',
      field: 'businessType',
      width: 120,
      slots: { default: 'businessType' },
    },
    {
      title: '描述',
      field: 'description',
      minWidth: 200,
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
      width: 180,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function goDesigner(id?: number) {
  const query: Record<string, any> = {};
  if (id) {
    query.id = id;
  }
  void router.push({ path: '/system/approval/designer', query });
}

async function handleToggle(row: any) {
  row.pending = true;
  try {
    await toggleApprovalFlowApi(row.id);
    message.success(row.enabled ? '已禁用' : '已启用');
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  } finally {
    row.pending = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="审批流管理">
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" @click="goDesigner()">
          新增审批流
        </Button>
        <Button class="mr-2" :icon="h(RefreshCw)" @click="gridApi.query()">
          刷新
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #businessType="{ row }">
        <Tag :color="businessTypeMap[row.businessType]?.color || 'default'">
          {{ businessTypeMap[row.businessType]?.label || row.businessType }}
        </Tag>
      </template>

      <template #enabled="{ row }">
        <Tag :color="row.enabled ? 'green' : 'red'">
          {{ row.enabled ? '启用' : '禁用' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="goDesigner(row.id)"
        >
          设计
        </Button>
        <Button
          type="link"
          :icon="h(Power)"
          :loading="row.pending"
          @click="handleToggle(row)"
        >
          {{ row.enabled ? '禁用' : '启用' }}
        </Button>
      </template>
    </Grid>
  </Page>
</template>
