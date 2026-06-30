<script lang="ts" setup>
import { ref, watch } from 'vue';
import { Upload, message } from 'ant-design-vue';
import type { UploadFile } from 'ant-design-vue';
import { uploadFileApi } from '#/api/core/attachment/file';
import { LucideUpload, LucideX } from '@vben/icons';

interface Props {
  entityType: string;        // 业务类型（product/avatar/contract/invoice/quotation/payment/common）
  entityId?: number;         // 业务ID（可选，先上传再创建业务时为空）
  accept?: string;           // 接受的文件类型
  maxCount?: number;         // 最大上传数
  listType?: 'picture-card' | 'text';  // 列表样式
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  accept: 'image/*',
  maxCount: 1,
  listType: 'picture-card',
  disabled: false,
});

const emit = defineEmits<{
  success: [response: any];
  change: [fileList: UploadFile[]];
  removed: [file: UploadFile];
}>();

const fileList = ref<UploadFile[]>([]);

// 自定义上传请求
const customRequest = async (options: any) => {
  const { file, onSuccess, onError, onProgress } = options;
  try {
    onProgress?.({ percent: 30 });
    const result: any = await uploadFileApi(file, props.entityType, props.entityId);
    onProgress?.({ percent: 100 });
    onSuccess?.(result, file);
    emit('success', result);
    message.success('上传成功');
  } catch (e: any) {
    console.error('上传失败:', e);
    onError?.(e);
    message.error(e?.message || '上传失败');
  }
};

const handleChange = (info: any) => {
  fileList.value = info.fileList;
  emit('change', info.fileList);
};

const handleRemove = (file: UploadFile) => {
  emit('removed', file);
  return true;
};

// 重置文件列表
const reset = () => {
  fileList.value = [];
};

// 设置文件列表（用于回显已有文件）
const setFileList = (list: UploadFile[]) => {
  fileList.value = list;
};

defineExpose({ reset, setFileList, fileList });
</script>

<template>
  <div class="file-uploader">
    <Upload
      :accept="accept"
      :max-count="maxCount"
      :file-list="fileList"
      :custom-request="customRequest"
      :list-type="listType"
      :disabled="disabled"
      @change="handleChange"
      @remove="handleRemove"
    >
      <template v-if="listType === 'picture-card'">
        <div class="upload-btn">
          <component :is="LucideUpload" class="upload-icon" />
          <div class="upload-text">点击上传</div>
        </div>
      </template>
      <template v-else>
        <a-button :disabled="disabled">
          <template #icon>
            <component :is="LucideUpload" />
          </template>
          点击上传
        </a-button>
      </template>
    </Upload>
  </div>
</template>

<style scoped>
.file-uploader {
  width: 100%;
}
.upload-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #8c8c8c;
}
.upload-icon {
  font-size: 24px;
  margin-bottom: 4px;
}
.upload-text {
  font-size: 13px;
}
</style>
