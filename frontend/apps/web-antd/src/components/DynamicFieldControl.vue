<script lang="ts" setup>
import type { FieldSchemaItem, OptionItem } from '#/components/FieldSchemaAdapter';

import { computed, onMounted, ref } from 'vue';
import dayjs from 'dayjs';

import { DatePicker, Input, InputNumber, Select, Switch } from 'ant-design-vue';

import {
  getFileOptions,
  getUserOptions,
  loadFileOptions,
  loadUserOptions,
} from '#/components/FieldSchemaAdapter';

interface Props {
  item: FieldSchemaItem;
  value?: any;
  disabled?: boolean;
  /** 新建模式：值为空时按 options 预填默认值（固定值 / 当前时间）；编辑回显不触发 */
  prefill?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  value: undefined,
  disabled: false,
  prefill: false,
});

const emit = defineEmits<{ 'update:value': [value: any] }>();

const userOptions = ref<OptionItem[]>([]);
const fileOptions = ref<OptionItem[]>([]);
onMounted(async () => {
  // 默认值预填（仅新建模式、值为空时）：固定值 / 当前时间（defaultMode='now'）
  if (
    props.prefill &&
    (props.value === undefined || props.value === null || props.value === '')
  ) {
    const opts: any = props.item.options ?? {};
    const dv = opts.defaultValue;
    if (dv !== undefined && dv !== null && dv !== '') {
      emit('update:value', dv);
    } else if (opts.defaultMode === 'now') {
      const t = Number(props.item.fieldType);
      if (t === 4) emit('update:value', dayjs().format('YYYY-MM-DD'));
      else if (t === 5) emit('update:value', dayjs().format('YYYY-MM-DD HH:mm:ss'));
    }
  }
  // 成员/附件选项模块级缓存，schema 加载时已预拉，此处兜底幂等
  if (Number(props.item.fieldType) === 10) {
    userOptions.value = getUserOptions();
    if (userOptions.value.length === 0) {
      userOptions.value = await loadUserOptions();
    }
  }
  if (Number(props.item.fieldType) === 9) {
    fileOptions.value = getFileOptions();
    if (fileOptions.value.length === 0) {
      fileOptions.value = await loadFileOptions();
    }
  }
});

// G4：占位提示支持 options.placeholder 自定义，按控件类型给默认文案
const phText = computed(() => props.item.options?.placeholder || '请输入');
const phSelect = computed(() => props.item.options?.placeholder || '请选择');
const phDate = computed(() => props.item.options?.placeholder || '请选择日期');

const choiceOptions = computed(() =>
  (props.item.options?.choices ?? [])
    .filter((c) => Number(c.active) === 1)
    .map((c) => ({ label: c.label, value: String(c.value) })),
);
</script>

<template>
  <Input
    v-if="item.fieldType === 1"
    :value="value"
    :disabled="disabled"
    :maxlength="500"
    :placeholder="phText"
    allow-clear
    @update:value="(v: string) => emit('update:value', v)"
  />
  <Input.TextArea
    v-else-if="item.fieldType === 2"
    :value="value"
    :disabled="disabled"
    :rows="Number(item.options?.rows) || 3"
    :maxlength="2000"
    :placeholder="phText"
    @update:value="(v: string) => emit('update:value', v)"
  />
  <InputNumber
    v-else-if="item.fieldType === 3 || item.fieldType === 11"
    :value="
      value === undefined || value === null || value === ''
        ? undefined
        : Number(value)
    "
    :disabled="disabled"
    :precision="item.options?.precision"
    style="width: 100%"
    :placeholder="phText"
    @update:value="(v: any) => emit('update:value', v)"
  />
  <DatePicker
    v-else-if="item.fieldType === 4"
    :value="value"
    value-format="YYYY-MM-DD"
    :disabled="disabled"
    style="width: 100%"
    :placeholder="phDate"
    @update:value="(v: any) => emit('update:value', v)"
  />
  <DatePicker
    v-else-if="item.fieldType === 5"
    :value="value"
    value-format="YYYY-MM-DD HH:mm:ss"
    show-time
    :disabled="disabled"
    style="width: 100%"
    :placeholder="phDate"
    @update:value="(v: any) => emit('update:value', v)"
  />
  <Select
    v-else-if="item.fieldType === 6"
    :value="value === undefined || value === null ? undefined : String(value)"
    :options="choiceOptions"
    :disabled="disabled"
    :placeholder="phSelect"
    allow-clear
    style="width: 100%"
    @update:value="(v: any) => emit('update:value', v)"
  />
  <Select
    v-else-if="item.fieldType === 7"
    :value="Array.isArray(value) ? value.map(String) : []"
    :options="choiceOptions"
    :disabled="disabled"
    mode="multiple"
    :placeholder="props.item.options?.placeholder || '请选择（可多选）'"
    style="width: 100%"
    @update:value="(v: any) => emit('update:value', v)"
  />
  <Switch
    v-else-if="item.fieldType === 8"
    :checked="value === true"
    :disabled="disabled"
    @change="(v: any) => emit('update:value', v === true)"
  />
  <Select
    v-else-if="item.fieldType === 9"
    :value="Array.isArray(value) ? value.map(String) : []"
    :options="fileOptions"
    :disabled="disabled"
    mode="multiple"
    show-search
    option-filter-prop="label"
    placeholder="请选择附件"
    style="width: 100%"
    @update:value="(v: any) => emit('update:value', v)"
  />
  <Select
    v-else-if="item.fieldType === 10"
    :value="Array.isArray(value) ? value.map(String) : []"
    :options="userOptions"
    :disabled="disabled"
    mode="multiple"
    show-search
    option-filter-prop="label"
    placeholder="请选择成员"
    style="width: 100%"
    @update:value="(v: any) => emit('update:value', v)"
  />
  <!-- 未知类型兜底：只读展示，不可编辑（避免脏数据写入） -->
  <span v-else>{{ Array.isArray(value) ? value.join('、') : (value ?? '') }}</span>
</template>
