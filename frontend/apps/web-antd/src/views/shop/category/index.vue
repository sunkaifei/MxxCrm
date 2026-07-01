<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { Button, Popconfirm, Tag, Tree, message } from 'ant-design-vue';
import CategoryDrawer from './drawer.vue';
import { categoryApi } from '#/api';
import type { CategoryVO } from '#/api/core/shop/category';

const categoryTree = ref<CategoryVO[]>([]);
const selectedCategory = ref<CategoryVO | null>(null);

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
  pagerConfig: {},
  cellConfig: {
    isHover: true,
  },
  stripe: true,

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
      title: '序号',
      type: 'seq',
      width: 70,
    },
    {
      title: '分类名称',
      field: 'name',
    },
    {
      title: '层级',
      field: 'level',
      slots: { default: 'level' },
      width: 80,
    },
    {
      title: '排序',
      field: 'sortOrder',
      width: 80,
    },
    {
      title: '状态',
      field: 'isShow',
      slots: { default: 'status' },
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

async function loadTree() {
  const result = await categoryApi.tree();
  categoryTree.value = result;
}

loadTree();

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: CategoryDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
      loadTree();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({
    create,
    row,
    parentId: selectedCategory.value?.id ?? 0,
  });
  drawerApi.open();
}

function handleCreate() {
  openDrawer(true);
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
    loadTree();
  }
}

function renderTreeNodes(data: CategoryVO[]): any[] {
  return data.map((item) => {
    const children =
      item.children && item.children.length > 0
        ? renderTreeNodes(item.children)
        : undefined;
    return {
      title: item.name,
      key: item.id.toString(),
      children,
      dataRef: item,
    };
  });
}

function onTreeSelect(selectedKeys: (string | number)[]) {
  if (selectedKeys.length > 0) {
    const key = String(selectedKeys[0]);
    const findNode = (
      nodes: CategoryVO[],
      targetKey: string,
    ): CategoryVO | null => {
      for (const node of nodes) {
        if (String(node.id) === targetKey) return node;
        if (node.children) {
          const found = findNode(node.children, targetKey);
          if (found) return found;
        }
      }
      return null;
    };
    selectedCategory.value = findNode(categoryTree.value, key);
  } else {
    selectedCategory.value = null;
  }
}
</script>

<template>
  <Page auto-content-height>
    <div class="flex gap-4 h-full">
      <div
        class="w-56 bg-white rounded-lg p-4 border border-gray-200 overflow-hidden"
      >
        <div class="flex justify-between items-center mb-4">
          <h3 class="font-semibold text-gray-800">分类树</h3>
          <Button type="primary" size="small" icon="plus" @click="handleCreate">
            新增
          </Button>
        </div>
        <div class="overflow-y-auto max-h-[calc(100vh-200px)]">
          <Tree
            :tree-data="renderTreeNodes(categoryTree)"
            default-expand-all
            :selected-keys="
              selectedCategory ? [selectedCategory.id.toString()] : []
            "
            @select="onTreeSelect"
          />
        </div>
      </div>

      <div class="flex-1">
        <Grid table-title="分类列表">
          <template #toolbar-tools>
            <Button class="mr-2" type="primary" @click="handleCreate">
              新增分类
            </Button>
          </template>

          <template #level="{ row }">
            <Tag
              :color="
                row.level === 1 ? 'blue' : row.level === 2 ? 'green' : 'orange'
              "
            >
              {{ row.level === 1 ? '一级' : row.level === 2 ? '二级' : '三级' }}
            </Tag>
          </template>

          <template #status="{ row }">
            <Tag :color="row.isShow === 1 ? 'success' : 'default'">
              {{ row.isShow === 1 ? '显示' : '隐藏' }}
            </Tag>
          </template>

          <template #action="{ row }">
            <Button
              type="primary"
              link
              :icon="h(LucideFilePenLine)"
              @click="() => handleEdit(row)"
            />
            <Popconfirm
              title="确定删除该分类吗？"
              ok-text="确定"
              cancel-text="取消"
              @confirm="() => handleDelete(row)"
            >
              <Button danger link :icon="h(LucideTrash2)" />
            </Popconfirm>
          </template>
        </Grid>
      </div>
    </div>
    <Drawer />
  </Page>
</template>
