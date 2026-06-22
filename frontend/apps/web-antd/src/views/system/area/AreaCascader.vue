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

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { Cascader } from 'ant-design-vue';
import { getAreaCascaderApi } from '#/api/core/system/area';

const props = withDefaults(defineProps<{
  modelValue?: string[];
  placeholder?: string;
  showSearch?: boolean;
  disabled?: boolean;
  changeOnSelect?: boolean;
}>(), {
  placeholder: '请选择地区',
  showSearch: true,
  disabled: false,
  changeOnSelect: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: string[] | undefined];
  change: [value: string[]];
}>();

const areaOptions = ref<any[]>([]);
const localValue = ref<string[] | undefined>(props.modelValue);

watch(() => props.modelValue, (newVal) => {
  localValue.value = newVal;
});

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

const handleChange = (value: string[]) => {
  localValue.value = value;
  emit('update:modelValue', value);
  emit('change', value);
};

onMounted(() => {
  loadAreaData();
});
</script>
