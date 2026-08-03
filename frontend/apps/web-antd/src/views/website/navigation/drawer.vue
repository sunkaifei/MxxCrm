<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { navigationApi } from '#/api';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增导航' : '修改导航'));

const dataTypeOptions = [
  { label: '自定义导航', value: 'custom' },
  { label: '文章分类', value: 'article_class' },
  { label: '自定义页面', value: 'customview' },
];

const navTypeOptions = [
  { label: '顶部导航', value: 'header' },
  { label: '底部导航', value: 'footer' },
];

const targetOptions = [
  { label: '当前窗口 (_self)', value: '_self' },
  { label: '新窗口 (_blank)', value: '_blank' },
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
      fieldName: 'name',
      label: '导航名称',
      componentProps: {
        placeholder: '请输入导航名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入导航名称' }),
    },
    {
      component: 'Select',
      fieldName: 'navType',
      label: '导航类型',
      defaultValue: 'header',
      componentProps: {
        options: navTypeOptions,
        placeholder: '请选择导航类型',
      },
      rules: z.string().min(1, { message: '请选择导航类型' }),
    },
    {
      component: 'Select',
      fieldName: 'dataType',
      label: '数据类型',
      defaultValue: 'custom',
      componentProps: {
        options: dataTypeOptions,
        placeholder: '请选择数据类型',
      },
    },
    {
      component: 'Input',
      fieldName: 'webUrl',
      label: '链接地址',
      componentProps: {
        placeholder: '请输入链接地址（含http://）',
        allowClear: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'value',
      label: '数据ID',
      componentProps: {
        min: 0,
        style: 'width: 100%',
        placeholder: '关联数据ID（如分类ID、页面ID）',
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'parentId',
      label: '父级ID',
      componentProps: {
        min: 0,
        style: 'width: 100%',
        placeholder: '父级导航ID，顶级为0或空',
      },
    },
    {
      component: 'Input',
      fieldName: 'icon',
      label: '图标',
      componentProps: {
        placeholder: '请输入图标类名或URL',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'target',
      label: '打开方式',
      defaultValue: '_self',
      componentProps: {
        options: targetOptions,
        placeholder: '请选择打开方式',
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
      fieldName: 'isShow',
      label: '是否显示',
      defaultValue: 1,
      componentProps: {
        options: [
          { label: '显示', value: 1 },
          { label: '隐藏', value: 0 },
        ],
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'isNewWindowOpen',
      label: '新窗口打开',
      defaultValue: 0,
      componentProps: {
        options: [
          { label: '是', value: 1 },
          { label: '否', value: 0 },
        ],
      },
    },
  ],
});

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
        await navigationApi.add(values);
        message.success('新增成功');
      } else {
        await navigationApi.update(data.value.row.id, values);
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
