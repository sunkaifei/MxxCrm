<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { categoryApi } from '#/api';
import { message } from 'ant-design-vue';
import type { CategorySaveDTO } from '#/api/core/shop/category';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增分类' : '编辑分类'));

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
      label: '分类名称',
      componentProps: {
        placeholder: '请输入分类名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入分类名称' }),
    },
    {
      component: 'Input',
      fieldName: 'icon',
      label: '分类图标',
      componentProps: {
        placeholder: '请输入图标URL',
        allowClear: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sortOrder',
      label: '排序值',
      defaultValue: 0,
      componentProps: {
        min: 0,
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'isShow',
      label: '是否显示',
      defaultValue: 1,
      componentProps: {
        optionType: 'button',
        class: 'flex flex-wrap',
        options: [
          { label: '显示', value: 1 },
          { label: '隐藏', value: 0 },
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
    const parentId = data.value?.parentId ?? 0;
    values.parentId = parentId;

    try {
      if (data.value?.create) {
        await categoryApi.save(values as CategorySaveDTO);
      } else {
        await categoryApi.update({
          ...values,
          id: data.value.row.id,
        } as CategorySaveDTO);
      }

      message.success(data.value?.create ? '新增成功' : '更新成功');
      drawerApi.setData({ needRefresh: true });
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      baseFormApi.setValues(data.value?.row || {});
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
