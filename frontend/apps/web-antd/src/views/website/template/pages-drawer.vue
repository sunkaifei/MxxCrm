<script lang="ts" setup>
import { h, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import {
  LucideFilePenLine,
  LucidePlus,
  LucideSearch,
  LucideTrash2,
} from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Empty,
  Input,
  message,
  Modal,
  Pagination,
  Popconfirm,
  Select,
  Skeleton,
  Space,
  Tag,
} from 'ant-design-vue';

import { deleteTemplateDataApi, getTemplateDataListApi } from '#/api';

import PageEditor from './page-editor.vue';

const accessStore = useAccessStore();

// 嵌套的页面编辑器抽屉
const [PageEditorInstance, pageEditorApi] = useVbenDrawer({
  connectedComponent: PageEditor,
  onClosed() {
    // 编辑器关闭后刷新列表
    loadPages();
    drawerData.value.onRefreshTemplates?.();
  },
});

// 从父组件接收的数据
const drawerData = ref<{
  onEditPage?: (data: { row?: any; templateId: number }) => void;
  onRefreshTemplates?: () => void;
  templateId: number;
  templateName: string;
}>({ templateId: 0, templateName: '' });

// 页面列表数据
const pages = ref<any[]>([]);
const loading = ref(false);
const total = ref(0);
const page = ref(1);
const pageSize = ref(10);

// 搜索条件
const keywords = ref('');
const typeId = ref<number | undefined>(undefined);
const status = ref<number | undefined>(undefined);

async function loadPages() {
  if (!drawerData.value.templateId) return;
  loading.value = true;
  try {
    const params: any = {
      templateId: drawerData.value.templateId,
      page: page.value,
      pageSize: pageSize.value,
    };
    if (keywords.value) params.keywords = keywords.value;
    if (typeId.value !== undefined) params.typeId = typeId.value;
    if (status.value !== undefined) params.status = status.value;

    const res: any = await getTemplateDataListApi(params);
    const data = res?.data || res;
    pages.value = data?.items || data?.rows || data?.list || [];
    total.value = data?.total || data?.count || 0;
  } catch (error: any) {
    pages.value = [];
    total.value = 0;
    message.error(error?.message || '加载页面列表失败');
  } finally {
    loading.value = false;
  }
}

function handleSearch() {
  page.value = 1;
  loadPages();
}

function handleReset() {
  keywords.value = '';
  typeId.value = undefined;
  status.value = undefined;
  page.value = 1;
  loadPages();
}

function handlePageChange(p: number) {
  page.value = p;
  loadPages();
}

function handlePageSizeChange(_current: number, size: number) {
  pageSize.value = size;
  page.value = 1;
  loadPages();
}

// 类型映射
const typeOptions = [
  { value: 1, label: '首页', color: 'blue' },
  { value: 2, label: '列表页', color: 'cyan' },
  { value: 3, label: '内容页', color: 'green' },
  { value: 4, label: '栏目封面', color: 'purple' },
  { value: 5, label: '报价页', color: 'orange' },
  { value: 6, label: '专题', color: 'red' },
  { value: 7, label: '产品列表', color: 'geekblue' },
  { value: 8, label: '产品详情', color: 'lime' },
  { value: 14, label: '页头', color: 'gold' },
  { value: 15, label: '页脚', color: 'volcano' },
];

function getTypeInfo(typeId: number) {
  return (
    typeOptions.find((o) => o.value === typeId) || {
      label: '未知',
      color: 'default',
    }
  );
}

function getTypeLabel(typeId: number): string {
  return getTypeInfo(typeId).label;
}

function getTypeColor(typeId: number): string {
  return getTypeInfo(typeId).color;
}

