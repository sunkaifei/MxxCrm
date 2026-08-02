<script lang="ts" setup>
import { h, onMounted, ref, watch } from 'vue';
import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';
import {
  LucideMonitor,
  LucideSmartphone,
  LucideTablet,
  LucideDisplay,
  LucidePlus,
  LucideEye,
  LucideSearch,
  LucideLayoutGrid,
  LucideTag,
  LucideUser,
  LucideTrash2,
  LucideFile,
  LucideUpload,
  LucideDownload,
} from '@vben/icons';
import {
  Button,
  message,
  Modal,
  Tag,
  Skeleton,
  Empty,
  Input,
  Affix,
  Tooltip,
} from 'ant-design-vue';
import { templateApi, siteApi } from '#/api';

// 无预览图时的占位图（内联 SVG，避免依赖外部服务）
const NO_PREVIEW_IMG =
  'data:image/svg+xml,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" width="600" height="400" viewBox="0 0 600 400"><rect width="600" height="400" fill="#f0f0f0"/><text x="300" y="200" font-family="sans-serif" font-size="24" fill="#bfbfbf" text-anchor="middle" dominant-baseline="middle">No Preview</text></svg>',
  );
import type { TemplateListVO } from '#/api/core/website/template';
import { requestClient } from '#/api/request';
import TemplateDrawer from './drawer.vue';
import PagesDrawer from './pages-drawer.vue';
import PageEditor from './page-editor.vue';

const accessStore = useAccessStore();

// --- 状态 ---
const templates = ref<TemplateListVO[]>([]);
const loading = ref(false);
const total = ref(0);
const page = ref(1);
const pageSize = ref(12);
const keyword = ref('');
const activeCategoryId = ref<number | undefined>(undefined);
const categories = ref<{ id: number; name: string; children?: any[] }[]>([]);

// 预览
const previewVisible = ref(false);
const previewTemplate = ref<TemplateListVO | null>(null);
const previewDetail = ref<any>(null);
const previewLoading = ref(false);
const previewMode = ref<'large' | 'site'>('large'); // 大图预览 / 站点预览

// 应用到网站
const applyVisible = ref(false);
const applyTemplate = ref<TemplateListVO | null>(null);
const applying = ref(false);

// 页面管理の状态
const selectedTemplate = ref<TemplateListVO | null>(null);
const editingPage = ref<{ templateId: number; row?: any } | null>(null);

// --- 页面管理抽屉 ---
const [PagesDrawerInstance, pagesDrawerApi] = useVbenDrawer({
  connectedComponent: PagesDrawer,
  onClosed() {
    selectedTemplate.value = null;
  },
});

// 页面编辑器抽屉
const [PageEditorInstance, pageEditorApi] = useVbenDrawer({
  connectedComponent: PageEditor,
  onClosed() {
    editingPage.value = null;
    loadTemplates();
  },
});

// 页面列表抽屉
function openPagesDrawer(item: TemplateListVO) {
  selectedTemplate.value = item;
  pagesDrawerApi.setData({
    templateId: Number(item.id),
    templateName: item.name || '',
    onEditPage: (data: { templateId: number; row?: any }) => {
      pagesDrawerApi.close();
      openPageEditor(data);
    },
    onRefreshTemplates: () => {
      loadTemplates();
    },
  });
  pagesDrawerApi.open();
}

// 页面编辑器
function openPageEditor(data: { templateId: number; row?: any }) {
  editingPage.value = data;
  pageEditorApi.setData(data);
  pageEditorApi.open();
}

// --- 模板主题编辑抽屉（旧模板市场） ---
const [TemplateDrawerInstance, templateDrawerApi] = useVbenDrawer({
  connectedComponent: TemplateDrawer,
  onClosed() {
    const d = templateDrawerApi.getData();
    if (d?.needRefresh) loadTemplates();
  },
});

function handleCreateTemplate() {
  templateDrawerApi.setData({ create: true });
  templateDrawerApi.open();
}

// --- 分类加载 ---
async function loadCategories() {
  try {
    const res: any = await requestClient.get('/api/system/template_category/options');
    const rawList = Array.isArray(res) ? res : res?.data || [];
    categories.value = rawList;
  } catch {
    categories.value = [];
  }
}

