<script lang="ts" setup>
import { computed, onMounted, ref, watch } from 'vue';

import type { EchartsUIType } from '@vben/plugins/echarts';

import { IconifyIcon } from '@vben/icons';

import { Card, Empty, Spin } from 'ant-design-vue';

import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { getBehaviorMetricsApi } from '#/api';

interface Props {
  year: number;
  month?: number;
  timeDimension: string;
}

const props = defineProps<Props>();

const loading = ref(false);
const data = ref<any>({});

const trendChartRef = ref<EchartsUIType>();
const { renderEcharts: renderTrend } = useEcharts(trendChartRef);

const summaryCards = computed(() => {
  const s = data.value?.summary || {};
  return [
    {
      title: '拜访次数',
      value: s.visitCount || 0,
      icon: 'lucide:map-pin',
      color: '#1890ff',
      bg: '#e6f7ff',
    },
    {
      title: '电话次数',
      value: s.phoneCount || 0,
      icon: 'lucide:phone',
      color: '#52c41a',
      bg: '#f6ffed',
    },
    {
      title: '跟进次数',
      value: s.followUpCount || 0,
      icon: 'lucide:message-square',
      color: '#fa8c16',
      bg: '#fff7e6',
    },
    {
      title: '转化率',
      value: Number(Number(s.conversionRate || 0).toFixed(2)),
      suffix: '%',
      icon: 'lucide:target',
      color: '#eb2f96',
      bg: '#fff0f6',
    },
  ];
});

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getBehaviorMetricsApi({
      year: props.year,
      month: props.month,
      time_dimension: props.timeDimension,
    });
    data.value = res?.data || res || {};
    renderCharts();
  } catch {
    data.value = {};
  } finally {
    loading.value = false;
  }
}

function renderCharts() {
  const trend = data.value?.trend || [];
  if (trend.length === 0) return;
  renderTrend({
    legend: { top: 0 },
    tooltip: { trigger: 'axis' },
    grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
    xAxis: {
      type: 'category',
      data: trend.map((t: any) => t.period),
      boundaryGap: false,
    },
    yAxis: [
      { type: 'value', name: '行为量' },
      { type: 'value', name: '业绩额', position: 'right' },
    ],
    series: [
      {
        name: '拜访',
        type: 'line',
        smooth: true,
        data: trend.map((t: any) => t.visitCount || 0),
        itemStyle: { color: '#1890ff' },
      },
      {
        name: '电话',
        type: 'line',
        smooth: true,
        data: trend.map((t: any) => t.phoneCount || 0),
        itemStyle: { color: '#52c41a' },
      },
      {
        name: '跟进',
        type: 'line',
        smooth: true,
        data: trend.map((t: any) => t.followUpCount || 0),
        itemStyle: { color: '#fa8c16' },
      },
      {
        name: '业绩额',
        type: 'line',
        yAxisIndex: 1,
        smooth: true,
        data: trend.map((t: any) => t.amount || 0),
        itemStyle: { color: '#eb2f96' },
        lineStyle: { type: 'dashed' },
      },
    ],
  });
}

watch(
  () => [props.year, props.month, props.timeDimension],
  () => loadData(),
);

onMounted(() => loadData());
</script>

<template>
  <Card :body-style="{ padding: '16px' }">
    <template #title>
      <div class="flex items-center gap-2">
        <IconifyIcon icon="lucide:activity" class="text-lg text-primary" />
        <span>行为指标 vs 业绩关联</span>
      </div>
    </template>
    <template #extra>
      <span class="text-xs text-gray-400">拜访/电话/跟进 与业绩关联分析</span>
    </template>

    <Spin :spinning="loading">
      <div v-if="!data?.summary" class="py-8">
        <Empty description="暂无行为数据" />
      </div>
      <div v-else>
        <!-- 4 个行为指标卡 -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-4">
          <div
            v-for="card in summaryCards"
            :key="card.title"
            class="metric-card"
            :style="{ background: card.bg }"
          >
            <div class="flex items-center justify-between mb-2">
              <IconifyIcon :icon="card.icon" :style="{ color: card.color, fontSize: '20px' }" />
            </div>
            <div class="text-xs text-gray-600">{{ card.title }}</div>
            <div class="text-xl font-bold mt-1" :style="{ color: card.color }">
              {{ card.value }}<span v-if="card.suffix" class="text-sm">{{ card.suffix }}</span>
            </div>
          </div>
        </div>

        <!-- 行为量 vs 业绩趋势 -->
        <div class="text-sm font-medium mb-2 text-gray-700">
          行为量趋势 vs 业绩趋势
        </div>
        <EchartsUI ref="trendChartRef" height="280px" />
      </div>
    </Spin>
  </Card>
</template>

<style scoped>
.metric-card {
  padding: 14px;
  border-radius: 8px;
  transition: all 0.3s;
}

.metric-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgb(0 0 0 / 10%);
}
</style>
