<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import { message, Tooltip } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import {
  createRequisitionApi,
  getRequisitionInfoApi,
  updateRequisitionApi,
} from '#/api';
import { $t } from '#/locales';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });
const items = ref<any[]>([]);

const drawerClass = computed(() => [
  'requisition-drawer',
  { 'requisition-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const typeOptions = [
  { label: '缺货补货', value: 'replenish' },
  { label: '辅材采购', value: 'consumable' },
  { label: '备货采购', value: 'stock' },
  { label: '其他', value: 'other' },
];

const urgencyOptions = [
  { label: '普通', value: 0 },
  { label: '紧急', value: 1 },
  { label: '非常紧急', value: 2 },
];

const formSchema: VbenFormSchema[] = [
  {
    component: 'Select',
    fieldName: 'type',
    label: '申请类型',
    rules: 'required',
    componentProps: {
      placeholder: '请选择申请类型',
      options: typeOptions,
      allowClear: true,
    },
  },
  {
    component: 'Input',
    fieldName: 'title',
    label: '标题',
    rules: 'required',
    componentProps: { placeholder: '请输入申请标题', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'department',
    label: '部门',
    componentProps: { placeholder: '请输入部门', allowClear: true },
  },
  {
    component: 'DatePicker',
    fieldName: 'expectedDate',
    label: '期望到货日',
    componentProps: { placeholder: '请选择日期', style: { width: '100%' } },
  },
  {
    component: 'RadioGroup',
    fieldName: 'urgency',
    label: '紧急程度',
    defaultValue: 0,
    componentProps: {
      optionType: 'button',
      class: 'flex flex-wrap',
      options: urgencyOptions,
    },
  },
  {
    component: 'InputNumber',
    fieldName: 'estimatedAmount',
    label: '预估总金额',
    componentProps: {
      placeholder: '0.00',
      min: 0,
      precision: 2,
      style: { width: '100%' },
    },
  },
  {
    component: 'Select',
    fieldName: 'currency',
    label: '币种',
    defaultValue: 1,
    componentProps: {
      placeholder: '请选择币种',
      options: [
        { label: 'CNY 人民币', value: 1 },
        { label: 'USD 美元', value: 2 },
        { label: 'EUR 欧元', value: 3 },
      ],
      allowClear: true,
    },
  },
  {
    component: 'Textarea',
    fieldName: 'reason',
    label: '申请原因',
    componentProps: {
      placeholder: '请输入申请原因',
      rows: 3,
      allowClear: true,
    },
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
        await createRequisitionApi(data);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateRequisitionApi({ ...data, id: drawerData.value.row.id });
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
      drawerData.value = drawerApi.getData<{
        create: boolean;
        row?: any;
      }>() || { create: true };
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
    const resp = await getRequisitionInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const num = (v: any) =>
      v === null || v === undefined ? undefined : Number(v);

    mainFormApi.setValues({
      type: data.type,
      title: data.title,
      department: data.department,
      expectedDate: data.expectedDate,
      urgency: num(data.urgency) ?? 0,
      estimatedAmount: data.estimatedAmount
        ? Number(data.estimatedAmount)
        : undefined,
      currency: num(data.currency) ?? 1,
      reason: data.reason,
      remark: data.remark,
    });
    items.value = data.items || [];
  } catch (error) {
    console.error('[采购申请] 加载详情失败:', error);
  }
}

function addItem() {
  items.value.push({
    productName: '',
    sku: '',
    spec: '',
    unit: '',
    quantity: 1,
    estimatedPrice: 0,
    estimatedAmount: 0,
    remark: '',
  });
}

function removeItem(index: number) {
  items.value.splice(index, 1);
}

function updateItemAmount(item: any) {
  const qty = Number.parseFloat(item.quantity) || 0;
  const price = Number.parseFloat(item.estimatedPrice) || 0;
  item.estimatedAmount = Number.parseFloat((qty * price).toFixed(2));
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="drawerData.create ? '新建采购申请' : '编辑采购申请'"
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '还原' : '最大化'">
        <button
          type="button"
          class="requisition-drawer__fs-btn"
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
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
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
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </button>
      </Tooltip>
    </template>

    <div class="requisition-drawer__body">
      <MainForm />

      <div class="mt-4">
        <div class="flex justify-between items-center mb-3">
          <h3 class="text-base font-semibold">商品明细</h3>
          <Button type="dashed" size="small" @click="addItem">添加明细</Button>
        </div>
        <table class="w-full border-collapse requisition-drawer__table">
          <thead>
            <tr>
              <th class="border px-2 py-1.5">产品名称</th>
              <th class="border px-2 py-1.5">SKU</th>
              <th class="border px-2 py-1.5">规格</th>
              <th class="border px-2 py-1.5">单位</th>
              <th class="border px-2 py-1.5">数量</th>
              <th class="border px-2 py-1.5">预估单价</th>
              <th class="border px-2 py-1.5">预估金额</th>
              <th class="border px-2 py-1.5">备注</th>
              <th class="border px-2 py-1.5">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, index) in items" :key="index">
              <td class="border px-2 py-1">
                <input
                  v-model="item.productName"
                  class="w-full border rounded px-2 py-1 text-sm"
                  placeholder="产品名称"
                />
              </td>
              <td class="border px-2 py-1">
                <input
                  v-model="item.sku"
                  class="w-full border rounded px-2 py-1 text-sm"
                  placeholder="SKU"
                />
              </td>
              <td class="border px-2 py-1">
                <input
                  v-model="item.spec"
                  class="w-full border rounded px-2 py-1 text-sm"
                  placeholder="规格"
                />
              </td>
              <td class="border px-2 py-1">
                <input
                  v-model="item.unit"
                  class="w-full border rounded px-2 py-1 text-sm"
                  placeholder="单位"
                />
              </td>
              <td class="border px-2 py-1">
                <input
                  v-model.number="item.quantity"
                  type="number"
                  min="0"
                  class="w-full border rounded px-2 py-1 text-sm"
                  placeholder="数量"
                  @input="updateItemAmount(item)"
                />
              </td>
              <td class="border px-2 py-1">
                <input
                  v-model.number="item.estimatedPrice"
                  type="number"
                  step="0.01"
                  min="0"
                  class="w-full border rounded px-2 py-1 text-sm"
                  placeholder="单价"
                  @input="updateItemAmount(item)"
                />
              </td>
              <td class="border px-2 py-1 text-sm">
                {{ item.estimatedAmount }}
              </td>
              <td class="border px-2 py-1">
                <input
                  v-model="item.remark"
                  class="w-full border rounded px-2 py-1 text-sm"
                  placeholder="备注"
                />
              </td>
              <td class="border px-2 py-1 text-center">
                <Button
                  type="link"
                  danger
                  size="small"
                  @click="removeItem(index)"
                >
                  删除
                </Button>
              </td>
            </tr>
            <tr v-if="items.length === 0">
              <td
                colspan="9"
                class="border px-4 py-8 text-center text-gray-400 text-sm"
              >
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
.requisition-drawer {
  width: 75vw !important;
}

.requisition-drawer--fullscreen {
  width: 100vw !important;
}

.requisition-drawer__fs-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin-right: 8px;
  color: rgb(0 0 0 / 45%);
  cursor: pointer;
  background: transparent;
  border: none;
  border-radius: 4px;
  transition: all 0.2s;
}

.requisition-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgb(0 0 0 / 6%);
}

.requisition-drawer__body {
  height: calc(100vh - 150px);
  padding: 0 8px;
  overflow-y: auto;
}

.requisition-drawer__table input {
  font-size: 12px;
}
</style>
