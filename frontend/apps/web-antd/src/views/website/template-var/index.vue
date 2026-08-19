<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import {
  LucideDownload,
  LucideFilePenLine,
  LucidePlus,
  LucideTrash2,
} from '@vben/icons';

import { Button, message, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteTemplateVarApi,
  getAllTemplateVarsApi,
  getTemplateVarListApi,
} from '#/api';

import TemplateVarDrawer from './drawer.vue';

const varTypeMap: Record<number, { color: string; label: string }> = {
  1: { label: '文本', color: 'blue' },
  2: { label: '数字', color: 'cyan' },
  3: { label: '布尔', color: 'orange' },
  4: { label: 'HTML', color: 'purple' },
  5: { label: '图片', color: 'green' },
};

const varGroupOptions = [
  { label: '默认', value: 'default' },
  { label: '联系信息', value: 'contact' },
  { label: '统计代码', value: 'stats' },
  { label: '品牌', value: 'brand' },
  { label: 'SEO', value: 'seo' },
  { label: '自定义', value: 'custom' },
];

const varGroupMap: Record<string, string> = {
  default: '默认',
  contact: '联系信息',
  stats: '统计代码',
  brand: '品牌',
  seo: 'SEO',
  custom: '自定义',
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
      componentProps: {
        placeholder: '变量KEY或标签',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'varGroup',
      label: '变量分组',
      componentProps: {
        options: varGroupOptions,
        placeholder: '请选择分组',
        allowClear: true,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getTemplateVarListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
          varGroup: formValues.varGroup || undefined,
        });
      },
    },
  },

  columns: [
    {
      title: '序号',
      type: 'seq',
      width: 70,
    },
    {
      title: '变量KEY',
      field: 'varKey',
      width: 180,
    },
    {
      title: '变量标签',
      field: 'varLabel',
      width: 140,
    },
    {
      title: '变量值',
      field: 'varValue',
      minWidth: 200,
      slots: { default: 'varValue' },
    },
    {
      title: '类型',
      field: 'varType',
      slots: { default: 'varType' },
      width: 90,
    },
    {
      title: '分组',
      field: 'varGroup',
      slots: { default: 'varGroup' },
      width: 110,
    },
    {
      title: '排序',
      field: 'sort',
      width: 80,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 80,
    },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 180,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: TemplateVarDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function handleAdd() {
  drawerApi.setData({ create: true });
  drawerApi.open();
}

function handleEdit(row: any) {
  drawerApi.setData({ create: false, row });
  drawerApi.open();
}

async function handleDelete(row: any) {
  await deleteTemplateVarApi([row.id]);
  message.success('删除成功');
  gridApi.query();
}

async function handleExportAll() {
  try {
    const res: any = await getAllTemplateVarsApi();
    const list = res?.rows || res?.list || res?.data?.rows || res || [];
    const blob = new Blob([JSON.stringify(list, null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `template-vars-${Date.now()}.json`;
    link.click();
    URL.revokeObjectURL(url);
    message.success('导出成功');
  } catch {
    message.error('导出失败');
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="模板变量管理">
      <template #toolbar-tools>
        <Button type="primary" :icon="h(LucidePlus)" @click="handleAdd">
          新增变量
        </Button>
        <Button :icon="h(LucideDownload)" class="ml-2" @click="handleExportAll">
          查看全部
        </Button>
      </template>

      <template #varValue="{ row }">
        <span
          :title="row.varValue"
          class="inline-block truncate"
          style="max-width: 260px"
        >
          {{ row.varValue || '—' }}
        </span>
      </template>

      <template #varType="{ row }">
        <Tag :color="varTypeMap[row.varType]?.color || 'default'">
          {{ varTypeMap[row.varType]?.label || '未知' }}
        </Tag>
      </template>

      <template #varGroup="{ row }">
        <Tag>{{ varGroupMap[row.varGroup] || row.varGroup }}</Tag>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 1" color="success">启用</Tag>
        <Tag v-else color="error">禁用</Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        >
          编辑
        </Button>
        <Popconfirm title="确定要删除该变量吗？" @confirm="handleDelete(row)">
          <Button type="primary" link danger :icon="h(LucideTrash2)">
            删除
          </Button>
        </Popconfirm>
      </template>
    </Grid>

    <Drawer />
  </Page>
</template>