// 删除
async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除页面「${row.name}」吗？`,
    okType: 'danger',
    onOk: async () => {
      try {
        await deleteTemplateDataApi([row.id]);
        message.success('删除成功');
        loadPages();
        drawerData.value.onRefreshTemplates?.();
      } catch {
        // 全局拦截器处理
      }
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
      page.value = 1;
      keywords.value = '';
      typeId.value = undefined;
      status.value = undefined;
      loadPages();
    }
  },
});
</script>

<template>
  <Drawer>
    <template #title>
      <span>页面管理 - {{ drawerData.templateName }}</span>
    </template>
    <div class="pages-drawer-content">
      <!-- 搜索栏 -->
      <div class="pages-search-bar">
        <Input
          v-model:value="keywords"
          placeholder="搜索页面名称"
          allow-clear
          style="width: 200px"
          @press-enter="handleSearch"
        >
          <template #prefix>
            <component :is="LucideSearch" style="font-size: 14px" />
          </template>
        </Input>
        <Select
          v-model:value="typeId"
          placeholder="页面类型"
          allow-clear
          style="width: 140px"
          :options="
            typeOptions.map((o) => ({ value: o.value, label: o.label }))
          "
          @change="handleSearch"
        />
        <Select
          v-model:value="status"
          placeholder="状态"
          allow-clear
          style="width: 100px"
          :options="[
            { value: 1, label: '启用' },
            { value: 0, label: '禁用' },
          ]"
          @change="handleSearch"
        />
        <Button
          size="small"
          type="primary"
          :icon="h(LucideSearch)"
          @click="handleSearch"
        >
          搜索
        </Button>
        <Button size="small" @click="handleReset">重置</Button>
      </div>

      <!-- 顶部操作栏 -->
      <div class="pages-toolbar">
        <div class="pages-toolbar-info">
          <span class="pages-count">共 {{ total }} 个页面</span>
        </div>
        <Space>
          <Button size="small" @click="loadPages">刷新</Button>
          <Button
            v-if="accessStore.hasAccessCode('template:data:add')"
            type="primary"
            size="small"
            :icon="h(LucidePlus)"
            @click="handleCreate"
          >
            新增页面
          </Button>
        </Space>
      </div>

      <!-- 加载中 -->
      <div v-if="loading" class="pages-loading">
        <div v-for="i in 4" :key="i" class="page-skeleton-row">
          <Skeleton active :paragraph="{ rows: 1, width: ['60%'] }" />
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else-if="pages.length === 0" class="pages-empty">
        <Empty description="该模板下暂无页面数据">
          <Button
            v-if="accessStore.hasAccessCode('template:data:add')"
            type="primary"
            size="small"
            @click="handleCreate"
          >
            新增第一个页面
          </Button>
        </Empty>
      </div>

      <!-- 页面列表 -->
      <div v-else class="pages-list">
        <div v-for="item in pages" :key="item.id" class="page-card">
          <div class="page-card-left">
            <div class="page-card-name">
              <span class="page-name-text" :title="item.name">{{
                item.name
              }}</span>
              <Tag :color="getTypeColor(item.typeId)" size="small">
                {{ getTypeLabel(item.typeId) }}
              </Tag>
            </div>
            <div class="page-card-meta">
              <span class="meta-item">排序: {{ item.sort ?? 0 }}</span>
              <span class="meta-divider">|</span>
              <span class="meta-item">
                <Tag
                  :color="item.status === 1 ? 'success' : 'default'"
                  size="small"
                >
                  {{ item.status === 1 ? '启用' : '禁用' }}
                </Tag>
              </span>
              <span class="meta-divider">|</span>
              <span class="meta-item"
                >创建: {{ formatDateTime(item.createTime) }}</span
              >
            </div>
          </div>
          <div class="page-card-right">
            <Button
              v-if="accessStore.hasAccessCode('template:data:update')"
              type="link"
              :icon="h(LucideFilePenLine)"
              @click="handleEdit(item)"
            >
              编辑
            </Button>
            <Popconfirm
              title="确定删除该页面吗？"
              ok-text="确定"
              cancel-text="取消"
              @confirm="handleDelete(item)"
            >
              <Button
                v-if="accessStore.hasAccessCode('template:data:delete')"
                type="link"
                danger
                :icon="h(LucideTrash2)"
              />
            </Popconfirm>
          </div>
        </div>
      </div>

      <!-- 分页 -->
      <div v-if="total > pageSize" class="pages-pagination">
        <Pagination
          :current="page"
          :total="total"
          :page-size="pageSize"
          :page-size-options="['10', '20', '50']"
          show-size-changer
          show-quick-jumper
          size="small"
          @change="handlePageChange"
          @show-size-change="handlePageSizeChange"
        />
      </div>
    </div>

    <!-- 嵌套的页面编辑器抽屉 -->
    <PageEditorInstance />
  </Drawer>
</template>

<style scoped>
.pages-drawer-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 0 4px;
}

.pages-search-bar {
  display: flex;
  flex-shrink: 0;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-bottom: 12px;
}

.pages-toolbar {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.pages-toolbar-info {
  display: flex;
  gap: 8px;
  align-items: center;
}

.pages-count {
  font-size: 13px;
  color: rgb(0 0 0 / 45%);
}

.pages-loading {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.page-skeleton-row {
  padding: 12px;
  background: rgb(0 0 0 / 2%);
  border-radius: 8px;
}

.pages-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
}

.pages-list {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
}

.page-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  cursor: default;
  background: var(--card-background, #fff);
  border: 1px solid var(--border-color, #f0f0f0);
  border-radius: 8px;
  transition:
    box-shadow 0.2s ease,
    border-color 0.2s ease;
}

.page-card:hover {
  border-color: var(--primary-color, #1677ff);
  box-shadow: 0 2px 8px rgb(22 119 255 / 8%);
}

.page-card-left {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.page-card-name {
  display: flex;
  gap: 8px;
  align-items: center;
}

.page-name-text {
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, rgb(0 0 0 / 88%));
  white-space: nowrap;
}

.page-card-meta {
  display: flex;
  gap: 6px;
  align-items: center;
  font-size: 12px;
  color: rgb(0 0 0 / 45%);
}

.meta-divider {
  color: rgb(0 0 0 / 15%);
}

.page-card-right {
  display: flex;
  flex-shrink: 0;
  gap: 4px;
  align-items: center;
  margin-left: 12px;
}

.pages-pagination {
  display: flex;
  flex-shrink: 0;
  justify-content: flex-end;
  padding-top: 16px;
}

/* 暗黑模式适配 */
:root.dark .page-card {
  --card-background: #1f1f1f;
  --border-color: #333;
  --text-primary: rgb(255 255 255 / 88%);
}

:root.dark .pages-count {
  color: rgb(255 255 255 / 45%);
}

:root.dark .page-skeleton-row {
  background: rgb(255 255 255 / 4%);
}

/* 响应式：小屏幕全宽 */
@media (max-width: 767px) {
  :deep(.ant-drawer-content-wrapper) {
    width: 100% !important;
  }
}
</style>
