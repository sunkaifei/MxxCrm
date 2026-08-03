<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { articleTagApi } from '#/api/core/website/article-tag';
import type { ArticleTagSaveDTO } from '#/api/core/website/article-tag';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value ? '新增文章标签' : '编辑文章标签',
);

const colorOptions = [
  { label: '默认', value: '' },
  { label: '红色', value: 'red' },
  { label: '橙色', value: 'orange' },
  { label: '黄色', value: 'gold' },
  { label: '绿色', value: 'green' },
  { label: '蓝色', value: 'blue' },
  { label: '紫色', value: 'purple' },
  { label: '青色', value: 'cyan' },
  { label: '粉色', value: 'pink' },
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
      label: '标签名称',
      componentProps: {
        placeholder: '请输入标签名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入标签名称' }),
    },
    {
      component: 'Input',
      fieldName: 'slug',
      label: '别名',
      componentProps: {
        placeholder: '请输入标签别名（slug）',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'color',
      label: '颜色',
      defaultValue: '',
      componentProps: {
        options: colorOptions,
        placeholder: '请选择标签颜色',
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
    const payload: ArticleTagSaveDTO = {
      name: values.name,
      slug: values.slug,
      color: values.color,
      sort: values.sort,
      status: values.status,
    };

    try {
      if (isCreate.value) {
        await articleTagApi.add(payload);
        message.success('新增成功');
      } else {
        await articleTagApi.update(data.value.row.id, payload);
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
        const detail: any = await articleTagApi.detail(data.value.row.id);
        const row = detail?.data || detail || data.value.row;
        baseFormApi.setValues({
          name: row.name,
          slug: row.slug,
          color: row.color || '',
          sort: row.sort ?? 0,
          status: row.status ?? 1,
        });
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