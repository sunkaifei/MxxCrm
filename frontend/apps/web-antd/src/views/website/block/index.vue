<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus } from '@vben/icons';

import { Button, message, Modal, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteBlockApi, getBlockListApi } from '#/api';

import BlockDrawer from './drawer.vue';

const blockTypeMap: Record<number, { color: string; label: string }> = {
  1: { label: '文本', color: 'blue' },
  2: { label: 'HTML', color: 'purple' },
  3: { label: '图片', color: 'green' },
  4: { label: '链接', color: 'orange' },
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
        placeholder: '请输入区块编码或名称',
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
        return await getBlockListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords || undefined,
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
      title: '区块编码',
      field: 'blockCode',
      width: 180,
    },
    {
      title: '区块名称',
      field: 'blockName',
      width: 200,
    },
    {
      title: '类型',
      field: 'blockType',
      width: 100,
      slots: { default: 'blockType' },
    },
    {
      title: '内容预览',
      field: 'content',
      slots: { default: 'content' },
      minWidth: 240,
    },
    {
      title: '排序',
      field: 'sort',
      width: 80,
    },
    {
      title: '状态',
      field: 'status',
      width: 90,
      slots: { default: 'status' },
    },
    {
      title: '创建时间',
      field: 'createTime',
      width: 160,
    },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 160,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: BlockDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({
    create,
    row,
  });
  drawerApi.open();
}

function handleAdd() {
  openDrawer(true);
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除区块"${row.blockName}"吗？`,
    okType: 'danger',
    onOk: async () => {
      await deleteBlockApi([row.id]);
      message.success('删除成功');
      gridApi.query();
    },
  });
}

function truncate(text: string, len = 60): string {
  if (!text) return '-';
  const str = String(text);
  return str.length > len ? `${str.slice(0, len)}…` : str;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="区块管理">
      <template #toolbar-tools>
        <Button type="primary" :icon="h(LucidePlus)" @click="handleAdd">
          新增区块
        </Button>
      </template>

      <template #blockType="{ row }">
        <Tag :color="blockTypeMap[row.blockType]?.color || 'default'">
          {{ blockTypeMap[row.blockType]?.label || '未知' }}
        </Tag>
      </template>

      <template #content="{ row }">
        <span class="text-xs text-gray-600" :title="row.content">
          {{ truncate(row.content) }}
        </span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 1" color="success">启用</Tag>
        <Tag v-else color="default">禁用</Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        >
          修改
        </Button>
        <Button type="primary" link danger @click="() => handleDelete(row)">
          删除
        </Button>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
