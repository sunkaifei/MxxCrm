<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref } from 'vue';

import { useVbenForm } from '@vben/common-ui';

import { Button, InputNumber, Table, Tooltip, message } from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import { createShipmentApi, getOrderInfoApi } from '#/api';

const props = withDefaults(
  defineProps<{ row?: any }>(),
  { row: () => ({}) },
);

const isFullscreen = ref(false);
const items = ref<any[]>([]);
const loading = ref(false);

const drawerClass = computed(() => [
  'sale-shipment-drawer',
  { 'sale-shipment-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const shippingMethodOptions = [
  { label: '快递', value: 1 },
  { label: '物流', value: 2 },
  { label: '自提', value: 3 },
  { label: '送货上门', value: 4 },
  { label: '其他', value: 5 },
];

const formSchema: VbenFormSchema[] = [
  {
    component: 'DatePicker',
    fieldName: 'shipmentDate',
    label: '发货日期',
    rules: 'required',
    componentProps: {
      placeholder: '请选择',
      style: 'width:100%',
      valueFormat: 'YYYY-MM-DD',
    },
  },
  {
    component: 'Input',
    fieldName: 'logisticsCompany',
    label: '物流公司',
    componentProps: { placeholder: '请输入物流公司' },
  },
  {
    component: 'Input',
    fieldName: 'trackingNo',
    label: '物流单号',
    componentProps: { placeholder: '请输入物流单号' },
  },
  {
    component: 'Select',
    fieldName: 'shippingMethod',
    label: '配送方式',
    componentProps: {
      placeholder: '请选择',
      options: shippingMethodOptions,
      allowClear: true,
    },
  },
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

const itemColumns = [
  { title: '产品名称', dataIndex: 'productName', width: 200 },
  { title: '订单数量', dataIndex: 'quantity', width: 100 },
  { title: '已发货数量', dataIndex: 'deliveredQuantity', width: 110 },
  { title: '本次发货数量', dataIndex: 'shipQuantity', width: 140 },
];

const totalShipQuantity = computed(() => {
  return items.value.reduce(
    (sum, item) => sum + (Number(item.shipQuantity) || 0),
    0,
  );
});

async function loadOrderInfo(orderId: number) {
  loading.value = true;
  try {
    const info: any = await getOrderInfoApi(orderId);
    const data = info || {};
    const allItems = Array.isArray(data.items) ? data.items : [];
    // 仅显示实物商品（product_type=1）
    items.value = allItems
      .filter((it: any) => Number(it.productType) === 1)
      .map((it: any) => ({
        id: it.id,
        productName: it.productName,
        quantity: Number(it.quantity) || 0,
        deliveredQuantity: Number(it.deliveredQuantity) || 0,
        shipQuantity: 0,
      }));

    // 默认填充收货信息（若订单已存在则带入）
    formApi.setValues({
      shipmentDate: new Date().toISOString().slice(0, 10),
      receiverName: data.receiverName || props.row?.receiverName,
      receiverPhone: data.receiverPhone || props.row?.receiverPhone,
      shippingAddress: data.shippingAddress || props.row?.shippingAddress,
      shippingMethod: data.shippingMethod,
    });
  } catch {
    items.value = [];
    formApi.setValues({
      shipmentDate: new Date().toISOString().slice(0, 10),
      receiverName: props.row?.receiverName,
      receiverPhone: props.row?.receiverPhone,
      shippingAddress: props.row?.shippingAddress,
    });
  } finally {
    loading.value = false;
  }
}

async function handleSubmit() {
  const { valid, values } = await formApi.validate();
  if (!valid) return;

  const shipItems = items.value
    .filter((it) => Number(it.shipQuantity) > 0)
    .map((it) => ({
      orderItemId: it.id,
      productName: it.productName,
      quantity: Number(it.shipQuantity),
    }));

  if (shipItems.length === 0) {
    message.error('请至少填写一条本次发货数量');
    return;
  }

  const data = {
    ...values,
    orderId: Number(props.row?.id),
    customerId: Number(props.row?.customerId) || undefined,
    items: shipItems,
  };

  try {
    await createShipmentApi(data);
    message.success('发货成功');
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
      items.value = [];
      if (props.row && props.row.id) {
        void loadOrderInfo(Number(props.row.id));
      }
    }
  },
});
</script>

<template>
  <Drawer
    title="新建发货单"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '退出全屏' : '全屏'">
        <button
          type="button"
          class="sale-shipment-drawer__fs-btn"
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
    <div class="mt-4">
      <div class="mb-2 font-medium">发货商品明细</div>
      <Table
        :columns="itemColumns"
        :data-source="items"
        :pagination="false"
        :loading="loading"
        bordered
        size="small"
        :row-key="(record: any) => record.id"
        :scroll="{ x: 600 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'productName'">
            <span>{{ record.productName }}</span>
          </template>
          <template v-else-if="column.dataIndex === 'quantity'">
            <span>{{ record.quantity }}</span>
          </template>
          <template v-else-if="column.dataIndex === 'deliveredQuantity'">
            <span>{{ record.deliveredQuantity }}</span>
          </template>
          <template v-else-if="column.dataIndex === 'shipQuantity'">
            <InputNumber
              v-model:value="record.shipQuantity"
              :min="0"
              :max="Math.max(0, record.quantity - record.deliveredQuantity)"
              :precision="0"
              style="width: 120px"
            />
          </template>
        </template>
        <template #footer>
          <div class="text-right">
            发货总数量：
            <span class="font-medium text-blue-600">
              {{ totalShipQuantity }}
            </span>
          </div>
        </template>
      </Table>
    </div>
    <div class="mt-4 flex justify-end gap-2">
      <Button @click="drawerApi.close()">取消</Button>
      <Button type="primary" @click="handleSubmit">保存</Button>
    </div>
  </Drawer>
</template>

<style>
.sale-shipment-drawer {
  width: 75vw !important;
}

.sale-shipment-drawer--fullscreen {
  width: 100vw !important;
}

.sale-shipment-drawer__fs-btn {
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

.sale-shipment-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}
</style>
