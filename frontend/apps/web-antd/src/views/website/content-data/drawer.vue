<script lang="ts" setup>
import type { ContentModelFieldVO } from '#/api/core/website/content-model';

import { computed, reactive, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import {
  DatePicker,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Select,
  TabPane,
  Tabs,
  Textarea,
} from 'ant-design-vue';

import { contentDataApi } from '#/api/core/website/content-data';
import { getUserListApi } from '#/api/core/system/user';

const AInput = Input;
const ATextarea = Textarea;
const AInputNumber = InputNumber;
const ASelect = Select;
const ADatePicker = DatePicker;
const AForm = Form;
const AFormItem = FormItem;
const ATabs = Tabs;
const ATabPane = TabPane;

const data = ref<any>();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value ? '新增内容' : '编辑内容',
);

const fields = computed<ContentModelFieldVO[]>(() => data.value?.fields || []);
const model = computed<any>(() => data.value?.model || {});
const code = computed<string>(() => data.value?.code || '');

const formState = reactive<Record<string, any>>({});

function parseOptions(raw?: string): { label: string; value: any }[] {
  if (!raw) return [];
  try {
    const arr = JSON.parse(raw);
    if (Array.isArray(arr)) {
      return arr.map((o: any) =>
        typeof o === 'object'
          ? { label: o.label ?? o.value, value: o.value }
          : { label: String(o), value: o },
      );
    }
  } catch {
    /* ignore */
  }
  return [];
}

/** 自定义字段类型（int 1-11）→ 控件 */
function customWidget(fieldType: number): string {
  switch (fieldType) {
    case 1:
      return 'input';
    case 2:
    case 3:
      return 'textarea';
    case 4:
      return 'number';
    case 5:
      return 'date';
    case 6:
      return 'select';
    case 7:
      return 'select';
    case 8:
      return 'multiselect';
    case 11:
      return 'user';
    case 9:
    case 10:
      return 'input';
    default:
      return 'input';
  }
}

// ===== 表单布局（模型上配置的 formLayout：列数/选项卡/字段摆放） =====
interface LayoutTab {
  fields: string[];
  key: string;
  name: string;
}
interface FormGroup {
  items: RenderField[];
  key: string;
  name: string;
}
const layout = ref<{ columns: number; tabs: LayoutTab[] } | null>(null);
const activeLayoutTab = ref('');

function parseLayout(m: any) {
  try {
    if (m?.formLayout) {
      const parsed = JSON.parse(m.formLayout);
      layout.value = {
        columns: [1, 2, 3].includes(Number(parsed.columns))
          ? Number(parsed.columns)
          : 2,
        tabs: Array.isArray(parsed.tabs)
          ? parsed.tabs.map((t: any, i: number) => ({
              key: t.key || `tab${i + 1}`,
              name: t.name || `选项卡${i + 1}`,
              fields: Array.isArray(t.fields) ? t.fields : [],
            }))
          : [],
      };
      return;
    }
  } catch {
    /* ignore */
  }
  layout.value = null;
}

// ===== 用户选项（字段类型 11=用户：下拉选择系统用户，存 user_id） =====
const userOptions = ref<{ label: string; value: number }[]>([]);

async function loadUserOptions() {
  if (userOptions.value.length > 0) return;
  try {
    const res: any = await getUserListApi({ page: 1, pageSize: 999 });
    const list = res?.items || res?.rows || [];
    userOptions.value = list.map((u: any) => ({
      label: u.realName || u.nickname || u.username || String(u.id),
      value: Number(u.id ?? u.userId),
    }));
  } catch {
    userOptions.value = [];
  }
}

interface RenderField {
  key: string;
  label: string;
  widget: string;
  options?: { label: string; value: any }[];
  required?: boolean;
  placeholder?: string;
}

