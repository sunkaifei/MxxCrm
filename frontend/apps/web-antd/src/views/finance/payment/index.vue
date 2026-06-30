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
  Modal,
  Tag,
  Textarea,
  message,
} from 'ant-design-vue';
import { RefreshCw } from 'lucide-vue-next';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  approvePaymentApi,
  cancelPaymentApi,
  confirmPaymentApi,
  getPaymentListApi,
} from '#/api/core/finance';
import { $t } from '#/locales';

import PaymentDrawer from './drawer.vue';

const drawerVisible = ref(false);

const paymentTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: '预付款', color: 'blue' },
  2: { label: '尾款', color: 'orange' },
  3: { label: '全款', color: 'green' },
};

const paymentMethodMap: Record<number, { label: string; color: string }> = {
  1: { label: '银行转账', color: 'blue' },
  2: { label: '现金', color: 'green' },
  3: { label: '支票', color: 'orange' },
  4: { label: '其他', color: 'default' },
};

const statusMap: Record<number, { label: string; color: string }> = {
  0: { label: '待审批', color: 'default' },
  1: { label: '已审批', color: 'processing' },
  2: { label: '已付款', color: 'green' },
  3: { label: '已取消', color: 'red' },
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
      label: '付款编号',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'supplierName',
      label: '供应商',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
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
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getPaymentListApi({
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
      title: '付款编号',
      field: 'paymentNo',
      minWidth: 160,
    },
    {
      title: '采购订单号',
      field: 'poNo',
      minWidth: 160,
    },
    {
      title: '供应商',
      field: 'supplierName',
      minWidth: 140,
    },
    {
      title: '付款类型',
      field: 'paymentType',
      width: 110,
      slots: { default: 'paymentType' },
    },
    {
      title: '付款金额',
      field: 'amount',
      width: 130,
      slots: { default: 'amount' },
    },
    {
      title: '付款方式',
      field: 'paymentMethod',
      width: 120,
      slots: { default: 'paymentMethod' },
    },
    {
      title: '状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: '申请时间',
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
    await approvePaymentApi({
      id: approveForm.id,
      approved: approveForm.approved,
      remark: approveForm.remark,
    });
    message.success('审批完成');
    approveVisible.value = false;
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '审批失败');
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
    message.warning('请选择付款日期');
    return;
  }
  confirmLoading.value = true;
  try {
    await confirmPaymentApi({
      id: confirmForm.id,
      paymentDate: confirmForm.paymentDate,
    });
    message.success('确认付款成功');
    confirmVisible.value = false;
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '确认付款失败');
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
    message.warning('请输入取消原因');
    return;
  }
  cancelLoading.value = true;
  try {
    await cancelPaymentApi({ id: cancelForm.id, remark: cancelForm.remark });
    message.success('取消成功');
    cancelVisible.value = false;
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '取消失败');
  } finally {
    cancelLoading.value = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="采购付款">
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" @click="handleApply">
          申请付款
        </Button>
        <Button class="mr-2" :icon="h(RefreshCw)" @click="gridApi.query()">
          刷新
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
        <Button
          v-if="row.status === 0"
          type="link"
          @click="openApprove(row)"
        >
          审批
        </Button>
        <Button
          v-if="row.status === 1"
          type="link"
          @click="openConfirm(row)"
        >
          确认付款
        </Button>
        <Button
          v-if="row.status === 3"
          type="link"
          danger
          @click="openCancel(row)"
        >
          取消
        </Button>
      </template>
    </Grid>

    <PaymentDrawer :visible="drawerVisible" @close="handleDrawerClose" />

    <Modal
      v-model:open="approveVisible"
      title="付款审批"
      :confirm-loading="approveLoading"
      @ok="handleApproveSubmit"
    >
      <Form :label-col="{ span: 5 }" :wrapper-col="{ span: 18 }" class="py-4">
        <FormItem label="审批结果">
          <Button
            :type="approveForm.approved ? 'primary' : 'default'"
            size="small"
            class="mr-2"
            @click="approveForm.approved = true"
          >
            通过
          </Button>
          <Button
            :type="!approveForm.approved ? 'primary' : 'default'"
            size="small"
            danger
            @click="approveForm.approved = false"
          >
            驳回
          </Button>
        </FormItem>
        <FormItem label="备注">
          <Textarea
            v-model:value="approveForm.remark"
            :rows="3"
            placeholder="请输入审批备注"
          />
        </FormItem>
      </Form>
    </Modal>

    <Modal
      v-model:open="confirmVisible"
      title="确认付款"
      :confirm-loading="confirmLoading"
      @ok="handleConfirmSubmit"
    >
      <Form :label-col="{ span: 5 }" :wrapper-col="{ span: 18 }" class="py-4">
        <FormItem label="付款日期" required>
          <DatePicker
            v-model:value="confirmForm.paymentDate"
            value-format="YYYY-MM-DD"
            style="width: 100%"
            placeholder="请选择付款日期"
          />
        </FormItem>
      </Form>
    </Modal>

    <Modal
      v-model:open="cancelVisible"
      title="取消付款"
      :confirm-loading="cancelLoading"
      @ok="handleCancelSubmit"
    >
      <Form :label-col="{ span: 5 }" :wrapper-col="{ span: 18 }" class="py-4">
        <FormItem label="取消原因" required>
          <Textarea
            v-model:value="cancelForm.remark"
            :rows="3"
            placeholder="请输入取消原因"
          />
        </FormItem>
      </Form>
    </Modal>
  </Page>
</template>
