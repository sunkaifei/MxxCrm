<script lang="ts" setup>
import { onMounted, ref } from 'vue';
import { useVbenModal } from '@vben/common-ui';
import {
  LucideImage,
  LucideFileText,
  LucideFilm,
  LucideFile,
  LucideSearch,
  LucideCheck,
} from '@vben/icons';

import {
  Button,
  Empty,
  Input,
  Pagination,
  message,
} from 'ant-design-vue';

import { getMediaListApi } from '#/api';

interface MediaItem {
  id: number;
  originalName?: string;
  storageName?: string;
  fileUrl?: string;
  fileExt?: string;
  fileSize?: number;
  fileType?: number;
  thumbSmall?: string;
  thumbMedium?: string;
  thumbLarge?: string;
}

const emit = defineEmits<{
  select: [items: MediaItem[]];
}>();

const props = withDefaults(
  defineProps<{
    multiple?: boolean;
    fileType?: number;
  }>(),
  { multiple: false },
);

// --- 状态 ---
const searchKeyword = ref('');
const mediaList = ref<MediaItem[]>([]);
const loading = ref(false);
const pagination = ref({ current: 1, pageSize: 24, total: 0 });
const selectedIds = ref<Set<number>>(new Set());
const selectedItems = ref<MediaItem[]>([]);

const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg', 'ico'];

const isImage = (item: MediaItem) => {
  if (item.fileType === 1) return true;
  const ext = item.fileExt?.toLowerCase() ?? '';
  return imageExtensions.includes(ext);
};

const getThumbUrl = (item: MediaItem) => {
  if (item.thumbMedium) return item.thumbMedium;
  if (item.thumbSmall) return item.thumbSmall;
  if (item.thumbLarge) return item.thumbLarge;
  if (isImage(item)) return item.fileUrl;
  return '';
};

const getFileIcon = (item: MediaItem) => {
  if (item.fileType === 1 || isImage(item)) return LucideImage;
  if (item.fileType === 2) return LucideFilm;
  if (item.fileType === 3) return LucideFileText;
  return LucideFile;
};

const formatSize = (size?: number) => {
  if (!size) return '-';
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
  return `${(size / (1024 * 1024)).toFixed(2)} MB`;
};

// --- 方法 ---
const loadMediaList = async () => {
  loading.value = true;
  try {
    const params: any = {
      page: pagination.value.current,
      pageSize: pagination.value.pageSize,
      keywords: searchKeyword.value || undefined,
      fileType: props.fileType || undefined,
    };
    const result: any = await getMediaListApi(params);
    const list = result?.items || result?.list || result?.rows || [];
    mediaList.value = list;
    pagination.value.total = result?.total || result?.count || 0;
  } catch {
    mediaList.value = [];
  } finally {
    loading.value = false;
  }
};

const handleSearch = () => {
  pagination.value.current = 1;
  loadMediaList();
};

const handlePageChange = (page: number, pageSize: number) => {
  pagination.value.current = page;
  pagination.value.pageSize = pageSize;
  loadMediaList();
};

const toggleSelect = (item: MediaItem) => {
  const id = item.id;
  if (props.multiple) {
    const next = new Set(selectedIds.value);
    const nextItems = [...selectedItems.value];
    if (next.has(id)) {
      next.delete(id);
      const idx = nextItems.findIndex((i) => i.id === id);
      if (idx >= 0) nextItems.splice(idx, 1);
    } else {
      next.add(id);
      nextItems.push(item);
    }
    selectedIds.value = next;
    selectedItems.value = nextItems;
  } else {
    selectedIds.value = new Set([id]);
    selectedItems.value = [item];
  }
};

const isSelected = (id: number) => selectedIds.value.has(id);

const handleConfirm = () => {
  if (selectedItems.value.length === 0) {
    message.warning('请先选择媒体文件');
    return;
  }
  emit('select', selectedItems.value);
};

