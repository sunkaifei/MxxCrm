<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { shopApi } from '#/api';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增店铺' : '编辑店铺'));

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
      fieldName: 'shopName',
      label: '店铺名称',
      componentProps: {
        placeholder: '请输入店铺名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入店铺名称' }),
    },
    {
      component: 'Input',
      fieldName: 'contactName',
      label: '联系人',
      componentProps: {
        placeholder: '请输入联系人',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'contactPhone',
      label: '联系电话',
      componentProps: {
        placeholder: '请输入联系电话',
        allowClear: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'commissionRate',
      label: '佣金比例(%)',
      defaultValue: 5,
      componentProps: {
        min: 0,
        max: 100,
      },
    },
    {
      component: 'Input',
      fieldName: 'shopDesc',
      label: '店铺简介',
      componentProps: {
        type: 'textarea',
        autosize: true,
        rows: 3,
        placeholder: '请输入店铺简介',
        allowClear: true,
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

    try {
      await shopApi.update({ ...values, id: data.value.row.id });
      message.success('更新成功');
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
