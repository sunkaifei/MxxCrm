<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Drawer,
  Dropdown,
  Input,
  Menu,
  Modal,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  createContractFromOrderApi,
  deleteOrderApi,
  getOrderListApi,
  getShipmentListApi,
  signShipmentApi,
  submitOrderApi,
  updateOrderStatusApi,
} from '#/api';
import { $t } from '#/locales';

import CustomerDetail from '../../crm/customer/detail.vue';
import SalesProcessGuide from '../components/SalesProcessGuide.vue';
import ShipmentDrawer from '../shipment/drawer.vue';
import OrderApprovalDrawer from './approval-drawer.vue';
import OrderDrawer from './drawer.vue';
import OrderViewDrawer from './view-drawer.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();
const router = useRouter();

// 全部订单 Tab 显示条件：超级管理员 / 系统管理员 / data_scope=全部数据
const canViewAll = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope =
    (userStore.userInfo as any)?.dataScope ??
    (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin'))
    return true;
  return dataScope === 1;
});

// 下属订单 Tab 显示条件：超级管理员 / 系统管理员 / 数据权限含部门（2/3/4）
const canViewSubordinate = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope =
    (userStore.userInfo as any)?.dataScope ??
    (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin'))
    return true;
  return dataScope === 2 || dataScope === 3 || dataScope === 4;
});

const allTabList = [
  { key: 'all', label: '全部订单' },
  { key: 'my', label: '我的订单' },
  { key: 'subordinate', label: '下属订单' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter((t) => keys.includes(t.key));
});

const activeTab = ref('my');

// 是否为下属视图（下属视图下只能查看，不能操作）
const isSubordinateView = computed(() => activeTab.value === 'subordinate');

// 是否超级管理员/系统管理员
const isSuperAdmin = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  return roles.includes('super_admin') || roles.includes('system_admin');
});

function handleTabChange(key: number | string) {
  activeTab.value = key as string;
  gridApi.query();
}

const orderStatusOptions = [
  { label: '草稿', value: 1 },
  { label: '待确认', value: 2 },
  { label: '已确认', value: 3 },
  { label: '备货中', value: 4 },
  { label: '部分发货', value: 5 },
  { label: '已发货', value: 6 },
  { label: '已取消', value: 7 },
  { label: '已交付', value: 8 },
  { label: '已签收', value: 9 },
  { label: '已完成', value: 10 },
  { label: '已作废', value: 11 },
];

const paymentStatusOptions = [
  { label: '未支付', value: 1 },
  { label: '部分支付', value: 2 },
  { label: '已支付', value: 3 },
  { label: '已退款', value: 4 },
];

const orderStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'blue',
  3: 'blue',
  4: 'orange',
  5: 'cyan',
  6: 'purple',
  7: 'red',
  8: 'cyan',
  9: 'green',
  10: 'blue',
  11: 'red',
};

const orderStatusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '待确认',
  3: '已确认',
  4: '备货中',
  5: '部分发货',
  6: '已发货',
  7: '已取消',
  8: '已交付',
  9: '已签收',
  10: '已完成',
  11: '已作废',
};

const paymentStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'orange',
  3: 'green',
  4: 'red',
};

const paymentStatusLabelMap: Record<number, string> = {
  1: '未支付',
  2: '部分支付',
  3: '已支付',
  4: '已退款',
};

const _approvalStatusOptions = [
  { label: '草稿', value: 0 },
  { label: '待审批', value: 1 },
  { label: '审批中', value: 2 },
  { label: '已通过', value: 3 },
  { label: '已驳回', value: 4 },
];
void _approvalStatusOptions;

const approvalStatusColorMap: Record<number, string> = {
  0: 'default',
  1: 'processing',
  2: 'warning',
  3: 'success',
  4: 'error',
};

const approvalStatusLabelMap: Record<number, string> = {
  0: '草稿',
  1: '待审批',
  2: '审批中',
  3: '已通过',
  4: '已驳回',
};

