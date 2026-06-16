<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { createApiApi, getApiListApi, updateApiApi } from '#/api';
import { methodList } from '#/store';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value
    ? $t('ui.modal.create', { moduleName: $t('page.system.api.module') })
    : $t('ui.modal.update', { moduleName: $t('page.system.api.module') }),
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
      component: 'ApiSelect',
      fieldName: 'parentId',
      label: $t('page.system.api.parentId'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        api: async () => {
          const result = await getApiListApi({
            onlyParent: true,
          });
          return result.items;
        },
        labelField: 'description',
        valueField: 'id',
      },
      rules: 'selectRequired',
    },
    {
      component: 'Input',
      fieldName: 'description',
      label: $t('page.system.api.description'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Select',
      fieldName: 'method',
      label: $t('page.system.api.method'),
      componentProps: {
        options: methodList,
        placeholder: $t('ui.placeholder.select'),
      },
      dependencies: {
        required(values) {
          return values.parentId !== 0;
        },
        triggerFields: ['parentId'],
      },
    },
    {
      component: 'Input',
      fieldName: 'path',
      label: $t('page.system.api.path'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      dependencies: {
        required(values) {
          return values.parentId !== 0;
        },
        triggerFields: ['parentId'],
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
        ? createApiApi(values)
        : updateApiApi(data.value.row.id, values));

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
