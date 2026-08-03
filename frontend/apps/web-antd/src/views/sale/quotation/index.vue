<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, Dropdown, Menu, Tabs, message, Modal, Popconfirm, Tag } from 'ant-design-vue';
import { LucideChevronDown } from '@vben/icons';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteQuotationApi, getQuotationInfoApi, getQuotationListApi, submitQuotationApprovalApi } from '#/api';
import { $t } from '#/locales';
import OrderDrawer from '../order/drawer.vue';
import QuotationDetail from './detail.vue';
import QuotationDrawer from './drawer.vue';
import SalesProcessGuide from '../components/SalesProcessGuide.vue';
import CustomerDetailDrawer from '../../crm/components/CustomerDetailDrawer.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();

// 全部报价单 Tab 显示条件：超级管理员 / 系统管理员 / data_scope=全部数据
const canViewAll = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 1;
});

// 下属报价单 Tab 显示条件：超级管理员 / 系统管理员 / 数据权限含部门（2/3/4）
const canViewSubordinate = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 2 || dataScope === 3 || dataScope === 4;
});

const allTabList = [
  { key: 'all', label: '全部报价单' },
  { key: 'my', label: '我的报价单' },
  { key: 'subordinate', label: '下属报价单' },
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

const detailVisible = ref(false);
const detailId = ref(0);

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

const approvalStatusOptions = [
  { label: '草稿', value: 1 },
  { label: '审批中', value: 2 },
  { label: '已通过', value: 3 },
  { label: '已驳回', value: 4 },
];

const approvalStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'processing',
  3: 'success',
  4: 'error',
};

const approvalStatusLabelMap: Record<number, string> = {
  0: '草稿',
  1: '草稿',
  2: '审批中',
  3: '已通过',
  4: '已驳回',
};

