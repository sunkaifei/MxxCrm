<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';
import { Page, useVbenDrawer } from '@vben/common-ui';
import {
  LucideUpload,
  LucideTrash2,
  LucideEye,
  LucideImage,
  LucideFileText,
  LucideFilm,
  LucideFile,
  LucideSearch,
  LucideFilePenLine,
} from '@vben/icons';

import {
  Button,
  Empty,
  Input,
  Modal,
  Pagination,
  Popconfirm,
  Select,
  Tag,
  Tree,
  Upload,
  message,
} from 'ant-design-vue';
import type { TreeProps } from 'ant-design-vue';

import {
  addMediaApi,
  deleteMediaApi,
  getMediaCategoryAllApi,
  getMediaListApi,
} from '#/api';
import { uploadFileApi } from '#/api/core/attachment/file';
import MediaDrawer from './drawer.vue';

const SelectOption = Select.Option;

// --- 状态 ---
const searchKeyword = ref('');
const selectedFileType = ref<number | undefined>(undefined);
const selectedCategoryId = ref<number | null>(null);
const selectedIds = ref<Set<number>>(new Set());
const treeData = ref<any[]>([]);
const mediaList = ref<any[]>([]);
const loading = ref(false);
const pagination = ref({ current: 1, pageSize: 24, total: 0 });

// 预览弹窗
const previewVisible = ref(false);
const previewImage = ref('');
const previewTitle = ref('');

// 文件类型映射：1=图片, 2=视频, 3=文档, 4=音频, 5=其他
const fileTypeOptions = [
  { value: 1, label: '图片' },
  { value: 2, label: '视频' },
  { value: 3, label: '文档' },
  { value: 4, label: '音频' },
  { value: 5, label: '其他' },
];

const fileTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: '图片', color: 'blue' },
  2: { label: '视频', color: 'purple' },
  3: { label: '文档', color: 'green' },
  4: { label: '音频', color: 'orange' },
  5: { label: '其他', color: 'default' },
};

const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg', 'ico'];

// --- 计算属性 ---
const isImage = (item: any) => {
  if (item.fileType === 1) return true;
  const ext = item.fileExt?.toLowerCase();
  return imageExtensions.includes(ext);
};

const getFileIcon = (item: any) => {
  if (item.fileType === 1 || isImage(item)) return LucideImage;
  if (item.fileType === 2) return LucideFilm;
  if (item.fileType === 3) return LucideFileText;
  return LucideFile;
};

const getThumbUrl = (item: any) => {
  if (item.thumbMedium) return item.thumbMedium;
  if (item.thumbSmall) return item.thumbSmall;
  if (item.thumbLarge) return item.thumbLarge;
  if (isImage(item)) return item.fileUrl;
  return '';
};

const formatSize = (size?: number) => {
  if (!size) return '-';
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
  return `${(size / (1024 * 1024)).toFixed(2)} MB`;
};

const hasSelected = computed(() => selectedIds.value.size > 0);

// --- 方法 ---
const buildTreeData = (list: any[]): any[] =>
  list.map((item) => ({
    title: item.categoryName,
    key: item.id,
    children: item.children?.length ? buildTreeData(item.children) : undefined,
  }));

const loadCategoryTree = async () => {
  try {
    const result: any = await getMediaCategoryAllApi();
    const list = Array.isArray(result) ? result : result?.data || [];
    treeData.value = buildTreeData(list);
  } catch (e) {
    console.error('加载分类树失败', e);
    treeData.value = [];
  }
};

const loadMediaList = async () => {
  loading.value = true;
  try {
    const params: any = {
      page: pagination.value.current,
      pageSize: pagination.value.pageSize,
      keywords: searchKeyword.value || undefined,
      fileType: selectedFileType.value || undefined,
      categoryId: selectedCategoryId.value || undefined,
    };
    const result: any = await getMediaListApi(params);
    const list = result?.items || result?.list || result?.rows || [];
    mediaList.value = list;
    pagination.value.total = result?.total || result?.count || 0;
  } catch (e) {
    console.error('加载媒体列表失败', e);
    mediaList.value = [];
  } finally {
    loading.value = false;
  }
};

const onSelectCategory: TreeProps['onSelect'] = (selectedKeys) => {
  selectedCategoryId.value = selectedKeys?.[0] ? Number(selectedKeys[0]) : null;
  pagination.value.current = 1;
  loadMediaList();
};

const handleSearch = () => {
  pagination.value.current = 1;
  loadMediaList();
};

