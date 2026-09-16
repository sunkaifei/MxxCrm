<script lang="ts" setup>
// 销售业绩卡（方案 6 节三期岗位卡：聚合接口 salesPerformance 分区，
// 当月新增客户/跟进次数/新增商机/成交金额（合同签订口径，Decimal 字符串需 Number()））
// 卡片形态参数化（调研文档 8.1/K11）：displayForm=value(数值四宫格，默认)|bar|line|pie
import type { EchartsUIType } from '@vben/plugins/echarts';

import { computed, nextTick, onMounted, ref, watch } from 'vue';

import { Card, Empty, Spin } from 'ant-design-vue';
import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { getWorkspaceSummaryShared } from '#/api/core/system/workspace-summary';
import type { SalesPerformanceSummary } from '#/api/core/system/workspace-summary';

import { useDashboardPermission, useWorkspaceCards } from '../config';

defineOptions({ name: 'WorkspaceSalesPerformanceCard' });

const props = defineProps<{
  /** 显示形态：未配置 = 数值四宫格（既有形态） */
  displayForm?: 'bar' | 'line' | 'pie' | 'value';
}>();

const { canShow } = useDashboardPermission();
const { cardConfigOf, workspaceTimeRange } = useWorkspaceCards();

const loading = ref(false);
const summary = ref<SalesPerformanceSummary>({
  dealAmount: 0,
  followUps: 0,
  newCustomers: 0,
  newOpportunities: 0,
});

// 生效形态：props 覆盖模型配置（配置面板保存写入 card_config.displayForm）
const effectiveForm = computed(
  () => props.displayForm || cardConfigOf('workspace_sales_performance')?.displayForm || 'value',
);

const chartRef = ref<EchartsUIType>();
const { renderEcharts } = useEcharts(chartRef);

const METRICS = computed(() => [
  { name: '新增客户', value: toNum(summary.value.newCustomers) },
  { name: '跟进次数', value: toNum(summary.value.followUps) },
  { name: '新增商机', value: toNum(summary.value.newOpportunities) },
  { name: '成交金额', value: toNum(summary.value.dealAmount) },
]);

const CHART_COLORS = ['#2185eb', '#13c2c2', '#722ed1', '#ff9326'];

function renderChart() {
  if (!['bar', 'line', 'pie'].includes(effectiveForm.value)) return;
  if (!chartRef.value) return;
  const data = METRICS.value;
  if (effectiveForm.value === 'pie') {
    renderEcharts({
      color: CHART_COLORS,
      tooltip: { trigger: 'item', formatter: '{b}: {c} ({d}%)' },
      legend: { bottom: 0, icon: 'circle', itemWidth: 8, itemHeight: 8 },
      series: [
        {
          type: 'pie',
          radius: ['42%', '68%'],
          center: ['50%', '44%'],
          itemStyle: { borderRadius: 6, borderColor: '#fff', borderWidth: 2 },
          label: { show: false },
          data: data.map((d) => ({ name: d.name, value: d.value })),
        },
      ],
    });
  } else {
    renderEcharts({
      color: CHART_COLORS,
      tooltip: { trigger: 'axis' },
      grid: { left: 8, right: 8, top: 28, bottom: 0, containLabel: true },
      xAxis: { type: 'category', data: data.map((d) => d.name), axisTick: { show: false } },
      yAxis: { type: 'value', splitLine: { lineStyle: { type: 'dashed' } } },
      series: [
        effectiveForm.value === 'line'
          ? {
              type: 'line',
              smooth: true,
              symbolSize: 7,
              areaStyle: { opacity: 0.15 },
              data: data.map((d) => d.value),
              label: { show: true, position: 'top' },
            }
          : {
              type: 'bar',
              barWidth: '42%',
              itemStyle: { borderRadius: [4, 4, 0, 0] },
              data: data.map((d) => d.value),
              label: { show: true, position: 'top' },
            },
      ],
    });
  }
}

watch(
  [summary, effectiveForm],
  () => {
    if (['bar', 'line', 'pie'].includes(effectiveForm.value)) {
      nextTick(() => renderChart());
    }
  },
  { deep: true },
);

function toNum(v: null | number | string | undefined): number {
  const n = Number(v ?? 0);
  return Number.isFinite(n) ? n : 0;
}

