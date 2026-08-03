<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, Modal, Popconfirm, Tag, Tabs } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteRefundApi,
  getRefundListApi,
  submitRefundApi,
  approveRefundApi,
  rejectRefundApi,
  receiveRefundApi,
  qualityCheckRefundApi,
  cancelRefundApi,
} from '#/api';
import { $t } from '#/locales';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import RefundDrawer from './drawer.vue';
import RefundDetail from './detail.vue';
import CustomerDetail from '../../crm/customer/detail.vue';

// 销售退款使用说明步骤数（与 i18n 中 page.sale.refund.guide.steps 数组对齐）
const guideStepCount = 5;

const accessStore = useAccessStore();
const userStore = useUserStore();

// 全部退货单 Tab 显示条件：超级管理员 / 系统管理员 / data_scope=全部数据
const canViewAll = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope =
    (userStore.userInfo as any)?.dataScope ??
    (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin'))
    return true;
  return dataScope === 1;
});

// 下属退货单 Tab 显示条件：超级管理员 / 系统管理员 / 数据权限含部门（2/3/4）
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
  { key: 'all', label: '全部退货单' },
  { key: 'my', label: '我的退货单' },
  { key: 'subordinate', label: '下属退货单' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter((t) => keys.includes(t.key));
});

const activeTab = ref('my');

function handleTabChange(key: string | number) {
  activeTab.value = key as string;
  gridApi.query();
}

// 退货状态映射：1=草稿,2=待审批,3=审批通过,4=待收货,5=已收货,6=质检中,7=已完成,8=已驳回,9=已取消
const refundStatusOptions = [
  { label: '草稿', value: 1 },
  { label: '待审批', value: 2 },
  { label: '审批通过', value: 3 },
  { label: '待收货', value: 4 },
  { label: '已收货', value: 5 },
  { label: '质检中', value: 6 },
  { label: '已完成', value: 7 },
  { label: '已驳回', value: 8 },
  { label: '已取消', value: 9 },
];

const refundStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'processing',
  3: 'success',
  4: 'warning',
  5: 'cyan',
  6: 'orange',
  7: 'green',
  8: 'error',
  9: 'default',
};

const refundStatusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '待审批',
  3: '审批通过',
  4: '待收货',
  5: '已收货',
  6: '质检中',
  7: '已完成',
  8: '已驳回',
  9: '已取消',
};

// 审批状态映射：0=草稿,1=待审批,2=审批中,3=已通过,4=已驳回
const approvalStatusOptions = [
  { label: '草稿', value: 0 },
  { label: '待审批', value: 1 },
  { label: '审批中', value: 2 },
  { label: '已通过', value: 3 },
  { label: '已驳回', value: 4 },
];

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

// 退货类型：1=整单退货, 2=部分退货
const refundTypeLabelMap: Record<number, string> = {
  1: '整单退货',
  2: '部分退货',
};

