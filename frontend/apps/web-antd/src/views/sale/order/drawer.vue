<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref, watch } from 'vue';

import { useVbenForm } from '@vben/common-ui';

import {
  Button,
  InputNumber,
  Tabs,
  TabPane,
  Table,
  Tooltip,
  message,
} from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import { createOrderApi, getOrderInfoApi, updateOrderApi } from '#/api';

const props = withDefaults(
  defineProps<{ create?: boolean; row?: any }>(),
  { create: true, row: () => ({}) },
);

const isEdit = computed(() => !props.create);
const activeTab = ref('basic');
const items = ref<any[]>([]);
const shippingFee = ref(0);
const taxAmount = ref(0);
const discountAmount = ref(0);
const otherFee = ref(0);
const isFullscreen = ref(false);

const drawerClass = computed(() => [
  'sale-order-drawer',
  { 'sale-order-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const orderTypeOptions = [
  { label: '销售订单', value: 1 },
  { label: '退货订单', value: 2 },
];

const currencyOptions = [
  { label: 'CNY 人民币', value: 1 },
  { label: 'USD 美元', value: 2 },
  { label: 'EUR 欧元', value: 3 },
  { label: 'GBP 英镑', value: 4 },
  { label: 'JPY 日元', value: 5 },
  { label: 'HKD 港币', value: 6 },
];

const shippingMethodOptions = [
  { label: '快递', value: 1 },
  { label: '物流', value: 2 },
  { label: '自提', value: 3 },
  { label: '送货上门', value: 4 },
];

const paymentMethodOptions = [
  { label: '银行转账', value: 1 },
  { label: '支付宝', value: 2 },
  { label: '微信支付', value: 3 },
  { label: '现金', value: 4 },
  { label: '支票', value: 5 },
  { label: '其他', value: 6 },
];

const basicFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'title',
    label: '订单标题',
    rules: 'required',
    componentProps: { placeholder: '如：XX公司采购订单' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'orderType',
    label: '订单类型',
    defaultValue: 1,
    componentProps: { placeholder: '请选择', options: orderTypeOptions },
  },
  {
    component: 'DatePicker',
    fieldName: 'orderDate',
    label: '下单日期',
    rules: 'required',
    componentProps: { placeholder: '请选择', style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
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
    component: 'InputNumber',
    fieldName: 'contactId',
    label: '联系人ID',
    componentProps: { placeholder: '请输入联系人ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'Input',
    fieldName: 'contactName',
    label: '联系人姓名',
    componentProps: { placeholder: '请输入联系人姓名' },
  },
  {
    component: 'InputNumber',
    fieldName: 'ownerUserId',
    label: '负责人ID',
    rules: 'required',
    componentProps: { placeholder: '请输入负责人ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'InputNumber',
    fieldName: 'deptId',
    label: '部门ID',
    componentProps: { placeholder: '请输入部门ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'InputNumber',
    fieldName: 'opportunityId',
    label: '商机ID',
    componentProps: { placeholder: '请输入商机ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'InputNumber',
    fieldName: 'quotationId',
    label: '报价单ID',
    componentProps: { placeholder: '请输入报价单ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'InputNumber',
    fieldName: 'contractId',
    label: '合同ID',
    componentProps: { placeholder: '请输入合同ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'Select',
    fieldName: 'currency',
    label: '币种',
    defaultValue: 1,
    componentProps: { placeholder: '请选择', options: currencyOptions },
  },
  {
    component: 'DatePicker',
    fieldName: 'deliveryDate',
    label: '预计交付日期',
    componentProps: { placeholder: '请选择', style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: { placeholder: '备注信息', rows: 3, showCount: true, maxlength: 500 },
    wrapperClass: 'col-span-2',
  },
];

const shippingFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'receiverName',
    label: '收货人',
    componentProps: { placeholder: '请输入收货人姓名' },
  },
  {
    component: 'Input',
    fieldName: 'receiverPhone',
    label: '收货人电话',
    componentProps: { placeholder: '请输入联系电话' },
  },
  {
    component: 'Textarea',
    fieldName: 'shippingAddress',
    label: '收货地址',
    componentProps: { placeholder: '请输入收货地址', rows: 2 },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Textarea',
    fieldName: 'billingAddress',
    label: '账单地址',
    componentProps: { placeholder: '请输入账单地址', rows: 2 },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'shippingMethod',
    label: '配送方式',
    componentProps: { placeholder: '请选择', options: shippingMethodOptions, allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'trackingNo',
    label: '物流单号',
    componentProps: { placeholder: '请输入物流单号' },
  },
];

const paymentFormSchema: VbenFormSchema[] = [
  {
    component: 'Select',
    fieldName: 'paymentMethod',
    label: '支付方式',
    componentProps: { placeholder: '请选择', options: paymentMethodOptions, allowClear: true },
  },
  {
    component: 'DatePicker',
    fieldName: 'paymentDueDate',
    label: '付款截止日期',
    componentProps: { placeholder: '请选择', style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
  },
  {
    component: 'InputNumber',
    fieldName: 'paidAmount',
    label: '已付金额',
    componentProps: { placeholder: '已付金额', style: 'width:100%', precision: 2, disabled: true },
  },
  {
    component: 'InputNumber',
    fieldName: 'unpaidAmount',
    label: '未付金额',
    componentProps: { placeholder: '未付金额', style: 'width:100%', precision: 2, disabled: true },
  },
];

const [BasicForm, basicFormApi] = useVbenForm({
  schema: basicFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

const [ShippingForm, shippingFormApi] = useVbenForm({
  schema: shippingFormSchema,
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

function calcLineAmount(item: any): number {
  const qty = Number(item.quantity) || 0;
  const price = Number(item.unitPrice) || 0;
  const disc = Number(item.discountRate) || 100;
  const tax = Number(item.taxRate) || 0;
  const gross = qty * price;
  const discountAmt = gross * (100 - disc) / 100;
  const afterDisc = gross - discountAmt;
  const taxAmt = afterDisc * tax / 100;
  const lineAmt = afterDisc + taxAmt;
  return Math.round(lineAmt * 100) / 100;
}

function updateLineAmount(index: number) {
  items.value[index].amount = calcLineAmount(items.value[index]);
}

const productAmount = computed(() => {
  return items.value.reduce((sum, item) => sum + (Number(item.amount) || 0), 0);
});

const totalAmount = computed(() => {
  const prod = productAmount.value;
  const ship = Number(shippingFee.value) || 0;
  const tax = Number(taxAmount.value) || 0;
  const other = Number(otherFee.value) || 0;
  const disc = Number(discountAmount.value) || 0;
  return Math.round((prod - disc + ship + tax + other) * 100) / 100;
});

const unpaidAmountComp = computed(() => {
  const paid = Number(props.row?.paidAmount) || 0;
  return Math.round((totalAmount.value - paid) * 100) / 100;
});

watch(
  [productAmount, shippingFee, taxAmount, discountAmount, otherFee, totalAmount],
  () => {
    paymentFormApi.setValues({
      paidAmount: Number(props.row?.paidAmount) || 0,
      unpaidAmount: unpaidAmountComp.value,
    });
  },
  { immediate: true },
);

const itemColumns = [
  { title: '商品名称', dataIndex: 'productName', width: 180 },
  { title: '商品编码', dataIndex: 'productCode', width: 120 },
  { title: '规格', dataIndex: 'spec', width: 120 },
  { title: '单位', dataIndex: 'unit', width: 70 },
  {
    title: '数量',
    dataIndex: 'quantity',
    width: 100,
  },
  {
    title: '单价',
    dataIndex: 'unitPrice',
    width: 120,
  },
  {
    title: '折扣率(%)',
    dataIndex: 'discountRate',
    width: 110,
  },
  {
    title: '税率(%)',
    dataIndex: 'taxRate',
    width: 100,
  },
  { title: '金额', dataIndex: 'amount', width: 110 },
  { title: '操作', dataIndex: 'action', width: 70 },
];

function addItem() {
  items.value.push({
    productName: '',
    productCode: '',
    spec: '',
    unit: '',
    quantity: 1,
    unitPrice: 0,
    discountRate: 100,
    taxRate: 0,
    amount: 0,
  });
}

function removeItem(index: number) {
  items.value.splice(index, 1);
}

watch(
  () => props.row,
  async (val) => {
    if (val && !props.create) {
      try {
        const info = await getOrderInfoApi(val.id);
        const data = info || val;
        basicFormApi.setValues({
          title: data.title,
          orderType: data.orderType ?? 1,
          orderDate: data.orderDate,
          customerId: data.customerId,
          customerName: data.customerName,
          contactId: data.contactId,
          contactName: data.contactName,
          ownerUserId: data.ownerUserId,
          deptId: data.deptId,
          opportunityId: data.opportunityId,
          quotationId: data.quotationId,
          contractId: data.contractId,
          currency: data.currency ?? 1,
          deliveryDate: data.deliveryDate,
          remark: data.remark,
        });
        shippingFormApi.setValues({
          receiverName: data.receiverName,
          receiverPhone: data.receiverPhone,
          shippingAddress: data.shippingAddress,
          billingAddress: data.billingAddress,
          shippingMethod: data.shippingMethod,
          trackingNo: data.trackingNo,
        });
        paymentFormApi.setValues({
          paymentMethod: data.paymentMethod,
          paymentDueDate: data.paymentDueDate,
          paidAmount: data.paidAmount ?? 0,
          unpaidAmount: data.unpaidAmount ?? 0,
        });
        items.value = Array.isArray(data.items)
          ? data.items.map((it: any) => ({ ...it }))
          : [];
        shippingFee.value = data.shippingFee ?? 0;
        taxAmount.value = data.taxAmount ?? 0;
        discountAmount.value = data.discountAmount ?? 0;
        otherFee.value = data.otherFee ?? 0;
      } catch {
        basicFormApi.setValues(val);
        items.value = [];
        shippingFee.value = 0;
        taxAmount.value = 0;
        discountAmount.value = 0;
        otherFee.value = 0;
      }
    }
  },
  { immediate: true },
);

async function handleSubmit() {
  const basicValid = await basicFormApi.validate();
  if (!basicValid.valid) {
    activeTab.value = 'basic';
    return;
  }
  if (items.value.length === 0) {
    message.error('请至少添加一条商品明细');
    activeTab.value = 'items';
    return;
  }
  const basicValues = await basicFormApi.getValues();
  const shippingValues = await shippingFormApi.getValues();
  const paymentValues = await paymentFormApi.getValues();

  const submitItems = items.value.map((item, idx) => ({
    ...item,
    sort: idx,
    quantity: Number(item.quantity) || 1,
    unitPrice: Number(item.unitPrice) || 0,
    discountRate: Number(item.discountRate) || 100,
    taxRate: Number(item.taxRate) || 0,
    amount: calcLineAmount(item),
  }));

  const data = {
    ...basicValues,
    ...shippingValues,
    paymentMethod: paymentValues.paymentMethod,
    paymentDueDate: paymentValues.paymentDueDate,
    items: submitItems,
    shippingFee: Number(shippingFee.value) || 0,
    taxAmount: Number(taxAmount.value) || 0,
    discountAmount: Number(discountAmount.value) || 0,
    otherFee: Number(otherFee.value) || 0,
    productAmount: productAmount.value,
    totalAmount: totalAmount.value,
  };

  try {
    if (isEdit.value) {
      await updateOrderApi({ ...data, id: props.row.id });
      message.success('更新成功');
    } else {
      await createOrderApi(data);
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
      isFullscreen.value = false;
      basicFormApi.resetForm();
      shippingFormApi.resetForm();
      paymentFormApi.resetForm();
      items.value = [];
      shippingFee.value = 0;
      taxAmount.value = 0;
      discountAmount.value = 0;
      otherFee.value = 0;
      if (!props.create && props.row) {
        basicFormApi.setValues({
          orderType: 1,
          currency: 1,
          orderDate: new Date().toISOString().slice(0, 10),
          ...props.row,
        });
        shippingFormApi.setValues(props.row);
        paymentFormApi.setValues({ paidAmount: 0, unpaidAmount: 0, ...props.row });
      } else {
        basicFormApi.setValues({
          orderType: 1,
          currency: 1,
          orderDate: new Date().toISOString().slice(0, 10),
        });
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
    :title="isEdit ? '编辑销售订单' : '新建销售订单'"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '退出全屏' : '全屏'">
        <button type="button" class="sale-order-drawer__fs-btn" @click="toggleFullscreen">
          <svg v-if="!isFullscreen" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M8 3H5a2 2 0 0 0-2 2v3" />
            <path d="M21 8V5a2 2 0 0 0-2-2h-3" />
            <path d="M3 16v3a2 2 0 0 0 2 2h3" />
            <path d="M16 21h3a2 2 0 0 0 2-2v-3" />
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
      <TabPane key="items" tab="商品明细">
        <div class="mb-3">
          <Button type="primary" @click="addItem">添加商品</Button>
        </div>
        <Table
          :columns="itemColumns"
          :data-source="items"
          :pagination="false"
          bordered
          size="small"
          row-key="(_: any, index: number) => index"
          :scroll="{ x: 1100 }"
        >
          <template #bodyCell="{ column, record, index }">
            <template v-if="column.dataIndex === 'productName'">
              <a-input v-model:value="record.productName" placeholder="商品名称" />
            </template>
            <template v-else-if="column.dataIndex === 'productCode'">
              <a-input v-model:value="record.productCode" placeholder="编码" />
            </template>
            <template v-else-if="column.dataIndex === 'spec'">
              <a-input v-model:value="record.spec" placeholder="规格" />
            </template>
            <template v-else-if="column.dataIndex === 'unit'">
              <a-input v-model:value="record.unit" placeholder="单位" style="width: 60px" />
            </template>
            <template v-else-if="column.dataIndex === 'quantity'">
              <InputNumber
                v-model:value="record.quantity"
                :min="1"
                :precision="0"
                style="width: 90px"
                @change="() => updateLineAmount(index)"
              />
            </template>
            <template v-else-if="column.dataIndex === 'unitPrice'">
              <InputNumber
                v-model:value="record.unitPrice"
                :min="0"
                :precision="2"
                style="width: 110px"
                @change="() => updateLineAmount(index)"
              />
            </template>
            <template v-else-if="column.dataIndex === 'discountRate'">
              <InputNumber
                v-model:value="record.discountRate"
                :min="0"
                :max="100"
                :precision="2"
                style="width: 100px"
                addon-after="%"
                @change="() => updateLineAmount(index)"
              />
            </template>
            <template v-else-if="column.dataIndex === 'taxRate'">
              <InputNumber
                v-model:value="record.taxRate"
                :min="0"
                :max="100"
                :precision="2"
                style="width: 90px"
                addon-after="%"
                @change="() => updateLineAmount(index)"
              />
            </template>
            <template v-else-if="column.dataIndex === 'amount'">
              <span class="font-medium">{{ (record.amount || 0).toFixed(2) }}</span>
            </template>
            <template v-else-if="column.dataIndex === 'action'">
              <Button type="link" danger size="small" @click="removeItem(index)">删除</Button>
            </template>
          </template>
        </Table>
        <div class="mt-4 flex flex-col items-end gap-2 pr-4">
          <div class="flex items-center gap-2">
            <span class="w-24 text-right text-gray-500">商品金额合计：</span>
            <span class="w-32 text-right">{{ productAmount.toFixed(2) }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right text-gray-500">整单折扣：</span>
            <InputNumber
              v-model:value="discountAmount"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right text-gray-500">运费：</span>
            <InputNumber
              v-model:value="shippingFee"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right text-gray-500">税额：</span>
            <InputNumber
              v-model:value="taxAmount"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right text-gray-500">其他费用：</span>
            <InputNumber
              v-model:value="otherFee"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2 border-t pt-2">
            <span class="w-24 text-right font-medium">订单总金额：</span>
            <span class="w-32 text-right text-lg font-bold text-red-500">
              {{ totalAmount.toFixed(2) }}
            </span>
          </div>
        </div>
      </TabPane>
      <TabPane key="shipping" tab="收货信息">
        <ShippingForm />
      </TabPane>
      <TabPane key="payment" tab="支付信息">
        <PaymentForm />
      </TabPane>
    </Tabs>
    <div class="flex justify-end gap-2 mt-4">
      <Button @click="closeDrawer">取消</Button>
      <Button type="primary" @click="handleSubmit">保存</Button>
    </div>
  </Drawer>
</template>

<style>
.sale-order-drawer {
  width: 75vw !important;
}

.sale-order-drawer--fullscreen {
  width: 100vw !important;
}

.sale-order-drawer__fs-btn {
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

.sale-order-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}
</style>
