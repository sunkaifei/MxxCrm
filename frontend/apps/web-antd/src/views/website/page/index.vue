<script lang="ts" setup>
import { h } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus } from '@vben/icons';
import { Button, Tag, Modal, message } from 'ant-design-vue';
import PageDrawer from './drawer.vue';
import { deletePageApi, getPageListApi } from '#/api';

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
        placeholder: '请输入页面编码或名称',
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
        return await getPageListApi({
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
      title: '页面编码',
      field: 'pageCode',
      width: 180,
    },
    {
      title: '页面名称',
      field: 'pageName',
      width: 200,
    },
    {
      title: '页面标题',
      field: 'pageTitle',
      width: 220,
    },
    {
      title: 'SEO关键词',
      field: 'seoKeywords',
      slots: { default: 'seoKeywords' },
      minWidth: 200,
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
  connectedComponent: PageDrawer,
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
    content: `确定要删除页面"${row.pageName}"吗？`,
    okType: 'danger',
    onOk: async () => {
      await deletePageApi([row.id]);
      message.success('删除成功');
      gridApi.query();
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="自定义页面管理">
      <template #toolbar-tools>
        <Button
          type="primary"
          :icon="h(LucidePlus)"
          @click="handleAdd"
        >
          新增页面
        </Button>
      </template>

      <template #seoKeywords="{ row }">
        <span class="text-xs text-gray-600">{{ row.seoKeywords || '-' }}</span>
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
