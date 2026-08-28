<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import { Button, Modal, Popconfirm, Tabs, Tag } from 'ant-design-vue';

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
import { PageUsageGuide } from '#/components/PageUsageGuide';
import { useDataScopeTabs } from '#/composables/use-data-scope-tabs';
import { $t } from '#/locales';

import ExpenseDrawer from './drawer.vue';

const guideStepCount = 5;
const accessStore = useAccessStore();

// 全部/下属 Tab 显示条件：超管（user_type=1）或系统管理员（data_scope=1）→ 全部；部门级（2/3/4）→ 含下属
const { canViewAll, canViewSubordinate } = useDataScopeTabs();

// 是否为下属视图（下属视图下只能查看，不能操作）
const isSubordinateView = computed(() => activeTab.value === 'subordinate');

const allTabList = [
  { key: 'all', labelKey: 'page.finance.expense.tab.all' },
  { key: 'my', labelKey: 'page.finance.expense.tab.my' },
  { key: 'subordinate', labelKey: 'page.finance.expense.tab.subordinate' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList
    .filter((t) => keys.includes(t.key))
    .map((t) => ({ key: t.key, label: $t(t.labelKey) }));
});

const activeTab = ref('my');

function handleTabChange(key: number | string) {
  activeTab.value = key as string;
  gridApi.query();
}

// 费用状态映射：1=草稿,2=待审批,3=审批中,4=已通过,5=已驳回,6=已打款
const expenseStatusOptions = [
  { label: $t('page.finance.expense.status.draft'), value: 1 },
  { label: $t('page.finance.expense.status.pending'), value: 2 },
  { label: $t('page.finance.expense.status.approving'), value: 3 },
  { label: $t('page.finance.expense.status.approved'), value: 4 },
  { label: $t('page.finance.expense.status.rejected'), value: 5 },
  { label: $t('page.finance.expense.status.paid'), value: 6 },
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
  1: $t('page.finance.expense.status.draft'),
  2: $t('page.finance.expense.status.pending'),
  3: $t('page.finance.expense.status.approving'),
  4: $t('page.finance.expense.status.approved'),
  5: $t('page.finance.expense.status.rejected'),
  6: $t('page.finance.expense.status.paid'),
};

// 审批状态映射：0=草稿,1=待审批,2=审批中,3=已通过,4=已驳回
const approvalStatusColorMap: Record<number, string> = {
  0: 'default',
  1: 'processing',
  2: 'warning',
  3: 'success',
  4: 'error',
};

const approvalStatusLabelMap: Record<number, string> = {
  0: $t('page.finance.expense.status.draft'),
  1: $t('page.finance.expense.status.pending'),
  2: $t('page.finance.expense.status.approving'),
  3: $t('page.finance.expense.status.approved'),
  4: $t('page.finance.expense.status.rejected'),
};

