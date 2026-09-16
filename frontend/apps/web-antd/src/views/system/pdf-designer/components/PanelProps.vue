<script lang="ts" setup>
/**
 * 右面板 · 属性（设计文档 §8.7：页面 / 元素 / 数据 / 样式 四组）。
 *
 * 设计要点：
 * - 所有几何输入单位统一 **mm**（与 layout_json 一致），不出现 pt/px。
 * - 「数据」页签承载 `bind.path` + `format` + `printIf`，即"数据绑定"职责。
 * - `printIf` 编译期若无法求值，**一律渲染**（避免误删内容），
 *   可静态判定的"条件永不成立"由绑定健康度检查器报出（§32.11）。
 */
import type { PdfAssetVO } from '#/types/pdf-layout';

import { computed, ref, watch } from 'vue';

import {
  Button,
  Input,
  InputNumber,
  Select,
  Switch,
  Tabs,
  Textarea,
} from 'ant-design-vue';

import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';
import { round } from '#/composables/pdf-designer/useUnits';

import ColorField from './ColorField.vue';
import TableColumnEditor from './TableColumnEditor.vue';

const props = defineProps<{
  assets: PdfAssetVO[];
  fieldTree: any[];
}>();

const emit = defineEmits<{
  (e: 'pick-asset', category: string): void;
}>();

const store = useDesignerStore();
const activeTab = ref('element');

const el = computed(() => store.singleSelected);
const page = computed(() => store.layout.page);

// ===== 骑缝章 / 打印偏移（P2-6/P2-7 UI 绑定）；layout 两个块均可选，缺省给默认 =====
const seamSeal = () =>
  store.layout.seamSeal ?? {
    enabled: false,
    assetId: null,
    position: 'right' as const,
    offsetX: 0,
    size: 20,
  };
function patchSeamSeal(p: Record<string, any>) {
  store.layout.seamSeal = { ...seamSeal(), ...p } as any;
  store.dirty = true;
}
const printOffset = () => store.layout.settings?.printOffset ?? { x: 0, y: 0 };
function patchPrintOffset(p: Record<string, any>) {
  const base = store.layout.settings ?? {};
  store.layout.settings = {
    ...base,
    printOffset: { ...printOffset(), ...p } as any,
  } as any;
  store.dirty = true;
}
const sealAssetId = computed({
  get: () => seamSeal().assetId ?? undefined,
  set: (v: any) => patchSeamSeal({ assetId: v == null ? null : Number(v) }),
});

const allFields = computed(() =>
  props.fieldTree.flatMap((g: any) => g.children ?? []),
);

const fieldOptions = computed(() =>
  allFields.value.map((f: any) => ({
    label: `${f.fieldName}  (${f.fieldPath})`,
    value: f.fieldPath,
  })),
);

const formatterOptions = [
  { label: '原样', value: '' },
  { label: '金额', value: 'money' },
  { label: '金额大写(中)', value: 'money_cn' },
  { label: '金额大写(中英)', value: 'money_en_cn' },
  { label: '日期', value: 'date' },
  { label: '日期时间', value: 'datetime' },
  { label: '数量', value: 'qty' },
  { label: '百分比', value: 'rate' },
  { label: '全大写', value: 'upper' },
  { label: '全小写', value: 'lower' },
];

const assetOptions = computed(() =>
  props.assets.map((a) => ({ label: a.name, value: Number(a.id) })),
);

function patch(p: Record<string, any>, label = '修改元素') {
  if (!el.value) return;
  store.updateElement(el.value.id, p, label);
}

function patchStyle(p: Record<string, any>) {
  if (!el.value) return;
  store.updateElement(el.value.id, { style: { ...el.value.style, ...p } }, '修改样式');
}

function patchProps(p: Record<string, any>, label = '修改属性') {
  if (!el.value) return;
  store.updateElement(el.value.id, { props: { ...el.value.props, ...p } }, label);
}

function bindPath() {
  return el.value?.bind?.path ?? '';
}

function setBind(path: any) {
  if (!el.value) return;
  patch({ bind: path ? { ...(el.value.bind ?? {}), path } : { path: null } }, '数据绑定');
}

