<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import {
  addTemplateVarApi,
  getTemplateVarDetailApi,
  updateTemplateVarApi,
} from '#/api';
import type { TemplateVarSaveDTO } from '#/api/core/website/template-var';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value ? '新增模板变量' : '编辑模板变量',
);

const varTypeOptions = [
  { label: '文本', value: 1 },
  { label: '数字', value: 2 },
  { label: '布尔', value: 3 },
  { label: 'HTML', value: 4 },
  { label: '图片', value: 5 },
];

const varGroupOptions = [
  { label: '默认', value: 'default' },
  { label: '联系信息', value: 'contact' },
  { label: '统计代码', value: 'stats' },
  { label: '品牌', value: 'brand' },
  { label: 'SEO', value: 'seo' },
  { label: '自定义', value: 'custom' },
];

const valueFields = [
  'varValueText',
  'varValueNumber',
  'varValueBool',
  'varValueHtml',
  'varValueImage',
] as const;

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
      fieldName: 'varKey',
      label: '变量KEY',
      componentProps: () => ({
        placeholder: '请输入变量KEY（唯一标识）',
        disabled: !isCreate.value,
      }),
      rules: z.string().min(1, { message: '请输入变量KEY' }),
    },
    {
      component: 'Input',
      fieldName: 'varLabel',
      label: '变量标签',
      componentProps: {
        placeholder: '请输入变量标签',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'varType',
      label: '变量类型',
      defaultValue: 1,
      componentProps: {
        options: varTypeOptions,
        placeholder: '请选择变量类型',
      },
      rules: z.any().refine((val) => val !== undefined && val !== null, {
        message: '请选择变量类型',
      }),
    },
    {
      component: 'Input',
      fieldName: 'varValueText',
      label: '变量值',
      componentProps: {
        placeholder: '请输入文本值',
        allowClear: true,
      },
      dependencies: {
        triggerFields: ['varType'],
        if: (values: Record<string, any>) => values.varType === 1,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'varValueNumber',
      label: '变量值',
      componentProps: {
        placeholder: '请输入数字值',
      },
      dependencies: {
        triggerFields: ['varType'],
        if: (values: Record<string, any>) => values.varType === 2,
      },
    },
    {
      component: 'Switch',
      fieldName: 'varValueBool',
      label: '变量值',
      defaultValue: false,
      dependencies: {
        triggerFields: ['varType'],
        if: (values: Record<string, any>) => values.varType === 3,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'varValueHtml',
      label: '变量值',
      componentProps: {
        placeholder: '请输入HTML内容',
        rows: 6,
      },
      dependencies: {
        triggerFields: ['varType'],
        if: (values: Record<string, any>) => values.varType === 4,
      },
    },
    {
      component: 'Input',
      fieldName: 'varValueImage',
      label: '变量值',
      componentProps: {
        placeholder: '请输入图片URL',
        allowClear: true,
      },
      dependencies: {
        triggerFields: ['varType'],
        if: (values: Record<string, any>) => values.varType === 5,
      },
    },
    {
      component: 'Select',
      fieldName: 'varGroup',
      label: '变量分组',
      defaultValue: 'default',
      componentProps: {
        options: varGroupOptions,
        placeholder: '请选择分组',
      },
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

// 根据类型将 varValue 分发到对应字段
function distributeVarValue(row: Record<string, any>): Record<string, any> {
  const values: Record<string, any> = { ...row };
  const raw = row.varValue ?? '';
  // 清空所有值字段
  valueFields.forEach((f) => {
    values[f] = undefined;
  });
  switch (row.varType) {
    case 1: {
      values.varValueText = raw;
      break;
    }
    case 2: {
      values.varValueNumber = raw === '' ? undefined : Number(raw);
      break;
    }
    case 3: {
      values.varValueBool = raw === 'true' || raw === '1';
      break;
    }
    case 4: {
      values.varValueHtml = raw;
      break;
    }
    case 5: {
      values.varValueImage = raw;
      break;
    }
    default: {
      values.varValueText = raw;
      break;
    }
  }
  return values;
}

// 根据类型从对应字段收集 varValue
function collectVarValue(values: Record<string, any>): string {
  switch (values.varType) {
    case 1: {
      return values.varValueText ?? '';
    }
    case 2: {
      return values.varValueNumber != null
        ? String(values.varValueNumber)
        : '';
    }
    case 3: {
      return values.varValueBool ? 'true' : 'false';
    }
    case 4: {
      return values.varValueHtml ?? '';
    }
    case 5: {
      return values.varValueImage ?? '';
    }
    default: {
      return '';
    }
  }
}

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
    const payload: TemplateVarSaveDTO = {
      varKey: values.varKey,
      varLabel: values.varLabel,
      varType: values.varType,
      varValue: collectVarValue(values),
      varGroup: values.varGroup,
      sort: values.sort,
      status: values.status,
    };

    try {
      if (isCreate.value) {
        await addTemplateVarApi(payload);
        message.success('新增成功');
      } else {
        await updateTemplateVarApi(data.value.row.id, payload);
        message.success('修改成功');
      }
      drawerApi.setData({ needRefresh: true });
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },

  async onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      if (!isCreate.value && data.value?.row?.id) {
        const detail: any = await getTemplateVarDetailApi(data.value.row.id);
        const row = detail?.data || detail || data.value.row;
        baseFormApi.setValues(distributeVarValue(row));
      } else {
        baseFormApi.setValues({});
      }
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
