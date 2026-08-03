<script lang="ts" setup>
import { computed, h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, message, Modal, Popconfirm, Tag, Tabs } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  confirmPaymentApi,
  deletePaymentApi,
  getPaymentListApi,
  rejectPaymentApi,
} from '#/api/core/sale/payment';
import { $t } from '#/locales';
import ApplicationDrawer from './application-drawer.vue';
import PaymentDrawer from './drawer.vue';
import SalesProcessGuide from '../components/SalesProcessGuide.vue';
import CustomerDetailDrawer from '../../crm/components/CustomerDetailDrawer.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();

const canViewAll = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 1;
});

const canViewSubordinate = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 2 || dataScope === 3 || dataScope === 4;
});

const allTabList = [
  { key: 'all', label: '全部回款' },
  { key: 'my', label: '我的回款' },
  { key: 'subordinate', label: '下属回款' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter(t => keys.includes(t.key));
});

const activeTab = ref('my');

function handleTabChange(key: string | number) {
  activeTab.value = key as string;
  gridApi.query();
}

// 客户详情抽屉
const customerDetailVisible = ref(false);
const customerDetailId = ref<number | string | undefined>(undefined);

function openCustomerDetail(row: any) {
  const id = row.customerId ?? row.customer_id;
  if (!id) {
    message.error('客户ID不存在');
    return;
  }
  customerDetailId.value = Number(id);
  customerDetailVisible.value = true;
}

const paymentMethodMap: Record<number, { label: string; color: string }> = {
  1: { label: '银行转账', color: 'blue' },
  2: { label: '支付宝', color: 'cyan' },
  3: { label: '微信支付', color: 'green' },
  4: { label: '现金', color: 'orange' },
  5: { label: '支票', color: 'purple' },
  6: { label: '其他', color: 'default' },
};

