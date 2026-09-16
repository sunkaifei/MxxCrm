<script lang="ts" setup>
/**
 * 精确预览（设计文档 §1.2 一致性闭环、§25.3）。
 *
 * 作用：用**真实出图链路**（layout_to_typst → typst-svg）编译当前页，
 * 把 SVG 半透明叠加在设计态画布上比对，把字体度量偏差**拦在设计阶段**，
 * 而不是留到客户收到的 PDF 上。
 */
import { computed, ref, watch } from 'vue';

import { Button, Modal, Slider, Spin, Tag } from 'ant-design-vue';

import { pdfPrecisePreviewApi } from '#/api/core/system/pdf-designer';
import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';

const visible = defineModel<boolean>('open', { default: false });

const store = useDesignerStore();
const loading = ref(false);
const svg = ref('');
const warnings = ref<string[]>([]);
const health = ref<any>(null);
const compileMs = ref(0);
const opacity = ref(0.5);
const showOverlay = ref(true);
const errorMsg = ref('');

const pageCount = ref(1);

async function load() {
  loading.value = true;
  errorMsg.value = '';
  svg.value = '';
  try {
    const res: any = await pdfPrecisePreviewApi({
      templateId: store.templateId ?? undefined,
      layoutJson: store.toPayload(),
      docType: store.docType,
      docId: store.realDocId ?? 0,
      dataMode: store.dataMode === 'real' ? 'real' : 'sample',
      preset: store.samplePreset,
      itemRows: store.itemRows,
      pages: 1,
      format: 'svg',
    });
    const data = res?.data ?? res;
    svg.value = data?.svg ?? '';
    warnings.value = data?.warnings ?? [];
    health.value = data?.health ?? null;
    compileMs.value = data?.compileMs ?? 0;
    pageCount.value = data?.pageCount ?? 1;
    store.issues = (data?.warnings ?? []).map((w: string) => ({
      level: 'warn',
      message: w,
    }));
  } catch (e: any) {
    errorMsg.value = e?.message ?? '编译失败';
  } finally {
    loading.value = false;
  }
}

watch(visible, (v) => {
  if (v) load();
});

const healthItems = computed(() => {
  const h = health.value;
  if (!h) return [] as Array<{ label: string; ok: boolean }>;
  return [
    { label: '跨页分页点已计算', ok: typeof h.pageBreakY === 'number' },
    { label: '无多余空白尾页', ok: !h.trailingBlankPage },
    { label: '合计位于末页', ok: !h.grandTotalNotOnLastPage },
  ];
});
</script>