const refundTypeColorMap: Record<number, string> = {
  1: 'orange',
  2: 'blue',
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
      componentProps: { placeholder: '退货单号/客户/标题', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'refundStatus',
      label: '退货状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: refundStatusOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'approvalStatus',
      label: '审批状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: approvalStatusOptions,
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
          pageNum: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
        };
        if (formValues.keywords) params.keywords = formValues.keywords;
        if (formValues.refundStatus) params.refundStatus = formValues.refundStatus;
        if (formValues.approvalStatus)
          params.approvalStatus = formValues.approvalStatus;
        if (formValues.dateRange && formValues.dateRange.length === 2) {
          params.startDate = formValues.dateRange[0];
          params.endDate = formValues.dateRange[1];
        }
        const result = await getRefundListApi(params);
        // 无数据 280px，有数据按内容自适应
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
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60, headerAlign: 'center' },
    {
      title: '退货单号',
      field: 'refundNo',
      width: 170,
      headerAlign: 'center',
      slots: { default: 'refundNo' },
    },
    { title: '退货标题', field: 'title', width: 200, headerAlign: 'center' },
    {
      title: '客户名称',
      field: 'customerName',
      width: 180,
      headerAlign: 'center',
      align: 'left',
      slots: { default: 'customerName' },
    },
    {
      title: '关联订单',
      field: 'orderNo',
      width: 150,
      headerAlign: 'center',
      slots: { default: 'orderNo' },
    },
    {
      title: '退货类型',
      field: 'refundType',
      width: 100,
      headerAlign: 'center',
      slots: { default: 'refundType' },
    },
    {
      title: '退货金额',
      field: 'refundAmount',
      width: 130,
      headerAlign: 'center',
      slots: { default: 'refundAmount' },
    },
    {
      title: '退货状态',
      field: 'refundStatus',
      width: 100,
      headerAlign: 'center',
      slots: { default: 'refundStatus' },
    },
    {
      title: '审批状态',
      field: 'approvalStatus',
      width: 100,
      headerAlign: 'center',
      slots: { default: 'approvalStatus' },
    },
    {
      title: '负责人',
      field: 'ownerUserName',
      width: 90,
      headerAlign: 'center',
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
  connectedComponent: RefundDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleCreate() {
  openDrawer(true);
}
function handleEdit(row: any) {
  openDrawer(false, row);
}

// ========== 详情抽屉 ==========
const detailVisible = ref(false);
const detailId = ref<number | null>(null);
const detailKey = ref(0);

function handleView(row: any) {
  detailId.value = Number(row.id);
  detailKey.value++;
  detailVisible.value = true;
}

function closeDetail() {
  detailVisible.value = false;
  detailId.value = null;
}

// ========== 客户详情抽屉 ==========
const customerDetailVisible = ref(false);
const customerDetailId = ref<number | null>(null);
const customerDetailKey = ref(0);

function openCustomerDetail(customerId: number) {
  if (!customerId) {
    window.$message.warning('该退货单未关联客户ID');
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

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteRefundApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleBatchDelete() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    window.$message.warning('请选择要删除的退货单');
    return;
  }
  const ids = records.map((r: any) => r.id);
  await deleteRefundApi(ids);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}

// ========== 流程操作 ==========

// 提交审批：仅草稿(1)/已驳回(8)状态
async function handleSubmitApproval(row: any) {
  Modal.confirm({
    title: '提交审批',
    content: '确定要提交该退货单进入审批流程吗？',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await submitRefundApi(row.id);
        window.$message.success('已提交审批');
        gridApi.query();
      } catch {
        window.$message.error('提交审批失败');
      }
    },
  });
}

// 审批通过：仅待审批(2)状态
async function handleApprove(row: any) {
  Modal.confirm({
    title: '审批通过',
    content: '确定要审批通过该退货单吗？',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await approveRefundApi(row.id);
        window.$message.success('审批通过');
        gridApi.query();
      } catch {
        window.$message.error('操作失败');
      }
    },
  });
}

// 审批驳回：仅待审批(2)状态
async function handleReject(row: any) {
  let reason = '';
  Modal.confirm({
    title: '审批驳回',
    content: () =>
      h(
        'div',
        { style: 'display:flex;flex-direction:column;gap:8px;' },
        [
          h('span', '确定要驳回该退货单吗？'),
          h('textarea', {
            placeholder: '请输入驳回原因',
            style: 'min-height:80px;padding:8px;border:1px solid #d9d9d9;border-radius:4px;',
            onInput: (e: any) => { reason = e.target.value; },
          }),
        ],
      ),
    okText: '确认驳回',
    cancelText: '取消',
    onOk: async () => {
      try {
        await rejectRefundApi(row.id, reason);
        window.$message.success('已驳回');
        gridApi.query();
      } catch {
        window.$message.error('操作失败');
      }
    },
  });
}

// 仓库收货：仅审批通过(3)/待收货(4)状态
async function handleReceive(row: any) {
  let logisticsNo = '';
  let logisticsCompany = '';
  Modal.confirm({
    title: '仓库收货',
    content: () =>
      h(
        'div',
        { style: 'display:flex;flex-direction:column;gap:8px;' },
        [
          h('span', '确认收货该退货单？'),
          h('input', {
            placeholder: '退货物流单号（可选）',
            style: 'padding:6px 8px;border:1px solid #d9d9d9;border-radius:4px;',
            onInput: (e: any) => { logisticsNo = e.target.value; },
          }),
          h('input', {
            placeholder: '退货物流公司（可选）',
            style: 'padding:6px 8px;border:1px solid #d9d9d9;border-radius:4px;',
            onInput: (e: any) => { logisticsCompany = e.target.value; },
          }),
        ],
      ),
    okText: '确认收货',
    cancelText: '取消',
    onOk: async () => {
      try {
        await receiveRefundApi({
          refundId: row.id,
          logisticsNo: logisticsNo || undefined,
          logisticsCompany: logisticsCompany || undefined,
        });
        window.$message.success('收货成功');
        gridApi.query();
      } catch {
        window.$message.error('操作失败');
      }
    },
  });
}

