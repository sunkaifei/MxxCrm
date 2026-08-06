<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import type { VbenFormSchema } from '@vben/common-ui';
import { $t } from '#/locales';
import { createPurchaseReturnApi, getPurchaseReturnInfoApi, updatePurchaseReturnApi } from '#/api';
import { Button, message, Tooltip } from 'ant-design-vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });
const items = ref<any[]>([]);

const drawerClass = computed(() => [
  'return-drawer',
  { 'return-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const formSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'receiptNo',
    label: '关联收货单号',
    componentProps: { placeholder: '请输入收货单号', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'supplierName',
    label: '供应商',
    componentProps: { placeholder: '请输入供应商', allowClear: true },
  },
  {
    component: 'DatePicker',
    fieldName: 'returnDate',
    label: '退货日期',
    componentProps: { placeholder: '请选择退货日期', allowClear: true },
  },
  {
    component: 'InputNumber',
    fieldName: 'totalAmount',
    label: '总金额',
    componentProps: { placeholder: '请输入总金额', min: 0, precision: 2 },
  },
  {
    component: 'Textarea',
    fieldName: 'reason',
    label: '退货原因',
    componentProps: { placeholder: '请输入退货原因', rows: 2, allowClear: true },
    formItemClass: 'col-span-2',
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: { placeholder: '请输入备注', rows: 2, allowClear: true },
    formItemClass: 'col-span-2',
  },
];

const [MainForm, mainFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: { class: 'w-full' },
  },
  schema: formSchema,
});

const [Drawer, drawerApi] = useVbenDrawer({
  async onConfirm() {
    try {
      const valid = await mainFormApi.validate();
      if (!valid.valid) return;

      confirmLoading.value = true;
      const values = await mainFormApi.getValues();

      const data = {
        ...values,
        items: items.value,
      };

      if (drawerData.value.create) {
        await createPurchaseReturnApi(data);
        message.success($t('ui.notification.create_success'));
      } else {
        await updatePurchaseReturnApi({ ...data, id: drawerData.value.row.id });
        message.success($t('ui.notification.update_success'));
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      confirmLoading.value = false;
    }
  },
  onCancel() {
    drawerApi.close();
  },
  onOpenChange(isOpen: boolean) {
    if (isOpen) {
      isFullscreen.value = false;
      drawerData.value = drawerApi.getData<{ create: boolean; row?: any }>() || { create: true };
      mainFormApi.resetForm();
      items.value = drawerData.value.row?.items || [];
      confirmLoading.value = false;
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getPurchaseReturnInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;

    mainFormApi.setValues({
      receiptNo: data.receiptNo,
      supplierName: data.supplierName,
      returnDate: data.returnDate,
      totalAmount: data.totalAmount,
      reason: data.reason,
      remark: data.remark,
    });
    items.value = data.items || [];
  } catch (e) {
    console.error('[退货单] 加载详情失败:', e);
  }
}

function addItem() {
  items.value.push({
    productName: '',
    productSku: '',
    unit: '',
    returnQuantity: 1,
    unitPrice: 0,
    amount: 0,
    reason: '',
  });
}

function removeItem(index: number) {
  items.value.splice(index, 1);
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="drawerData.create ? '新建退货单' : '编辑退货单'"
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '还原' : '最大化'">
        <button type="button" class="return-drawer__fs-btn" @click="toggleFullscreen">
          <svg v-if="!isFullscreen" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </button>
      </Tooltip>
    </template>

    <div class="return-drawer__body">
      <MainForm />

      <div class="mt-4">
        <div class="flex justify-between items-center mb-3">
          <h3 class="text-base font-semibold">退货明细</h3>
          <Button type="dashed" size="small" @click="addItem">添加明细</Button>
        </div>
        <table class="w-full border-collapse return-drawer__table">
          <thead>
            <tr>
              <th class="border px-2 py-1.5">产品名称</th>
              <th class="border px-2 py-1.5">SKU</th>
              <th class="border px-2 py-1.5">单位</th>
              <th class="border px-2 py-1.5">退货数量</th>
              <th class="border px-2 py-1.5">单价</th>
              <th class="border px-2 py-1.5">金额</th>
              <th class="border px-2 py-1.5">退货原因</th>
              <th class="border px-2 py-1.5">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, index) in items" :key="index">
              <td class="border px-2 py-1">
                <input v-model="item.productName" class="w-full border rounded px-2 py-1 text-sm" placeholder="产品名称" />
              </td>
              <td class="border px-2 py-1">
                <input v-model="item.productSku" class="w-full border rounded px-2 py-1 text-sm" placeholder="SKU" />
              </td>
              <td class="border px-2 py-1">
                <input v-model="item.unit" class="w-full border rounded px-2 py-1 text-sm" placeholder="单位" />
              </td>
              <td class="border px-2 py-1">
                <input v-model.number="item.returnQuantity" type="number" min="0" class="w-full border rounded px-2 py-1 text-sm" placeholder="退货数量" />
              </td>
              <td class="border px-2 py-1">
                <input v-model.number="item.unitPrice" type="number" min="0" class="w-full border rounded px-2 py-1 text-sm" placeholder="单价" />
              </td>
              <td class="border px-2 py-1">
                <input v-model.number="item.amount" type="number" min="0" class="w-full border rounded px-2 py-1 text-sm" placeholder="金额" />
              </td>
              <td class="border px-2 py-1">
                <input v-model="item.reason" class="w-full border rounded px-2 py-1 text-sm" placeholder="退货原因" />
              </td>
              <td class="border px-2 py-1 text-center">
                <Button type="link" danger size="small" @click="removeItem(index)">删除</Button>
              </td>
            </tr>
            <tr v-if="items.length === 0">
              <td colspan="8" class="border px-4 py-8 text-center text-gray-400 text-sm">
                暂无明细，点击上方按钮添加
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </Drawer>
</template>

<style>
.return-drawer {
  width: 75vw !important;
}

.return-drawer--fullscreen {
  width: 100vw !important;
}

.return-drawer__fs-btn {
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

.return-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

.return-drawer__body {
  padding: 0 8px;
  overflow-y: auto;
  height: calc(100vh - 150px);
}

.return-drawer__table input {
  font-size: 12px;
}
</style>