const handleReset = () => {
  searchKeyword.value = '';
  selectedFileType.value = undefined;
  selectedCategoryId.value = null;
  pagination.value.current = 1;
  loadMediaList();
};

const handlePageChange = (page: number, pageSize: number) => {
  pagination.value.current = page;
  pagination.value.pageSize = pageSize;
  loadMediaList();
};

const handlePreview = (item: any) => {
  if (!isImage(item)) return;
  previewTitle.value = item.originalName || item.title || '';
  previewImage.value = item.fileUrl || '';
  previewVisible.value = true;
};

const toggleSelect = (id: number) => {
  const next = new Set(selectedIds.value);
  if (next.has(id)) {
    next.delete(id);
  } else {
    next.add(id);
  }
  selectedIds.value = next;
};

const handleBatchDelete = async () => {
  if (selectedIds.value.size === 0) return;
  try {
    await deleteMediaApi(Array.from(selectedIds.value));
    message.success('删除成功');
    selectedIds.value = new Set();
    loadMediaList();
  } catch (e) {
    console.error(e);
  }
};

async function handleDelete(item: any) {
  try {
    await deleteMediaApi([item.id]);
    message.success('删除成功');
    loadMediaList();
  } catch (e) {
    console.error(e);
  }
}

// --- 上传 ---
async function uploadFile(options: any) {
  const { file, onSuccess, onError, onProgress } = options;
  try {
    onProgress?.({ percent: 30 });
    const res: any = await uploadFileApi(file, 'website-media');
    const url = res?.data?.url || res?.url;
    onProgress?.({ percent: 100 });
    onSuccess?.(res, file);

    // 上传成功后调用 addMediaApi 创建媒体记录
    if (url) {
      const fileName = file.name || '';
      const ext = fileName.split('.').pop()?.toLowerCase() || '';
      await addMediaApi({
        originalName: fileName,
        fileUrl: url,
        filePath: res?.data?.path || res?.path || url,
        fileExt: ext,
        fileSize: file.size,
        fileType: imageExtensions.includes(ext) ? 1 : 5,
        mimeType: file.type,
        categoryId: selectedCategoryId.value || undefined,
      });
    }
    message.success('上传成功');
    loadMediaList();
  } catch (e: any) {
    onError?.(e);
    message.error(e?.message || '上传失败');
  }
}

// --- 抽屉 ---
const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: MediaDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      loadMediaList();
    }
  },
});

function handleEdit(item: any) {
  drawerApi.setData({
    create: false,
    row: item,
  });
  drawerApi.open();
}

// --- 生命周期 ---
onMounted(() => {
  loadCategoryTree();
  loadMediaList();
});
</script>

