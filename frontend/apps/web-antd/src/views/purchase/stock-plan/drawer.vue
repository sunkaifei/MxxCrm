<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import type { VbenFormSchema } from '@vben/common-ui';
import { $t } from '#/locales';
import { createStockPlanApi, getStockPlanInfoApi, updateStockPlanApi } from '#/api';
import { message, Tooltip } from 'ant-design-vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

const drawerClass = computed(() => [
  'stock-plan-drawer',
  { 'stock-plan-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const demandSourceOptions = [
  { label: '销售预测', value: 'sales_forecast' },
  { label: '安全库存', value: 'safety_stock' },
  { label: '订单需求', value: 'order_demand' },
  { label: '手动创建', value: 'manual' },
  { label: '其他', value: 'other' },
];

const formSchema: VbenFormSchema[] = [
  {
    component: 'InputNumber',
    fieldName: 'productId',
    label: '产品ID',
    rules: 'required',
    componentProps: { placeholder: '请输入产品ID', min: 1, precision: 0, style: { width: '100%' } },
  },
  {
    component: 'DatePicker',
    fieldName: 'planDate',
    label: '计划日期',
    componentProps: { placeholder: '请选择日期', style: { width: '100%' } },
  },
  {
    component: 'InputNumber',
    fieldName: 'demandQuantity',
    label: '需求量',
    rules: 'required',
    componentProps: { placeholder: '请输入需求量', min: 0, precision: 0, style: { width: '100%' } },
  },
  {
    component: 'Select',
    fieldName: 'demandSource',
    label: '需求来源',
    componentProps: { placeholder: '请选择需求来源', options: demandSourceOptions, allowClear: true },
  },
  {
    component: 'InputNumber',
    fieldName: 'safetyStock',
    label: '安全库存',
    componentProps: { placeholder: '0', min: 0, precision: 0, style: { width: '100%' } },
  },
  {
    component: 'InputNumber',
    fieldName: 'supplierId',
    label: '供应商ID',
    componentProps: { placeholder: '请输入供应商ID', min: 1, precision: 0, style: { width: '100%' } },
  },
  {
    component: 'InputNumber',
    fieldName: 'leadTimeDays',
    label: '提前期（天）',
    componentProps: { placeholder: '0', min: 0, precision: 0, style: { width: '100%' } },
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

      if (drawerData.value.create) {
        await createStockPlanApi(values);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateStockPlanApi({ ...values, id: drawerData.value.row.id });
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
      confirmLoading.value = false;
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getStockPlanInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const num = (v: any) => (v === null || v === undefined ? undefined : Number(v));

    mainFormApi.setValues({
      productId: num(data.productId),
      planDate: data.planDate,
      demandQuantity: num(data.demandQuantity),
      demandSource: data.demandSource,
      safetyStock: num(data.safetyStock),
      supplierId: num(data.supplierId),
      leadTimeDays: num(data.leadTimeDays),
      remark: data.remark,
    });
  } catch (e) {
    console.error('[备货计划] 加载详情失败:', e);
  }
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="drawerData.create ? '新建备货计划' : '编辑备货计划'"
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '还原' : '最大化'">
        <button type="button" class="stock-plan-drawer__fs-btn" @click="toggleFullscreen">
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

    <div class="stock-plan-drawer__body">
      <MainForm />
    </div>
  </Drawer>
</template>

<style>
.stock-plan-drawer {
  width: 75vw !important;
}

.stock-plan-drawer--fullscreen {
  width: 100vw !important;
}

.stock-plan-drawer__fs-btn {
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

.stock-plan-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

.stock-plan-drawer__body {
  padding: 0 8px;
  overflow-y: auto;
  height: calc(100vh - 150px);
}
</style>