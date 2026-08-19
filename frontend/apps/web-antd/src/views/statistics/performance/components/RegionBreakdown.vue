<script lang="ts" setup>
import { computed, onMounted, ref, watch } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Card, Empty, Spin, Table, Tag } from 'ant-design-vue';

import { getRegionBreakdownApi } from '#/api';

interface Props {
  year: number;
  month?: number;
  timeDimension: string;
}

const props = defineProps<Props>();

const loading = ref(false);
const data = ref<any[]>([]);

const columns = [
  { title: '排名', dataIndex: 'rank', width: 70 },
  { title: '省份', dataIndex: 'province', width: 120 },
  {
    title: '业绩金额',
    dataIndex: 'amount',
    align: 'right' as const,
    width: 140,
  },
  {
    title: '客户数',
    dataIndex: 'customerCount',
    align: 'right' as const,
    width: 100,
  },
  {
    title: '占比',
    dataIndex: 'share',
    align: 'right' as const,
    width: 100,
  },
  {
    title: '热度',
    dataIndex: 'heat',
    width: 160,
  },
];

function formatCurrency(val?: number) {
  if (!val) return '¥0';
  if (val >= 10_000) return `¥${(val / 10_000).toFixed(1)}万`;
  return `¥${val.toLocaleString()}`;
}

const maxAmount = computed(() =>
  Math.max(1, ...data.value.map((d) => d.amount || 0)),
);

function heatColor(amount: number) {
  const ratio = amount / maxAmount.value;
  if (ratio > 0.75) return '#ff4d4f';
  if (ratio > 0.5) return '#fa8c16';
  if (ratio > 0.25) return '#faad14';
  if (ratio > 0.1) return '#52c41a';
  return '#1890ff';
}

function heatWidth(amount: number) {
  return `${(amount / maxAmount.value) * 100}%`;
}

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getRegionBreakdownApi({
      year: props.year,
      month: props.month,
      time_dimension: props.timeDimension,
    });
    data.value = res?.data || res || [];
  } catch {
    data.value = [];
  } finally {
    loading.value = false;
  }
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
        <IconifyIcon icon="lucide:map-pin" class="text-lg text-primary" />
        <span>区域维度拆解</span>
      </div>
    </template>
    <template #extra>
      <span class="text-xs text-gray-400">省份业绩分布与热度</span>
    </template>

    <Spin :spinning="loading">
      <div v-if="data.length === 0" class="py-8">
        <Empty description="暂无区域数据" />
      </div>
      <Table
        v-else
        :columns="columns"
        :data-source="data"
        :pagination="{ pageSize: 10 }"
        row-key="rank"
        size="small"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'amount'">
            {{ formatCurrency(record.amount) }}
          </template>
          <template v-else-if="column.dataIndex === 'share'">
            <Tag color="blue">{{ Number(record.share || 0).toFixed(2) }}%</Tag>
          </template>
          <template v-else-if="column.dataIndex === 'heat'">
            <div class="heat-bar-track">
              <div
                class="heat-bar-fill"
                :style="{
                  width: heatWidth(record.amount),
                  background: heatColor(record.amount),
                }"
              ></div>
            </div>
          </template>
        </template>
      </Table>
    </Spin>
  </Card>
</template>

<style scoped>
.heat-bar-track {
  width: 100%;
  height: 8px;
  overflow: hidden;
  background: #f0f0f0;
  border-radius: 4px;
}

.heat-bar-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.5s ease;
}
</style>