<template>
  <Page auto-content-height class="media-page">
    <div class="media-layout">
      <!-- ===== 左侧分类树 ===== -->
      <aside class="aside-bar">
        <div class="aside-header">
          <h3 class="aside-title">媒体分类</h3>
        </div>
        <div class="aside-tree">
          <Tree
            :tree-data="treeData"
            :default-expand-all="true"
            :selected-keys="selectedCategoryId ? [selectedCategoryId] : []"
            @select="onSelectCategory"
          />
          <div v-if="treeData.length === 0" class="aside-empty">
            <span>暂无分类</span>
          </div>
        </div>
      </aside>

      <!-- ===== 右侧主区域 ===== -->
      <main class="main-area">
        <!-- 工具栏 -->
        <div class="toolbar">
          <div class="toolbar-top">
            <div class="toolbar-info">
              <span class="info-text">
                共 <strong>{{ pagination.total }}</strong> 项
              </span>
              <Tag v-if="selectedIds.size > 0" color="blue" class="ml-3">
                已选 {{ selectedIds.size }} 项
              </Tag>
            </div>

            <div class="toolbar-actions">
              <Popconfirm
                v-if="hasSelected"
                title="确定删除选中的文件吗？"
                @confirm="handleBatchDelete"
              >
                <Button danger size="small">
                  <template #icon>
                    <component :is="LucideTrash2" />
                  </template>
                  批量删除 ({{ selectedIds.size }})
                </Button>
              </Popconfirm>

              <Upload
                :custom-request="uploadFile"
                :show-upload-list="false"
                multiple
                accept="image/*,video/*,audio/*,.pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.txt"
              >
                <Button type="primary" size="small">
                  <template #icon>
                    <component :is="LucideUpload" />
                  </template>
                  上传媒体
                </Button>
              </Upload>
            </div>
          </div>

          <div class="toolbar-bottom">
            <Input
              v-model:value="searchKeyword"
              placeholder="搜索文件名、标题…"
              class="filter-search"
              allow-clear
              @press-enter="handleSearch"
              @change="handleSearch"
            >
              <template #prefix>
                <component
                  :is="LucideSearch"
                  style="color: #bfbfbf; font-size: 14px"
                />
              </template>
            </Input>

            <Select
              v-model:value="selectedFileType"
              placeholder="文件类型"
              class="filter-select"
              allow-clear
              @change="handleSearch"
            >
              <SelectOption
                v-for="opt in fileTypeOptions"
                :key="opt.value"
                :value="opt.value"
              >
                {{ opt.label }}
              </SelectOption>
            </Select>

            <Button type="primary" size="small" @click="handleSearch">
              搜索
            </Button>
            <Button size="small" @click="handleReset">重置</Button>
          </div>
        </div>

        <!-- 内容区 -->
        <div class="content-area">
          <div class="content-card" :class="{ 'is-empty': mediaList.length === 0 }">
            <div v-if="!loading && mediaList.length === 0" class="empty-wrapper">
              <Empty description="暂无媒体文件" />
            </div>

            <!-- 网格视图 -->
            <div v-if="mediaList.length > 0" class="grid-wrap">
              <div
                v-for="item in mediaList"
                :key="item.id"
                class="grid-card"
                :class="{ 'is-selected': selectedIds.has(item.id) }"
              >
                <!-- 缩略图 -->
                <div class="gc-thumb" @click="handlePreview(item)">
                  <img
                    v-if="getThumbUrl(item)"
                    :src="getThumbUrl(item)"
                    :alt="item.originalName || item.title"
                    class="gc-img"
                    loading="lazy"
                  />
                  <div v-else class="gc-placeholder">
                    <component
                      :is="getFileIcon(item)"
                      class="gc-placeholder-icon"
                    />
                  </div>
                  <div v-if="isImage(item)" class="gc-hover-mask">
                    <Button size="small" ghost @click.stop="handlePreview(item)">
                      <template #icon>
                        <component :is="LucideEye" />
                      </template>
                      预览
                    </Button>
                  </div>
                  <div class="gc-check" @click.stop="toggleSelect(item.id)">
                    <input
                      type="checkbox"
                      :checked="selectedIds.has(item.id)"
                      class="gc-checkbox"
                    />
                  </div>
                </div>

                <!-- 信息 -->
                <div class="gc-body">
                  <div
                    class="gc-name"
                    :title="item.originalName || item.title"
                    @click="handleEdit(item)"
                  >
                    {{ item.originalName || item.title || '-' }}
                  </div>
                  <div class="gc-meta">
                    <span class="gc-size">{{ formatSize(item.fileSize) }}</span>
                    <span v-if="item.fileExt" class="gc-ext">
                      {{ (item.fileExt || '').toUpperCase() }}
                    </span>
                  </div>
                  <div class="gc-footer">
                    <Tag
                      :color="fileTypeMap[item.fileType]?.color || 'default'"
                      class="gc-tag"
                    >
                      {{ fileTypeMap[item.fileType]?.label || '未知' }}
                    </Tag>
                    <div class="gc-actions">
                      <Button
                        type="text"
                        size="small"
                        title="编辑"
                        @click.stop="handleEdit(item)"
                      >
                        <template #icon>
                          <component :is="LucideFilePenLine" style="font-size: 14px" />
                        </template>
                      </Button>
                      <Popconfirm
                        title="确定删除？"
                        @confirm="handleDelete(item)"
                      >
                        <Button
                          type="text"
                          size="small"
                          danger
                          title="删除"
                          @click.stop
                        >
                          <template #icon>
                            <component
                              :is="LucideTrash2"
                              style="font-size: 14px"
                            />
                          </template>
                        </Button>
                      </Popconfirm>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 分页 -->
            <div v-if="mediaList.length > 0" class="pagination-bar">
              <span class="page-total">
                共 {{ pagination.total }} 项
              </span>
              <Pagination
                :current="pagination.current"
                :page-size="pagination.pageSize"
                :total="pagination.total"
                show-size-changer
                show-quick-jumper
                size="small"
                @change="handlePageChange"
                @show-size-change="handlePageChange"
              />
            </div>
          </div>
        </div>
      </main>
    </div>

    <!-- 预览弹窗 -->
    <Modal
      v-model:open="previewVisible"
      :title="previewTitle"
      :footer="null"
      width="auto"
      :centered="true"
      :mask-closable="true"
      class="preview-modal"
    >
      <img :src="previewImage" :alt="previewTitle" class="preview-img" />
    </Modal>

    <Drawer />
  </Page>
