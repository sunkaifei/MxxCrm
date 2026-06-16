<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { createDictApi, updateDictApi } from '#/api';
import { statusList } from '#/store';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value
    ? $t('ui.modal.create', { moduleName: $t('page.system.dict.module') })
    : $t('ui.modal.update', { moduleName: $t('page.system.dict.module') }),
);

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
      fieldName: 'dictName',
      label: $t('page.system.dict.dictName'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Input',
      fieldName: 'dictType',
      label: $t('page.system.dict.dictType'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Input',
      fieldName: 'itemKey',
      label: $t('page.system.dict.itemKey'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Input',
      fieldName: 'itemValue',
      label: $t('page.system.dict.itemValue'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Input',
      fieldName: 'remark',
      label: $t('ui.table.remark'),
      componentProps: {
        type: 'textarea',
        autosize: true,
        rows: 5,
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: $t('ui.table.sortId'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      defaultValue: 1,
      label: $t('ui.table.status'),
      rules: 'selectRequired',
      componentProps: {
        optionType: 'button',
        class: 'flex flex-wrap',
        options: statusList,
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
      await (data.value?.create
        ? createDictApi(values)
        : updateDictApi(data.value.row.id, values));

      window.$message.success(
        data.value?.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
      drawerApi.setData({ needRefresh: true });
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();

      if (data.value?.row?.meta && data.value?.row?.meta?.authority) {
        const authority = data.value.row.meta.authority;
        data.value.row.meta.authority = authority.join(',');
      }

      baseFormApi.setValues(data.value?.row);

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
