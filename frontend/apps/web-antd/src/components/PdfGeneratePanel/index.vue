<script lang="ts" setup>
import type { PdfRecordVO } from '#/api/core/system/pdf';
import type { PdfTemplateOptionVO } from '#/api/core/system/pdf-template';

import { computed, onMounted, ref } from 'vue';

import { Button, message, Modal, Select, Tag } from 'ant-design-vue';

import {
  downloadPdfApi,
  generatePdfApi,
  getPdfRecordListApi,
  previewPdfApi,
} from '#/api/core/system/pdf';
import { getPdfTemplateOptionsApi } from '#/api/core/system/pdf-template';

interface Props {
  /** 单据类型：quotation/order/contract */
  docType: string;
  /** 单据 ID */
  docId: number | string;
}

const props = defineProps<Props>();

// 模板选项
const selectedTemplateId = ref<number | undefined>();
const templateOptions = ref<{ label: string; value: number }[]>([]);

// 历史记录
const records = ref<PdfRecordVO[]>([]);

// 操作状态
const generating = ref(false);
const previewing = ref(false);
const previewVisible = ref(false);
const previewSource = ref('');

// 最新已生成的 PDF（用于头部状态展示）
const latestPdf = computed(() => {
  return records.value.find((r) => r.status !== 0);
});

// 加载模板列表
async function loadTemplateOptions() {
  try {
    const res: any = await getPdfTemplateOptionsApi(props.docType);
    const list: PdfTemplateOptionVO[] = Array.isArray(res)
      ? res
      : (res?.items ?? []);
    templateOptions.value = list.map((t: any) => ({
      label: t.name || t.templateName || t.label || String(t.id),
      value: t.id,
    }));
    if (!selectedTemplateId.value && templateOptions.value.length > 0) {
      selectedTemplateId.value = templateOptions.value[0]?.value;
    }
  } catch (error) {
    console.error('加载 PDF 模板列表失败:', error);
  }
}

// 加载历史记录
async function loadRecords() {
  try {
    const res: any = await getPdfRecordListApi({
      docType: props.docType,
      docId: props.docId,
    });
    records.value = Array.isArray(res) ? res : (res?.items ?? []);
  } catch (error) {
    console.error('加载 PDF 历史记录失败:', error);
  }
}

// 生成 PDF
async function handleGenerate() {
  if (!selectedTemplateId.value) {
    message.warning('请先选择模板');
    return;
  }
  generating.value = true;
  try {
    await generatePdfApi({
      docType: props.docType,
      docId: props.docId,
      templateId: selectedTemplateId.value,
    });
    message.success('PDF 生成成功');
    await loadRecords();
  } catch (error: any) {
    console.error('生成 PDF 失败:', error);
    message.error(error?.message || 'PDF 生成失败');
  } finally {
    generating.value = false;
  }
}

// 预览（弹出 Modal 显示 typst 源码）
async function handlePreview() {
  if (!selectedTemplateId.value) {
    message.warning('请先选择模板');
    return;
  }
  previewing.value = true;
  try {
    const res: any = await previewPdfApi({
      docType: props.docType,
      docId: props.docId,
      templateId: selectedTemplateId.value,
    });
    previewSource.value =
      typeof res === 'string'
        ? res
        : (res?.source ??
          res?.typst ??
          res?.content ??
          JSON.stringify(res, null, 2));
    previewVisible.value = true;
  } catch (error: any) {
    console.error('预览 PDF 失败:', error);
    message.error(error?.message || '预览失败');
  } finally {
    previewing.value = false;
  }
}

// 下载
async function handleDownload(record: PdfRecordVO) {
  try {
    const blob: any = await downloadPdfApi(record.id);
    const blobData = blob instanceof Blob ? blob : new Blob([blob]);
    const url = window.URL.createObjectURL(blobData);
    const link = document.createElement('a');
    link.href = url;
    link.download = record.fileName || 'document.pdf';
    document.body.append(link);
    link.click();
    link.remove();
    window.URL.revokeObjectURL(url);
  } catch (error: any) {
    console.error('下载 PDF 失败:', error);
    message.error(error?.message || '下载失败');
  }
}

onMounted(() => {
  loadTemplateOptions();
  loadRecords();
});
</script>

<template>
  <div class="pdf-generate-panel">
    <div class="panel-header">
      <span class="title">PDF 文件</span>
      <Tag v-if="latestPdf" color="green">已生成</Tag>
      <Tag v-else color="default">未生成</Tag>
    </div>

    <div class="template-select">
      <Select
        v-model:value="selectedTemplateId"
        :options="templateOptions"
        placeholder="选择模板"
        style="width: 240px"
      />
      <Button :loading="previewing" @click="handlePreview">预览</Button>
      <Button type="primary" :loading="generating" @click="handleGenerate">
        生成 PDF
      </Button>
    </div>

    <div v-if="records.length > 0" class="history-list">
      <div v-for="record in records" :key="record.id" class="history-item">
        <span class="history-time">{{ record.createTime }}</span>
        <Tag v-if="record.triggerType === 'auto'" color="blue">自动</Tag>
        <Tag v-else color="orange">手动</Tag>
        <span v-if="record.status === 0" class="status-failed">失败</span>
        <Button type="link" size="small" @click="handleDownload(record)">
          下载
        </Button>
      </div>
    </div>

    <Modal
      v-model:open="previewVisible"
      title="Typst 源码预览"
      :width="800"
      :footer="null"
      :destroy-on-close="true"
    >
      <pre class="typst-source">{{ previewSource }}</pre>
    </Modal>
  </div>
</template>

<style scoped>
.pdf-generate-panel {
  padding: 16px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.panel-header {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 12px;
}

.panel-header .title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.template-select {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 12px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px dashed hsl(var(--border));
}

.history-item {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 6px 0;
  font-size: 13px;
}

.history-time {
  min-width: 160px;
  color: hsl(var(--muted-foreground));
}

.status-failed {
  font-size: 12px;
  color: #ef4444;
}

.typst-source {
  max-height: 60vh;
  padding: 12px;
  margin: 0;
  overflow: auto;
  font-family: 'Fira Code', Consolas, Monaco, monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #e2e8f0;
  word-break: break-all;
  white-space: pre-wrap;
  background: #1e293b;
  border-radius: 6px;
}
</style>