</template>

<style>
.media-page {
  height: 100%;
  padding: 0 !important;
  background: #f5f5f5;
}

.media-layout {
  display: flex;
  min-height: calc(100vh - 110px);
}

/* ---- 侧栏 ---- */
.aside-bar {
  width: 220px;
  background: #fff;
  border-right: 1px solid #e8e8e8;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.aside-header {
  padding: 16px 16px 12px;
  border-bottom: 1px solid #f0f0f0;
}
.aside-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #1a1a1a;
}

.aside-tree {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
}

.aside-empty {
  text-align: center;
  padding: 40px 0;
  color: #bfbfbf;
}

/* ---- 主区域 ---- */
.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #f5f5f5;
}

/* ---- 工具栏 ---- */
.toolbar {
  background: #fff;
  border-bottom: 1px solid #f0f0f0;
  flex-shrink: 0;
}

.toolbar-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px 10px;
}

.toolbar-info {
  display: flex;
  align-items: center;
  font-size: 13px;
  color: #595959;
}
.info-text strong {
  color: #1677ff;
  margin: 0 2px;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.toolbar-bottom {
  padding: 0 20px 14px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.filter-search {
  width: 320px;
}
.filter-select {
  width: 140px;
}

/* ---- 内容区 ---- */
.content-area {
  flex: 1;
}

.content-card {
  padding: 15px;
  border: 1px solid #f0f0f0;
  width: 100%;
  min-height: 100%;
  box-sizing: border-box;
  overflow: visible;
}
.content-card.is-empty {
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-wrapper {
  display: flex;
  justify-content: center;
  padding: 60px 0;
}

/* ===== 网格视图 ===== */
.grid-wrap {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
}

.grid-card {
  position: relative;
  background: #fff;
  border-radius: 8px;
  border: 1px solid #f0f0f0;
  cursor: pointer;
  transition: all 0.2s;
}
.grid-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  border-color: #d9d9d9;
}
.grid-card.is-selected {
  border-color: #1677ff;
  box-shadow: 0 0 0 2px rgba(22, 119, 255, 0.15);
}

.gc-thumb {
  position: relative;
  width: 100%;
  padding-top: 75%;
  background: #fafafa;
  overflow: hidden;
  border-radius: 8px 8px 0 0;
}
.gc-img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.gc-placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #fafafa;
}
.gc-placeholder-icon {
  font-size: 40px;
  color: #bfbfbf;
}
.gc-hover-mask {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s;
}
.grid-card:hover .gc-hover-mask {
  opacity: 1;
}
.gc-check {
  position: absolute;
  top: 8px;
  left: 8px;
  background: rgba(255, 255, 255, 0.85);
  border-radius: 4px;
  padding: 2px 4px;
  line-height: 0;
}
.gc-checkbox {
  cursor: pointer;
  width: 14px;
  height: 14px;
}

.gc-body {
  padding: 10px 12px;
}
.gc-name {
  font-size: 13px;
  font-weight: 500;
  color: #262626;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}
.gc-name:hover {
  color: #1677ff;
}
.gc-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  font-size: 12px;
  color: #8c8c8c;
}
.gc-ext {
  background: #f5f5f5;
  padding: 0 6px;
  border-radius: 3px;
  font-size: 11px;
  color: #595959;
}
.gc-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #f5f5f5;
}
.gc-tag {
  font-size: 11px;
  line-height: 18px;
  padding: 0 6px;
  margin: 0;
}
.gc-actions {
  display: flex;
  gap: 2px;
}

/* ---- 分页 ---- */
.pagination-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 16px;
  padding: 12px 4px 4px;
  border-top: 1px solid #f0f0f0;
  background: #fafafa;
  border-radius: 0 0 6px 6px;
}
.page-total {
  font-size: 13px;
  color: #595959;
  padding: 4px 10px;
  background: #fff;
  border: 1px solid #f0f0f0;
  border-radius: 4px;
  line-height: 20px;
}

/* ---- 预览弹窗 ---- */
.preview-modal .ant-modal-body {
  padding: 0;
  line-height: 0;
}
.preview-img {
  max-width: 90vw;
  max-height: 85vh;
  display: block;
}

.ml-3 {
  margin-left: 12px;
}
</style>
