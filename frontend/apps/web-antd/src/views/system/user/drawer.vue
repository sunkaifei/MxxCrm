<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { z } from '@vben/common-ui';

import { Button, Drawer, message } from 'ant-design-vue';

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
import { calcMaxProbation } from '#/utils/probation';

const props = defineProps<{
  open: boolean;
  create: boolean;
  row?: any;
}>();

const emit = defineEmits<{
  (e: 'update:open', open: boolean): void;
  (e: 'saved'): void;
}>();

const innerOpen = ref(false);
const loading = ref(false);

watch(
  () => props.open,
  (val) => {
    if (innerOpen.value === val) return;
    innerOpen.value = val;
    if (val) handleOpen();
  },
);
watch(innerOpen, (val) => {
  if (val !== props.open) emit('update:open', val);
});

const isCreate = computed(() => props.create);
const getTitle = computed(() =>
  isCreate.value
    ? $t('ui.modal.create', { moduleName: $t('page.system.user.module') })
    : $t('ui.modal.update', { moduleName: $t('page.system.user.module') }),
);

// 借鉴 UserDetailDrawer（点击姓名打开）：antd Drawer 用 :width 原生 prop 控制宽度，取窗口 75%，上限 1200px
const fullWidth = computed(() => `${Math.min(window.innerWidth * 0.75, 1200)}px`);

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
        // 手机号仅创建时可录入，编辑时锁定，由用户登录后到用户中心自行修改
        disabled: computed(() => !isCreate.value),
      },
      help: () =>
        isCreate.value ? '' : $t('page.system.user.mobileEditLocked'),
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
        // 邮箱仅创建时可录入，编辑时锁定，由用户登录后到用户中心自行修改
        disabled: computed(() => !isCreate.value),
      },
      help: () =>
        isCreate.value ? '' : $t('page.system.user.emailEditLocked'),
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
      component: 'Select',
      fieldName: 'contractType',
      label: $t('page.system.user.contractType'),
      defaultValue: 1,
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        options: [
          { label: '固定期限', value: 1 },
          { label: '无固定期限', value: 2 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'contractMonths',
      label: $t('page.system.user.contractMonths'),
      defaultValue: 36,
      dependencies: {
        triggerFields: ['contractType'],
        if: (values: any) => values?.contractType !== 2,
      },
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        options: [
          { label: '6 个月（试用期上限 1 月）', value: 6 },
          { label: '1 年（试用期上限 2 月）', value: 12 },
          { label: '2 年（试用期上限 2 月）', value: 24 },
          { label: '3 年（试用期上限 6 月）', value: 36 },
          { label: '5 年（试用期上限 6 月）', value: 60 },
        ],
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'probationMonths',
      label: $t('page.system.user.probationMonths'),
      help: $t('page.system.user.probationLegalTip'),
      defaultValue: 2,
      dependencies: {
        triggerFields: ['contractType', 'contractMonths'],
        componentProps: (values: any, { formApi }: any) => {
          const max = calcMaxProbation(values?.contractType, values?.contractMonths);
          return {
            placeholder: max === 0 ? '该合同期限依法不得约定试用期' : $t('ui.placeholder.input'),
            allowClear: true,
            min: 0,
            max,
            precision: 0,
            disabled: max === 0,
            onChange: (v: number | null) => {
              const val = Number(v ?? 0);
              if (val > max) {
                message.warning(`按合同期限，试用期最高 ${max} 个月`);
                formApi.setFieldValue('probationMonths', max);
              }
            },
          };
        },
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
        class: 'w-[60px]',
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
        disabled: computed(() => props.row?.userType === 1),
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

async function handleOpen() {
  if (isCreate.value) {
    baseFormApi.resetForm();
    baseFormApi.setValues({
      create: true,
      status: 1,
      sort: 0,
      gender: 2,
      salaryEnabled: 1,
      bizEnabled: 1,
    });
    return;
  }

  loading.value = true;
  const rowId = props.row?.id;
  try {
    const detail: any = await getUserDetailApi(rowId);
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
  } finally {
    loading.value = false;
  }
}

async function handleSave() {
  const validate = await baseFormApi.validate();
  if (!validate.valid) {
    return;
  }

  loading.value = true;

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
      values.id = props.row.id;
      values.create = undefined;
      values.password = undefined;
      await updateUserApi(values);
    }

    message.success(
      isCreate.value
        ? $t('ui.notification.create_success')
        : $t('ui.notification.update_success'),
    );
    innerOpen.value = false;
    emit('saved');
  } catch {
    // 错误由全局拦截器处理，保留抽屉打开以便用户修改后重试
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <Drawer
    :open="innerOpen"
    :width="fullWidth"
    placement="right"
    :mask-closable="true"
    :closable="true"
    :title="getTitle"
    @close="innerOpen = false"
  >
    <BaseForm />
    <template #footer>
      <div class="flex items-center justify-end gap-2">
        <Button @click="innerOpen = false">
          {{ $t('ui.button.cancel') }}
        </Button>
        <Button type="primary" :loading="loading" @click="handleSave">
          {{ $t('ui.button.ok') }}
        </Button>
      </div>
    </template>
  </Drawer>
</template>
