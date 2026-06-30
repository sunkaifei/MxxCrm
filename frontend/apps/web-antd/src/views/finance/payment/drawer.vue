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

import { applyPaymentApi } from '#/api/core/finance';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'close', needRefresh?: boolean): void;
}>();

const formRef = ref();
const loading = ref(false);

const paymentTypeOptions = [
  { value: 1, label: '预付款' },
  { value: 2, label: '尾款' },
  { value: 3, label: '全款' },
];

const paymentMethodOptions = [
  { value: 1, label: '银行转账' },
  { value: 2, label: '现金' },
  { value: 3, label: '支票' },
  { value: 4, label: '其他' },
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
    await applyPaymentApi(formData);
    message.success('申请成功');
    emit('close', true);
  } catch (e: any) {
    message.error(e?.message || '申请失败');
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
    title="申请付款"
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
        label="关联采购单"
        :rules="[{ required: true, message: '请输入关联采购单号' }]"
      >
        <Input
          v-model:value="formData.poNo"
          placeholder="请输入关联采购单号"
          allow-clear
        />
      </FormItem>

      <FormItem
        name="paymentType"
        label="付款类型"
        :rules="[{ required: true, message: '请选择付款类型' }]"
      >
        <Select
          v-model:value="formData.paymentType"
          :options="paymentTypeOptions"
          placeholder="请选择付款类型"
        />
      </FormItem>

      <FormItem
        name="amount"
        label="付款金额"
        :rules="[{ required: true, message: '请输入付款金额' }]"
      >
        <InputNumber
          v-model:value="formData.amount"
          :min="0"
          :precision="2"
          style="width: 100%"
          placeholder="请输入付款金额"
        />
      </FormItem>

      <FormItem
        name="paymentMethod"
        label="付款方式"
        :rules="[{ required: true, message: '请选择付款方式' }]"
      >
        <Select
          v-model:value="formData.paymentMethod"
          :options="paymentMethodOptions"
          placeholder="请选择付款方式"
        />
      </FormItem>

      <FormItem name="paymentAccount" label="付款账户">
        <Input
          v-model:value="formData.paymentAccount"
          placeholder="请输入付款账户"
          allow-clear
        />
      </FormItem>

      <FormItem name="remark" label="备注">
        <Textarea
          v-model:value="formData.remark"
          :rows="3"
          placeholder="请输入备注"
          allow-clear
        />
      </FormItem>
    </Form>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button @click="handleClose">取消</Button>
        <Button type="primary" :loading="loading" @click="handleSubmit">
          提交
        </Button>
      </div>
    </template>
  </Drawer>
</template>
