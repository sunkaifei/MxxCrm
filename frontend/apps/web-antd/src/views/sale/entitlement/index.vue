<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import { Button, message, Modal, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteEntitlementApi,
  getEntitlementInfoApi,
  getEntitlementListApi,
  renewEntitlementApi,
  updateEntitlementApi,
} from '#/api/core/sale/entitlement';
import CustomerSelectModal from '#/views/crm/components/CustomerSelectModal.vue';
import { getCustomerListApi } from '#/api/core/crm/customer';
import { $t } from '#/locales';

import EntitlementCreateDrawer from './drawer.vue';
import EntitlementDetailDrawer from './detail-drawer.vue';

const accessStore = useAccessStore();

// 权益状态映射：1=待激活 2=生效中 3=已暂停 4=已到期 5=已取消
const statusMap: Record<number, { color: string; label: string }> = {
  1: { label: '待激活', color: 'default' },
  2: { label: '生效中', color: 'green' },
  3: { label: '已暂停', color: 'orange' },
  4: { label: '已到期', color: 'red' },
  5: { label: '已取消', color: 'default' },
};

// 权益类型映射：1=服务期 2=订阅 3=技术支持 4=资源包 5=SLA
const entitlementTypeMap: Record<number, { color: string; label: string }> = {
  1: { label: '服务期', color: 'blue' },
  2: { label: '订阅', color: 'cyan' },
  3: { label: '技术支持', color: 'orange' },
  4: { label: '资源包', color: 'green' },
  5: { label: 'SLA', color: 'purple' },
};

// P1.2 快捷 Tab：全部/生效中/30 天内到期/已到期（D11）
const activeTab = ref('all');
const tabs = [
  { key: 'all', label: '全部' },
  { key: 'active', label: '生效中' },
  { key: 'expiring30', label: '30 天内到期' },
  { key: 'expired', label: '已到期' },
];

// 续约弹窗（P0.1：字段契约 oldEntitlementId；P0.5：订单 ID 输入）
const renewVisible = ref(false);
const renewLoading = ref(false);
const renewRow = ref<any>({});
const newOrderId = ref<number | undefined>(undefined);

// P0.6 新建/详情抽屉
const createVisible = ref(false);
const detailVisible = ref(false);
const detailData = ref<any>({});
const detailLoading = ref(false);

// P0.5 客户选择器
const custSelectVisible = ref(false);
const customerId = ref<number | undefined>(undefined);
const customerLabel = ref('');

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: {
        placeholder: '权益编号/商品名',
        allowClear: true,
      },
    },
    {
      component: 'ApiSelect',
      fieldName: 'customerId',
      label: '客户',
      componentProps: {
        placeholder: '搜索并选择客户',
        allowClear: true,
        showSearch: true,
        filterOption: false,
        remote: true,
        params: { companyName: '' },
        api: async (params: any) => {
          const res: any = await getCustomerListApi({
            page: 1,
            pageSize: 20,
            ...(params?.companyName ? { companyName: params.companyName } : {}),
          });
          return res?.items || [];
        },
        labelField: 'companyName',
        valueField: 'id',
        onSearch(keyword: string) {
          gridApi.formApi?.updateSchema([
            { fieldName: 'customerId', componentProps: { params: { companyName: keyword } } },
          ]);
        },
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'orderId',
      label: '关联订单',
      componentProps: {
        placeholder: '订单ID',
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
    {
      component: 'RangePicker',
      fieldName: 'endDateRange',
      label: '到期范围',
      componentProps: {
        placeholder: ['开始日期', '结束日期'],
        valueFormat: 'YYYY-MM-DD',
      },
    },
    {
      component: 'Select',
      fieldName: 'autoRenew',
      label: '自动续约',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '是', value: 1 },
          { label: '否', value: 0 },
        ],
      },
    },
  ],
};

