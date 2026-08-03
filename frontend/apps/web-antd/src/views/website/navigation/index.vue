<script lang="ts" setup>
import { h } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus } from '@vben/icons';
import { Button, Tag, Modal, message } from 'ant-design-vue';
import NavigationDrawer from './drawer.vue';
import { navigationApi } from '#/api';

defineOptions({ name: 'WebsiteNavigation' });

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'navType',
      label: '导航类型',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '顶部导航', value: 'header' },
          { label: '底部导航', value: 'footer' },
        ],
        placeholder: '请选择导航类型',
        allowClear: true,
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
  pagerConfig: { enabled: false },
  cellConfig: {},
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async (_page, formValues) => {
        const res: any = await navigationApi.list({
          navType: formValues.navType || undefined,
        });
        return { items: res || [], total: (res || []).length };
      },
      delete: async ({ body }) => {
        await navigationApi.delete(body.removeRecords);
      },
    },
  },
  columns: [
    { title: '序号', type: 'seq', width: 70 },
    { title: '导航名称', field: 'name', width: 180 },
    { title: '链接地址', field: 'webUrl', width: 220 },
    {
      title: '导航类型',
      field: 'navType',
      width: 100,
      slots: { default: 'navType' },
    },
    {
      title: '数据类型',
      field: 'dataType',
      width: 120,
      slots: { default: 'dataType' },
    },
    { title: '排序', field: 'sort', width: 80 },
    {
      title: '显示',
      field: 'isShow',
      width: 80,
      slots: { default: 'isShow' },
    },
    {
      title: '新窗口',
      field: 'isNewWindowOpen',
      width: 80,
      slots: { default: 'isNewWindowOpen' },
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
  connectedComponent: NavigationDrawer,
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
    content: `确定要删除导航"${row.name}"吗？`,
    okType: 'danger',
    onOk: async () => {
      await navigationApi.delete([row.id]);
      message.success('删除成功');
      gridApi.query();
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="导航管理">
      <template #toolbar-tools>
        <Button
          type="primary"
          :icon="h(LucidePlus)"
          @click="handleAdd"
        >
          新增导航
        </Button>
      </template>

      <template #navType="{ row }">
        <Tag v-if="row.navType === 'header'" color="blue">顶部导航</Tag>
        <Tag v-else-if="row.navType === 'footer'" color="green">底部导航</Tag>
        <Tag v-else color="default">{{ row.navType || '—' }}</Tag>
      </template>

      <template #dataType="{ row }">
        <Tag v-if="row.dataType === 'custom'" color="default">自定义</Tag>
        <Tag v-else-if="row.dataType === 'article_class'" color="cyan">文章分类</Tag>
        <Tag v-else-if="row.dataType === 'customview'" color="purple">自定义页面</Tag>
        <Tag v-else color="default">{{ row.dataType || '—' }}</Tag>
      </template>

      <template #isShow="{ row }">
        <Tag v-if="row.isShow === 1" color="success">显示</Tag>
        <Tag v-else color="error">隐藏</Tag>
      </template>

      <template #isNewWindowOpen="{ row }">
        <Tag v-if="row.isNewWindowOpen === 1" color="blue">是</Tag>
        <Tag v-else color="default">否</Tag>
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
