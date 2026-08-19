<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucidePlus } from '@vben/icons';

import { Button, message, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { categoryApi } from '#/api';

import CategoryDrawer from './drawer.vue';

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: '分类名称',
      componentProps: {
        placeholder: '请输入分类名称',
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
  pagerConfig: { enabled: false },
  cellConfig: {},
  rowConfig: { height: 48 },
  stripe: true,
  treeConfig: {
    transform: false,
    rowField: 'id',
    parentField: 'parentId',
    childrenField: 'children',
    accordion: false,
    expandAll: true,
  },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async () => {
        return await categoryApi.tree();
      },
    },
  },

  columns: [
    {
      title: '分类名称',
      field: 'categoryName',
      treeNode: true,
      minWidth: 240,
    },
    {
      title: '排序',
      field: 'sort',
      width: 80,
    },
    {
      title: '状态',
      field: 'isShow',
      slots: { default: 'status' },
      width: 80,
    },
    {
      title: '页面模式',
      field: 'pageType',
      slots: { default: 'pageType' },
      width: 100,
    },
    {
      title: '内容类型',
      field: 'contentType',
      slots: { default: 'contentType' },
      width: 100,
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
      width: 220,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: CategoryDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any, parentId?: any) {
  drawerApi.setData({
    create,
    row,
    parentId,
  });
  drawerApi.open();
}

function handleCreate() {
  openDrawer(true);
}

function handleCreateChild(row: any) {
  openDrawer(true, null, row.id);
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await categoryApi.delete({ id: row.id });
    message.success('删除成功');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function expandAll() {
  gridApi.grid?.setAllTreeExpand(true);
}

function collapseAll() {
  gridApi.grid?.setAllTreeExpand(false);
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="分类管理">
      <template #toolbar-tools>
        <Button
          class="mr-2"
          type="primary"
          :icon="h(LucidePlus)"
          @click="handleCreate"
        >
          新增分类
        </Button>
        <Button class="mr-2" @click="expandAll">展开全部</Button>
        <Button class="mr-2" @click="collapseAll">折叠全部</Button>
      </template>

      <template #status="{ row }">
        <Tag :color="row.isShow === 1 ? 'success' : 'default'">
          {{ row.isShow === 1 ? '显示' : '隐藏' }}
        </Tag>
      </template>

      <template #pageType="{ row }">
        <Tag :color="row.pageType === 1 ? 'blue' : 'cyan'">
          {{ row.pageType === 1 ? '封面模式' : '列表模式' }}
        </Tag>
      </template>

      <template #contentType="{ row }">
        <Tag :color="row.contentType === 1 ? 'green' : 'orange'">
          {{
            row.contentType === 1
              ? '文章'
              : row.contentType === 3
                ? '自定义链接'
                : ''
          }}
        </Tag>
      </template>

      <template #action="{ row }">
        <div class="flex items-center justify-center" style="gap: 12px">
          <a
            class="text-blue-600 cursor-pointer"
            @click="() => handleCreateChild(row)"
          >
            新增子项
          </a>
          <a
            class="text-blue-600 cursor-pointer"
            @click="() => handleEdit(row)"
          >
            编辑
          </a>
          <Popconfirm
            title="确定删除该分类吗？"
            ok-text="确定"
            cancel-text="取消"
            @confirm="() => handleDelete(row)"
          >
            <a class="text-red-500 cursor-pointer">删除</a>
          </Popconfirm>
        </div>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