function handleCustSelect(cust: any) {
  customerId.value = cust.id;
  customerLabel.value = cust.companyName || cust.shortName || String(cust.id);
  custSelectVisible.value = false;
}

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
    slots: { buttons: 'toolbar-buttons' },
  },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: {},
  stripe: true,
  checkboxConfig: { highlight: true },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
          customerId: formValues.customerId,
          orderId: formValues.orderId,
          status: formValues.status,
          entitlementType: formValues.entitlementType,
          autoRenew: formValues.autoRenew,
        };
        // P1.2 快捷 Tab 映射
        if (activeTab.value === 'active') params.status = 2;
        if (activeTab.value === 'expired') params.status = 4;
        if (activeTab.value === 'expiring30') {
          params.status = 2;
          const today = new Date();
          const in30 = new Date(today.getTime() + 30 * 86_400_000);
          const fmt = (d: Date) => d.toISOString().slice(0, 10);
          params.startDate = fmt(today);
          params.endDate = fmt(in30);
        }
        // 到期日期范围
        if (Array.isArray(formValues.endDateRange) && formValues.endDateRange.length === 2) {
          params.startDate = formValues.endDateRange[0];
          params.endDate = formValues.endDateRange[1];
        }
        return await getEntitlementListApi(params);
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 70 },
    { title: '权益编号', field: 'entitlementNo', width: 170 },
    { title: '客户', field: 'customerName', minWidth: 140 },
    { title: '商品名', field: 'productName', minWidth: 150 },
    { title: '关联订单', field: 'orderNo', width: 150 },
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
    { title: '开始日期', field: 'startDate', width: 115 },
    { title: '结束日期', field: 'endDate', width: 115 },
    {
      title: '剩余天数',
      field: 'remainingDays',
      width: 110,
      slots: { default: 'remainingDays' },
    },
    {
      title: '自动续约',
      field: 'autoRenew',
      width: 95,
      slots: { default: 'autoRenew' },
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

function handleTabChange(key: string) {
  activeTab.value = key;
  gridApi.query();
}

async function openDetail(row: any) {
  detailVisible.value = true;
  detailLoading.value = true;
  try {
    const res: any = await getEntitlementInfoApi(row.id);
    detailData.value = res?.data || res || {};
  } finally {
    detailLoading.value = false;
  }
}

function openCreate() {
  createVisible.value = true;
}

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
    // P0.1：字段契约 oldEntitlementId + newOrderId
    await renewEntitlementApi({
      oldEntitlementId: renewRow.value.id,
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

async function handleDelete(row: any) {
  Modal.confirm({
    title: '删除确认',
    content: `确定要删除权益「${row.entitlementNo || ''}」吗？删除后进入回收站，30 天内可还原。`,
    okText: $t('ui.button.ok'),
    cancelText: $t('ui.button.cancel'),
    onOk: async () => {
      try {
        await deleteEntitlementApi([row.id]);
        message.success('删除成功，已进入回收站');
        gridApi.query();
      } catch {
        message.error('删除失败');
      }
    },
  });
}

function onCreated() {
  createVisible.value = false;
  message.success('新建权益成功');
  gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <div class="mb-2 flex flex-wrap items-center gap-2">
      <Button
        v-for="t in tabs"
        :key="t.key"
        :type="activeTab === t.key ? 'primary' : 'default'"
        size="small"
        @click="() => handleTabChange(t.key)"
      >
        {{ t.label }}
      </Button>
    </div>
    <Grid :table-title="$t('page.sale.entitlement.title')">
      <template #toolbar-buttons>
        <Button
          v-if="accessStore.hasAccessCode('sale:entitlement:save')"
          type="primary"
          @click="() => openCreate()"
        >
          新建权益
        </Button>
      </template>

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
        <Tag
          v-if="row.status && statusMap[row.status]"
          :color="statusMap[row.status]?.color"
        >
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
          v-if="accessStore.hasAccessCode('sale:entitlement:view')"
          type="link"
          size="small"
          @click="() => openDetail(row)"
        >
          详情
        </Button>
        <Button
          v-if="
            accessStore.hasAccessCode('sale:entitlement:renew') &&
            row.status !== 5
          "
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
        <Button
          v-if="
            accessStore.hasAccessCode('sale:entitlement:delete') &&
            row.status !== 2
          "
          type="link"
          size="small"
          danger
          @click="() => handleDelete(row)"
        >
          删除
        </Button>
      </template>
    </Grid>

    <!-- 续约弹窗（P0.1 契约修正） -->
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
        <p class="mb-2">请输入续约关联的订单 ID，续约后服务期自动顺延。</p>
        <a-input-number
          v-model:value="newOrderId"
          placeholder="请输入续约订单ID"
          style="width: 100%"
          :controls="false"
        />
      </div>
    </Modal>

    <!-- 客户选择器（P0.5 筛选四件套） -->
    <CustomerSelectModal
      v-model:visible="custSelectVisible"
      @select="handleCustSelect"
    />

    <!-- 新建权益抽屉（P0.6） -->
    <EntitlementCreateDrawer
      v-model:visible="createVisible"
      @success="onCreated"
    />

    <!-- 详情抽屉（P0.2/P0.6） -->
    <EntitlementDetailDrawer
      v-model:visible="detailVisible"
      :detail="detailData"
      :loading="detailLoading"
    />
  </Page>
</template>
