<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message, Upload } from 'ant-design-vue';
import type { UploadFile } from 'ant-design-vue';
import { templateApi } from '#/api';
import { uploadFileApi } from '#/api/core/attachment/file';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增模板' : '编辑模板'));

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
      fieldName: 'name',
      label: '模板名称',
      componentProps: {
        placeholder: '请输入模板名称（唯一标识）',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入模板名称' }),
    },
    {
      component: 'Input',
      fieldName: 'previewUrl',
      label: '演示网址',
      componentProps: {
        placeholder: '模板在线演示地址',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'remark',
      label: '备注说明',
      componentProps: {
        type: 'textarea',
        autosize: { minRows: 3, maxRows: 6 },
        placeholder: '模板简介、功能说明等',
        allowClear: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: '排序',
      defaultValue: 0,
      componentProps: { min: 0 },
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

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[60%] max-w-[100vw]',
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) return;
    setLoading(true);
    const values = await baseFormApi.getValues();
    // 合并预览图URL
    values.previewPic = previewPicUrl.value || undefined;
    try {
      if (isCreate.value) {
        await templateApi.add(values);
        message.success('新增成功');
      } else {
        await templateApi.update(data.value.row.id, values);
        message.success('修改成功');
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      setLoading(false);
    }
  },
  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row || {};
      baseFormApi.setValues(row);
      previewPicUrl.value = row.previewPic || '';
      syncFileList(previewPicUrl.value);
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

// 预览图上传
const fileList = ref<UploadFile[]>([]);
const previewPicUrl = ref('');
const uploading = ref(false);

async function handleUpload(file: File) {
  uploading.value = true;
  try {
    const res: any = await uploadFileApi(file, 'common');
    // 尝试多种 response 格式兼容
    const url = res?.data?.url || res?.url || res?.data;
    if (url && typeof url === 'string') {
      previewPicUrl.value = url;
      syncFileList(url);
      message.success('上传成功');
    } else {
      message.error('上传返回异常：未获取到图片地址');
    }
    return false;
  } catch (err: any) {
    message.error(err?.message || '上传失败，请检查网络或服务器配置');
    return false;
  } finally {
    uploading.value = false;
  }
}

function handleRemove() {
  fileList.value = [];
  previewPicUrl.value = '';
}

function syncFileList(url: string) {
  if (url) {
    fileList.value = [{ uid: '-1', name: 'preview', status: 'done', url }];
  } else {
    fileList.value = [];
  }
}
</script>

<template>
  <Drawer :title="getTitle">
    <BaseForm />

    <!-- 预览图上传 -->
    <div class="upload-section">
      <div class="upload-section-header">
        <span class="upload-section-label">预览图</span>
        <span class="upload-section-tip">建议尺寸 600x400，支持 JPG/PNG/WebP</span>
      </div>
      <div class="upload-section-body">
        <Upload
          :file-list="fileList"
          :before-upload="(file: File) => { handleUpload(file); return false; }"
          :remove="handleRemove"
          list-type="picture-card"
          accept="image/*"
          :disabled="uploading"
        >
          <div v-if="fileList.length < 1" class="upload-placeholder">
            <div :class="['upload-icon', uploading ? 'uploading' : '']">
              <template v-if="uploading">...</template>
              <template v-else>+</template>
            </div>
            <div class="upload-text">{{ uploading ? '上传中…' : '上传预览图' }}</div>
          </div>
        </Upload>
        <div v-if="previewPicUrl" class="upload-preview-url">
          图片地址：<span class="url-text">{{ previewPicUrl }}</span>
        </div>
      </div>
    </div>
  </Drawer>
</template>

<style scoped>
.upload-section {
  margin: 16px 16px 0;
  padding: 16px;
  background: var(--card-background, #fafafa);
  border: 1px solid var(--border-color, #f0f0f0);
  border-radius: 8px;
}

.upload-section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.upload-section-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary, rgba(0, 0, 0, 0.88));
}

.upload-section-tip {
  font-size: 12px;
  color: var(--text-secondary, rgba(0, 0, 0, 0.45));
}

.upload-section-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.upload-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 8px;
}

.upload-icon {
  font-size: 24px;
  line-height: 1;
  margin-bottom: 4px;
  color: rgba(0, 0, 0, 0.45);
}

.upload-icon.uploading {
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.upload-text {
  font-size: 12px;
  color: rgba(0, 0, 0, 0.45);
}

.upload-preview-url {
  font-size: 12px;
  color: var(--text-secondary, rgba(0, 0, 0, 0.45));
  word-break: break-all;
  line-height: 1.5;
}

.url-text {
  color: var(--primary-color, #1677ff);
  font-family: monospace;
}

/* 暗黑模式适配 */
:root.dark .upload-section {
  --card-background: #1f1f1f;
  --border-color: #333;
  --text-primary: rgba(255, 255, 255, 0.88);
  --text-secondary: rgba(255, 255, 255, 0.45);
}

@media (max-width: 767px) {
  :deep(.ant-drawer-content-wrapper) {
    width: 100% !important;
    max-width: 100vw !important;
  }
}
</style>