// Decimal 字符串金额格式化（千分位 + 两位小数）
const dealAmountText = computed(() => {
  return toNum(summary.value.dealAmount).toLocaleString('en-US', {
    maximumFractionDigits: 2,
    minimumFractionDigits: 2,
  });
});

// 四项指标全为 0 视为本月暂无业绩，展示空态
const hasData = computed(() => {
  return (
    summary.value.newCustomers > 0 ||
    summary.value.followUps > 0 ||
    summary.value.newOpportunities > 0 ||
    toNum(summary.value.dealAmount) > 0
  );
});

async function loadSummary() {
  // 模块权限过滤：无权限不请求，直接空态兜底
  if (!canShow('salesPerformance')) {
    summary.value = {
      dealAmount: 0,
      followUps: 0,
      newCustomers: 0,
      newOpportunities: 0,
    };
    return;
  }
  loading.value = true;
  try {
    const res = await getWorkspaceSummaryShared(
      false,
      undefined,
      workspaceTimeRange.value,
    );
    summary.value = res?.salesPerformance || {
      dealAmount: 0,
      followUps: 0,
      newCustomers: 0,
      newOpportunities: 0,
    };
  } catch {
    summary.value = {
      dealAmount: 0,
      followUps: 0,
      newCustomers: 0,
      newOpportunities: 0,
    };
  } finally {
    loading.value = false;
  }
}

// 时间范围切换：重拉数据（合并缓存键含 timeRange，切范围必发新请求）
watch(workspaceTimeRange, () => {
  loadSummary();
});

onMounted(() => {
  loadSummary();
});

defineExpose({ reload: loadSummary });
</script>

<template>
  <Card
    class="flex h-full flex-col overflow-hidden"
    :body-style="{ flex: '1 1 0', minHeight: 0, overflowY: 'auto' }"
  >
    <template #title>
      <div class="flex items-center gap-2">
        <span
          class="inline-block size-2 rounded-full bg-indigo-500"
          aria-hidden="true"
        ></span>
        <span>
          {{ $t('page.dashboard.workspace.cards.salesPerformance.title') }}
        </span>
      </div>
    </template>
    <Spin :spinning="loading">
      <div id="__dbg_card" style="font-size:11px;color:#999;padding:2px 6px">
        form={{ effectiveForm }} hasData={{ hasData }} loading={{ loading }}
      </div>
      <!-- 图表形态（柱状/折线/饼图）：ECharts 渲染 -->
      <div
        v-if="['bar', 'line', 'pie'].includes(effectiveForm)"
        v-show="hasData"
        class="h-[220px] w-full"
      >
        <EchartsUI ref="chartRef" />
      </div>
      <div v-else-if="hasData" class="flex flex-col gap-2">
        <div class="grid grid-cols-2 gap-2">
          <div
            class="rounded-md bg-blue-50 px-3 py-2 dark:bg-blue-950"
          >
            <div class="text-xl font-semibold text-blue-600">
              {{ summary.newCustomers }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.dashboard.workspace.cards.salesPerformance.newCustomers') }}
            </div>
          </div>
          <div
            class="rounded-md bg-cyan-50 px-3 py-2 dark:bg-cyan-950"
          >
            <div class="text-xl font-semibold text-cyan-600">
              {{ summary.followUps }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.dashboard.workspace.cards.salesPerformance.followUps') }}
            </div>
          </div>
          <div
            class="rounded-md bg-purple-50 px-3 py-2 dark:bg-purple-950"
          >
            <div class="text-xl font-semibold text-purple-600">
              {{ summary.newOpportunities }}
            </div>
            <div class="text-xs text-gray-500">
              {{
                $t(
                  'page.dashboard.workspace.cards.salesPerformance.newOpportunities',
                )
              }}
            </div>
          </div>
          <div
            class="rounded-md bg-orange-50 px-3 py-2 dark:bg-orange-950"
          >
            <div class="text-base font-semibold text-orange-500">
              ¥ {{ dealAmountText }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.dashboard.workspace.cards.salesPerformance.dealAmount') }}
            </div>
          </div>
        </div>
      </div>
      <Empty
        v-else
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        :description="
          $t('page.dashboard.workspace.cards.salesPerformance.empty')
        "
        class="py-8"
      />
    </Spin>
  </Card>
</template>
