<script lang="ts" setup>
import type { EchartsUIType } from '@vben/plugins/echarts';

import { computed, nextTick, onMounted, ref, watch } from 'vue';

import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { IconifyIcon } from '@vben/icons';

import { Card, Empty, Spin, Statistic, Tag } from 'ant-design-vue';

import { getSalesFunnelApi } from '#/api';

interface Props {
  year: number;
  month?: number;
  timeDimension: string;
}

const props = defineProps<Props>();

const loading = ref(false);
const data = ref<any>({});

const chartRef = ref<EchartsUIType>();
const { renderEcharts } = useEcharts(chartRef);

function formatCurrency(val?: any) {
  const num = Number(val);
  if (!num || Number.isNaN(num)) return '¥0';
  if (num >= 100000000) return `¥${(num / 100000000).toFixed(2)}亿`;
  if (num >= 10000) return `¥${(num / 10000).toFixed(1)}万`;
  return `¥${num.toLocaleString()}`;
}

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getSalesFunnelApi({
      year: props.year,
      month: props.month,
      time_dimension: props.timeDimension,
    });
    data.value = res?.data || res || {};
    await nextTick();
    renderChart();
  } catch {
    data.value = {};
  } finally {
    loading.value = false;
  }
}

function renderChart() {
  const stages = data.value?.stages || [];
  const funnelData: { name: string; value: number; amount: number; conversionRate: number }[] = stages.map((s: any) => ({
    name: s.stage,
    value: Number(s.count || 0),
    amount: Number(s.amount || 0),
    conversionRate: Number(s.conversionRate || 0),
  }));

  renderEcharts({
    tooltip: {
      trigger: 'item',
      formatter: (params: any) => {
        const d = params.data;
        const lines = [
          `<b>${d.name}</b>`,
          `数量: ${d.value} 个`,
        ];
        if (d.amount) lines.push(`金额: ${formatCurrency(d.amount)}`);
        if (d.conversionRate)
          lines.push(`转化率: ${Number(d.conversionRate).toFixed(2)}%`);
        return lines.join('<br/>');
      },
    },
    legend: {
      data: funnelData.map((d) => d.name),
      top: 8,
      itemGap: 16,
    },
    grid: { top: 60 },
    series: [
      {
        name: '销售漏斗',
        type: 'funnel',
        left: '10%',
        top: 60,
        bottom: 20,
        width: '80%',
        min: 0,
        max: funnelData.length
          ? Math.max(...funnelData.map((d) => d.value), 1)
          : 1,
        minSize: '20%',
        maxSize: '100%',
        sort: 'descending',
        gap: 2,
        label: {
          show: true,
          position: 'inside',
          formatter: (params: any) => {
            const d = params.data;
            return `${d.name}\n${d.value} 个`;
          },
          color: '#fff',
          fontSize: 12,
          fontWeight: 600,
        },
        labelLine: { length: 10, lineStyle: { width: 1, type: 'solid' } },
        itemStyle: {
          borderColor: '#fff',
          borderWidth: 1,
        },
        emphasis: {
          label: { fontSize: 14 },
        },
        data: funnelData,
                color: [
                  '#1890ff',
                  '#13c2c2',
                  '#722ed1',
                  '#eb2f96',
                  '#fa8c16',
                  '#52c41a',
                ],
              },
    ],
  });
}

const stages = computed(() => data.value?.stages || []);

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
        <IconifyIcon icon="lucide:filter" class="text-lg text-primary" />
        <span>销售漏斗转化分析</span>
      </div>
    </template>
    <template #extra>
      <span class="text-xs text-gray-400">5阶段转化率+平均周期+赢单率</span>
    </template>

    <Spin :spinning="loading">
      <div v-if="stages.length === 0" class="py-8">
        <Empty description="暂无漏斗数据" />
      </div>

      <div v-else>
        <!-- 漏斗图 -->
        <EchartsUI ref="chartRef" height="320px" />

        <!-- 关键指标 -->
        <div class="grid grid-cols-3 gap-3 mt-4 pt-4 border-t border-gray-100">
          <Statistic
            title="平均成交周期"
            :value="data.avgCycleDays || 0"
            suffix="天"
          />
          <Statistic
            title="赢单率"
            :value="data.winRate || 0"
            :precision="2"
            suffix="%"
            :value-style="{
              color: (data.winRate || 0) >= 30 ? '#52c41a' : '#faad14',
            }"
          />
          <Statistic title="总线索数" :value="data.totalLeads || 0" />
        </div>

        <!-- 健康度提示 -->
        <div class="mt-3 flex items-center justify-end gap-2">
          <Tag :color="(data.winRate || 0) >= 30 ? 'success' : 'warning'">
            赢单率{{ (data.winRate || 0) >= 30 ? '健康' : '偏低' }}
          </Tag>
          <Tag :color="(data.avgCycleDays || 0) <= 30 ? 'success' : 'warning'">
            周期{{ (data.avgCycleDays || 0) <= 30 ? '合理' : '偏长' }}
          </Tag>
        </div>
      </div>
    </Spin>
  </Card>
</template>
