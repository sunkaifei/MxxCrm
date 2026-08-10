<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import type { VbenFormSchema } from '@vben/common-ui';
import { $t } from '#/locales';
import { createReceiptApi, getReceiptInfoApi, updateReceiptApi, getWarehouseListApi } from '#/api';
import { Button, message, Tooltip } from 'ant-design-vue';
import ProductSelectModal from '../../sale/components/ProductSelectModal.vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });
const items = ref<any[]>([]);
const productSelectVisible = ref(false);
const warehouseOptions = ref<{ value: number; label: string }[]>([]);
const currentWarehouseId = ref<number | undefined>(undefined);

async function loadWarehouses() {
  try {
    const res = await getWarehouseListApi({ pageSize: 200 });
    const list = res?.list || res?.items || [];
    warehouseOptions.value = list.map((w: any) => ({ value: Number(w.id), label: w.name }));
  } catch {}
}
loadWarehouses();

// 已选产品ID列表（排除已添加的产品）
const excludeProductIds = computed(() =>
  items.value.map((it: any) => Number(it.productId)).filter(Boolean),
);

const drawerClass = computed(() => [
  'receipt-drawer',
  { 'receipt-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const formSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'purchaseNo',
    label: '采购单号',
    rules: 'required',
    componentProps: { placeholder: '请输入采购单号', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'supplierName',
    label: '供应商',
    componentProps: { placeholder: '请输入供应商', allowClear: true },
  },
  {
    component: 'Select',
    fieldName: 'warehouseId',
    label: '收货仓库',
    componentProps: () => ({
      options: warehouseOptions.value,
      placeholder: '请选择收货仓库',
      allowClear: true,
      onChange: (val: any) => {
        currentWarehouseId.value = val ?? undefined;
      },
    }),
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
        warehouseId: values.warehouseId || undefined,
        items: items.value,
      };

      if (drawerData.value.create) {
        await createReceiptApi(data);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateReceiptApi({ ...data, id: drawerData.value.row.id });
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
      currentWarehouseId.value = undefined;
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
    const resp = await getReceiptInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;

    mainFormApi.setValues({
      purchaseNo: data.purchaseNo,
      supplierName: data.supplierName,
      warehouseId: data.warehouseId,
      remark: data.remark,
    });
    currentWarehouseId.value = data.warehouseId ? Number(data.warehouseId) : undefined;
    items.value = data.items || [];
  } catch (e) {
    console.error('[收货单] 加载详情失败:', e);
  }
}

function openProductSelect() {
  productSelectVisible.value = true;
}

function onProductSelected(selectedItems: any[]) {
  selectedItems.forEach((item) => {
    items.value.push({
      productId: item.productId,
      productName: item.productName || '',
      productCode: item.productCode || '',
      spec: item.spec || '',
      skuId: item.skuId,
      skuCode: item.skuCode,
      unit: item.unit || '',
      unitPrice: item.unitPrice || 0,
      orderQuantity: 1,
      receivedQuantity: 0,
      currentQuantity: 0,
      remark: '',
    });
  });
}

function removeItem(index: number) {
  items.value.splice(index, 1);
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="drawerData.create ? '新建收货单' : '编辑收货单'"
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '还原' : '最大化'">
        <button type="button" class="receipt-drawer__fs-btn" @click="toggleFullscreen">
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

    <div class="receipt-drawer__body">
      <MainForm />

      <div class="mt-4">
        <div class="flex justify-between items-center mb-3">
          <h3 class="text-base font-semibold">收货明细</h3>
          <Button type="dashed" size="small" @click="openProductSelect">选择产品</Button>
        </div>
        <table class="w-full border-collapse receipt-drawer__table">
          <thead>
            <tr>
              <th class="border px-2 py-1.5">产品名称</th>
              <th class="border px-2 py-1.5">采购数量</th>
              <th class="border px-2 py-1.5">已收数量</th>
              <th class="border px-2 py-1.5">本次收货数量</th>
              <th class="border px-2 py-1.5">备注</th>
              <th class="border px-2 py-1.5">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, index) in items" :key="index">
              <td class="border px-2 py-1 text-sm">
                <div>{{ item.productName }}</div>
                <div v-if="item.spec" class="text-xs text-gray-400">{{ item.spec }}</div>
              </td>
              <td class="border px-2 py-1">
                <input v-model.number="item.orderQuantity" type="number" min="0" class="w-full border rounded px-2 py-1 text-sm" placeholder="采购数量" />
              </td>
              <td class="border px-2 py-1">
                <input v-model.number="item.receivedQuantity" type="number" min="0" class="w-full border rounded px-2 py-1 text-sm" placeholder="已收数量" />
              </td>
              <td class="border px-2 py-1">
                <input v-model.number="item.currentQuantity" type="number" min="0" class="w-full border rounded px-2 py-1 text-sm" placeholder="本次收货" />
              </td>
              <td class="border px-2 py-1">
                <input v-model="item.remark" class="w-full border rounded px-2 py-1 text-sm" placeholder="备注" />
              </td>
              <td class="border px-2 py-1 text-center">
                <Button type="link" danger size="small" @click="removeItem(index)">删除</Button>
              </td>
            </tr>
            <tr v-if="items.length === 0">
              <td colspan="6" class="border px-4 py-8 text-center text-gray-400 text-sm">
                暂无明细，点击上方按钮添加
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <ProductSelectModal
      v-model:visible="productSelectVisible"
      :exclude-ids="excludeProductIds"
      :warehouse-id="currentWarehouseId"
      @select="onProductSelected"
    />
  </Drawer>
</template>

<style>
.receipt-drawer {
  width: 75vw !important;
}

.receipt-drawer--fullscreen {
  width: 100vw !important;
}

.receipt-drawer__fs-btn {
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

.receipt-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

.receipt-drawer__body {
  padding: 0 8px;
  overflow-y: auto;
  height: calc(100vh - 150px);
}

.receipt-drawer__table input {
  font-size: 12px;
}
</style>
