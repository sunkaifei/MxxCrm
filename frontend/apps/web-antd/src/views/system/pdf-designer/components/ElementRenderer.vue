<script lang="ts" setup>
/**
 * 单个元素的画布渲染（设计文档 §2.4 十三类元素、§26.2 组件树）。
 *
 * 设计态渲染原则（§1.2 一致性闭环）：
 * - 只做**接近最终效果**的快速预览，不追求与 Typst 逐像素一致；
 * - 精确校验收敛到「精确预览」（后端 typst-svg 回传）；
 * - 数据视图（§32）只影响本组件如何取值，不影响 layout_json。
 */
import type { PdfElement } from '#/types/pdf-layout';

import { computed } from 'vue';

import { mmToPx } from '#/composables/pdf-designer/useUnits';

const props = defineProps<{
  dataMode: string;
  el: PdfElement;
  /** 后端回的样例/真实数据上下文 */
  ctx: Record<string, any>;
  selected: boolean;
}>();

const emit = defineEmits<{ (e: 'update', id: string): void }>();

/** 数据路径解析：支持 `order.order_no` / `item.price` */
function resolve(path?: null | string): any {
  if (!path) return undefined;
  const parts = path.replaceAll(/^\{\{|\}\}$/g, '').trim().split('.');
  // `item.xxx` 取第一条明细作为设计态代表值
  if (parts[0] === 'item') {
    const items = props.ctx?.items;
    if (Array.isArray(items) && items.length > 0) {
      return items[0]?.[parts[1] ?? ''];
    }
    return undefined;
  }
  let cur: any = props.ctx;
  for (const p of parts) {
    if (cur === null || cur === undefined) return undefined;
    cur = cur[p];
  }
  return cur;
}

/** 静态文案里的 {{path}} 插值 */
function interpolate(text: string): string {
  if (!text) return '';
  return text.replaceAll(/\{\{([^}]+)\}\}/g, (_m, p) => {
    const v = resolve(String(p).trim());
    return v === null || v === undefined || v === '' ? '—' : String(v);
  });
}

const boxStyle = computed(() => {
  const e = props.el;
  return {
    left: `${mmToPx(e.x)}px`,
    top: `${mmToPx(e.y)}px`,
    width: `${mmToPx(e.w)}px`,
    height: `${mmToPx(e.h)}px`,
    transform: e.rotate ? `rotate(${e.rotate}deg)` : undefined,
    zIndex: String((e.z ?? 0) + 1),
  };
});

const textStyle = computed(() => {
  const s = props.el.style ?? {};
  return {
    fontFamily: s.fontFamily ?? undefined,
    fontSize: `${mmToPx(((s.fontSize ?? 10) / 72) * 25.4)}px`,
    color: s.color ?? '#000',
    fontWeight: s.bold ? 600 : 400,
    fontStyle: s.italic ? 'italic' : 'normal',
    textDecoration: s.underline ? 'underline' : 'none',
    textAlign: s.align ?? 'left',
    lineHeight: String(s.lineHeight ?? 1.5),
    justifyContent:
      s.align === 'center'
        ? 'center'
        : s.align === 'right'
          ? 'flex-end'
          : 'flex-start',
    alignItems:
      s.valign === 'middle'
        ? 'center'
        : s.valign === 'bottom'
          ? 'flex-end'
          : 'flex-start',
  } as Record<string, any>;
});

/** 展示文本（四模式，§32.4） */
const shownText = computed(() => {
  const e = props.el;
  if (props.dataMode === 'placeholder' || props.dataMode === 'outline') {
    return e.bind?.path ? `{${e.bind.path}}` : '（未绑定）';
  }
  const raw =
    e.type === 'field'
      ? resolve(e.bind?.path)
      : interpolate(String(e.value ?? ''));
  if (raw === null || raw === undefined || raw === '') {
    return props.dataMode === 'sample' ? '—' : '';
  }
  return String(raw);
});

const isPlaceholderMode = computed(
  () => props.dataMode === 'placeholder' || props.dataMode === 'outline',
);

const isEmptyValue = computed(() => shownText.value === '—');

/** 表格设计态渲染：按行密度取明细 */
const tableRows = computed(() => {
  const items = props.ctx?.items;
  const arr = Array.isArray(items) ? items : [];
  const want = Math.max(1, Math.min(arr.length || 1, arr.length));
  const rows = arr.slice(0, want);
  if (rows.length === 0 && props.dataMode === 'sample') {
    return [{ _empty: true }];
  }
  return rows;
});

const tableColumns = computed(
  () => (props.el.props?.columns ?? []) as Array<Record<string, any>>,
);

