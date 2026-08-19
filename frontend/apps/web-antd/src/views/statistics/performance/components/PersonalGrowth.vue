<script lang="ts" setup>
import type { EchartsUIType } from '@vben/plugins/echarts';

import { onMounted, ref } from 'vue';

import { IconifyIcon } from '@vben/icons';
import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { Card, Empty, Spin, Statistic } from 'ant-design-vue';

import { getPersonalGrowthApi } from '#/api';

const loading = ref(false);
const data = ref<any>({});

const growthChartRef = ref<EchartsUIType>();
const { renderEcharts: renderGrowth } = useEcharts(growthChartRef);

function formatCurrency(val?: number) {
  if (!val) return '¥0';
  if (val >= 100_000_000) return `¥${(val / 100_000_000).toFixed(2)}亿`;
  if (val >= 10_000) return `¥${(val / 10_000).toFixed(1)}万`;
  return `¥${val.toLocaleString()}`;
}

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getPersonalGrowthApi({});
    data.value = res?.data || res || {};
    renderCharts();
  } catch {
    data.value = {};
  } finally {
    loading.value = false;
  }
}

function renderCharts() {
  const monthlyTrend = data.value?.monthlyTrend || [];
  if (monthlyTrend.length === 0) return;
  renderGrowth({
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const p = params[0];
        return `${p.axisValue}<br/>${formatCurrency(p.value)}`;
      },
    },
    grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
    xAxis: {
      type: 'category',
      data: monthlyTrend.map((m: any) => m.month),
      axisLabel: { rotate: 35 },
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        formatter: (val: number) => formatCurrency(val),
      },
    },
    series: [
      {
        type: 'line',
        smooth: true,
        data: monthlyTrend.map((m: any) => m.amount || 0),
        areaStyle: {
          color: {
            type: 'linear',
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              { offset: 0, color: 'rgba(24, 144, 255, 0.5)' },
              { offset: 1, color: 'rgba(24, 144, 255, 0.05)' },
            ],
          },
        },
        lineStyle: { color: '#1890ff', width: 2 },
        itemStyle: { color: '#1890ff' },
        markPoint: {
          data: [
            { type: 'max', name: '历史最佳' },
            { type: 'min', name: '最低' },
          ],
        },
        markLine: {
          data: [{ type: 'average', name: '平均值' }],
        },
      },
    ],
  });
}

onMounted(() => loadData());
</script>

<template>
  <Card :body-style="{ padding: '16px' }">
    <template #title>
      <div class="flex items-center gap-2">
        <IconifyIcon icon="lucide:user-check" class="text-lg text-primary" />
        <span>个人成长档案</span>
      </div>
    </template>
    <template #extra>
      <span v-if="data?.hireDate" class="text-xs text-gray-400">
        入职时间：{{ data.hireDate }}
      </span>
    </template>

    <Spin :spinning="loading">
      <div v-if="!data?.monthlyTrend" class="py-8">
        <Empty description="暂无成长数据" />
      </div>
      <div v-else>
        <!-- 关键指标 -->
        <div class="grid grid-cols-3 gap-3 mb-4">
          <Statistic
            title="累计成交金额"
            :value="data.totalAmount || 0"
            :value-style="{ color: '#1890ff' }"
            :formatter="() => formatCurrency(data.totalAmount || 0)"
          />
          <Statistic
            title="累计合同数"
            :value="data.totalContractCount || 0"
            suffix="份"
            :value-style="{ color: '#52c41a' }"
          />
          <Statistic
            title="历史最佳月份"
            :value="data.bestMonth?.month || '-'"
            :value-style="{ color: '#fa8c16', fontSize: '16px' }"
          />
        </div>

        <!-- 成长曲线 -->
        <div class="text-sm font-medium mb-2 text-gray-700">
          入职以来月度业绩成长曲线
        </div>
        <EchartsUI ref="growthChartRef" height="280px" />
      </div>
    </Spin>
  </Card>
</template>
