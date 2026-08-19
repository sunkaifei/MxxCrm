<script lang="ts" setup>
import type { AnalysisOverviewItem } from '@vben/common-ui';
import type { TabOption } from '@vben/types';

import { onMounted, ref } from 'vue';

import {
  AnalysisChartCard,
  AnalysisChartsTabs,
  AnalysisOverview,
} from '@vben/common-ui';
import {
  LucideBanknote,
  LucideFileText,
  LucideTarget,
  LucideUsers,
} from '@vben/icons';

import { getCustomerFunnelApi, getPaymentCompletionApi } from '#/api';

import AnalyticsTrends from './analytics-trends.vue';
import AnalyticsVisitsData from './analytics-visits-data.vue';
import AnalyticsVisitsSales from './analytics-visits-sales.vue';
import AnalyticsVisitsSource from './analytics-visits-source.vue';
import AnalyticsVisits from './analytics-visits.vue';

const overviewItems = ref<AnalysisOverviewItem[]>([
  {
    icon: LucideUsers,
    title: '客户总数',
    totalTitle: '线索总数',
    totalValue: 0,
    value: 0,
  },
  {
    icon: LucideFileText,
    title: '合同总额',
    totalTitle: '本年合同金额(元)',
    totalValue: 0,
    value: 0,
  },
  {
    icon: LucideBanknote,
    title: '回款总额',
    totalTitle: '本年回款金额(元)',
    totalValue: 0,
    value: 0,
  },
  {
    decimals: 2,
    icon: LucideTarget,
    suffix: '%',
    title: '回款完成率',
    totalTitle: '未回款金额(元)',
    totalValue: 0,
    value: 0,
  },
]);

const chartTabs: TabOption[] = [
  { label: '业绩趋势', value: 'trends' },
  { label: '回款月度趋势', value: 'visits' },
];

const currentYear = new Date().getFullYear();

async function loadOverview() {
  try {
    const [funnel, completion] = await Promise.all([
      getCustomerFunnelApi({ year: currentYear }),
      getPaymentCompletionApi({ year: currentYear }),
    ]);

    const f = funnel ?? {};
    const c = completion ?? {};

    const [
      overviewCustomers,
      overviewContracts,
      overviewPayments,
      overviewRate,
    ] = overviewItems.value;

    if (overviewCustomers) {
      overviewCustomers.value = f.totalCustomers ?? 0;
      overviewCustomers.totalValue = f.totalLeads ?? 0;
    }

    if (overviewContracts) {
      overviewContracts.value = c.totalContractAmount ?? 0;
      overviewContracts.totalValue = c.totalContractAmount ?? 0;
    }

    if (overviewPayments) {
      overviewPayments.value = c.totalPaymentAmount ?? 0;
      overviewPayments.totalValue = c.totalPaymentAmount ?? 0;
    }

    if (overviewRate) {
      overviewRate.value = Number(c.completionRate) || 0;
      overviewRate.totalValue = c.unpaidAmount ?? 0;
    }
  } catch {
    // keep zeros on error
  }
}

onMounted(() => {
  loadOverview();
});
</script>

<template>
  <div class="p-5">
    <AnalysisOverview :items="overviewItems" />
    <AnalysisChartsTabs :tabs="chartTabs" class="mt-5">
      <template #trends>
        <AnalyticsTrends />
      </template>
      <template #visits>
        <AnalyticsVisits />
      </template>
    </AnalysisChartsTabs>

    <div class="mt-5 w-full md:flex">
      <AnalysisChartCard
        class="mt-5 md:mt-0 md:mr-4 md:w-1/3"
        title="客户来源分析"
      >
        <AnalyticsVisitsData />
      </AnalysisChartCard>
      <AnalysisChartCard
        class="mt-5 md:mt-0 md:mr-4 md:w-1/3"
        title="合同状态分析"
      >
        <AnalyticsVisitsSource />
      </AnalysisChartCard>
      <AnalysisChartCard class="mt-5 md:mt-0 md:w-1/3" title="客户类型分布">
        <AnalyticsVisitsSales />
      </AnalysisChartCard>
    </div>
  </div>
</template>
