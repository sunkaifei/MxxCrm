<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer, z } from '@vben/common-ui';

import { message } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { addPageApi, updatePageApi } from '#/api';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增页面' : '修改页面'));

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
      fieldName: 'pageCode',
      label: '页面编码',
      componentProps: {
        placeholder: '请输入页面编码（唯一标识，如 about-us）',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入页面编码' }),
    },
    {
      component: 'Input',
      fieldName: 'pageName',
      label: '页面名称',
      componentProps: {
        placeholder: '请输入页面名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入页面名称' }),
    },
    {
      component: 'Input',
      fieldName: 'pageTitle',
      label: '页面标题',
      componentProps: {
        placeholder: '请输入页面标题（浏览器标题栏）',
        allowClear: true,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'pageContent',
      label: '页面内容',
      componentProps: {
        placeholder: '请输入页面内容（支持HTML）',
        allowClear: true,
        rows: 10,
      },
    },
    {
      component: 'Input',
      fieldName: 'seoKeywords',
      label: 'SEO关键词',
      componentProps: {
        placeholder: '多个关键词用英文逗号分隔',
        allowClear: true,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'seoDescription',
      label: 'SEO描述',
      componentProps: {
        placeholder: '请输入SEO描述，建议 50-200 字',
        allowClear: true,
        rows: 3,
      },
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
          { label: '禁用', value: 0 },
        ],
      },
    },
  ],
});

// 编辑模式下 pageCode 只读
function setCodeReadonly(readonly: boolean) {
  baseFormApi.updateSchema([
    {
      fieldName: 'pageCode',
      componentProps: { disabled: readonly },
    },
  ]);
}

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[60%] max-w-[100vw]',
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

    try {
      if (isCreate.value) {
        await addPageApi(values);
        message.success('新增成功');
      } else {
        await updatePageApi(data.value.row.id, values);
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
      baseFormApi.setValues(row);
      setCodeReadonly(!isCreate.value);
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
@media (max-width: 767px) {
  .vben-drawer .ant-drawer-content-wrapper {
    width: 100% !important;
    max-width: 100vw !important;
  }
}
</style>
