<script lang="ts" setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { Page } from '@vben/common-ui';
import {
  LucideUpload,
  LucideGrid3x3,
  LucideList,
  LucideTrash2,
  LucideEye,
  LucideDownload,
  LucideImage,
  LucideFileText,
  LucideFilm,
  LucideFile,
  LucideSearch,
} from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Input,
  Tree,
  Modal,
  message,
  Popconfirm,
  Popover,
  Tooltip,
  Empty,
  Pagination,
  Tag,
  Select,
  Upload,
  Checkbox,
} from 'ant-design-vue';
import type { TreeProps } from 'ant-design-vue';

const SelectOption = Select.Option;

import {
  deleteFileApi,
  downloadFileApi,
  getFileListApi,
  getAttachmentCategoryTreeApi,
  uploadFileApi,
} from '#/api';
import { $t } from '#/locales';

// --- State ---
const viewMode = ref<'grid' | 'list'>('grid');
const searchKeyword = ref('');
const searchEntityType = ref<string | undefined>(undefined);
const selectedFileType = ref('all');
const selectedCategoryId = ref<number | null>(null);
const selectedIds = ref<Set<number>>(new Set());
const treeData = ref<any[]>([]);
const fileList = ref<any[]>([]);
const loading = ref(false);
const pagination = ref({ current: 1, pageSize: 24, total: 0 });

const previewVisible = ref(false);
const previewImage = ref('');
const previewTitle = ref('');
const previewBlobUrl = ref<string | null>(null);

const uploadPopover = ref(false);
const uploadEntityType = ref('common');

