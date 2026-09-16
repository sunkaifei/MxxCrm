<script lang="ts" setup>
/**
 * 分页预览（设计文档 §34.6）。
 *
 * 「设计态单页 / 预览态多页」是文档类设计器的普遍范式：
 * 设计器只编辑**页型母版**，多页是编译**算**出来的结果。
 *
 * 三个本方案独有能力（业界无直接对应物）：
 * 1. 缩略图栏 + 主视图；
 * 2. **反查定位**：点预览里的元素 → 回编辑态选中（用 pageBreakY 做粗略映射）；
 * 3. **健康检查面板**：表头重复 / 合计落位 / 多余空白尾页自动判定，
 *    把 P0 验收标准 TC-08 从"出图后肉眼比对"变成自动化。
 */
import { computed, ref, watch } from 'vue';

import { Alert, Button, Empty, Modal, Spin, Tag } from 'ant-design-vue';

import { pdfPrecisePreviewApi, pdfPreviewSvgUrl } from '#/api/core/system/pdf-designer';
import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';

const visible = defineModel<boolean>('open', { default: false });

const emit = defineEmits<{ (e: 'locate', pageIndex: number): void }>();

const store = useDesignerStore();
const loading = ref(false);
const errorMsg = ref('');
const pages = ref<Array<{ index: number; url: string }>>([]);
const active = ref(1);
const health = ref<any>(null);
const warnings = ref<string[]>([]);
const pageBreakY = ref<number | null>(null);
const compileMs = ref(0);

async function load() {
  loading.value = true;
  errorMsg.value = '';
  pages.value = [];
  try {
    const res: any = await pdfPrecisePreviewApi({
      templateId: store.templateId ?? undefined,
      layoutJson: store.toPayload(),
      docType: store.docType,
      docId: store.realDocId ?? 0,
      dataMode: store.dataMode === 'real' ? 'real' : 'sample',
      preset: store.samplePreset,
      itemRows: store.itemRows,
      pages: 'all',
      format: 'svg',
    });
    const data = res?.data ?? res;
    pages.value = data?.pages ?? [];
    health.value = data?.health ?? null;
    warnings.value = data?.warnings ?? [];
    pageBreakY.value = data?.pageBreakY ?? null;
    compileMs.value = data?.compileMs ?? 0;
    const count = data?.pageCount ?? 1;
    store.issues = (data?.warnings ?? []).map((w: string) => ({
      level: 'warn',
      message: w,
    }));
    const token = data?.token;
    if (token && pages.value.length === 0) {
      pages.value = Array.from({ length: count }, (_v, i) => ({
        index: i + 1,
        url: pdfPreviewSvgUrl(token, i + 1),
      }));
    }
    active.value = 1;
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
  const h = health.value ?? {};
  return [
    {
      label: '表头在每页重复',
      ok: !h.headerNotRepeated,
      tip: '明细表格 headerRepeat 已开启',
    },
    {
      label: '合计落在末页',
      ok: !h.grandTotalNotOnLastPage,
      tip: '表格 footer 跟随表格末尾（§17.4）',
    },
    {
      label: '无多余空白尾页',
      ok: !h.trailingBlankPage,
      tip: '避免"恰好占满仍多出一页"（TC-16）',
    },
    {
      label: '跨页分页点已计算',
      ok: typeof pageBreakY.value === 'number',
      tip: '按表格 y 坐标切分 HEAD/TABLE/FOOT',
    },
  ];
});

function locateCurrent() {
  emit('locate', active.value);
}
</script>