// --- 弹窗 API ---
const [Modal, modalApi] = useVbenModal({
  title: '选择媒体文件',
  class: 'media-picker-modal',
  onConfirm: handleConfirm,
  onCancel() {
    modalApi.close();
  },
  async onOpenChange(isOpen) {
    if (isOpen) {
      resetState();
      loadMediaList();
    }
  },
});

function resetState() {
  searchKeyword.value = '';
  selectedIds.value = new Set();
  selectedItems.value = [];
  pagination.value = { current: 1, pageSize: 24, total: 0 };
}

onMounted(() => {
  // 弹窗初始化时不加载，等打开时再加载
});
</script>

<template>
  <Modal>
    <div class="media-picker">
      <!-- 搜索栏 -->
      <div class="picker-search">
        <Input
          v-model:value="searchKeyword"
          placeholder="搜索文件名…"
          allow-clear
          @press-enter="handleSearch"
        >
          <template #prefix>
            <component :is="LucideSearch" style="color: #bfbfbf; font-size: 14px" />
          </template>
        </Input>
        <Button type="primary" size="small" @click="handleSearch">搜索</Button>
      </div>

      <!-- 媒体网格 -->
      <div class="picker-grid">
        <div v-if="loading" class="picker-loading">加载中…</div>
        <div v-else-if="mediaList.length === 0" class="picker-empty">
          <Empty description="暂无媒体文件" />
        </div>
        <div v-else class="grid-wrap">
          <div
            v-for="item in mediaList"
            :key="item.id"
            class="grid-card"
            :class="{ 'is-selected': isSelected(item.id) }"
            @click="toggleSelect(item)"
          >
            <!-- 缩略图 -->
            <div class="gc-thumb">
              <img
                v-if="getThumbUrl(item)"
                :src="getThumbUrl(item)"
                :alt="item.originalName"
                class="gc-img"
                loading="lazy"
              />
              <div v-else class="gc-placeholder">
                <component :is="getFileIcon(item)" class="gc-placeholder-icon" />
              </div>
              <!-- 选中标记 -->
              <div v-if="isSelected(item.id)" class="gc-selected-badge">
                <component :is="LucideCheck" />
              </div>
            </div>

            <!-- 信息 -->
            <div class="gc-body">
              <div class="gc-name" :title="item.originalName">
                {{ item.originalName || '-' }}
              </div>
              <div class="gc-meta">
                <span class="gc-size">{{ formatSize(item.fileSize) }}</span>
                <span v-if="item.fileExt" class="gc-ext">
                  {{ (item.fileExt || '').toUpperCase() }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 分页 -->
      <div v-if="mediaList.length > 0" class="picker-pagination">
        <span class="page-total">共 {{ pagination.total }} 项</span>
        <Pagination
          :current="pagination.current"
          :page-size="pagination.pageSize"
          :total="pagination.total"
          size="small"
          show-size-changer
          show-quick-jumper
          @change="handlePageChange"
          @show-size-change="handlePageChange"
        />
      </div>
    </div>
  </Modal>
</template>

<style scoped>
.media-picker {
  min-height: 400px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.picker-search {
  display: flex;
  gap: 8px;
}

.picker-loading {
  text-align: center;
  padding: 60px 0;
  color: #8c8c8c;
}

.picker-empty {
  padding: 40px 0;
}

.grid-wrap {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 12px;
}

.grid-card {
  background: #fff;
  border: 1px solid #f0f0f0;
  border-radius: 8px;
  cursor: pointer;
  overflow: hidden;
  transition: all 0.2s;
}
.grid-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
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
  font-size: 32px;
  color: #bfbfbf;
}
.gc-selected-badge {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 22px;
  height: 22px;
  background: #1677ff;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 14px;
}

.gc-body {
  padding: 8px 10px;
}
.gc-name {
  font-size: 12px;
  color: #262626;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.gc-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 4px;
  font-size: 11px;
  color: #8c8c8c;
}
.gc-ext {
  background: #f5f5f5;
  padding: 0 5px;
  border-radius: 3px;
}

.picker-pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 8px;
  border-top: 1px solid #f0f0f0;
}
.page-total {
  font-size: 12px;
  color: #8c8c8c;
}
</style>