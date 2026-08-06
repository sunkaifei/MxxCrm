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
 label: $t('page.purchase.po.form.purchaseNo'),
 rules: 'required',
 componentProps: {
 placeholder: $t('ui.placeholder.input'),
 allowClear: true,
 },
 },
 {
 component: 'Input',
 fieldName: 'supplierId',
 label: $t('page.purchase.po.form.supplierId'),
 componentProps: {
 placeholder: $t('ui.placeholder.input'),
 allowClear: true,
 },
 },
 {
 component: 'Input',
 fieldName: 'amount',
 label: $t('page.purchase.po.form.amount'),
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
        <h3 class="text-lg font-semibold">{{ $t('page.purchase.po.item.title') }}</h3>
        <button class="btn btn-primary" @click="addItem">
          {{ $t('page.purchase.po.item.addItem') }}
        </button>
      </div>
      <table class="w-full border-collapse">
        <thead>
          <tr>
            <th class="border px-4 py-2">{{ $t('page.purchase.po.item.column.productName') }}</th>
            <th class="border px-4 py-2">{{ $t('page.purchase.po.item.column.sku') }}</th>
            <th class="border px-4 py-2">{{ $t('page.purchase.po.item.column.quantity') }}</th>
            <th class="border px-4 py-2">{{ $t('page.purchase.po.item.column.unitPrice') }}</th>
            <th class="border px-4 py-2">{{ $t('page.purchase.po.item.column.discount') }}</th>
            <th class="border px-4 py-2">{{ $t('page.purchase.po.item.column.taxRate') }}</th>
            <th class="border px-4 py-2">{{ $t('page.purchase.po.item.column.subtotal') }}</th>
            <th class="border px-4 py-2">{{ $t('page.purchase.po.item.column.action') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(item, index) in items" :key="index">
            <td class="border px-4 py-2">
              <input
                v-model="item.productName"
                class="w-full border rounded px-2 py-1"
                :placeholder="$t('page.purchase.po.item.placeholder.productName')"
              />
            </td>
            <td class="border px-4 py-2">
              <input
                v-model="item.sku"
                class="w-full border rounded px-2 py-1"
                :placeholder="$t('page.purchase.po.item.placeholder.sku')"
              />
            </td>
            <td class="border px-4 py-2">
              <input
                v-model.number="item.quantity"
                type="number"
                class="w-full border rounded px-2 py-1"
                :placeholder="$t('page.purchase.po.item.placeholder.quantity')"
                @input="updateSubtotal(item)"
              />
            </td>
            <td class="border px-4 py-2">
              <input
                v-model.number="item.unitPrice"
                type="number"
                step="0.01"
                class="w-full border rounded px-2 py-1"
                :placeholder="$t('page.purchase.po.item.placeholder.unitPrice')"
                @input="updateSubtotal(item)"
              />
            </td>
            <td class="border px-4 py-2">
              <input
                v-model.number="item.discount"
                type="number"
                class="w-full border rounded px-2 py-1"
                :placeholder="$t('page.purchase.po.item.placeholder.discount')"
                @input="updateSubtotal(item)"
              />
            </td>
            <td class="border px-4 py-2">
              <input
                v-model.number="item.taxRate"
                type="number"
                class="w-full border rounded px-2 py-1"
                :placeholder="$t('page.purchase.po.item.placeholder.taxRate')"
                @input="updateSubtotal(item)"
              />
            </td>
            <td class="border px-4 py-2">{{ item.totalAmount }}</td>
            <td class="border px-4 py-2">
              <button class="text-red-500" @click="removeItem(index)">{{ $t('page.purchase.po.item.delete') }}</button>
            </td>
          </tr>
          <tr v-if="items.length === 0">
            <td colspan="8" class="border px-4 py-8 text-center">
              {{ $t('page.purchase.po.item.empty') }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </Drawer>
</template>