// --- 模板列表加载 ---
async function loadTemplates() {
  loading.value = true;
  try {
    const res: any = await templateApi.list({
      page: page.value,
      pageSize: pageSize.value,
      keywords: keyword.value || undefined,
      status: 1,
      categoryId: activeCategoryId.value,
    });
    const items = res?.items || [];
    templates.value = items;
    total.value = res?.total || 0;
  } catch {
    templates.value = [];
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadCategories();
  loadTemplates();
});

// 搜索防抖
let searchTimer: any;
watch(keyword, () => {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    page.value = 1;
    loadTemplates();
  }, 300);
});

// 分类切换
function selectCategory(catId: number | undefined) {
  activeCategoryId.value = catId;
  page.value = 1;
  loadTemplates();
}

// 分页
function handlePageChange(p: number) {
  page.value = p;
  loadTemplates();
}

function handlePageSizeChange(_current: number, size: number) {
  pageSize.value = size;
  page.value = 1;
  loadTemplates();
}

// --- 设备支持图标 ---
function deviceBadges(row: TemplateListVO) {
  const badges: { show: boolean; icon: any; label: string; color: string }[] = [
    { show: row.terminalPc === 1, icon: LucideMonitor, label: '电脑端', color: 'blue' },
    { show: row.terminalMobile === 1, icon: LucideSmartphone, label: '手机端', color: 'green' },
    { show: row.terminalIpad === 1, icon: LucideTablet, label: '平板', color: 'orange' },
    { show: row.terminalDisplay === 1, icon: LucideDisplay, label: '展示机', color: 'purple' },
  ];
  return badges.filter(b => b.show);
}

// --- 预览 ---
function handlePreviewLarge(item: TemplateListVO) {
  previewTemplate.value = item;
  previewMode.value = 'large';
  previewVisible.value = true;
  loadPreviewDetail(item);
}

function handlePreviewSite(item: TemplateListVO) {
  previewTemplate.value = item;
  previewMode.value = 'site';
  previewVisible.value = true;
  loadPreviewDetail(item);
}

async function loadPreviewDetail(item: TemplateListVO) {
  previewLoading.value = true;
  previewDetail.value = null;
  try {
    const res: any = await templateApi.detail(Number(item.id));
    previewDetail.value = res?.data || res;
  } catch {
    previewDetail.value = null;
  } finally {
    previewLoading.value = false;
  }
}

// --- 应用到网站 ---
async function handleApply(item: TemplateListVO) {
  applyTemplate.value = item;
  applyVisible.value = true;
}

async function confirmApply() {
  if (!applyTemplate.value) {
    message.warning('请选择要应用的模板');
    return;
  }
  applying.value = true;
  try {
    // 单站模式：直接应用到当前站点
    await siteApi.updateCurrent({
      templateId: Number(applyTemplate.value.id),
    } as any);
    message.success(`已将模板「${applyTemplate.value.name}」应用到当前站点`);
    applyVisible.value = false;
  } catch {
    // 全局拦截器处理
  } finally {
    applying.value = false;
  }
}

// --- 删除 ---
async function handleDelete(item: TemplateListVO) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除模板「${item.name}」吗？`,
    okType: 'danger',
    onOk: async () => {
      await templateApi.delete([Number(item.id)]);
      message.success('删除成功');
      loadTemplates();
    },
  });
}

// --- 导出模板 ---
async function handleExport(item: TemplateListVO) {
  try {
    const blob = await templateApi.exportTemplate(Number(item.id));
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${item.name || 'template'}.mtp`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    message.success('导出成功');
  } catch {
    message.error('导出失败');
  }
}

// --- 导入模板 ---
function handleImportTemplate() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.mtp,.zip';
  input.onchange = async (e: any) => {
    const file = e.target?.files?.[0];
    if (!file) return;
    try {
      await templateApi.importTemplate(file);
      message.success('模板导入成功');
      loadTemplates();
    } catch (err: any) {
      message.error(err?.message || '导入失败');
    }
  };
  input.click();
}
</script>

