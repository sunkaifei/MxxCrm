<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';

import {
  Button,
  Drawer,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Popconfirm,
  Select,
  Tag,
} from 'ant-design-vue';
import { Plus } from 'lucide-vue-next';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  createPaymentRecordApi,
  deletePaymentRecordApi,
  getPaymentRecordListApi,
  updatePaymentRecordApi,
} from '#/api';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import { UserPickerModal } from '#/components/UserPickerModal';
import { $t } from '#/locales';

// 付款记录使用说明步骤数（与 i18n 中 page.finance.paymentRecord.guide.steps 数组对齐）
const guideStepCount = 5;

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  schema: [
    {
      component: 'InputNumber',
      fieldName: 'userId',
      label: $t('page.finance.paymentRecord.column.userId'),
      componentProps: {
        placeholder: $t('page.finance.paymentRecord.placeholder.userId'),
      },
    },
    {
      component: 'Select',
      fieldName: 'paymentType',
      label: $t('page.finance.paymentRecord.column.paymentType'),
      componentProps: {
        placeholder: $t('page.finance.common.all'),
        allowClear: true,
        options: [
          {
            value: 1,
            label: $t('page.finance.paymentRecord.paymentType.member'),
          },
          {
            value: 2,
            label: $t('page.finance.paymentRecord.paymentType.product'),
          },
          {
            value: 3,
            label: $t('page.finance.paymentRecord.paymentType.recharge'),
          },
          {
            value: 4,
            label: $t('page.finance.paymentRecord.paymentType.other'),
          },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.finance.paymentRecord.column.status'),
      componentProps: {
        placeholder: $t('page.finance.common.all'),
        allowClear: true,
        options: [
          { value: 0, label: $t('page.finance.paymentRecord.status.pending') },
          { value: 1, label: $t('page.finance.paymentRecord.status.success') },
          { value: 2, label: $t('page.finance.paymentRecord.status.failed') },
          { value: 3, label: $t('page.finance.paymentRecord.status.refunded') },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  height: 'auto',
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        };
        const result: any = await getPaymentRecordListApi(params);
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '280px' : '';
        }
        return result;
      },
    },
  },
  columns: [
    { type: 'seq', width: 60, title: $t('page.finance.common.seq') },
    { field: 'id', title: 'ID', width: 80 },
    {
      field: 'userId',
      title: $t('page.finance.paymentRecord.column.userId'),
      width: 100,
    },
    {
      field: 'orderId',
      title: $t('page.finance.paymentRecord.column.orderId'),
      width: 160,
    },
    {
      field: 'paymentType',
      title: $t('page.finance.paymentRecord.column.paymentType'),
      width: 120,
      slots: { default: 'paymentType' },
    },
    {
      field: 'amount',
      title: $t('page.finance.paymentRecord.column.amount'),
      width: 120,
      align: 'right',
      formatter: ({ cellValue }) => `¥${Number(cellValue || 0).toFixed(2)}`,
    },
    {
      field: 'payMethod',
      title: $t('page.finance.paymentRecord.column.payMethod'),
      width: 120,
      slots: { default: 'payMethod' },
    },
    {
      field: 'status',
      title: $t('page.finance.paymentRecord.column.status'),
      width: 100,
      slots: { default: 'status' },
    },
    {
      field: 'transactionId',
      title: $t('page.finance.paymentRecord.column.transactionId'),
      width: 160,
    },
    {
      field: 'payTime',
      title: $t('page.finance.paymentRecord.column.payTime'),
      width: 160,
    },
    {
      field: 'remark',
      title: $t('page.finance.paymentRecord.column.remark'),
      minWidth: 120,
    },
    {
      field: 'createTime',
      title: $t('page.finance.paymentRecord.column.createTime'),
      width: 160,
    },
    {
      field: 'action',
      title: $t('page.finance.common.action'),
      width: 160,
      fixed: 'right',
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ formOptions, gridOptions });

// ===== Drawer 表单 =====
const drawerVisible = ref(false);
const drawerMode = ref<'create' | 'edit'>('create');
const drawerLoading = ref(false);
const formRef = ref();
const formData = reactive({
  id: 0,
  userId: undefined as number | undefined,
  memberProductId: undefined as number | undefined,
  orderId: '',
  paymentType: 1,
  amount: 0,
  payMethod: 1,
  status: 0,
  transactionId: '',
  payTime: '',
  remark: '',
});

function resetForm() {
  formData.id = 0;
  formData.userId = undefined;
  formData.memberProductId = undefined;
  formData.orderId = '';
  formData.paymentType = 1;
  formData.amount = 0;
  formData.payMethod = 1;
  formData.status = 0;
  formData.transactionId = '';
  formData.payTime = '';
  formData.remark = '';
}

function openCreate() {
  resetForm();
  drawerMode.value = 'create';
  drawerVisible.value = true;
}

function openEdit(row: any) {
  formData.id = row.id;
  formData.userId = row.userId;
  formData.memberProductId = row.memberProductId;
  formData.orderId = row.orderId ?? '';
  formData.paymentType = row.paymentType ?? 1;
  formData.amount = row.amount;
  formData.payMethod = row.payMethod ?? 1;
  formData.status = row.status ?? 0;
  formData.transactionId = row.transactionId ?? '';
  formData.payTime = row.payTime ?? '';
  formData.remark = row.remark ?? '';
  drawerMode.value = 'edit';
  drawerVisible.value = true;
}

async function handleSubmit() {
  try {
    await formRef.value?.validate();
  } catch {
    return;
  }

  if (!formData.userId) {
    message.warning($t('page.finance.paymentRecord.message.userIdRequired'));
    return;
  }

  const payload: any = {
    userId: formData.userId,
    memberProductId: formData.memberProductId || undefined,
    orderId: formData.orderId || undefined,
    paymentType: formData.paymentType,
    amount: formData.amount,
    payMethod: formData.payMethod,
    status: formData.status,
    transactionId: formData.transactionId || undefined,
    payTime: formData.payTime || undefined,
    remark: formData.remark || undefined,
  };

  drawerLoading.value = true;
  try {
    if (drawerMode.value === 'create') {
      await createPaymentRecordApi(payload);
      message.success($t('page.finance.paymentRecord.message.createSuccess'));
    } else {
      await updatePaymentRecordApi(formData.id, payload);
      message.success($t('page.finance.paymentRecord.message.updateSuccess'));
    }
    drawerVisible.value = false;
    gridApi.query();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.paymentRecord.message.saveFailed'),
    );
  } finally {
    drawerLoading.value = false;
  }
}

