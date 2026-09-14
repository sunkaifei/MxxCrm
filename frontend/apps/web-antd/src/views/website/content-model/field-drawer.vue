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

/** 解析字段选项 JSON（与通用内容页 parseOptions 同口径） */
function parseFieldOptions(raw?: string): { label: string; value: any }[] {
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

/** 提交/回显时，按字段类型在「当前类型的默认值字段名」与存储值间换算 */
function defaultValueFieldName(fieldType?: number): string {
  const t = Number(fieldType);
  if (t === 4) return 'defaultValueNumber';
  if (t === 5) return 'defaultValueDate';
  if (t === 8) return 'defaultValueMulti';
  if (t === 6 || t === 7) return 'defaultValueSelect';
  return 'defaultValueText';
}

// reactive 化：框架会 watch options.schema（use-vben-form.ts），改 schema 即可换控件
const formOptions = reactive({
  showDefaultActions: false,
  // 大屏两列网格，窄屏单列
  wrapperClass: 'grid-cols-1 md:grid-cols-2 gap-x-6',
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
      // 与后端 is_valid_identifier 保持一致：字母/下划线开头，仅含字母数字下划线
      rules: z
        .string()
        .min(1, { message: '请输入字段名称' })
        .regex(/^[a-zA-Z_][a-zA-Z0-9_]*$/, {
          message:
            '字段名称须为英文标识符（字母或下划线开头，仅含字母、数字、下划线）',
        }),
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
        // 下拉面板挂 body（z≈1050），本抽屉 zIndex 2500，需抬高才可见
        dropdownStyle: { zIndex: 3000 },
      },
      rules: z.any().refine((val) => val !== undefined && val !== null, {
        message: '请选择字段类型',
      }),
    },
    {
      component: 'Textarea',
      fieldName: 'fieldOptions',
      label: '字段选项',
      help: '仅「下拉选择/单选/多选」类型需要；JSON 数组格式，如 [{"label":"选项A","value":"a"}]，默认值下拉也将引用这里的选项',
      componentProps: {
        placeholder:
          '[{"label":"选项A","value":"a"},{"label":"选项B","value":"b"}]',
        allowClear: true,
        rows: 4,
      },
      dependencies: {
        triggerFields: ['fieldType'],
        if: (values: Record<string, any>) => [6, 7, 8].includes(values.fieldType),
      },
    },
    // ===== 默认值：按「字段类型」互斥渲染（dependencies.if 实测可靠），防跨类型乱填 =====
    {
      component: 'Input',
      fieldName: 'defaultValueText',
      label: '默认值',
      help: '文本类型的默认内容',
      componentProps: {
        placeholder: '默认文本',
        allowClear: true,
      },
      dependencies: {
        triggerFields: ['fieldType'],
        if: (values: Record<string, any>) =>
          [1, 2, 3, 9, 10].includes(values.fieldType),
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'defaultValueNumber',
      label: '默认值',
      help: '数字类型的默认值，只能填数字',
      componentProps: {
        placeholder: '只能填数字，如 100',
        class: 'w-full',
      },
      dependencies: {
        triggerFields: ['fieldType'],
        if: (values: Record<string, any>) => values.fieldType === 4,
      },
    },
    {
      component: 'Select',
      fieldName: 'defaultValueMode',
      label: '默认值类型',
      defaultValue: '',
      componentProps: {
        dropdownStyle: { zIndex: 3000 },
        options: [
          { label: '不设默认', value: '' },
          { label: '指定日期', value: 'fixed' },
          { label: '当前时间（录入内容时动态取）', value: 'now' },
        ],
      },
      dependencies: {
        triggerFields: ['fieldType'],
        if: (values: Record<string, any>) => values.fieldType === 5,
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'defaultValueDate',
      label: '指定日期',
      help: '日期类型的默认值，从日历选择',
      componentProps: {
        placeholder: '从日历选择日期',
        valueFormat: 'YYYY-MM-DD',
        style: 'width:100%',
      },
      dependencies: {
        triggerFields: ['fieldType', 'defaultValueMode'],
        if: (values: Record<string, any>) =>
          values.fieldType === 5 && values.defaultValueMode === 'fixed',
      },
    },
    {
      component: 'Select',
      fieldName: 'defaultValueSelect',
      label: '默认值',
      help: '默认值需从「字段选项」中选择',
      componentProps: (values: Record<string, any>) => ({
        options: parseFieldOptions(values.fieldOptions),
        placeholder: '请选择默认值',
        allowClear: true,
        dropdownStyle: { zIndex: 3000 },
      }),
      dependencies: {
        triggerFields: ['fieldType', 'fieldOptions'],
        if: (values: Record<string, any>) =>
          values.fieldType === 6 || values.fieldType === 7,
      },
    },
    {
      component: 'Select',
      fieldName: 'defaultValueMulti',
      label: '默认值',
      help: '可多选，提交时以逗号合并保存',
      componentProps: (values: Record<string, any>) => ({
        options: parseFieldOptions(values.fieldOptions),
        mode: 'multiple',
        placeholder: '可多选',
        dropdownStyle: { zIndex: 3000 },
      }),
      dependencies: {
        triggerFields: ['fieldType', 'fieldOptions'],
        if: (values: Record<string, any>) => values.fieldType === 8,
      },
    },

    {
      component: 'Input',
      fieldName: 'placeholder',
      label: '占位提示',
      help: '前台录入内容时，输入框内显示的灰色提示文字（如：请输入姓名）',
      componentProps: {
        placeholder: '如：请输入姓名',
        allowClear: true,
      },
    },
    {
      component: 'Switch',
      fieldName: 'isRequired',
      label: '是否必填',
      defaultValue: false,
      componentProps: { class: 'w-auto' },
    },
    {
      component: 'Switch',
      fieldName: 'isSearchable',
      label: '是否可搜索',
      defaultValue: false,
      componentProps: { class: 'w-auto' },
    },
    {
      component: 'Switch',
      fieldName: 'isListShow',
      label: '列表显示',
      defaultValue: true,
      componentProps: { class: 'w-auto' },
    },
    {
      component: 'Switch',
      fieldName: 'isDetailShow',
      label: '详情显示',
      defaultValue: true,
      componentProps: { class: 'w-auto' },
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
        dropdownStyle: { zIndex: 3000 },
        options: [
          { label: '启用', value: 1 },
          { label: '禁用', value: 0 },
        ],
      },
    },
  ] as any,
});

