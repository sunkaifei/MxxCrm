<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref, watch } from 'vue';

import { useVbenForm } from '@vben/common-ui';

import { Button, Tooltip, message } from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import { createInvoiceApi, getInvoiceInfoApi, updateInvoiceApi } from '#/api';

const props = withDefaults(
  defineProps<{ create?: boolean; row?: any }>(),
  { create: true, row: () => ({}) },
);

const isEdit = computed(() => !props.create);
const isFullscreen = ref(false);

const drawerClass = computed(() => [
  'sale-invoice-drawer',
  { 'sale-invoice-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const invoiceTypeOptions = [
  { label: '增值税专用发票', value: 1 },
  { label: '增值税普通发票', value: 2 },
  { label: '形式发票(PI)', value: 3 },
  { label: '商业发票(CI)', value: 4 },
];

const currencyOptions = [
  { label: 'CNY 人民币', value: 1 },
  { label: 'USD 美元', value: 2 },
  { label: 'EUR 欧元', value: 3 },
  { label: 'GBP 英镑', value: 4 },
  { label: 'JPY 日元', value: 5 },
  { label: 'HKD 港币', value: 6 },
];

const formSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'title',
    label: '发票标题',
    rules: 'required',
    componentProps: { placeholder: '请输入发票标题' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'invoiceType',
    label: '发票类型',
    defaultValue: 1,
    componentProps: { placeholder: '请选择', options: invoiceTypeOptions },
  },
  {
    component: 'DatePicker',
    fieldName: 'invoiceDate',
    label: '开票日期',
    rules: 'required',
    componentProps: { placeholder: '请选择', style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
  },
  {
    component: 'InputNumber',
    fieldName: 'contractId',
    label: '合同ID',
    componentProps: { placeholder: '请输入合同ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'InputNumber',
    fieldName: 'orderId',
    label: '订单ID',
    componentProps: { placeholder: '请输入订单ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'InputNumber',
    fieldName: 'customerId',
    label: '客户ID',
    rules: 'required',
    componentProps: { placeholder: '请输入客户ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'Input',
    fieldName: 'customerName',
    label: '客户名称',
    componentProps: { placeholder: '请输入客户名称' },
  },
  {
    component: 'DatePicker',
    fieldName: 'dueDate',
    label: '到期日',
    componentProps: { placeholder: '请选择', style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
  },
  {
    component: 'InputNumber',
    fieldName: 'amount',
    label: '金额',
    rules: 'required',
    componentProps: { placeholder: '请输入金额', style: 'width:100%', precision: 2, min: 0 },
  },
  {
    component: 'InputNumber',
    fieldName: 'taxRate',
    label: '税率',
    componentProps: {
      placeholder: '请输入税率',
      style: 'width:100%',
      precision: 2,
      min: 0,
      max: 100,
      addonAfter: '%',
    },
  },
  {
    component: 'InputNumber',
    fieldName: 'taxAmount',
    label: '税额',
    componentProps: {
      placeholder: '自动计算',
      style: 'width:100%',
      precision: 2,
      min: 0,
    },
  },
  {
    component: 'Select',
    fieldName: 'currency',
    label: '币种',
    defaultValue: 1,
    componentProps: { placeholder: '请选择', options: currencyOptions },
  },
  {
    component: 'Input',
    fieldName: 'buyerName',
    label: '购买方名称',
    componentProps: { placeholder: '请输入购买方名称' },
  },
  {
    component: 'Input',
    fieldName: 'buyerTaxNo',
    label: '购买方税号',
    componentProps: { placeholder: '请输入购买方税号' },
  },
  {
    component: 'Input',
    fieldName: 'buyerBank',
    label: '开户行',
    componentProps: { placeholder: '请输入开户行' },
  },
  {
    component: 'Input',
    fieldName: 'taxNo',
    label: '销方税号',
    componentProps: { placeholder: '请输入销方税号' },
  },
  {
    component: 'Textarea',
    fieldName: 'buyerAddress',
    label: '购买方地址',
    componentProps: { placeholder: '请输入购买方地址、电话', rows: 2 },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: { placeholder: '备注信息', rows: 3, showCount: true, maxlength: 500 },
    wrapperClass: 'col-span-2',
  },
];

const [Form, formApi] = useVbenForm({
  schema: formSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

watch(
  () => props.row,
  async (val) => {
    if (val && !props.create) {
      try {
        const info = await getInvoiceInfoApi(val.id);
        const data = info || val;
        formApi.setValues({
          title: data.title,
          invoiceType: data.invoiceType ?? 1,
          invoiceDate: data.invoiceDate,
          contractId: data.contractId,
          orderId: data.orderId,
          customerId: data.customerId,
          customerName: data.customerName,
          dueDate: data.dueDate,
          amount: data.amount ?? 0,
          taxRate: data.taxRate ?? 0,
          taxAmount: data.taxAmount ?? 0,
          currency: data.currency ?? 1,
          taxNo: data.taxNo,
          buyerName: data.buyerName,
          buyerTaxNo: data.buyerTaxNo,
          buyerBank: data.buyerBank,
          buyerAddress: data.buyerAddress,
          remark: data.remark,
        });
      } catch {
        formApi.setValues(val);
      }
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
      await updateInvoiceApi({ ...data, id: props.row.id });
      message.success('更新成功');
    } else {
      await createInvoiceApi(data);
      message.success('创建成功');
    }
    drawerApi.setData({ needRefresh: true });
    drawerApi.close();
  } catch {
    message.error('操作失败');
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    await handleSubmit();
  },
  onOpenChange(isOpen) {
    if (isOpen) {
      isFullscreen.value = false;
      formApi.resetForm();
      if (!props.create && props.row) {
        formApi.setValues({
          invoiceType: 1,
          currency: 1,
          ...props.row,
        });
      } else {
        formApi.setValues({
          invoiceType: 1,
          currency: 1,
          invoiceDate: new Date().toISOString().slice(0, 10),
        });
      }
    }
  },
});
</script>

<template>
  <Drawer
    :title="isEdit ? '编辑发票' : '新建发票'"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '退出全屏' : '全屏'">
        <button
          type="button"
          class="sale-invoice-drawer__fs-btn"
          @click="toggleFullscreen"
        >
          <svg
            v-if="!isFullscreen"
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M8 3H5a2 2 0 0 0-2 2v3" />
            <path d="M21 8V5a2 2 0 0 0-2-2h-3" />
            <path d="M3 16v3a2 2 0 0 0 2 2h3" />
            <path d="M16 21h3a2 2 0 0 0 2-2v-3" />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M8 3v3a2 2 0 0 1-2 2H3" />
            <path d="M21 8h-3a2 2 0 0 1-2-2V3" />
            <path d="M3 16h3a2 2 0 0 1 2 2v3" />
            <path d="M16 21v-3a2 2 0 0 1 2-2h3" />
          </svg>
        </button>
      </Tooltip>
    </template>
    <Form />
  </Drawer>
</template>

<style>
.sale-invoice-drawer {
  width: 75vw !important;
}

.sale-invoice-drawer--fullscreen {
  width: 100vw !important;
}

.sale-invoice-drawer__fs-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin-right: 8px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(0, 0, 0, 0.45);
  cursor: pointer;
  transition: all 0.2s;
}

.sale-invoice-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}
</style>