<template>
  <Page auto-content-height>
    <div class="template-market">
      <!-- 左侧分类导航 -->
      <aside class="template-sidebar">
        <Affix>
          <div class="sidebar-header">
            <h3 class="sidebar-title">
              <component :is="LucideLayoutGrid" style="margin-right: 6px" />
              模板分类
            </h3>
          </div>
          <div class="sidebar-search">
            <Input
              v-model:value="keyword"
              placeholder="搜索模板..."
              allow-clear
              size="middle"
            >
              <template #prefix>
                <component :is="LucideSearch" style="color: #bfbfbf; font-size: 14px" />
              </template>
            </Input>
          </div>
          <div class="sidebar-category-list">
            <div
              class="category-item"
              :class="{ active: activeCategoryId === undefined }"
              @click="selectCategory(undefined)"
            >
              <span class="category-label">全部模板</span>
              <span class="category-count">{{ total }}</span>
            </div>
            <div
              v-for="cat in categories"
              :key="cat.id"
              class="category-item"
              :class="{ active: activeCategoryId === cat.id }"
              @click="selectCategory(cat.id)"
            >
              <span class="category-label">{{ cat.name }}</span>
            </div>
          </div>

          <div
            v-if="accessStore.hasAccessCode('template:add')"
            class="sidebar-footer"
          >
            <Button
              type="primary"
              block
              :icon="h(LucidePlus)"
              @click="handleCreateTemplate"
            >
              新增模板
            </Button>
          </div>
        </Affix>
      </aside>

      <!-- 右侧内容区 -->
      <main class="template-main">
        <!-- 顶部信息栏 -->
        <div class="template-topbar">
          <div class="topbar-info">
            <h2 class="topbar-title">
              {{ activeCategoryId ? (categories.find(c => c.id === activeCategoryId)?.name || '模板列表') : '全部模板' }}
            </h2>
            <span class="topbar-count">共 {{ total }} 个模板</span>
          </div>
          <div class="topbar-actions">
            <Button
              type="primary"
              @click="handleImportTemplate"
            >
              <template #icon><component :is="LucideUpload" /></template>
              导入模板
            </Button>
            <Button
              v-if="accessStore.hasAccessCode('template:add')"
              :icon="h(LucidePlus)"
              @click="handleCreateTemplate"
            >
              新增模板
            </Button>
          </div>
        </div>

        <!-- 卡片网格 -->
        <div class="template-grid-wrapper">
          <!-- 加载中 -->
          <div v-if="loading" class="template-grid">
            <div v-for="i in 6" :key="i" class="template-card-skeleton">
              <div class="skeleton-image" />
              <div class="skeleton-info">
                <Skeleton active :paragraph="{ rows: 2, width: ['80%', '60%'] }" :avatar="{ size: 'small' }" />
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-else-if="templates.length === 0" class="template-empty">
            <Empty description="暂无模板数据" />
          </div>

          <!-- 模板卡片 -->
          <div v-else class="template-grid">
            <div
              v-for="item in templates"
              :key="item.id"
              class="template-card"
            >
              <!-- 预览图区域 -->
              <div class="card-image-wrapper">
                <img
                  :src="item.previewPic || NO_PREVIEW_IMG"
                  :alt="item.name"
                  class="card-preview-img"
                  loading="lazy"
                />
                <!-- 悬停遮罩 -->
                <div class="card-overlay">
                  <div class="overlay-buttons">
                    <Button
                      type="primary"
                      size="large"
                      :icon="h(LucideEye)"
                      @click="handlePreviewLarge(item)"
                    >
                      预览大图
                    </Button>
                    <Button
                      size="large"
                      ghost
                      style="color: #fff; border-color: #fff"
                      :icon="h(LucideMonitor)"
                      @click="handlePreviewSite(item)"
                    >
                      预览模板
                    </Button>
                  </div>
                </div>
                <!-- 状态标记 -->
                <div v-if="item.status === 1" class="card-status-tag">
                  <Tag color="success" size="small">启用</Tag>
                </div>
                <div v-else class="card-status-tag">
                  <Tag color="default" size="small">禁用</Tag>
                </div>
              </div>

              <!-- 信息区域 -->
              <div class="card-info">
                <div class="card-title-row">
                  <h3 class="card-title" :title="item.name">{{ item.name }}</h3>
                </div>

                <div class="card-device-row">
                  <span class="device-label">访问支持：</span>
                  <div class="device-tags">
                    <template v-if="deviceBadges(item).length > 0">
                      <Tag
                        v-for="dev in deviceBadges(item)"
                        :key="dev.label"
                        :color="dev.color"
                        size="small"
                      >
                        <component :is="dev.icon" style="font-size: 11px; margin-right: 2px" />
                        {{ dev.label }}
                      </Tag>
                    </template>
                    <span v-else class="no-device">未设置</span>
                  </div>
                </div>

                <div class="card-desc-row">
                  <span class="desc-label">简介：</span>
                  <span class="desc-text">
                    {{ (item as any).remark || '暂无简介' }}
                  </span>
                </div>

                <div class="card-footer-row">
                  <div class="card-provider">
                    <component :is="LucideUser" class="provider-icon" />
                    <span>官方</span>
                  </div>
                  <div class="card-actions">
                    <Tooltip title="管理页面">
                      <Button
                        type="primary"
                        size="small"
                        :icon="h(LucideFile)"
                        @click="openPagesDrawer(item)"
                      >
                        页面
                      </Button>
                    </Tooltip>
                    <Tooltip title="导出模板">
                      <Button size="small" @click="handleExport(item)">
                        <template #icon><component :is="LucideDownload" /></template>
                      </Button>
                    </Tooltip>
                    <Tooltip title="应用到网站">
                      <Button
                        type="link"
                        size="small"
                        :icon="h(LucideTag)"
                        @click="handleApply(item)"
                      />
                    </Tooltip>
                    <Tooltip title="删除">
                      <Button
                        type="link"
                        danger
                        size="small"
                        :icon="h(LucideTrash2)"
                        @click="handleDelete(item)"
                      />
                    </Tooltip>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 分页 -->
        <div v-if="total > pageSize" class="template-pagination">
          <a-pagination
            :current="page"
            :total="total"
            :page-size="pageSize"
            show-size-changer
            :page-size-options="['12', '24', '48']"
            show-quick-jumper
            @change="handlePageChange"
            @show-size-change="handlePageSizeChange"
          />
        </div>
      </main>
    </div>

    <!-- 预览弹窗 -->
    <Modal
      v-model:open="previewVisible"
      :title="previewMode === 'large' ? '预览大图' : '预览模板'"
      width="85%"
      :footer="null"
      destroy-on-close
      :mask-closable="true"
      class="template-preview-modal"
    >
      <div v-if="previewLoading" class="preview-loading">
        <Skeleton active :paragraph="{ rows: 6 }" />
      </div>

      <div v-else class="preview-content">
        <!-- 预览模式：大图 -->
        <div v-if="previewMode === 'large'" class="preview-large">
          <img
            :src="previewTemplate?.previewPic || NO_PREVIEW_IMG"
            :alt="previewTemplate?.name"
            class="preview-large-img"
          />
        </div>

        <!-- 预览模式：站点 -->
        <div v-else class="preview-site">
          <iframe
            v-if="previewDetail?.previewUrl"
            :src="previewDetail.previewUrl"
            class="preview-iframe"
            frameborder="0"
          />
          <div v-else class="preview-iframe-empty">
            <Empty description="暂未设置演示网址" />
          </div>
        </div>

        <!-- 详情信息面板 -->
        <div v-if="previewDetail" class="preview-info-panel">
          <div class="info-header">
            <h3 class="info-title">{{ previewDetail.name }}</h3>
            <div class="info-device-tags">
              <Tag v-if="previewDetail.terminalPc === 1" color="blue">
                <component :is="LucideMonitor" style="font-size: 12px; margin-right: 3px" />电脑端
              </Tag>
              <Tag v-if="previewDetail.terminalMobile === 1" color="green">
                <component :is="LucideSmartphone" style="font-size: 12px; margin-right: 3px" />手机端
              </Tag>
              <Tag v-if="previewDetail.terminalIpad === 1" color="orange">
                <component :is="LucideTablet" style="font-size: 12px; margin-right: 3px" />平板
              </Tag>
              <Tag v-if="previewDetail.terminalDisplay === 1" color="purple">
                <component :is="LucideDisplay" style="font-size: 12px; margin-right: 3px" />展示机
              </Tag>
            </div>
          </div>

          <div class="info-grid">
            <div class="info-item">
              <span class="info-label">模板文件夹</span>
              <span class="info-value">{{ previewDetail.templateFolder || '—' }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">演示网址</span>
              <span class="info-value">
                <a v-if="previewDetail.previewUrl" :href="previewDetail.previewUrl" target="_blank" rel="noopener">
                  {{ previewDetail.previewUrl }}
                </a>
                <span v-else style="color: #999">—</span>
              </span>
            </div>
          </div>

          <div class="info-desc">
            <span class="info-label">简介说明</span>
            <p class="info-desc-text">{{ previewDetail.remark || '暂无简介' }}</p>
          </div>

          <div class="info-actions">
            <Button
              type="primary"
              size="large"
              :icon="h(LucideTag)"
              @click="() => { previewVisible = false; previewTemplate && handleApply(previewTemplate); }"
            >
              应用到网站
            </Button>
            <Button
              size="large"
              @click="previewVisible = false"
            >
              关闭
            </Button>
          </div>
        </div>
      </div>
    </Modal>

    <!-- 应用到网站弹窗 -->
    <Modal
      v-model:open="applyVisible"
      title="应用模板到当前站点"
      width="480px"
      :confirm-loading="applying"
      ok-text="确认应用"
      @ok="confirmApply"
      @cancel="applyVisible = false"
    >
      <div style="padding: 8px 0">
        <div style="color: #666; font-size: 14px; line-height: 1.8">
          确认将模板 <strong style="color: #1677ff">{{ applyTemplate?.name }}</strong> 应用到当前站点吗？
          <br />
          <span style="color: #999; font-size: 12px">
            单站模式下模板将直接应用到默认站点，切换后整站外观会立即改变。
          </span>
        </div>
      </div>
    </Modal>

    <!-- 模板主题编辑抽屉 -->
    <TemplateDrawerInstance />
    <!-- 页面列表抽屉（75%） -->
    <PagesDrawerInstance />
    <!-- 页面编辑器抽屉（75%，可全屏） -->
    <PageEditorInstance />
  </Page>
</template>

<style scoped>
/* ========== 主布局 ========== */
.template-market {
  display: flex;
  gap: 20px;
  align-items: flex-start;
}

/* ========== 左侧侧边栏 ========== */
.template-sidebar {
  width: 220px;
  flex-shrink: 0;
  position: sticky;
  top: 16px;
}
.sidebar-header {
  padding: 4px 4px 12px;
}
.sidebar-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: rgba(0, 0, 0, 0.88);
  display: flex;
  align-items: center;
}
.sidebar-search {
  margin-bottom: 12px;
  padding: 0 2px;
}
.sidebar-category-list {
  background: #fff;
  border-radius: 8px;
  padding: 6px 0;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.03), 0 1px 6px -1px rgba(0, 0, 0, 0.02);
  max-height: calc(100vh - 280px);
  overflow-y: auto;
}
.category-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  cursor: pointer;
  transition: background 0.2s ease;
  font-size: 14px;
  color: rgba(0, 0, 0, 0.75);
  position: relative;
}
.category-item:hover {
  background: #f5f7fa;
  color: #1677ff;
}
.category-item.active {
  background: #e6f4ff;
  color: #1677ff;
  font-weight: 500;
}
.category-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 16px;
  background: #1677ff;
  border-radius: 0 2px 2px 0;
}
.category-count {
  font-size: 12px;
  color: rgba(0, 0, 0, 0.4);
  background: #f0f0f0;
  padding: 0 6px;
  border-radius: 10px;
  font-weight: 400;
}
.category-item.active .category-count {
  background: #fff;
  color: #1677ff;
}
.sidebar-footer {
  margin-top: 16px;
  padding: 0 2px;
}

