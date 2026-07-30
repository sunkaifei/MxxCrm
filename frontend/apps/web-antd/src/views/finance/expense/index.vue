<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Modal, Popconfirm, Tag, Tabs } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  approveExpenseApi,
  batchDeleteExpenseApi,
  getExpenseListApi,
  getExpenseTypeListApi,
  paymentExpenseApi,
  rejectExpenseApi,
  submitExpenseApi,
} from '#/api';
import { $t } from '#/locales';
import ExpenseDrawer from './drawer.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();

// 全部费用申请 Tab 显示条件：超级管理员 / 系统管理员 / data_scope=全部数据
const canViewAll = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope =
    (userStore.userInfo as any)?.dataScope ??
    (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin'))
    return true;
  return dataScope === 1;
});

// 下属费用申请 Tab 显示条件：超级管理员 / 系统管理员 / 数据权限含部门（2/3/4）
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
  { key: 'all', label: '全部费用申请' },
  { key: 'my', label: '我的费用申请' },
  { key: 'subordinate', label: '下属费用申请' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter((t) => keys.includes(t.key));
});

const activeTab = ref('my');

function handleTabChange(key: string) {
  activeTab.value = key;
  gridApi.query();
}

// 费用状态映射：1=草稿,2=待审批,3=审批中,4=已通过,5=已驳回,6=已打款
const expenseStatusOptions = [
  { label: '草稿', value: 1 },
  { label: '待审批', value: 2 },
  { label: '审批中', value: 3 },
  { label: '已通过', value: 4 },
  { label: '已驳回', value: 5 },
  { label: '已打款', value: 6 },
];

const expenseStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'processing',
  3: 'warning',
  4: 'success',
  5: 'error',
  6: 'green',
};

const expenseStatusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '待审批',
  3: '审批中',
  4: '已通过',
  5: '已驳回',
  6: '已打款',
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

// 费用类型选项（从 API 加载）
const expenseTypeOptions = ref<{ label: string; value: number }[]>([]);
const expenseTypeMap = ref<Record<number, { name: string; color: string }>>({});

async function loadExpenseTypes() {
  try {
    const res: any = await getExpenseTypeListApi({
      page: 1,
      pageSize: 100,
      enabled: 1,
    });
    const data = res?.data ?? res ?? {};
    const list = data.list || data.items || data.rows || data || [];
    const arr = Array.isArray(list) ? list : [];
    expenseTypeOptions.value = arr.map((t: any) => ({
      label: t.typeName || t.name || '',
      value: t.id,
    }));
    const map: Record<number, { name: string; color: string }> = {};
    arr.forEach((t: any) => {
      map[t.id] = {
        name: t.typeName || t.name || '',
        color: t.color || 'blue',
      };
    });
    expenseTypeMap.value = map;
  } catch (e) {
    console.error('[费用申请] 加载费用类型失败:', e);
    expenseTypeOptions.value = [];
  }
}

onMounted(() => {
  loadExpenseTypes();
});

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '费用标题',
      componentProps: { placeholder: '费用编号/标题', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'expenseType',
      label: '费用类型',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: expenseTypeOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: expenseStatusOptions,
      },
    },
    {
      component: 'RangePicker',
      fieldName: 'dateRange',
      label: '申请时间',
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
  cellConfig: { isHover: true },
  rowConfig: { height: 'auto' },
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
        if (formValues.expenseType) params.expenseType = formValues.expenseType;
        if (formValues.status) params.status = formValues.status;
        if (formValues.dateRange && formValues.dateRange.length === 2) {
          params.startDate = formValues.dateRange[0];
          params.endDate = formValues.dateRange[1];
        }
        const result = await getExpenseListApi(params);
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
      title: '费用编号',
      field: 'expenseNo',
      width: 170,
      headerAlign: 'center',
      slots: { default: 'expenseNo' },
    },
    { title: '费用标题', field: 'title', width: 200, headerAlign: 'center' },
    {
      title: '费用类型',
      field: 'expenseType',
      width: 120,
      headerAlign: 'center',
      slots: { default: 'expenseType' },
    },
    {
      title: '金额',
      field: 'totalAmount',
      width: 130,
      headerAlign: 'center',
      slots: { default: 'totalAmount' },
    },
    {
      title: '申请人',
      field: 'applicantName',
      width: 100,
      headerAlign: 'center',
    },
    {
      title: '状态',
      field: 'status',
      width: 100,
      headerAlign: 'center',
      slots: { default: 'status' },
    },
    {
      title: '审批状态',
      field: 'approvalStatus',
      width: 100,
      headerAlign: 'center',
      slots: { default: 'approvalStatus' },
    },
    {
      title: '申请日期',
      field: 'applyDate',
      width: 120,
      headerAlign: 'center',
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      headerAlign: 'center',
      slots: { default: 'action' },
      width: 280,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: ExpenseDrawer,
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

// ========== 查看详情（复用编辑抽屉只读模式） ==========
function handleView(row: any) {
  drawerApi.setData({ create: false, row, readonly: true });
  drawerApi.open();
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await batchDeleteExpenseApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleBatchDelete() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    window.$message.warning('请选择要删除的费用申请');
    return;
  }
  const ids = records.map((r: any) => r.id);
  await batchDeleteExpenseApi(ids);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}

