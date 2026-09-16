<script lang="ts" setup>
/**
 * 导入 PDF → 反推模板（设计文档 §9 阶段 A：底图模式）。
 *
 * 流程：
 * 1. 浏览器用 pdfjs 把选中的页渲染成 PNG（保持原始页面尺寸比例）；
 * 2. PNG 上传到素材库（category=background）；
 * 3. 调 `parse-pdf` 生成"底图铺满纸张 + 预置明细表骨架"的可拖拽草稿；
 * 4. 用户直接在底图上对齐文字——**所见即所得 100% 还原**，无需自动识别元素。
 *
 * 为什么不追求全自动元素识别：业界共识是"自动打底 + 人工校准"，
 * 元素识别列为 v2 能力（§9.3）。
 */
import { computed, ref, watch , markRaw } from 'vue';

import { Alert, Button, InputNumber, Modal, Radio, Spin, Upload } from 'ant-design-vue';
import { LucideUpload } from '@vben/icons';

import {
  parsePdfToLayoutApi,
  uploadPdfAssetApi,
} from '#/api/core/system/pdf-designer';

const visible = defineModel<boolean>('open', { default: false });

const props = defineProps<{ docType: string }>();

const emit = defineEmits<{
  (e: 'done', layoutJson: any): void;
}>();

const step = ref<'done' | 'render' | 'select' | 'upload'>('select');
const fileName = ref('');
const pdfDoc = ref<any>(null);
const pageCount = ref(0);
const pageNo = ref(1);
const scale = ref(2);
const previewUrl = ref('');
const pageSize = ref({ w: 0, h: 0 });
const errorMsg = ref('');
const warnings = ref<string[]>([]);
const busy = ref(false);

const mode = ref<'background' | 'blank'>('background');

// §9 已知坑：workerSrc 用 URL 指向 worker 文件时，Vite 下会加载出
// **第二份 pdfjs 类副本**（假 worker 回退在主线程 import worker 文件），
// 与主线程预打包副本私有字段错位 → `#pagePromises` 报错。
// 改用 Vite 原生 `?worker`：由打包器构建 worker 并以 workerPort 直连，
// 主/worker 版本恒配对。
// @ts-ignore ?worker 导入无类型声明
import PdfWorker from 'pdfjs-dist/build/pdf.worker.min.mjs?worker';

async function getPdfjs() {
  const pdfjs: any = await import('pdfjs-dist');
  pdfjs.GlobalWorkerOptions.workerPort = new PdfWorker();
  return pdfjs;
}

function beforeUpload(file: File) {
  fileName.value = file.name;
  void openPdf(file);
  // 阻止 antd 默认上传行为
  return false;
}

async function openPdf(file: File) {
  errorMsg.value = '';
  busy.value = true;
  step.value = 'render';
  try {
    const pdfjs = await getPdfjs();
    const buf = await file.arrayBuffer();
    const doc = await pdfjs.getDocument({ data: new Uint8Array(buf) }).promise;
    pdfDoc.value = markRaw(doc);
    pageCount.value = doc.numPages;
    pageNo.value = 1;
    await renderPage();
    step.value = 'upload';
  } catch (e: any) {
    console.error('[pdfimport] stack', e?.stack || e);
    errorMsg.value = e?.message ?? 'PDF 解析失败';
    step.value = 'select';
  } finally {
    busy.value = false;
  }
}

async function renderPage() {
  if (!pdfDoc.value) return;
  const page = await pdfDoc.value.getPage(pageNo.value);
  const viewport = page.getViewport({ scale: scale.value });
  const canvas = document.createElement('canvas');
  canvas.width = Math.floor(viewport.width);
  canvas.height = Math.floor(viewport.height);
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('无法创建画布上下文');
  await page.render({ canvasContext: ctx, viewport }).promise;
  previewUrl.value = canvas.toDataURL('image/png');
  // 记录原始页面尺寸（pt → mm），用于与 A4 对齐
  const base = page.getViewport({ scale: 1 });
  pageSize.value = { w: (base.width / 72) * 25.4, h: (base.height / 72) * 25.4 };
}

watch(pageNo, () => void renderPage());

/** 把 dataURL 转 Blob（避免 base64 体积膨胀 33%） */
async function dataUrlToFile(dataUrl: string, name: string): Promise<File> {
  const resp = await fetch(dataUrl);
  const blob = await resp.blob();
  return new File([blob], name, { type: 'image/png' });
}

async function submit() {
  busy.value = true;
  errorMsg.value = '';
  try {
    if (mode.value === 'blank') {
      // 不铺底图：直接生成空白 A4 草稿
      emit('done', null);
      visible.value = false;
      return;
    }
    if (!previewUrl.value) throw new Error('请先渲染页面');
    const file = await dataUrlToFile(
      previewUrl.value,
      `${fileName.value.replace(/\.pdf$/i, '')}_p${pageNo.value}.png`,
    );
    const asset: any = await uploadPdfAssetApi(
      file,
      'background',
      `${fileName.value} 第${pageNo.value}页`,
    );
    const assetId = Number(asset?.id ?? asset?.data?.id);
    if (!assetId) throw new Error('底图上传失败，未返回素材 ID');

    const res: any = await parsePdfToLayoutApi({
      assetId,
      docType: props.docType,
      mode: 'background',
      pageWidthMm: pageSize.value.w || 210,
      pageHeightMm: pageSize.value.h || 297,
    });
    const data = res?.data ?? res;
    warnings.value = data?.warnings ?? [];
    step.value = 'done';
    emit('done', data?.layoutJson ?? null);
    if (warnings.value.length === 0) visible.value = false;
  } catch (e: any) {
    errorMsg.value = e?.message ?? '生成模板失败';
  } finally {
    busy.value = false;
  }
}