/* ========== 右侧主内容 ========== */
.template-main {
  flex: 1;
  min-width: 0;
}
.template-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
.topbar-info {
  display: flex;
  align-items: baseline;
  gap: 12px;
}
.topbar-title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  color: rgba(0, 0, 0, 0.88);
  letter-spacing: -0.5px;
}
.topbar-count {
  font-size: 13px;
  color: rgba(0, 0, 0, 0.45);
}

/* ========== 卡片网格 ========== */
.template-grid-wrapper {
  min-height: 400px;
}
.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 18px;
}

/* ========== 模板卡片 ========== */
.template-card {
  background: #fff;
  border-radius: 10px;
  overflow: hidden;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04), 0 1px 6px rgba(0, 0, 0, 0.04);
  transition: transform 0.25s ease, box-shadow 0.25s ease;
  display: flex;
  flex-direction: column;
  border: 1px solid #f0f0f0;
}
.template-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  border-color: transparent;
}

/* 预览图 */
.card-image-wrapper {
  position: relative;
  width: 100%;
  aspect-ratio: 3 / 2;
  overflow: hidden;
  background: #f5f5f5;
}
.card-preview-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.5s ease;
}
.template-card:hover .card-preview-img {
  transform: scale(1.06);
}

/* 悬停遮罩 */
.card-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(to bottom, rgba(0, 0, 0, 0.1) 0%, rgba(0, 0, 0, 0.6) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.25s ease;
}
.template-card:hover .card-overlay {
  opacity: 1;
}
.overlay-buttons {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
}

