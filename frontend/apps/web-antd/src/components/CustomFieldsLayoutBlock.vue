<script lang="ts" setup>
/**
 * 自定义字段布局块（表单模式）
 * 按布局元数据（layout_type=1）渲染自定义字段：选项卡分组 + 半行/整行 + 拖拽保存的顺序；
 * 未编排字段自动追加在末尾（unassignedPolicy=append，新字段永不丢失）。
 * 值容器由父组件传入（reactive 对象），提交/回显逻辑保留在父组件（buildSubmitPayload 既有链路）。
 */
import { computed, onMounted, ref } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Form, TabPane, Tabs } from 'ant-design-vue';

import DynamicFieldControl from './DynamicFieldControl.vue';
import { fetchFormLayout, type ParsedFormLayout } from './FormLayoutManager';

const props = defineProps<{
  disabled?: (item: any) => boolean;
  /** 字段元数据（useFieldSchema().items.value，已按当前用户角色过滤） */
  items: any[];
  module: string;
  /** 值容器（父组件 reactive 对象，key=fieldKey） */
  values: Record<string, any>;
  /** 新建模式：值为空时预填默认值 */
  prefill?: boolean;
}>();

const layout = ref<ParsedFormLayout | null>(null);
const activeTab = ref<string>('');

onMounted(async () => {
  layout.value = await fetchFormLayout(props.module, 1);
  if (layout.value?.tabs.length) activeTab.value = layout.value.tabs[0]!.key;
});

/** 布局排序后的全部字段（未编排的追加在后） */
const orderedItems = computed(() => {
  const arr = [...props.items];
  if (!layout.value) return arr;
  const orderMap = new Map<string, number>();
  layout.value.fields.forEach((f, i) => orderMap.set(f.key, i));
  const inLayout = arr.filter((i) => orderMap.has(i.fieldKey));
  const rest = arr.filter((i) => !orderMap.has(i.fieldKey));
  inLayout.sort(
    (a, b) => orderMap.get(a.fieldKey)! - orderMap.get(b.fieldKey)!,
  );
  return [...inLayout, ...rest];
});

/** 无布局/无选项卡：单区平铺 */
const flatItems = computed(() =>
  layout.value?.tabs.length ? [] : orderedItems.value,
);

/** 选项卡分组渲染：tab 内字段 + 未归入选项卡字段的默认区 */
const tabGroups = computed(() => {
  if (!layout.value?.tabs.length) return [];
  const byTab = new Map<string, any[]>();
  const defaultTab: any[] = [];
  for (const item of orderedItems.value) {
    const hit = layout.value!.fields.find((f) => f.key === item.fieldKey);
    if (hit?.tab && layout.value!.tabs.some((t) => t.key === hit.tab)) {
      if (!byTab.has(hit.tab)) byTab.set(hit.tab, []);
      byTab.get(hit.tab)!.push(item);
    } else {
      defaultTab.push(item);
    }
  }
  const groups = layout.value.tabs.map((t) => ({
    key: t.key,
    title: t.title,
    items: byTab.get(t.key) ?? [],
  }));
  if (defaultTab.length > 0) {
    groups.unshift({ key: '__default__', title: '基本信息', items: defaultTab });
  }
  return groups;
});

function spanClass(item: any): string {
  if (!layout.value) {
    return Number(item.fieldType) === 2 ? 'col-span-2' : 'col-span-1';
  }
  const hit = layout.value.fields.find((f) => f.key === item.fieldKey);
  return hit?.span === 2 ? 'col-span-2' : 'col-span-1';
}
</script>

<template>
  <div v-if="orderedItems.length > 0" class="custom-fields-layout-block">
    <div class="mb-1 flex items-center gap-1 text-xs text-gray-400">
      <IconifyIcon icon="lucide:sparkles" />
      扩展字段
    </div>

    <!-- 平铺（无布局/无选项卡） -->
    <Form v-if="flatItems.length > 0" layout="vertical" class="grid grid-cols-1 gap-x-6 md:grid-cols-2">
      <Form.Item
        v-for="item in flatItems"
        :key="item.fieldKey"
        :label="item.fieldLabel"
        :required="Number(item.required) === 1"
        :class="spanClass(item)"
      >
        <DynamicFieldControl
          v-model:value="values[item.fieldKey]"
          :item="item"
          :prefill="prefill"
          :disabled="disabled ? disabled(item) : false"
        />
      </Form.Item>
    </Form>

    <!-- 选项卡分组 -->
    <Tabs v-else v-model:activeKey="activeTab">
      <TabPane v-for="g in tabGroups" :key="g.key" :tab="g.title">
        <Form layout="vertical" class="grid grid-cols-1 gap-x-6 md:grid-cols-2">
          <Form.Item
            v-for="item in g.items"
            :key="item.fieldKey"
            :label="item.fieldLabel"
            :required="Number(item.required) === 1"
            :class="spanClass(item)"
          >
            <DynamicFieldControl
              v-model:value="values[item.fieldKey]"
              :item="item"
              :disabled="disabled ? disabled(item) : false"
            />
          </Form.Item>
          <div v-if="g.items.length === 0" class="col-span-2 py-4 text-center text-gray-300">
            该选项卡暂无字段
          </div>
        </Form>
      </TabPane>
    </Tabs>
  </div>
</template>
