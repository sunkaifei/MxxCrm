<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref, watch } from 'vue';

import { useVbenForm } from '@vben/common-ui';

import { Button, Tabs, TabPane, Tooltip, message } from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import { createPaymentApi, getPaymentInfoApi, updatePaymentApi } from '#/api';

const props = withDefaults(
  defineProps<{ create?: boolean; row?: any }>(),
  { create: true, row: () => ({}) },
);

const isEdit = computed(() => !props.create);
const activeTab = ref('basic');
const isFullscreen = ref(false);
const displayAmount = ref(0);

const drawerClass = computed(() => [
  'sale-payment-drawer',
  { 'sale-payment-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const paymentMethodOptions = [
  { label: '银行转账', value: 1 },
  { label: '支付宝', value: 2 },
  { label: '微信支付', value: 3 },
  { label: '现金', value: 4 },
  { label: '支票', value: 5 },
  { label: '其他', value: 6 },
];

const currencyOptions = [
  { label: 'CNY 人民币', value: 1 },
  { label: 'USD 美元', value: 2 },
  { label: 'EUR 欧元', value: 3 },
  { label: 'GBP 英镑', value: 4 },
  { label: 'JPY 日元', value: 5 },
  { label: 'HKD 港币', value: 6 },
];

const statusOptions = [
  { label: '待确认', value: 1, color: 'orange' },
  { label: '已确认', value: 2, color: 'green' },
  { label: '已驳回', value: 3, color: 'red' },
  { label: '已取消', value: 4, color: 'default' },
];

const basicFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'paymentNo',
    label: '回款编号',
    componentProps: { placeholder: '保存后自动生成', disabled: true },
  },
  {
    component: 'DatePicker',
    fieldName: 'paymentDate',
    label: '到账日期',
    rules: 'required',
    componentProps: {
      placeholder: '请选择实际到账日期',
      style: 'width:100%',
      valueFormat: 'YYYY-MM-DD',
    },
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
    fieldName: 'planId',
    label: '计划期次ID',
    componentProps: { placeholder: '关联回款计划期次', style: 'width:100%', min: 1 },
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
    component: 'Select',
    fieldName: 'currency',
    label: '币种',
    defaultValue: 1,
    componentProps: { placeholder: '请选择', options: currencyOptions },
  },
  {
    component: 'InputNumber',
    fieldName: 'amount',
    label: '回款金额',
    rules: 'required',
    componentProps: {
      placeholder: '请输入回款金额',
      style: 'width:100%',
      precision: 2,
      min: 0,
      prefix: '¥',
      onChange: (val: any) => { displayAmount.value = Number(val) || 0; },
    },
  },
  {
    component: 'Select',
    fieldName: 'paymentMethod',
    label: '支付方式',
    defaultValue: 1,
    rules: 'required',
    componentProps: { placeholder: '请选择支付方式', options: paymentMethodOptions },
  },
  {
    component: 'Select',
    fieldName: 'status',
    label: '回款状态',
    defaultValue: 1,
    componentProps: { placeholder: '请选择状态', options: statusOptions },
  },
];

const paymentFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'payer',
    label: '付款方',
    componentProps: { placeholder: '请输入付款方名称（如公司名称）' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'payerAccount',
    label: '付款方账号',
    componentProps: { placeholder: '请输入付款方银行账号' },
  },
  {
    component: 'Input',
    fieldName: 'bankFlowNo',
    label: '银行流水号',
    componentProps: { placeholder: '请输入银行流水号' },
  },
  {
    component: 'Input',
    fieldName: 'attachment',
    label: '回单附件',
    componentProps: { placeholder: '请输入回单附件URL' },
    wrapperClass: 'col-span-2',
  },
];

