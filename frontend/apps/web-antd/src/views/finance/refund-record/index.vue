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
  createRefundRecordApi,
  deleteRefundRecordApi,
  getRefundRecordListApi,
  updateRefundRecordApi,
} from '#/api';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import { UserPickerModal } from '#/components/UserPickerModal';
import { $t } from '#/locales';

// 退款记录使用说明步骤数（与 i18n 中 page.finance.refundRecord.guide.steps 数组对齐）
const guideStepCount = 5;

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  schema: [
    {
      component: 'InputNumber',
      fieldName: 'userId',
      label: $t('page.finance.refundRecord.column.userId'),
      componentProps: {
        placeholder: $t('page.finance.refundRecord.placeholder.userId'),
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.finance.refundRecord.column.status'),
      componentProps: {
        placeholder: $t('page.finance.common.all'),
        allowClear: true,
        options: [
          { value: 0, label: $t('page.finance.refundRecord.status.pending') },
          { value: 1, label: $t('page.finance.refundRecord.status.success') },
          { value: 2, label: $t('page.finance.refundRecord.status.failed') },
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
        const result: any = await getRefundRecordListApi(params);
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          if (items.length === 0) {
            gridEl.style.setProperty('height', '280px', 'important');
          } else {
            gridEl.style.removeProperty('height');
          }
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
      title: $t('page.finance.refundRecord.column.userId'),
      width: 100,
    },
    {
      field: 'paymentRecordId',
      title: $t('page.finance.refundRecord.column.paymentRecordId'),
      width: 140,
    },
    {
      field: 'amount',
      title: $t('page.finance.refundRecord.column.amount'),
      width: 120,
      align: 'right',
      formatter: ({ cellValue }) => `¥${Number(cellValue || 0).toFixed(2)}`,
    },
    {
      field: 'status',
      title: $t('page.finance.refundRecord.column.status'),
      width: 100,
      slots: { default: 'status' },
    },
    {
      field: 'transactionId',
      title: $t('page.finance.refundRecord.column.transactionId'),
      width: 160,
    },
    {
      field: 'refundTime',
      title: $t('page.finance.refundRecord.column.refundTime'),
      width: 160,
    },
    {
      field: 'reason',
      title: $t('page.finance.refundRecord.column.reason'),
      minWidth: 160,
    },
    {
      field: 'remark',
      title: $t('page.finance.refundRecord.column.remark'),
      minWidth: 120,
    },
    {
      field: 'createTime',
      title: $t('page.finance.refundRecord.column.createTime'),
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
  paymentRecordId: undefined as number | undefined,
  amount: 0,
  status: 0,
  transactionId: '',
  refundTime: '',
  reason: '',
  remark: '',
});

function resetForm() {
  formData.id = 0;
  formData.userId = undefined;
  formData.paymentRecordId = undefined;
  formData.amount = 0;
  formData.status = 0;
  formData.transactionId = '';
  formData.refundTime = '';
  formData.reason = '';
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
  formData.paymentRecordId = row.paymentRecordId;
  formData.amount = row.amount;
  formData.status = row.status ?? 0;
  formData.transactionId = row.transactionId ?? '';
  formData.refundTime = row.refundTime ?? '';
  formData.reason = row.reason ?? '';
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

  if (!formData.userId || !formData.paymentRecordId) {
    message.warning($t('page.finance.refundRecord.message.required'));
    return;
  }

  const payload: any = {
    userId: formData.userId,
    paymentRecordId: formData.paymentRecordId,
    amount: formData.amount,
    status: formData.status,
    transactionId: formData.transactionId || undefined,
    refundTime: formData.refundTime || undefined,
    reason: formData.reason || undefined,
    remark: formData.remark || undefined,
  };

  drawerLoading.value = true;
  try {
    if (drawerMode.value === 'create') {
      await createRefundRecordApi(payload);
      message.success($t('page.finance.refundRecord.message.createSuccess'));
    } else {
      await updateRefundRecordApi(formData.id, payload);
      message.success($t('page.finance.refundRecord.message.updateSuccess'));
    }
    drawerVisible.value = false;
    gridApi.query();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.refundRecord.message.saveFailed'),
    );
  } finally {
    drawerLoading.value = false;
  }
}

async function handleDelete(row: any) {
  try {
    await deleteRefundRecordApi(row.id);
    message.success($t('page.finance.refundRecord.message.deleteSuccess'));
    gridApi.query();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.refundRecord.message.deleteFailed'),
    );
  }
}

const statusMap: Record<number, { color: string; text: string }> = {
  0: { color: 'orange', text: $t('page.finance.refundRecord.status.pending') },
  1: { color: 'green', text: $t('page.finance.refundRecord.status.success') },
  2: { color: 'red', text: $t('page.finance.refundRecord.status.failed') },
};
</script>

<template>
  <Page>
    <PageUsageGuide
      :title="$t('page.finance.refundRecord.guide.title')"
      :brief="$t('page.finance.refundRecord.guide.brief')"
      :expand-text="$t('page.finance.refundRecord.guide.expand')"
      :collapse-text="$t('page.finance.refundRecord.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.refundRecord.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.refundRecord.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Grid>
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" @click="openCreate">
          <template #icon><Plus /></template>
          {{ $t('page.finance.refundRecord.button.add') }}
        </Button>
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
          :title="$t('page.finance.refundRecord.message.deleteConfirm')"
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
          ? $t('page.finance.refundRecord.drawer.titleCreate')
          : $t('page.finance.refundRecord.drawer.titleEdit')
      "
      width="480"
      :confirm-loading="drawerLoading"
      @ok="handleSubmit"
    >
      <Form ref="formRef" layout="vertical" class="pt-4">
        <FormItem
          :label="$t('page.finance.refundRecord.column.userId')"
          name="userId"
          required
        >
          <UserPickerModal v-model:value="formData.userId" />
        </FormItem>
        <FormItem
          :label="$t('page.finance.refundRecord.column.paymentRecordId')"
          name="paymentRecordId"
          required
        >
          <InputNumber
            v-model:value="formData.paymentRecordId"
            :placeholder="
              $t('page.finance.refundRecord.placeholder.paymentRecordId')
            "
            :min="1"
            style="width: 100%"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.refundRecord.column.amount')"
          name="amount"
          required
        >
          <InputNumber
            v-model:value="formData.amount"
            :min="0"
            :precision="2"
            :placeholder="$t('page.finance.refundRecord.placeholder.amount')"
            style="width: 100%"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.refundRecord.column.status')"
          name="status"
        >
          <Select
            v-model:value="formData.status"
            :options="[
              {
                value: 0,
                label: $t('page.finance.refundRecord.status.pending'),
              },
              {
                value: 1,
                label: $t('page.finance.refundRecord.status.success'),
              },
              {
                value: 2,
                label: $t('page.finance.refundRecord.status.failed'),
              },
            ]"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.refundRecord.column.transactionId')"
          name="transactionId"
        >
          <Input
            v-model:value="formData.transactionId"
            :placeholder="
              $t('page.finance.refundRecord.placeholder.transactionId')
            "
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.refundRecord.column.refundTime')"
          name="refundTime"
        >
          <Input
            v-model:value="formData.refundTime"
            :placeholder="
              $t('page.finance.refundRecord.placeholder.refundTime')
            "
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.refundRecord.column.reason')"
          name="reason"
        >
          <Input.TextArea
            v-model:value="formData.reason"
            :rows="2"
            :placeholder="$t('page.finance.refundRecord.placeholder.reason')"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.refundRecord.column.remark')"
          name="remark"
        >
          <Input.TextArea
            v-model:value="formData.remark"
            :rows="2"
            :placeholder="$t('page.finance.refundRecord.placeholder.remark')"
          />
        </FormItem>
      </Form>
    </Drawer>
  </Page>
</template>
