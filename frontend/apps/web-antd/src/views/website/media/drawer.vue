<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { Image, message } from 'ant-design-vue';
import {
  getMediaCategoryAllApi,
  getMediaDetailApi,
  updateMediaApi,
} from '#/api';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增媒体' : '编辑媒体'));

// 分类树数据（用于 TreeSelect）
const categoryTreeData = ref<any[]>([]);

const buildCategoryTree = (list: any[]): any[] =>
  list.map((item) => ({
    title: item.categoryName,
    value: item.id,
    key: item.id,
    children: item.children?.length ? buildCategoryTree(item.children) : undefined,
  }));

async function loadCategoryTree() {
  try {
    const result: any = await getMediaCategoryAllApi();
    const list = Array.isArray(result) ? result : result?.data || [];
    categoryTreeData.value = buildCategoryTree(list);
  } catch (e) {
    console.error('加载分类树失败', e);
    categoryTreeData.value = [];
  }
}

onMounted(() => {
  loadCategoryTree();
});

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Input',
      fieldName: 'altText',
      label: 'ALT文本',
      componentProps: {
        placeholder: '请输入图片ALT文本（用于SEO和无障碍）',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'title',
      label: '标题',
      componentProps: {
        placeholder: '请输入标题',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'caption',
      label: '说明文字',
      componentProps: {
        placeholder: '请输入说明文字（图片下方展示）',
        allowClear: true,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'description',
      label: '描述',
      componentProps: {
        placeholder: '请输入详细描述',
        allowClear: true,
        rows: 4,
      },
    },
    {
      component: 'TreeSelect',
      fieldName: 'categoryId',
      label: '所属分类',
      componentProps: {
        treeData: categoryTreeData,
        placeholder: '请选择分类',
        allowClear: true,
        treeDefaultExpandAll: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'tags',
      label: '标签',
      componentProps: {
        mode: 'tags',
        placeholder: '输入标签后按回车添加，可多选',
        allowClear: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: '排序',
      defaultValue: 0,
      componentProps: {
        min: 0,
        style: 'width: 100%',
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      label: '状态',
      defaultValue: 1,
      componentProps: {
        options: [
          { label: '启用', value: 1 },
          { label: '禁用', value: 0 },
        ],
      },
    },
  ],
});

// 当前媒体详情（用于图片预览）
const currentMedia = ref<any>({});

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[50%] max-w-[100vw]',
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) {
      return;
    }

    setLoading(true);

    const values = await baseFormApi.getValues();

    try {
      if (isCreate.value) {
        // 新增模式一般由上传触发，此处不做处理
        message.info('请通过上传按钮新增媒体');
      } else {
        await updateMediaApi(data.value.row.id, values);
        message.success('保存成功');
        drawerApi.setData({ needRefresh: true });
        drawerApi.close();
      }
    } finally {
      setLoading(false);
    }
  },

  async onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row || {};

      // 编辑模式：拉取最新详情后填充表单
      if (!isCreate.value && row.id) {
        try {
          const detail: any = await getMediaDetailApi(row.id);
          const detailData = detail?.data || detail || row;
          currentMedia.value = detailData;
          baseFormApi.setValues(detailData);
        } catch (e) {
          console.error('加载详情失败', e);
          currentMedia.value = row;
          baseFormApi.setValues(row);
        }
      } else {
        currentMedia.value = row;
        baseFormApi.setValues(row);
      }
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

const previewUrl = computed(() => {
  const m = currentMedia.value;
  return m.thumbMedium || m.thumbLarge || m.thumbSmall || m.fileUrl || '';
});

const fileName = computed(() => {
  const m = currentMedia.value;
  return m.originalName || m.title || '-';
});

const fileSize = computed(() => {
  const size = currentMedia.value?.fileSize;
  if (!size) return '-';
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
  return `${(size / (1024 * 1024)).toFixed(2)} MB`;
});
</script>

<template>
  <Drawer :title="getTitle">
    <!-- 图片预览 -->
    <div v-if="previewUrl" class="media-preview">
      <Image
        :src="previewUrl"
        :width="200"
        fit="cover"
        class="rounded border border-gray-200"
      />
      <div class="media-preview-info">
        <div class="info-row">
          <span class="info-label">文件名：</span>
          <span class="info-value" :title="fileName">{{ fileName }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">大小：</span>
          <span class="info-value">{{ fileSize }}</span>
        </div>
        <div v-if="currentMedia.fileExt" class="info-row">
          <span class="info-label">类型：</span>
          <span class="info-value">{{ (currentMedia.fileExt || '').toUpperCase() }}</span>
        </div>
      </div>
    </div>

    <BaseForm />
  </Drawer>
</template>

<style scoped>
.media-preview {
  display: flex;
  gap: 16px;
  padding: 0 16px 16px;
  align-items: flex-start;
}

.media-preview-info {
  flex: 1;
  min-width: 0;
}

.info-row {
  display: flex;
  align-items: center;
  font-size: 13px;
  margin-bottom: 6px;
}

.info-label {
  color: rgba(0, 0, 0, 0.45);
  flex-shrink: 0;
  width: 60px;
}

.info-value {
  color: rgba(0, 0, 0, 0.75);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>

<style>
@media (max-width: 767px) {
  .vben-drawer .ant-drawer-content-wrapper {
    width: 100% !important;
    max-width: 100vw !important;
  }
}
</style>
