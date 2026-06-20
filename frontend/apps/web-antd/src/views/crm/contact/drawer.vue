<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { createContactApi, updateContactApi } from '#/api';

const data = ref();

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.contact.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.contact.title') }),
);

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: { class: 'w-full' },
  },
  schema: [
    // 基本信息
    {
      component: 'Divider',
      fieldName: '_div1',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '基本信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'name',
      label: '姓名',
      rules: 'required',
      componentProps: { placeholder: '请输入姓名', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'title',
      label: '职位',
      componentProps: { placeholder: '如 采购经理', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'roleType',
      label: '角色',
      componentProps: {
        placeholder: '选择角色',
        allowClear: true,
        options: [
          { label: '决策人', value: 'decision_maker' },
          { label: '影响者', value: 'influencer' },
          { label: '使用者', value: 'user' },
          { label: '其他', value: 'other' },
        ],
      },
    },
    // 联系方式
    {
      component: 'Divider',
      fieldName: '_div2',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '联系方式' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'email',
      label: '邮箱',
      componentProps: { placeholder: 'email@example.com', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'mobile',
      label: '手机号',
      componentProps: { placeholder: '手机号', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'phone',
      label: '座机',
      componentProps: { placeholder: '座机号码', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'whatsapp',
      label: 'WhatsApp',
      componentProps: { placeholder: 'WhatsApp 号码', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'wechat',
      label: '微信',
      componentProps: { placeholder: '微信号', allowClear: true },
    },
    // 其他信息
    {
      component: 'Divider',
      fieldName: '_div3',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '其他信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'DatePicker',
      fieldName: 'birthday',
      label: '生日',
      componentProps: { placeholder: '选择日期', class: 'w-full', allowClear: true },
    },
    {
      component: 'Textarea',
      fieldName: 'notes',
      label: '备注',
      formItemClass: 'col-span-2',
      componentProps: { placeholder: '备注信息', rows: 3, allowClear: true },
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() { drawerApi.close(); },
  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) return;
    setLoading(true);
    const values = await baseFormApi.getValues();
    try {
      await (data.value?.create
        ? createContactApi(values)
        : updateContactApi({ ...values, id: data.value.row.id }));
      message.success(data.value?.create
        ? $t('ui.notification.create_success')
        : $t('ui.notification.update_success'));
      drawerApi.setData({ needRefresh: true });
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },
  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};
      baseFormApi.setValues(row);
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle" :width="580">
    <BaseForm />
  </Drawer>
</template>