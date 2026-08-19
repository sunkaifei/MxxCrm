<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer, z } from '@vben/common-ui';

import { message } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import {
  createUserApi,
  getAdminOptionsApi,
  getDeptTreeApi,
  getPostOptionsApi,
  getRoleOptionsApi,
  getUserDetailApi,
  updateUserApi,
} from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value
    ? $t('ui.modal.create', { moduleName: $t('page.system.user.module') })
    : $t('ui.modal.update', { moduleName: $t('page.system.user.module') }),
);

const genderOptions = computed(() => [
  { label: $t('page.system.user.genderMale'), value: 0 },
  { label: $t('page.system.user.genderFemale'), value: 1 },
  { label: $t('page.system.user.genderUnknown'), value: 2 },
]);

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Divider',
      fieldName: '_div1',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({
        default: () => $t('page.system.user.basicInfo'),
      }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'userName',
      label: $t('page.system.user.username'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z
        .string()
        .min(1, { message: $t('ui.formRules.required') })
        .regex(/^[a-zA-Z0-9_]+$/, {
          message: '用户名只能包含字母、数字和下划线，不能包含空格',
        }),
    },
    {
      component: 'Input',
      fieldName: 'nickName',
      label: $t('page.system.user.nickName'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'VbenInputPassword',
      fieldName: 'password',
      label: $t('ui.table.password'),
      dependencies: {
        triggerFields: ['_div1'],
        if: (_values, { formApi }: any) => !!formApi.getValues()?.create,
        componentProps: (_values, { formApi }: any) => {
          if (formApi.getValues()?.create) {
            return {
              passwordStrength: true,
              placeholder: $t('ui.placeholder.input'),
            };
          }
          return {};
        },
      },
      help: () =>
        isCreate.value ? $t('page.system.user.defaultPasswordTip') : '',
    },
    {
      component: 'Select',
      fieldName: 'gender',
      label: $t('page.system.user.gender'),
      defaultValue: 2,
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: genderOptions,
      },
    },
    {
      component: 'Input',
      fieldName: 'mobile',
      label: $t('page.system.user.mobile'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
        maxlength: 11,
      },
      rules: z
        .string()
        .min(1, { message: $t('ui.formRules.required') })
        .regex(/^1[3-9]\d{9}$/, {
          message: $t('page.system.user.mobileFormatError'),
        }),
    },
    {
      component: 'Input',
      fieldName: 'email',
      label: $t('page.system.user.email'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'hireDate',
      label: $t('page.system.user.hireDate'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        valueFormat: 'YYYY-MM-DD',
        allowClear: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'probationMonths',
      label: $t('page.system.user.probationMonths'),
      help: $t('page.system.user.probationMonthsTip'),
      defaultValue: 0,
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
        min: 0,
        max: 24,
        precision: 0,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'probationRatio',
      label: $t('page.system.user.probationRatio'),
      help: $t('page.system.user.probationRatioTip'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
        min: 0,
        max: 1,
        step: 0.05,
        precision: 2,
      },
    },
    {
      component: 'Switch',
      fieldName: 'salaryEnabled',
      label: $t('page.system.user.salaryEnabled'),
      help: $t('page.system.user.salaryEnabledTip'),
      defaultValue: 1,
      componentProps: {
        checkedValue: 1,
        unCheckedValue: 0,
        checkedChildren: $t('page.system.user.salaryYes'),
        unCheckedChildren: $t('page.system.user.salaryNo'),
      },
      rules: 'selectRequired',
    },
    {
      component: 'Divider',
      fieldName: '_div2',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({
        default: () => $t('page.system.user.deptRoleInfo'),
      }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'ApiTreeSelect',
      fieldName: 'deptIds',
      label: $t('page.system.user.dept'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        treeCheckable: true,
        showCheckedStrategy: 'SHOW_PARENT',
        treeDefaultExpandAll: true,
        api: async () => {
          return await getDeptTreeApi();
        },
      },
      formItemClass: 'col-span-2',
      rules: z
        .array(z.any(), { required_error: $t('ui.formRules.required') })
        .min(1, {
          message: $t('ui.formRules.selectAtLeastOne', {
            name: $t('page.system.user.dept'),
          }),
        }),
    },
    {
      component: 'ApiSelect',
      fieldName: 'roleIds',
      label: $t('page.system.user.role'),
      componentProps: {
        mode: 'multiple',
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        showSearch: true,
        optionFilterProp: 'label',
        api: async () => {
          return await getRoleOptionsApi();
        },
      },
      rules: z
        .array(z.any(), { required_error: $t('ui.formRules.required') })
        .min(1, {
          message: $t('ui.formRules.selectAtLeastOne', {
            name: $t('page.system.user.role'),
          }),
        }),
    },
    {
      component: 'ApiSelect',
      fieldName: 'postIds',
      label: $t('page.system.user.post'),
      componentProps: {
        mode: 'multiple',
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        showSearch: true,
        optionFilterProp: 'label',
        api: async () => {
          return await getPostOptionsApi();
        },
      },
      rules: z
        .array(z.any(), { required_error: $t('ui.formRules.required') })
        .min(1, {
          message: $t('ui.formRules.selectAtLeastOne', {
            name: $t('page.system.user.post'),
          }),
        }),
    },
    {
      component: 'ApiSelect',
      fieldName: 'directManagerId',
      label: $t('page.system.user.directManager'),
      help: $t('page.system.user.directManagerTip'),
      componentProps: {
        placeholder: $t('page.system.user.directManagerPlaceholder'),
        allowClear: true,
        showSearch: true,
        optionFilterProp: 'label',
        api: async () => {
          return await getAdminOptionsApi();
        },
      },
    },
    {
      component: 'Divider',
      fieldName: '_div3',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({
        default: () => $t('page.system.user.statusInfo'),
      }),
      formItemClass: 'col-span-2',
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
        disabled: computed(() => data.value?.row?.userType === 1),
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
      component: 'Textarea',
      fieldName: 'remark',
      label: $t('ui.table.remark'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
        rows: 3,
        showCount: true,
        maxlength: 200,
      },
      formItemClass: 'col-span-2',
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

    // 直属上级：清空时显式传 0，后端据此清除 direct_manager_id
    // 否则 undefined 会被 JSON 序列化时丢弃，导致后端无法区分"不更新"和"清空"
    if (
      values.directManagerId === null ||
      values.directManagerId === undefined ||
      values.directManagerId === ''
    ) {
      values.directManagerId = 0;
    }

    try {
      if (isCreate.value) {
        values.create = undefined;
        if (!values.password) {
          values.password = '123456';
        }
        await createUserApi(values);
      } else {
        values.id = data.value.row.id;
        values.create = undefined;
        values.password = undefined;
        await updateUserApi(values);
      }

      message.success(
        isCreate.value
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

      if (data.value?.create) {
        baseFormApi.resetForm();
        baseFormApi.setValues({
          create: true,
          status: 1,
          sort: 0,
          gender: 2,
        });
        setLoading(false);
      } else {
        setLoading(true);
        const rowId = data.value?.row?.id;
        getUserDetailApi(rowId)
          .then((detail: any) => {
            const row = { ...detail, create: false };
            if (Array.isArray(row.roleIds)) {
              row.roleIds = row.roleIds
                .filter((v: any) => v !== null && v !== undefined)
                .map(String);
            }
            if (Array.isArray(row.deptIds)) {
              row.deptIds = row.deptIds
                .filter((v: any) => v !== null && v !== undefined)
                .map(String);
            }
            if (Array.isArray(row.postIds)) {
              row.postIds = row.postIds
                .filter((v: any) => v !== null && v !== undefined)
                .map(String);
            }
            baseFormApi.setValues(row);
          })
          .finally(() => {
            setLoading(false);
          });
      }
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
