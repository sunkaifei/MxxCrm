<script lang="ts" setup>
/**
 * 自定义字段列表筛选条（G3 通用组件，与客户列表筛选同款交互）
 * 页面接入三步：
 *   1. <CustomFieldFilterBar ref="cfBarRef" :fields="cfFields" @change="gridApi.query()" />
 *   2. schema 加载后：cfFields.value = fieldSchema.getFilterFields()
 *   3. 列表查询参数展开：...cfBarRef?.getParams()
 */
import type { FilterFieldMeta } from '#/components/FieldSchemaAdapter';

import { computed, ref, watch } from 'vue';

import {
  Button,
  Input,
  InputNumber,
  Select,
  DatePicker,
} from 'ant-design-vue';

const props = defineProps<{
  fields: FilterFieldMeta[];
}>();
const emit = defineEmits<{ change: [] }>();

const cfFieldKey = ref<string>();
const cfOp = ref<string>();
const cfValue = ref<any>();
const cfSort = ref<string>();
const cfSortOrder = ref<'asc' | 'desc'>('desc');
// 范围模式（数字/金额/日期/日期时间）：起止两段输入，op 固定 between
const rangeMode = ref(false);
const rangeStart = ref<any>();
const rangeEnd = ref<any>();

const activeItem = computed(() =>
  props.fields.find((f) => f.fieldKey === cfFieldKey.value),
);
const supportsRange = computed(() =>
  [3, 11, 4, 5].includes(Number(activeItem.value?.fieldType)),
);
const opOptions = computed(() =>
  (activeItem.value?.operators ?? []).map((o) => ({
    label: o.label,
    value: o.value,
  })),
);

watch(cfFieldKey, () => {
  // 切字段：重置操作符与值（数组型/布尔有固定操作符）
  cfOp.value = activeItem.value?.fixedOp;
  cfValue.value = undefined;
  rangeMode.value = false;
  rangeStart.value = undefined;
  rangeEnd.value = undefined;
  emit('change');
});

function setRangeMode(v: boolean) {
  rangeMode.value = v;
  cfOp.value = v ? 'between' : activeItem.value?.fixedOp;
  cfValue.value = undefined;
  rangeStart.value = undefined;
  rangeEnd.value = undefined;
  emit('change');
}

function onFilterChange() {
  emit('change');
}

function onSortChange() {
  emit('change');
}

function clearFilter() {
  cfFieldKey.value = undefined;
  cfOp.value = undefined;
  cfValue.value = undefined;
  rangeMode.value = false;
  rangeStart.value = undefined;
  rangeEnd.value = undefined;
  emit('change');
}

/** 页面查询参数展开项（camelCase，与后端 ListQuery 的 cf_* 对应） */
function getParams() {
  const item = activeItem.value;
  const base = {
    cfSort: cfSort.value || undefined,
    cfSortOrder: cfSort.value ? cfSortOrder.value : undefined,
  };
  if (!item) return { ...base, cfKey: undefined, cfOp: undefined, cfVal: undefined };

  // 范围模式：起止齐备才生效，op 固定 between，值 JSON 数组字符串（后端 deserialize_cf_val 解析）
  if (rangeMode.value) {
    const hasStart = rangeStart.value !== undefined && rangeStart.value !== null && rangeStart.value !== '';
    const hasEnd = rangeEnd.value !== undefined && rangeEnd.value !== null && rangeEnd.value !== '';
    if (!hasStart && !hasEnd) return { ...base, cfKey: undefined, cfOp: undefined, cfVal: undefined };
    // 只填一端时退化为单边比较
    if (hasStart !== hasEnd) {
      return {
        cfKey: cfFieldKey.value,
        cfOp: hasStart ? 'gte' : 'lte',
        cfVal: hasStart ? rangeStart.value : rangeEnd.value,
        ...base,
      };
    }
    return {
      cfKey: cfFieldKey.value,
      cfOp: 'between',
      cfVal: JSON.stringify([rangeStart.value, rangeEnd.value]),
      ...base,
    };
  }

  if (cfValue.value === undefined || cfValue.value === null || cfValue.value === '') {
    return { ...base, cfKey: undefined, cfOp: undefined, cfVal: undefined };
  }
  return {
    cfKey: cfFieldKey.value,
    cfOp: item.fixedOp ?? cfOp.value,
    cfVal: cfValue.value,
    ...base,
  };
}

defineExpose({ getParams });
</script>