function setFormat(format: any) {
  if (!el.value) return;
  patch({ bind: { ...(el.value.bind ?? {}), format: format || null } }, '格式化');
}

function setI18n(kind: 'en' | 'zh', value: string) {
  if (!el.value) return;
  patch({ i18n: { ...(el.value.i18n ?? {}), [kind]: value } }, '双语文案');
}

function setPrintIfField(field: any) {
  if (!el.value) return;
  const cur = el.value.printIf;
  patch(
    {
      printIf: field
        ? { field, op: cur?.op ?? 'eq', value: cur?.value ?? '' }
        : null,
    },
    '条件显示',
  );
}

function setPrintIfOp(op: any) {
  if (!el.value?.printIf) return;
  patch({ printIf: { ...el.value.printIf, op } }, '条件显示');
}

function setPrintIfValue(value: string) {
  if (!el.value?.printIf) return;
  patch({ printIf: { ...el.value.printIf, value } }, '条件显示');
}

// ---------------------------------------------------------------- 页面属性
function patchPage(p: Record<string, any>) {
  const next = JSON.parse(JSON.stringify(store.layout.page));
  Object.assign(next, p);
  store.commit('修改页面属性');
  store.layout.page = next;
}

// ---------------------------------------------------------------- 联次
function copies() {
  return (
    store.layout.settings?.copies ?? {
      enabled: false,
      count: 1,
      mode: 'per_page' as const,
      labels: [] as string[],
    }
  );
}

function patchCopies(p: Record<string, any>) {
  store.commit('联次设置');
  store.layout.settings = {
    ...store.layout.settings,
    copies: { ...copies(), ...p },
  };
}

function setCopyLabels(v: string) {
  patchCopies({
    labels: String(v)
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean),
  });
}

/** 纸张尺寸/方向变化时同步 width/height */watch(
  () => [page.value.size, page.value.orientation],
  () => {
    const base: Record<string, { h: number; w: number }> = {
      A4: { h: 297, w: 210 },
      A5: { h: 210, w: 148 },
      A3: { h: 420, w: 297 },
      B5: { h: 250, w: 176 },
      Letter: { h: 279, w: 216 },
      Legal: { h: 356, w: 216 },
    };
    const b = base[page.value.size];
    if (!b) return;
    const landscape = page.value.orientation === 'landscape';
    store.layout.page.width = landscape ? b.h : b.w;
    store.layout.page.height = landscape ? b.w : b.h;
  },
);

// ---------------------------------------------------------------- 页面背景
const bgAssetId = computed({
  get: () => (page.value.background.value ? Number(page.value.background.value) : undefined),
  set: (v: number | undefined) => {
    patchPage({
      background: {
        ...page.value.background,
        type: v ? 'image' : 'none',
        value: v ? String(v) : null,
      },
    });
  },
});
</script>