/** 按模型布局（formLayout）分组成选项卡；未配置则单组平铺 */
const formGroups = computed<FormGroup[]>(() => {
  const all = renderFields.value;
  const L = layout.value;
  if (!L || L.tabs.length === 0) {
    return [{ key: 'default', name: '默认', items: all }];
  }
  const byKey = new Map(all.map((f) => [f.key, f]));
  const groups: FormGroup[] = L.tabs.map((t) => ({
    key: t.key,
    name: t.name,
    items: (t.fields || [])
      .map((k) => byKey.get(k))
      .filter((f): f is RenderField => !!f),
  }));
  // 未分配字段追加到最后一个选项卡，保证不丢
  const used = new Set(L.tabs.flatMap((t) => t.fields || []));
  const rest = all.filter((f) => !used.has(f.key));
  if (rest.length > 0 && groups.length > 0) {
    groups[groups.length - 1]!.items.push(...rest);
  }
  return groups;
});

const gridStyle = computed(() => ({
  display: 'grid',
  gap: '0 16px',
  gridTemplateColumns: `repeat(${layout.value?.columns ?? 2}, minmax(0, 1fr))`,
}));

const renderFields = computed<RenderField[]>(() => {
  const list: RenderField[] = [];
  const m = model.value;

  list.push({ key: 'title', label: '标题', widget: 'input', required: true });
  if (m.hasSummary) list.push({ key: 'summary', label: '摘要', widget: 'textarea' });
  if (m.hasContent) list.push({ key: 'content', label: '正文', widget: 'textarea' });
  if (m.hasCover)
    list.push({ key: 'coverImage', label: '封面图(URL)', widget: 'input' });
  if (m.hasAuthor) list.push({ key: 'author', label: '作者', widget: 'input' });
  if (m.hasSeo) {
    list.push({ key: 'seoTitle', label: 'SEO 标题', widget: 'input' });
    list.push({ key: 'seoKeywords', label: 'SEO 关键词', widget: 'input' });
    list.push({ key: 'seoDescription', label: 'SEO 描述', widget: 'textarea' });
  }
  list.push({ key: 'sort', label: '排序', widget: 'number' });
  list.push({
    key: 'status',
    label: '状态',
    widget: 'select',
    options: [
      { label: '启用', value: 1 },
      { label: '禁用', value: 0 },
    ],
  });

  for (const f of fields.value) {
    const widget = customWidget(f.fieldType);
    let options: any[] | undefined;
    if (widget === 'select' || widget === 'multiselect') {
      options = parseOptions(f.fieldOptions);
    }
    if (widget === 'user') {
      options = userOptions.value;
    }
    list.push({
      key: f.fieldName,
      label: f.fieldLabel || f.fieldName,
      widget,
      options,
      required: f.isRequired === 1,
      placeholder: f.placeholder || undefined,
    });
  }
  return list;
});

