<script lang="ts" setup>
import { computed } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Card, Empty, Spin, Statistic, Tag } from 'ant-design-vue';

interface Props {
  loading?: boolean;
  data?: {
    completedAmount?: number;
    pipelineAmount?: number;
    historicalWinRate?: number;
    forecastAmount?: number;
    targetAmount?: number;
    gapAmount?: number;
    status?: string;
    pipelineCoverage?: number;
  };
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  data: () => ({}),
});

function formatCurrency(val?: number) {
  if (!val) return '¥0';
  if (val >= 100000000) return `¥${(val / 100000000).toFixed(2)}亿`;
  if (val >= 10000) return `¥${(val / 10000).toFixed(1)}万`;
  return `¥${val.toLocaleString()}`;
}

const statusConfig = computed(() => {
  const map: Record<string, { color: string; bg: string; text: string; icon: string }> = {
    green: { color: '#52c41a', bg: '#f6ffed', text: '达标在望', icon: 'lucide:check-circle' },
    yellow: { color: '#faad14', bg: '#fffbe6', text: '关注缺口', icon: 'lucide:alert-circle' },
    red: { color: '#ff4d4f', bg: '#fff2f0', text: '缺口预警', icon: 'lucide:alert-triangle' },
    warning: { color: '#fa8c16', bg: '#fff7e6', text: '预警', icon: 'lucide:alert-triangle' },
  };
  return map[props.data?.status || 'yellow'] || map.yellow;
});

const coverageText = computed(() => {
  const cov = props.data?.pipelineCoverage || 0;
  if (cov < 1) return { text: '危险', color: '#ff4d4f' };
  if (cov < 3) return { text: '正常', color: '#faad14' };
  return { text: '健康', color: '#52c41a' };
});
</script>

<template>
  <Card :body-style="{ padding: '16px' }" class="forecast-card">
    <Spin :spinning="loading">
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center gap-2">
          <IconifyIcon icon="lucide:trending-up" class="text-lg text-primary" />
          <span class="font-semibold">业绩预测</span>
        </div>
        <Tag :color="statusConfig.color" style="margin: 0">
          <IconifyIcon :icon="statusConfig.icon" class="mr-1" />
          {{ statusConfig.text }}
        </Tag>
      </div>

      <div v-if="!data?.targetAmount" class="py-6">
        <Empty description="暂无预测数据" />
      </div>

      <div v-else>
        <!-- 主预测数据 -->
        <div
          class="main-metric mb-3"
          :style="{ background: statusConfig.bg, borderColor: statusConfig.color }"
        >
          <div class="text-xs text-gray-500 mb-1">预测达成金额</div>
          <div class="text-2xl font-bold" :style="{ color: statusConfig.color }">
            {{ formatCurrency(data.forecastAmount) }}
          </div>
          <div class="text-xs text-gray-500 mt-1">
            = 已完成 {{ formatCurrency(data.completedAmount) }}
            + 在途商机可能贡献
            {{ formatCurrency((data.pipelineAmount || 0) * (data.historicalWinRate || 0)) }}
          </div>
        </div>

        <!-- 数据栅格 -->
        <div class="grid grid-cols-4 gap-2">
          <div class="metric-block">
            <div class="text-xs text-gray-500">已完成</div>
            <div class="text-sm font-semibold text-green-600">
              {{ formatCurrency(data.completedAmount) }}
            </div>
          </div>
          <div class="metric-block">
            <div class="text-xs text-gray-500">在途商机</div>
            <div class="text-sm font-semibold text-blue-600">
              {{ formatCurrency(data.pipelineAmount) }}
            </div>
          </div>
          <div class="metric-block">
            <div class="text-xs text-gray-500">历史成交率</div>
            <div class="text-sm font-semibold text-purple-600">
              {{ ((data.historicalWinRate || 0) * 100).toFixed(2) }}%
            </div>
          </div>
          <div class="metric-block">
            <div class="text-xs text-gray-500">缺口</div>
            <div class="text-sm font-semibold" :style="{ color: statusConfig.color }">
              {{ formatCurrency(data.gapAmount) }}
            </div>
          </div>
        </div>

        <!-- Pipeline 健康度 -->
        <div class="mt-3 pt-3 border-t border-gray-100">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-1">
              <IconifyIcon icon="lucide:activity" class="text-sm text-gray-500" />
              <span class="text-xs text-gray-500">Pipeline 健康度</span>
            </div>
            <Tag :color="coverageText.color">
              {{ (data.pipelineCoverage || 0).toFixed(2) }} 倍 ·
              {{ coverageText.text }}
            </Tag>
          </div>
          <div class="text-xs text-gray-400 mt-1">
            在途商机金额 ÷ 目标缺口，&lt;1倍危险，1~3倍正常，&gt;3倍健康
          </div>
        </div>
      </div>
    </Spin>
  </Card>
</template>

<style scoped>
.forecast-card :deep(.ant-card-body) {
  padding: 16px;
}

.main-metric {
  padding: 12px 14px;
  border-radius: 8px;
  border-left: 4px solid;
}

.metric-block {
  padding: 8px 10px;
  background: #fafafa;
  border-radius: 6px;
  text-align: center;
}
</style>
