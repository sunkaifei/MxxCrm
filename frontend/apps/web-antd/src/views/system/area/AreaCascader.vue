<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';

import { Cascader } from 'ant-design-vue';

import { getAreaCascaderApi } from '#/api/core/system/area';

const props = withDefaults(
  defineProps<{
    changeOnSelect?: boolean;
    disabled?: boolean;
    modelValue?: (number | string)[];
    placeholder?: string;
    showSearch?: boolean;
  }>(),
  {
    modelValue: undefined,
    placeholder: '请选择地区',
    showSearch: true,
    disabled: false,
    changeOnSelect: false,
  },
);

const emit = defineEmits<{
  change: [value: (number | string)[]];
  'update:modelValue': [value: (number | string)[] | undefined];
}>();

const areaOptions = ref<any[]>([]);
const localValue = ref<(number | string)[] | undefined>(props.modelValue);

watch(
  () => props.modelValue,
  (newVal) => {
    localValue.value = newVal;
  },
);

const loadAreaData = async () => {
  try {
    const res = await getAreaCascaderApi();
    if (res.code === 200 && res.data) {
      areaOptions.value = res.data;
    }
  } catch (error) {
    console.error('加载地区数据失败:', error);
  }
};

const handleChange = (value: any) => {
  localValue.value = value;
  emit('update:modelValue', value);
  emit('change', value);
};

onMounted(() => {
  loadAreaData();
});
</script>

<template>
  <Cascader
    v-model="localValue"
    :options="areaOptions"
    :placeholder="placeholder"
    :show-search="showSearch"
    :disabled="disabled"
    :change-on-select="changeOnSelect"
    :field-names="{ label: 'label', value: 'value', children: 'children' }"
    @change="handleChange"
  />
</template>
