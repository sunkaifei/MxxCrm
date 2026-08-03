<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';

import { Button, Modal, Tag, Tooltip, message } from 'ant-design-vue';
import { RefreshCw } from 'lucide-vue-next';
import { useRouter } from 'vue-router';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteApprovalFlowApi,
  getApprovalFlowListApi,
  toggleApprovalFlowApi,
} from '#/api';
import { $t } from '#/locales';

const router = useRouter();

const businessTypeMap: Record<string, { label: string; color: string }> = {
  contract: { label: '合同', color: 'blue' },
  quotation: { label: '报价单', color: 'green' },
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
  cellConfig: {},
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
      title: '类型',
      field: 'isSystem',
      width: 100,
      slots: { default: 'isSystem' },
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
      width: 210,
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

async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除审批流「${row.flowName}」吗？`,
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        await deleteApprovalFlowApi(row.id);
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

      <template #isSystem="{ row }">
        <Tag v-if="row.isSystem === 1" color="blue">系统内置</Tag>
        <Tag v-else color="default">自定义</Tag>
      </template>

      <template #enabled="{ row }">
        <Tag :color="row.enabled ? 'green' : 'red'">
          {{ row.enabled ? '启用' : '禁用' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button type="link" @click="goDesigner(row.id)">设计</Button>
        <Button
          type="link"
          :loading="row.pending"
          @click="handleToggle(row)"
        >
          {{ row.enabled ? '禁用' : '启用' }}
        </Button>
        <Tooltip
          v-if="row.isSystem === 1"
          title="系统内置审批流不可删除，如需停用请使用禁用功能"
        >
          <Button type="link" danger disabled>删除</Button>
        </Tooltip>
        <Button
          v-else
          type="link"
          danger
          @click="handleDelete(row)"
        >
          删除
        </Button>
      </template>
    </Grid>
  </Page>
</template>