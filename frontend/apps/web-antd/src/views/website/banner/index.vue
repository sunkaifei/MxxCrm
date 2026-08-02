<script lang="ts" setup>
import { h } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus } from '@vben/icons';
import { Button, Tag, Image, Modal, message } from 'ant-design-vue';
import BannerDrawer from './drawer.vue';
import { deleteBannerApi, getBannerListApi } from '#/api';

const positionOptions = [
  { label: '首页顶部', value: 'home_top' },
  { label: '首页中部', value: 'home_middle' },
  { label: '首页底部', value: 'home_bottom' },
  { label: '侧栏顶部', value: 'sidebar_top' },
  { label: '侧栏底部', value: 'sidebar_bottom' },
  { label: '分类顶部', value: 'category_top' },
];

const positionMap: Record<string, string> = {
  home_top: '首页顶部',
  home_middle: '首页中部',
  home_bottom: '首页底部',
  sidebar_top: '侧栏顶部',
  sidebar_bottom: '侧栏底部',
  category_top: '分类顶部',
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
        placeholder: '请输入标题',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'position',
      label: '位置',
      componentProps: {
        options: positionOptions,
        placeholder: '请选择位置',
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
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getBannerListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords || undefined,
          position: formValues.position || undefined,
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
      title: '标题',
      field: 'title',
      width: 200,
    },
    {
      title: '图片',
      field: 'imageUrl',
      width: 120,
      slots: { default: 'image' },
    },
    {
      title: '链接URL',
      field: 'linkUrl',
      width: 200,
    },
    {
      title: '位置',
      field: 'position',
      width: 120,
      slots: { default: 'position' },
    },
    {
      title: '排序',
      field: 'sort',
      width: 80,
    },
    {
      title: '时间范围',
      field: 'timeRange',
      width: 280,
      slots: { default: 'timeRange' },
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
  connectedComponent: BannerDrawer,
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
    content: `确定要删除Banner"${row.title}"吗？`,
    okType: 'danger',
    onOk: async () => {
      await deleteBannerApi([row.id]);
      message.success('删除成功');
      gridApi.query();
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="Banner管理">
      <template #toolbar-tools>
        <Button
          type="primary"
          :icon="h(LucidePlus)"
          @click="handleAdd"
        >
          新增Banner
        </Button>
      </template>

      <template #image="{ row }">
        <Image
          v-if="row.imageUrl"
          :src="row.imageUrl"
          :width="80"
          :height="50"
          fit="cover"
          class="rounded"
        />
        <span v-else class="text-gray-400">-</span>
      </template>

      <template #position="{ row }">
        <Tag color="blue">{{ positionMap[row.position] || row.position || '-' }}</Tag>
      </template>

      <template #timeRange="{ row }">
        <span class="text-xs text-gray-600">
          <template v-if="row.startTime || row.endTime">
            {{ row.startTime || '-' }} ~ {{ row.endTime || '-' }}
          </template>
          <template v-else>-</template>
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
