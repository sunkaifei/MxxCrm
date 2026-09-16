<script lang="ts" setup>
import type { TemplateListVO } from '#/api/core/website/template';

import { h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import {
  LucideCheckCircle,
  LucideDisplay,
  LucideEllipsis,
  LucideEye,
  LucideFile,
  LucideMonitor,
  LucidePlus,
  LucideSmartphone,
  LucideTablet,
  LucideTag,
  LucideUpload,
  LucideUser,
} from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import {
  Button,
  Empty,
  message,
  Modal,
  Popover,
  Skeleton,
  Tag,
} from 'ant-design-vue';

import { siteApi, templateApi } from '#/api';

import TemplateDrawer from './drawer.vue';
import PagesDrawer from './pages-drawer.vue';

// 无预览图时的占位图（内联 SVG，避免依赖外部服务）
const NO_PREVIEW_IMG = `data:image/svg+xml,${encodeURIComponent(
  '<svg xmlns="http://www.w3.org/2000/svg" width="600" height="400" viewBox="0 0 600 400"><rect width="600" height="400" fill="#f0f0f0"/><text x="300" y="200" font-family="sans-serif" font-size="24" fill="#bfbfbf" text-anchor="middle" dominant-baseline="middle">No Preview</text></svg>',
)}`;

const accessStore = useAccessStore();

// --- 状态 ---
const templates = ref<TemplateListVO[]>([]);
const loading = ref(false);
const total = ref(0);
const page = ref(1);
const pageSize = ref(12);

// 预览
const previewVisible = ref(false);
const previewTemplate = ref<null | TemplateListVO>(null);
const previewDetail = ref<any>(null);
const previewLoading = ref(false);
const previewMode = ref<'large' | 'site'>('large'); // 大图预览 / 站点预览

// 应用到网站
const applyVisible = ref(false);
const applyTemplate = ref<null | TemplateListVO>(null);
const applying = ref(false);

// --- 「⋯」更多菜单 ---
// ant-design-vue 4.x 移除了 <Menu.Item> 模板子组件写法（旧写法渲染为未知组件 →
// 菜单弹层为空、肉眼"点了没反应"），必须改用 :items 声明式。
// 受控 open + 连点保护保留：防止双击/连点把菜单 open→close→open 抖没。
const menuOpenId = ref<null | number | string>(null);
let menuOpenedAt = 0;
function handleMenuOpenChange(id: number | string, open: boolean) {
  if (open) {
    menuOpenedAt = Date.now();
    menuOpenId.value = id;
    return;
  }
  if (Date.now() - menuOpenedAt < 300) {
    return;
  }
  menuOpenId.value = null;
}

// 页面管理の状态
const selectedTemplate = ref<null | TemplateListVO>(null);

// --- 当前站点正在使用的模板 ---
// 来源：GET /api/system/site/current 的 templateId / mobileTemplateId
// 注意：后端 SiteDetailVO 用 serialize_option_u64_to_string，这两个字段是**字符串**，比较前必须 Number()
const currentTemplateId = ref<null | number>(null);
const currentMobileTemplateId = ref<null | number>(null);
const currentTemplateName = ref('');
// 当前生效模板的完整详情：用于顶部「当前生效」区块（缩略图 / 设备支持 / 简介）
// 取不到时为 null，顶部区块整体不渲染，不影响列表浏览
const currentTemplateDetail = ref<any>(null);

/**
 * 读取「当前站点正在使用的模板」。
 *
 * 失败时静默降级（例如操作员没有 system:site:view 权限），
 * 只影响卡片上的「当前使用」标记与顶部区块，不影响模板列表本身的浏览。
 */
async function loadCurrentTemplate() {
  try {
    const site: any = await siteApi.getCurrent();
    const pc = Number(site?.templateId ?? 0) || null;
    const mobile = Number(site?.mobileTemplateId ?? 0) || null;
    currentTemplateId.value = pc;
    // 移动端模板与 PC 模板相同时不重复标记
    currentMobileTemplateId.value = mobile && mobile !== pc ? mobile : null;

    if (pc) {
      try {
        const res: any = await templateApi.detail(pc);
        const d = res?.data || res;
        currentTemplateDetail.value = d || null;
        currentTemplateName.value = d?.name || '';
      } catch {
        currentTemplateDetail.value = null;
        currentTemplateName.value = '';
      }
    } else {
      currentTemplateDetail.value = null;
      currentTemplateName.value = '';
    }
  } catch {
    currentTemplateId.value = null;
    currentMobileTemplateId.value = null;
    currentTemplateName.value = '';
    currentTemplateDetail.value = null;
  }
}

/** 顶部「当前生效」区块：管理当前模板的页面 */
function openCurrentPagesDrawer() {
  if (!currentTemplateId.value) return;
  selectedTemplate.value = null;
  pagesDrawerApi.setData({
    templateId: currentTemplateId.value,
    templateName: currentTemplateName.value || '',
    onRefreshTemplates: () => {
      loadTemplates();
    },
  });
  pagesDrawerApi.open();
}

/** 顶部「当前生效」区块：放大查看当前模板预览图 */
function handlePreviewCurrentLarge() {
  if (!currentTemplateDetail.value) return;
  previewTemplate.value = currentTemplateDetail.value;
  previewMode.value = 'large';
  previewVisible.value = true;
}

/** 该模板是否为当前站点正在使用的模板 */
function isCurrentTemplate(row?: null | TemplateListVO) {
  return (
    !!row &&
    currentTemplateId.value !== null &&
    Number(row.id) === currentTemplateId.value
  );
}

/** 该模板是否为当前站点的移动端模板 */
function isCurrentMobileTemplate(row?: null | TemplateListVO) {
  return (
    !!row &&
    currentMobileTemplateId.value !== null &&
    Number(row.id) === currentMobileTemplateId.value
  );
}

// --- 页面管理抽屉 ---
const [PagesDrawerInstance, pagesDrawerApi] = useVbenDrawer({
  connectedComponent: PagesDrawer,
  onClosed() {
    selectedTemplate.value = null;
  },
});

// 页面列表抽屉
function openPagesDrawer(item: TemplateListVO) {
  selectedTemplate.value = item;
  pagesDrawerApi.setData({
    templateId: Number(item.id),
    templateName: item.name || '',
    onRefreshTemplates: () => {
      loadTemplates();
    },
  });
  pagesDrawerApi.open();
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

function handleEditTemplate(item: TemplateListVO) {
  templateDrawerApi.setData({ create: false, row: item });
  templateDrawerApi.open();
}

// --- 模板列表加载 ---
async function loadTemplates() {
  loading.value = true;
  try {
    const res: any = await templateApi.list({
      page: page.value,
      pageSize: pageSize.value,
      status: 1,
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
  loadTemplates();
  loadCurrentTemplate();
});

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
  const badges: { color: string; icon: any; label: string; show: boolean }[] = [
    {
      show: row.terminalPc === 1,
      icon: LucideMonitor,
      label: '电脑端',
      color: 'blue',
    },
    {
      show: row.terminalMobile === 1,
      icon: LucideSmartphone,
      label: '手机端',
      color: 'green',
    },
    {
      show: row.terminalIpad === 1,
      icon: LucideTablet,
      label: '平板',
      color: 'orange',
    },
    {
      show: row.terminalDisplay === 1,
      icon: LucideDisplay,
      label: '展示机',
      color: 'purple',
    },
  ];
  return badges.filter((b) => b.show);
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
    // 立刻回写本地状态：卡片描边/徽标、顶部「当前生效」区块、底部「使用中」都无需刷新页面即可切换
    currentTemplateId.value = Number(applyTemplate.value.id);
    currentTemplateName.value = applyTemplate.value.name || '';
    // 列表项已带 previewPic / remark / terminal* 等字段，足够渲染顶部区块，省掉一次详情请求
    currentTemplateDetail.value = { ...(applyTemplate.value as any) };
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
    document.body.append(a);
    a.click();
    a.remove();
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
  input.addEventListener('change', async (e: any) => {
    const file = e.target?.files?.[0];
    if (!file) return;
    try {
      await templateApi.importTemplate(file);
      message.success('模板导入成功');
      loadTemplates();
    } catch (error: any) {
      message.error(error?.message || '导入失败');
    }
  });
  input.click();
}
</script>

<template>
  <Page auto-content-height>
    <div class="template-market">
      <!-- 右侧内容区 -->
      <main class="template-main">
        <!-- 顶部信息栏 -->
        <div class="template-topbar">
          <div class="topbar-info">
            <h2 class="topbar-title">模板管理</h2>
            <span class="topbar-count">共 {{ total }} 个模板</span>
          </div>
          <div class="topbar-actions">
            <Button type="primary" @click="handleImportTemplate">
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

        <!-- 当前生效模板：从候选列表中「抽出来」独立置顶（对标 Shopify 主题页 / WordPress 激活态）
             解决两个问题：① 分页时当前模板不在本页、卡片高亮看不到；② 用户要的是「下一步做什么」而不只是一个状态 -->
        <div v-if="currentTemplateDetail" class="current-panel">
          <div class="current-panel-thumb" @click="handlePreviewCurrentLarge">
            <img
              :src="currentTemplateDetail.previewPic || NO_PREVIEW_IMG"
              :alt="currentTemplateDetail.name"
              loading="lazy"
            />
            <div class="current-panel-thumb-mask">
              <component :is="LucideEye" />
              看大图
            </div>
          </div>

          <div class="current-panel-body">
            <div class="current-panel-eyebrow">
              <span class="current-panel-dot"></span>
              网站当前生效模板
            </div>
            <h3 class="current-panel-name" :title="currentTemplateDetail.name">
              {{ currentTemplateDetail.name || `#${currentTemplateId}` }}
            </h3>
            <div class="current-panel-meta">
              <span class="current-panel-meta-label">访问支持</span>
              <template v-if="deviceBadges(currentTemplateDetail).length > 0">
                <Tag
                  v-for="dev in deviceBadges(currentTemplateDetail)"
                  :key="dev.label"
                  :color="dev.color"
                  size="small"
                >
                  <component
                    :is="dev.icon"
                    style="margin-right: 2px; font-size: 11px"
                  />
                  {{ dev.label }}
                </Tag>
              </template>
              <span v-else class="no-device">未设置</span>
              <span v-if="currentMobileTemplateId" class="current-panel-mobile">
                <component :is="LucideSmartphone" />
                移动端另有专用模板
              </span>
            </div>
            <p class="current-panel-desc">
              {{ currentTemplateDetail.remark || '暂无简介' }}
            </p>
          </div>

          <div class="current-panel-actions">
            <Button type="primary" size="large" @click="openCurrentPagesDrawer">
              <template #icon><component :is="LucideFile" /></template>
              管理页面
            </Button>
            <Button size="large" @click="handlePreviewCurrentLarge">
              查看效果
            </Button>
          </div>
        </div>

        <!-- 卡片网格 -->
        <div class="template-grid-wrapper">
          <!-- 分区标题：把「已生效」和「可切换」在视觉上分开，避免用户把两处当成同一层级 -->
          <div v-if="currentTemplateDetail" class="grid-section-title">
            <span>可切换的模板</span>
            <span class="grid-section-hint">
              点击卡片上的「立即使用」即可切换，切换前建议先「预览模板」
            </span>
          </div>
          <!-- 加载中 -->
          <div v-if="loading" class="template-grid">
            <div v-for="i in 6" :key="i" class="template-card-skeleton">
              <div class="skeleton-image"></div>
              <div class="skeleton-info">
                <Skeleton
                  active
                  :paragraph="{ rows: 2, width: ['80%', '60%'] }"
                  :avatar="{ size: 'small' }"
                />
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
              :class="{ 'is-current': isCurrentTemplate(item) }"
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
                <!-- 状态标记（右上）
                     原本无条件显示绿色「启用」标签，但列表接口本就按 status=1 过滤，
                     这个标签永远不会变成"禁用" —— 纯噪音，且容易被误读成"正在使用"。
                     改为只在非启用时才提示，把右上角还给真正需要告警的信息。 -->
                <div v-if="item.status !== 1" class="card-status-tag">
                  <Tag color="default" size="small">已停用</Tag>
                </div>
                <!-- 移动端专用模板标记（左上）。
                     PC 端「当前使用」不再在此重复标记：底部 40px 状态条 + 卡片描边环 + 顶部生效区块
                     已经表达三次，图上的小药丸既重复又小，属于噪音。 -->
                <div
                  v-if="isCurrentMobileTemplate(item)"
                  class="card-current-badge"
                  title="当前站点移动端正在使用该模板"
                >
                  <component :is="LucideSmartphone" />
                  <span>移动端使用</span>
                </div>
              </div>

              <!-- 信息区域 -->
              <div class="card-info">
                <div class="card-title-row">
                  <h3 class="card-title" :title="item.name">{{ item.name }}</h3>
                  <span class="card-provider">
                    <component :is="LucideUser" class="provider-icon" />
                    官方
                  </span>
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
                        <component
                          :is="dev.icon"
                          style="margin-right: 2px; font-size: 11px"
                        />
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

                <!-- 动作区：主操作定宽放大 + 次级操作收进「更多」菜单
                     对齐主流做法（Shopify 主题卡 / 凡科「使用模板」）：一个全宽主按钮 + 一个 ⋯ 菜单 -->
                <div class="card-actions">
                  <div v-if="isCurrentTemplate(item)" class="card-use-state">
                    <component :is="LucideCheckCircle" />
                    <span>当前使用中</span>
                  </div>
                  <Button
                    v-else
                    class="card-use-btn"
                    type="primary"
                    ghost
                    size="large"
                    :icon="h(LucideTag)"
                    @click="handleApply(item)"
                  >
                    立即使用
                  </Button>

                  <Popover
                    :open="menuOpenId === item.id"
                    placement="bottomRight"
                    trigger="click"
                    @open-change="(v) => handleMenuOpenChange(item.id, v)"
                  >
                    <Button class="card-more-btn" size="large">
                      <component :is="LucideEllipsis" />
                    </Button>
                    <template #content>
                      <!-- 自定义菜单项：antdv 4.x 的 <Menu.Item> 已移除，:items 在 Dropdown overlay
                           中渲染为空（实测），改用 Popover + 自定义节点，视觉走 card-more-* 样式 -->
                      <div class="card-more-menu">
                        <div
                          class="card-more-item"
                          @click="openPagesDrawer(item)"
                        >
                          <IconifyIcon icon="lucide:file" class="size-4" />
                          管理页面
                        </div>
                        <div
                          class="card-more-item"
                          @click="handleEditTemplate(item)"
                        >
                          <IconifyIcon icon="lucide:file-pen-line" class="size-4" />
                          编辑模板信息
                        </div>
                        <div
                          class="card-more-item"
                          @click="handleExport(item)"
                        >
                          <IconifyIcon icon="lucide:download" class="size-4" />
                          导出模板
                        </div>
                        <div class="card-more-divider"></div>
                        <div
                          class="card-more-item card-more-danger"
                          @click="handleDelete(item)"
                        >
                          <IconifyIcon icon="lucide:trash-2" class="size-4" />
                          删除模板
                        </div>
                      </div>
                    </template>
                  </Popover>
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
          ></iframe>
          <div v-else class="preview-iframe-empty">
            <Empty description="暂未设置演示网址" />
          </div>
        </div>

        <!-- 详情信息面板 -->
        <div v-if="previewDetail" class="preview-info-panel">
          <div class="info-header">
            <h3 class="info-title">{{ previewDetail.name }}</h3>
            <Tag
              v-if="isCurrentTemplate(previewTemplate)"
              color="processing"
              class="info-current-tag"
            >
              <component
                :is="LucideCheckCircle"
                style="margin-right: 3px; font-size: 12px"
              />当前使用中
            </Tag>
            <div class="info-device-tags">
              <Tag v-if="previewDetail.terminalPc === 1" color="blue">
                <component
                  :is="LucideMonitor"
                  style="margin-right: 3px; font-size: 12px"
                />电脑端
              </Tag>
              <Tag v-if="previewDetail.terminalMobile === 1" color="green">
                <component
                  :is="LucideSmartphone"
                  style="margin-right: 3px; font-size: 12px"
                />手机端
              </Tag>
              <Tag v-if="previewDetail.terminalIpad === 1" color="orange">
                <component
                  :is="LucideTablet"
                  style="margin-right: 3px; font-size: 12px"
                />平板
              </Tag>
              <Tag v-if="previewDetail.terminalDisplay === 1" color="purple">
                <component
                  :is="LucideDisplay"
                  style="margin-right: 3px; font-size: 12px"
                />展示机
              </Tag>
            </div>
          </div>

          <div class="info-grid">
            <div class="info-item">
              <span class="info-label">模板文件夹</span>
              <span class="info-value">{{
                previewDetail.templateFolder || '—'
              }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">演示网址</span>
              <span class="info-value">
                <a
                  v-if="previewDetail.previewUrl"
                  :href="previewDetail.previewUrl"
                  target="_blank"
                  rel="noopener"
                >
                  {{ previewDetail.previewUrl }}
                </a>
                <span v-else style="color: #999">—</span>
              </span>
            </div>
          </div>

          <div class="info-desc">
            <span class="info-label">简介说明</span>
            <p class="info-desc-text">
              {{ previewDetail.remark || '暂无简介' }}
            </p>
          </div>

          <div class="info-actions">
            <!-- 已是当前模板：不给「使用」按钮，直接给下一步动作（对标 WordPress.com 激活后的引导浮层） -->
            <template v-if="isCurrentTemplate(previewTemplate)">
              <div class="info-current-hint">
                <component :is="LucideCheckCircle" />
                当前站点正在使用该模板
              </div>
              <Button size="large" @click="openCurrentPagesDrawer">
                <template #icon><component :is="LucideFile" /></template>
                管理页面
              </Button>
            </template>
            <template v-else>
              <Button
                class="info-apply-btn"
                type="primary"
                size="large"
                :icon="h(LucideTag)"
                @click="
                  () => {
                    previewVisible = false;
                    previewTemplate && handleApply(previewTemplate);
                  }
                "
              >
                立即使用此模板
              </Button>
            </template>
            <Button size="large" @click="previewVisible = false"> 关闭 </Button>
          </div>
        </div>
      </div>
    </Modal>

    <!-- 切换到网站弹窗：切换是有后果的破坏性操作，必须给出「从什么变成什么」与影响范围 -->
    <Modal
      v-model:open="applyVisible"
      title="切换网站模板"
      width="520px"
      :confirm-loading="applying"
      ok-text="确认切换"
      cancel-text="再想想"
      @ok="confirmApply"
      @cancel="applyVisible = false"
    >
      <div class="apply-modal">
        <div class="apply-switch">
          <div class="apply-switch-side">
            <span class="apply-switch-label">当前使用</span>
            <span class="apply-switch-name is-from">
              {{ currentTemplateName || '未设置' }}
            </span>
          </div>
          <component :is="LucideTag" class="apply-switch-arrow" />
          <div class="apply-switch-side">
            <span class="apply-switch-label">切换为</span>
            <span class="apply-switch-name is-to">
              {{ applyTemplate?.name }}
            </span>
          </div>
        </div>

        <div class="apply-warning">
          <strong>切换后网站前台外观会立即改变</strong>
          <ul>
            <li>新模板的页面会覆盖当前站点正在展示的页面</li>
            <li>若未提前导出备份，原模板的版式配置可能无法完全还原</li>
            <li>建议先点「再想想」回到列表，用「预览模板」确认效果后再切换</li>
          </ul>
        </div>
      </div>
    </Modal>

    <!-- 模板主题编辑抽屉 -->
    <TemplateDrawerInstance />
    <!-- 页面列表抽屉（75%） -->
    <PagesDrawerInstance />
  </Page>
</template>

<style scoped>
/* ========== 主布局 ========== */
.template-market {
  width: 100%;
}

/* ========== 主内容 ========== */
.template-main {
  width: 100%;
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
  gap: 12px;
  align-items: baseline;
}

.topbar-title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  color: rgb(0 0 0 / 88%);
  letter-spacing: -0.5px;
}

.topbar-count {
  font-size: 13px;
  color: rgb(0 0 0 / 45%);
}

/* ========== 当前生效模板（置顶区块） ==========
   语义：这是「状态 + 下一步」，不是候选卡片。所以用主色浅底 + 左侧色条，
   与下方白底候选卡在视觉上分层，避免用户把它当成"再点一次"的按钮。 */
.current-panel {
  display: flex;
  gap: 18px;
  align-items: center;
  padding: 16px 20px 16px 16px;
  margin-bottom: 22px;
  background: #f0f7ff;
  border: 1px solid #bae0ff;
  border-left: 4px solid #1677ff;
  border-radius: 10px;
}

.current-panel-thumb {
  position: relative;
  flex-shrink: 0;
  width: 148px;
  overflow: hidden;
  cursor: pointer;
  aspect-ratio: 16 / 10;
  background: #e6f0fb;
  border-radius: 6px;
}

.current-panel-thumb img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.current-panel-thumb-mask {
  position: absolute;
  inset: 0;
  display: flex;
  gap: 4px;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: #fff;
  background: rgb(0 0 0 / 55%);
  opacity: 0;
  transition: opacity 0.2s ease;
}

.current-panel-thumb:hover .current-panel-thumb-mask {
  opacity: 1;
}

.current-panel-body {
  flex: 1;
  min-width: 0;
}

.current-panel-eyebrow {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 6px;
  font-size: 12px;
  font-weight: 500;
  color: #1677ff;
  letter-spacing: 0.3px;
}

.current-panel-dot {
  position: relative;
  width: 7px;
  height: 7px;
  background: #1677ff;
  border-radius: 50%;
}

/* 呼吸点：表达"正在生效"这一持续状态，与静态标签区分开。
   只做 opacity 动画（不触发重排），并尊重降低动效偏好 */
.current-panel-dot::after {
  position: absolute;
  inset: -3px;
  content: '';
  border: 1px solid #1677ff;
  border-radius: 50%;
  animation: current-pulse 2s ease-out infinite;
}

@keyframes current-pulse {
  0% {
    opacity: 0.7;
    transform: scale(1);
  }

  100% {
    opacity: 0;
    transform: scale(1.9);
  }
}

@media (prefers-reduced-motion: reduce) {
  .current-panel-dot::after {
    animation: none;
  }
}

.current-panel-name {
  margin: 0 0 8px;
  overflow: hidden;
  font-size: 17px;
  font-weight: 600;
  color: rgb(0 0 0 / 88%);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.current-panel-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  font-size: 12px;
  line-height: 1.7;
}

.current-panel-meta-label {
  color: rgb(0 0 0 / 45%);
}

.current-panel-mobile {
  display: inline-flex;
  gap: 3px;
  align-items: center;
  color: #389e0d;
}

.current-panel-desc {
  display: -webkit-box;
  margin: 8px 0 0;
  overflow: hidden;
  font-size: 12px;
  line-height: 1.6;
  color: rgb(0 0 0 / 55%);
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
}

.current-panel-actions {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
  gap: 8px;
  width: 132px;
}

/* ========== 候选列表分区标题 ========== */
.grid-section-title {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: baseline;
  margin-bottom: 14px;
}

.grid-section-title > span:first-child {
  font-size: 15px;
  font-weight: 600;
  color: rgb(0 0 0 / 78%);
}

.grid-section-hint {
  font-size: 12px;
  color: rgb(0 0 0 / 45%);
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
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #fff;
  border: 1px solid #f0f0f0;
  border-radius: 10px;
  box-shadow:
    0 1px 2px rgb(0 0 0 / 4%),
    0 1px 6px rgb(0 0 0 / 4%);
  transition:
    transform 0.25s ease,
    box-shadow 0.25s ease;
}

.template-card:hover {
  border-color: transparent;
  box-shadow: 0 8px 24px rgb(0 0 0 / 12%);
  transform: translateY(-4px);
}

/* 当前使用中的卡片：主色描边环
   ① 用 box-shadow 而非 border-width，避免 2px 边框挤压内容导致布局跳动
   ② 强度对齐项目既有惯例（见 site/settings.vue 的 .tpl-card.active） */
.template-card.is-current {
  border-color: #1677ff;
  box-shadow:
    0 0 0 3px rgb(22 119 255 / 15%),
    0 4px 14px rgb(22 119 255 / 10%);
}

.template-card.is-current:hover {
  border-color: #1677ff;
  box-shadow:
    0 0 0 3px rgb(22 119 255 / 24%),
    0 10px 26px rgb(22 119 255 / 18%);
  transform: translateY(-4px);
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
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(
    to bottom,
    rgb(0 0 0 / 10%) 0%,
    rgb(0 0 0 / 60%) 100%
  );
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

/* 移动端专用模板标记（左上角实心药丸，置于悬停遮罩之上，始终可见） */
.card-current-badge {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 2;
  display: inline-flex;
  gap: 4px;
  align-items: center;
  padding: 3px 9px;
  font-size: 12px;
  font-weight: 600;
  line-height: 18px;
  color: #fff;
  background: #389e0d;
  border-radius: 999px;
}

.card-current-badge > :first-child {
  font-size: 13px;
}

/* 卡片动作区：全宽主按钮 + ⋯ 菜单
   之前的做法是 6 个 size="small" 图标按钮挤在一行（间距 6px），
   最关键的「应用到网站」还是 icon-only 的 link 按钮 —— 主次完全倒挂。 */
.card-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  padding-top: 12px;
  margin-top: auto;
  border-top: 1px solid #f5f5f5;
}

/* 主操作：占满剩余宽度、40px 高，一眼可辨、点击热区足够。
   用 primary + ghost（蓝边蓝字白底）而不是实心蓝：
   一页 9~12 张卡片，9 个实心蓝按钮会变成一堵蓝墙，反而让"当前使用"的蓝色状态失去分量。 */
.card-use-btn {
  flex: 1;
  height: 40px;
  font-size: 14px;
}

/* 已是当前模板：不给"再点一次"的入口。
   做成无边框的实心浅蓝状态条（对比 CTA 的白底蓝边），一眼能区分"状态"与"按钮" */
.card-use-state {
  display: inline-flex;
  flex: 1;
  gap: 6px;
  align-items: center;
  justify-content: center;
  height: 40px;
  font-size: 14px;
  font-weight: 500;
  color: #1677ff;
  cursor: default;
  background: #e6f4ff;
  border-radius: 6px;
}

.card-use-state > :first-child {
  font-size: 16px;
}

/* 次级操作入口：固定方形，与前一个按钮同高 */
.card-more-btn {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  padding: 0;
  font-size: 16px;
  color: rgb(0 0 0 / 55%);
}

.card-more-menu {
  min-width: 168px;
}

/* 菜单项 = 图标 + 文字，强制同一行、不换行。
   ⚠️ 类名前缀是 `ant-dropdown-menu-*`，不是 `ant-menu-*`：
   Menu 一旦放进 Dropdown，antd 会给它套一层 dropdown 前缀
   （DOM 实测：<li class="ant-dropdown-menu-item"> + .ant-dropdown-menu-title-content
   + .ant-dropdown-menu-item-icon）。两套都写上，避免换个容器就失效。
   Dropdown 浮层会被 teleport 到 body，但作用域属性跟着模板元素一起走，
   所以 .card-more-menu 上的 scoped 规则对内部节点依然生效，:deep 可以照常用。 */
.card-more-menu :deep(.ant-dropdown-menu-item),
.card-more-menu :deep(.ant-menu-item) {
  white-space: nowrap;
}

.card-more-menu :deep(.ant-dropdown-menu-title-content),
.card-more-menu :deep(.ant-menu-title-content) {
  white-space: nowrap;
}

/* 图标由 Menu.Item 的 icon 属性定位，这里只统一色调（与卡片内次级图标一致）；
   危险项（删除）必须放行，否则会被染成灰图标、丢掉 antd 的红色语义 */
.card-more-menu :deep(.ant-dropdown-menu-item-icon),
.card-more-menu :deep(.ant-menu-item-icon) {
  color: rgb(0 0 0 / 45%);
}

.card-more-menu :deep(.ant-dropdown-menu-item-danger .ant-dropdown-menu-item-icon),
.card-more-menu :deep(.ant-menu-item-danger .ant-menu-item-icon) {
  color: inherit;
}

.card-more-menu :deep(.ant-menu-item-danger .ant-menu-item-icon) {
  color: inherit;
}

/* 信息区域 */
.card-info {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 8px;
  padding: 14px 16px 12px;
}

.card-title-row {
  display: flex;
  gap: 8px;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: 2px;
}

.card-title {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 15px;
  font-weight: 600;
  color: rgb(0 0 0 / 88%);
  white-space: nowrap;
}

/* 提供方：从底部独立一行降级为标题右侧的弱信息，把底部整行让给主操作 */
.card-provider {
  display: flex;
  flex-shrink: 0;
  gap: 3px;
  align-items: center;
  font-size: 12px;
  color: rgb(0 0 0 / 40%);
}

.provider-icon {
  font-size: 12px;
}

.card-device-row,
.card-desc-row {
  display: flex;
  gap: 6px;
  align-items: flex-start;
  font-size: 12px;
  line-height: 1.6;
}

.device-label,
.desc-label {
  flex-shrink: 0;
  color: rgb(0 0 0 / 45%);
}

.device-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.no-device {
  color: rgb(0 0 0 / 35%);
}

.desc-text {
  display: -webkit-box;
  overflow: hidden;
  -webkit-line-clamp: 2;
  color: rgb(0 0 0 / 70%);
  -webkit-box-orient: vertical;
}

/* 骨架屏 */
.template-card-skeleton {
  overflow: hidden;
  background: #fff;
  border-radius: 10px;
  box-shadow: 0 1px 2px rgb(0 0 0 / 4%);
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
  overflow: hidden;
  border-radius: 12px;
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
  display: flex;
  flex: 1;
  align-items: center;
  justify-content: center;
  overflow: auto;
  background: #f0f2f5;
}

.preview-large-img {
  display: block;
  max-width: 100%;
  max-height: 78vh;
  object-fit: contain;
}

.preview-iframe {
  width: 100%;
  height: 78vh;
  background: #fff;
  border: none;
}

.preview-iframe-empty {
  padding: 80px 40px;
}

/* 详情面板 */
.preview-info-panel {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
  width: 320px;
  padding: 24px 20px;
  background: #fff;
  border-left: 1px solid #f0f0f0;
}

.info-header {
  margin-bottom: 16px;
}

.info-title {
  margin: 0 0 10px;
  font-size: 18px;
  font-weight: 600;
  color: rgb(0 0 0 / 88%);
}

/* 预览面板：当前使用中标记 */
.info-current-tag {
  margin-bottom: 8px;
}

.info-current-hint {
  display: inline-flex;
  flex: 1;
  gap: 6px;
  align-items: center;
  justify-content: center;
  height: 40px;
  font-size: 14px;
  color: #1677ff;
  background: #e6f4ff;
  border: 1px solid #91caff;
  border-radius: 6px;
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
  color: rgb(0 0 0 / 45%);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.info-value {
  font-size: 14px;
  color: rgb(0 0 0 / 88%);
  word-break: break-all;
}

.info-desc {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 16px;
}

.info-desc-text {
  margin: 0;
  font-size: 13px;
  line-height: 1.7;
  color: rgb(0 0 0 / 75%);
}

.info-actions {
  display: flex;
  gap: 10px;
  padding-top: 16px;
  border-top: 1px solid #f0f0f0;
}

/* 「立即使用此模板」占满剩余宽度：预览场景下的最终决策按钮，必须是面板里最强的元素 */
.info-apply-btn {
  flex: 1;
}

/* ========== 切换确认弹窗 ========== */
.apply-modal {
  padding: 4px 0;
}

/* 变更前后对照：把「从什么变成什么」讲清楚，比一句"确认应用吗"有效得多 */
.apply-switch {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 14px;
  margin-bottom: 16px;
  background: #fafafa;
  border-radius: 8px;
}

.apply-switch-side {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.apply-switch-label {
  font-size: 12px;
  color: rgb(0 0 0 / 45%);
}

.apply-switch-name {
  overflow: hidden;
  font-size: 14px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.apply-switch-name.is-from {
  color: rgb(0 0 0 / 55%);
  text-decoration: line-through;
  text-decoration-color: rgb(0 0 0 / 30%);
}

.apply-switch-name.is-to {
  color: #1677ff;
}

.apply-switch-arrow {
  flex-shrink: 0;
  font-size: 18px;
  color: rgb(0 0 0 / 25%);
}

/* 影响范围提示：切换不可逆，必须显式告知 */
.apply-warning {
  padding: 12px 14px;
  font-size: 12px;
  line-height: 1.7;
  color: rgb(0 0 0 / 65%);
  background: #fffbe6;
  border: 1px solid #ffe58f;
  border-radius: 8px;
}

.apply-warning strong {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  color: #d46b08;
}

.apply-warning ul {
  padding-left: 18px;
  margin: 0;
}

.apply-warning li + li {
  margin-top: 2px;
}

/* ===== 「⋯」Popover 自定义菜单项 ===== */
.card-more-menu {
  min-width: 150px;
}

.card-more-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  white-space: nowrap;
}

.card-more-item:hover {
  background: hsl(var(--primary) / 10%);
  color: hsl(var(--primary));
}

.card-more-danger {
  color: #e54545;
}

.card-more-danger:hover {
  background: rgb(229 69 69 / 10%);
  color: #e54545;
}

.card-more-divider {
  height: 1px;
  margin: 4px 0;
  background: hsl(var(--foreground) / 12%);
}
</style>
