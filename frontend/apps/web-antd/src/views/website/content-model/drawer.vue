<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import {
  addContentModelApi,
  getContentModelDetailApi,
  updateContentModelApi,
} from '#/api';
import type { ContentModelSaveDTO } from '#/api/core/website/content-model';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value ? '新增内容模型' : '编辑内容模型',
);

const switchFields = [
  'hasTitle',
  'hasContent',
  'hasCover',
  'hasAuthor',
  'hasSummary',
  'hasSeo',
  'hasImages',
  'hasAttachment',
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
      fieldName: 'modelCode',
      label: '模型编码',
      componentProps: () => ({
        placeholder: '请输入模型编码（如 article）',
        disabled: !isCreate.value,
      }),
      rules: z.string().min(1, { message: '请输入模型编码' }),
    },
    {
      component: 'Input',
      fieldName: 'modelName',
      label: '模型名称',
      componentProps: {
        placeholder: '请输入模型名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入模型名称' }),
    },
    {
      component: 'Input',
      fieldName: 'modelIcon',
      label: '模型图标',
      componentProps: {
        placeholder: '请输入图标名（如 ant-design:file-outlined）',
        allowClear: true,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'description',
      label: '描述',
      componentProps: {
        placeholder: '请输入描述',
        allowClear: true,
        rows: 3,
      },
    },
    {
      component: 'Switch',
      fieldName: 'hasTitle',
      label: '支持标题',
      defaultValue: true,
    },
    {
      component: 'Switch',
      fieldName: 'hasContent',
      label: '支持正文',
      defaultValue: true,
    },
    {
      component: 'Switch',
      fieldName: 'hasCover',
      label: '支持封面',
      defaultValue: false,
    },
    {
      component: 'Switch',
      fieldName: 'hasAuthor',
      label: '支持作者',
      defaultValue: false,
    },
    {
      component: 'Switch',
      fieldName: 'hasSummary',
      label: '支持摘要',
      defaultValue: false,
    },
    {
      component: 'Switch',
      fieldName: 'hasSeo',
      label: '支持SEO',
      defaultValue: false,
    },
    {
      component: 'Switch',
      fieldName: 'hasImages',
      label: '支持图集',
      defaultValue: false,
    },
    {
      component: 'Switch',
      fieldName: 'hasAttachment',
      label: '支持附件',
      defaultValue: false,
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

    try {
      if (isCreate.value) {
        await addContentModelApi(values as ContentModelSaveDTO);
        message.success('新增成功');
      } else {
        await updateContentModelApi(
          data.value.row.id,
          values as ContentModelSaveDTO,
        );
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
        const detail: any = await getContentModelDetailApi(data.value.row.id);
        const row = detail?.data || detail || data.value.row;
        // 数字 0/1 转布尔给 Switch
        const values: Record<string, any> = { ...row };
        switchFields.forEach((key) => {
          values[key] = values[key] === 1;
        });
        baseFormApi.setValues(values);
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
