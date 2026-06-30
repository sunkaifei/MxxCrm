<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';

import {
  Button,
  InputNumber,
  Modal,
  Select,
  Tag,
  message,
} from 'ant-design-vue';
import { RefreshCw } from 'lucide-vue-next';
import { useRouter } from 'vue-router';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  approveSalaryApi,
  batchApproveSalaryApi,
  batchPaySalaryApi,
  calculateSalaryApi,
  getSalaryListApi,
  paySalaryApi,
} from '#/api/core/finance';
import { $t } from '#/locales';

const router = useRouter();

const statusMap: Record<number, { label: string; color: string }> = {
  0: { label: '待审核', color: 'blue' },
  1: { label: '已审核', color: 'orange' },
  2: { label: '已发放', color: 'green' },
};

const statusOptions = [
  { value: 0, label: '待审核' },
  { value: 1, label: '已审核' },
  { value: 2, label: '已发放' },
];

const monthOptions = Array.from({ length: 12 }, (_, i) => ({
  value: i + 1,
  label: `${i + 1}月`,
}));

const now = new Date();

const calcVisible = ref(false);
const calcLoading = ref(false);
const calcForm = reactive({
  year: now.getFullYear(),
  month: now.getMonth() + 1,
});

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
      component: 'InputNumber',
      fieldName: 'year',
      label: '年份',
      defaultValue: now.getFullYear(),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        min: 2020,
        max: 2099,
        style: { width: '100%' },
      },
    },
    {
      component: 'Select',
      fieldName: 'month',
      label: '月份',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: monthOptions,
      },
    },
    {
      component: 'Input',
      fieldName: 'employeeName',
      label: '员工姓名',
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
  checkboxConfig: {
    highlight: true,
  },
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getSalaryListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
  },

  columns: [
    {
      type: 'checkbox',
      width: 50,
    },
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: '员工姓名',
      field: 'employeeName',
      minWidth: 120,
    },
    {
      title: '部门',
      field: 'deptName',
      minWidth: 120,
    },
    {
      title: '年月',
      field: 'year',
      minWidth: 120,
      slots: { default: 'yearMonth' },
    },
    {
      title: '底薪',
      field: 'baseSalary',
      width: 120,
      slots: { default: 'baseSalary' },
    },
    {
      title: '提成金额',
      field: 'commissionAmount',
      width: 120,
      slots: { default: 'commissionAmount' },
    },
    {
      title: '绩效奖金',
      field: 'performanceBonus',
      width: 120,
      slots: { default: 'performanceBonus' },
    },
    {
      title: '扣款',
      field: 'deduction',
      width: 120,
      slots: { default: 'deduction' },
    },
    {
      title: '应发工资',
      field: 'totalAmount',
      width: 130,
      slots: { default: 'totalAmount' },
    },
    {
      title: '状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 240,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function goDetail(row: any) {
  void router.push({ path: `/finance/salary/detail/${row.id}` });
}

async function handleApprove(row: any) {
  row.pending = true;
  try {
    await approveSalaryApi(row.id);
    message.success('审核成功');
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '审核失败');
  } finally {
    row.pending = false;
  }
}

async function handlePay(row: any) {
  row.pending = true;
  try {
    await paySalaryApi(row.id);
    message.success('发放成功');
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '发放失败');
  } finally {
    row.pending = false;
  }
}

function handleAdjust(row: any) {
  goDetail(row);
}

async function handleCalculate() {
  calcLoading.value = true;
  try {
    await calculateSalaryApi({
      year: calcForm.year,
      month: calcForm.month,
    });
    message.success('核算完成');
    calcVisible.value = false;
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '核算失败');
  } finally {
    calcLoading.value = false;
  }
}

async function handleBatchApprove() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    message.warning('请选择需要审核的记录');
    return;
  }
  const ids = records.map((r: any) => r.id);
  try {
    await batchApproveSalaryApi(ids);
    message.success('批量审核成功');
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '批量审核失败');
  }
}

async function handleBatchPay() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    message.warning('请选择需要发放的记录');
    return;
  }
  const ids = records.map((r: any) => r.id);
  try {
    await batchPaySalaryApi(ids);
    message.success('批量发放成功');
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '批量发放失败');
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="工资核算">
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" @click="calcVisible = true">
          执行核算
        </Button>
        <Button class="mr-2" @click="handleBatchApprove">
          批量审核
        </Button>
        <Button class="mr-2" @click="handleBatchPay">
          批量发放
        </Button>
        <Button class="mr-2" :icon="h(RefreshCw)" @click="gridApi.query()">
          刷新
        </Button>
      </template>

      <template #yearMonth="{ row }">
        {{ row.year }}年{{ row.month }}月
      </template>

      <template #baseSalary="{ row }">
        {{ formatMoney(row.baseSalary) }}
      </template>
      <template #commissionAmount="{ row }">
        {{ formatMoney(row.commissionAmount) }}
      </template>
      <template #performanceBonus="{ row }">
        {{ formatMoney(row.performanceBonus) }}
      </template>
      <template #deduction="{ row }">
        {{ formatMoney(row.deduction) }}
      </template>
      <template #totalAmount="{ row }">
        <span class="font-medium">{{ formatMoney(row.totalAmount) }}</span>
      </template>

      <template #status="{ row }">
        <Tag :color="statusMap[row.status]?.color || 'default'">
          {{ statusMap[row.status]?.label || row.status }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button type="link" @click="goDetail(row)">查看详情</Button>
        <Button
          v-if="row.status === 0"
          type="link"
          :loading="row.pending"
          @click="handleApprove(row)"
        >
          审核
        </Button>
        <Button
          v-if="row.status === 0"
          type="link"
          @click="handleAdjust(row)"
        >
          调整
        </Button>
        <Button
          v-if="row.status === 1"
          type="link"
          :loading="row.pending"
          @click="handlePay(row)"
        >
          发放
        </Button>
      </template>
    </Grid>

    <Modal
      v-model:open="calcVisible"
      title="执行工资核算"
      :confirm-loading="calcLoading"
      @ok="handleCalculate"
    >
      <div class="flex items-center gap-3 py-4">
        <span>年份</span>
        <InputNumber
          v-model:value="calcForm.year"
          :min="2020"
          :max="2099"
          style="width: 120px"
        />
        <span>月份</span>
        <Select
          v-model:value="calcForm.month"
          :options="monthOptions"
          style="width: 120px"
        />
      </div>
    </Modal>
  </Page>
</template>
