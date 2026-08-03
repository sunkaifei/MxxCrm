<script lang="ts" setup>
import { onMounted, ref, watch } from 'vue';

import type { EchartsUIType } from '@vben/plugins/echarts';

import { IconifyIcon } from '@vben/icons';

import { Card, Empty, Spin, Table, Tag } from 'ant-design-vue';

import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { getCustomerBreakdownApi } from '#/api';

interface Props {
  year: number;
  month?: number;
  timeDimension: string;
}

const props = defineProps<Props>();

const loading = ref(false);
const data = ref<any>({});

const newVsOldChartRef = ref<EchartsUIType>();
const abcChartRef = ref<EchartsUIType>();
const { renderEcharts: renderNewVsOld } = useEcharts(newVsOldChartRef);
const { renderEcharts: renderAbc } = useEcharts(abcChartRef);

const top10Columns = [
  { title: '排名', dataIndex: 'rank', width: 70 },
  { title: '客户名称', dataIndex: 'customerName' },
  { title: '贡献金额', dataIndex: 'amount', align: 'right' as const, width: 130 },
  {
    title: '同比',
    dataIndex: 'growth',
    align: 'right' as const,
    width: 100,
  },
];

function formatCurrency(val?: number) {
  if (!val) return '¥0';
  if (val >= 10000) return `¥${(val / 10000).toFixed(1)}万`;
  return `¥${val.toLocaleString()}`;
}

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getCustomerBreakdownApi({
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
  // 新老客户饼图
  const newVsOld = data.value?.newVsOld || [];
  renderNewVsOld({
    legend: { bottom: 0, orient: 'horizontal' },
    tooltip: {
      trigger: 'item',
      formatter: '{b}: {c} ({d}%)',
    },
    series: [
      {
        type: 'pie',
        radius: ['40%', '70%'],
        data:
          newVsOld.length > 0
            ? newVsOld.map((i: any) => ({ name: i.name, value: i.amount }))
            : [{ name: '暂无', value: 1 }],
        label: { formatter: '{b}\n{d}%', show: true },
        itemStyle: { borderRadius: 6, borderColor: '#fff', borderWidth: 2 },
      },
    ],
    color: ['#1890ff', '#52c41a'],
  });

  // ABC 分级饼图
  const abc = data.value?.abcDistribution || [];
  renderAbc({
    legend: { bottom: 0, orient: 'horizontal' },
    tooltip: {
      trigger: 'item',
      formatter: '{b}: {c} ({d}%)',
    },
    series: [
      {
        type: 'pie',
        radius: ['40%', '70%'],
        data:
          abc.length > 0
            ? abc.map((i: any) => ({ name: i.name, value: i.amount }))
            : [{ name: '暂无', value: 1 }],
        label: { formatter: '{b}\n{d}%', show: true },
        itemStyle: { borderRadius: 6, borderColor: '#fff', borderWidth: 2 },
      },
    ],
    color: ['#ff4d4f', '#faad14', '#1890ff'],
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
        <IconifyIcon icon="lucide:users" class="text-lg text-primary" />
        <span>客户维度拆解</span>
      </div>
    </template>

    <Spin :spinning="loading">
      <div v-if="!data?.newVsOld && !data?.abcDistribution" class="py-8">
        <Empty description="暂无客户拆解数据" />
      </div>
      <div v-else class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <!-- 新老客户 -->
        <div>
          <div class="text-sm font-medium mb-2 text-gray-700">新客户 vs 老客户业绩占比</div>
          <EchartsUI ref="newVsOldChartRef" height="220px" />
        </div>
        <!-- ABC 分级 -->
        <div>
          <div class="text-sm font-medium mb-2 text-gray-700">A/B/C 客户分级贡献</div>
          <EchartsUI ref="abcChartRef" height="220px" />
        </div>
      </div>

      <!-- Top10 客户排行 -->
      <div v-if="data?.top10?.length > 0" class="mt-4">
        <div class="text-sm font-medium mb-2 text-gray-700">Top 10 客户贡献排行</div>
        <Table
          :columns="top10Columns"
          :data-source="data?.top10 || []"
          :pagination="false"
          row-key="rank"
          size="small"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.dataIndex === 'amount'">
              {{ formatCurrency(record.amount) }}
            </template>
            <template v-else-if="column.dataIndex === 'growth'">
              <Tag :color="record.growth >= 0 ? 'green' : 'red'">
                {{ record.growth >= 0 ? '+' : '' }}{{ Number(record.growth || 0).toFixed(2) }}%
              </Tag>
            </template>
          </template>
        </Table>
      </div>
    </Spin>
  </Card>
</template>
