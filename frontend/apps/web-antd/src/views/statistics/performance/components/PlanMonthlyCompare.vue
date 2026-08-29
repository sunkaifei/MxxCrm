<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { Drawer, Table, Tag } from 'ant-design-vue';

import { getMonthlyCompareApi } from '#/api/core/statistics/performance-plan';
import { $t } from '#/locales';

defineOptions({ name: 'PlanMonthlyCompare' });

const props = defineProps<{
  employeeId?: number;
  employeeName?: string;
  visible: boolean;
  year?: number;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
}>();

interface CompareRow {
  month: number;
  contractTarget: number | string;
  contractActual: number | string;
  paymentTarget: number | string;
  paymentActual: number | string;
  contractRate: number | string;
  paymentRate: number | string;
}

const loading = ref(false);
const rows = ref<CompareRow[]>([]);

const toNum = (v: null | number | string | undefined) => {
  const n = typeof v === 'string' ? Number.parseFloat(v) : (v ?? 0);
  return Number.isFinite(n) ? n : 0;
};

/** 金额缩写：亿/万（万以下保留原值） */
const fmtAmount = (v: null | number | string | undefined) => {
  const num = toNum(v);
  if (num >= 100_000_000)
    return `¥${(num / 100_000_000).toFixed(2)}${$t('page.statistics.performancePlan.yiUnit')}`;
  if (num >= 10_000)
    return `¥${(num / 10_000).toFixed(1)}${$t('page.statistics.performancePlan.wanUnit')}`;
  return `¥${num.toFixed(2)}`;
};

const fmtRate = (v: null | number | string | undefined) => `${toNum(v).toFixed(2)}%`;

const rateColor = (v: null | number | string | undefined) => {
  const n = toNum(v);
  if (n >= 100) return 'green';
  if (n >= 60) return 'blue';
  return 'red';
};

const columns = [
  { title: $t('page.statistics.performancePlan.month'), dataIndex: 'month', key: 'month', width: 70, align: 'center' as const },
  { title: $t('page.statistics.performancePlan.contractTarget'), dataIndex: 'contractTarget', key: 'contractTarget', align: 'right' as const },
  { title: $t('page.statistics.performancePlan.contractActual'), dataIndex: 'contractActual', key: 'contractActual', align: 'right' as const },
  { title: $t('page.statistics.performancePlan.contractRate'), dataIndex: 'contractRate', key: 'contractRate', width: 110, align: 'center' as const },
  { title: $t('page.statistics.performancePlan.paymentTarget'), dataIndex: 'paymentTarget', key: 'paymentTarget', align: 'right' as const },
  { title: $t('page.statistics.performancePlan.paymentActual'), dataIndex: 'paymentActual', key: 'paymentActual', align: 'right' as const },
  { title: $t('page.statistics.performancePlan.paymentRate'), dataIndex: 'paymentRate', key: 'paymentRate', width: 110, align: 'center' as const },
];

const summary = computed(() => {
  const acc = {
    contractTarget: 0,
    contractActual: 0,
    paymentTarget: 0,
    paymentActual: 0,
  };
  for (const r of rows.value) {
    acc.contractTarget += toNum(r.contractTarget);
    acc.contractActual += toNum(r.contractActual);
    acc.paymentTarget += toNum(r.paymentTarget);
    acc.paymentActual += toNum(r.paymentActual);
  }
  return acc;
});

const loadData = async () => {
  if (!props.employeeId || !props.year) return;
  loading.value = true;
  try {
    rows.value = await getMonthlyCompareApi({
      employeeId: props.employeeId,
      year: props.year,
    });
  } finally {
    loading.value = false;
  }
};

watch(
  () => props.visible,
  (v) => {
    if (v) {
      rows.value = [];
      loadData();
    }
  },
);

const handleClose = () => emit('update:visible', false);
</script>

<template>
  <Drawer
    :open="visible"
    :title="
      $t('page.statistics.performancePlan.monthlyCompareTitle', {
        name: employeeName || $t('page.statistics.performancePlan.employee'),
        year: year ?? '',
      })
    "
    width="760px"
    destroy-on-close
    @close="handleClose"
  >
    <Table
      :columns="columns"
      :data-source="rows"
      :loading="loading"
      :pagination="false"
      row-key="month"
      size="small"
      bordered
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'month'">
{{
          $t('page.statistics.performancePlan.monthValue', { n: record.month })
        }}
</template>
        <template v-else-if="column.key === 'contractTarget'">{{ fmtAmount(record.contractTarget) }}</template>
        <template v-else-if="column.key === 'contractActual'">{{ fmtAmount(record.contractActual) }}</template>
        <template v-else-if="column.key === 'contractRate'">
          <Tag :color="rateColor(record.contractRate)">{{ fmtRate(record.contractRate) }}</Tag>
        </template>
        <template v-else-if="column.key === 'paymentTarget'">{{ fmtAmount(record.paymentTarget) }}</template>
        <template v-else-if="column.key === 'paymentActual'">{{ fmtAmount(record.paymentActual) }}</template>
        <template v-else-if="column.key === 'paymentRate'">
          <Tag :color="rateColor(record.paymentRate)">{{ fmtRate(record.paymentRate) }}</Tag>
        </template>
      </template>
      <template #summary>
        <Table.Summary fixed>
          <Table.SummaryRow>
            <Table.SummaryCell :index="0" align="center">
              {{ $t('page.statistics.performancePlan.total') }}
            </Table.SummaryCell>
            <Table.SummaryCell align="right">{{ fmtAmount(summary.contractTarget) }}</Table.SummaryCell>
            <Table.SummaryCell align="right">{{ fmtAmount(summary.contractActual) }}</Table.SummaryCell>
            <Table.SummaryCell align="center">—</Table.SummaryCell>
            <Table.SummaryCell align="right">{{ fmtAmount(summary.paymentTarget) }}</Table.SummaryCell>
            <Table.SummaryCell align="right">{{ fmtAmount(summary.paymentActual) }}</Table.SummaryCell>
            <Table.SummaryCell align="center">—</Table.SummaryCell>
          </Table.SummaryRow>
        </Table.Summary>
      </template>
    </Table>
  </Drawer>
</template>