function resetForm() {
  Object.keys(formState).forEach((k) => delete formState[k]);
}

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    const errs: string[] = [];
    for (const f of renderFields.value) {
      const v = formState[f.key];
      const empty =
        v === undefined || v === null || v === '' || (Array.isArray(v) && v.length === 0);
      if (f.required && empty) errs.push(f.label);
    }
    if (errs.length > 0) {
      message.warning(`请填写：${errs.join('、')}`);
      return;
    }

    setLoading(true);
    const payload: Record<string, any> = {};
    for (const f of renderFields.value) {
      let v = formState[f.key];
      if (Array.isArray(v)) v = v.join(','); // 多选 → 逗号拼接
      if (v !== undefined) payload[f.key] = v;
    }

    try {
      if (isCreate.value) {
        await contentDataApi.add(code.value, payload);
        message.success('新增成功');
      } else {
        await contentDataApi.update(code.value, data.value.row.id, payload);
        message.success('修改成功');
      }
      drawerApi.setData({ needRefresh: true });
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },

  async onOpenChange(isOpen) {
    if (!isOpen) return;
    data.value = drawerApi.getData<Record<string, any>>();
    resetForm();
    parseLayout(data.value?.model);
    activeLayoutTab.value = formGroups.value[0]?.key || '';
    // 用户类型字段需要用户选项
    if (fields.value.some((f) => Number(f.fieldType) === 11)) {
      await loadUserOptions();
    }

    if (!isCreate.value && data.value?.row?.id) {
      try {
        const detail: any = await contentDataApi.detail(code.value, data.value.row.id);
        const row = detail?.data || detail || data.value.row;
        // 仅回填渲染字段
        for (const f of renderFields.value) {
          let v = row[f.key];
          if (f.widget === 'multiselect' && typeof v === 'string') {
            v = v ? v.split(',').filter(Boolean) : [];
          }
          formState[f.key] = v ?? undefined;
        }
      } catch {
        // 回退用行数据
        for (const f of renderFields.value) {
          formState[f.key] = data.value.row[f.key] ?? undefined;
        }
      }
    } else {
      // 默认值（$now 哨兵 = 录入时动态取当前时间）
      formState.status = 1;
      const now = new Date();
      const pad = (n: number) => String(n).padStart(2, '0');
      const nowStr = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} ${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`;
      for (const f of fields.value) {
        if (f.defaultValue === '$now' && customWidget(f.fieldType) === 'date') {
          formState[f.fieldName] = nowStr;
          continue;
        }
        if (f.defaultValue !== undefined && f.defaultValue !== null && f.defaultValue !== '') {
          formState[f.fieldName] = f.defaultValue;
        }
      }
    }
    setLoading(false);
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle">
    <AForm layout="vertical">
      <ATabs
        v-model:activeKey="activeLayoutTab"
        :class="{ 'no-tab-bar': formGroups.length === 1 }"
      >
        <ATabPane
          v-for="g in formGroups"
          :key="g.key"
          :tab="g.name"
        >
          <div class="form-grid" :style="gridStyle">
            <AFormItem
              v-for="f in g.items"
              :key="f.key"
              :label="f.label"
              :required="f.required"
            >
              <AInput
                v-if="f.widget === 'input'"
                v-model:value="formState[f.key]"
                :placeholder="f.placeholder"
                allow-clear
              />
              <ATextarea
                v-else-if="f.widget === 'textarea'"
                v-model:value="formState[f.key]"
                :rows="4"
                :placeholder="f.placeholder"
                allow-clear
              />
              <AInputNumber
                v-else-if="f.widget === 'number'"
                v-model:value="formState[f.key]"
                class="w-full"
              />
              <ADatePicker
                v-else-if="f.widget === 'date'"
                v-model:value="formState[f.key]"
                class="w-full"
                show-time
                value-format="YYYY-MM-DD HH:mm:ss"
              />
              <ASelect
                v-else-if="f.widget === 'select'"
                v-model:value="formState[f.key]"
                :options="f.options"
                :placeholder="f.placeholder"
                allow-clear
              />
              <ASelect
                v-else-if="f.widget === 'user'"
                v-model:value="formState[f.key]"
                :options="f.options"
                show-search
                option-filter-prop="label"
                :placeholder="f.placeholder || '请选择用户'"
                allow-clear
              />
              <ASelect
                v-else-if="f.widget === 'multiselect'"
                v-model:value="formState[f.key]"
                mode="multiple"
                :options="f.options"
                :placeholder="f.placeholder"
                allow-clear
              />
            </AFormItem>
          </div>
        </ATabPane>
      </ATabs>
    </AForm>
  </Drawer>
</template>

<style>
.ant-picker-dropdown,
.ant-select-dropdown {
  z-index: 3000 !important;
}

/* 平铺布局（未配置选项卡）时隐藏标签栏，仅保留网格表单 */
.no-tab-bar > .ant-tabs-nav {
  display: none;
}

.form-grid {
  row-gap: 4px;
}
</style>