async function handleDelete(row: any) {
  try {
    await deletePaymentRecordApi(row.id);
    message.success($t('page.finance.paymentRecord.message.deleteSuccess'));
    gridApi.query();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.paymentRecord.message.deleteFailed'),
    );
  }
}

const paymentTypeMap: Record<number, { color: string; text: string }> = {
  1: {
    color: 'blue',
    text: $t('page.finance.paymentRecord.paymentType.member'),
  },
  2: {
    color: 'cyan',
    text: $t('page.finance.paymentRecord.paymentType.product'),
  },
  3: {
    color: 'purple',
    text: $t('page.finance.paymentRecord.paymentType.recharge'),
  },
  4: {
    color: 'default',
    text: $t('page.finance.paymentRecord.paymentType.other'),
  },
};
const payMethodMap: Record<number, { color: string; text: string }> = {
  1: {
    color: 'green',
    text: $t('page.finance.paymentRecord.payMethod.wechat'),
  },
  2: { color: 'blue', text: $t('page.finance.paymentRecord.payMethod.alipay') },
  3: { color: 'orange', text: $t('page.finance.paymentRecord.payMethod.bank') },
};
const statusMap: Record<number, { color: string; text: string }> = {
  0: { color: 'orange', text: $t('page.finance.paymentRecord.status.pending') },
  1: { color: 'green', text: $t('page.finance.paymentRecord.status.success') },
  2: { color: 'red', text: $t('page.finance.paymentRecord.status.failed') },
  3: {
    color: 'default',
    text: $t('page.finance.paymentRecord.status.refunded'),
  },
};
</script>

