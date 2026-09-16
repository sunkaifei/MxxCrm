<script lang="ts" setup>
/**
 * 状态栏（设计文档 §8.8、§34.4）。
 *
 * 注意：编辑态的页码显示为 **「设计稿 · 单页母版」**，而不是「1/1」——
 * 那张纸描述的是"元素怎么排"，不是"某张单据的第 1 页"（§34.4 语义澄清）。
 */
import { computed } from 'vue';

import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';
import { fmtMm } from '#/composables/pdf-designer/useUnits';

const store = useDesignerStore();

const selectedText = computed(() => {
  if (store.selection.length === 0) return '未选中';
  if (store.selection.length > 1) return `已选中 ${store.selection.length} 个元素`;
  const e = store.singleSelected;
  if (!e) return '未选中';
  return `${e.name}  x:${fmtMm(e.x)} y:${fmtMm(e.y)} w:${fmtMm(e.w)} h:${fmtMm(e.h)}`;
});

const outCount = computed(() => store.outOfPaperCount());

const issueCount = computed(
  () => store.issues.filter((i: any) => i.level === 'error').length,
);
</script>

<template>
  <div class="pdf-statusbar">
    <span class="sb-item sb-page">设计稿 · 单页母版</span>
    <span class="sb-sep" />
    <span class="sb-item">{{ selectedText }}</span>
    <span class="sb-sep" />
    <span class="sb-item">纸张 {{ fmtMm(store.pageWidth, 0) }} × {{ fmtMm(store.pageHeight, 0) }}</span>
    <span class="sb-sep" />
    <span class="sb-item">缩放 {{ Math.round(store.zoom * 100) }}%</span>
    <span class="sb-spacer" />
    <span v-if="outCount > 0" class="sb-item sb-warn" title="纸外元素在正式出图时会被裁切">
      ⚠ {{ outCount }} 个元素在纸外
    </span>
    <span v-if="issueCount > 0" class="sb-item sb-error" title="打开「精确预览」查看详情">
      ✕ {{ issueCount }} 项绑定错误
    </span>
    <span class="sb-sep" />
    <span class="sb-item sb-mode">
      数据：{{ store.dataMode === 'sample' ? `样例·${store.samplePreset}·${store.itemRows}行` : store.dataMode }}
    </span>
  </div>
</template>

<style scoped>
.pdf-statusbar {
  display: flex;
  gap: 8px;
  align-items: center;
  height: 26px;
  padding: 0 12px;
  font-size: 11px;
  color: #8c8c8c;
  background: #fafafa;
  border-top: 1px solid #f0f0f0;
}
.sb-sep {
  width: 1px;
  height: 12px;
  background: #e8e8e8;
}
.sb-spacer {
  flex: 1;
}
.sb-page {
  color: #1677ff;
}
.sb-warn {
  color: #d46b08;
}
.sb-error {
  color: #cf1322;
}
.sb-mode {
  color: #595959;
}
</style>