const statusMap: Record<number, { label: string; color: string }> = {
  1: { label: '待确认', color: 'default' },
  2: { label: '已确认', color: 'green' },
  3: { label: '已驳回', color: 'red' },
  4: { label: '已取消', color: 'default' },
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'paymentNo',
      label: '回款编号',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'orderNo',
      label: '订单编号',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '回款状态',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '待确认', value: 1 },
          { label: '已确认', value: 2 },
          { label: '已驳回', value: 3 },
          { label: '已取消', value: 4 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'paymentMethod',
      label: '支付方式',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '银行转账', value: 1 },
          { label: '支付宝', value: 2 },
          { label: '微信支付', value: 3 },
          { label: '现金', value: 4 },
          { label: '支票', value: 5 },
          { label: '其他', value: 6 },
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
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  rowConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const result = await getPaymentListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
          paymentNo: formValues.paymentNo,
          orderNo: formValues.orderNo,
          status: formValues.status,
          paymentMethod: formValues.paymentMethod,
        });
        // 无数据 280px，有数据按内容自适应
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '280px' : '';
        }
        // 等DOM渲染完成后同步固定列行高并居中内容
        const syncFixedColumn = (retry = 0) => {
          const $el = gridApi.grid?.$el as HTMLElement | undefined;
          if (!$el) return;
          const mainBody = $el.querySelector('.vxe-table--body-wrapper tbody');
          const fixedRightBody = $el.querySelector('.vxe-table--fixed-right-wrapper tbody');
          if (!mainBody || !fixedRightBody) {
            if (retry < 3) setTimeout(() => syncFixedColumn(retry + 1), 200);
            return;
          }
          const rows1 = mainBody.querySelectorAll('tr.vxe-body--row');
          const rows2 = fixedRightBody.querySelectorAll('tr.vxe-body--row');
          const len = Math.min(rows1.length, rows2.length);
          if (len === 0) return;
          for (let i = 0; i < len; i++) {
            const h = (rows1[i] as HTMLElement).offsetHeight;
            if (h === 0) continue;
            (rows2[i] as HTMLElement).style.height = h + 'px';
            const tds = (rows2[i] as HTMLElement).querySelectorAll('td');
            tds.forEach((td: Element) => {
              const cell = td.querySelector('.vxe-cell');
              if (cell) {
                (cell as HTMLElement).style.display = 'flex';
                (cell as HTMLElement).style.alignItems = 'center';
                (cell as HTMLElement).style.justifyContent = 'center';
                (cell as HTMLElement).style.height = h + 'px';
              }
            });
          }
        };
        requestAnimationFrame(() => {
          syncFixedColumn();
          setTimeout(() => syncFixedColumn(), 200);
          setTimeout(() => syncFixedColumn(), 500);
        });
        return result;
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
      title: '回款编号',
      field: 'paymentNo',
      width: 180,
    },
    {
      title: '订单编号',
      field: 'orderNo',
      width: 160,
      slots: { default: 'orderNo' },
    },
    {
      title: '客户名称',
      field: 'customerName',
      width: 140,
      slots: { default: 'customerName' },
    },
    {
      title: '回款金额',
      field: 'amount',
      width: 120,
      align: 'right',
      slots: { default: 'amount' },
    },
    {
      title: '未核销金额',
      field: 'unappliedAmount',
      width: 120,
      align: 'right',
      slots: { default: 'unappliedAmount' },
    },
    {
      title: '支付方式',
      field: 'paymentMethod',
      width: 100,
      slots: { default: 'paymentMethod' },
    },
    {
      title: '回款状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: '到账日期',
      field: 'paymentDate',
      width: 120,
    },
    {
      title: '登记时间',
      field: 'createTime',
      width: 160,
      slots: { default: 'createTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 260,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: PaymentDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

const [AppDrawer, appDrawerApi] = useVbenDrawer({
  connectedComponent: ApplicationDrawer,
  onClosed() {
    const data = appDrawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function openApplicationDrawer(row: any) {
  appDrawerApi.setData({ row });
  appDrawerApi.open();
}

async function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deletePaymentApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleCreate() {
  openDrawer(true);
}

function handleConfirm(row: any) {
  Modal.confirm({
    title: '确认回款',
    content: `确定要确认回款「${row.paymentNo || ''}」吗？确认后将联动更新订单已付金额。`,
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await confirmPaymentApi(row.id);
        window.$message.success('确认成功');
        gridApi.query();
      } catch {
        window.$message.error('确认失败');
      }
    },
  });
}

function handleReject(row: any) {
  Modal.confirm({
    title: '驳回回款',
    content: `确定要驳回回款「${row.paymentNo || ''}」吗？`,
    okText: '确认',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        await rejectPaymentApi(row.id);
        window.$message.success('驳回成功');
        gridApi.query();
      } catch {
        window.$message.error('驳回失败');
      }
    },
  });
}
</script>

<template>
  <Page>
    <SalesProcessGuide current-step="payment" />
    <Grid :table-title="''">
      <template #form-header>
        <Tabs v-model:activeKey="activeTab" class="mb-3" @change="handleTabChange">
          <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:payment:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.sale.payment.button.create') }}
        </Button>
      </template>

      <template #orderNo="{ row }">
        <a v-if="row.orderNo" class="text-blue-500 hover:underline" @click="() => $router.push(`/sale/order?orderNo=${row.orderNo}`)">
          {{ row.orderNo }}
        </a>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #customerName="{ row }">
        <a v-if="row.customerId" class="text-blue-600 cursor-pointer hover:text-blue-800" @click="() => openCustomerDetail(row)">{{ row.customerName || '-' }}</a>
        <span v-else class="text-gray-300">{{ row.customerName || '-' }}</span>
      </template>

      <template #amount="{ row }">
        <span class="font-medium text-blue-600">
          ¥{{ Number(row.amount || 0).toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
        </span>
      </template>

      <template #unappliedAmount="{ row }">
        <span :class="Number(row.unappliedAmount || 0) > 0 ? 'text-orange-600 font-medium' : 'text-gray-400'">
          ¥{{ Number(row.unappliedAmount || 0).toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
        </span>
      </template>

      <template #paymentMethod="{ row }">
        <Tag v-if="row.paymentMethod && paymentMethodMap[row.paymentMethod]" :color="paymentMethodMap[row.paymentMethod]?.color">
          {{ paymentMethodMap[row.paymentMethod]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status && statusMap[row.status]" :color="statusMap[row.status]?.color">
          {{ statusMap[row.status]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('sale:payment:confirm') && row.status === 1"
          type="link"
          size="small"
          @click="() => handleConfirm(row)"
        >
          确认
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:payment:confirm') && row.status === 1"
          type="link"
          size="small"
          danger
          @click="() => handleReject(row)"
        >
          驳回
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:payment:confirm') && row.status === 2"
          type="link"
          size="small"
          @click="() => openApplicationDrawer(row)"
        >
          核销
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:payment:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.sale.payment.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('sale:payment:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
    <AppDrawer />
    <CustomerDetailDrawer v-model:visible="customerDetailVisible" :id="customerDetailId" />
  </Page>
</template>
