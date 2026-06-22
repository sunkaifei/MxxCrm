<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { Divider, message } from 'ant-design-vue';
import { createCustomerApi, updateCustomerApi, getCountriesApi } from '#/api';
import TagSelector from '../components/TagSelector.vue';

const data = ref();
const tagSelectorRef = ref<InstanceType<typeof TagSelector>>();

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.customer.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.customer.title') }),
);

const isCreate = computed(() => data.value?.create);

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
      fieldName: 'companyName',
      label: '公司名称',
      rules: 'required',
      componentProps: { placeholder: '请输入公司名称', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'shortName',
      label: '简称',
      componentProps: { placeholder: '请输入公司简称', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'level',
      label: '客户等级',
      componentProps: {
        placeholder: '请选择客户等级',
        allowClear: true,
        options: [
          { label: '无级别', value: 1 },
          { label: '重点客户', value: 2 },
          { label: '优质客户', value: 3 },
          { label: '普通客户', value: 4 },
          { label: '其他', value: 5 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'source',
      label: '客户来源',
      componentProps: {
        placeholder: '请选择来源',
        allowClear: true,
        options: [
          { label: '展会', value: '展会' },
          { label: '阿里国际站', value: '阿里国际站' },
          { label: '老客户推荐', value: '老客户推荐' },
          { label: '官网', value: '官网' },
          { label: '社交媒体', value: '社交媒体' },
          { label: '其他', value: '其他' },
        ],
      },
    },
    {
      component: 'Input',
      fieldName: 'industry',
      label: '行业',
      componentProps: { placeholder: '如 IT、外贸、制造', allowClear: true },
    },
    // 地区信息
    {
      component: 'Divider',
      fieldName: '_div2',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '地区信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'ApiSelect',
      fieldName: 'country',
      label: '国家',
      componentProps: {
        placeholder: '请选择国家',
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) => {
          return (
            option.label?.toLowerCase().includes(input.toLowerCase()) ||
            option.value?.toLowerCase().includes(input.toLowerCase())
          );
        },
        api: async () => {
          const result = await getCountriesApi();
          const items = Array.isArray(result) ? result : [];
          return items.map((item: any) => ({
            label: item.name,
            value: item.name,
            labelEn: item.nameEn,
          }));
        },
      },
    },
    {
      component: 'Input',
      fieldName: 'region',
      label: '省/州',
      componentProps: { placeholder: '省/州', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'address',
      label: '详细地址',
      formItemClass: 'col-span-2',
      componentProps: { placeholder: '详细地址', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'website',
      label: '网站',
      componentProps: { placeholder: 'https://', allowClear: true },
    },
    // 财务信息
    {
      component: 'Divider',
      fieldName: '_div3',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '财务信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Select',
      fieldName: 'currency',
      label: '币种',
      defaultValue: 'CNY',
      componentProps: {
        options: [
          { label: 'CNY (人民币)', value: 'CNY' },
          { label: 'USD (美元)', value: 'USD' },
          { label: 'EUR (欧元)', value: 'EUR' },
          { label: 'GBP (英镑)', value: 'GBP' },
          { label: 'JPY (日元)', value: 'JPY' },
        ],
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'creditLimit',
      label: '信用额度',
      componentProps: { placeholder: '信用额度', min: 0, class: 'w-full', addonAfter: '元' },
    },
    {
      component: 'InputNumber',
      fieldName: 'creditDays',
      label: '账期(天)',
      componentProps: { placeholder: '账期', min: 0, class: 'w-full', addonAfter: '天' },
    },
    // 其他信息
    {
      component: 'Divider',
      fieldName: '_div4',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '其他信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'assignedTo',
      label: '负责人',
      componentProps: { placeholder: '负责人ID', allowClear: true },
    },
    {
      component: 'DatePicker',
      fieldName: 'cooperatedAt',
      label: '合作起始日期',
      componentProps: { placeholder: '选择日期', class: 'w-full', allowClear: true },
    },
    {
      component: 'Textarea',
      fieldName: 'description',
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
      const result = await (data.value?.create
        ? createCustomerApi(values)
        : updateCustomerApi({ ...values, id: data.value.row.id }));

      // 创建模式：保存标签到新创建的实体
      if (data.value?.create && tagSelectorRef.value) {
        const newId = result?.id || result?.data?.id;
        if (newId) {
          await tagSelectorRef.value.saveToEntity(newId);
        }
      }

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
  <Drawer :title="getTitle" :width="620">
    <BaseForm />
    <Divider />
    <div class="mb-2 text-sm font-medium text-gray-700">标签</div>
    <TagSelector
      ref="tagSelectorRef"
      entity-type="customer"
      :entity-id="isCreate ? null : data?.row?.id"
    />
  </Drawer>
</template>