function cellValue(row: any, col: Record<string, any>) {
  if (row?._empty) return col.total ? '' : '—';
  const raw = resolve(col.bind);
  // 明细列（item.xxx）从行内取
  const v =
    String(col.bind ?? '').startsWith('item.') && row
      ? row[String(col.bind).slice(5)]
      : raw;
  if (v === null || v === undefined || v === '') return '';
  if (col.format === 'money') {
    const n = Number(v);
    return Number.isFinite(n)
      ? n.toLocaleString('zh-CN', { maximumFractionDigits: 2, minimumFractionDigits: 2 })
      : String(v);
  }
  return String(v);
}

function colWidth(col: Record<string, any>): string {
  return col.width === 'auto' || !col.width ? 'auto' : `${mmToPx(col.width)}px`;
}
</script>

<template>
  <!-- 静态文本 / 数据字段 -->
  <div
    v-if="el.type === 'text' || el.type === 'field' || el.type === 'pageNumber'"
    class="pdf-el pdf-el-text"
    :class="{
      'is-placeholder': isPlaceholderMode && el.type === 'field',
      'is-empty': isEmptyValue && !isPlaceholderMode,
      'is-selected': selected,
    }"
    :style="[boxStyle, textStyle]"
    @dblclick.stop="emit('update', el.id)"
  >
    <span class="pdf-el-text-inner">{{ shownText }}</span>
  </div>

  <!-- 图片 / Logo -->
  <div v-else-if="el.type === 'image'" class="pdf-el pdf-el-image" :style="boxStyle">
    <img v-if="el.props?.url" :src="el.props.url" alt="logo" />
    <div v-else class="pdf-el-fallback">图片</div>
    <div v-if="!el.props?.url" class="pdf-el-tag">{{ el.name }}</div>
  </div>

  <!-- 电子签章 -->
  <div v-else-if="el.type === 'seal'" class="pdf-el pdf-el-seal" :style="boxStyle">
    <img v-if="el.props?.url" :src="el.props.url" alt="seal" />
    <div v-else class="pdf-el-fallback">签章</div>
  </div>

  <!-- 手写签名 -->
  <div
    v-else-if="el.type === 'signature'"
    class="pdf-el pdf-el-signature"
    :style="boxStyle"
  >
    <img v-if="el.props?.url" :src="el.props.url" alt="signature" />
    <div v-else class="pdf-el-fallback">签名区</div>
  </div>

  <!-- 条码 / 二维码 -->
  <div
    v-else-if="el.type === 'barcode' || el.type === 'qrcode'"
    class="pdf-el pdf-el-code"
    :style="boxStyle"
  >
    <div class="pdf-el-code-stripes" :class="{ 'is-qr': el.type === 'qrcode' }">
      <span v-for="n in 28" :key="n" :style="{ opacity: n % 3 === 0 ? 0.25 : 1 }" />
    </div>
    <div class="pdf-el-tag">{{ el.type === 'qrcode' ? 'QR' : 'CODE128' }}</div>
  </div>

  <!-- 直线 / 矩形 / 圆形 -->
  <div
    v-else-if="el.type === 'line'"
    class="pdf-el pdf-el-line"
    :style="{
      ...boxStyle,
      backgroundColor: el.style?.color ?? '#000',
      borderWidth: `${mmToPx(el.style?.borderWidth ?? 0.2)}px`,
    }"
  />
  <div
    v-else-if="el.type === 'rect'"
    class="pdf-el pdf-el-rect"
    :style="{
      ...boxStyle,
      borderWidth: `${mmToPx(el.style?.borderWidth ?? 0.2)}px`,
      borderColor: el.style?.borderColor ?? '#000',
      backgroundColor: el.props?.fill ?? 'transparent',
    }"
  />
  <div
    v-else-if="el.type === 'ellipse'"
    class="pdf-el pdf-el-ellipse"
    :style="{
      ...boxStyle,
      borderWidth: `${mmToPx(el.style?.borderWidth ?? 0.2)}px`,
      borderColor: el.style?.borderColor ?? '#000',
    }"
  />

  <!-- 明细表格 -->
  <div v-else-if="el.type === 'table'" class="pdf-el pdf-el-table" :style="boxStyle">
    <table>
      <thead>
        <tr>
          <th v-if="el.props?.showIndex" style="width: 28px">#</th>
          <th
            v-for="(c, i) in tableColumns"
            :key="i"
            :style="{ width: colWidth(c), textAlign: c.align ?? 'left' }"
          >
            {{ typeof c.title === 'string' ? c.title : (c.title?.zh ?? '') }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, ri) in tableRows" :key="ri">
          <td v-if="el.props?.showIndex" style="text-align: center">{{ ri + 1 }}</td>
          <td
            v-for="(c, ci) in tableColumns"
            :key="ci"
            :style="{ textAlign: c.align ?? 'left' }"
          >
            {{ cellValue(row, c) }}
          </td>
        </tr>
      </tbody>
      <tfoot v-if="tableColumns.some((c) => c.total)">
        <tr>
          <td v-if="el.props?.showIndex" />
          <td
            v-for="(c, ci) in tableColumns"
            :key="`f${ci}`"
            :style="{ textAlign: c.align ?? 'left', fontWeight: 600 }"
          >
            {{ c.total ? `合计(${cellValue({}, c) || '—'})` : '' }}
          </td>
        </tr>
      </tfoot>
    </table>
    <div class="pdf-el-table-hint">数据源：{{ el.props?.dataSource ?? 'items' }}</div>
  </div>

  <!-- 重复块 -->
  <div v-else-if="el.type === 'repeater'" class="pdf-el pdf-el-repeater" :style="boxStyle">
    <div class="pdf-el-repeater-head">重复块 · {{ el.props?.dataSource ?? 'items2' }}</div>
    <div v-for="n in 2" :key="n" class="pdf-el-repeater-row">
      第 {{ n }} 条明细…
    </div>
  </div>

  <!-- 兜底 -->
  <div v-else class="pdf-el pdf-el-fallback-box" :style="boxStyle">
    {{ el.name }}
  </div>