const currencyLabelMap: Record<number, string> = {
  1: 'CNY',
  2: 'USD',
  3: 'EUR',
  4: 'GBP',
  5: 'JPY',
  6: 'HKD',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: { placeholder: '订单号/客户/标题', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'orderStatus',
      label: '订单状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: orderStatusOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'paymentStatus',
      label: '支付状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: paymentStatusOptions,
      },
    },
    {
      component: 'RangePicker',
      fieldName: 'dateRange',
      label: '日期范围',
      componentProps: {
        placeholder: ['开始日期', '结束日期'],
        style: 'width:100%',
        valueFormat: 'YYYY-MM-DD',
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  rowConfig: {},
  stripe: true,
  checkboxConfig: { checkMethod: () => true },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
        };
        if (formValues.keywords) params.keywords = formValues.keywords;
        if (formValues.orderStatus) params.orderStatus = formValues.orderStatus;
        if (formValues.paymentStatus)
          params.paymentStatus = formValues.paymentStatus;
        if (formValues.dateRange && formValues.dateRange.length === 2) {
          params.startDate = formValues.dateRange[0];
          params.endDate = formValues.dateRange[1];
        }
        const result = await getOrderListApi(params);
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
          const fixedRightBody = $el.querySelector(
            '.vxe-table--fixed-right-wrapper tbody',
          );
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
            (rows2[i] as HTMLElement).style.height = `${h}px`;
            const tds = (rows2[i] as HTMLElement).querySelectorAll('td');
            tds.forEach((td: Element) => {
              const cell = td.querySelector('.vxe-cell');
              if (cell) {
                (cell as HTMLElement).style.display = 'flex';
                (cell as HTMLElement).style.alignItems = 'center';
                (cell as HTMLElement).style.justifyContent = 'center';
                (cell as HTMLElement).style.height = `${h}px`;
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
    { type: 'checkbox', width: 50 },
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 60,
      headerAlign: 'center',
    },
    {
      title: '订单号',
      field: 'orderNo',
      width: 160,
      headerAlign: 'center',
      slots: { default: 'orderNo' },
    },
    {
      title: '订单标题',
      field: 'title',
      width: 200,
      headerAlign: 'center',
      slots: { default: 'title' },
    },
    {
      title: '客户名称',
      field: 'customerName',
      width: 180,
      headerAlign: 'center',
      align: 'left',
      slots: { default: 'customerName' },
    },
    {
      title: '订单金额',
      field: 'totalAmount',
      width: 140,
      headerAlign: 'center',
      slots: { default: 'totalAmount' },
    },
    {
      title: '订单状态',
      field: 'orderStatus',
      width: 100,
      headerAlign: 'center',
      slots: { default: 'orderStatus' },
    },
    {
      title: '支付状态',
      field: 'paymentStatus',
      width: 100,
      headerAlign: 'center',
      slots: { default: 'paymentStatus' },
    },
    {
      title: '负责人',
      field: 'ownerUserName',
      width: 90,
      headerAlign: 'center',
    },
    {
      title: '下单日期',
      field: 'orderDate',
      width: 110,
      headerAlign: 'center',
    },
    {
      title: '审批状态',
      field: 'approvalStatus',
      width: 90,
      headerAlign: 'center',
      slots: { default: 'approvalStatus' },
    },
    {
      title: '创建时间',
      field: 'createTime',
      width: 160,
      headerAlign: 'center',
      slots: { default: 'createTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      headerAlign: 'center',
      slots: { default: 'action' },
      width: 320,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: OrderDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

const [ShipmentFormDrawer, shipmentDrawerApi] = useVbenDrawer({
  connectedComponent: ShipmentDrawer,
  onClosed() {
    const data = shipmentDrawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setState({ width: undefined } as any);
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function openShipmentDrawer(row: any) {
  shipmentDrawerApi.setData({ row });
  shipmentDrawerApi.open();
}

function handleCreate() {
  openDrawer(true);
}
function handleEdit(row: any) {
  openDrawer(false, row);
}
function handleView(row: any) {
  viewDrawerOrderId.value = row.id;
  viewDrawerVisible.value = true;
}

// 详情抽屉底部操作动作分发（关闭详情抽屉 → 调用对应 handler）
function handleViewDrawerAction(type: string, row: any) {
  viewDrawerVisible.value = false;
  switch (type) {
    case 'edit': {
      handleEdit(row);
      break;
    }
    case 'ship': {
      openShipmentDrawer(row);
      break;
    }
    case 'signContract': {
      handleCreateContract(row);
      break;
    }
    case 'submitApproval': {
      handleSubmitApproval(row);
      break;
    }
    case 'viewApproval': {
      approvalOrderId.value = row.id;
      approvalDrawerVisible.value = true;
      break;
    }
    case 'viewContract': {
      handleViewContract(row);
      break;
    }
    case 'viewOpportunity': {
      if (row.opportunityId) {
        router.push({
          path: '/crm/opportunity',
          query: { viewOpportunityId: row.opportunityId },
        });
      }
      break;
    }
    case 'viewQuotation': {
      if (row.quotationId) {
        router.push({
          path: '/sale/quotation',
          query: { viewQuotationId: row.quotationId },
        });
      }
      break;
    }
  }
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteOrderApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleBatchDelete() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    window.$message.warning('请选择要删除的订单');
    return;
  }
  const ids = records.map((r: any) => r.id);
  await deleteOrderApi(ids);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}

// 备货：将订单状态从已确认(3)改为备货中(4)
async function handleStockUp(row: any) {
  Modal.confirm({
    title: '确认备货',
    content: '确定要将此订单标记为备货中吗？',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await updateOrderStatusApi({
          id: row.id,
          orderStatus: 4,
        });
        window.$message.success('订单已标记为备货中');
        gridApi.query();
      } catch {
        window.$message.error('操作失败');
      }
    },
  });
}

// 签收：逐个签收该订单下所有发货单
async function handleSign(row: any) {
  Modal.confirm({
    title: '确认签收',
    content: '确定要签收该订单的所有发货单吗？',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        const res: any = await getShipmentListApi({
          orderId: row.id,
          page: 1,
          pageSize: 100,
        });
        const shipments =
          res?.items || res?.data?.items || res?.data || res || [];
        if (!Array.isArray(shipments) || shipments.length === 0) {
          window.$message.warning('未查询到发货单');
          return;
        }
        for (const s of shipments) {
          await signShipmentApi(s.id);
        }
        window.$message.success('签收成功');
        gridApi.query();
      } catch {
        window.$message.error('签收失败');
      }
    },
  });
}

async function handleComplete(row: any) {
  Modal.confirm({
    title: '确认完成订单',
    content: '确定要将此订单标记为已完成吗？',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await updateOrderStatusApi({
          id: row.id,
          orderStatus: 10,
        });
        window.$message.success('订单已完成');
        gridApi.query();
      } catch {
        window.$message.error('操作失败');
      }
    },
  });
}

// ========== 订单作废 ==========

const voidRemarkVisible = ref(false);
const voidRemarkText = ref('');
const voidOrderRow = ref<any>(null);

function openVoidModal(row: any) {
  voidOrderRow.value = row;
  voidRemarkText.value = '';
  voidRemarkVisible.value = true;
}

function cancelVoidModal() {
  voidRemarkVisible.value = false;
  voidOrderRow.value = null;
  voidRemarkText.value = '';
}

async function submitVoidOrder() {
  if (!voidOrderRow.value) return;
  if (!voidRemarkText.value.trim()) {
    window.$message.warning('请填写作废备注');
    return;
  }
  try {
    await updateOrderStatusApi({
      id: voidOrderRow.value.id,
      orderStatus: 11,
      remark: voidRemarkText.value.trim(),
    });
    window.$message.success('订单已作废');
    voidRemarkVisible.value = false;
    voidOrderRow.value = null;
    voidRemarkText.value = '';
    gridApi.query();
  } catch {
    window.$message.error('操作失败');
  }
}

function handleVoidOrder(row: any) {
  Modal.confirm({
    title: '确认订单作废',
    content: '订单作废后将不可恢复，确定要作废该订单吗？',
    okText: '确认作废',
    okButtonProps: { danger: true },
    cancelText: '取消',
    onOk: () => {
      openVoidModal(row);
    },
  });
}

// ========== 审批流 ==========

const approvalDrawerVisible = ref(false);
const approvalOrderId = ref<null | number>(null);
const currentUserId = ref<number | undefined>(undefined);

// ========== 详情抽屉 ==========

const viewDrawerVisible = ref(false);
const viewDrawerOrderId = ref<null | number>(null);

// 从 accessStore 中获取当前用户ID
const userInfo = (window as any).$userInfo || {};
currentUserId.value = userInfo.id || undefined;

// 提交审批
async function handleSubmitApproval(row: any) {
  Modal.confirm({
    title: '提交审批',
    content: '确定要提交该订单进入审批流程吗？',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await submitOrderApi(row.id);
        window.$message.success('已提交审批');
        gridApi.query();
      } catch {
        window.$message.error('提交审批失败');
      }
    },
  });
}