// ========== 流程操作 ==========

// 提交审批：仅草稿(1)/已驳回(5)状态
async function handleSubmitApproval(row: any) {
  Modal.confirm({
    title: '提交审批',
    content: '确定要提交该费用申请进入审批流程吗？',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await submitExpenseApi(row.id);
        window.$message.success('已提交审批');
        gridApi.query();
      } catch {
        window.$message.error('提交审批失败');
      }
    },
  });
}

// 审批通过：待审批(2)/审批中(3)状态
async function handleApprove(row: any) {
  Modal.confirm({
    title: '审批通过',
    content: '确定要审批通过该费用申请吗？',
    okText: '确认',
    cancelText: '取消',
    onOk: async () => {
      try {
        await approveExpenseApi(row.id);
        window.$message.success('审批通过');
        gridApi.query();
      } catch {
        window.$message.error('操作失败');
      }
    },
  });
}

// 审批驳回：待审批(2)/审批中(3)状态
async function handleReject(row: any) {
  let reason = '';
  Modal.confirm({
    title: '审批驳回',
    content: () =>
      h(
        'div',
        { style: 'display:flex;flex-direction:column;gap:8px;' },
        [
          h('span', '确定要驳回该费用申请吗？'),
          h('textarea', {
            placeholder: '请输入驳回原因',
            style:
              'min-height:80px;padding:8px;border:1px solid #d9d9d9;border-radius:4px;',
            onInput: (e: any) => {
              reason = e.target.value;
            },
          }),
        ],
      ),
    okText: '确认驳回',
    cancelText: '取消',
    onOk: async () => {
      try {
        await rejectExpenseApi(row.id, reason);
        window.$message.success('已驳回');
        gridApi.query();
      } catch {
        window.$message.error('操作失败');
      }
    },
  });
}

