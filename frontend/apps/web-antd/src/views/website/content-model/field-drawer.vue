<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import {
  addContentModelFieldApi,
  updateContentModelFieldApi,
} from '#/api';
import type { ContentModelFieldSaveDTO } from '#/api/core/website/content-model';

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

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Input',
      fieldName: 'fieldName',
      label: '字段名称',
      componentProps: {
        placeholder: '请输入英文标识符（如 author）',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入字段名称' }),
    },
    {
      component: 'Input',
      fieldName: 'fieldLabel',
      label: '字段标签',
      componentProps: {
        placeholder: '请输入中文显示名',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'fieldType',
      label: '字段类型',
      defaultValue: 1,
      componentProps: {
        options: fieldTypeOptions,
        placeholder: '请选择字段类型',
      },
      rules: z.any().refine((val) => val !== undefined && val !== null, {
        message: '请选择字段类型',
      }),
    },
    {
      component: 'Textarea',
      fieldName: 'fieldOptions',
      label: '字段选项',
      componentProps: {
        placeholder:
          'JSON 格式，如 [{"label":"选项A","value":"a"}]',
        allowClear: true,
        rows: 4,
      },
      dependencies: {
        triggerFields: ['fieldType'],
        if: (values: Record<string, any>) =>
          [6, 7, 8].includes(values.fieldType),
      },
    },
    {
      component: 'Input',
      fieldName: 'defaultValue',
      label: '默认值',
      componentProps: {
        placeholder: '请输入默认值',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'placeholder',
      label: '占位提示',
      componentProps: {
        placeholder: '请输入占位提示文字',
        allowClear: true,
      },
    },
    {
      component: 'Switch',
      fieldName: 'isRequired',
      label: '是否必填',
      defaultValue: false,
    },
    {
      component: 'Switch',
      fieldName: 'isSearchable',
      label: '是否可搜索',
      defaultValue: false,
    },
    {
      component: 'Switch',
      fieldName: 'isListShow',
      label: '列表显示',
      defaultValue: true,
    },
    {
      component: 'Switch',
      fieldName: 'isDetailShow',
      label: '详情显示',
      defaultValue: true,
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: '排序',
      defaultValue: 0,
      componentProps: {
        min: 0,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      defaultValue: 1,
      componentProps: {
        options: [
          { label: '启用', value: 1 },
          { label: '禁用', value: 0 },
        ],
      },
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) {
      return;
    }

    setLoading(true);

    const values = await baseFormApi.getValues();
    // Switch 返回布尔值，转为 0/1
    switchFields.forEach((key) => {
      values[key] = values[key] ? 1 : 0;
    });
    // 关联模型 ID
    values.modelId = data.value?.modelId;

    try {
      if (isCreate.value) {
        await addContentModelFieldApi(values as ContentModelFieldSaveDTO);
        message.success('新增成功');
      } else {
        await updateContentModelFieldApi(
          data.value.row.id,
          values as ContentModelFieldSaveDTO,
        );
        message.success('修改成功');
      }
      drawerApi.setData({ needRefresh: true });
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row || {};
      // 数字 0/1 转布尔给 Switch
      const values: Record<string, any> = { ...row };
      switchFields.forEach((key) => {
        values[key] = values[key] === 1;
      });
      baseFormApi.setValues(values);
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle">
    <BaseForm />
  </Drawer>
</template>
