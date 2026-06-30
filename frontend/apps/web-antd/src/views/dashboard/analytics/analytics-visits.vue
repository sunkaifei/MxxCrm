<script lang="ts" setup>
import type { EchartsUIType } from '@vben/plugins/echarts';

import { onMounted, ref } from 'vue';

import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { getPaymentMonthlyTrendApi } from '#/api';

const chartRef = ref<EchartsUIType>();
const { renderEcharts } = useEcharts(chartRef);

onMounted(async () => {
  const year = new Date().getFullYear();
  let months: any[] = [];
  try {
    const resp = await getPaymentMonthlyTrendApi({ year });
    months = resp?.months ?? [];
  } catch {
    // empty data
  }

  const xData = months.map((m: any) => `${m.month}月`);
  const paymentData = months.map((m: any) => Number(m.paymentAmount ?? 0));
  const contractData = months.map((m: any) => Number(m.contractAmount ?? 0));

  renderEcharts({
    grid: { bottom: 0, containLabel: true, left: '1%', right: '1%', top: '5%' },
    legend: { data: ['合同额', '回款额'], top: 0 },
    series: [
      {
        barMaxWidth: 30,
        data: contractData,
        name: '合同额',
        type: 'bar',
      },
      {
        barMaxWidth: 30,
        data: paymentData,
        name: '回款额',
        type: 'bar',
      },
    ],
    tooltip: {
      axisPointer: { lineStyle: { width: 1 } },
      trigger: 'axis',
      valueFormatter: (val: any) => `¥${Number(val).toLocaleString()}`,
    },
    xAxis: {
      data: xData.length > 0 ? xData : Array.from({ length: 12 }).map((_, i) => `${i + 1}月`),
      type: 'category',
    },
    yAxis: {
      splitNumber: 4,
      type: 'value',
    },
  });
});
</script>

<template>
  <EchartsUI ref="chartRef" />
</template>
