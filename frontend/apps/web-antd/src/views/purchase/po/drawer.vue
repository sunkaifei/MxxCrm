<script lang="ts" setup>import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { createPurchaseOrderApi, updatePurchaseOrderApi } from '#/api';
import { statusList } from '#/store';
const data = ref();
const items = ref<any[]>([]);
const getTitle = computed(() => data.value?.create
 ? $t('ui.modal.create', { moduleName: $t('page.purchase.po.title') })
 : $t('ui.modal.update', { moduleName: $t('page.purchase.po.title') }));
const [BaseForm, baseFormApi] = useVbenForm({
 showDefaultActions: false,
 commonConfig: {
 componentProps: {
 class: 'w-full',
 },
 },
 schema: [
 {
 component: 'Input',
 fieldName: 'purchaseNo',
 label: '采购单号',
 rules: 'required',
 componentProps: {
 placeholder: $t('ui.placeholder.input'),
 allowClear: true,
 },
 },
 {
 component: 'Input',
 fieldName: 'supplierId',
 label: '供应商ID',
 componentProps: {
 placeholder: $t('ui.placeholder.input'),
 allowClear: true,
 },
 },
 {
 component: 'Input',
 fieldName: 'amount',
 label: '采购金额',
 componentProps: {
 placeholder: $t('ui.placeholder.input'),
 allowClear: true,
 },
 },
 {
 component: 'RadioGroup',
 fieldName: 'status',
 defaultValue: 'draft',
 label: $t('ui.table.status'),
 rules: 'selectRequired',
 componentProps: {
 optionType: 'button',
 class: 'flex flex-wrap',
 options: statusList,
 },
 },
 ],
});
const [Drawer, drawerApi] = useVbenDrawer({
 onCancel() {
 drawerApi.close();
 },
 async onConfirm() {
 const validate = await baseFormApi.validate();
 if (!validate.valid) {
 return;
 }
 setLoading(true);
 const values = await baseFormApi.getValues();
 try {
 await (data.value?.create
 ? createPurchaseOrderApi({ ...values, items: items.value })
 : updatePurchaseOrderApi({ ...values, id: data.value.row.id, items: items.value }));
 message.success(data.value?.create
 ? $t('ui.notification.create_success')
 : $t('ui.notification.update_success'));
 drawerApi.setData({ needRefresh: true });
 }
 finally {
 drawerApi.close();
 setLoading(false);
 }
 },
 onOpenChange(isOpen) {
 if (isOpen) {
 data.value = drawerApi.getData<Record<string, any>>();
 const row = data.value?.row ? { ...data.value.row } : {};
 baseFormApi.setValues(row);
 items.value = data.value?.row?.items || [];
 setLoading(false);
 }
 },
});
function setLoading(loading: boolean) {
 drawerApi.setState({ loading });
}
function addItem() {
 items.value.push({
 productName: '',
 sku: '',
 quantity: 1,
 unitPrice: 0,
 discount: 0,
 taxRate: 0,
 totalAmount: 0,
 });
}
function removeItem(index: number) {
 items.value.splice(index, 1);
}
function updateSubtotal(item: any) {
 const quantity = parseFloat(item.quantity) || 0;
 const unitPrice = parseFloat(item.unitPrice) || 0;
 const discount = parseFloat(item.discount) || 0;
 const taxRate = parseFloat(item.taxRate) || 0;
 const subtotal = quantity * unitPrice * (1 - discount / 100);
 const taxAmount = subtotal * (taxRate / 100);
 item.totalAmount = (subtotal + taxAmount).toFixed(2);
}
</script>

<template>
  <Drawer :title="getTitle">
    <BaseForm />
    <div class="mt-4">
      <div class="flex justify-between items-center mb-3">
        <h3 class="text-lg font-semibold">采购明细</h3>
        <button class="btn btn-primary" @click="addItem">
          添加明细
        </button>
      </div>
      <table class="w-full border-collapse">
        <thead>
          <tr>
            <th class="border px-4 py-2">产品名称</th>
            <th class="border px-4 py-2">SKU</th>
            <th class="border px-4 py-2">数量</th>
            <th class="border px-4 py-2">单价</th>
            <th class="border px-4 py-2">折扣(%)</th>
            <th class="border px-4 py-2">税率(%)</th>
            <th class="border px-4 py-2">小计</th>
            <th class="border px-4 py-2">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(item, index) in items" :key="index">
            <td class="border px-4 py-2">
              <input
                v-model="item.productName"
                class="w-full border rounded px-2 py-1"
                placeholder="产品名称"
              />
            </td>
            <td class="border px-4 py-2">
              <input
                v-model="item.sku"
                class="w-full border rounded px-2 py-1"
                placeholder="SKU"
              />
            </td>
            <td class="border px-4 py-2">
              <input
                v-model.number="item.quantity"
                type="number"
                class="w-full border rounded px-2 py-1"
                placeholder="数量"
                @input="updateSubtotal(item)"
              />
            </td>
            <td class="border px-4 py-2">
              <input
                v-model.number="item.unitPrice"
                type="number"
                step="0.01"
                class="w-full border rounded px-2 py-1"
                placeholder="单价"
                @input="updateSubtotal(item)"
              />
            </td>
            <td class="border px-4 py-2">
              <input
                v-model.number="item.discount"
                type="number"
                class="w-full border rounded px-2 py-1"
                placeholder="折扣"
                @input="updateSubtotal(item)"
              />
            </td>
            <td class="border px-4 py-2">
              <input
                v-model.number="item.taxRate"
                type="number"
                class="w-full border rounded px-2 py-1"
                placeholder="税率"
                @input="updateSubtotal(item)"
              />
            </td>
            <td class="border px-4 py-2">{{ item.totalAmount }}</td>
            <td class="border px-4 py-2">
              <button class="text-red-500" @click="removeItem(index)">删除</button>
            </td>
          </tr>
          <tr v-if="items.length === 0">
            <td colspan="8" class="border px-4 py-8 text-center">
              暂无采购明细，点击上方按钮添加
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </Drawer>
</template>