<template>
  <Page :title="$t('page.finance.paymentRecord.title')">
    <PageUsageGuide
      :title="$t('page.finance.paymentRecord.guide.title')"
      :brief="$t('page.finance.paymentRecord.guide.brief')"
      :expand-text="$t('page.finance.paymentRecord.guide.expand')"
      :collapse-text="$t('page.finance.paymentRecord.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.paymentRecord.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.paymentRecord.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Grid>
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" @click="openCreate">
          <template #icon><Plus /></template>
          {{ $t('page.finance.paymentRecord.button.add') }}
        </Button>
      </template>
      <template #paymentType="{ row }">
        <Tag :color="paymentTypeMap[row.paymentType]?.color || 'default'">
          {{ paymentTypeMap[row.paymentType]?.text || row.paymentType }}
        </Tag>
      </template>
      <template #payMethod="{ row }">
        <Tag :color="payMethodMap[row.payMethod]?.color || 'default'">
          {{ payMethodMap[row.payMethod]?.text || row.payMethod }}
        </Tag>
      </template>
      <template #status="{ row }">
        <Tag :color="statusMap[row.status]?.color || 'default'">
          {{ statusMap[row.status]?.text || row.status }}
        </Tag>
      </template>
      <template #action="{ row }">
        <Button type="link" size="small" @click="openEdit(row)">
          {{ $t('page.finance.common.edit') }}
        </Button>
        <Popconfirm
          :title="$t('page.finance.paymentRecord.message.deleteConfirm')"
          @confirm="handleDelete(row)"
        >
          <Button type="link" size="small" danger>
            {{ $t('page.finance.common.delete') }}
          </Button>
        </Popconfirm>
      </template>
    </Grid>

    <Drawer
      v-model:open="drawerVisible"
      :title="
        drawerMode === 'create'
          ? $t('page.finance.paymentRecord.drawer.titleCreate')
          : $t('page.finance.paymentRecord.drawer.titleEdit')
      "
      width="480"
      :confirm-loading="drawerLoading"
      @ok="handleSubmit"
    >
      <Form ref="formRef" layout="vertical" class="pt-4">
        <FormItem
          :label="$t('page.finance.paymentRecord.column.userId')"
          name="userId"
          required
        >
          <UserPickerModal v-model:value="formData.userId" />
        </FormItem>
        <FormItem
          :label="$t('page.finance.paymentRecord.column.memberProductId')"
          name="memberProductId"
        >
          <InputNumber
            v-model:value="formData.memberProductId"
            :min="1"
            :placeholder="
              $t('page.finance.paymentRecord.placeholder.memberProductId')
            "
            style="width: 100%"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.paymentRecord.column.orderId')"
          name="orderId"
        >
          <Input
            v-model:value="formData.orderId"
            :placeholder="$t('page.finance.paymentRecord.placeholder.orderId')"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.paymentRecord.column.paymentType')"
          name="paymentType"
        >
          <Select
            v-model:value="formData.paymentType"
            :options="[
              {
                value: 1,
                label: $t('page.finance.paymentRecord.paymentType.member'),
              },
              {
                value: 2,
                label: $t('page.finance.paymentRecord.paymentType.product'),
              },
              {
                value: 3,
                label: $t('page.finance.paymentRecord.paymentType.recharge'),
              },
              {
                value: 4,
                label: $t('page.finance.paymentRecord.paymentType.other'),
              },
            ]"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.paymentRecord.column.amount')"
          name="amount"
          required
        >
          <InputNumber
            v-model:value="formData.amount"
            :min="0"
            :precision="2"
            :placeholder="$t('page.finance.paymentRecord.placeholder.amount')"
            style="width: 100%"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.paymentRecord.column.payMethod')"
          name="payMethod"
        >
          <Select
            v-model:value="formData.payMethod"
            :options="[
              {
                value: 1,
                label: $t('page.finance.paymentRecord.payMethod.wechat'),
              },
              {
                value: 2,
                label: $t('page.finance.paymentRecord.payMethod.alipay'),
              },
              {
                value: 3,
                label: $t('page.finance.paymentRecord.payMethod.bank'),
              },
            ]"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.paymentRecord.column.status')"
          name="status"
        >
          <Select
            v-model:value="formData.status"
            :options="[
              {
                value: 0,
                label: $t('page.finance.paymentRecord.status.pending'),
              },
              {
                value: 1,
                label: $t('page.finance.paymentRecord.status.success'),
              },
              {
                value: 2,
                label: $t('page.finance.paymentRecord.status.failed'),
              },
              {
                value: 3,
                label: $t('page.finance.paymentRecord.status.refunded'),
              },
            ]"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.paymentRecord.column.transactionId')"
          name="transactionId"
        >
          <Input
            v-model:value="formData.transactionId"
            :placeholder="
              $t('page.finance.paymentRecord.placeholder.transactionId')
            "
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.paymentRecord.column.payTime')"
          name="payTime"
        >
          <Input
            v-model:value="formData.payTime"
            :placeholder="$t('page.finance.paymentRecord.placeholder.payTime')"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.paymentRecord.column.remark')"
          name="remark"
        >
          <Input.TextArea
            v-model:value="formData.remark"
            :rows="2"
            :placeholder="$t('page.finance.paymentRecord.placeholder.remark')"
          />
        </FormItem>
      </Form>
    </Drawer>
  </Page>
</template>
