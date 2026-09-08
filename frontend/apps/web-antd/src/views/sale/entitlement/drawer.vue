<script lang="ts" setup>
import { reactive, ref, watch } from 'vue';

import { message, Drawer, Form, FormItem, Input, InputNumber, Select, Textarea, DatePicker } from 'ant-design-vue';
import dayjs from 'dayjs';

import { createEntitlementApi } from '#/api/core/sale/entitlement';
import { getCustomerListApi } from '#/api/core/crm/customer';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'success'): void;
  (e: 'update:visible', v: boolean): void;
}>();

const submitting = ref(false);
const custOptions = ref<{ label: string; value: number }[]>([]);
const custLoading = ref(false);

async function onCustSearch(kw: string) {
  custLoading.value = true;
  try {
    const res: any = await getCustomerListApi({
      page: 1,
      pageSize: 20,
      ...(kw ? { companyName: kw } : {}),
    });
    custOptions.value = (res?.items || []).map((c: any) => ({
      label: c.companyName || c.shortName || String(c.id),
      value: Number(c.id),
    }));
  } finally {
    custLoading.value = false;
  }
}

const formData = reactive({
  customerId: undefined as number | undefined,
  orderId: undefined as number | undefined,
  productId: undefined as number | undefined,
  productName: '',
  entitlementType: 1 as number,
  startDate: dayjs().format('YYYY-MM-DD'),
  durationMonths: 12 as number,
  autoRenew: 0 as number,
  totalQuota: undefined as number | undefined,
  slaLevel: '',
  remark: '',
});

watch(
  () => props.visible,
  (v) => {
    if (v) {
      formData.customerId = undefined;
      formData.orderId = undefined;
      formData.productId = undefined;
      formData.productName = '';
      formData.entitlementType = 1;
      formData.startDate = dayjs().format('YYYY-MM-DD');
      formData.durationMonths = 12;
      formData.autoRenew = 0;
      formData.totalQuota = undefined;
      formData.slaLevel = '';
      formData.remark = '';
    }
  },
);

function close() {
  emit('update:visible', false);
}

async function handleSubmit() {
  if (!formData.customerId) {
    message.warning('请填写客户 ID');
    return;
  }
  if (!formData.productName) {
    message.warning('请填写商品名称');
    return;
  }
  submitting.value = true;
  try {
    await createEntitlementApi({
      customerId: formData.customerId,
      orderId: formData.orderId,
      productId: formData.productId,
      productName: formData.productName,
      entitlementType: formData.entitlementType,
      startDate: formData.startDate,
      durationMonths: formData.durationMonths,
      endDate: dayjs(formData.startDate)
        .add(formData.durationMonths, 'month')
        .format('YYYY-MM-DD'),
      autoRenew: formData.autoRenew,
      totalQuota: formData.totalQuota,
      slaLevel: formData.slaLevel || undefined,
      remark: formData.remark || undefined,
    });
    emit('success');
  } catch {
    // 全局拦截器处理
  } finally {
    submitting.value = false;
  }
}

</script>

<template>
  <Drawer
    :open="visible"
    title="新建权益"
    width="520px"
    :destroy-on-close="true"
    @close="close"
  >
    <Form layout="vertical">
      <FormItem label="客户" required>
        <Select
          v-model:value="formData.customerId"
          class="w-full"
          show-search
          :filter-option="false"
          :options="custOptions"
          :loading="custLoading"
          @search="onCustSearch"
          placeholder="搜索并选择客户"
        />
      </FormItem>
      <FormItem label="关联订单 ID（选填）">
        <InputNumber
          v-model:value="formData.orderId"
          class="w-full"
          placeholder="订单 ID（订单生成的权益仅管理员可删）"
          :controls="false"
        />
      </FormItem>
      <FormItem label="产品 ID（选填）">
        <InputNumber
          v-model:value="formData.productId"
          class="w-full"
          placeholder="产品 ID"
          :controls="false"
        />
      </FormItem>
      <FormItem label="商品名称" required>
        <Input v-model:value="formData.productName" placeholder="请输入商品名称" />
      </FormItem>
      <FormItem label="权益类型" required>
        <Select
          v-model:value="formData.entitlementType"
          :options="[
            { label: '服务期', value: 1 },
            { label: '订阅周期', value: 2 },
            { label: '技术支持', value: 3 },
            { label: '资源包', value: 4 },
            { label: 'SLA', value: 5 },
          ]"
        />
      </FormItem>
      <FormItem label="开始日期" required>
        <DatePicker
          :value="dayjs(formData.startDate)"
          class="w-full"
          value-format="YYYY-MM-DD"
          @change="(d: any) => (formData.startDate = d ? dayjs(d).format('YYYY-MM-DD') : formData.startDate)"
        />
      </FormItem>
      <FormItem label="服务时长（月）">
        <InputNumber
          v-model:value="formData.durationMonths"
          class="w-full"
          :min="1"
          :controls="false"
        />
      </FormItem>
      <FormItem label="自动续约">
        <Select
          v-model:value="formData.autoRenew"
          :options="[
            { label: '否', value: 0 },
            { label: '是', value: 1 },
          ]"
        />
      </FormItem>
      <FormItem label="总配额（资源包类，选填）">
        <InputNumber
          v-model:value="formData.totalQuota"
          class="w-full"
          placeholder="如 100（次数/工时）"
          :controls="false"
        />
      </FormItem>
      <FormItem label="SLA 等级（选填）">
        <Input v-model:value="formData.slaLevel" placeholder="如：金牌 / 银牌" />
      </FormItem>
      <FormItem label="备注">
        <Textarea v-model:value="formData.remark" :rows="3" placeholder="备注" />
      </FormItem>
    </Form>
    <template #footer>
      <div class="flex justify-end gap-2">
        <a-button @click="close">取消</a-button>
        <a-button type="primary" :loading="submitting" @click="handleSubmit">
          提交
        </a-button>
      </div>
    </template>
  </Drawer>
</template>
