<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  DatePicker,
  Form,
  FormItem,
  message,
  Modal,
  Tag,
  Textarea,
} from 'ant-design-vue';
import { RefreshCw } from 'lucide-vue-next';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  approveFinancePaymentApi,
  cancelFinancePaymentApi,
  confirmFinancePaymentApi,
  getFinancePaymentListApi,
} from '#/api/core/finance';
import { $t } from '#/locales';

import PaymentDrawer from './drawer.vue';

const drawerVisible = ref(false);

const paymentTypeMap: Record<number, { color: string; label: string }> = {
  1: { label: $t('page.finance.payment.paymentType.prepay'), color: 'blue' },
  2: { label: $t('page.finance.payment.paymentType.final'), color: 'orange' },
  3: { label: $t('page.finance.payment.paymentType.full'), color: 'green' },
};

const paymentMethodMap: Record<number, { color: string; label: string }> = {
  1: {
    label: $t('page.finance.payment.paymentMethod.bankTransfer'),
    color: 'blue',
  },
  2: { label: $t('page.finance.payment.paymentMethod.cash'), color: 'green' },
  3: { label: $t('page.finance.payment.paymentMethod.check'), color: 'orange' },
  4: {
    label: $t('page.finance.payment.paymentMethod.other'),
    color: 'default',
  },
};

const statusMap: Record<number, { color: string; label: string }> = {
  0: { label: $t('page.finance.payment.status.pending'), color: 'default' },
  1: { label: $t('page.finance.payment.status.approved'), color: 'processing' },
  2: { label: $t('page.finance.payment.status.paid'), color: 'green' },
  3: { label: $t('page.finance.payment.status.canceled'), color: 'red' },
};

const statusOptions = Object.entries(statusMap).map(([value, item]) => ({
  value: Number(value),
  label: item.label,
}));

