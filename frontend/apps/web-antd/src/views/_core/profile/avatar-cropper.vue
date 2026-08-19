<script setup lang="ts">
import { computed, ref } from 'vue';
import { Cropper } from 'vue-advanced-cropper';
import 'vue-advanced-cropper/dist/style.css';

import { Button, message, Modal, Slider } from 'ant-design-vue';

import { uploadFileApi } from '#/api/core/attachment/file';
import { $t } from '#/locales';

const props = defineProps<{
  avatarUrl?: string;
  visible: boolean;
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
    message.error($t('page.system.profile.cropper.selectImageFile'));
    return;
  }

  const reader = new FileReader();
  reader.addEventListener('load', (event) => {
    imageSrc.value = event.target?.result as string;
    previewUrl.value = '';
    zoom.value = 1;
    currentZoom.value = 1;
  });
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

const handleZoomChange = (val: [number, number] | number) => {
  // Slider 的 change 事件值类型为 number | [number, number]，非区间模式取第一个值
  const zoomVal = Array.isArray(val) ? (val[0] ?? 0) : val;
  if (cropperRef.value && currentZoom.value > 0) {
    const ratio = zoomVal / currentZoom.value;
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
      message.error($t('page.system.profile.cropper.cropFailed'));
      uploading.value = false;
      return;
    }

    result.canvas.toBlob(
      async (blob: Blob | null) => {
        if (!blob) {
          message.error($t('page.system.profile.cropper.generateFailed'));
          uploading.value = false;
          return;
        }

        const file = new File([blob], 'avatar.jpg', { type: 'image/jpeg' });
        const res: any = await uploadFileApi(file, 'avatar');

        if (res?.url || res?.uploadUrl) {
          const url = res.url || res.uploadUrl;
          message.success($t('page.system.profile.cropper.uploadSuccess'));
          emit('success', url);
          handleClose();
        } else {
          message.error($t('page.system.profile.cropper.uploadFailed'));
        }
        uploading.value = false;
      },
      'image/jpeg',
      0.92,
    );
  } catch (error: any) {
    message.error(error?.message || $t('page.system.profile.cropper.uploadFailed'));
    uploading.value = false;
  }
};
</script>

<template>
  <Modal
    v-model:open="visible"
    :title="$t('page.system.profile.cropper.title')"
    :width="680"
    :confirm-loading="uploading"
    @ok="handleConfirm"
    @cancel="handleClose"
    :ok-text="$t('page.system.profile.cropper.confirmUpload')"
    :cancel-text="$t('page.system.profile.cropper.cancel')"
    :z-index="2000"
  >
    <div class="avatar-cropper">
      <!-- 裁剪区域 -->
      <div class="cropper-wrapper">
        <div v-if="!imageSrc" class="upload-placeholder">
          <label class="upload-btn">
            <input
              type="file"
              accept="image/*"
              @change="handleChooseFile"
              hidden
            />
            <div class="upload-placeholder-content">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="48"
                height="48"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="17 8 12 3 7 8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
              <span>{{ $t('page.system.profile.cropper.clickSelect') }}</span>
              <span class="upload-hint">{{ $t('page.system.profile.cropper.formatHint') }}</span>
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
          <div class="preview-label">{{ $t('page.system.profile.cropper.preview') }}</div>
          <div class="preview-circle">
            <img v-if="previewUrl" :src="previewUrl" class="preview-img" />
          </div>
          <span class="preview-tip">{{ $t('page.system.profile.cropper.previewTipLine1') }}<br />{{ $t('page.system.profile.cropper.previewTipLine2') }}</span>
        </div>

        <!-- 控制按钮 -->
        <div class="control-section">
          <div class="control-row">
            <span class="control-label">{{ $t('page.system.profile.cropper.zoom') }}</span>
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
            <span class="control-label">{{ $t('page.system.profile.cropper.rotate') }}</span>
            <div class="btn-group">
              <Button size="small" @click="handleRotate(-90)">-90°</Button>
              <Button size="small" @click="handleRotate(90)">+90°</Button>
            </div>
          </div>
          <div class="control-row">
            <span class="control-label">{{ $t('page.system.profile.cropper.flip') }}</span>
            <div class="btn-group">
              <Button size="small" @click="handleFlipH">{{ $t('page.system.profile.cropper.flipH') }}</Button>
              <Button size="small" @click="handleFlipV">{{ $t('page.system.profile.cropper.flipV') }}</Button>
            </div>
          </div>
          <div class="control-row">
            <span class="control-label"></span>
            <div class="btn-group">
              <Button size="small" @click="handleReset">{{ $t('page.system.profile.cropper.reset') }}</Button>
              <label class="reselect-btn">
                <input
                  type="file"
                  accept="image/*"
                  @change="handleChooseFile"
                  hidden
                />
                <Button size="small" type="primary">{{ $t('page.system.profile.cropper.reselect') }}</Button>
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
  position: relative;
  flex-shrink: 0;
  width: 380px;
  height: 380px;
  overflow: hidden;
  background: #2a2a2a;
  border-radius: 8px;
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
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  cursor: pointer;
  background: #2a2a2a;
  border: 2px dashed #4a4a4a;
  border-radius: 8px;
  transition: all 0.3s;
}

.avatar-cropper .upload-placeholder:hover {
  background: #1f2d3d;
  border-color: #1890ff;
}

.avatar-cropper .upload-btn {
  cursor: pointer;
}

.avatar-cropper .upload-placeholder-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
  color: #999;
}

.avatar-cropper .upload-hint {
  font-size: 12px;
  color: #666;
}

.avatar-cropper .control-panel {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 24px;
  min-width: 0;
}

.avatar-cropper .preview-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
}

.avatar-cropper .preview-label {
  font-size: 14px;
  font-weight: 500;
  color: #595959;
}

.avatar-cropper .preview-circle {
  width: 120px;
  height: 120px;
  overflow: hidden;
  background: #f5f5f5;
  border: 2px solid #e8e8e8;
  border-radius: 50%;
}

.avatar-cropper .preview-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-cropper .preview-tip {
  font-size: 12px;
  line-height: 1.5;
  color: #999;
  text-align: center;
}

.avatar-cropper .control-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.avatar-cropper .control-row {
  display: flex;
  gap: 12px;
  align-items: center;
}

.avatar-cropper .control-label {
  flex-shrink: 0;
  width: 56px;
  font-size: 13px;
  color: #595959;
}

.avatar-cropper .btn-group {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.avatar-cropper .reselect-btn {
  cursor: pointer;
}
</style>
