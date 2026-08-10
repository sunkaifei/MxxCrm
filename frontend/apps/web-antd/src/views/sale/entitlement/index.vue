<script lang="ts" setup>
import { ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import { Button, Modal, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import type { VbenFormProps } from '@vben/common-ui';
import {
  getEntitlementListApi,
  renewEntitlementApi,
  updateEntitlementApi,
} from '#/api';
import { $t } from '#/locales';

const accessStore = useAccessStore();

// 权益状态映射：1=待激活 2=生效中 3=已暂停 4=已到期 5=已取消
const statusMap: Record<number, { label: string; color: string }> = {
  1: { label: '待激活', color: 'default' },
  2: { label: '生效中', color: 'green' },
  3: { label: '已暂停', color: 'orange' },
  4: { label: '已到期', color: 'red' },
  5: { label: '已取消', color: 'default' },
};

// 权益类型映射：1=服务期 2=订阅 3=技术支持 4=资源包 5=SLA
const entitlementTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: '服务期', color: 'blue' },
  2: { label: '订阅', color: 'cyan' },
  3: { label: '技术支持', color: 'orange' },
  4: { label: '资源包', color: 'green' },
  5: { label: 'SLA', color: 'purple' },
};

// 续约弹窗
const renewVisible = ref(false);
const renewLoading = ref(false);
const renewRow = ref<any>({});
const newOrderId = ref<undefined | number>(undefined);

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'InputNumber',
      fieldName: 'customerId',
      label: '客户ID',
      componentProps: {
        placeholder: '请输入客户ID',
        allowClear: true,
        controls: false,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '待激活', value: 1 },
          { label: '生效中', value: 2 },
          { label: '已暂停', value: 3 },
          { label: '已到期', value: 4 },
          { label: '已取消', value: 5 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'entitlementType',
      label: '权益类型',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '服务期', value: 1 },
          { label: '订阅', value: 2 },
          { label: '技术支持', value: 3 },
          { label: '资源包', value: 4 },
          { label: 'SLA', value: 5 },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          customerId: formValues.customerId,
          status: formValues.status,
          entitlementType: formValues.entitlementType,
        };
        return await getEntitlementListApi(params);
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
      title: '权益编号',
      field: 'entitlementNo',
      width: 180,
    },
    {
      title: '客户ID',
      field: 'customerId',
      width: 100,
    },
    {
      title: '商品名',
      field: 'productName',
      minWidth: 160,
    },
    {
      title: '权益类型',
      field: 'entitlementType',
      width: 110,
      slots: { default: 'entitlementType' },
    },
    {
      title: '状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: '开始日期',
      field: 'startDate',
      width: 120,
    },
    {
      title: '结束日期',
      field: 'endDate',
      width: 120,
    },
    {
      title: '剩余天数',
      field: 'remainingDays',
      width: 120,
      slots: { default: 'remainingDays' },
    },
    {
      title: '服务时长(月)',
      field: 'durationMonths',
      width: 120,
      align: 'right',
    },
    {
      title: '自动续约',
      field: 'autoRenew',
      width: 100,
      slots: { default: 'autoRenew' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 200,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function openRenew(row: any) {
  renewRow.value = row;
  newOrderId.value = undefined;
  renewVisible.value = true;
}

async function handleRenewSubmit() {
  if (!newOrderId.value) {
    message.warning('请输入续约订单ID');
    return;
  }
  renewLoading.value = true;
  try {
    await renewEntitlementApi({
      id: renewRow.value.id,
      newOrderId: newOrderId.value,
    });
    message.success('续约成功');
    renewVisible.value = false;
    gridApi.query();
  } catch {
    // 错误由全局拦截器处理
  } finally {
    renewLoading.value = false;
  }
}

function handleToggleStatus(row: any) {
  // 已暂停(3) -> 激活(2)；生效中(2) -> 暂停(3)
  const isPaused = row.status === 3;
  const actionText = isPaused ? '激活' : '暂停';
  const targetStatus = isPaused ? 2 : 3;
  Modal.confirm({
    title: `${actionText}确认`,
    content: `确定要${actionText}权益「${row.entitlementNo || ''}」吗？`,
    okText: $t('ui.button.ok'),
    cancelText: $t('ui.button.cancel'),
    onOk: async () => {
      try {
        await updateEntitlementApi({ id: row.id, status: targetStatus });
        message.success(`${actionText}成功`);
        gridApi.query();
      } catch {
        message.error(`${actionText}失败`);
      }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.sale.entitlement.title')">
      <template #entitlementType="{ row }">
        <Tag
          v-if="row.entitlementType && entitlementTypeMap[row.entitlementType]"
          :color="entitlementTypeMap[row.entitlementType]?.color"
        >
          {{ row.entitlementTypeName || entitlementTypeMap[row.entitlementType]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status && statusMap[row.status]" :color="statusMap[row.status]?.color">
          {{ row.statusName || statusMap[row.status]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #remainingDays="{ row }">
        <span
          v-if="row.remainingDays != null"
          :class="{ 'text-red-500 font-semibold': row.remainingDays < 0 }"
        >
          剩余{{ row.remainingDays }}天
        </span>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #autoRenew="{ row }">
        <Tag v-if="row.autoRenew === 1" color="green">是</Tag>
        <Tag v-else color="default">否</Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('sale:entitlement:update')"
          type="link"
          size="small"
          @click="() => openRenew(row)"
        >
          {{ $t('page.sale.entitlement.button.renew') }}
        </Button>
        <Button
          v-if="
            accessStore.hasAccessCode('sale:entitlement:update') &&
            (row.status === 2 || row.status === 3)
          "
          type="link"
          size="small"
          @click="() => handleToggleStatus(row)"
        >
          {{ row.status === 3 ? '激活' : '暂停' }}
        </Button>
      </template>
    </Grid>

    <Modal
      v-model:open="renewVisible"
      :title="$t('page.sale.entitlement.button.renew')"
      :confirm-loading="renewLoading"
      :ok-text="$t('ui.button.ok')"
      :cancel-text="$t('ui.button.cancel')"
      @ok="handleRenewSubmit"
    >
      <div class="py-4">
        <p class="mb-3">
          权益编号：<span class="font-mono">{{ renewRow.entitlementNo }}</span>
        </p>
        <p class="mb-4">
          请输入续约订单ID，续约后将延长该权益的服务期限。
        </p>
        <a-input-number
          v-model:value="newOrderId"
          placeholder="请输入续约订单ID"
          style="width: 100%"
          :controls="false"
        />
      </div>
    </Modal>
  </Page>
</template>