// 查看审批详情
function handleViewApproval(row: any) {
  approvalOrderId.value = row.id;
  approvalDrawerVisible.value = true;
}

// 查看合同（跳转到合同管理页面）
function handleViewContract(row: any) {
  router.push({
    path: '/crm/contract',
    query: { viewContractId: row.contractId },
  });
}

// 创建合同（从已审批通过的订单）
async function handleCreateContract(row: any) {
  Modal.confirm({
    title: '创建合同',
    content: '确定要基于此订单创建合同吗？客户和订单信息将自动填充且不可修改。',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        const result: any = await createContractFromOrderApi(row.id);
        const contractId = result?.data?.id || result?.id || result;
        window.$message.success('合同已创建，正在跳转...');
        // 跳转到合同管理页面并打开编辑抽屉
        router.push({
          path: '/crm/contract',
          query: { editContractId: contractId },
        });
      } catch {
        window.$message.error('创建合同失败');
      }
    },
  });
}
// ========== 客户详情抽屉 ==========
const customerDetailVisible = ref(false);
const customerDetailId = ref<null | number>(null);
const customerDetailKey = ref(0);

function openCustomerDetail(customerId: number) {
  if (!customerId) {
    window.$message.warning('该订单未关联客户ID');
    return;
  }
  customerDetailId.value = customerId;
  customerDetailKey.value++;
  customerDetailVisible.value = true;
}
function closeCustomerDetail() {
  customerDetailVisible.value = false;
  customerDetailId.value = null;
}
</script>