const otherFormSchema: VbenFormSchema[] = [
  {
    component: 'InputNumber',
    fieldName: 'ownerUserId',
    label: '负责人ID',
    componentProps: { placeholder: '负责人ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'InputNumber',
    fieldName: 'deptId',
    label: '部门ID',
    componentProps: { placeholder: '部门ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: { placeholder: '备注信息', rows: 4, showCount: true, maxlength: 500 },
    wrapperClass: 'col-span-2',
  },
];

const [BasicForm, basicFormApi] = useVbenForm({
  schema: basicFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

const [PaymentForm, paymentFormApi] = useVbenForm({
  schema: paymentFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

const [OtherForm, otherFormApi] = useVbenForm({
  schema: otherFormSchema,
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
        const info = await getPaymentInfoApi(val.id);
        const data = info || val;
        basicFormApi.setValues({
          paymentNo: data.paymentNo,
          paymentDate: data.paymentDate,
          contractId: data.contractId,
          orderId: data.orderId,
          planId: data.planId,
          customerId: data.customerId,
          customerName: data.customerName,
          currency: data.currency ?? 1,
          amount: data.amount ?? 0,
          paymentMethod: data.paymentMethod ?? 1,
          status: data.status ?? 1,
        });
        displayAmount.value = Number(data.amount) || 0;
        paymentFormApi.setValues({
          payer: data.payer,
          payerAccount: data.payerAccount,
          bankFlowNo: data.bankFlowNo,
          attachment: data.attachment,
        });
        otherFormApi.setValues({
          ownerUserId: data.ownerUserId,
          deptId: data.deptId,
          remark: data.remark,
        });
      } catch {
        basicFormApi.setValues(val);
      }
    }
  },
  { immediate: true },
);

async function handleSubmit() {
  const validResult = await basicFormApi.validate();
  if (!validResult.valid) {
    activeTab.value = 'basic';
    return false;
  }
  const basicValues = await basicFormApi.getValues();
  const paymentValues = await paymentFormApi.getValues();
  const otherValues = await otherFormApi.getValues();

  const data = {
    ...basicValues,
    ...paymentValues,
    ...otherValues,
  };

  drawerApi.setState({ confirmLoading: true });
  try {
    if (isEdit.value) {
      await updatePaymentApi({ ...data, id: props.row.id });
      message.success('更新成功');
    } else {
      await createPaymentApi(data);
      message.success('登记成功');
    }
    drawerApi.setData({ needRefresh: true });
    drawerApi.close();
  } catch {
    message.error('操作失败');
  } finally {
    drawerApi.setState({ confirmLoading: false });
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onOpenChange(isOpen) {
    if (isOpen) {
      isFullscreen.value = false;
      displayAmount.value = 0;
      basicFormApi.resetForm();
      paymentFormApi.resetForm();
      otherFormApi.resetForm();
      if (!props.create && props.row) {
        basicFormApi.setValues({
          currency: 1,
          paymentMethod: 1,
          status: 1,
          ...props.row,
        });
        paymentFormApi.setValues(props.row);
        otherFormApi.setValues(props.row);
      } else {
        basicFormApi.setValues({
          currency: 1,
          paymentMethod: 1,
          status: 1,
          paymentDate: new Date().toISOString().slice(0, 10),
        });
      }
    }
  },
  onCancel() {
    drawerApi.close();
    drawerApi.setData({ needRefresh: true });
  },
  onConfirm: handleSubmit,
});

function closeDrawer() {
  drawerApi.close();
  drawerApi.setData({ needRefresh: true });
}
</script>

<template>
  <Drawer
    :title="isEdit ? '编辑回款记录' : '登记回款'"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '退出全屏' : '全屏'">
        <button
          type="button"
          class="sale-payment-drawer__fs-btn"
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

    <Tabs v-model:activeKey="activeTab">
      <TabPane key="basic" tab="基本信息">
        <BasicForm />
      </TabPane>
      <TabPane key="payment" tab="收款信息">
        <PaymentForm />
        <div class="mt-4 rounded-lg bg-gray-50 p-4">
          <div class="mb-2 text-sm text-gray-500">回款提示</div>
          <div class="text-xs text-gray-400 leading-6">
            1. 回款登记后状态默认为"待确认"，财务确认后变为"已确认"<br />
            2. 银行流水号用于财务对账，建议填写<br />
            3. 如果关联了合同和回款计划期次，系统会自动更新计划的已收金额<br />
            4. 回单附件支持上传银行回单截图或PDF文件
          </div>
        </div>
      </TabPane>
      <TabPane key="other" tab="其他信息">
        <OtherForm />
      </TabPane>
    </Tabs>

    <div class="mt-6 border-t pt-4">
      <div class="flex items-center justify-between rounded-lg bg-blue-50 px-4 py-3">
        <span class="text-sm text-gray-600">本次回款金额</span>
        <span class="text-xl font-bold text-blue-600">
          ¥{{ displayAmount.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
        </span>
      </div>
    </div>
  </Drawer>
</template>

<style>
.sale-payment-drawer {
  width: 75vw !important;
}

.sale-payment-drawer--fullscreen {
  width: 100vw !important;
}

.sale-payment-drawer__fs-btn {
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

.sale-payment-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}
</style>
