<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideEye, LucideFilePenLine, LucidePlus } from '@vben/icons';

import { Button, Image, message, Modal, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { articleApi, categoryApi } from '#/api';

import ArticleDrawer from './drawer.vue';

const categoryTree = ref<any[]>([]);
const detailModalVisible = ref(false);
const detailData = ref<any>(null);

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
          { label: '待审核', value: 0 },
          { label: '已通过', value: 1 },
          { label: '已驳回', value: 2 },
        ],
        placeholder: '请选择状态',
        allowClear: true,
      },
    },
    {
      component: 'TreeSelect',
      fieldName: 'categoryId',
      label: '分类',
      componentProps: {
        treeData: categoryTree,
        placeholder: '请选择分类',
        allowClear: true,
        treeDefaultExpandAll: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'keyword',
      label: '关键词',
      componentProps: {
        placeholder: '请输入文章标题',
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
        return await articleApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status || undefined,
          keyword: formValues.keyword,
          categoryId: formValues.categoryId || undefined,
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
      title: 'ID',
      field: 'id',
      width: 80,
    },
    {
      title: '文章标题',
      field: 'title',
      width: 240,
    },
    {
      title: '分类',
      field: 'categoryId',
      width: 120,
    },
    {
      title: '作者',
      field: 'author',
      width: 120,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 100,
    },
    {
      title: '置顶',
      field: 'istop',
      slots: { default: 'istop' },
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
      width: 200,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: ArticleDrawer,
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
    content: `确定要删除文章"${row.title}"吗？`,
    okType: 'danger',
    onOk: async () => {
      await articleApi.delete([row.id]);
      message.success('删除成功');
      gridApi.query();
    },
  });
}

async function viewDetail(row: any) {
  detailData.value = await articleApi.detail(row.id);
  detailModalVisible.value = true;
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
    <Grid table-title="文章管理">
      <template #toolbar-tools>
        <Button type="primary" :icon="h(LucidePlus)" @click="handleAdd">
          新增文章
        </Button>
      </template>

      <template #status="{ row }">
        <Tag
          :color="
            row.status === 0
              ? 'warning'
              : row.status === 1
                ? 'success'
                : 'error'
          "
        >
          {{
            row.status === 0 ? '待审核' : row.status === 1 ? '已通过' : '已驳回'
          }}
        </Tag>
      </template>

      <template #istop="{ row }">
        <Tag v-if="row.istop === 1" color="red">置顶</Tag>
        <span v-else class="text-gray-400">-</span>
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideEye)"
          @click="() => viewDetail(row)"
        >
          详情
        </Button>
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

    <Modal v-model:open="detailModalVisible" title="文章详情" width="800">
      <div v-if="detailData" class="space-y-4">
        <div class="flex gap-4">
          <Image
            v-if="detailData.titleImage"
            :src="detailData.titleImage"
            width="120"
            height="120"
            fit="cover"
            class="rounded"
          />
          <div class="flex-1 space-y-2">
            <h3 class="font-semibold text-lg">{{ detailData.title }}</h3>
            <p class="text-gray-500">{{ detailData.shortTitle || '-' }}</p>
            <div class="flex gap-4 text-sm text-gray-600">
              <span>作者：{{ detailData.author || '-' }}</span>
              <span>分类ID：{{ detailData.categoryId ?? '-' }}</span>
            </div>
            <div class="flex gap-4 text-sm text-gray-600">
              <span>短链接：{{ detailData.shortUrl || '-' }}</span>
              <span v-if="detailData.originalLink">
                原文链接：<a
                  :href="detailData.originalLink"
                  target="_blank"
                  class="text-blue-500"
                  >{{ detailData.originalLink }}</a
                >
              </span>
            </div>
            <div class="flex gap-4 text-sm text-gray-600">
              <span>
                状态：
                <Tag
                  :color="
                    detailData.status === 0
                      ? 'warning'
                      : detailData.status === 1
                        ? 'success'
                        : 'error'
                  "
                >
                  {{
                    detailData.status === 0
                      ? '待审核'
                      : detailData.status === 1
                        ? '已通过'
                        : '已驳回'
                  }}
                </Tag>
              </span>
              <span>
                置顶：
                <Tag v-if="detailData.istop === 1" color="red">是</Tag>
                <span v-else class="text-gray-400">否</span>
              </span>
              <span>
                推荐：
                <Tag v-if="detailData.isrecommend === 1" color="blue">是</Tag>
                <span v-else class="text-gray-400">否</span>
              </span>
            </div>
            <div class="text-sm text-gray-500">
              创建时间：{{ detailData.createTime || '-' }}
            </div>
          </div>
        </div>
        <div v-if="detailData.description" class="border-t pt-4">
          <h4 class="font-semibold mb-2">摘要</h4>
          <p class="text-gray-600">{{ detailData.description }}</p>
        </div>
        <div v-if="detailData.content" class="border-t pt-4">
          <h4 class="font-semibold mb-2">内容</h4>
          <!-- eslint-disable-next-line vue/no-v-html -- 文章富文本内容，可信来源 -->
          <div class="bg-gray-50 p-4 rounded" v-html="detailData.content"></div>
        </div>
      </div>
    </Modal>
  </Page>
</template>