// 质检完成：仅已收货(5)/质检中(6)状态
async function handleQualityCheck(row: any) {
  let result = 1;
  let remark = '';
  Modal.confirm({
    title: '质检完成',
    content: () =>
      h(
        'div',
        { style: 'display:flex;flex-direction:column;gap:8px;' },
        [
          h('div', { style: 'display:flex;align-items:center;gap:12px;' }, [
            h('span', '质检结果：'),
            h('select', {
              style: 'padding:6px 8px;border:1px solid #d9d9d9;border-radius:4px;flex:1;',
              onChange: (e: any) => { result = Number(e.target.value); },
            }, [
              h('option', { value: 1 }, '合格'),
              h('option', { value: 2 }, '不合格'),
            ]),
          ]),
          h('textarea', {
            placeholder: '质检备注（可选）',
            style: 'min-height:80px;padding:8px;border:1px solid #d9d9d9;border-radius:4px;',
            onInput: (e: any) => { remark = e.target.value; },
          }),
        ],
      ),
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await qualityCheckRefundApi({
          refundId: row.id,
          qualityCheckResult: result,
          qualityCheckRemark: remark || undefined,
        });
        window.$message.success('质检完成');
        gridApi.query();
      } catch {
        window.$message.error('操作失败');
      }
    },
  });
}

// 取消退货单：仅草稿(1)/待审批(2)/已驳回(8)状态
async function handleCancel(row: any) {
  Modal.confirm({
    title: '取消退货单',
    content: '确定要取消该退货单吗？取消后不可恢复。',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await cancelRefundApi(row.id);
        window.$message.success('已取消');
        gridApi.query();
      } catch {
        window.$message.error('操作失败');
      }
    },
  });
}
</script>

