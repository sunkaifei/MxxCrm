<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, watch } from 'vue';

import { useVbenForm } from '@vben/common-ui';

import { Button, Divider, InputNumber, message } from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import { createQuotationApi, updateQuotationApi } from '#/api';

const props = withDefaults(
  defineProps<{ create?: boolean; row?: any }>(),
  { create: true, row: () => ({}) },
);

const isEdit = computed(() => !props.create);

const currencyOptions = [
  { label: 'CNY', value: 'CNY' },
  { label: 'USD', value: 'USD' },
  { label: 'EUR', value: 'EUR' },
  { label: 'GBP', value: 'GBP' },
  { label: 'JPY', value: 'JPY' },
  { label: 'HKD', value: 'HKD' },
  { label: 'AUD', value: 'AUD' },
];

const statusOptions = [
  { label: '草稿', value: 'draft' },
  { label: '已发送', value: 'sent' },
];

const formSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'title',
    label: '报价单标题',
    rules: 'required',
    componentProps: { placeholder: '如：ABC集团年度续约报价' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'customerId',
    label: '客户ID',
    componentProps: { placeholder: '请输入客户ID', type: 'number' },
  },
  {
    component: 'Input',
    fieldName: 'contactId',
    label: '联系人ID',
    componentProps: { placeholder: '请输入联系人ID', type: 'number' },
  },
  {
    component: 'Input',
    fieldName: 'opportunityId',
    label: '商机ID',
    componentProps: { placeholder: '请输入商机ID', type: 'number' },
  },
  {
    component: 'Select',
    fieldName: 'status',
    label: '状态',
    componentProps: { placeholder: '请选择', options: statusOptions },
  },
  {
    component: 'Divider',
    fieldName: 'divider_amount',
    label: '',
    componentProps: { plain: true, orientation: 'left' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'InputNumber',
    fieldName: 'grandTotal',
    label: '金额',
    componentProps: { placeholder: '请输入金额', style: 'width:100%', precision: 2 },
  },
  {
    component: 'Select',
    fieldName: 'currency',
    label: '币种',
    componentProps: { placeholder: '请选择', options: currencyOptions },
  },
  {
    component: 'InputNumber',
    fieldName: 'taxAmount',
    label: '税额',
    componentProps: { placeholder: '请输入税额', style: 'width:100%', precision: 2 },
  },
  {
    component: 'InputNumber',
    fieldName: 'discountAmount',
    label: '折扣金额',
    componentProps: { placeholder: '请输入折扣', style: 'width:100%', precision: 2 },
  },
  {
    component: 'DatePicker',
    fieldName: 'validUntil',
    label: '有效期',
    componentProps: { placeholder: '请选择', style: 'width:100%' },
  },
  {
    component: 'Input',
    fieldName: 'assignedTo',
    label: '负责人ID',
    componentProps: { placeholder: '请输入负责人ID', type: 'number' },
  },
  {
    component: 'Divider',
    fieldName: 'divider_delivery',
    label: '',
    componentProps: { plain: true, orientation: 'left' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'paymentTerms',
    label: '付款条件',
    componentProps: { placeholder: '如：T/T 30% advance, 70% before shipment' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'deliveryTerms',
    label: '交货条款',
    componentProps: { placeholder: '如：FOB Shenzhen' },
  },
  {
    component: 'DatePicker',
    fieldName: 'deliveryDate',
    label: '交货日期',
    componentProps: { placeholder: '请选择', style: 'width:100%' },
  },
  {
    component: 'Input',
    fieldName: 'portOfLoading',
    label: '装运港',
    componentProps: { placeholder: '如：深圳' },
  },
  {
    component: 'Input',
    fieldName: 'portOfDestination',
    label: '目的港',
    componentProps: { placeholder: '如：洛杉矶' },
  },
  {
    component: 'Divider',
    fieldName: 'divider_bank',
    label: '',
    componentProps: { plain: true, orientation: 'left' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Textarea',
    fieldName: 'bankInfo',
    label: '银行信息',
    componentProps: { placeholder: 'Beneficiary / Account No / SWIFT / Bank', rows: 3 },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Textarea',
    fieldName: 'notes',
    label: '备注',
    componentProps: { placeholder: '价格有效期30天，过期自动作废', rows: 2, showCount: true, maxlength: 500 },
    wrapperClass: 'col-span-2',
  },
];

const [Form, formApi] = useVbenForm({
  schema: formSchema,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
  actionWrapperClass: 'col-span-2',
});

watch(
  () => props.row,
  (val) => {
    if (val && !props.create) {
      formApi.setValues(val);
    }
  },
  { immediate: true },
);

async function handleSubmit() {
  const { valid, values } = await formApi.validate();
  if (!valid) return;
  try {
    const data = { ...values };
    if (isEdit.value) {
      await updateQuotationApi({ ...data, id: props.row.id });
      message.success('更新成功');
    } else {
      await createQuotationApi(data);
      message.success('创建成功');
    }
    closeDrawer();
  } catch {
    message.error('操作失败');
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onOpenChange(isOpen) {
    if (isOpen) {
      formApi.resetForm();
      if (!props.create && props.row) {
        formApi.setValues(props.row);
      }
    }
  },
});

function closeDrawer() {
  drawerApi.close();
  drawerApi.setData({ needRefresh: true });
}
</script>

<template>
  <Drawer
    :title="isEdit ? '编辑报价单' : '新建报价单'"
    :width="620"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <Form />
    <div class="flex justify-end gap-2 mt-4">
      <Button @click="closeDrawer">取消</Button>
      <Button type="primary" @click="handleSubmit">保存</Button>
    </div>
  </Drawer>
</template>