const currencySymbolMap: Record<number, string> = {
  1: '¥',
  2: '$',
  3: '€',
  4: '£',
  5: '¥',
  6: 'HK$',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '报价单号/标题',
      componentProps: { placeholder: '请输入报价单号或标题', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'approvalStatus',
      label: '审批状态',
      componentProps: { placeholder: '全部', allowClear: true, options: approvalStatusOptions },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,
  checkboxConfig: { checkMethod: () => true },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getQuotationListApi({ page: page.currentPage, pageSize: page.pageSize, listType: activeTab.value, ...formValues });
      },
    },
  },
  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '报价编号', field: 'quotationNo', width: 150, headerAlign: 'center', align: 'left', slots: { default: 'quotationNo' } },
    { title: '标题', field: 'title', width: 200, headerAlign: 'center', align: 'left', slots: { default: 'title' } },
    { title: '客户名称', field: 'customerName', width: 140, headerAlign: 'center', align: 'left', slots: { default: 'customerName' } },
    { title: '报价金额', field: 'grandTotal', width: 140, slots: { default: 'grandTotal' } },
    { title: '审批状态', field: 'approvalStatus', width: 100, slots: { default: 'approvalStatus' } },
    { title: '报价日期', field: 'quotationDate', width: 110 },
    { title: '负责人', field: 'ownerUserName', width: 100, headerAlign: 'center', align: 'left' },
    { title: '创建时间', field: 'createTime', width: 160, slots: { default: 'createTime' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 240 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: QuotationDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

const [OrderFormDrawer, orderDrawerApi] = useVbenDrawer({
  connectedComponent: OrderDrawer,
  onClosed() {
    const data = orderDrawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteQuotationApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleBatchDelete() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    window.$message.warning('请选择要删除的报价单');
    return;
  }
  const ids = records.map((r: any) => r.id);
  await deleteQuotationApi(ids);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}

function handleSubmitApproval(row: any) {
  Modal.confirm({
    title: '提交审批',
    content: '确认提交此报价单进行审批？提交后将进入审批流程，审批期间无法修改。',
    okText: '确认提交',
    cancelText: '取消',
    onOk: async () => {
      try {
        await submitQuotationApprovalApi(row.id);
        window.$message.success('已提交审批');
        gridApi.query();
      } catch {
        // error handled by interceptor
      }
    },
  });
}

async function handleConvertToOrder(row: any) {
  row.pending = true;
  try {
    const info: any = await getQuotationInfoApi(row.id);
    const quotationData = info?.data ?? info;
    orderDrawerApi.setData({
      create: true,
      fromQuotation: quotationData,
    });
    orderDrawerApi.open();
  } finally {
    row.pending = false;
  }
}

function openDetail(row: any) {
  detailId.value = row.id;
  detailVisible.value = true;
}

function handleDetailEdit(id: string) {
  detailVisible.value = false;
  const record = gridApi.grid.getTableData().fullData.find((r: any) => String(r.id) === id);
  if (record) openDrawer(false, record);
}
</script>

<template>
  <Page auto-content-height>
    <SalesProcessGuide current-step="quotation" />
    <Grid :table-title="$t('page.sale.quotation.title')">
      <template #form-header>
        <Tabs v-model:activeKey="activeTab" class="mb-3" @change="handleTabChange">
          <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:quotation:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新建报价单
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:quotation:delete')"
          class="mr-2"
          @click="handleBatchDelete"
        >
          批量删除
        </Button>
      </template>

      <template #quotationNo="{ row }">
        <a class="text-blue-600 cursor-pointer" @click="openDetail(row)">{{ row.quotationNo }}</a>
      </template>

      <template #title="{ row }">
        <a class="text-blue-600 cursor-pointer" @click="openDetail(row)">{{ row.title }}</a>
      </template>

      <template #customerName="{ row }">
        <a v-if="row.customerId" class="text-blue-600 cursor-pointer hover:text-blue-800" @click="() => openCustomerDetail(row)">{{ row.customerName || '-' }}</a>
        <span v-else>{{ row.customerName || '-' }}</span>
      </template>

      <template #grandTotal="{ row }">
        {{ currencySymbolMap[row.currency] || '¥' }} {{ row.grandTotal?.toLocaleString?.() ?? row.grandTotal }}
      </template>

      <template #approvalStatus="{ row }">
        <Tag :color="approvalStatusColorMap[row.approvalStatus] || 'default'">
          {{ approvalStatusLabelMap[row.approvalStatus] || '草稿' }}
        </Tag>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <a
          v-if="accessStore.hasAccessCode('sale:quotation:edit') && (!row.approvalStatus || row.approvalStatus === 1 || row.approvalStatus === 4)"
          class="text-blue-600 cursor-pointer mr-3"
          @click="() => handleSubmitApproval(row)"
        >
          提交审批
        </a>
        <Dropdown :trigger="['click']">
          <a class="text-blue-600 cursor-pointer" @click.prevent>
            更多<LucideChevronDown class="inline-block ml-0.5" :size="12" />
          </a>
          <template #overlay>
            <Menu>
              <Menu.Item
                v-if="row.approvalStatus === 3"
                key="toOrder"
                @click="handleConvertToOrder(row)"
              >
                一键转订单
              </Menu.Item>
              <Menu.Item
                v-if="accessStore.hasAccessCode('sale:quotation:edit') && (!row.approvalStatus || row.approvalStatus === 1 || row.approvalStatus === 4)"
                key="edit"
                @click="() => handleEdit(row)"
              >
                修改
              </Menu.Item>
              <Popconfirm
                v-if="accessStore.hasAccessCode('sale:quotation:delete') && (!row.approvalStatus || row.approvalStatus === 1 || row.approvalStatus === 4)"
                :title="$t('ui.text.do_you_want_delete', { moduleName: '报价单' })"
                :ok-text="$t('ui.button.ok')"
                :cancel-text="$t('ui.button.cancel')"
                @confirm="handleDelete(row)"
              >
                <Menu.Item key="delete" danger>删除</Menu.Item>
              </Popconfirm>
            </Menu>
          </template>
        </Dropdown>
      </template>
    </Grid>
    <FormDrawer />
    <OrderFormDrawer />
    <Drawer
      v-model:open="detailVisible"
      placement="right"
      :width="960"
      title="报价单详情"
      :destroy-on-close="true"
      :body-style="{ padding: 0 }"
    >
      <QuotationDetail v-if="detailVisible" :id="detailId" @edit="handleDetailEdit" />
    </Drawer>
    <CustomerDetailDrawer v-model:visible="customerDetailVisible" :id="customerDetailId" />
  </Page>
</template>