// 打款：仅已通过(4)状态
async function handlePayment(row: any) {
  let paymentAmount = Number(row.totalAmount ?? 0);
  let paymentDate = '';
  let paymentAccount = '';
  let transactionNo = '';
  let remark = '';
  Modal.confirm({
    title: '费用打款',
    content: () =>
      h(
        'div',
        { style: 'display:flex;flex-direction:column;gap:8px;' },
        [
          h('div', { style: 'display:flex;align-items:center;gap:8px;' }, [
            h('span', { style: 'width:90px;text-align:right;' }, '打款金额：'),
            h('input', {
              type: 'number',
              value: paymentAmount,
              style:
                'flex:1;padding:6px 8px;border:1px solid #d9d9d9;border-radius:4px;',
              onInput: (e: any) => {
                paymentAmount = Number(e.target.value);
              },
            }),
          ]),
          h('div', { style: 'display:flex;align-items:center;gap:8px;' }, [
            h('span', { style: 'width:90px;text-align:right;' }, '打款日期：'),
            h('input', {
              type: 'date',
              style:
                'flex:1;padding:6px 8px;border:1px solid #d9d9d9;border-radius:4px;',
              onInput: (e: any) => {
                paymentDate = e.target.value;
              },
            }),
          ]),
          h('div', { style: 'display:flex;align-items:center;gap:8px;' }, [
            h('span', { style: 'width:90px;text-align:right;' }, '收款账号：'),
            h('input', {
              placeholder: '收款账号（可选）',
              style:
                'flex:1;padding:6px 8px;border:1px solid #d9d9d9;border-radius:4px;',
              onInput: (e: any) => {
                paymentAccount = e.target.value;
              },
            }),
          ]),
          h('div', { style: 'display:flex;align-items:center;gap:8px;' }, [
            h('span', { style: 'width:90px;text-align:right;' }, '交易号：'),
            h('input', {
              placeholder: '第三方交易号（可选）',
              style:
                'flex:1;padding:6px 8px;border:1px solid #d9d9d9;border-radius:4px;',
              onInput: (e: any) => {
                transactionNo = e.target.value;
              },
            }),
          ]),
          h('textarea', {
            placeholder: '备注（可选）',
            style:
              'min-height:60px;padding:8px;border:1px solid #d9d9d9;border-radius:4px;',
            onInput: (e: any) => {
              remark = e.target.value;
            },
          }),
        ],
      ),
    okText: '确认打款',
    cancelText: '取消',
    onOk: async () => {
      try {
        await paymentExpenseApi({
          id: row.id,
          paymentAmount,
          paymentDate: paymentDate || undefined,
          paymentAccount: paymentAccount || undefined,
          transactionNo: transactionNo || undefined,
          remark: remark || undefined,
        });
        window.$message.success('打款成功');
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
    <Grid :table-title="''">
      <template #form-header>
        <Tabs v-model:activeKey="activeTab" class="mb-3" @change="handleTabChange">
          <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('finance:expense:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新建费用申请
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('finance:expense:delete')"
          class="mr-2"
          @click="handleBatchDelete"
        >
          批量删除
        </Button>
      </template>

      <template #expenseNo="{ row }">
        <a
          v-if="accessStore.hasAccessCode('finance:expense:list')"
          class="text-blue-600 cursor-pointer"
          @click="handleView(row)"
        >
          {{ row.expenseNo }}
        </a>
        <span v-else>{{ row.expenseNo }}</span>
      </template>

      <template #expenseType="{ row }">
        <Tag :color="expenseTypeMap[row.expenseType]?.color || 'blue'">
          {{ expenseTypeMap[row.expenseType]?.name || row.expenseTypeName || '-' }}
        </Tag>
      </template>

      <template #totalAmount="{ row }">
        <span class="font-medium text-red-500">
          {{ Number(row.totalAmount ?? row.amount ?? 0).toFixed(2) }}
        </span>
      </template>

      <template #status="{ row }">
        <Tag :color="expenseStatusColorMap[row.status] ?? 'default'">
          {{ expenseStatusLabelMap[row.status] ?? row.status ?? '-' }}
        </Tag>
      </template>

      <template #approvalStatus="{ row }">
        <Tag :color="approvalStatusColorMap[row.approvalStatus] ?? 'default'">
          {{ approvalStatusLabelMap[row.approvalStatus] ?? '未知' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <!-- 查看 -->
        <a
          v-if="accessStore.hasAccessCode('finance:expense:list')"
          class="text-blue-600 cursor-pointer mx-1"
          @click="handleView(row)"
        >查看</a>
        <!-- 编辑：草稿(1)或已驳回(5) -->
        <a
          v-if="
            accessStore.hasAccessCode('finance:expense:save') &&
            (row.status === 1 || row.status === 5)
          "
          class="text-blue-600 cursor-pointer mx-1"
          @click="handleEdit(row)"
        >编辑</a>
        <!-- 提交审批：草稿(1)或已驳回(5) -->
        <a
          v-if="
            accessStore.hasAccessCode('finance:expense:save') &&
            (row.status === 1 || row.status === 5)
          "
          class="text-blue-600 cursor-pointer mx-1"
          @click="handleSubmitApproval(row)"
        >提交审批</a>
        <!-- 审批通过：待审批(2)/审批中(3) -->
        <a
          v-if="
            accessStore.hasAccessCode('finance:expense:approve') &&
            (row.status === 2 || row.status === 3)
          "
          class="text-green-600 cursor-pointer mx-1"
          @click="handleApprove(row)"
        >审批</a>
        <!-- 审批驳回：待审批(2)/审批中(3) -->
        <a
          v-if="
            accessStore.hasAccessCode('finance:expense:approve') &&
            (row.status === 2 || row.status === 3)
          "
          class="text-orange-600 cursor-pointer mx-1"
          @click="handleReject(row)"
        >驳回</a>
        <!-- 打款：已通过(4) -->
        <a
          v-if="
            accessStore.hasAccessCode('finance:expense:payment') &&
            row.status === 4
          "
          class="text-green-600 cursor-pointer mx-1"
          @click="handlePayment(row)"
        >打款</a>
        <!-- 删除：仅草稿(1)/已驳回(5) -->
        <Popconfirm
          v-if="
            accessStore.hasAccessCode('finance:expense:delete') &&
            (row.status === 1 || row.status === 5)
          "
          :title="$t('ui.text.do_you_want_delete', { moduleName: '费用申请' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <a class="text-red-500 cursor-pointer mx-1">删除</a>
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />
  </Page>
</template>