<template>
  <Page>
    <PageUsageGuide
      :title="$t('page.sale.refund.guide.title')"
      :brief="$t('page.sale.refund.guide.brief')"
      :expand-text="$t('page.sale.refund.guide.expand')"
      :collapse-text="$t('page.sale.refund.guide.collapse')"
    >
      <div
        v-for="i in guideStepCount"
        :key="i"
        class="page-guide-step-item"
      >
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.sale.refund.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.sale.refund.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Grid :table-title="''">
      <template #form-header>
        <Tabs v-model:activeKey="activeTab" class="mb-3" @change="handleTabChange">
          <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:refund:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新建退货单
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:refund:delete')"
          class="mr-2"
          @click="handleBatchDelete"
        >
          批量删除
        </Button>
      </template>

      <template #refundNo="{ row }">
        <a
          v-if="accessStore.hasAccessCode('sale:refund:list')"
          class="text-blue-600 cursor-pointer"
          @click="handleView(row)"
        >
          {{ row.refundNo }}
        </a>
        <span v-else>{{ row.refundNo }}</span>
      </template>

      <template #customerName="{ row }">
        <a
          v-if="row.customerId"
          class="text-blue-600 cursor-pointer hover:text-blue-800"
          @click="() => openCustomerDetail(Number(row.customerId))"
        >
          {{ row.customerName || '-' }}
        </a>
        <span v-else>{{ row.customerName || '-' }}</span>
      </template>

      <template #orderNo="{ row }">
        <span>{{ row.orderNo || '-' }}</span>
      </template>

      <template #refundType="{ row }">
        <Tag :color="refundTypeColorMap[row.refundType] || 'default'">
          {{ refundTypeLabelMap[row.refundType] || row.refundType || '-' }}
        </Tag>
      </template>

      <template #refundAmount="{ row }">
        <span class="font-medium text-red-500">
          {{ Number(row.refundAmount ?? 0).toFixed(2) }}
        </span>
      </template>

      <template #refundStatus="{ row }">
        <Tag :color="refundStatusColorMap[row.refundStatus] ?? 'default'">
          {{ refundStatusLabelMap[row.refundStatus] ?? row.refundStatus ?? '-' }}
        </Tag>
      </template>

      <template #approvalStatus="{ row }">
        <Tag :color="approvalStatusColorMap[row.approvalStatus] ?? 'default'">
          {{ approvalStatusLabelMap[row.approvalStatus] ?? '未知' }}
        </Tag>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <a
          v-if="accessStore.hasAccessCode('sale:refund:list')"
          class="text-blue-600 cursor-pointer mx-1"
          @click="() => handleView(row)"
        >查看</a>
        <!-- 编辑：草稿(1)或已驳回(8)或已取消(9) -->
        <a
          v-if="
            accessStore.hasAccessCode('sale:refund:update') &&
            (row.refundStatus === 1 || row.refundStatus === 8 || row.refundStatus === 9)
          "
          class="text-blue-600 cursor-pointer mx-1"
          @click="() => handleEdit(row)"
        >编辑</a>
        <!-- 提交审批：草稿(1)或已驳回(8) -->
        <a
          v-if="
            accessStore.hasAccessCode('sale:refund:update') &&
            (row.refundStatus === 1 || row.refundStatus === 8)
          "
          class="text-blue-600 cursor-pointer mx-1"
          @click="() => handleSubmitApproval(row)"
        >提交审批</a>
        <!-- 审批通过：待审批(2) -->
        <a
          v-if="
            accessStore.hasAccessCode('sale:refund:approve') &&
            row.refundStatus === 2
          "
          class="text-green-600 cursor-pointer mx-1"
          @click="() => handleApprove(row)"
        >审批通过</a>
        <!-- 审批驳回：待审批(2) -->
        <a
          v-if="
            accessStore.hasAccessCode('sale:refund:approve') &&
            row.refundStatus === 2
          "
          class="text-orange-600 cursor-pointer mx-1"
          @click="() => handleReject(row)"
        >驳回</a>
        <!-- 仓库收货：审批通过(3)/待收货(4) -->
        <a
          v-if="
            accessStore.hasAccessCode('sale:refund:update') &&
            (row.refundStatus === 3 || row.refundStatus === 4)
          "
          class="text-blue-600 cursor-pointer mx-1"
          @click="() => handleReceive(row)"
        >收货</a>
        <!-- 质检完成：已收货(5)/质检中(6) -->
        <a
          v-if="
            accessStore.hasAccessCode('sale:refund:update') &&
            (row.refundStatus === 5 || row.refundStatus === 6)
          "
          class="text-blue-600 cursor-pointer mx-1"
          @click="() => handleQualityCheck(row)"
        >质检</a>
        <!-- 取消：草稿(1)/待审批(2)/已驳回(8) -->
        <a
          v-if="
            accessStore.hasAccessCode('sale:refund:update') &&
            (row.refundStatus === 1 || row.refundStatus === 2 || row.refundStatus === 8)
          "
          class="text-orange-600 cursor-pointer mx-1"
          @click="() => handleCancel(row)"
        >取消</a>
        <!-- 删除：仅草稿(1)/已驳回(8)/已取消(9) -->
        <Popconfirm
          v-if="
            accessStore.hasAccessCode('sale:refund:delete') &&
            (row.refundStatus === 1 || row.refundStatus === 8 || row.refundStatus === 9)
          "
          :title="$t('ui.text.do_you_want_delete', { moduleName: '退货单' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <a class="text-red-500 cursor-pointer mx-1">删除</a>
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />
    <Drawer
      v-model:open="detailVisible"
      :width="1100"
      placement="right"
      :destroy-on-close="false"
      :mask-closable="false"
      :closable="true"
      title="退货单详情"
      :body-style="{ padding: 0, overflow: 'auto', height: '100%' }"
      @close="closeDetail"
    >
      <RefundDetail
        v-if="detailVisible && detailId"
        :key="detailKey"
        :id="detailId"
      />
    </Drawer>
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
  </Page>
</template>
