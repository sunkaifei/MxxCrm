<script lang="ts" setup>
/**
 * 表格列编辑器（设计文档 §2.5 表格元素规格）。
 *
 * 列宽语义：数字 = 固定 mm；`auto` = 自适应（编译期由 Typst `auto` 承接）。
 * 合计语义：`sum` / `avg` / `count` / `max` / `min`，编译为表格 `footer` 行（§17.4）。
 */
import { computed } from 'vue';

import {
  Button,
  Input,
  InputNumber,
  Popconfirm,
  Select,
  Switch,
} from 'ant-design-vue';
import { LucideArrowDown, LucideArrowUp, LucidePlus, LucideTrash2 } from '@vben/icons';

import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';

const store = useDesignerStore();

const cols = computed(() => store.singleSelected?.props?.columns ?? []);
const el = computed(() => store.singleSelected);

const FORMAT_OPTIONS = [
  { label: '原样', value: '' },
  { label: '金额 (1,234.00)', value: 'money' },
  { label: '日期', value: 'date' },
  { label: '数量', value: 'qty' },
  { label: '百分比', value: 'rate' },
  { label: '大写金额(中)', value: 'money_cn' },
  { label: '大写金额(英)', value: 'money_en_cn' },
];

const TOTAL_OPTIONS = [
  { label: '不合计', value: '' },
  { label: '求和', value: 'sum' },
  { label: '平均', value: 'avg' },
  { label: '计数', value: 'count' },
  { label: '最大', value: 'max' },
  { label: '最小', value: 'min' },
];

/** 部分可绑定字段（明细相关） */
const ITEM_FIELDS = [
  'item.product_name',
  'item.product_code',
  'item.spec',
  'item.unit',
  'item.quantity',
  'item.price',
  'item.unit_price',
  'item.amount',
  'item.remark',
];

function mutate(fn: (list: any[]) => void) {
  if (!el.value) return;
  const next = JSON.parse(JSON.stringify(cols.value));
  fn(next);
  store.updateElement(el.value.id, { props: { ...el.value.props, columns: next } }, '编辑表格列');
}

function addCol() {
  mutate((list) => {
    list.push({
      title: '新列',
      bind: 'item.product_name',
      width: 'auto',
      align: 'left',
      format: '',
      total: null,
    });
  });
}

function removeCol(i: number) {
  mutate((list) => list.splice(i, 1));
}

function move(i: number, dir: -1 | 1) {
  mutate((list) => {
    const j = i + dir;
    if (j < 0 || j >= list.length) return;
    const [x] = list.splice(i, 1);
    list.splice(j, 0, x);
  });
}
</script>