// 费用类型选项（从 API 加载）
const expenseTypeOptions = ref<{ label: string; value: number }[]>([]);
const expenseTypeMap = ref<Record<number, { color: string; name: string }>>({});

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
    const map: Record<number, { color: string; name: string }> = {};
    arr.forEach((t: any) => {
      map[t.id] = {
        name: t.typeName || t.name || '',
        color: t.color || 'blue',
      };
    });
    expenseTypeMap.value = map;
  } catch (error) {
    console.error('[expense] load expense types failed:', error);
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
      label: $t('page.finance.expense.column.expenseTitle'),
      componentProps: {
        placeholder: $t('page.finance.expense.search.keywordsPlaceholder'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'expenseType',
      label: $t('page.finance.expense.column.expenseType'),
      componentProps: {
        placeholder: $t('page.finance.common.all'),
        allowClear: true,
        options: expenseTypeOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.finance.expense.column.status'),
      componentProps: {
        placeholder: $t('page.finance.common.all'),
        allowClear: true,
        options: expenseStatusOptions,
      },
    },
    {
      component: 'RangePicker',
      fieldName: 'dateRange',
      label: $t('page.finance.expense.column.applyTime'),
      componentProps: {
        placeholder: [
          $t('page.finance.expense.search.startDate'),
          $t('page.finance.expense.search.endDate'),
        ],
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
    { type: 'checkbox', width: 50 },
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 60,
      headerAlign: 'center',
    },
    {
      title: $t('page.finance.expense.column.expenseNo'),
      field: 'expenseNo',
      width: 170,
      headerAlign: 'center',
      slots: { default: 'expenseNo' },
    },
    {
      title: $t('page.finance.expense.column.expenseTitle'),
      field: 'title',
      width: 200,
      headerAlign: 'center',
    },
    {
      title: $t('page.finance.expense.column.expenseType'),
      field: 'expenseType',
      width: 120,
      headerAlign: 'center',
      slots: { default: 'expenseType' },
    },
    {
      title: $t('page.finance.expense.column.amount'),
      field: 'totalAmount',
      width: 130,
      headerAlign: 'center',
      slots: { default: 'totalAmount' },
    },
    {
      title: $t('page.finance.expense.column.applicant'),
      field: 'applicantName',
      width: 100,
      headerAlign: 'center',
    },
    {
      title: $t('page.finance.expense.column.status'),
      field: 'status',
      width: 100,
      headerAlign: 'center',
      slots: { default: 'status' },
    },
    {
      title: $t('page.finance.expense.column.approvalStatus'),
      field: 'approvalStatus',
      width: 100,
      headerAlign: 'center',
      slots: { default: 'approvalStatus' },
    },
    {
      title: $t('page.finance.expense.column.applyDate'),
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
    window.$message.warning($t('page.finance.expense.message.selectToDelete'));
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
    title: $t('page.finance.expense.modal.submitTitle'),
    content: $t('page.finance.expense.modal.submitContent'),
    okText: $t('page.finance.common.confirm'),
    cancelText: $t('page.finance.common.cancel'),
    onOk: async () => {
      try {
        await submitExpenseApi(row.id);
        window.$message.success($t('page.finance.expense.message.submitted'));
        gridApi.query();
      } catch {
        window.$message.error($t('page.finance.expense.message.submitFailed'));
      }
    },
  });
}

// 审批通过：待审批(2)/审批中(3)状态
async function handleApprove(row: any) {
  Modal.confirm({
    title: $t('page.finance.expense.modal.approveTitle'),
    content: $t('page.finance.expense.modal.approveContent'),
    okText: $t('page.finance.common.confirm'),
    cancelText: $t('page.finance.common.cancel'),
    onOk: async () => {
      try {
        await approveExpenseApi(row.id);
        window.$message.success($t('page.finance.expense.message.approved'));
        gridApi.query();
      } catch {
        window.$message.error($t('page.finance.common.failed'));
      }
    },
  });
}

// 审批驳回：待审批(2)/审批中(3)状态
async function handleReject(row: any) {
  let reason = '';
  Modal.confirm({
    title: $t('page.finance.expense.modal.rejectTitle'),
    content: () =>
      h('div', { style: 'display:flex;flex-direction:column;gap:8px;' }, [
        h('span', $t('page.finance.expense.modal.rejectContent')),
        h('textarea', {
          placeholder: $t('page.finance.expense.modal.rejectReasonPlaceholder'),
          style:
            'min-height:80px;padding:8px;border:1px solid #d9d9d9;border-radius:4px;',
          onInput: (e: any) => {
            reason = e.target.value;
          },
        }),
      ]),
    okText: $t('page.finance.expense.modal.confirmReject'),
    cancelText: $t('page.finance.common.cancel'),
    onOk: async () => {
      try {
        await rejectExpenseApi(row.id, reason);
        window.$message.success($t('page.finance.expense.message.rejected'));
        gridApi.query();
      } catch {
        window.$message.error($t('page.finance.common.failed'));
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
    title: $t('page.finance.expense.modal.paymentTitle'),
    content: () =>
      h('div', { style: 'display:flex;flex-direction:column;gap:8px;' }, [
        h('div', { style: 'display:flex;align-items:center;gap:8px;' }, [
          h(
            'span',
            { style: 'width:90px;text-align:right;' },
            $t('page.finance.expense.modal.paymentAmount'),
          ),
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
          h(
            'span',
            { style: 'width:90px;text-align:right;' },
            $t('page.finance.expense.modal.paymentDate'),
          ),
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
          h(
            'span',
            { style: 'width:90px;text-align:right;' },
            $t('page.finance.expense.modal.paymentAccount'),
          ),
          h('input', {
            placeholder: $t(
              'page.finance.expense.modal.paymentAccountPlaceholder',
            ),
            style:
              'flex:1;padding:6px 8px;border:1px solid #d9d9d9;border-radius:4px;',
            onInput: (e: any) => {
              paymentAccount = e.target.value;
            },
          }),
        ]),
        h('div', { style: 'display:flex;align-items:center;gap:8px;' }, [
          h(
            'span',
            { style: 'width:90px;text-align:right;' },
            $t('page.finance.expense.modal.transactionNo'),
          ),
          h('input', {
            placeholder: $t(
              'page.finance.expense.modal.transactionNoPlaceholder',
            ),
            style:
              'flex:1;padding:6px 8px;border:1px solid #d9d9d9;border-radius:4px;',
            onInput: (e: any) => {
              transactionNo = e.target.value;
            },
          }),
        ]),
        h('textarea', {
          placeholder: $t('page.finance.expense.modal.remarkPlaceholder'),
          style:
            'min-height:60px;padding:8px;border:1px solid #d9d9d9;border-radius:4px;',
          onInput: (e: any) => {
            remark = e.target.value;
          },
        }),
      ]),
    okText: $t('page.finance.expense.modal.confirmPayment'),
    cancelText: $t('page.finance.common.cancel'),
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
        window.$message.success($t('page.finance.expense.message.paid'));
        gridApi.query();
      } catch {
        window.$message.error($t('page.finance.common.failed'));
      }
    },
  });
}
</script>

<template>
  <Page>
    <PageUsageGuide
      :title="$t('page.finance.expense.guide.title')"
      :brief="$t('page.finance.expense.guide.brief')"
      :expand-text="$t('page.finance.expense.guide.expand')"
      :collapse-text="$t('page.finance.expense.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.expense.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.expense.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
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
            accessStore.hasAccessCode('finance:expense:save')
          "
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.finance.expense.button.create') }}
        </Button>
        <Button
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('finance:expense:delete')
          "
          class="mr-2"
          @click="handleBatchDelete"
        >
          {{ $t('page.finance.expense.message.batchDelete') }}
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
          {{
            expenseTypeMap[row.expenseType]?.name || row.expenseTypeName || '-'
          }}
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
          {{
            approvalStatusLabelMap[row.approvalStatus] ??
            $t('page.finance.expense.message.unknown')
          }}
        </Tag>
      </template>

      <template #action="{ row }">
        <!-- 查看 -->
        <a
          v-if="accessStore.hasAccessCode('finance:expense:list')"
          class="text-blue-600 cursor-pointer mx-1"
          @click="handleView(row)"
          >{{ $t('page.finance.common.view') }}</a
        >
        <!-- 编辑：草稿(1)或已驳回(5) -->
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('finance:expense:save') &&
            (row.status === 1 || row.status === 5)
          "
          class="text-blue-600 cursor-pointer mx-1"
          @click="handleEdit(row)"
          >{{ $t('page.finance.common.edit') }}</a
        >
        <!-- 提交审批：草稿(1)或已驳回(5) -->
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('finance:expense:save') &&
            (row.status === 1 || row.status === 5)
          "
          class="text-blue-600 cursor-pointer mx-1"
          @click="handleSubmitApproval(row)"
          >{{ $t('page.finance.expense.button.submit') }}</a
        >
        <!-- 审批通过：待审批(2)/审批中(3) -->
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('finance:expense:audit') &&
            (row.status === 2 || row.status === 3)
          "
          class="text-green-600 cursor-pointer mx-1"
          @click="handleApprove(row)"
          >{{ $t('page.finance.expense.button.approve') }}</a
        >
        <!-- 审批驳回：待审批(2)/审批中(3) -->
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('finance:expense:audit') &&
            (row.status === 2 || row.status === 3)
          "
          class="text-orange-600 cursor-pointer mx-1"
          @click="handleReject(row)"
          >{{ $t('page.finance.expense.button.reject') }}</a
        >
        <!-- 打款：已通过(4) -->
        <a
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('finance:expense:payment') &&
            row.status === 4
          "
          class="text-green-600 cursor-pointer mx-1"
          @click="handlePayment(row)"
          >{{ $t('page.finance.expense.button.payment') }}</a
        >
        <!-- 删除：仅草稿(1)/已驳回(5) -->
        <Popconfirm
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('finance:expense:delete') &&
            (row.status === 1 || row.status === 5)
          "
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.finance.expense.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <a class="text-red-500 cursor-pointer mx-1">{{
            $t('page.finance.common.delete')
          }}</a>
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />
  </Page>
</template>