const [BaseForm, baseFormApi] = useVbenForm(formOptions as any);

const [Drawer, drawerApi] = useVbenDrawer({
  // 75% 宽；zIndex 需高于字段管理 AntDrawer（实际 2000），否则被遮住
  class: 'w-[75%]',
  zIndex: 2500,
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
    // 默认值：取当前类型对应的字段值回填 defaultValue，多选以逗号合并；
    // 日期类型按「默认值类型」落值：'' 不设 / 'fixed'→指定日期 / 'now'→哨兵 $now（内容录入时动态取当天）
    const dvField = defaultValueFieldName(values.fieldType);
    let dv = values[dvField];
    if (Array.isArray(dv)) dv = dv.join(',');
    if (Number(values.fieldType) === 5) {
      dv =
        values.defaultValueMode === 'now'
          ? '$now'
          : values.defaultValueMode === 'fixed'
            ? (values.defaultValueDate || '')
            : '';
    }
    values.defaultValue = dv ?? '';
    // 清理中间字段，避免多余键提交
    [
      'defaultValueText',
      'defaultValueNumber',
      'defaultValueDate',
      'defaultValueSelect',
      'defaultValueMulti',
      'defaultValueMode',
    ].forEach((k) => delete values[k]);
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
      // 默认值：按字段类型分发到对应的互斥字段（多选拆数组；日期解析模式）
      const dvName = defaultValueFieldName(values.fieldType);
      if (Number(values.fieldType) === 5) {
        if (values.defaultValue === '$now') {
          values.defaultValueMode = 'now';
        } else if (values.defaultValue) {
          values.defaultValueMode = 'fixed';
          values.defaultValueDate = values.defaultValue;
        } else {
          values.defaultValueMode = '';
        }
      } else {
        values[dvName] = values.defaultValue || '';
      }
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

<style>
/* 抽屉内 antd 弹层（日历/下拉）默认 z≈1050，低于 vben 抽屉 2500 会被遮；统一抬高 */
.ant-picker-dropdown,
.ant-select-dropdown {
  z-index: 3000 !important;
}
</style>