/* 状态标签 */
.card-status-tag {
  position: absolute;
  top: 10px;
  right: 10px;
}

/* 信息区域 */
.card-info {
  padding: 14px 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}
.card-title-row {
  margin-bottom: 2px;
}
.card-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: rgba(0, 0, 0, 0.88);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-device-row,
.card-desc-row {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  font-size: 12px;
  line-height: 1.6;
}
.device-label,
.desc-label {
  color: rgba(0, 0, 0, 0.45);
  flex-shrink: 0;
}
.device-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.no-device {
  color: rgba(0, 0, 0, 0.35);
}
.desc-text {
  color: rgba(0, 0, 0, 0.7);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* 底部 */
.card-footer-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 8px;
  margin-top: auto;
  border-top: 1px solid #f5f5f5;
}
.card-provider {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: rgba(0, 0, 0, 0.55);
}
.provider-icon {
  font-size: 13px;
}
.card-actions {
  display: flex;
  gap: 6px;
}

/* 骨架屏 */
.template-card-skeleton {
  background: #fff;
  border-radius: 10px;
  overflow: hidden;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}
.skeleton-image {
  width: 100%;
  aspect-ratio: 3 / 2;
  background: #f5f5f5;
}
.skeleton-info {
  padding: 14px 16px;
}

