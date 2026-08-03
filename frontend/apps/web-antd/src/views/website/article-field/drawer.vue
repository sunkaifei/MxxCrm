<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import {
  addArticleFieldApi,
  categoryApi,
  updateArticleFieldApi,
} from '#/api';
import type { ArticleFieldSaveDTO } from '#/api/core/website/article-field';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value ? '新增字段定义' : '编辑字段定义',
);

const categoryTree = ref<any[]>([]);

// 字段类型：1=文本 2=富文本 3=图片 4=数字 5=日期 6=下拉 7=多选
const fieldTypeOptions = [
  { label: '文本', value: 1 },
  { label: '富文本', value: 2 },
  { label: '图片', value: 3 },
  { label: '数字', value: 4 },
  { label: '日期', value: 5 },
  { label: '下拉', value: 6 },
  { label: '多选', value: 7 },
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
      component: 'TreeSelect',
      fieldName: 'categoryId',
      label: '所属栏目',
      componentProps: {
        treeData: categoryTree,
        placeholder: '请选择所属栏目',
        allowClear: true,
        treeDefaultExpandAll: true,
      },
      rules: z
        .any()
        .refine((val) => val !== undefined && val !== null && val !== '', {
          message: '请选择所属栏目',
        }),
    },
    {
      component: 'Input',
      fieldName: 'fieldName',
      label: '字段名',
      componentProps: {
        placeholder: '请输入英文标识（如 author）',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入字段名' }),
    },
    {
      component: 'Input',
      fieldName: 'fieldLabel',
      label: '字段标签',
      componentProps: {
        placeholder: '请输入中文显示名',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入字段标签' }),
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
      label: '选项配置',
      componentProps: {
        placeholder:
          'JSON 数组格式，如 [{"label":"选项A","value":"a"}]',
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
      component: 'Switch',
      fieldName: 'isRequired',
      label: '是否必填',
      defaultValue: false,
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: '排序',
      defaultValue: 0,
      componentProps: {
        min: 0,
        style: 'width: 100%',
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      label: '状态',
      defaultValue: 1,
      componentProps: {
        options: [
          { label: '启用', value: 1 },
          { label: '停用', value: 0 },
        ],
      },
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[50%] max-w-[100vw]',
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
    values.isRequired = values.isRequired ? 1 : 0;

    try {
      if (isCreate.value) {
        await addArticleFieldApi(values as ArticleFieldSaveDTO);
        message.success('新增成功');
      } else {
        await updateArticleFieldApi(
          data.value.row.id,
          values as ArticleFieldSaveDTO,
        );
        message.success('修改成功');
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row || {};
      // 数字 0/1 转布尔给 Switch
      const values: Record<string, any> = { ...row };
      values.isRequired = values.isRequired === 1;
      baseFormApi.setValues(values);
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

async function loadCategoryTree() {
  const result = await categoryApi.tree();
  const mapTree = (nodes: any[]): any[] =>
    nodes.map((node) => ({
      title: node.name,
      value: node.id,
      key: node.id,
      children: node.children ? mapTree(node.children) : undefined,
    }));
  categoryTree.value = mapTree(result);
}

onMounted(() => {
  loadCategoryTree();
});
</script>

<template>
  <Drawer :title="getTitle">
    <BaseForm />
  </Drawer>
</template>

<style>
@media (max-width: 767px) {
  .vben-drawer .ant-drawer-content-wrapper {
    width: 100% !important;
    max-width: 100vw !important;
  }
}
</style>
