<script lang="ts" setup>
import { h, onMounted, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus } from '@vben/icons';
import { Button, Modal, Tag, message } from 'ant-design-vue';
import ArticleFieldDrawer from './drawer.vue';
import {
  categoryApi,
  deleteArticleFieldApi,
  getArticleFieldListApi,
} from '#/api';

const categoryTree = ref<any[]>([]);

// 字段类型映射：1=文本 2=富文本 3=图片 4=数字 5=日期 6=下拉 7=多选
const fieldTypeMap: Record<number, string> = {
  1: '文本',
  2: '富文本',
  3: '图片',
  4: '数字',
  5: '日期',
  6: '下拉',
  7: '多选',
};

const fieldTypeColorMap: Record<number, string> = {
  1: 'blue',
  2: 'purple',
  3: 'cyan',
  4: 'green',
  5: 'orange',
  6: 'gold',
  7: 'magenta',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'TreeSelect',
      fieldName: 'categoryId',
      label: '所属栏目',
      componentProps: {
        treeData: categoryTree,
        placeholder: '请选择栏目',
        allowClear: true,
        treeDefaultExpandAll: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'fieldName',
      label: '字段名',
      componentProps: {
        placeholder: '请输入字段名',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '启用', value: 1 },
          { label: '停用', value: 0 },
        ],
        placeholder: '请选择状态',
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
        return await getArticleFieldListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          categoryId: formValues.categoryId || undefined,
          fieldName: formValues.fieldName || undefined,
          status: formValues.status || undefined,
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
      title: '字段名',
      field: 'fieldName',
      width: 160,
    },
    {
      title: '字段标签',
      field: 'fieldLabel',
      width: 160,
    },
    {
      title: '字段类型',
      field: 'fieldType',
      width: 100,
      slots: { default: 'fieldType' },
    },
    {
      title: '是否必填',
      field: 'isRequired',
      width: 90,
      slots: { default: 'isRequired' },
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
      width: 170,
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
  connectedComponent: ArticleFieldDrawer,
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
    content: `确定要删除字段"${row.fieldName}"吗？`,
    okType: 'danger',
    onOk: async () => {
      await deleteArticleFieldApi([row.id]);
      message.success('删除成功');
      gridApi.query();
    },
  });
}

async function loadCategoryTree() {
  const result = await categoryApi.tree();
  const mapTree = (nodes: any[]): any[] =>
    nodes.map((node) => ({
      title: node.name,
      value: node.id,
      key: node.id,
      children: node.children ? mapTree(node.children) : undefined,
    }));
  categoryTree.value = mapTree(result);
}

onMounted(() => {
  loadCategoryTree();
});
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="文章自定义字段管理">
      <template #toolbar-tools>
        <Button
          type="primary"
          :icon="h(LucidePlus)"
          @click="handleAdd"
        >
          新增字段
        </Button>
      </template>

      <template #fieldType="{ row }">
        <Tag :color="fieldTypeColorMap[row.fieldType] || 'default'">
          {{ fieldTypeMap[row.fieldType] || '未知' }}
        </Tag>
      </template>

      <template #isRequired="{ row }">
        <Tag v-if="row.isRequired === 1" color="red">必填</Tag>
        <Tag v-else color="default">否</Tag>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 1" color="success">启用</Tag>
        <Tag v-else color="default">停用</Tag>
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
