<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus } from '@vben/icons';

import { Button, message, Modal, Tabs, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { navigationApi } from '#/api';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';

import RecycleBin from '../../crm/components/RecycleBin.vue';
import NavigationDrawer from './drawer.vue';

defineOptions({ name: 'WebsiteNavigation' });

const { isSuperAdmin } = useSuperAdminGuard();
const activeTab = ref('list');

const navTypeLabels: Record<string, string> = {
  header: '顶部导航',
  footer: '底部导航',
  sidebar: '侧边栏',
  mobile: '移动端',
  topbar: '顶栏',
};

const dataTypeLabels: Record<string, string> = {
  custom: '自定义',
  article_class: '文章分类',
  product_class: '产品分类',
  customview: '自定义页面',
  article: '文章',
  product: '产品',
  link_group: '分组标题',
};

// 父级名称映射（列表展示「父级」列）
const navNameMap = ref<Record<number, string>>({});
async function loadNavNameMap() {
  try {
    const res: any = await navigationApi.list({ page: 1, pageSize: 200 });
    const map: Record<number, string> = {};
    (res?.items || []).forEach((n: any) => (map[n.id] = n.name));
    navNameMap.value = map;
  } catch {
    // 忽略
  }
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '导航名称',
      componentProps: { placeholder: '输入名称关键词', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'navType',
      label: '导航位置',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '顶部导航', value: 'header' },
          { label: '底部导航', value: 'footer' },
          { label: '侧边栏', value: 'sidebar' },
          { label: '移动端', value: 'mobile' },
          { label: '顶栏', value: 'topbar' },
        ],
        placeholder: '请选择导航位置',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'dataType',
      label: '数据来源',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '自定义导航', value: 'custom' },
          { label: '文章分类', value: 'article_class' },
          { label: '产品分类', value: 'product_class' },
          { label: '自定义页面', value: 'customview' },
          { label: '文章', value: 'article' },
          { label: '产品', value: 'product' },
          { label: '分组标题', value: 'link_group' },
        ],
        placeholder: '请选择数据来源',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'isShow',
      label: '显示状态',
      componentProps: {
        options: [
          { label: '全部', value: undefined },
          { label: '显示', value: 1 },
          { label: '隐藏', value: 0 },
        ],
        placeholder: '请选择显示状态',
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
  pagerConfig: { enabled: true, pageSize: 20 },
  editConfig: { trigger: 'click', mode: 'cell', beforeEditMethod: () => true },
  cellConfig: { isHover: true } as any,
  stripe: true,
  checkboxConfig: { highlight: true } as any,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const res: any = await navigationApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords || undefined,
          navType: formValues.navType || undefined,
          dataType: formValues.dataType || undefined,
          isShow: formValues.isShow ?? undefined,
        });
        return res;
      },
      delete: async ({ body }) => {
        await navigationApi.delete(
          (body.removeRecords || []).map((r: any) => r.id),
        );
      },
    },
  },
  columns: [
    { title: '勾选', type: 'checkbox', width: 50 },
    { title: '序号', type: 'seq', width: 60 },
    { title: '导航名称', field: 'name', minWidth: 160 },
    {
      title: '链接地址',
      field: 'webUrl',
      minWidth: 200,
      slots: { default: 'webUrl' },
    },
    {
      title: '位置',
      field: 'navType',
      width: 100,
      slots: { default: 'navType' },
    },
    {
      title: '来源',
      field: 'dataType',
      width: 110,
      slots: { default: 'dataType' },
    },
    {
      title: '父级',
      field: 'parentId',
      width: 120,
      slots: { default: 'parentId' },
    },
    {
      title: '图标',
      field: 'icon',
      width: 70,
      slots: { default: 'icon' },
    },
    {
      title: '排序',
      field: 'sort',
      width: 90,
      editRender: { name: 'InputNumber', props: { min: 0, controls: false } },
    },
    {
      title: '显示',
      field: 'isShow',
      width: 80,
      slots: { default: 'isShow' },
    },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 150,
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
      loadNavNameMap();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
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
    content: `确定要删除导航"${row.name}"吗？删除后进入回收站，可还原。`,
    okType: 'danger',
    onOk: async () => {
      await navigationApi.delete([row.id]);
      message.success('已移入回收站');
      gridApi.query();
      loadNavNameMap();
    },
  });
}

async function handleBatchDelete() {
  const rows = (gridApi.grid?.getCheckboxRecords?.() || []) as any[];
  if (rows.length === 0) {
    message.warning('请先勾选要删除的导航');
    return;
  }
  Modal.confirm({
    title: '批量删除',
    content: `确定删除选中的 ${rows.length} 条导航吗？删除后进入回收站。`,
    okType: 'danger',
    onOk: async () => {
      await navigationApi.delete(rows.map((r) => r.id));
      message.success('已移入回收站');
      gridApi.query();
      loadNavNameMap();
    },
  });
}

// 排序单元格编辑结束 → 落库
async function onEditClosed({ row, column }: any) {
  if (column?.field !== 'sort') return;
  try {
    await navigationApi.updateSort(row.id, Number(row.sort) || 0);
    message.success('排序已更新');
  } catch {
    // 全局拦截器处理
  }
}

function handleTabChange(key: any) {
  if (key === 'recycle') return;
  gridApi.query();
}

onMounted(() => {
  loadNavNameMap();
});
</script>

<template>
  <Page auto-content-height>
    <div class="p-2 bg-white dark:bg-black rounded mb-2">
      <Tabs v-model:activeKey="activeTab" @change="handleTabChange">
        <Tabs.TabPane key="list" tab="导航列表" />
        <Tabs.TabPane v-if="isSuperAdmin" key="recycle" tab="回收站" />
      </Tabs>
    </div>

    <RecycleBin v-show="activeTab === 'recycle'" module="navigation" />

    <Grid v-show="activeTab !== 'recycle'" table-title="导航管理" @edit-closed="onEditClosed">
      <template #toolbar-tools>
        <Button type="primary" :icon="h(LucidePlus)" @click="handleAdd">
          新增导航
        </Button>
        <Button danger class="ml-2" @click="handleBatchDelete">批量删除</Button>
      </template>

      <template #webUrl="{ row }">
        <a
          v-if="row.webUrl && row.webUrl !== '#'"
          :href="row.webUrl"
          target="_blank"
          class="text-blue-500"
        >{{ row.webUrl }}</a>
        <span v-else class="text-gray-400">{{ row.webUrl || '—' }}</span>
      </template>

      <template #navType="{ row }">
        <Tag color="blue">{{ navTypeLabels[row.navType] || row.navType || '—' }}</Tag>
      </template>

      <template #dataType="{ row }">
        <Tag color="cyan">{{ dataTypeLabels[row.dataType] || row.dataType || '—' }}</Tag>
      </template>

      <template #parentId="{ row }">
        <span v-if="row.parentId && row.parentId > 0">
          {{ navNameMap[row.parentId] || `#${row.parentId}` }}
        </span>
        <span v-else class="text-gray-400">顶级</span>
      </template>

      <template #icon="{ row }">
        <i v-if="row.icon" :class="row.icon" />
        <span v-else class="text-gray-300">—</span>
      </template>

      <template #isShow="{ row }">
        <Tag v-if="row.isShow === 1" color="success">显示</Tag>
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
