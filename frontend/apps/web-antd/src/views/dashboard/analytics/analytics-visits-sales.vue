<script lang="ts" setup>
import type { EchartsUIType } from '@vben/plugins/echarts';

import { onMounted, ref } from 'vue';

import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { getCustomerTypeStatsApi } from '#/api';

const chartRef = ref<EchartsUIType>();
const { renderEcharts } = useEcharts(chartRef);

onMounted(async () => {
  const year = new Date().getFullYear();
  let list: any[] = [];
  try {
    list = (await getCustomerTypeStatsApi({ year })) ?? [];
  } catch {
    // empty data
  }

  const xData = (Array.isArray(list) ? list : []).map(
    (item: any) => item.customerType ?? '未知',
  );
  const countData = (Array.isArray(list) ? list : []).map((item: any) =>
    Number(item.totalCount ?? 0),
  );
  const contractData = (Array.isArray(list) ? list : []).map((item: any) =>
    Number(item.contractCount ?? 0),
  );

  renderEcharts({
    grid: { bottom: 0, containLabel: true, left: '3%', right: '4%', top: '10%' },
    legend: { data: ['客户数', '成单数'], top: 0 },
    series: [
      {
        barMaxWidth: 30,
        data: countData,
        name: '客户数',
        type: 'bar',
      },
      {
        barMaxWidth: 30,
        data: contractData,
        name: '成单数',
        type: 'bar',
      },
    ],
    tooltip: { trigger: 'axis' },
    xAxis: {
      data: xData.length > 0 ? xData : ['暂无数据'],
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