function formatMoney(val: any) {
  if (val === null || val === undefined || val === '') return '-';
  return `¥${Number(val).toLocaleString()}`;
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'paymentNo',
      label: $t('page.finance.payment.column.paymentNo'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'supplierName',
      label: $t('page.finance.payment.column.supplierName'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.finance.payment.column.status'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: statusOptions,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getFinancePaymentListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: $t('page.finance.payment.column.paymentNo'),
      field: 'paymentNo',
      minWidth: 160,
    },
    {
      title: $t('page.finance.payment.column.purchaseOrderNo'),
      field: 'poNo',
      minWidth: 160,
    },
    {
      title: $t('page.finance.payment.column.supplierName'),
      field: 'supplierName',
      minWidth: 140,
    },
    {
      title: $t('page.finance.payment.column.paymentType'),
      field: 'paymentType',
      width: 110,
      slots: { default: 'paymentType' },
    },
    {
      title: $t('page.finance.payment.column.paymentAmount'),
      field: 'amount',
      width: 130,
      slots: { default: 'amount' },
    },
    {
      title: $t('page.finance.payment.column.paymentMethod'),
      field: 'paymentMethod',
      width: 120,
      slots: { default: 'paymentMethod' },
    },
    {
      title: $t('page.finance.payment.column.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('page.finance.payment.column.applyTime'),
      field: 'createTime',
      width: 170,
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 220,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function handleApply() {
  drawerVisible.value = true;
}

function handleDrawerClose(needRefresh?: boolean) {
  drawerVisible.value = false;
  if (needRefresh) {
    gridApi.query();
  }
}

// 审批弹窗
const approveVisible = ref(false);
const approveLoading = ref(false);
const approveForm = reactive({
  id: 0,
  approved: true,
  remark: '',
});

function openApprove(row: any) {
  approveForm.id = row.id;
  approveForm.approved = true;
  approveForm.remark = '';
  approveVisible.value = true;
}

async function handleApproveSubmit() {
  approveLoading.value = true;
  try {
    await approveFinancePaymentApi({
      id: approveForm.id,
      approved: approveForm.approved,
      remark: approveForm.remark,
    });
    message.success($t('page.finance.payment.message.approveSuccess'));
    approveVisible.value = false;
    gridApi.query();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.payment.message.approveFailed'),
    );
  } finally {
    approveLoading.value = false;
  }
}

// 确认付款弹窗
const confirmVisible = ref(false);
const confirmLoading = ref(false);
const confirmForm = reactive({
  id: 0,
  paymentDate: '',
});

function openConfirm(row: any) {
  confirmForm.id = row.id;
  confirmForm.paymentDate = '';
  confirmVisible.value = true;
}

async function handleConfirmSubmit() {
  if (!confirmForm.paymentDate) {
    message.warning($t('page.finance.payment.message.paymentDateRequired'));
    return;
  }
  confirmLoading.value = true;
  try {
    await confirmFinancePaymentApi({
      id: confirmForm.id,
      paymentDate: confirmForm.paymentDate,
    });
    message.success($t('page.finance.payment.message.confirmSuccess'));
    confirmVisible.value = false;
    gridApi.query();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.payment.message.confirmFailed'),
    );
  } finally {
    confirmLoading.value = false;
  }
}

// 取消弹窗
const cancelVisible = ref(false);
const cancelLoading = ref(false);
const cancelForm = reactive({
  id: 0,
  remark: '',
});

function openCancel(row: any) {
  cancelForm.id = row.id;
  cancelForm.remark = '';
  cancelVisible.value = true;
}

async function handleCancelSubmit() {
  if (!cancelForm.remark) {
    message.warning($t('page.finance.payment.message.cancelReasonRequired'));
    return;
  }
  cancelLoading.value = true;
  try {
    await cancelFinancePaymentApi({
      id: cancelForm.id,
      remark: cancelForm.remark,
    });
    message.success($t('page.finance.payment.message.cancelSuccess'));
    cancelVisible.value = false;
    gridApi.query();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.payment.message.cancelFailed'),
    );
  } finally {
    cancelLoading.value = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.finance.payment.title')">
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" @click="handleApply">
          {{ $t('page.finance.payment.button.apply') }}
        </Button>
        <Button class="mr-2" :icon="h(RefreshCw)" @click="gridApi.query()">
          {{ $t('page.finance.common.refresh') }}
        </Button>
      </template>

      <template #paymentType="{ row }">
        <Tag :color="paymentTypeMap[row.paymentType]?.color || 'default'">
          {{ paymentTypeMap[row.paymentType]?.label || row.paymentType }}
        </Tag>
      </template>

      <template #paymentMethod="{ row }">
        <Tag :color="paymentMethodMap[row.paymentMethod]?.color || 'default'">
          {{ paymentMethodMap[row.paymentMethod]?.label || row.paymentMethod }}
        </Tag>
      </template>

      <template #amount="{ row }">
        {{ formatMoney(row.amount) }}
      </template>

      <template #status="{ row }">
        <Tag :color="statusMap[row.status]?.color || 'default'">
          {{ statusMap[row.status]?.label || row.status }}
        </Tag>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button v-if="row.status === 0" type="link" @click="openApprove(row)">
          {{ $t('page.finance.payment.button.approveAction') }}
        </Button>
        <Button v-if="row.status === 1" type="link" @click="openConfirm(row)">
          {{ $t('page.finance.payment.button.confirm') }}
        </Button>
        <Button
          v-if="row.status === 3"
          type="link"
          danger
          @click="openCancel(row)"
        >
          {{ $t('page.finance.common.cancel') }}
        </Button>
      </template>
    </Grid>

    <PaymentDrawer :visible="drawerVisible" @close="handleDrawerClose" />

    <Modal
      v-model:open="approveVisible"
      :title="$t('page.finance.payment.button.approve')"
      :confirm-loading="approveLoading"
      @ok="handleApproveSubmit"
    >
      <Form :label-col="{ span: 5 }" :wrapper-col="{ span: 18 }" class="py-4">
        <FormItem :label="$t('page.finance.payment.modal.approveResult')">
          <Button
            :type="approveForm.approved ? 'primary' : 'default'"
            size="small"
            class="mr-2"
            @click="approveForm.approved = true"
          >
            {{ $t('page.finance.payment.modal.approvePass') }}
          </Button>
          <Button
            :type="!approveForm.approved ? 'primary' : 'default'"
            size="small"
            danger
            @click="approveForm.approved = false"
          >
            {{ $t('page.finance.payment.modal.approveReject') }}
          </Button>
        </FormItem>
        <FormItem :label="$t('page.finance.common.remark')">
          <Textarea
            v-model:value="approveForm.remark"
            :rows="3"
            :placeholder="
              $t('page.finance.payment.modal.approveRemarkPlaceholder')
            "
          />
        </FormItem>
      </Form>
    </Modal>

    <Modal
      v-model:open="confirmVisible"
      :title="$t('page.finance.payment.button.confirm')"
      :confirm-loading="confirmLoading"
      @ok="handleConfirmSubmit"
    >
      <Form :label-col="{ span: 5 }" :wrapper-col="{ span: 18 }" class="py-4">
        <FormItem
          :label="$t('page.finance.payment.modal.paymentDate')"
          required
        >
          <DatePicker
            v-model:value="confirmForm.paymentDate"
            value-format="YYYY-MM-DD"
            style="width: 100%"
            :placeholder="
              $t('page.finance.payment.modal.paymentDatePlaceholder')
            "
          />
        </FormItem>
      </Form>
    </Modal>

    <Modal
      v-model:open="cancelVisible"
      :title="$t('page.finance.payment.button.cancel')"
      :confirm-loading="cancelLoading"
      @ok="handleCancelSubmit"
    >
      <Form :label-col="{ span: 5 }" :wrapper-col="{ span: 18 }" class="py-4">
        <FormItem
          :label="$t('page.finance.payment.modal.cancelReason')"
          required
        >
          <Textarea
            v-model:value="cancelForm.remark"
            :rows="3"
            :placeholder="
              $t('page.finance.payment.modal.cancelReasonPlaceholder')
            "
          />
        </FormItem>
      </Form>
    </Modal>
  </Page>
</template>
