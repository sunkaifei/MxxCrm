<script lang="ts" setup>
/**
 * 顶部工具栏（设计文档 §8.3）。
 *
 * 分组：模板 | 编辑 | 排列 | 视图 | 数据视图 | 预览 | 输出
 * 数据视图组是 §32 的入口：占位符 / 样例 / 真实 / 轮廓 四模式 + 样本预设集。
 */
import type { PdfDataMode, PdfSamplePreset } from '#/types/pdf-layout';

import { computed } from 'vue';

import {
  LucideAlignCenterHorizontal,
  LucideAlignCenterVertical,
  LucideAlignEndHorizontal,
  LucideAlignEndVertical,
  LucideAlignStartHorizontal,
  LucideAlignStartVertical,
  LucideArrowLeft,
  LucideCopy,
  LucideDownload,
  LucideFileInput,
  LucideGrid3x3,
  LucideHistory,
  LucideLayoutGrid,
  LucideMagnet,
  LucideMaximize2,
  LucideMonitor,
  LucideRedo2,
  LucideSave,
  LucideShieldCheck,
  LucideSparkles,
  LucideStar,
  LucideUndo2,
  LucideUpload,
  LucideZoomIn,
  LucideZoomOut,
} from '@vben/icons';

import { Button, Divider, InputNumber, Select, Tooltip } from 'ant-design-vue';

import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';

const emit = defineEmits<{
  (e: 'align', kind: string): void;
  (e: 'back'): void;
  (e: 'data-change'): void;
  (e: 'export'): void;
  (e: 'import'): void;
  (e: 'import-pdf'): void;
  (e: 'page-preview'): void;
  (e: 'precise-preview'): void;
  (e: 'save'): void;
  (e: 'save-as'): void;
  (e: 'set-default'): void;
  (e: 'versions'): void;
}>();

const store = useDesignerStore();

const MODES: Array<{ icon: any; label: string; value: PdfDataMode }> = [
  { value: 'sample', label: '样例', icon: LucideSparkles },
  { value: 'placeholder', label: '占位符', icon: LucideLayoutGrid },
  { value: 'real', label: '真实', icon: LucideShieldCheck },
  { value: 'outline', label: '轮廓', icon: LucideMonitor },
];

const PRESETS: Array<{ label: string; value: PdfSamplePreset }> = [
  { value: 'typical', label: '典型' },
  { value: 'long', label: '超长' },
  { value: 'empty', label: '空值' },
  { value: 'zero', label: '零值' },
  { value: 'mixed', label: '混合' },
  { value: 'unicode', label: '特殊字符' },
  { value: 'longlist', label: '长明细' },
];

const zoomPercent = computed({
  get: () => Math.round(store.zoom * 100),
  set: (v: number) => store.setZoom((v || 100) / 100),
});

const canAlign = computed(() => store.selection.length >= 1);
const canDistribute = computed(() => store.selection.length >= 3);

/** 数据视图切换时的联动：长明细预设自动切到 20 行 */
function setMode(m: PdfDataMode) {
  store.dataMode = m;
  emit('data-change');
  if (m === 'real' && !store.realDocId) {
    // 由父级弹出单据选择器
    emit('precise-preview');
  }
}

function setPreset(p: PdfSamplePreset) {
  store.samplePreset = p;
  if (p === 'empty') store.itemRows = 0;
  else if (p === 'longlist') store.itemRows = 20;
  else if (store.itemRows === 0 || store.itemRows === 20) store.itemRows = 3;
  emit('data-change');
}
</script>

