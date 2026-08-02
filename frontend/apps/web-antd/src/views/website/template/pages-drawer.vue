<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';
import {
  LucideFilePenLine,
  LucideTrash2,
  LucidePlus,
} from '@vben/icons';
import { Button, message, Modal, Popconfirm, Tag, Empty, Skeleton, Space } from 'ant-design-vue';
import { getTemplateDataListByTemplateApi, deleteTemplateDataApi } from '#/api';
import { formatDateTime } from '@vben/utils';

const accessStore = useAccessStore();

// 从父组件接收的数据
const drawerData = ref<{
  templateId: number;
  templateName: string;
  onEditPage?: (data: { templateId: number; row?: any }) => void;
  onRefreshTemplates?: () => void;
}>({ templateId: 0, templateName: '' });

// 页面列表数据
const pages = ref<any[]>([]);
const loading = ref(false);
const dataLoaded = ref(false);

async function loadPages() {
  if (!drawerData.value.templateId) return;
  loading.value = true;
  try {
    const res: any = await getTemplateDataListByTemplateApi(drawerData.value.templateId);
    pages.value = Array.isArray(res) ? res : res?.data || res?.items || [];
    dataLoaded.value = true;
  } catch {
    pages.value = [];
  } finally {
    loading.value = false;
  }
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
  return typeOptions.find((o) => o.value === typeId) || { label: '未知', color: 'default' };
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
  drawerData.value.onEditPage?.({ templateId: drawerData.value.templateId, row });
}

// 新增
function handleCreate() {
  drawerData.value.onEditPage?.({ templateId: drawerData.value.templateId, row: null });
}

const [Drawer, drawerApi] = useVbenDrawer({
  width: '75%',
  onCancel() {
    drawerApi.close();
  },
  onOpenChange(isOpen) {
    if (isOpen) {
      drawerData.value = drawerApi.getData<any>() || { templateId: 0, templateName: '' };
      dataLoaded.value = false;
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
      <!-- 顶部操作栏 -->
      <div class="pages-toolbar">
        <div class="pages-toolbar-info">
          <span class="pages-count">共 {{ pages.length }} 个页面</span>
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
      <div v-if="loading && !dataLoaded" class="pages-loading">
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
              <span class="page-name-text" :title="item.name">{{ item.name }}</span>
              <Tag :color="getTypeColor(item.typeId)" size="small">{{ getTypeLabel(item.typeId) }}</Tag>
            </div>
            <div class="page-card-meta">
              <span class="meta-item">排序: {{ item.sort ?? 0 }}</span>
              <span class="meta-divider">|</span>
              <span class="meta-item">
                <Tag :color="item.status === 1 ? 'success' : 'default'" size="small">
                  {{ item.status === 1 ? '启用' : '禁用' }}
                </Tag>
              </span>
              <span class="meta-divider">|</span>
              <span class="meta-item">创建: {{ formatDateTime(item.createTime) }}</span>
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
    </div>
  </Drawer>
</template>

<style scoped>
.pages-drawer-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 0 4px;
}

.pages-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  flex-shrink: 0;
}

.pages-toolbar-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pages-count {
  font-size: 13px;
  color: rgba(0, 0, 0, 0.45);
}

.pages-loading {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.page-skeleton-row {
  padding: 12px;
  background: rgba(0, 0, 0, 0.02);
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
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  flex: 1;
}

.page-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--card-background, #fff);
  border: 1px solid var(--border-color, #f0f0f0);
  border-radius: 8px;
  transition: box-shadow 0.2s ease, border-color 0.2s ease;
  cursor: default;
}

.page-card:hover {
  border-color: var(--primary-color, #1677ff);
  box-shadow: 0 2px 8px rgba(22, 119, 255, 0.08);
}

.page-card-left {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.page-card-name {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page-name-text {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, rgba(0, 0, 0, 0.88));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.page-card-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: rgba(0, 0, 0, 0.45);
}

.meta-divider {
  color: rgba(0, 0, 0, 0.15);
}

.page-card-right {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  margin-left: 12px;
}

/* 暗黑模式适配 */
:root.dark .page-card {
  --card-background: #1f1f1f;
  --border-color: #333;
  --text-primary: rgba(255, 255, 255, 0.88);
}

:root.dark .pages-count {
  color: rgba(255, 255, 255, 0.45);
}

:root.dark .page-skeleton-row {
  background: rgba(255, 255, 255, 0.04);
}

/* 响应式：小屏幕全宽 */
@media (max-width: 767px) {
  :deep(.ant-drawer-content-wrapper) {
    width: 100% !important;
  }
}
</style>