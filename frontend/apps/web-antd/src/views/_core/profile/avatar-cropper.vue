<script setup lang="ts">
import { ref, computed } from 'vue';
import { Cropper } from 'vue-advanced-cropper';
import 'vue-advanced-cropper/dist/style.css';

import {
  Modal,
  Button,
  Slider,
  message,
} from 'ant-design-vue';

import { uploadFileApi } from '#/api/core/attachment/file';

const props = defineProps<{
  visible: boolean;
  avatarUrl?: string;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'success', url: string): void;
}>();

const cropperRef = ref<any>(null);
const imageSrc = ref<string>('');
const previewUrl = ref<string>('');
const zoom = ref<number>(1);
const currentZoom = ref<number>(1);
const uploading = ref<boolean>(false);

const visible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val),
});

const handleClose = () => {
  visible.value = false;
  imageSrc.value = '';
  previewUrl.value = '';
  zoom.value = 1;
  currentZoom.value = 1;
};

const handleChooseFile = (e: Event) => {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  if (!file.type.startsWith('image/')) {
    message.error('请选择图片文件');
    return;
  }

  const reader = new FileReader();
  reader.onload = (event) => {
    imageSrc.value = event.target?.result as string;
    previewUrl.value = '';
    zoom.value = 1;
    currentZoom.value = 1;
  };
  reader.readAsDataURL(file);
  input.value = '';
};

// 裁剪区域变化时，实时生成预览图（与框选范围一致）
const handleChange = (event: any) => {
  const canvas = event?.canvas;
  if (canvas) {
    previewUrl.value = canvas.toDataURL('image/jpeg', 0.92);
  }
};

const handleReady = () => {
  currentZoom.value = 1;
  zoom.value = 1;
  // 初始化时同步一次预览
  if (cropperRef.value) {
    const result = cropperRef.value.getResult();
    if (result?.canvas) {
      previewUrl.value = result.canvas.toDataURL('image/jpeg', 0.92);
    }
  }
};

const handleZoomChange = (val: number) => {
  if (cropperRef.value && currentZoom.value > 0) {
    const ratio = val / currentZoom.value;
    cropperRef.value.zoom(ratio);
  }
};

const handleZoomUpdate = (e: any) => {
  currentZoom.value = e.zoom;
  zoom.value = Math.round(e.zoom * 100) / 100;
};

const handleRotate = (degree: number) => {
  if (cropperRef.value) {
    cropperRef.value.rotate(degree);
  }
};

const handleFlipH = () => {
  if (cropperRef.value) {
    cropperRef.value.flipHorizontal();
  }
};

const handleFlipV = () => {
  if (cropperRef.value) {
    cropperRef.value.flipVertical();
  }
};

const handleReset = () => {
  if (cropperRef.value) {
    cropperRef.value.reset();
    currentZoom.value = 1;
    zoom.value = 1;
    const result = cropperRef.value.getResult();
    if (result?.canvas) {
      previewUrl.value = result.canvas.toDataURL('image/jpeg', 0.92);
    }
  }
};

const handleConfirm = async () => {
  if (!cropperRef.value || !imageSrc.value) return;

  uploading.value = true;
  try {
    const result = cropperRef.value.getResult();
    if (!result?.canvas) {
      message.error('裁剪失败');
      uploading.value = false;
      return;
    }

    result.canvas.toBlob(async (blob: Blob | null) => {
      if (!blob) {
        message.error('生成图片失败');
        uploading.value = false;
        return;
      }

      const file = new File([blob], 'avatar.jpg', { type: 'image/jpeg' });
      const res: any = await uploadFileApi(file, 'avatar');

      if (res?.url || res?.uploadUrl) {
        const url = res.url || res.uploadUrl;
        message.success('头像上传成功');
        emit('success', url);
        handleClose();
      } else {
        message.error('上传失败');
      }
      uploading.value = false;
    }, 'image/jpeg', 0.92);
  } catch (e: any) {
    message.error(e?.message || '上传失败');
    uploading.value = false;
  }
};
</script>

