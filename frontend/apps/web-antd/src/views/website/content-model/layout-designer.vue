<script lang="ts" setup>
// 内容模型「表单布局设计器」：配置内容录入/编辑抽屉的列数、选项卡与字段摆放。
// 产出 formLayout JSON 存到模型上（mxx_content_model.form_layout），通用内容页按它渲染。
import { computed, ref, watch } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Button, Input, message, Select, Tabs, TabPane } from 'ant-design-vue';
import draggable from 'vuedraggable';

import { updateContentModelApi } from '#/api';

const props = defineProps<{
  fields: { fieldName: string; fieldLabel?: string }[];
  model: any;
}>();
const emit = defineEmits<{ saved: [] }>();
const visible = defineModel<boolean>('open', { default: false });

interface LayoutTab {
  fields: string[];
  key: string;
  name: string;
}

const columns = ref<number>(2);
const tabs = ref<LayoutTab[]>([]);
const activeTab = ref('');
const saving = ref(false);

/** 固定字段（按模型能力开关过滤）+ 自定义字段 */
const FIXED_FIELDS = computed(() => {
  const m = props.model || {};
  const list: { key: string; label: string }[] = [
    { key: 'title', label: '标题' },
  ];
  if (Number(m.hasSummary) === 1) list.push({ key: 'summary', label: '摘要' });
  if (Number(m.hasContent) === 1) list.push({ key: 'content', label: '正文' });
  if (Number(m.hasCover) === 1) list.push({ key: 'coverImage', label: '封面图' });
  if (Number(m.hasAuthor) === 1) list.push({ key: 'author', label: '作者' });
  if (Number(m.hasSeo) === 1) {
    list.push({ key: 'seoTitle', label: 'SEO 标题' });
    list.push({ key: 'seoKeywords', label: 'SEO 关键词' });
    list.push({ key: 'seoDescription', label: 'SEO 描述' });
  }
  list.push({ key: 'sort', label: '排序' });
  list.push({ key: 'status', label: '状态' });
  return list;
});

const customFields = computed(() =>
  (props.fields || []).map((f) => ({
    key: f.fieldName || '',
    label: f.fieldLabel || f.fieldName || '',
  })),
);

const allFieldLabels = computed(() => {
  const map: Record<string, string> = {};
  for (const f of [...FIXED_FIELDS.value, ...customFields.value]) {
    map[f.key] = f.label;
  }
  return map;
});

/** 已分配到选项卡的字段 key 集合 */
const assignedKeys = computed(() => {
  const set = new Set<string>();
  for (const t of tabs.value) for (const k of t.fields) set.add(k);
  return set;
});

/** 未分配字段：固定字段 + 自定义字段 - 已分配 */
const unassigned = computed(() =>
  [...FIXED_FIELDS.value, ...customFields.value].filter(
    (f) => !assignedKeys.value.has(f.key),
  ),
);

function initFromModel() {
  const m = props.model || {};
  let parsed: any = null;
  try {
    parsed = m.formLayout ? JSON.parse(m.formLayout) : null;
  } catch {
    parsed = null;
  }
  columns.value = [1, 2, 3].includes(Number(parsed?.columns))
    ? Number(parsed.columns)
    : 2;
  tabs.value = Array.isArray(parsed?.tabs) && parsed.tabs.length > 0
    ? parsed.tabs.map((t: any, i: number) => ({
        key: t.key || `tab${i + 1}`,
        name: t.name || `选项卡${i + 1}`,
        fields: Array.isArray(t.fields) ? [...t.fields] : [],
      }))
    : [{ key: 'tab1', name: '基础信息', fields: [] }];
  activeTab.value = tabs.value[0]?.key || '';
}

watch(
  () => [visible.value, props.model?.id],
  ([v]) => {
    if (v) initFromModel();
  },
  { immediate: true },
);

function addTab() {
  const key = `tab${Date.now()}`;
  tabs.value.push({ key, name: `选项卡${tabs.value.length + 1}`, fields: [] });
  activeTab.value = key;
}

function removeTab(key: string) {
  const idx = tabs.value.findIndex((t) => t.key === key);
  if (idx === -1) return;
  tabs.value[idx]!.fields = [];
  tabs.value.splice(idx, 1);
  if (tabs.value.length === 0) addTab();
  if (activeTab.value === key) activeTab.value = tabs.value[0]?.key || '';
}

function assignField(key: string) {
  const target =
    tabs.value.find((t) => t.key === activeTab.value) || tabs.value[0];
  if (!target) return;
  if (!target.fields.includes(key)) target.fields.push(key);
}

function moveField(tab: LayoutTab, idx: number, dir: -1 | 1) {
  const next = idx + dir;
  if (next < 0 || next >= tab.fields.length) return;
  const [f] = tab.fields.splice(idx, 1);
  if (f !== undefined) tab.fields.splice(next, 0, f);
}

function removeField(tab: LayoutTab, idx: number) {
  tab.fields.splice(idx, 1);
}

