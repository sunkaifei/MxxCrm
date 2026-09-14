<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, nextTick, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { LucidePlus } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, message, Modal, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  copyTemplateDataApi,
  deleteTemplateDataApi,
  getTemplateDataListApi,
} from '#/api';

import PageEditor from './page-editor.vue';

const accessStore = useAccessStore();

// 嵌套的页面编辑器抽屉
const [PageEditorInstance, pageEditorApi] = useVbenDrawer({
  connectedComponent: PageEditor,
  onClosed() {
    // 编辑器关闭后刷新列表
    gridApi.query();
    drawerData.value.onRefreshTemplates?.();
  },
});

// 从父组件接收的数据
const drawerData = ref<{
  onRefreshTemplates?: () => void;
  templateId: number;
  templateName: string;
}>({ templateId: 0, templateName: '' });

// 最大化/还原
const isFullscreen = ref(false);
function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
  drawerApi.setState({ class: isFullscreen.value ? 'w-full' : 'w-[75%]' });
}

// 类型映射（T-P0.2：与后端渲染槽位一致——4=模板片段、6=栏目封面/自定义页；9=前台登录、12=购物车）
const typeOptions = [
  { value: 1, label: '首页', color: 'blue' },
  { value: 2, label: '列表页', color: 'cyan' },
  { value: 3, label: '内容页', color: 'green' },
  { value: 4, label: '模板片段', color: 'purple' },
  { value: 5, label: '报价页', color: 'orange' },
  { value: 6, label: '栏目封面/自定义页', color: 'red' },
  { value: 7, label: '产品列表', color: 'geekblue' },
  { value: 8, label: '产品详情', color: 'lime' },
  { value: 9, label: '前台登录页', color: 'magenta' },
  { value: 12, label: '购物车页', color: 'gold' },
  { value: 14, label: '页头', color: 'gold' },
  { value: 15, label: '页脚', color: 'volcano' },
];

function getTypeLabel(typeId: number): string {
  return (
    typeOptions.find((o) => o.value === typeId)?.label || `类型${typeId}`
  );
}

function getTypeColor(typeId: number): string {
  return typeOptions.find((o) => o.value === typeId)?.color || 'default';
}

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
        placeholder: '搜索页面名称',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'typeId',
      label: '页面类型',
      componentProps: {
        placeholder: '请选择页面类型',
        allowClear: true,
        options: typeOptions.map((o) => ({ value: o.value, label: o.label })),
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: '请选择状态',
        allowClear: true,
        options: [
          { value: 1, label: '启用' },
          { value: 0, label: '禁用' },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: false,
  },
  height: 'auto',
  pagerConfig: {},
  stripe: true,
  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }, formValues) => {
        // 抽屉数据未就绪（onOpenChange 之前）不请求
        if (!drawerData.value.templateId) {
          return { items: [], total: 0 };
        }
        return await getTemplateDataListApi({
          templateId: drawerData.value.templateId,
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords || undefined,
          typeId: formValues.typeId || undefined,
          status: formValues.status || undefined,
        });
      },
    },
  },
  columns: [
    { title: '序号', type: 'seq', width: 60 },
    {
      title: '页面名称',
      field: 'name',
      minWidth: 220,
      align: 'left',
    },
    {
      title: '页面类型',
      field: 'typeId',
      width: 150,
      slots: { default: 'typeDefault' },
    },
    {
      title: '状态',
      field: 'status',
      width: 90,
      slots: { default: 'statusDefault' },
    },
    { title: '排序', field: 'sort', width: 70 },
    { title: '创建时间', field: 'createTime', width: 165 },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      width: 150,
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 复制
async function handleCopy(row: any) {
  try {
    await copyTemplateDataApi(Number(row.id));
    message.success('复制成功，副本已创建（默认禁用）');
    gridApi.query();
    drawerData.value.onRefreshTemplates?.();
  } catch {
    // 全局拦截器处理
  }
}

// 删除
function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除页面「${row.name}」吗？`,
    okType: 'danger',
    onOk: async () => {
      await deleteTemplateDataApi([row.id]);
      message.success('删除成功');
      gridApi.query();
      drawerData.value.onRefreshTemplates?.();
    },
  });
}

// 编辑
function handleEdit(row: any) {
  pageEditorApi.setData({ templateId: drawerData.value.templateId, row });
  pageEditorApi.open();
}

// 新增
function handleCreate() {
  pageEditorApi.setData({ templateId: drawerData.value.templateId, row: null });
  pageEditorApi.open();
}

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[75%]',
  onCancel() {
    drawerApi.close();
  },
  onOpenChange(isOpen) {
    if (isOpen) {
      drawerData.value = drawerApi.getData<any>() || {
        templateId: 0,
        templateName: '',
      };
      isFullscreen.value = false;
      drawerApi.setState({ class: 'w-[75%]' });
      // 等 Grid 挂载完成后再查询，否则首次打开时 query 会在挂载前丢失
      nextTick(() => gridApi.query());
    }
  },
});
</script>

<template>
  <Drawer>
    <template #title>
      <div class="flex w-full items-center justify-between pr-2">
        <span>页面管理 - {{ drawerData.templateName }}</span>
        <Button type="link" size="small" @click="toggleFullscreen">
          {{ isFullscreen ? '还原' : '最大化' }}
        </Button>
      </div>
    </template>

    <div class="h-full">
      <Grid table-title="页面列表">
        <template #toolbar-tools>
          <Button
            v-if="accessStore.hasAccessCode('template:data:add')"
            type="primary"
            :icon="h(LucidePlus)"
            @click="handleCreate"
          >
            新增页面
          </Button>
        </template>

        <template #typeDefault="{ row }">
          <Tag :color="getTypeColor(row.typeId)">
            {{ getTypeLabel(row.typeId) }}
          </Tag>
        </template>

        <template #statusDefault="{ row }">
          <Tag v-if="row.status === 1" color="success">启用</Tag>
          <Tag v-else color="error">禁用</Tag>
        </template>

        <template #action="{ row }">
          <Button
            v-if="accessStore.hasAccessCode('template:data:update')"
            type="link"
            size="small"
            @click="() => handleEdit(row)"
          >
            编辑
          </Button>
          <Button
            v-if="accessStore.hasAccessCode('template:data:add')"
            type="link"
            size="small"
            @click="() => handleCopy(row)"
          >
            复制
          </Button>
          <Popconfirm
            v-if="accessStore.hasAccessCode('template:data:delete')"
            title="确定删除该页面吗？"
            ok-text="确定"
            cancel-text="取消"
            @confirm="handleDelete(row)"
          >
            <Button type="link" danger size="small">删除</Button>
          </Popconfirm>
        </template>
      </Grid>
    </div>

    <!-- 嵌套的页面编辑器抽屉 -->
    <PageEditorInstance />
  </Drawer>
</template>

<style scoped>
/* 标题行右侧最大化按钮与标题文字同一水平线 */
:deep(.ant-drawer-header) {
  align-items: center;
}
</style>