<template>
  <Modal
    v-model:open="visible"
    :footer="null"
    :width="1180"
    title="分页预览（数据驱动的完整分页结果）"
  >
    <div class="pg-toolbar">
      <Button size="small" :loading="loading" @click="load">重新分页</Button>
      <Tag :color="compileMs > 800 ? 'orange' : 'green'">编译 {{ compileMs }}ms</Tag>
      <Tag color="blue">共 {{ pages.length }} 页</Tag>
      <Button size="small" @click="locateCurrent">按当前页反查编辑态</Button>
      <span class="pg-hint">
        多页由编译期根据明细行数计算得出；编辑态只有一张「页型母版」（§34.4）。
      </span>
    </div>

    <Alert
      v-if="errorMsg"
      :message="errorMsg"
      show-icon
      type="error"
      style="margin-bottom: 10px"
    />

    <div class="pg-body">
      <Spin
        :spinning="loading"
        :tip="loading ? '正在编译全部分页…' : ''"
        wrapper-class-name="pg-spin"
      >
        <div class="pg-body-inner">
          <!-- 缩略图栏 -->
          <div class="pg-thumbs">
            <div
              v-for="p in pages"
              :key="p.index"
              class="pg-thumb"
              :class="{ 'is-active': active === p.index }"
              @click="active = p.index"
            >
              <img :src="p.url" alt="page" loading="lazy" />
              <span class="pg-thumb-no">第 {{ p.index }} 页</span>
            </div>
            <Empty v-if="pages.length === 0 && !loading" :image="Empty.PRESENTED_IMAGE_SIMPLE" description="暂无分页" />
          </div>

          <!-- 主视图 -->
          <div class="pg-main">
            <img
              v-if="pages[active - 1]"
              :src="pages[active - 1]!.url"
              alt="page"
              class="pg-main-img"
            />
            <div v-else-if="!loading" class="pg-main-empty">编译后在此查看每一页</div>
          </div>

          <!-- 健康检查 -->
          <div class="pg-side">
            <div class="pg-side-title">健康检查</div>
            <div v-for="h in healthItems" :key="h.label" class="pg-health">
              <span :class="h.ok ? 'pg-ok' : 'pg-bad'">{{ h.ok ? '✓' : '✕' }}</span>
              <span class="pg-health-label">{{ h.label }}</span>
            </div>

            <div class="pg-side-title">编译告警（{{ warnings.length }}）</div>
            <div v-if="warnings.length === 0" class="pg-none">无告警</div>
            <div v-for="(w, i) in warnings" :key="i" class="pg-warn">{{ w }}</div>

            <div class="pg-side-title">分页定位</div>
            <div class="pg-none">
              明细区起始 y ≈
              {{ pageBreakY === null ? '—' : `${pageBreakY.toFixed(1)}mm` }}
            </div>
          </div>
        </div>
      </Spin>
    </div>
  </Modal>
</template>

<style scoped>
.pg-toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 10px;
}
.pg-hint {
  font-size: 11px;
  color: #bfbfbf;
}
.pg-body :deep(.pg-spin) {
  width: 100%;
}
.pg-body-inner {
  display: flex;
  gap: 12px;
  height: 64vh;
}
.pg-thumbs {
  width: 132px;
  padding: 4px;
  overflow-y: auto;
  background: #fafafa;
  border-radius: 4px;
}
.pg-thumb {
  padding: 4px;
  margin-bottom: 8px;
  cursor: pointer;
  background: #fff;
  border: 1px solid #f0f0f0;
  border-radius: 4px;
}
.pg-thumb.is-active {
  border-color: #1677ff;
  box-shadow: 0 0 0 2px rgb(22 119 255 / 12%);
}
.pg-thumb img {
  display: block;
  width: 100%;
}
.pg-thumb-no {
  display: block;
  margin-top: 2px;
  font-size: 10px;
  color: #8c8c8c;
  text-align: center;
}
.pg-main {
  display: flex;
  flex: 1;
  align-items: flex-start;
  justify-content: center;
  overflow: auto;
  background: #f0f2f5;
  border-radius: 4px;
}
.pg-main-img {
  width: 100%;
  max-width: 720px;
  margin: 12px;
  background: #fff;
  box-shadow: 0 2px 8px rgb(0 0 0 / 12%);
}
.pg-main-empty {
  padding-top: 80px;
  font-size: 12px;
  color: #bfbfbf;
}
.pg-side {
  width: 250px;
  padding: 8px;
  overflow-y: auto;
  background: #fafafa;
  border-radius: 4px;
}
.pg-side-title {
  margin: 10px 0 6px;
  font-size: 12px;
  font-weight: 600;
  color: #595959;
}
.pg-side-title:first-child {
  margin-top: 0;
}
.pg-health {
  display: flex;
  gap: 6px;
  align-items: center;
  padding: 3px 0;
  font-size: 12px;
}
.pg-health-label {
  flex: 1;
}
.pg-ok {
  color: #389e0d;
}
.pg-bad {
  color: #cf1322;
}
.pg-warn {
  padding: 4px 6px;
  margin-bottom: 4px;
  font-size: 11px;
  line-height: 1.5;
  color: #d46b08;
  background: #fff7e6;
  border-radius: 4px;
}
.pg-none {
  font-size: 11px;
  color: #bfbfbf;
}
</style>
