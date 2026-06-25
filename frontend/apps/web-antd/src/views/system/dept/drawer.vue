<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import {
  createDeptApi,
  getAdminOptionsApi,
  getDeptTreeApi,
  updateDeptApi,
} from '#/api';
import { statusList } from '#/store';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value
    ? $t('ui.modal.create', { moduleName: $t('page.system.dept.title') })
    : $t('ui.modal.update', { moduleName: $t('page.system.dept.title') }),
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
      component: 'ApiTreeSelect',
      fieldName: 'parentId',
      label: $t('page.system.dept.parentId'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        treeDefaultExpandAll: true,
        api: async () => {
          const result = await getDeptTreeApi();
          const list = Array.isArray(result) ? result : [];
          return [
            {
              value: '0',
              label: $t('page.system.dept.topLevel') || '顶级部门',
              children: list,
            },
          ];
        },
      },
    },
    {
      component: 'Input',
      fieldName: 'deptName',
      label: $t('page.system.dept.deptName'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Input',
      fieldName: 'code',
      label: $t('page.system.dept.code'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'ApiSelect',
      fieldName: 'leaderId',
      label: $t('page.system.dept.leader'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        showSearch: true,
        optionFilterProp: 'label',
        api: async () => {
          return await getAdminOptionsApi();
        },
      },
    },
    {
      component: 'Input',
      fieldName: 'phone',
      label: $t('page.system.dept.phone'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'email',
      label: $t('page.system.dept.email'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: $t('ui.table.sortId'),
      defaultValue: 0,
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
        min: 0,
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

    if (values.parentId === null || values.parentId === undefined || values.parentId === '') {
      values.parentId = '0';
    }

    try {
      await (data.value?.create
        ? createDeptApi(values)
        : updateDeptApi(data.value.row.id, values));

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

      if (data.value?.create) {
        baseFormApi.resetForm();
        baseFormApi.setValues({
          parentId: '0',
          status: 1,
          sort: 0,
        });
      } else {
        const row = { ...data.value?.row };
        if (row.parentId === 0 || row.parentId === '0' || !row.parentId) {
          row.parentId = '0';
        }
        baseFormApi.setValues(row);
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