/* 空状态 */
.template-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  background: #fff;
  border-radius: 10px;
}

/* ========== 分页 ========== */
.template-pagination {
  display: flex;
  justify-content: flex-end;
  padding: 20px 0 4px;
}

/* ========== 预览弹窗 ========== */
:deep(.template-preview-modal .ant-modal-content) {
  border-radius: 12px;
  overflow: hidden;
}
:deep(.template-preview-modal .ant-modal-body) {
  padding: 0;
}
.preview-loading {
  padding: 40px;
}
.preview-content {
  display: flex;
  min-height: 600px;
  max-height: 78vh;
}
.preview-large,
.preview-site {
  flex: 1;
  background: #f0f2f5;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: auto;
}
.preview-large-img {
  max-width: 100%;
  max-height: 78vh;
  object-fit: contain;
  display: block;
}
.preview-iframe {
  width: 100%;
  height: 78vh;
  border: none;
  background: #fff;
}
.preview-iframe-empty {
  padding: 80px 40px;
}

/* 详情面板 */
.preview-info-panel {
  width: 320px;
  flex-shrink: 0;
  background: #fff;
  border-left: 1px solid #f0f0f0;
  padding: 24px 20px;
  display: flex;
  flex-direction: column;
}
.info-header {
  margin-bottom: 16px;
}
.info-title {
  margin: 0 0 10px;
  font-size: 18px;
  font-weight: 600;
  color: rgba(0, 0, 0, 0.88);
}
.info-device-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.info-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
}
.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.info-label {
  font-size: 12px;
  color: rgba(0, 0, 0, 0.45);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.info-value {
  font-size: 14px;
  color: rgba(0, 0, 0, 0.88);
  word-break: break-all;
}
.info-desc {
  flex: 1;
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.info-desc-text {
  margin: 0;
  font-size: 13px;
  color: rgba(0, 0, 0, 0.75);
  line-height: 1.7;
}
.info-actions {
  display: flex;
  gap: 10px;
  padding-top: 16px;
  border-top: 1px solid #f0f0f0;
}

/* ========== 响应式 ========== */
@media (max-width: 900px) {
  .template-market {
    flex-direction: column;
  }
  .template-sidebar {
    width: 100%;
    position: static;
  }
  .sidebar-category-list {
    max-height: none;
    display: flex;
    flex-wrap: wrap;
    padding: 8px;
    gap: 4px;
  }
  .category-item {
    padding: 6px 14px;
    border-radius: 16px;
    background: #f5f5f5;
  }
  .category-item.active {
    background: #1677ff;
    color: #fff;
  }
  .category-item.active::before {
    display: none;
  }
  .category-count {
    display: none;
  }
}
</style>