<template>
  <div class="panel-props">
    <Tabs v-model:activeKey="activeTab" size="small">
      <!-- ============================ 元素 ============================ -->
      <Tabs.TabPane key="element" tab="元素">
        <div v-if="!el" class="pp-empty">
          未选中元素。点击画布上的元素，或框选多个元素。
          <template v-if="store.selection.length > 1">
            <br />当前已选中 {{ store.selection.length }} 个元素。
          </template>
        </div>
        <div v-else class="pp-body">
          <div class="pp-row">
            <label>名称</label>
            <Input
              :value="el.name"
              size="small"
              @update:value="(v: any) => patch({ name: v }, '重命名元素')"
            />
          </div>
          <div class="pp-row">
            <label>类型</label>
            <span class="pp-static">{{ el.type }}</span>
          </div>
          <div class="pp-grid2">
            <div class="pp-row">
              <label>X (mm)</label>
              <InputNumber
                :step="0.5"
                :value="el.x"
                size="small"
                @update:value="(v: any) => patch({ x: round(Number(v ?? 0)) })"
              />
            </div>
            <div class="pp-row">
              <label>Y (mm)</label>
              <InputNumber
                :step="0.5"
                :value="el.y"
                size="small"
                @update:value="(v: any) => patch({ y: round(Number(v ?? 0)) })"
              />
            </div>
            <div class="pp-row">
              <label>宽 (mm)</label>
              <InputNumber
                :min="1"
                :step="0.5"
                :value="el.w"
                size="small"
                @update:value="(v: any) => patch({ w: round(Number(v ?? 1)) })"
              />
            </div>
            <div class="pp-row">
              <label>高 (mm)</label>
              <InputNumber
                :min="0.2"
                :step="0.5"
                :value="el.h"
                size="small"
                @update:value="(v: any) => patch({ h: round(Number(v ?? 1)) })"
              />
            </div>
          </div>
          <div class="pp-row">
            <label>旋转 (°)</label>
            <InputNumber
              :max="360"
              :min="-360"
              :value="el.rotate ?? 0"
              size="small"
              @update:value="(v: any) => patch({ rotate: Number(v ?? 0) })"
            />
          </div>
          <div class="pp-row">
            <label>锁定</label>
            <Switch
              :checked="el.locked"
              size="small"
              @update:checked="(v: any) => patch({ locked: v }, '锁定状态')"
            />
          </div>
          <div class="pp-row">
            <label>仅设计态可见</label>
            <Switch
              :checked="el.hiddenInOutput"
              size="small"
              @update:checked="(v: any) => patch({ hiddenInOutput: v }, '可见性')"
            />
          </div>

          <!-- 文本类 -->
          <template v-if="el.type === 'text' || el.type === 'pageNumber'">
            <div class="pp-row pp-col">
              <label>文本内容</label>
              <Textarea
                :rows="2"
                :value="el.value ?? ''"
                placeholder="支持 {{order.order_no}} 插值；页码可用 {{page}} / {{pages}}"
                size="small"
                @update:value="(v: any) => patch({ value: v })"
              />
            </div>
          </template>

          <!-- 图片 / 签章 / 签名 -->
          <template
            v-if="['image', 'seal', 'signature'].includes(el.type)"
          >
            <div class="pp-row pp-col">
              <label>选择素材</label>
              <Select
                :options="assetOptions"
                :value="el.props?.assetId"
                allow-clear
                placeholder="从素材库选择"
                show-search
                size="small"
                @update:value="(v: any) => patchProps({ assetId: v }, '选择素材')"
              />
            </div>
            <div class="pp-row">
              <label>保持比例</label>
              <Switch
                :checked="el.props?.keepRatio !== false"
                size="small"
                @update:checked="(v: any) => patchProps({ keepRatio: v })"
              />
            </div>
            <Button size="small" type="link" @click="emit('pick-asset', el.type === 'seal' ? 'seal' : 'logo')">
              去素材库上传
            </Button>
          </template>

          <!-- 条码 / 二维码 -->
          <template v-if="['barcode', 'qrcode'].includes(el.type)">
            <div class="pp-row pp-col">
              <label>编码内容</label>
              <Input
                :value="el.props?.value ?? ''"
                placeholder="{{order.order_no}} 或固定值"
                size="small"
                @update:value="(v: any) => patchProps({ value: v }, '条码内容')"
              />
            </div>
            <div v-if="el.type === 'barcode'" class="pp-row">
              <label>码制</label>
              <Select
                :options="[
                  { label: 'Code128', value: 'code128' },
                  { label: 'Code39', value: 'code39' },
                  { label: 'EAN13', value: 'ean13' },
                ]"
                :value="el.props?.format ?? 'code128'"
                size="small"
                @update:value="(v: any) => patchProps({ format: v }, '码制')"
              />
            </div>
            <div class="pp-row">
              <label>显示可读文本</label>
              <Switch
                :checked="el.props?.showText"
                size="small"
                @update:checked="(v: any) => patchProps({ showText: v })"
              />
            </div>
          </template>

          <!-- 表格 -->
          <template v-if="el.type === 'table'">
            <TableColumnEditor />
          </template>

          <!-- 重复块 -->
          <template v-if="el.type === 'repeater'">
            <div class="pp-row pp-col">
              <label>数据源</label>
              <Input
                :value="el.props?.dataSource ?? 'items2'"
                size="small"
                @update:value="(v: any) => patchProps({ dataSource: v }, '重复块数据源')"
              />
            </div>
          </template>

          <!-- 条件显示 -->
          <div class="pp-section">
            <div class="pp-section-title">
              条件显示（printIf）
              <span class="pp-section-hint">留空则始终输出</span>
            </div>
            <div class="pp-row">
              <label>字段</label>
              <Select
                :options="fieldOptions"
                :value="el.printIf?.field"
                allow-clear
                placeholder="选择字段"
                show-search
                size="small"
                @update:value="setPrintIfField"
              />
            </div>
            <template v-if="el.printIf">
              <div class="pp-row">
                <label>运算符</label>
                <Select
                  :options="[
                    { label: '等于', value: 'eq' },
                    { label: '不等于', value: 'ne' },
                    { label: '大于', value: 'gt' },
                    { label: '大于等于', value: 'gte' },
                    { label: '小于', value: 'lt' },
                    { label: '小于等于', value: 'lte' },
                    { label: '包含', value: 'contains' },
                    { label: '非空', value: 'notEmpty' },
                  ]"
                  :value="el.printIf.op"
                  size="small"
                  @update:value="setPrintIfOp"
                />
              </div>
              <div v-if="el.printIf.op !== 'notEmpty'" class="pp-row">
                <label>比较值</label>
                <Input
                  :value="String(el.printIf.value ?? '')"
                  size="small"
                  @update:value="setPrintIfValue"
                />
              </div>
            </template>
          </div>
        </div>
      </Tabs.TabPane>

      <!-- ============================ 数据 ============================ -->
      <Tabs.TabPane key="data" tab="数据">
        <div v-if="!el" class="pp-empty">选中一个元素后可配置数据绑定</div>
        <div v-else class="pp-body">
          <div class="pp-row">
            <label>绑定字段</label>
            <Select
              :options="fieldOptions"
              :value="bindPath()"
              allow-clear
              placeholder="从字段树选择"
              show-search
              size="small"
              @update:value="setBind"
            />
          </div>
          <div class="pp-row">
            <label>格式化</label>
            <Select
              :options="formatterOptions"
              :value="el.bind?.format ?? ''"
              size="small"
              @update:value="setFormat"
            />
          </div>
          <div class="pp-row">
            <label>空值占位</label>
            <Input
              :value="el.props?.emptyText"
              placeholder="留空则显示 —"
              size="small"
              @update:value="(v: any) => patchProps({ emptyText: v }, '空值占位')"
            />
          </div>
          <div class="pp-row pp-col">
            <label>双语文案（外贸单据，§21.2）</label>
            <Input
              :value="el.i18n?.zh ?? ''"
              placeholder="中文文案"
              size="small"
              @update:value="(v: any) => setI18n('zh', v)"
            />
            <Input
              :value="el.i18n?.en ?? ''"
              placeholder="English text"
              size="small"
              style="margin-top: 4px"
              @update:value="(v: any) => setI18n('en', v)"
            />
          </div>
        </div>
      </Tabs.TabPane>

      <!-- ============================ 样式 ============================ -->
      <Tabs.TabPane key="style" tab="样式">
        <div v-if="!el" class="pp-empty">选中元素后可调整样式</div>
        <div v-else class="pp-body">
          <div class="pp-grid2">
            <div class="pp-row">
              <label>字号 (pt)</label>
              <InputNumber
                :min="4"
                :step="0.5"
                :value="el.style?.fontSize ?? 10"
                size="small"
                @update:value="(v: any) => patchStyle({ fontSize: Number(v ?? 10) })"
              />
            </div>
            <div class="pp-row">
              <label>行高</label>
              <InputNumber
                :min="1"
                :step="0.1"
                :value="el.style?.lineHeight ?? 1.5"
                size="small"
                @update:value="(v: any) => patchStyle({ lineHeight: Number(v ?? 1.5) })"
              />
            </div>
          </div>
          <div class="pp-row">
            <label>颜色</label>
            <ColorField
              :value="el.style?.color ?? '#000000'"
              @update:value="(v: string) => patchStyle({ color: v })"
            />
          </div>
          <div class="pp-row">
            <label>字体</label>
            <Select
              :options="[
                { label: '思源黑体', value: 'Source Han Sans SC' },
                { label: '思源宋体', value: 'Source Han Serif SC' },
              ]"
              :value="el.style?.fontFamily ?? 'Source Han Sans SC'"
              size="small"
              @update:value="(v: any) => patchStyle({ fontFamily: v })"
            />
          </div>
          <div class="pp-row">
            <label>加粗/斜体/下划线</label>
            <div class="pp-inline">
              <Switch :checked="el.style?.bold" size="small" @update:checked="(v: any) => patchStyle({ bold: v })" />
              <Switch :checked="el.style?.italic" size="small" @update:checked="(v: any) => patchStyle({ italic: v })" />
              <Switch :checked="el.style?.underline" size="small" @update:checked="(v: any) => patchStyle({ underline: v })" />
            </div>
          </div>
          <div class="pp-grid2">
            <div class="pp-row">
              <label>水平对齐</label>
              <Select
                :options="[
                  { label: '左', value: 'left' },
                  { label: '中', value: 'center' },
                  { label: '右', value: 'right' },
                  { label: '两端', value: 'justify' },
                ]"
                :value="el.style?.align ?? 'left'"
                size="small"
                @update:value="(v: any) => patchStyle({ align: v })"
              />
            </div>
            <div class="pp-row">
              <label>垂直对齐</label>
              <Select
                :options="[
                  { label: '上', value: 'top' },
                  { label: '中', value: 'middle' },
                  { label: '下', value: 'bottom' },
                ]"
                :value="el.style?.valign ?? 'top'"
                size="small"
                @update:value="(v: any) => patchStyle({ valign: v })"
              />
            </div>
          </div>
          <div class="pp-grid2">
            <div class="pp-row">
              <label>边框 (mm)</label>
              <InputNumber
                :min="0"
                :step="0.1"
                :value="el.style?.borderWidth ?? 0"
                size="small"
                @update:value="(v: any) => patchStyle({ borderWidth: Number(v ?? 0) })"
              />
            </div>
            <div class="pp-row">
              <label>内边距 (mm)</label>
              <InputNumber
                :min="0"
                :step="0.2"
                :value="el.style?.padding ?? 0"
                size="small"
                @update:value="(v: any) => patchStyle({ padding: Number(v ?? 0) })"
              />
            </div>
          </div>
          <div class="pp-row">
            <label>边框颜色</label>
            <ColorField
              :value="el.style?.borderColor ?? '#000000'"
              @update:value="(v: string) => patchStyle({ borderColor: v })"
            />
          </div>
        </div>
      </Tabs.TabPane>

      <!-- ============================ 页面 ============================ -->
      <Tabs.TabPane key="page" tab="页面">
        <div class="pp-body">
          <div class="pp-grid2">
            <div class="pp-row">
              <label>纸张</label>
              <Select
                :options="[
                  { label: 'A4 (210×297)', value: 'A4' },
                  { label: 'A5 (148×210)', value: 'A5' },
                  { label: 'A3 (297×420)', value: 'A3' },
                  { label: 'B5 (176×250)', value: 'B5' },
                  { label: 'Letter (216×279)', value: 'Letter' },
                  { label: 'Legal (216×356)', value: 'Legal' },
                  { label: '自定义', value: 'custom' },
                ]"
                :value="page.size"
                size="small"
                @update:value="(v: any) => patchPage({ size: v, ...(v === 'custom' ? { width: page.width > 0 ? page.width : 210, height: page.height > 0 ? page.height : 297 } : {}) })"
              />
            </div>
            <div v-if="page.size === 'custom'" class="pp-grid2">
              <div class="pp-row">
                <label>宽 (mm)</label>
                <InputNumber
                  :min="50"
                  :max="2000"
                  :value="page.width || 210"
                  size="small"
                  @update:value="(v: any) => patchPage({ width: Number(v ?? 210) })"
                />
              </div>
              <div class="pp-row">
                <label>高 (mm)</label>
                <InputNumber
                  :min="50"
                  :max="2000"
                  :value="page.height || 297"
                  size="small"
                  @update:value="(v: any) => patchPage({ height: Number(v ?? 297) })"
                />
              </div>
            </div>
            <div class="pp-row">
              <label>方向</label>
              <Select
                :options="[
                  { label: '纵向', value: 'portrait' },
                  { label: '横向', value: 'landscape' },
                ]"
                :value="page.orientation"
                size="small"
                @update:value="(v: any) => patchPage({ orientation: v })"
              />
            </div>
          </div>
          <div class="pp-section-title">页边距 (mm)</div>
          <div class="pp-grid2">
            <div class="pp-row">
              <label>上</label>
              <InputNumber :value="page.margin.top" size="small" @update:value="(v: any) => patchPage({ margin: { ...page.margin, top: Number(v ?? 0) } })" />
            </div>
            <div class="pp-row">
              <label>下</label>
              <InputNumber :value="page.margin.bottom" size="small" @update:value="(v: any) => patchPage({ margin: { ...page.margin, bottom: Number(v ?? 0) } })" />
            </div>
            <div class="pp-row">
              <label>左</label>
              <InputNumber :value="page.margin.left" size="small" @update:value="(v: any) => patchPage({ margin: { ...page.margin, left: Number(v ?? 0) } })" />
            </div>
            <div class="pp-row">
              <label>右</label>
              <InputNumber :value="page.margin.right" size="small" @update:value="(v: any) => patchPage({ margin: { ...page.margin, right: Number(v ?? 0) } })" />
            </div>
          </div>

          <div class="pp-section-title">页面底图（预印纸 / 套打）</div>
          <div class="pp-row pp-col">
            <Select
              v-model:value="bgAssetId"
              :options="assetOptions"
              allow-clear
              placeholder="不设底图"
              size="small"
            />
            <div class="pp-hint">
              底图**不会**输出到 PDF，仅用于定位对齐。开启"仅预览"或"预印纸"后
              正式出图会完全忽略，避免与预印内容重叠打印（§19.3）。
            </div>
          </div>
          <div class="pp-row">
            <label>仅预览</label>
            <Switch
              :checked="page.background.previewOnly"
              size="small"
              @update:checked="(v: any) => patchPage({ background: { ...page.background, previewOnly: v } })"
            />
          </div>

          <div class="pp-section-title">水印</div>
          <div class="pp-row pp-col">
            <Input
              :value="page.watermark.text ?? ''"
              placeholder="水印文字（如：样张 / DRAFT）"
              size="small"
              @update:value="(v: any) => patchPage({ watermark: { ...page.watermark, text: v } })"
            />
          </div>
          <div class="pp-grid2">
            <div class="pp-row">
              <label>透明度</label>
              <InputNumber :max="1" :min="0" :step="0.02" :value="page.watermark.opacity" size="small" @update:value="(v: any) => patchPage({ watermark: { ...page.watermark, opacity: Number(v ?? 0.08) } })" />
            </div>
            <div class="pp-row">
              <label>角度</label>
              <InputNumber :value="page.watermark.rotate" size="small" @update:value="(v: any) => patchPage({ watermark: { ...page.watermark, rotate: Number(v ?? 45) } })" />
            </div>
          </div>

          <div class="pp-section-title">联次（多联复写纸，§19.2）</div>
          <div class="pp-row">
            <label>启用联次</label>
            <Switch
              :checked="copies().enabled"
              size="small"
              @update:checked="(v: any) => patchCopies({ enabled: v })"
            />
          </div>
          <div class="pp-row">
            <label>联数</label>
            <InputNumber
              :max="6"
              :min="1"
              :value="copies().count ?? 1"
              size="small"
              @update:value="(v: any) => patchCopies({ count: Number(v ?? 1) })"
            />
          </div>
          <div class="pp-row pp-col">
            <label>联次名称（逗号分隔）</label>
            <Input
              :value="(copies().labels ?? []).join(',')"
              placeholder="存根联,客户联,财务联"
              size="small"
              @update:value="setCopyLabels"
            />
          </div>

          <div class="pp-section-title">骑缝章（多页合同，§20/P2-7）</div>
          <div class="pp-row">
            <label>启用骑缝章</label>
            <Switch
              :checked="seamSeal().enabled"
              size="small"
              @update:checked="(v: any) => patchSeamSeal({ enabled: v })"
            />
          </div>
          <template v-if="seamSeal().enabled">
            <div class="pp-row pp-col">
              <Select
                v-model:value="sealAssetId"
                :options="assetOptions"
                allow-clear
                placeholder="选择章图片素材"
                size="small"
              />
            </div>
            <div class="pp-grid2">
              <div class="pp-row">
                <label>位置</label>
                <Select
                  :options="[
                    { label: '右侧', value: 'right' },
                    { label: '左侧', value: 'left' },
                  ]"
                  :value="seamSeal().position"
                  size="small"
                  @update:value="(v: any) => patchSeamSeal({ position: v })"
                />
              </div>
              <div class="pp-row">
                <label>尺寸 (mm)</label>
                <InputNumber
                  :min="10"
                  :max="80"
                  :value="seamSeal().size"
                  size="small"
                  @update:value="(v: any) => patchSeamSeal({ size: Number(v ?? 20) })"
                />
              </div>
            </div>
            <div class="pp-row">
              <label>上下偏移</label>
              <InputNumber
                :min="-100"
                :max="100"
                :value="(seamSeal() as any).offset ?? 0"
                size="small"
                @update:value="(v: any) => patchSeamSeal({ offset: Number(v ?? 0) })"
              />
            </div>
          </template>

          <div class="pp-section-title">打印偏移校准 (mm, §19.4)</div>
          <div class="pp-grid2">
            <div class="pp-row">
              <label>水平 X</label>
              <InputNumber
                :min="-50"
                :max="50"
                :step="0.5"
                :value="printOffset().x"
                size="small"
                @update:value="(v: any) => patchPrintOffset({ x: Number(v ?? 0) })"
              />
            </div>
            <div class="pp-row">
              <label>垂直 Y</label>
              <InputNumber
                :min="-50"
                :max="50"
                :step="0.5"
                :value="printOffset().y"
                size="small"
                @update:value="(v: any) => patchPrintOffset({ y: Number(v ?? 0) })"
              />
            </div>
          </div>
          <div class="pp-hint">
            套打对位微调：X&gt;0 整体右移，Y&gt;0 整体下移（编译时叠加到页边距）。
          </div>
        </div>
      </Tabs.TabPane>
    </Tabs>
  </div>