<template>
  <div class="col-editor">
    <div class="col-head">
      <span class="col-head-title">列定义（{{ cols.length }} 列）</span>
      <Button size="small" type="link" @click="addCol">
        <LucidePlus class="ce-icon" />添加列
      </Button>
    </div>

    <div v-for="(c, i) in cols" :key="i" class="col-row">
      <div class="col-row-head">
        <Input
          :value="typeof c.title === 'string' ? c.title : (c.title?.zh ?? '')"
          placeholder="列标题"
          size="small"
          @update:value="(v: any) => mutate((l) => { l[i].title = v; })"
        />
        <Button size="small" type="text" @click="move(i, -1)">
          <LucideArrowUp class="ce-icon" />
        </Button>
        <Button size="small" type="text" @click="move(i, 1)">
          <LucideArrowDown class="ce-icon" />
        </Button>
        <Popconfirm title="删除该列？" @confirm="removeCol(i)">
          <Button danger size="small" type="text">
            <LucideTrash2 class="ce-icon" />
          </Button>
        </Popconfirm>
      </div>

      <div class="col-row-grid">
        <label>绑定字段</label>
        <Select
          :options="ITEM_FIELDS.map((f) => ({ label: f, value: f }))"
          :value="c.bind"
          show-search
          size="small"
          @update:value="(v: any) => mutate((l) => { l[i].bind = v; })"
        />

        <label>列宽(mm)</label>
        <div class="col-flex">
          <Switch
            :checked="c.width === 'auto'"
            checked-children="自适应"
            size="small"
            un-checked-children="固定"
            @update:checked="(v: any) => mutate((l) => { l[i].width = v ? 'auto' : 30; })"
          />
          <InputNumber
            v-if="c.width !== 'auto'"
            :min="5"
            :value="Number(c.width)"
            size="small"
            style="width: 76px"
            @update:value="(v: any) => mutate((l) => { l[i].width = Number(v ?? 20); })"
          />
        </div>

        <label>对齐</label>
        <Select
          :options="[
            { label: '左', value: 'left' },
            { label: '中', value: 'center' },
            { label: '右', value: 'right' },
          ]"
          :value="c.align"
          size="small"
          @update:value="(v: any) => mutate((l) => { l[i].align = v; })"
        />

        <label>格式化</label>
        <Select
          :options="FORMAT_OPTIONS"
          :value="c.format ?? ''"
          size="small"
          @update:value="(v: any) => mutate((l) => { l[i].format = v; })"
        />

        <label>合计</label>
        <Select
          :options="TOTAL_OPTIONS"
          :value="c.total ?? ''"
          size="small"
          @update:value="(v: any) => mutate((l) => { l[i].total = v || null; })"
        />
      </div>
    </div>

    <div class="col-table-opts">
      <div class="col-opt-row">
        <span>显示行号</span>
        <Switch
          :checked="el?.props?.showIndex"
          size="small"
          @update:checked="(v: any) => el && store.updateElement(el.id, { props: { ...el.props, showIndex: v } }, '表格选项')"
        />
      </div>
      <div class="col-opt-row">
        <span>表头跨页重复</span>
        <Switch
          :checked="el?.props?.headerRepeat"
          size="small"
          @update:checked="(v: any) => el && store.updateElement(el.id, { props: { ...el.props, headerRepeat: v } }, '表格选项')"
        />
      </div>
      <div class="col-opt-row">
        <span>每页固定行数</span>
        <InputNumber
          :min="0"
          :value="el?.props?.rowsPerPage ?? 0"
          size="small"
          style="width: 88px"
          @update:value="(v: any) => el && store.updateElement(el.id, { props: { ...el.props, rowsPerPage: Number(v ?? 0) } }, '每页行数')"
        />
      </div>
      <div class="col-opt-row">
        <span>行高 (mm)</span>
        <InputNumber
          :min="4"
          :step="0.5"
          :value="el?.props?.rowHeight ?? 8"
          size="small"
          style="width: 88px"
          @update:value="(v: any) => el && store.updateElement(el.id, { props: { ...el.props, rowHeight: Number(v ?? 8) } }, '行高')"
        />
      </div>
      <div class="col-opt-row">
        <span>空数据文案</span>
        <Input
          :value="el?.props?.emptyText"
          size="small"
          style="width: 120px"
          @update:value="(v: any) => el && store.updateElement(el.id, { props: { ...el.props, emptyText: v } }, '空文案')"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.col-editor {
  font-size: 12px;
}
.col-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}
.col-head-title {
  color: #595959;
}
.ce-icon {
  width: 13px;
  height: 13px;
}
.col-row {
  padding: 6px;
  margin-bottom: 6px;
  background: #fafafa;
  border: 1px solid #f0f0f0;
  border-radius: 4px;
}
.col-row-head {
  display: flex;
  gap: 2px;
  align-items: center;
  margin-bottom: 4px;
}
.col-row-grid {
  display: grid;
  grid-template-columns: 66px 1fr;
  gap: 4px 6px;
  align-items: center;
}
.col-row-grid > label {
  font-size: 11px;
  color: #8c8c8c;
}
.col-flex {
  display: flex;
  gap: 6px;
  align-items: center;
}
.col-table-opts {
  padding-top: 6px;
  border-top: 1px dashed #f0f0f0;
}
.col-opt-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}
.col-opt-row > span {
  font-size: 11px;
  color: #8c8c8c;
}
</style>