<template>
  <Page>
    <SalesProcessGuide current-step="order" />
    <Grid table-title="">
      <template #form-header>
        <Tabs
          v-model:active-key="activeTab"
          class="mb-3"
          @change="handleTabChange"
        >
          <Tabs.TabPane
            v-for="tab in tabList"
            :key="tab.key"
            :tab="tab.label"
          />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Button
          v-if="
            !isSubordinateView &&
            !isSuperAdmin &&
            accessStore.hasAccessCode('sale:order:save')
          "
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新建订单
        </Button>
        <Button
          v-if="
            !isSubordinateView && accessStore.hasAccessCode('sale:order:delete')
          "
          class="mr-2"
          @click="handleBatchDelete"
        >
          批量删除
        </Button>
      </template>

      <template #orderNo="{ row }">
        <a
          v-if="accessStore.hasAccessCode('sale:order:list')"
          class="cursor-pointer"
          style="color: hsl(var(--primary))"
          @click="handleView(row)"
        >
          {{ row.orderNo }}
        </a>
        <span v-else>{{ row.orderNo }}</span>
      </template>

      <template #title="{ row }">
        <a
          class="cursor-pointer hover:underline"
          style="color: hsl(var(--primary))"
          @click="() => handleView(row)"
        >
          {{ row.title || '-' }}
        </a>
      </template>

      <template #customerName="{ row }">
        <a
          v-if="row.customerId"
          class="cursor-pointer hover:underline"
          style="color: hsl(var(--primary))"
          @click="() => openCustomerDetail(Number(row.customerId))"
        >
          {{ row.customerName || '-' }}
        </a>
        <span v-else>{{ row.customerName || '-' }}</span>
      </template>

      <template #totalAmount="{ row }">
        {{ currencyLabelMap[row.currency] || 'CNY' }}
        {{ row.totalAmount?.toLocaleString?.() ?? row.totalAmount ?? 0 }}
      </template>

      <template #orderStatus="{ row }">
        <Tag :color="orderStatusColorMap[row.orderStatus]">
          {{ orderStatusLabelMap[row.orderStatus] || row.orderStatus }}
        </Tag>
      </template>

      <template #paymentStatus="{ row }">
        <Tag :color="paymentStatusColorMap[row.paymentStatus]">
          {{ paymentStatusLabelMap[row.paymentStatus] || row.paymentStatus }}
        </Tag>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #approvalStatus="{ row }">
        <Tag :color="approvalStatusColorMap[row.approvalStatus] ?? 'default'">
          {{ approvalStatusLabelMap[row.approvalStatus] ?? '未知' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <!-- 详情：所有状态都可查看 -->
        <a
          v-if="accessStore.hasAccessCode('sale:order:list')"
          class="cursor-pointer mx-1"
          style="color: hsl(var(--primary))"
          @click="() => handleView(row)"
          >详情</a
        >
        <!-- 提交审批：草稿(0)或已驳回(4)状态 -->
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('sale:order:update') &&
            (row.approvalStatus === 0 || row.approvalStatus === 4)
          "
          class="cursor-pointer mx-1"
          style="color: hsl(var(--primary))"
          @click="() => handleSubmitApproval(row)"
          >提交审批</a
        >
        <!-- 查看审批：待审批(1)或审批中(2)或已通过(3)或已驳回(4) -->
        <a
          v-if="
            accessStore.hasAccessCode('sale:order:list') &&
            row.approvalStatus >= 1 &&
            row.approvalStatus <= 4
          "
          class="cursor-pointer mx-1"
          style="color: hsl(var(--primary))"
          @click="() => handleViewApproval(row)"
          >审批</a
        >
        <!-- 更多操作：编辑/删除/订单作废 -->
        <Dropdown
          v-if="
            !isSubordinateView &&
            ((accessStore.hasAccessCode('sale:order:update') &&
              (row.approvalStatus === 0 || row.approvalStatus === 4)) ||
              (accessStore.hasAccessCode('sale:order:delete') &&
                (row.approvalStatus === 0 || row.approvalStatus === 4)) ||
              (accessStore.hasAccessCode('sale:order:update') &&
                row.approvalStatus === 3 &&
                !row.contractId))
          "
        >
          <a class="cursor-pointer mx-1" style="color: hsl(var(--primary))"
            >更多<span class="text-xs">▾</span></a
          >
          <template #overlay>
            <Menu>
              <!-- 编辑：草稿(0)或已驳回(4)，审批中(2)/已通过(3)不可编辑 -->
              <Menu.Item
                v-if="
                  !isSubordinateView &&
                  accessStore.hasAccessCode('sale:order:update') &&
                  (row.approvalStatus === 0 || row.approvalStatus === 4)
                "
                @click="() => handleEdit(row)"
              >
                编辑
              </Menu.Item>
              <!-- 订单作废：审批通过(3)且未签合同 -->
              <Menu.Item
                v-if="
                  !isSubordinateView &&
                  accessStore.hasAccessCode('sale:order:update') &&
                  row.approvalStatus === 3 &&
                  !row.contractId
                "
                @click="() => handleVoidOrder(row)"
              >
                <span style="color: hsl(0deg 84% 60%)">订单作废</span>
              </Menu.Item>
              <!-- 删除：草稿(0)或已驳回(4)允许，审批中(2)/已通过(3)不可删除 -->
              <Menu.Item
                v-if="
                  !isSubordinateView &&
                  accessStore.hasAccessCode('sale:order:delete') &&
                  (row.approvalStatus === 0 || row.approvalStatus === 4)
                "
                @click="() => handleDelete(row)"
              >
                <span style="color: hsl(0deg 84% 60%)">删除</span>
              </Menu.Item>
            </Menu>
          </template>
        </Dropdown>
        <!-- 查看合同：审批已通过(3)且已关联合同 -->
        <a
          v-if="
            accessStore.hasAccessCode('sale:order:update') &&
            row.approvalStatus === 3 &&
            row.contractId
          "
          class="cursor-pointer mx-1"
          style="color: hsl(var(--primary))"
          @click="() => handleViewContract(row)"
          >查看合同</a
        >
        <!-- 签署合同：审批已通过(3)且未关联合同 -->
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('sale:order:update') &&
            row.approvalStatus === 3 &&
            !row.contractId
          "
          class="cursor-pointer mx-1 font-medium"
          style="color: hsl(142deg 71% 45%)"
          @click="() => handleCreateContract(row)"
          >签署合同</a
        >
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('sale:order:update') &&
            row.orderStatus === 3
          "
          class="cursor-pointer mx-1"
          style="color: hsl(var(--primary))"
          @click="() => handleStockUp(row)"
          >备货</a
        >
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('sale:order:update') &&
            (row.orderStatus === 3 ||
              row.orderStatus === 4 ||
              row.orderStatus === 5)
          "
          class="cursor-pointer mx-1"
          style="color: hsl(var(--primary))"
          @click="() => openShipmentDrawer(row)"
          >发货</a
        >
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('sale:order:update') &&
            row.orderStatus === 6
          "
          class="cursor-pointer mx-1"
          style="color: hsl(var(--primary))"
          @click="() => handleSign(row)"
          >签收</a
        >
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('sale:order:update') &&
            row.orderStatus === 9
          "
          class="cursor-pointer mx-1"
          style="color: hsl(var(--primary))"
          @click="() => handleComplete(row)"
          >完成</a
        >
      </template>
    </Grid>
    <FormDrawer />
    <ShipmentFormDrawer />
    <OrderViewDrawer
      v-model:visible="viewDrawerVisible"
      :order-id="viewDrawerOrderId"
      @refresh="gridApi.query()"
      @action="handleViewDrawerAction"
    />
    <OrderApprovalDrawer
      v-model:visible="approvalDrawerVisible"
      :order-id="approvalOrderId"
      @success="gridApi.query()"
    />
    <Drawer
      v-model:open="customerDetailVisible"
      :width="1100"
      placement="right"
      :destroy-on-close="false"
      :mask-closable="false"
      :closable="true"
      title="客户详情"
      :body-style="{ padding: 0, overflow: 'auto', height: '100%' }"
      @close="closeCustomerDetail"
    >
      <CustomerDetail
        v-if="customerDetailVisible && customerDetailId"
        :key="customerDetailKey"
        :id="customerDetailId"
      />
    </Drawer>
    <!-- 订单作废备注弹窗 -->
    <Modal
      v-model:open="voidRemarkVisible"
      title="订单作废备注"
      ok-text="确认作废"
      cancel-text="取消"
      :ok-button-props="{ danger: true }"
      @ok="submitVoidOrder"
      @cancel="cancelVoidModal"
    >
      <div class="py-2">
        <p class="mb-3" style="color: hsl(var(--muted-foreground))">
          请填写作废原因（必填）：
        </p>
        <Input.TextArea
          v-model:value="voidRemarkText"
          :rows="4"
          placeholder="请输入作废原因..."
          :max-length="500"
          show-count
        />
      </div>
    </Modal>
  </Page>
</template>
