<script lang="ts" setup>
import type { EchartsUIType } from '@vben/plugins/echarts';

import { onMounted, ref } from 'vue';

import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { getCustomerSourceStatsApi } from '#/api';

const chartRef = ref<EchartsUIType>();
const { renderEcharts } = useEcharts(chartRef);

onMounted(async () => {
  const year = new Date().getFullYear();
  let list: any[] = [];
  try {
    list = (await getCustomerSourceStatsApi({ year })) ?? [];
  } catch {
    // empty data
  }

  const pieData = (Array.isArray(list) ? list : []).map((item: any) => ({
    name: item.source ?? '未知',
    value: Number(item.totalCount ?? 0),
  }));

  renderEcharts({
    legend: { bottom: 0, orient: 'horizontal' },
    series: [
      {
        avoidLabelOverlap: true,
        data: pieData.length > 0 ? pieData : [{ name: '暂无数据', value: 1 }],
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)',
          },
        },
        itemStyle: {
          borderRadius: 6,
          borderColor: '#fff',
          borderWidth: 2,
        },
        label: { formatter: '{b}: {c}', show: true },
        radius: ['40%', '65%'],
        type: 'pie',
      },
    ],
    tooltip: { trigger: 'item' },
  });
});
</script>

<template>
  <EchartsUI ref="chartRef" />
</template>