<template>
  <Modal
    v-model:open="visible"
    :footer="null"
    :width="1100"
    title="精确预览（真实出图链路编译）"
    wrap-class-name="pdf-preview-modal"
  >
    <div class="pv-toolbar">
      <Button size="small" :loading="loading" @click="load">重新编译</Button>
      <span class="pv-tag">
        <Tag :color="compileMs > 300 ? 'orange' : 'green'">编译 {{ compileMs }}ms</Tag>
        <Tag>共 {{ pageCount }} 页</Tag>
      </span>
      <span class="pv-overlay">
        <span>叠加</span>
        <Slider v-model:value="opacity" :max="1" :min="0" :step="0.05" style="width: 120px" />
      </span>
      <Button size="small" @click="showOverlay = !showOverlay">
        {{ showOverlay ? '隐藏叠加' : '显示叠加' }}
      </Button>
    </div>

    <div v-if="errorMsg" class="pv-error">{{ errorMsg }}</div>

    <div class="pv-body">
      <Spin :spinning="loading" tip="正在用 Typst 编译…">
        <div class="pv-paper-wrap">
          <!-- 设计态底稿（灰） -->
          <div class="pv-paper pv-paper-design">
            <div class="pv-placeholder">设计态预览见左侧画布</div>
          </div>
          <!-- 真实编译结果叠加 -->
          <div
            v-if="svg && showOverlay"
            class="pv-paper pv-paper-svg"
            :style="{ opacity }"
            v-html="svg"
          />
          <!-- 仅看编译结果 -->
          <div v-if="svg && !showOverlay" class="pv-paper pv-paper-svg" v-html="svg" />
        </div>
      </Spin>

      <div class="pv-side">
        <div class="pv-side-title">分页健康检查</div>
        <div v-for="h in healthItems" :key="h.label" class="pv-health">
          <span :class="h.ok ? 'pv-ok' : 'pv-bad'">{{ h.ok ? '✓' : '✕' }}</span>
          <span>{{ h.label }}</span>
        </div>
        <div v-if="healthItems.length === 0" class="pv-hint">
          编译成功后显示跨页与合计落位判定。
        </div>

        <div class="pv-side-title">编译告警（{{ warnings.length }}）</div>
        <div v-if="warnings.length === 0" class="pv-hint">无告警</div>
        <div v-for="(w, i) in warnings" :key="i" class="pv-warn-item">{{ w }}</div>

        <div class="pv-side-title">说明</div>
        <ul class="pv-tips">
          <li>此处即「正式出图」的同一条链路，所见即最终结果。</li>
          <li>设计态与编译结果若有位移差异，应调整元素位置直至贴合。</li>
          <li>数据视图跟随工具栏设置（当前：{{ store.dataMode }}）。</li>
        </ul>
      </div>
    </div>
  </Modal>
</template>

<style scoped>
.pv-toolbar {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 10px;
}
.pv-tag {
  display: flex;
  gap: 4px;
}
.pv-overlay {
  display: flex;
  gap: 6px;
  align-items: center;
  font-size: 12px;
  color: #8c8c8c;
}
.pv-error {
  padding: 10px;
  margin-bottom: 10px;
  font-size: 12px;
  color: #cf1322;
  background: #fff1f0;
  border: 1px solid #ffccc7;
  border-radius: 4px;
}
.pv-body {
  display: flex;
  gap: 14px;
  max-height: 70vh;
}
.pv-paper-wrap {
  position: relative;
  flex: 1;
  overflow: auto;
  background: #f0f2f5;
  border-radius: 4px;
}
.pv-paper {
  width: 794px;
  min-height: 1123px;
  margin: 12px auto;
  background: #fff;
  box-shadow: 0 2px 8px rgb(0 0 0 / 12%);
}
.pv-paper-design {
  position: absolute;
  top: 12px;
  left: 50%;
  margin: 0;
  transform: translateX(-50%);
  border: 1px dashed #d9d9d9;
}
.pv-paper-svg {
  position: relative;
  z-index: 2;
  background: transparent;
  box-shadow: none;
}
.pv-paper-svg :deep(svg) {
  width: 100%;
  height: auto;
}
.pv-placeholder {
  padding-top: 40px;
  font-size: 12px;
  color: #d9d9d9;
  text-align: center;
}
.pv-side {
  width: 300px;
  padding: 8px;
  overflow-y: auto;
  background: #fafafa;
  border-radius: 4px;
}
.pv-side-title {
  margin: 10px 0 6px;
  font-size: 12px;
  font-weight: 600;
  color: #595959;
}
.pv-side-title:first-child {
  margin-top: 0;
}
.pv-health {
  display: flex;
  gap: 6px;
  align-items: center;
  padding: 3px 0;
  font-size: 12px;
}
.pv-ok {
  color: #389e0d;
}
.pv-bad {
  color: #cf1322;
}
.pv-warn-item {
  padding: 4px 6px;
  margin-bottom: 4px;
  font-size: 11px;
  line-height: 1.5;
  color: #d46b08;
  background: #fff7e6;
  border-radius: 4px;
}
.pv-hint {
  font-size: 11px;
  color: #bfbfbf;
}
.pv-tips {
  padding-left: 16px;
  margin: 0;
  font-size: 11px;
  line-height: 1.8;
  color: #8c8c8c;
}
</style>