<template>
  <div class="mb-2 flex flex-wrap items-center gap-2">
    <Select
      v-model:value="cfFieldKey"
      :options="fields.map((f) => ({ label: f.fieldLabel, value: f.fieldKey }))"
      allow-clear
      class="w-44"
      placeholder="自定义字段"
    />
    <template v-if="cfFieldKey">
      <Select
        v-if="supportsRange"
        :value="rangeMode ? 'range' : 'single'"
        :options="[
          { label: '指定值', value: 'single' },
          { label: '范围', value: 'range' },
        ]"
        class="w-28"
        @change="
          (v: any) => {
            setRangeMode(v === 'range');
          }
        "
      />
      <Select
        v-if="opOptions.length > 0 && !rangeMode"
        v-model:value="cfOp"
        :options="opOptions"
        class="w-32"
        placeholder="操作符"
      />
      <!-- 布尔：固定 开/关 -->
      <Select
        v-else-if="Number(activeItem?.fieldType) === 8"
        v-model:value="cfValue"
        :options="[
          { label: '是', value: 'true' },
          { label: '否', value: 'false' },
        ]"
        allow-clear
        class="w-40"
        placeholder="请选择"
      />
      <!-- 单选 -->
      <Select
        v-else-if="Number(activeItem?.fieldType) === 6"
        v-model:value="cfValue"
        :options="activeItem?.choices ?? []"
        allow-clear
        class="w-40"
        placeholder="请选择"
      />
      <!-- 多选/附件/成员：包含任一 -->
      <Select
        v-else-if="[7, 9, 10].includes(Number(activeItem?.fieldType))"
        v-model:value="cfValue"
        :options="activeItem?.choices ?? []"
        allow-clear
        class="w-48"
        mode="multiple"
        placeholder="包含任一"
      />
      <!-- 数字/金额 -->
      <InputNumber
        v-else-if="[3, 11].includes(Number(activeItem?.fieldType))"
        v-model:value="cfValue"
        class="w-40"
        placeholder="请输入数值"
      />
      <!-- 日期 -->
      <DatePicker
        v-else-if="Number(activeItem?.fieldType) === 4"
        v-model:value="cfValue"
        class="w-40"
        placeholder="请选择日期"
        value-format="YYYY-MM-DD"
      />
      <!-- 日期时间 -->
      <DatePicker
        v-else-if="Number(activeItem?.fieldType) === 5"
        v-model:value="cfValue"
        class="w-56"
        placeholder="请选择时间"
        show-time
        value-format="YYYY-MM-DD HH:mm:ss"
      />
      <!-- 文本/多行文本：包含 -->
      <Input
        v-else
        v-model:value="cfValue"
        allow-clear
        class="w-48"
        placeholder="请输入关键字"
        @press-enter="onFilterChange"
      />
    </template>
    <template v-if="cfFieldKey && rangeMode && supportsRange">
      <template v-if="Number(activeItem?.fieldType) === 4 || Number(activeItem?.fieldType) === 5">
        <DatePicker
          v-model:value="rangeStart"
          class="w-40"
          :placeholder="Number(activeItem?.fieldType) === 5 ? '开始时间' : '开始日期'"
          :show-time="Number(activeItem?.fieldType) === 5"
          :value-format="Number(activeItem?.fieldType) === 5 ? 'YYYY-MM-DD HH:mm:ss' : 'YYYY-MM-DD'"
          @change="onFilterChange"
        />
        <span class="text-gray-400">~</span>
        <DatePicker
          v-model:value="rangeEnd"
          class="w-40"
          :placeholder="Number(activeItem?.fieldType) === 5 ? '结束时间' : '结束日期'"
          :show-time="Number(activeItem?.fieldType) === 5"
          :value-format="Number(activeItem?.fieldType) === 5 ? 'YYYY-MM-DD HH:mm:ss' : 'YYYY-MM-DD'"
          @change="onFilterChange"
        />
      </template>
      <template v-else>
        <InputNumber
          v-model:value="rangeStart"
          class="w-36"
          placeholder="最小值"
          @change="onFilterChange"
        />
        <span class="text-gray-400">~</span>
        <InputNumber
          v-model:value="rangeEnd"
          class="w-36"
          placeholder="最大值"
          @change="onFilterChange"
        />
      </template>
    </template>
    <Select
      v-model:value="cfSort"
      :options="fields.map((f) => ({ label: `按「${f.fieldLabel}」排序`, value: f.fieldKey }))"
      allow-clear
      class="w-52"
      placeholder="自定义排序"
      @change="onSortChange"
    />
    <Select
      v-if="cfSort"
      v-model:value="cfSortOrder"
      :options="[
        { label: '降序', value: 'desc' },
        { label: '升序', value: 'asc' },
      ]"
      class="w-24"
      @change="onSortChange"
    />
    <Button v-if="cfFieldKey || cfSort" type="link" @click="clearFilter">清空</Button>
  </div>
</template>
