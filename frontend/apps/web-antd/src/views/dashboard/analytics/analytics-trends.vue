<script lang="ts" setup>
import type { EchartsUIType } from '@vben/plugins/echarts';

import { onMounted, ref } from 'vue';

import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { getMonthlyPerformanceApi } from '#/api';

const chartRef = ref<EchartsUIType>();
const { renderEcharts } = useEcharts(chartRef);

onMounted(async () => {
  const year = new Date().getFullYear();
  let months: any[] = [];
  try {
    const resp = await getMonthlyPerformanceApi({ year });
    months = resp?.months ?? [];
  } catch {
    // empty data
  }

  const xData = months.map((m: any) => `${m.month}月`);
  const contractData = months.map((m: any) =>
    Number(m.contractActual ?? 0),
  );
  const paymentData = months.map((m: any) => Number(m.paymentActual ?? 0));

  renderEcharts({
    grid: { bottom: 0, containLabel: true, left: '1%', right: '1%', top: '5%' },
    legend: { data: ['合同额', '回款额'], top: 0 },
    series: [
      {
        areaStyle: {},
        data: contractData,
        itemStyle: { color: '#5ab1ef' },
        name: '合同额',
        smooth: true,
        type: 'line',
      },
      {
        areaStyle: {},
        data: paymentData,
        itemStyle: { color: '#019680' },
        name: '回款额',
        smooth: true,
        type: 'line',
      },
    ],
    tooltip: {
      axisPointer: {
        lineStyle: { color: '#019680', width: 1 },
      },
      trigger: 'axis',
      valueFormatter: (val: any) => `¥${Number(val).toLocaleString()}`,
    },
    xAxis: {
      axisTick: { show: false },
      boundaryGap: false,
      data: xData.length > 0 ? xData : Array.from({ length: 12 }).map((_, i) => `${i + 1}月`),
      type: 'category',
    },
    yAxis: [
      {
        axisTick: { show: false },
        splitArea: { show: true },
        splitNumber: 4,
        type: 'value',
      },
    ],
  });
});
</script>

<template>
  <EchartsUI ref="chartRef" />
</template>
