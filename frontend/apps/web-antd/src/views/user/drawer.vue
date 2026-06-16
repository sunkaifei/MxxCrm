<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer, z } from '@vben/common-ui';

import { useVbenForm } from '#/adapter/form';
import { createUserManageApi, updateUserManageApi } from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value
    ? $t('ui.modal.create', { moduleName: $t('page.user.module') })
    : $t('ui.modal.update', { moduleName: $t('page.user.module') }),
);

type FormValues = Record<string, any>;

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
      fieldName: 'username',
      label: $t('page.user.username'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Input',
      fieldName: 'nickname',
      label: $t('page.user.nickname'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: 'required',
    },
    {
      component: 'VbenInputPassword',
      fieldName: 'password',
      label: $t('ui.table.password'),
      componentProps: {
        passwordStrength: true,
        placeholder: $t('ui.placeholder.input'),
      },
      help: '默认密码: 123456',
    },
    {
      component: 'Input',
      fieldName: 'mobile',
      label: $t('page.user.mobile'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'email',
      label: $t('page.user.email'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'motto',
      label: $t('page.user.motto'),
      componentProps: {
        type: 'textarea',
        autosize: true,
        rows: 3,
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      defaultValue: '1',
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

    const values: FormValues = await baseFormApi.getValues();

    try {
      if (data.value?.create) {
        await createUserManageApi(values as any);
      } else {
        await updateUserManageApi(data.value.row.id, values as any);
      }

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