async function save() {
  saving.value = true;
  try {
    const m = props.model || {};
    await updateContentModelApi(m.id, {
      id: m.id,
      modelCode: m.modelCode,
      modelName: m.modelName,
      modelIcon: m.modelIcon || undefined,
      description: m.description || undefined,
      hasTitle: Number(m.hasTitle) || 0,
      hasContent: Number(m.hasContent) || 0,
      hasCover: Number(m.hasCover) || 0,
      hasAuthor: Number(m.hasAuthor) || 0,
      hasSummary: Number(m.hasSummary) || 0,
      hasSeo: Number(m.hasSeo) || 0,
      hasImages: Number(m.hasImages) || 0,
      hasAttachment: Number(m.hasAttachment) || 0,
      listTemplateId: m.listTemplateId ? Number(m.listTemplateId) : undefined,
      detailTemplateId: m.detailTemplateId ? Number(m.detailTemplateId) : undefined,
      sort: Number(m.sort) || 0,
      status: Number(m.status) || 1,
      formLayout: JSON.stringify({ columns: columns.value, tabs: tabs.value }),
    } as any);
    message.success('布局已保存，内容页将按此渲染');
    emit('saved');
    visible.value = false;
  } catch {
    message.error('保存失败');
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="layout-designer">
    <div class="ld-toolbar">
      <span>每行字段数：</span>
      <Select
        v-model:value="columns"
        :options="[
          { label: '一行 1 个（单列）', value: 1 },
          { label: '一行 2 个', value: 2 },
          { label: '一行 3 个', value: 3 },
        ]"
        style="width: 160px"
      />
      <Button class="ml-4" size="small" @click="addTab">
        <IconifyIcon icon="lucide:plus" class="mr-1 size-3.5" />
        添加选项卡
      </Button>
      <span class="ld-hint">不添加选项卡即平铺布局</span>
    </div>

    <Tabs
      v-model:activeKey="activeTab"
      type="editable-card"
      hide-add
      @edit="(key: any) => removeTab(String(key))"
    >
      <TabPane
        v-for="(t, i) in tabs"
        :key="t.key"
        :closable="tabs.length > 1"
      >
        <template #tab>
          <Input
            v-model:value="t.name"
            size="small"
            style="width: 110px"
            @click.stop
          />
        </template>
        <div class="ld-tab-body">
          <div v-if="t.fields.length === 0" class="ld-empty">
            该选项卡还没有字段，从下方「未分配字段」点击加入
          </div>
          <draggable
            :list="t.fields"
            item-key="(el) => el"
            group="cmsLayoutFields"
            :animation="200"
            class="ld-field-drag"
          >
            <template #item="{ element: k, index: fi }">
              <div class="ld-field-chip">
                <IconifyIcon icon="lucide:grip-vertical" class="ld-chip-drag" />
                <span>{{ allFieldLabels[k] || k }}</span>
                <span class="ld-chip-actions">
                  <IconifyIcon
                    icon="lucide:x"
                    class="size-3.5"
                    @click="removeField(t, fi)"
                  />
                </span>
              </div>
            </template>
          </draggable>
        </div>
        <div
          v-if="i === tabs.map((x) => x.key).indexOf(activeTab)"
          class="ld-assign"
        >
          <span class="ld-assign-tip">未分配字段（点击加入当前选项卡）：</span>
          <div class="ld-unassigned">
            <div
              v-for="f in unassigned"
              :key="f.key"
              class="ld-unassign-chip"
              @click="assignField(f.key)"
            >
              <IconifyIcon icon="lucide:plus" class="mr-1 size-3" />
              {{ f.label }}
            </div>
            <span v-if="unassigned.length === 0" class="ld-empty">
              全部字段已分配
            </span>
          </div>
        </div>
      </TabPane>
    </Tabs>

    <div class="ld-footer">
      <Button
        type="primary"
        :loading="saving"
        @click="save"
      >
        保存布局
      </Button>
    </div>
  </div>
</template>

<style scoped>
.ld-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}
.ld-hint {
  color: hsl(var(--foreground) / 45%);
  font-size: 12px;
}
.ld-tab-body {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  min-height: 60px;
  padding: 10px;
  border: 1px dashed hsl(var(--primary) / 40%);
  border-radius: 8px;
}
.ld-field-drag {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  min-height: 40px;
}
.ld-chip-drag {
  width: 14px;
  height: 14px;
  color: hsl(var(--muted-foreground));
  cursor: grab;
}
.ld-field-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: hsl(var(--primary) / 8%);
  border-radius: 6px;
  font-size: 13px;
}
.ld-chip-actions {
  display: inline-flex;
  gap: 4px;
  color: hsl(var(--foreground) / 55%);
  cursor: pointer;
}
.ld-chip-actions svg:hover {
  color: hsl(var(--primary));
}
.ld-assign {
  margin-top: 12px;
}
.ld-assign-tip {
  font-size: 12px;
  color: hsl(var(--foreground) / 55%);
}
.ld-unassigned {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 6px;
}
.ld-unassign-chip {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border: 1px dashed hsl(var(--primary) / 50%);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
}
.ld-unassign-chip:hover {
  background: hsl(var(--primary) / 8%);
}
.ld-empty {
  color: hsl(var(--foreground) / 40%);
  font-size: 12px;
}
.ld-footer {
  margin-top: 16px;
}
</style>