</template>

<style scoped>
.panel-props {
  height: 100%;
  padding: 0 8px 12px;
  overflow-y: auto;
}
.pp-body {
  padding-top: 4px;
}
.pp-empty {
  padding: 28px 12px;
  font-size: 12px;
  line-height: 1.8;
  color: #bfbfbf;
  text-align: center;
}
.pp-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 6px;
}
.pp-col {
  display: block;
}
.pp-row > label {
  flex: none;
  width: 76px;
  font-size: 12px;
  color: #8c8c8c;
}
.pp-col > label {
  display: block;
  margin-bottom: 4px;
}
.pp-row :deep(.ant-input-number),
.pp-row :deep(.ant-select) {
  flex: 1;
  width: 100%;
}
.pp-static {
  font-size: 12px;
  color: #595959;
}
.pp-grid2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0 8px;
}
.pp-grid2 .pp-row > label {
  width: 58px;
}
.pp-inline {
  display: flex;
  gap: 8px;
}
.pp-section {
  padding-top: 8px;
  margin-top: 8px;
  border-top: 1px dashed #f0f0f0;
}
.pp-section-title {
  margin: 10px 0 6px;
  font-size: 12px;
  font-weight: 500;
  color: #595959;
}
.pp-section-hint {
  margin-left: 6px;
  font-size: 11px;
  font-weight: 400;
  color: #bfbfbf;
}
.pp-hint {
  margin-top: 4px;
  font-size: 11px;
  line-height: 1.6;
  color: #bfbfbf;
}
</style>
