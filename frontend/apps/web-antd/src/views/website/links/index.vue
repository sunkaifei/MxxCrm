<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus } from '@vben/icons';

import { Button, message, Modal, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { linksApi } from '#/api';

import LinksDrawer from './drawer.vue';

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '显示', value: 1 },
          { label: '隐藏', value: 0 },
        ],
        placeholder: '请选择状态',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'keyword',
      label: '关键词',
      componentProps: {
        placeholder: '请输入链接名称',
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
        return await linksApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status || undefined,
          keyword: formValues.keyword,
        });
      },
      delete: async ({ body }) => {
        await linksApi.delete(body.removeRecords);
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
      title: '链接名称',
      field: 'linkName',
      width: 280,
    },
    {
      title: '链接地址',
      field: 'linkUrl',
      width: 180,
    },
    {
      title: '链接类型',
      field: 'linkType',
      slots: { default: 'linkType' },
      width: 100,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 80,
    },
    {
      title: '排序',
      field: 'sort',
      width: 80,
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
  connectedComponent: LinksDrawer,
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
    content: `确定要删除链接"${row.linkName}"吗？`,
    okType: 'danger',
    onOk: async () => {
      await linksApi.delete([row.id]);
      message.success('删除成功');
      gridApi.query();
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="友情链接管理">
      <template #toolbar-tools>
        <Button type="primary" :icon="h(LucidePlus)" @click="handleAdd">
          新增链接
        </Button>
      </template>

      <template #linkType="{ row }">
        <Tag v-if="row.linkType === 0" color="blue">文字链接</Tag>
        <Tag v-else color="green">Logo链接</Tag>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 1" color="success">显示</Tag>
        <Tag v-else color="error">隐藏</Tag>
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
