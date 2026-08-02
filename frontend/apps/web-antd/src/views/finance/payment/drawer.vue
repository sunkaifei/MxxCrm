<script lang="ts" setup>
import { reactive, ref, watch } from 'vue';

import {
  Button,
  Drawer,
  Form,
  FormItem,
  Input,
  InputNumber,
  Select,
  Textarea,
  message,
} from 'ant-design-vue';

import { applyFinancePaymentApi } from '#/api/core/finance';
import { $t } from '#/locales';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'close', needRefresh?: boolean): void;
}>();

const formRef = ref();
const loading = ref(false);

const paymentTypeOptions = [
  { value: 1, label: $t('page.finance.payment.paymentType.prepay') },
  { value: 2, label: $t('page.finance.payment.paymentType.final') },
  { value: 3, label: $t('page.finance.payment.paymentType.full') },
];

const paymentMethodOptions = [
  { value: 1, label: $t('page.finance.payment.paymentMethod.bankTransfer') },
  { value: 2, label: $t('page.finance.payment.paymentMethod.cash') },
  { value: 3, label: $t('page.finance.payment.paymentMethod.check') },
  { value: 4, label: $t('page.finance.payment.paymentMethod.other') },
];

const formData = reactive<any>({
  poNo: '',
  paymentType: 1,
  amount: 0,
  paymentMethod: 1,
  paymentAccount: '',
  remark: '',
});

function resetForm() {
  formData.poNo = '';
  formData.paymentType = 1;
  formData.amount = 0;
  formData.paymentMethod = 1;
  formData.paymentAccount = '';
  formData.remark = '';
  formRef.value?.resetFields();
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      resetForm();
    }
  },
);

async function handleSubmit() {
  try {
    await formRef.value.validate();
  } catch {
    return;
  }
  loading.value = true;
  try {
    await applyFinancePaymentApi(formData);
    message.success($t('page.finance.payment.drawer.applySuccess'));
    emit('close', true);
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.payment.drawer.applyFailed'));
  } finally {
    loading.value = false;
  }
}

function handleClose() {
  emit('close');
}
</script>

<template>
  <Drawer
    :open="visible"
    :title="$t('page.finance.payment.drawer.titleApply')"
    :width="560"
    :mask-closable="false"
    :destroy-on-close="true"
    @close="handleClose"
  >
    <Form
      ref="formRef"
      :model="formData"
      :label-col="{ span: 6 }"
      :wrapper-col="{ span: 18 }"
    >
      <FormItem
        name="poNo"
        :label="$t('page.finance.payment.drawer.purchaseOrderNo')"
        :rules="[{ required: true, message: $t('page.finance.payment.drawer.purchaseOrderRequired') }]"
      >
        <Input
          v-model:value="formData.poNo"
          :placeholder="$t('page.finance.payment.drawer.purchaseOrderPlaceholder')"
          allow-clear
        />
      </FormItem>

      <FormItem
        name="paymentType"
        :label="$t('page.finance.payment.drawer.paymentType')"
        :rules="[{ required: true, message: $t('page.finance.payment.drawer.paymentTypeRequired') }]"
      >
        <Select
          v-model:value="formData.paymentType"
          :options="paymentTypeOptions"
          :placeholder="$t('page.finance.payment.drawer.paymentTypePlaceholder')"
        />
      </FormItem>

      <FormItem
        name="amount"
        :label="$t('page.finance.payment.drawer.paymentAmount')"
        :rules="[{ required: true, message: $t('page.finance.payment.drawer.paymentAmountRequired') }]"
      >
        <InputNumber
          v-model:value="formData.amount"
          :min="0"
          :precision="2"
          style="width: 100%"
          :placeholder="$t('page.finance.payment.drawer.paymentAmountPlaceholder')"
        />
      </FormItem>

      <FormItem
        name="paymentMethod"
        :label="$t('page.finance.payment.drawer.paymentMethod')"
        :rules="[{ required: true, message: $t('page.finance.payment.drawer.paymentMethodRequired') }]"
      >
        <Select
          v-model:value="formData.paymentMethod"
          :options="paymentMethodOptions"
          :placeholder="$t('page.finance.payment.drawer.paymentMethodPlaceholder')"
        />
      </FormItem>

      <FormItem name="paymentAccount" :label="$t('page.finance.payment.drawer.bankAccount')">
        <Input
          v-model:value="formData.paymentAccount"
          :placeholder="$t('page.finance.payment.drawer.bankAccountPlaceholder')"
          allow-clear
        />
      </FormItem>

      <FormItem name="remark" :label="$t('page.finance.payment.drawer.remark')">
        <Textarea
          v-model:value="formData.remark"
          :rows="3"
          :placeholder="$t('page.finance.payment.drawer.remarkPlaceholder')"
          allow-clear
        />
      </FormItem>
    </Form>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button @click="handleClose">{{ $t('page.finance.common.cancel') }}</Button>
        <Button type="primary" :loading="loading" @click="handleSubmit">
          {{ $t('page.finance.common.submit') }}
        </Button>
      </div>
    </template>
  </Drawer>
</template>