<template>
  <div class="pdf-toolbar">
    <!-- 模板 -->
    <div class="tb-group">
      <Tooltip title="返回模板列表">
        <Button size="small" type="text" @click="emit('back')">
          <LucideArrowLeft class="tb-icon" />
        </Button>
      </Tooltip>
      <span class="tb-name" :title="store.templateName">
        {{ store.templateName || '未命名模板' }}
      </span>
      <span v-if="store.dirty" class="tb-dot" title="有未保存的修改">●</span>
      <span class="tb-ver">v{{ store.version }}</span>
    </div>

    <Divider type="vertical" />

    <!-- 保存 -->
    <div class="tb-group">
      <Button size="small" type="primary" @click="emit('save')">
        <LucideSave class="tb-icon" />保存
      </Button>
      <Tooltip title="另存为新模板">
        <Button size="small" @click="emit('save-as')">
          <LucideCopy class="tb-icon" />另存为
        </Button>
      </Tooltip>
    </div>

    <Divider type="vertical" />

    <!-- 撤销重做 -->
    <div class="tb-group">
      <Tooltip :title="`撤销 (Ctrl+Z)`">
        <Button :disabled="!store.canUndo" size="small" type="text" @click="store.undo()">
          <LucideUndo2 class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip :title="`重做 (Ctrl+Shift+Z)`">
        <Button :disabled="!store.canRedo" size="small" type="text" @click="store.redo()">
          <LucideRedo2 class="tb-icon" />
        </Button>
      </Tooltip>
    </div>

    <Divider type="vertical" />

    <!-- 排列 -->
    <div class="tb-group">
      <Tooltip title="左对齐">
        <Button :disabled="!canAlign" size="small" type="text" @click="emit('align', 'left')">
          <LucideAlignStartVertical class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip title="水平居中">
        <Button :disabled="!canAlign" size="small" type="text" @click="emit('align', 'hcenter')">
          <LucideAlignCenterVertical class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip title="右对齐">
        <Button :disabled="!canAlign" size="small" type="text" @click="emit('align', 'right')">
          <LucideAlignEndVertical class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip title="顶对齐">
        <Button :disabled="!canAlign" size="small" type="text" @click="emit('align', 'top')">
          <LucideAlignStartHorizontal class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip title="垂直居中">
        <Button :disabled="!canAlign" size="small" type="text" @click="emit('align', 'vcenter')">
          <LucideAlignCenterHorizontal class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip title="底对齐">
        <Button :disabled="!canAlign" size="small" type="text" @click="emit('align', 'bottom')">
          <LucideAlignEndHorizontal class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip title="水平等距分布">
        <Button :disabled="!canDistribute" size="small" type="text" @click="emit('align', 'dist-h')">
          <LucideLayoutGrid class="tb-icon" />
        </Button>
      </Tooltip>
    </div>

    <Divider type="vertical" />

    <!-- 视图 -->
    <div class="tb-group">
      <Tooltip title="网格">
        <Button
          :type="store.showGrid ? 'primary' : 'text'"
          ghost
          size="small"
          @click="store.showGrid = !store.showGrid"
        >
          <LucideGrid3x3 class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip title="吸附">
        <Button
          :type="store.snap ? 'primary' : 'text'"
          ghost
          size="small"
          @click="store.snap = !store.snap"
        >
          <LucideMagnet class="tb-icon" />
        </Button>
      </Tooltip>
      <Button size="small" type="text" @click="store.zoomBy(-0.1)">
        <LucideZoomOut class="tb-icon" />
      </Button>
      <InputNumber
        v-model:value="zoomPercent"
        :max="400"
        :min="25"
        addon-after="%"
        size="small"
        style="width: 96px"
      />
      <Button size="small" type="text" @click="store.zoomBy(0.1)">
        <LucideZoomIn class="tb-icon" />
      </Button>
      <Tooltip title="适应窗口">
        <Button size="small" type="text" @click="emit('page-preview')">
          <LucideMaximize2 class="tb-icon" />
        </Button>
      </Tooltip>
    </div>

    <Divider type="vertical" />

    <!-- 数据视图（§32） -->
    <div class="tb-group tb-data">
      <span class="tb-label">数据</span>
      <Button
        v-for="m in MODES"
        :key="m.value"
        :type="store.dataMode === m.value ? 'primary' : 'default'"
        size="small"
        @click="setMode(m.value)"
      >
        <component :is="m.icon" class="tb-icon" />{{ m.label }}
      </Button>
      <Select
        v-if="store.dataMode === 'sample'"
        :options="PRESETS"
        :value="store.samplePreset"
        size="small"
        style="width: 92px"
        @update:value="(v: any) => setPreset(v as PdfSamplePreset)"
      />
      <InputNumber
        v-if="store.dataMode === 'sample'"
        :max="500"
        :min="0"
        addon-before="行"
        size="small"
        style="width: 92px"
        :value="store.itemRows"
        @update:value="(v: any) => (store.itemRows = Number(v ?? 0))"
      />
    </div>

    <div class="tb-spacer" />

    <!-- 预览与输出 -->
    <div class="tb-group">
      <Tooltip title="上传 PDF，反推成可继续拖拽的底图模板">
        <Button size="small" @click="emit('import-pdf')">
          <LucideUpload class="tb-icon" />导入 PDF
        </Button>
      </Tooltip>
      <Tooltip title="用真实出图链路编译当前页并叠加比对（设计阶段拦截字体度量偏差）">
        <Button size="small" @click="emit('precise-preview')">
          <LucideFileInput class="tb-icon" />精确预览
        </Button>
      </Tooltip>
      <Tooltip title="按数据完整分页编译全部页（校验跨页、表头重复、合计落位）">
        <Button size="small" @click="emit('page-preview')">
          <LucideLayoutGrid class="tb-icon" />分页预览
        </Button>
      </Tooltip>
      <Tooltip title="版本历史">
        <Button size="small" type="text" @click="emit('versions')">
          <LucideHistory class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip title="导出模板包（含素材）">
        <Button size="small" type="text" @click="emit('export')">
          <LucideDownload class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip title="导入模板包">
        <Button size="small" type="text" @click="emit('import')">
          <LucideUpload class="tb-icon" />
        </Button>
      </Tooltip>
      <Tooltip title="设为默认模板（影响该单据类型的全部出图）">
        <Button size="small" @click="emit('set-default')">
          <LucideStar class="tb-icon" />设为默认
        </Button>
      </Tooltip>
    </div>
  </div>
</template>

<style scoped>
.pdf-toolbar {
  display: flex;
  gap: 2px;
  align-items: center;
  height: 44px;
  padding: 0 10px;
  background: #fff;
  border-bottom: 1px solid #f0f0f0;
}
.tb-group {
  display: flex;
  gap: 4px;
  align-items: center;
}
.tb-spacer {
  flex: 1;
}
.tb-icon {
  width: 14px;
  height: 14px;
}
.tb-name {
  max-width: 220px;
  overflow: hidden;
  font-size: 13px;
  font-weight: 500;
  color: #262626;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tb-dot {
  font-size: 10px;
  color: #faad14;
}
.tb-ver {
  padding: 0 5px;
  font-size: 11px;
  line-height: 16px;
  color: #8c8c8c;
  background: #f5f5f5;
  border-radius: 8px;
}
.tb-label {
  font-size: 11px;
  color: #8c8c8c;
}
.tb-data {
  padding: 0 4px;
  background: #fafafa;
  border-radius: 6px;
}
</style>