const canSubmit = computed(() => !busy.value && (mode.value === 'blank' || !!previewUrl.value));
</script>

<template>
  <Modal
    v-model:open="visible"
    :confirm-loading="busy"
    :ok-button-props="{ disabled: !canSubmit }"
    :ok-text="step === 'done' ? '完成' : '生成可编辑模板'"
    :width="980"
    title="导入 PDF 反推模板（底图模式）"
    @ok="submit"
  >
    <Alert
      message="底图模式：整页渲染成底图，在其上拖拽摆放字段，视觉 100% 还原。元素自动识别为 v2 能力。"
      show-icon
      type="info"
    />

    <div v-if="errorMsg" class="ip-error">{{ errorMsg }}</div>

    <div class="ip-body">
      <div class="ip-left">
        <Upload :before-upload="beforeUpload" :show-upload-list="false" accept=".pdf">
          <Button :loading="busy" type="primary">
            <LucideUpload class="ip-icon" />选择 PDF 文件
          </Button>
        </Upload>
        <div v-if="fileName" class="ip-file">{{ fileName }}</div>

        <div class="ip-row">
          <span class="ip-label">来源页</span>
          <InputNumber
            v-model:value="pageNo"
            :disabled="pageCount <= 1"
            :max="pageCount"
            :min="1"
            size="small"
          />
          <span class="ip-muted">/ {{ pageCount || '-' }}</span>
        </div>
        <div class="ip-row">
          <span class="ip-label">渲染倍率</span>
          <InputNumber
            v-model:value="scale"
            :max="4"
            :min="1"
            :step="0.5"
            size="small"
            @change="() => renderPage()"
          />
        </div>
        <div class="ip-row">
          <span class="ip-label">纸张</span>
          <Radio.Group v-model:value="mode" size="small">
            <Radio value="background">铺底图</Radio>
            <Radio value="blank">空白 A4</Radio>
          </Radio.Group>
        </div>

        <div v-if="pageSize.w" class="ip-size">
          源页尺寸：{{ pageSize.w.toFixed(1) }} × {{ pageSize.h.toFixed(1) }} mm
          <span v-if="Math.abs(pageSize.w - 210) > 2 || Math.abs(pageSize.h - 297) > 2" class="ip-warn">
            （非 A4，已在模板中按源尺寸建纸）
          </span>
        </div>

        <div v-if="warnings.length > 0" class="ip-warnings">
          <div v-for="(w, i) in warnings" :key="i">· {{ w }}</div>
        </div>
      </div>

      <div class="ip-right">
        <Spin :spinning="busy" tip="正在渲染页面…">
          <div v-if="previewUrl" class="ip-preview">
            <img :src="previewUrl" alt="pdf page" />
          </div>
          <div v-else class="ip-preview-empty">
            选择 PDF 后在此预览将作为底图的页面
          </div>
        </Spin>
      </div>
    </div>
  </Modal>
</template>

<style scoped>
.ip-error {
  padding: 8px 10px;
  margin-top: 10px;
  font-size: 12px;
  color: #cf1322;
  background: #fff1f0;
  border: 1px solid #ffccc7;
  border-radius: 4px;
}
.ip-body {
  display: flex;
  gap: 16px;
  margin-top: 12px;
}
.ip-left {
  width: 300px;
}
.ip-icon {
  width: 14px;
  height: 14px;
}
.ip-file {
  margin-top: 8px;
  font-size: 12px;
  color: #595959;
  word-break: break-all;
}
.ip-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-top: 10px;
}
.ip-label {
  width: 62px;
  font-size: 12px;
  color: #8c8c8c;
}
.ip-muted {
  font-size: 12px;
  color: #bfbfbf;
}
.ip-size {
  margin-top: 10px;
  font-size: 11px;
  color: #8c8c8c;
}
.ip-warn {
  color: #d46b08;
}
.ip-warnings {
  padding: 8px;
  margin-top: 10px;
  font-size: 11px;
  line-height: 1.7;
  color: #d46b08;
  background: #fff7e6;
  border-radius: 4px;
}
.ip-right {
  flex: 1;
  overflow: auto;
  background: #f0f2f5;
  border-radius: 4px;
}
.ip-preview {
  padding: 12px;
  text-align: center;
}
.ip-preview img {
  max-width: 100%;
  background: #fff;
  box-shadow: 0 2px 8px rgb(0 0 0 / 12%);
}
.ip-preview-empty {
  padding: 120px 20px;
  font-size: 12px;
  color: #bfbfbf;
  text-align: center;
}
</style>
