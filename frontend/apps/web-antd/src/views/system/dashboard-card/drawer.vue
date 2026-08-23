<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer, z } from '@vben/common-ui';

import { message } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import {
  createDashboardCardApi,
  updateDashboardCardApi,
} from '#/api/core/system/dashboard-card';
import { $t } from '#/locales';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value
    ? $t('ui.modal.create', {
        moduleName: $t('page.system.dashboardCard.module'),
      })
    : $t('ui.modal.update', {
        moduleName: $t('page.system.dashboardCard.module'),
      }),
);

const formSchema: any[] = [
  {
    component: 'Input',
    fieldName: 'cardCode',
    label: $t('page.system.dashboardCard.cardCode'),
    componentProps: {
      placeholder: $t('ui.placeholder.input'),
      disabled: !isCreate.value,
    },
    rules: z.string().min(1, { message: $t('ui.formRules.required') }),
  },
  {
    component: 'Input',
    fieldName: 'cardName',
    label: $t('page.system.dashboardCard.cardName'),
    componentProps: {
      placeholder: $t('ui.placeholder.input'),
    },
    rules: z.string().min(1, { message: $t('ui.formRules.required') }),
  },
  {
    component: 'Input',
    fieldName: 'pageKey',
    label: $t('page.system.dashboardCard.pageKey'),
    componentProps: {
      placeholder: $t('page.system.dashboardCard.pageKeyPlaceholder'),
    },
    rules: z.string().min(1, { message: $t('ui.formRules.required') }),
  },
  {
    component: 'InputNumber',
    fieldName: 'sortOrder',
    label: $t('page.system.dashboardCard.sortOrder'),
    defaultValue: 1,
    componentProps: {
      min: 0,
      max: 9999,
      precision: 0,
      class: 'w-full',
    },
  },
  {
    component: 'Switch',
    fieldName: 'status',
    label: $t('page.system.dashboardCard.status'),
    defaultValue: 1,
    componentProps: {
      checkedValue: 1,
      unCheckedValue: 0,
      checkedChildren: $t('ui.switch.active'),
      unCheckedChildren: $t('ui.switch.inactive'),
    },
  },
  {
    component: 'Input',
    fieldName: 'remark',
    label: $t('ui.form.description'),
    componentProps: {
      type: 'textarea',
      rows: 3,
      placeholder: $t('ui.placeholder.input'),
    },
  },
];

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: formSchema,
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
        ? createDashboardCardApi(values)
        : updateDashboardCardApi({ id: data.value.row.id, ...values }));

      message.success(
        data.value?.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } catch {
      // 错误由全局拦截器处理，保留抽屉打开以便用户修改后重试
    } finally {
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
