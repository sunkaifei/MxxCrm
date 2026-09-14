<script lang="ts" setup>
import type { ContentModelFieldSaveDTO } from '#/api/core/website/content-model';

import { computed, reactive, ref } from 'vue';

import { useVbenDrawer, z } from '@vben/common-ui';

import { message } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { addContentModelFieldApi, updateContentModelFieldApi } from '#/api';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value ? '新增模型字段' : '编辑模型字段',
);

const switchFields = [
  'isRequired',
  'isSearchable',
  'isListShow',
  'isDetailShow',
] as const;

const fieldTypeOptions = [
  { label: '单行文本', value: 1 },
  { label: '多行文本', value: 2 },
  { label: '富文本', value: 3 },
  { label: '数字', value: 4 },
  { label: '日期', value: 5 },
  { label: '下拉选择', value: 6 },
  { label: '单选', value: 7 },
  { label: '多选', value: 8 },
  { label: '图片', value: 9 },
  { label: '文件', value: 10 },
];

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle">
    <BaseForm />
  </Drawer>
</template>