<template>
  <Modal
    v-model:open="visible"
    title="上传头像"
    :width="680"
    :confirm-loading="uploading"
    @ok="handleConfirm"
    @cancel="handleClose"
    ok-text="确认上传"
    cancel-text="取消"
    :z-index="2000"
  >
    <div class="avatar-cropper">
      <!-- 裁剪区域 -->
      <div class="cropper-wrapper">
        <div v-if="!imageSrc" class="upload-placeholder">
          <label class="upload-btn">
            <input type="file" accept="image/*" @change="handleChooseFile" hidden />
            <div class="upload-placeholder-content">
              <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="17 8 12 3 7 8"/>
                <line x1="12" y1="3" x2="12" y2="15"/>
              </svg>
              <span>点击选择图片</span>
              <span class="upload-hint">支持 JPG、PNG、GIF 格式</span>
            </div>
          </label>
        </div>
        <Cropper
          v-else
          ref="cropperRef"
          class="cropper"
          :src="imageSrc"
          :stencil-props="{ aspectRatio: 1, handlers: { show: true } }"
          image-restriction="stencil"
          :stencil-size="{ width: 240, height: 240 }"
          :default-size="{ width: 240, height: 240 }"
          :min-width="50"
          :min-height="50"
          @ready="handleReady"
          @change="handleChange"
          @zoom="handleZoomUpdate"
        />
      </div>

      <!-- 预览和控制区 -->
      <div v-if="imageSrc" class="control-panel">
        <!-- 预览 -->
        <div class="preview-section">
          <div class="preview-label">预览效果</div>
          <div class="preview-circle">
            <img v-if="previewUrl" :src="previewUrl" class="preview-img" />
          </div>
          <span class="preview-tip">正方形裁剪区域<br />显示为圆形头像</span>
        </div>

        <!-- 控制按钮 -->
        <div class="control-section">
          <div class="control-row">
            <span class="control-label">缩放</span>
            <Slider
              v-model:value="zoom"
              :min="0.1"
              :max="5"
              :step="0.1"
              style="width: 200px"
              @change="handleZoomChange"
            />
          </div>
          <div class="control-row">
            <span class="control-label">旋转</span>
            <div class="btn-group">
              <Button size="small" @click="handleRotate(-90)">-90°</Button>
              <Button size="small" @click="handleRotate(90)">+90°</Button>
            </div>
          </div>
          <div class="control-row">
            <span class="control-label">翻转</span>
            <div class="btn-group">
              <Button size="small" @click="handleFlipH">水平</Button>
              <Button size="small" @click="handleFlipV">垂直</Button>
            </div>
          </div>
          <div class="control-row">
            <span class="control-label"></span>
            <div class="btn-group">
              <Button size="small" @click="handleReset">重置</Button>
              <label class="reselect-btn">
                <input type="file" accept="image/*" @change="handleChooseFile" hidden />
                <Button size="small" type="primary">重新选择</Button>
              </label>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Modal>
</template>

<style>
.avatar-cropper {
  display: flex;
  gap: 28px;
}

.avatar-cropper .cropper-wrapper {
  width: 380px;
  height: 380px;
  background: #2a2a2a;
  border-radius: 8px;
  overflow: hidden;
  position: relative;
  flex-shrink: 0;
}

.avatar-cropper .cropper {
  width: 100%;
  height: 100%;
}

.avatar-cropper .vue-advanced-cropper {
  width: 100%;
  height: 100%;
  background: #1a1a1a !important;
}

.avatar-cropper .upload-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border: 2px dashed #4a4a4a;
  border-radius: 8px;
  transition: all 0.3s;
  background: #2a2a2a;
}

.avatar-cropper .upload-placeholder:hover {
  border-color: #1890ff;
  background: #1f2d3d;
}

.avatar-cropper .upload-btn {
  cursor: pointer;
}

.avatar-cropper .upload-placeholder-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: #999;
}

.avatar-cropper .upload-hint {
  font-size: 12px;
  color: #666;
}

.avatar-cropper .control-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 24px;
  min-width: 0;
}

.avatar-cropper .preview-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.avatar-cropper .preview-label {
  font-size: 14px;
  color: #595959;
  font-weight: 500;
}

.avatar-cropper .preview-circle {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  overflow: hidden;
  border: 2px solid #e8e8e8;
  background: #f5f5f5;
}

.avatar-cropper .preview-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-cropper .preview-tip {
  font-size: 12px;
  color: #999;
  text-align: center;
  line-height: 1.5;
}

.avatar-cropper .control-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.avatar-cropper .control-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.avatar-cropper .control-label {
  width: 56px;
  font-size: 13px;
  color: #595959;
  flex-shrink: 0;
}

.avatar-cropper .btn-group {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.avatar-cropper .reselect-btn {
  cursor: pointer;
}
</style>
