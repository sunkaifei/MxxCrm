<script lang="ts" setup>
import { onMounted, ref, watch } from 'vue';

import type { EchartsUIType } from '@vben/plugins/echarts';

import { IconifyIcon } from '@vben/icons';

import { Card, Empty, Spin, Table } from 'ant-design-vue';

import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { getProductBreakdownApi } from '#/api';

interface Props {
  year: number;
  month?: number;
  timeDimension: string;
}

const props = defineProps<Props>();

const loading = ref(false);
const data = ref<any>({});

const categoryChartRef = ref<EchartsUIType>();
const { renderEcharts: renderCategory } = useEcharts(categoryChartRef);

const productColumns = [
  { title: '排名', dataIndex: 'rank', width: 60 },
  {
    title: '产品名称',
    dataIndex: 'productName',
    ellipsis: true,
  },
  { title: '销售金额', dataIndex: 'amount', align: 'right' as const, width: 120 },
  { title: '销售数量', dataIndex: 'count', align: 'right' as const, width: 90 },
  {
    title: '占比',
    dataIndex: 'share',
    align: 'right' as const,
    width: 80,
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
    const res: any = await getProductBreakdownApi({
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
  // 用产品列表做饼图：取销售额 Top 5，其余合并为"其他"
  const products: any[] = (data.value?.products || [])
    .slice()
    .sort((a: any, b: any) => Number(b.amount || 0) - Number(a.amount || 0));
  const top = products.slice(0, 5).map((p: any) => ({
    name: p.productName || p.product_name || '未知产品',
    value: Number(p.amount || 0),
  }));
  const restSum = products
    .slice(5)
    .reduce((s: number, p: any) => s + Number(p.amount || 0), 0);
  if (restSum > 0) {
    top.push({ name: '其他', value: restSum });
  }

  renderCategory({
    legend: { bottom: 0, orient: 'horizontal', type: 'scroll' },
    tooltip: {
      trigger: 'item',
      formatter: '{b}: ¥{c} ({d}%)',
    },
    series: [
      {
        type: 'pie',
        radius: ['40%', '70%'],
        data: top.length > 0 ? top : [{ name: '暂无', value: 1 }],
        label: { formatter: '{b}\n{d}%', show: true },
        itemStyle: { borderRadius: 6, borderColor: '#fff', borderWidth: 2 },
      },
    ],
    color: [
      '#1890ff',
      '#52c41a',
      '#faad14',
      '#722ed1',
      '#eb2f96',
      '#bfbfbf',
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
        <IconifyIcon icon="lucide:package" class="text-lg text-primary" />
        <span>产品维度拆解</span>
      </div>
    </template>

    <Spin :spinning="loading">
      <div v-if="!data?.products" class="py-8">
        <Empty description="暂无产品拆解数据" />
      </div>
      <div class="grid grid-cols-1 lg:grid-cols-5 gap-4">
        <!-- 品类占比（占 2/5） -->
        <div class="lg:col-span-2">
          <div class="text-sm font-medium mb-2 text-gray-700">产品 Top5 占比</div>
          <EchartsUI ref="categoryChartRef" height="200px" />
        </div>
        <!-- 产品排行（占 3/5） -->
        <div class="lg:col-span-3">
          <div class="text-sm font-medium mb-2 text-gray-700">产品销量排行</div>
          <Table
            :columns="productColumns"
            :data-source="data?.products || []"
            :pagination="{ pageSize: 5 }"
            row-key="rank"
            size="small"
            :scroll="{ x: 500, y: 240 }"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.dataIndex === 'amount'">
                {{ formatCurrency(record.amount) }}
              </template>
              <template v-else-if="column.dataIndex === 'share'">
                {{ Number(record.share || 0).toFixed(2) }}%
              </template>
            </template>
          </Table>
        </div>
      </div>
    </Spin>
  </Card>
</template>