</template>

<style scoped>
.pdf-el {
  position: absolute;
  overflow: hidden;
  box-sizing: border-box;
}
.pdf-el-text {
  display: flex;
  padding: 0 1px;
  white-space: pre-wrap;
}
.pdf-el-text-inner {
  width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
}
.pdf-el-text.is-placeholder {
  color: #1677ff;
  background: rgb(22 119 255 / 6%);
  border: 1px dashed rgb(22 119 255 / 45%);
}
.pdf-el-text.is-empty {
  color: #bfbfbf;
  background: rgb(250 173 20 / 8%);
}
.pdf-el-image img,
.pdf-el-seal img,
.pdf-el-signature img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}
.pdf-el-fallback {
  display: flex;
  width: 100%;
  height: 100%;
  font-size: 11px;
  color: #bfbfbf;
  align-items: center;
  justify-content: center;
  border: 1px dashed #d9d9d9;
}
.pdf-el-tag {
  position: absolute;
  right: 0;
  bottom: 0;
  padding: 0 3px;
  font-size: 9px;
  line-height: 13px;
  color: #8c8c8c;
  background: rgb(255 255 255 / 85%);
}
.pdf-el-code {
  border: 1px dashed #d9d9d9;
}
.pdf-el-code-stripes {
  display: flex;
  width: 100%;
  height: 100%;
  align-items: stretch;
}
.pdf-el-code-stripes > span {
  flex: 1;
  margin-right: 1px;
  background: #262626;
}
.pdf-el-code-stripes.is-qr {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 1px;
  padding: 2px;
}
.pdf-el-code-stripes.is-qr > span:nth-child(n + 8) {
  display: none;
}
.pdf-el-line {
  border-radius: 1px;
}
.pdf-el-rect,
.pdf-el-ellipse {
  border-style: solid;
}
.pdf-el-ellipse {
  border-radius: 50%;
}
.pdf-el-table {
  border: 1px solid #262626;
}
.pdf-el-table table {
  width: 100%;
  font-size: 10px;
  border-collapse: collapse;
  table-layout: fixed;
}
.pdf-el-table th,
.pdf-el-table td {
  padding: 1px 3px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  border: 1px solid #8c8c8c;
}
.pdf-el-table th {
  font-weight: 600;
  background: #fafafa;
}
.pdf-el-table-hint {
  position: absolute;
  right: 2px;
  bottom: 2px;
  font-size: 9px;
  color: #8c8c8c;
}
.pdf-el-repeater {
  padding: 2px;
  border: 1px dashed #722ed1;
}
.pdf-el-repeater-head {
  margin-bottom: 2px;
  font-size: 10px;
  color: #722ed1;
}
.pdf-el-repeater-row {
  padding: 1px 2px;
  font-size: 10px;
  background: rgb(114 46 209 / 6%);
}
.pdf-el-fallback-box {
  display: flex;
  font-size: 11px;
  color: #bfbfbf;
  align-items: center;
  justify-content: center;
  border: 1px dashed #d9d9d9;
}
</style>
