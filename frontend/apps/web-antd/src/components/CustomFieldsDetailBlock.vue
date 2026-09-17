<script lang="ts" setup>
/**
 * 自定义字段布局块（详情模式）
 * 按 layout_type=2 布局渲染只读字段：顺序 + 半行/整行（Descriptions span）；
 * 显示判断 = 布局编排 ∩ 字段可见权限（items 已由后端按角色过滤下发）；
 * 无布局时回落"有值才显示"的既有行为（报价/订单详情原逻辑）。
 */
import { computed, onMounted, ref } from 'vue';

import { Descriptions, TabPane, Tabs } from 'ant-design-vue';

import { fetchFormLayout, type ParsedFormLayout } from './FormLayoutManager';

const props = defineProps<{
  /** 详情列数（Descriptions column），默认 2 */
  column?: number;
  /** 自定义字段值格式化（useFieldSchema().formatFieldValue） */
  formatter: (item: any, value: any) => string;
  items: any[];
  module: string;
  /** 记录的 customFields 值对象 */
  values: Record<string, any>;
}>();

const layout = ref<ParsedFormLayout | null>(null);
const activeTab = ref<string>('');

onMounted(async () => {
  layout.value = await fetchFormLayout(props.module, 2);
  if (layout.value?.tabs.length) activeTab.value = layout.value.tabs[0]!.key;
});

function displayValue(item: any): string {
  return props.formatter(item, props.values?.[item.fieldKey]);
}

/** 展示条件：布局编排内恒显示（空值显示 '-'）；未编排字段仅有值时追加 */
const orderedItems = computed(() => {
  const arr = [...props.items];
  if (!layout.value) {
    // 回落：仅展示有值字段（与报价/订单详情既有行为一致）
    return arr.filter((i) => {
      const v = props.values?.[i.fieldKey];
      return v !== undefined && v !== null && v !== '';
    });
  }
  const orderMap = new Map<string, number>();
  layout.value.fields.forEach((f, i) => orderMap.set(f.key, i));
  const inLayout = arr.filter((i) => orderMap.has(i.fieldKey));
  const rest = arr.filter((i) => {
    if (orderMap.has(i.fieldKey)) return false;
    const v = props.values?.[i.fieldKey];
    return layout.value!.unassignedPolicy !== 'hidden' && v !== undefined && v !== null && v !== '';
  });
  inLayout.sort((a, b) => orderMap.get(a.fieldKey)! - orderMap.get(b.fieldKey)!);
  return [...inLayout, ...rest];
});

const flatItems = computed(() =>
  layout.value?.tabs.length ? [] : orderedItems.value,
);

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

function descSpan(item: any): number {
  if (!layout.value) return 1;
  const hit = layout.value.fields.find((f) => f.key === item.fieldKey);
  return hit?.span === 2 ? 2 : 1;
}
</script>

<template>
  <div v-if="orderedItems.length > 0">
    <Descriptions v-if="flatItems.length > 0" :column="column ?? 2" bordered size="small">
      <Descriptions.Item
        v-for="item in flatItems"
        :key="item.fieldKey"
        :label="item.fieldLabel"
        :span="descSpan(item)"
      >
        {{ displayValue(item) || '-' }}
      </Descriptions.Item>
    </Descriptions>

    <Tabs v-else v-model:activeKey="activeTab">
      <TabPane v-for="g in tabGroups" :key="g.key" :tab="g.title">
        <Descriptions :column="column ?? 2" bordered size="small">
          <Descriptions.Item
            v-for="item in g.items"
            :key="item.fieldKey"
            :label="item.fieldLabel"
            :span="descSpan(item)"
          >
            {{ displayValue(item) || '-' }}
          </Descriptions.Item>
        </Descriptions>
      </TabPane>
    </Tabs>
  </div>
</template>