const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg', 'ico'];
const docExtensions = ['pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'txt', 'md'];
const videoExtensions = ['mp4', 'avi', 'mov', 'wmv', 'flv', 'mkv', 'webm'];

const entityTypeOptions = [
  { value: 'product', color: 'blue' },
  { value: 'avatar', color: 'cyan' },
  { value: 'contract', color: 'red' },
  { value: 'invoice', color: 'orange' },
  { value: 'quotation', color: 'purple' },
  { value: 'payment', color: 'green' },
  { value: 'common', color: 'default' },
];

const fileTypeOptions = [
  { value: 'all', label: '全部文件' },
  { value: 'image', label: '图片' },
  { value: 'doc', label: '文档' },
  { value: 'video', label: '视频' },
  { value: 'other', label: '其他' },
];

// --- Computed ---
const getEntityTypeColor = (type: string) =>
  entityTypeOptions.find((e) => e.value === type)?.color || 'default';

const getEntityTypeLabel = (type: string) => {
  if (!type) return '-';
  return $t(`page.attachment.file.entityTypes.${type}`);
};

const isImage = (ext: string) => imageExtensions.includes(ext?.toLowerCase());
const isDoc = (ext: string) => docExtensions.includes(ext?.toLowerCase());
const isVideo = (ext: string) => videoExtensions.includes(ext?.toLowerCase());

const getFileIcon = (item: any) => {
  const ext = item.ext?.toLowerCase();
  if (imageExtensions.includes(ext)) return LucideImage;
  if (docExtensions.includes(ext)) return LucideFileText;
  if (videoExtensions.includes(ext)) return LucideFilm;
  return LucideFile;
};

const isItemPublic = (item: any) => !!(item.isPublic ?? item.is_public);
const getItemEntityType = (item: any) => item.entityType || item.entity_type || '';
const getItemUploadedBy = (item: any) => item.uploadedName || item.uploaded_name || item.uploadedBy || item.uploaded_by || '-';

const displayList = computed(() => {
  let list = fileList.value;
  if (selectedFileType.value === 'image') {
    list = list.filter((item) => isImage(item.ext));
  } else if (selectedFileType.value === 'doc') {
    list = list.filter((item) => isDoc(item.ext));
  } else if (selectedFileType.value === 'video') {
    list = list.filter((item) => isVideo(item.ext));
  } else if (selectedFileType.value === 'other') {
    list = list.filter((item) => !isImage(item.ext) && !isDoc(item.ext) && !isVideo(item.ext));
  }
  return list;
});

const statsData = computed(() => {
  const all = fileList.value;
  return {
    total: all.length,
    images: all.filter((f: any) => isImage(f.ext)).length,
    docs: all.filter((f: any) => isDoc(f.ext)).length,
    videos: all.filter((f: any) => isVideo(f.ext)).length,
    totalSize: all.reduce((sum: number, f: any) => sum + (f.size || 0), 0),
  };
});

const formatSize = (size: number) => {
  if (!size) return '-';
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
  return `${(size / (1024 * 1024)).toFixed(2)} MB`;
};

const formatTotalSize = (bytes: number) => {
  if (!bytes) return '0 B';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};

// --- Methods ---
const loadCategoryTree = async () => {
  try {
    const result: any = await getAttachmentCategoryTreeApi();
    const list = Array.isArray(result) ? result : [];
    treeData.value = list.map((item: any) => ({
      title: item.label,
      key: item.value,
      children: item.children?.length ? buildTreeData(item.children) : undefined,
    }));
  } catch (e) {
    console.error('加载分类树失败', e);
  }
};

const buildTreeData = (list: any[]): any[] =>
  list.map((item) => ({
    title: item.label,
    key: item.value,
    children: item.children?.length ? buildTreeData(item.children) : undefined,
  }));

const loadFileList = async () => {
  loading.value = true;
  try {
    const params: any = {
      pageNum: pagination.value.current,
      pageSize: pagination.value.pageSize,
      name: searchKeyword.value || undefined,
      typeId: selectedCategoryId.value || undefined,
      entityType: searchEntityType.value || undefined,
    };
    const result: any = await getFileListApi(params);
    fileList.value = result?.items || [];
    pagination.value.total = result?.total || 0;
  } finally {
    loading.value = false;
  }
};

const onSelectCategory: TreeProps['onSelect'] = (selectedKeys) => {
  selectedCategoryId.value = selectedKeys?.[0] ? Number(selectedKeys[0]) : null;
  pagination.value.current = 1;
  loadFileList();
};

const handleSearch = () => {
  pagination.value.current = 1;
  loadFileList();
};

const handleReset = () => {
  searchKeyword.value = '';
  searchEntityType.value = undefined;
  selectedFileType.value = 'all';
  selectedCategoryId.value = null;
  pagination.value.current = 1;
  loadFileList();
};

const selectFileType = (type: string) => {
  selectedFileType.value = type;
};

const handlePageChange = (page: number, pageSize: number) => {
  pagination.value.current = page;
  pagination.value.pageSize = pageSize;
  loadFileList();
};

const revokePreviewBlobUrl = () => {
  if (previewBlobUrl.value) {
    URL.revokeObjectURL(previewBlobUrl.value);
    previewBlobUrl.value = null;
  }
};

const handlePreview = async (item: any) => {
  if (!isImage(item.ext)) return;
  previewTitle.value = item.originalName || item.name || item.original_name;
  if (isItemPublic(item)) {
    revokePreviewBlobUrl();
    previewImage.value = item.uploadUrl || item.upload_url;
    previewVisible.value = true;
  } else {
    try {
      const res: any = await downloadFileApi(item.id, 'preview');
      const blob = res instanceof Blob ? res : new Blob([res]);
      revokePreviewBlobUrl();
      const url = URL.createObjectURL(blob);
      previewBlobUrl.value = url;
      previewImage.value = url;
      previewVisible.value = true;
    } catch (e) {
      message.error($t('page.attachment.file.message.previewFail'));
    }
  }
};

const handlePreviewClose = () => {
  revokePreviewBlobUrl();
  previewImage.value = '';
};

const handleDownload = async (item: any) => {
  try {
    const res: any = await downloadFileApi(item.id, 'download');
    const blob = res instanceof Blob ? res : new Blob([res]);
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = item.originalName || item.name || `file_${item.id}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  } catch (e) {
    message.error($t('page.attachment.file.message.downloadFail'));
  }
};

const handleDelete = async (item: any) => {
  try {
    await deleteFileApi([item.id]);
    message.success($t('ui.notification.delete_success'));
    loadFileList();
  } catch (e) {
    console.error(e);
  }
};

const handleBatchDelete = async () => {
  if (selectedIds.value.size === 0) return;
  try {
    await deleteFileApi(Array.from(selectedIds.value));
    message.success($t('ui.notification.delete_success'));
    selectedIds.value = new Set();
    loadFileList();
  } catch (e) {
    console.error(e);
  }
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

// --- Upload ---
const uploadFile = async (options: any) => {
  const { file, onSuccess, onError, onProgress } = options;
  try {
    onProgress?.({ percent: 30 });
    const result: any = await uploadFileApi(file, uploadEntityType.value);
    onProgress?.({ percent: 100 });
    onSuccess?.(result, file);
    message.success('上传成功');
    uploadPopover.value = false;
    loadFileList();
  } catch (e: any) {
    onError?.(e);
    message.error(e?.message || '上传失败');
  }
};

// --- Lifecycle ---
onMounted(() => {
  loadCategoryTree();
  loadFileList();
});

onBeforeUnmount(() => {
  revokePreviewBlobUrl();
});
</script>

<template>
  <Page auto-content-height class="attachment-page">
    <div class="attachment-layout">
      <!-- ===== Sidebar ===== -->
      <aside class="aside-bar">
        <div class="aside-header">
          <h3 class="aside-title">{{ $t('page.attachment.file.search.category') }}</h3>
        </div>
        <div class="aside-tree">
          <Tree
            :tree-data="treeData"
            :default-expand-all="true"
            :selected-keys="selectedCategoryId ? [String(selectedCategoryId)] : []"
            @select="onSelectCategory"
          />
          <div v-if="treeData.length === 0" class="aside-empty">
            <span>-</span>
          </div>
        </div>
      </aside>

      <!-- ===== Main ===== -->
      <main class="main-area">
        <!-- ===== Toolbar: Stats + Filter + Actions ===== -->
        <div class="toolbar">
          <!-- Top row: Stats tabs + right actions -->
          <div class="toolbar-top">
            <div class="stat-tabs">
              <div
                class="stat-tab"
                :class="{ active: selectedFileType === 'all' }"
                @click="selectFileType('all')"
              >
                <span class="stat-tab-num">{{ statsData.total }}</span>
                <span class="stat-tab-label">全部文件</span>
              </div>
              <div
                class="stat-tab"
                :class="{ active: selectedFileType === 'image' }"
                @click="selectFileType('image')"
              >
                <component :is="LucideImage" class="stat-tab-icon" />
                <span class="stat-tab-num">{{ statsData.images }}</span>
                <span class="stat-tab-label">图片</span>
              </div>
              <div
                class="stat-tab"
                :class="{ active: selectedFileType === 'doc' }"
                @click="selectFileType('doc')"
              >
                <component :is="LucideFileText" class="stat-tab-icon" />
                <span class="stat-tab-num">{{ statsData.docs }}</span>
                <span class="stat-tab-label">文档</span>
              </div>
              <div
                class="stat-tab"
                :class="{ active: selectedFileType === 'video' }"
                @click="selectFileType('video')"
              >
                <component :is="LucideFilm" class="stat-tab-icon" />
                <span class="stat-tab-num">{{ statsData.videos }}</span>
                <span class="stat-tab-label">视频</span>
              </div>
              <div class="stat-divider-v" />
              <div class="stat-size">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                     stroke-linecap="round" stroke-linejoin="round" class="stat-size-icon">
                  <path d="M22 12H2" /><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" />
                  <line x1="6" y1="16" x2="6.01" y2="16" /><line x1="10" y1="16" x2="10.01" y2="16" />
                </svg>
                <span>{{ formatTotalSize(statsData.totalSize) }}</span>
              </div>
            </div>

            <div class="toolbar-actions">
              <Popconfirm
                v-if="selectedIds.size > 0"
                title="确定删除选中的文件吗？"
                @confirm="handleBatchDelete"
              >
                <Button danger size="small">
                  <template #icon><component :is="LucideTrash2" /></template>
                  删除 ({{ selectedIds.size }})
                </Button>
              </Popconfirm>

              <div class="view-toggle">
                <Tooltip title="网格视图">
                  <Button
                    :type="viewMode === 'grid' ? 'primary' : 'default'"
                    size="small"
                    @click="viewMode = 'grid'"
                  >
                    <template #icon><component :is="LucideGrid3x3" /></template>
                  </Button>
                </Tooltip>
                <Tooltip title="列表视图">
                  <Button
                    :type="viewMode === 'list' ? 'primary' : 'default'"
                    size="small"
                    @click="viewMode = 'list'"
                  >
                    <template #icon><component :is="LucideList" /></template>
                  </Button>
                </Tooltip>
              </div>

              <Tooltip title="上传文件">
                <Popover
                  v-model:open="uploadPopover"
                  trigger="click"
                  placement="bottomRight"
                >
                  <template #content>
                    <div class="upload-popover">
                      <div class="upload-popover-header">上传文件</div>
                      <Select
                        v-model:value="uploadEntityType"
                        style="width: 100%; margin-bottom: 12px"
                      >
                        <SelectOption v-for="opt in entityTypeOptions" :key="opt.value" :value="opt.value">
                          {{ getEntityTypeLabel(opt.value) }}
                        </SelectOption>
                      </Select>
                      <Upload
                        :custom-request="uploadFile"
                        :max-count="1"
                        accept="*"
                        list-type="text"
                      >
                        <Button type="primary" block>
                          <template #icon><component :is="LucideUpload" /></template>
                          选择文件上传
                        </Button>
                      </Upload>
                      <div class="upload-tip">支持图片、文档、PDF、视频等格式</div>
                    </div>
                  </template>
                  <Button type="primary">
                    <template #icon><component :is="LucideUpload" /></template>
                    上传
                  </Button>
                </Popover>
              </Tooltip>
            </div>
          </div>

          <!-- Bottom row: Search + filters -->
          <div class="toolbar-bottom">
            <div class="filter-row">
              <Input
                v-model:value="searchKeyword"
                placeholder="搜索文件名、上传人…"
                class="filter-search"
                allow-clear
                @pressEnter="handleSearch"
                @change="handleSearch"
              >
                <template #prefix>
                  <component :is="LucideSearch" style="color: #bfbfbf; font-size: 14px" />
                </template>
              </Input>

              <Select
                v-model:value="searchEntityType"
                placeholder="业务来源"
                class="filter-select"
                allow-clear
                @change="handleSearch"
              >
                <SelectOption v-for="opt in entityTypeOptions" :key="opt.value" :value="opt.value">
                  {{ getEntityTypeLabel(opt.value) }}
                </SelectOption>
              </Select>

              <Button type="primary" @click="handleSearch">搜索</Button>
              <Button @click="handleReset">重置</Button>
            </div>
          </div>
        </div>

        <!-- Content area -->
        <div class="content-area">
          <div class="content-card" :class="{ 'is-empty': displayList.length === 0 }">
            <div v-if="!loading && displayList.length === 0" class="empty-wrapper">
              <Empty description="暂无文件" />
            </div>

            <!-- ===== Grid View ===== -->
            <div v-if="viewMode === 'grid' && displayList.length > 0" class="grid-wrap">
              <div
                v-for="item in displayList"
                :key="item.id"
                class="grid-card"
                :class="{ 'is-selected': selectedIds.has(item.id) }"
                @click="toggleSelect(item.id)"
              >
                <!-- Thumb -->
                <div class="gc-thumb" @click.stop="handlePreview(item)">
                  <img
                    v-if="isImage(item.ext)"
                    :src="item.uploadUrl || item.upload_url"
                    :alt="item.originalName || item.name"
                    class="gc-img"
                    loading="lazy"
                  />
                  <div v-else class="gc-placeholder">
                    <component :is="getFileIcon(item)" class="gc-placeholder-icon" />
                  </div>
                  <div v-if="isImage(item.ext)" class="gc-hover-mask">
                    <Button size="small" ghost @click.stop="handlePreview(item)">
                      <template #icon><component :is="LucideEye" /></template>
                      预览
                    </Button>
                  </div>
                  <!-- Checkbox -->
                  <div class="gc-check" @click.stop="toggleSelect(item.id)">
                    <Checkbox :checked="selectedIds.has(item.id)" />
                  </div>
                </div>
                <!-- Info -->
                <div class="gc-body">
                  <div class="gc-name" :title="item.originalName || item.name">
                    {{ item.originalName || item.name }}
                  </div>
                  <div class="gc-meta">
                    <span class="gc-size">{{ formatSize(item.size) }}</span>
                    <span class="gc-ext">{{ (item.ext || '').toUpperCase() }}</span>
                  </div>
                  <div class="gc-footer">
                    <Tag :color="getEntityTypeColor(getItemEntityType(item))" class="gc-tag">
                      {{ getEntityTypeLabel(getItemEntityType(item)) }}
                    </Tag>
                    <div class="gc-actions">
                      <Tooltip title="下载">
                        <Button type="text" size="small" @click.stop="handleDownload(item)">
                          <template #icon><component :is="LucideDownload" style="font-size:14px" /></template>
                        </Button>
                      </Tooltip>
                      <Popconfirm title="确定删除？" @confirm.stop="handleDelete(item)">
                        <Tooltip title="删除">
                          <Button type="text" size="small" danger @click.stop>
                            <template #icon><component :is="LucideTrash2" style="font-size:14px" /></template>
                          </Button>
                        </Tooltip>
                      </Popconfirm>
                    </div>
                  </div>
                  <!-- 悬停展开：上传人 / 上传时间 -->
                  <div class="gc-extra">
                    <span class="gc-extra-label">上传人</span>
                    <span class="gc-extra-value">{{ getItemUploadedBy(item) }}</span>
                    <span class="gc-extra-dot">·</span>
                    <span class="gc-extra-label">时间</span>
                    <span class="gc-extra-value">{{ formatDateTime(item.createTime || item.create_time) }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- ===== List View ===== -->
            <div v-if="viewMode === 'list' && displayList.length > 0" class="list-wrap">
              <table class="file-table">
                <thead>
                  <tr>
                    <th style="width: 36px"></th>
                    <th>文件名</th>
                    <th style="width: 110px">业务来源</th>
                    <th style="width: 70px">类型</th>
                    <th style="width: 90px">大小</th>
                    <th style="width: 80px">权限</th>
                    <th style="width: 110px">上传人</th>
                    <th style="width: 160px">上传时间</th>
                    <th style="width: 180px">操作</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="item in displayList"
                    :key="item.id"
                    class="file-row"
                    :class="{ 'row-selected': selectedIds.has(item.id) }"
                    @click="toggleSelect(item.id)"
                  >
                    <td @click.stop="toggleSelect(item.id)">
                      <Checkbox :checked="selectedIds.has(item.id)" />
                    </td>
                    <td>
                      <div class="cell-name">
                        <div class="cell-icon-wrap">
                          <img
                            v-if="isImage(item.ext)"
                            :src="item.uploadUrl || item.upload_url"
                            class="cell-thumb"
                          />
                          <component v-else :is="getFileIcon(item)" class="cell-icon" />
                        </div>
                        <span class="cell-filename" :title="item.originalName || item.name">
                          {{ item.originalName || item.name }}
                        </span>
                      </div>
                    </td>
                    <td>
                      <Tag :color="getEntityTypeColor(getItemEntityType(item))">
                        {{ getEntityTypeLabel(getItemEntityType(item)) }}
                      </Tag>
                    </td>
                    <td><span class="cell-ext">{{ (item.ext || '-').toUpperCase() }}</span></td>
                    <td>{{ formatSize(item.size) }}</td>
                    <td>
                      <Tag :color="isItemPublic(item) ? 'green' : 'default'" style="font-size:11px">
                        {{ isItemPublic(item) ? '公开' : '私有' }}
                      </Tag>
                    </td>
                    <td>{{ getItemUploadedBy(item) }}</td>
                    <td>{{ formatDateTime(item.createTime || item.create_time) }}</td>
                    <td @click.stop>
                      <div class="cell-actions">
                        <Button v-if="isImage(item.ext)" type="link" size="small" @click="handlePreview(item)">预览</Button>
                        <Button type="link" size="small" @click="handleDownload(item)">下载</Button>
                        <Popconfirm title="确定删除？" @confirm="handleDelete(item)">
                          <Button type="link" size="small" danger>删除</Button>
                        </Popconfirm>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>

            <!-- Pagination -->
            <div v-if="displayList.length > 0" class="pagination-bar">
              <span class="page-total">共 {{ pagination.total }} 项</span>
              <Pagination
                :current="pagination.current"
                :page-size="pagination.pageSize"
                :total="pagination.total"
                show-size-changer
                show-quick-jumper
                size="small"
                @change="handlePageChange"
                @showSizeChange="handlePageChange"
              />
            </div>
          </div>
        </div>
      </main>
    </div>

    <!-- ===== Preview Modal ===== -->
    <Modal
      v-model:open="previewVisible"
      :title="previewTitle"
      :footer="null"
      width="auto"
      :centered="true"
      :mask-closable="true"
      class="preview-modal"
      @cancel="handlePreviewClose"
    >
      <img :src="previewImage" :alt="previewTitle" class="preview-img" />
    </Modal>
  </Page>
</template>

<style>
/* ============================================= */
/*  Attachment File Management - Professional    */
/* ============================================= */

.attachment-page {
  height: 100%;
  padding: 0 !important;
  background: #f5f5f5;
}

.attachment-layout {
  display: flex;
  /* 使用 min-height 而非 height，让整体高度随右侧列表内容自适应扩展，
     不被视口或左侧分类树高度限制 */
  min-height: calc(100vh - 110px);
}

/* ---- Sidebar ---- */
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

/* ---- Main ---- */
.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  /* 去掉 overflow: hidden，让内容自然撑开，不在内部形成滚动条 */
  background: #f5f5f5;
}

/* ---- Toolbar ---- */
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

.stat-tabs {
  display: flex;
  align-items: center;
  gap: 0;
}

.stat-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 16px;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s;
}
.stat-tab:hover {
  background: #f5f5f5;
}
.stat-tab.active {
  background: #e6f4ff;
}
.stat-tab.active .stat-tab-num {
  color: #1677ff;
}
.stat-tab.active .stat-tab-label {
  color: #1677ff;
}

.stat-tab-icon {
  font-size: 16px;
  color: #8c8c8c;
}
.stat-tab.active .stat-tab-icon {
  color: #1677ff;
}

.stat-tab-num {
  font-size: 18px;
  font-weight: 700;
  color: #262626;
  line-height: 1;
}

.stat-tab-label {
  font-size: 13px;
  color: #595959;
}

.stat-divider-v {
  width: 1px;
  height: 28px;
  background: #f0f0f0;
  margin: 0 12px;
}

.stat-size {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #595959;
  font-size: 13px;
}
.stat-size-icon {
  width: 18px;
  height: 18px;
  color: #8c8c8c;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.view-toggle {
  display: flex;
  gap: 2px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  overflow: hidden;
}
.view-toggle .ant-btn {
  border: none;
  border-radius: 0;
}

.toolbar-bottom {
  padding: 0 20px 14px;
}

.filter-row {
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

/* ---- Upload Popover ---- */
.upload-popover {
  width: 280px;
}
.upload-popover-header {
  font-weight: 600;
  font-size: 14px;
  margin-bottom: 12px;
}
.upload-tip {
  font-size: 12px;
  color: #8c8c8c;
  margin-top: 8px;
  text-align: center;
}

/* ---- Content ---- */
.content-area {
  flex: 1;
  /* 去掉 overflow-y: auto，不在内部形成滚动条，让外层页面自然滚动 */
}

.content-card {
  padding: 15px;
  border: 1px solid #f0f0f0;
  width: 100%;
  /* 最小高度占满容器，内容多时自适应扩展，不遮挡内部元素 */
  min-height: 100%;
  box-sizing: border-box;
  /* 使用 visible 让悬停展开的 .gc-extra 浮层不被裁剪 */
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

/* ===== Grid View ===== */
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
  box-shadow: 0 2px 8px rgba(0,0,0,0.08);
  border-color: #d9d9d9;
  /* 悬停时取消底部圆角，让 .gc-extra 顶部边框无缝接续 */
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
}
.grid-card.is-selected {
  border-color: #1677ff;
  box-shadow: 0 0 0 2px rgba(22,119,255,0.15);
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
  background: rgba(0,0,0,0.35);
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
  background: rgba(255,255,255,0.85);
  border-radius: 4px;
  padding: 2px;
  line-height: 0;
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

/* 悬停展开：上传人/时间（绝对定位，与卡片合为一体，用分割线隔开，不撑高卡片/不推移布局） */
.gc-extra {
  position: absolute;
  top: 100%;
  left: -1px;
  right: -1px;
  z-index: 20;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  padding: 8px 12px;
  font-size: 12px;
  line-height: 18px;
  color: #8c8c8c;
  background: #fff;
  border: 1px solid #f0f0f0;
  border-top: 1px solid #f5f5f5;
  border-radius: 0 0 8px 8px;
  opacity: 0;
  transform: translateY(-4px);
  pointer-events: none;
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.grid-card:hover .gc-extra {
  opacity: 1;
  transform: translateY(0);
  pointer-events: auto;
  /* 与卡片悬停状态保持一致的边框和阴影，视觉上连成一体 */
  border-color: #d9d9d9;
  border-top-color: #f5f5f5;
  box-shadow: 0 2px 8px rgba(0,0,0,0.08);
}
.gc-extra-label {
  color: #bfbfbf;
}
.gc-extra-value {
  color: #595959;
}
.gc-extra-dot {
  color: #d9d9d9;
  margin: 0 2px;
}

/* ===== List View ===== */
.list-wrap {
  background: #fff;
  border-radius: 8px;
  border: 1px solid #f0f0f0;
}

.file-table {
  width: 100%;
  border-collapse: collapse;
}
.file-table th,
.file-table td {
  padding: 10px 12px;
  text-align: left;
  border-bottom: 1px solid #f0f0f0;
  font-size: 13px;
}
.file-table th {
  background: #fafafa;
  font-weight: 500;
  color: #595959;
  white-space: nowrap;
}
.file-row:hover {
  background: #fafafa;
}
.file-row.row-selected {
  background: #e6f4ff;
}

.cell-name {
  display: flex;
  align-items: center;
  gap: 10px;
}
.cell-icon-wrap {
  width: 36px;
  height: 36px;
  border-radius: 4px;
  overflow: hidden;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #fafafa;
}
.cell-thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.cell-icon {
  font-size: 18px;
  color: #8c8c8c;
}
.cell-filename {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 300px;
}
.cell-ext {
  font-size: 12px;
  color: #595959;
  font-weight: 500;
}
.cell-actions {
  display: flex;
  gap: 2px;
  flex-wrap: nowrap;
}

/* ---- Pagination ---- */
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

/* ---- Preview Modal ---- */
.preview-modal .ant-modal-body {
  padding: 0;
  line-height: 0;
}
.preview-img {
  max-width: 90vw;
  max-height: 85vh;
  display: block;
}
</style>