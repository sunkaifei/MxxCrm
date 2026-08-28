<script lang="ts" setup>
// 客户分析驾驶舱：结论先行（KPI）→ 过程诊断（漏斗 + 分层转化）→ 结构洞察（类型/来源/行业）→ 精确明细
import type { EchartsUIType } from '@vben/plugins/echarts';

import { computed, onMounted, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import {
  LucideBanknote,
  LucideTarget,
  LucideUserCheck,
  LucideUsers,
} from '@vben/icons';
import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import { Card, Col, Progress, Row, Table, Tabs, Tag } from 'ant-design-vue';

import {
  getCustomerFunnelApi,
  getCustomerIndustryStatsApi,
  getCustomerSourceStatsApi,
  getCustomerTypeStatsApi,
} from '#/api/core/statistics';
import { $t } from '#/locales';

import TimeFilter from '../components/time-filter.vue';

const TabPane = Tabs.TabPane;

// ============ 数据状态 ============
interface DimStatRow {
  conversionRate: number;
  contractCount: number;
  totalCount: number;
}
const customerTypeData = ref<(DimStatRow & { name: string })[]>([]);
const customerSourceData = ref<(DimStatRow & { name: string })[]>([]);
const customerIndustryData = ref<
  (DimStatRow & { contractAmount: number; name: string })[]
>([]);
const funnelTotals = ref({
  contracts: 0,
  customers: 0,
  leads: 0,
  opps: 0,
});
const timeParams = ref<{
  end_date?: string;
  start_date?: string;
  year?: number;
}>({});

const loadData = async () => {
  try {
    const [typeRes, sourceRes, industryRes, funnelRes] = await Promise.all([
      getCustomerTypeStatsApi(timeParams.value),
      getCustomerSourceStatsApi(timeParams.value),
      getCustomerIndustryStatsApi(timeParams.value),
      getCustomerFunnelApi(timeParams.value),
    ]);

    // requestClient.get 返回 { code, data, msg }，data 字段即为后端实际数据
    const typeList: any[] = Array.isArray(typeRes)
      ? typeRes
      : ((typeRes as any)?.data ?? []);
    customerTypeData.value = typeList.map((item) => ({
      contractCount: Number(item.contract_count) || 0,
      conversionRate: Number(item.conversion_rate) || 0,
      name: String(item.customer_type ?? '-'),
      totalCount: Number(item.total_count) || 0,
    }));

    const sourceList: any[] = Array.isArray(sourceRes)
      ? sourceRes
      : ((sourceRes as any)?.data ?? []);
    customerSourceData.value = sourceList.map((item) => ({
      contractCount: Number(item.contract_count) || 0,
      conversionRate: Number(item.conversion_rate) || 0,
      name: String(item.source ?? '-'),
      totalCount: Number(item.total_count) || 0,
    }));

    const industryList: any[] = Array.isArray(industryRes)
      ? industryRes
      : ((industryRes as any)?.data ?? []);
    customerIndustryData.value = industryList
      .map((item) => ({
        contractAmount: Number(item.contract_amount) || 0,
        contractCount: Number(item.contract_count) || 0,
        conversionRate: Number(item.conversion_rate) || 0,
        name: String(item.industry ?? '-'),
        totalCount: Number(item.total_count) || 0,
      }))
      // 行业按签约额降序，价值最大的行业始终在最上方
      .sort((a, b) => b.contractAmount - a.contractAmount);

    const fd = (funnelRes as any)?.data ?? (funnelRes as any) ?? {};
    funnelTotals.value = {
      contracts: Number(fd.total_contracts) || 0,
      customers: Number(fd.total_customers) || 0,
      leads: Number(fd.total_leads) || 0,
      opps: Number(fd.total_opportunities) || 0,
    };
  } catch (error) {
    console.error($t('page.statistics.loadCustomerFailed'), error);
  }
};

function handleTimeChange(params: {
  end_date?: string;
  start_date?: string;
  year?: number;
}) {
  timeParams.value = params;
  loadData();
}

onMounted(loadData);

// ============ KPI 汇总（由维度明细聚合，口径一致） ============
function sumBy(rows: { [k: string]: any }[], key: string) {
  return rows.reduce((acc, r) => acc + (Number(r[key]) || 0), 0);
}
const totalCustomers = computed(() =>
  sumBy(customerTypeData.value, 'totalCount'),
);
const wonCustomers = computed(() =>
  sumBy(customerTypeData.value, 'contractCount'),
);
const overallRate = computed(() =>
  totalCustomers.value > 0
    ? (wonCustomers.value / totalCustomers.value) * 100
    : 0,
);
const signedAmountWan = computed(
  () => sumBy(customerIndustryData.value, 'contractAmount') / 10_000,
);

/** 金额展示：1 亿以内显示万元，超过则换算亿元 */
function formatAmount(wan: number) {
  return wan >= 10_000
    ? `${(wan / 10_000).toFixed(2)}${$t('page.statistics.yiUnit') ?? '亿'}`
    : `${wan.toLocaleString(undefined, { maximumFractionDigits: 1 })}${$t('page.statistics.currencyFormat')}`;
}

const kpiCards = computed(() => [
  {
    accent: '#2563eb',
    bg: 'rgba(37, 99, 235, .08)',
    icon: LucideUsers,
    title: $t('page.statistics.kpiTotalCustomers'),
    value: totalCustomers.value.toLocaleString(),
  },
  {
    accent: '#10b981',
    bg: 'rgba(16, 185, 129, .08)',
    icon: LucideUserCheck,
    title: $t('page.statistics.kpiContractCustomers'),
    value: wonCustomers.value.toLocaleString(),
  },
  {
    accent: '#8b5cf6',
    bg: 'rgba(139, 92, 246, .08)',
    icon: LucideTarget,
    suffix: '%',
    title: $t('page.statistics.kpiOverallRate'),
    value: overallRate.value.toFixed(2),
  },
  {
    accent: '#f59e0b',
    bg: 'rgba(245, 158, 11, .08)',
    icon: LucideBanknote,
    prefix: '¥',
    title: $t('page.statistics.kpiSignedAmount'),
    value: formatAmount(signedAmountWan.value),
  },
]);

// ============ 分层转化率（堵点诊断） ============
const stageRates = computed(() => [
  {
    from: funnelTotals.value.leads,
    label: $t('page.statistics.leadsToCustomer'),
    to: funnelTotals.value.customers,
  },
  {
    from: funnelTotals.value.customers,
    label: $t('page.statistics.customerToOpp'),
    to: funnelTotals.value.opps,
  },
  {
    from: funnelTotals.value.opps,
    label: $t('page.statistics.oppToContract'),
    to: funnelTotals.value.contracts,
  },
].map((s) => ({
  ...s,
  rate: s.from > 0 ? Math.min((s.to / s.from) * 100, 100) : 0,
})));

/** 找到转化率最低的环节作为瓶颈高亮 */
const bottleneckIdx = computed(() => {
  let idx = -1;
  let min = Number.POSITIVE_INFINITY;
  stageRates.value.forEach((s, i) => {
    if (s.rate < min) {
      min = s.rate;
      idx = i;
    }
  });
  return stageRates.value.length > 0 && min < 100 ? idx : -1;
});

const fullChainRate = computed(() =>
  funnelTotals.value.leads > 0
    ? (funnelTotals.value.contracts / funnelTotals.value.leads) * 100
    : 0,
);

// ============ 图表：公共样式与工具 ============
// 分类色板：CRM 数据蓝体系 + 中性扩展，避免紫色渐变类俗套配色
const PALETTE = [
  '#2563eb', '#0ea5e9', '#10b981', '#f59e0b',
  '#8b5cf6', '#ef4444', '#14b8a6', '#f97316',
];
const AXIS_LABEL_STYLE = { color: 'rgba(148, 163, 184, 1)', fontSize: 12 };
const SPLIT_LINE_STYLE = {
  lineStyle: { color: 'rgba(100, 116, 139, 0.18)' },
};
const EMPTY_OPTION = {
  title: {
    left: 'center' as const,
    show: true,
    subtext: '',
    text: $t('page.statistics.noData'),
    top: 'middle',
  },
};

const funnelRef = ref<EchartsUIType>();
const { renderEcharts: renderFunnel } = useEcharts(funnelRef);
const typeChartRef = ref<EchartsUIType>();
const { renderEcharts: renderTypeChart } = useEcharts(typeChartRef);
const sourceChartRef = ref<EchartsUIType>();
const { renderEcharts: renderSourceChart } = useEcharts(sourceChartRef);
const industryChartRef = ref<EchartsUIType>();
const { renderEcharts: renderIndustryChart } = useEcharts(industryChartRef);

const funnelStages = computed(() => [
  { count: funnelTotals.value.leads, name: $t('page.statistics.funnelStageLeads') },
  { count: funnelTotals.value.customers, name: $t('page.statistics.funnelStageCustomers') },
  { count: funnelTotals.value.opps, name: $t('page.statistics.funnelStageOpps') },
  { count: funnelTotals.value.contracts, name: $t('page.statistics.funnelStageContracts') },
]);

watch(funnelStages, (stages) => {
  const isEmpty = stages.every((s) => s.count === 0);
  renderFunnel(
    isEmpty
      ? EMPTY_OPTION
      : {
          series: [
            {
              bottom: 12,
              data: stages.map((s, i) => ({
                itemStyle: {
                  color: ['#1e40af', '#2563eb', '#60a5fa', '#93c5fd'][i],
                },
                label:
                  i >= 2
                    ? { color: '#1e3a8a', fontWeight: 500 }
                    : { color: '#fff', fontWeight: 500 },
                name: s.name,
                value: s.count,
              })),
              gap: 4,
              itemStyle: { borderColor: 'transparent' },
              label: {
                fontSize: 13,
                formatter: (p: any) => `{n|${p.name}}\n{v|${p.value}}`,
                position: 'inside',
                rich: {
                  n: { color: 'inherit', fontSize: 13, lineHeight: 20 },
                  v: { color: 'inherit', fontSize: 17, fontWeight: 700 },
                },
                show: true,
              },
              left: '12%',
              minSize: '18%',
              right: '12%',
              sort: 'descending',
              top: 12,
              type: 'funnel',
            },
          ],
          tooltip: {
            formatter: (p: any) => {
              const cur = stages[p.dataIndex]?.count ?? 0;
              const prevStage =
                p.dataIndex > 0 ? stages[p.dataIndex - 1] : undefined;
              const rate =
                prevStage && prevStage.count > 0
                  ? ` · ${((cur / prevStage.count) * 100).toFixed(1)}%`
                  : '';
              const first = stages[0];
              const percentAll =
                first && first.count > 0
                  ? ` (${((cur / first.count) * 100).toFixed(1)}%)`
                  : '';
              return `${p.name}<br/><b>${cur}</b>${percentAll}${rate}`;
            },
            trigger: 'item',
          },
        },
  );
});

watch(customerTypeData, (rows) => {
  if (rows.length === 0) {
    renderTypeChart(EMPTY_OPTION);
    return;
  }
  const pieData = rows.map((r) => ({ name: r.name, value: r.totalCount }));
  renderTypeChart({
    legend: { bottom: 0, icon: 'circle', type: 'scroll' },
    series: [
      {
        avoidLabelOverlap: true,
        center: ['50%', '42%'],
        data: pieData,
        emphasis: {
          itemStyle: {
            shadowBlur: 12,
            shadowColor: 'rgba(0, 0, 0, 0.35)',
          },
        },
        itemStyle: { borderColor: '#fff', borderRadius: 8, borderWidth: 2 },
        label: { show: false },
        radius: ['52%', '74%'],
        type: 'pie',
      },
    ],
    title: {
      left: 'center',
      subtext: $t('page.statistics.customerCount'),
      text: totalCustomers.value.toLocaleString(),
      textStyle: { fontSize: 26, fontWeight: 700 },
      top: '32%',
    },
    tooltip: {
      trigger: 'item',
      formatter: (p: any) =>
        `${p.name}<br/>${$t('page.statistics.customerCount')}: ${p.value} (${p.percent}%)<br/>${$t('page.statistics.contractCount')}: ${rows[p.dataIndex]?.contractCount ?? 0}`,
    },
  });
});

watch([customerSourceData, customerTypeData], ([sources]) => {
  if (sources.length === 0) {
    renderSourceChart(EMPTY_OPTION);
    return;
  }
  renderSourceChart({
    grid: {
      bottom: 30,
      containLabel: true,
      left: 8,
      right: 16,
      top: 36,
    },
    legend: { right: 8, top: 0, icon: 'roundRect', itemWidth: 12, itemHeight: 12 },
    series: [
      {
        barGap: '20%',
        data: sources.map((r) => r.totalCount),
        itemStyle: { borderRadius: [4, 4, 0, 0], color: PALETTE[0]! },
        name: $t('page.statistics.customerCount'),
        type: 'bar',
      },
      {
        data: sources.map((r) => r.contractCount),
        itemStyle: { borderRadius: [4, 4, 0, 0], color: PALETTE[2]! },
        name: $t('page.statistics.contractCount'),
        type: 'bar',
      },
    ],
    xAxis: {
      axisLabel: AXIS_LABEL_STYLE,
      axisLine: { lineStyle: SPLIT_LINE_STYLE.lineStyle },
      axisTick: { show: false },
      data: sources.map((r) => r.name),
      type: 'category',
    },
    yAxis: {
      axisLabel: AXIS_LABEL_STYLE,
      splitLine: SPLIT_LINE_STYLE,
      type: 'value',
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      formatter: (params: any) => {
        const row = sources[(params?.[0]?.dataIndex as number) ?? 0];
        const lines = params
          .map((p: any) => `${p.marker}${p.seriesName}: ${p.value}`)
          .join('<br/>');
        return `${row?.name ?? ''}<br/>${lines}<br/>${$t('page.statistics.conversionRate')}: <b>${row?.conversionRate.toFixed(2) ?? 0}%</b>`;
      },
    },
  });
});

watch(customerIndustryData, (rows) => {
  if (rows.length === 0) {
    renderIndustryChart(EMPTY_OPTION);
    return;
  }
  renderIndustryChart({
    grid: { bottom: 8, containLabel: true, left: 8, right: 56, top: 8 },
    series: [
      {
        barMaxWidth: 16,
        data: rows.map((r, i) => ({
          itemStyle: {
            borderRadius: [0, 4, 4, 0],
            color: PALETTE[i % PALETTE.length]!,
          },
          value: Number((r.contractAmount / 10_000).toFixed(1)),
        })),
        name: $t('page.statistics.contractAmount'),
        type: 'bar',
      },
    ],
    xAxis: {
      axisLabel: { ...AXIS_LABEL_STYLE, formatter: '{value}' },
      splitLine: SPLIT_LINE_STYLE,
      type: 'value',
    },
    yAxis: {
      axisLabel: AXIS_LABEL_STYLE,
      axisTick: { show: false },
      axisLine: { show: false },
      data: rows.map((r) => r.name),
      inverse: true,
      type: 'category',
    },
    tooltip: {
      trigger: 'item',
      formatter: (p: any) => {
        const row = rows[p.dataIndex];
        return `${row?.name ?? ''}<br/>${p.marker}${$t('page.statistics.contractAmount')}: ¥${p.value}${$t('page.statistics.currencyFormat')}<br/>${$t('page.statistics.customerCount')}: ${row?.totalCount ?? 0} · ${$t('page.statistics.contractCount')}: ${row?.contractCount ?? 0}`;
      },
    },
  });
});

// ============ 明细表格列定义 ============
function formatCurrency(val: number) {
  return `¥${(val / 10_000).toFixed(1)}${$t('page.statistics.currencyFormat')}`;
}

const baseColumns = (
  firstTitle: string,
  dataIndex = 'name',
): any[] => [
  { title: firstTitle, dataIndex },
  {
    align: 'right' as const,
    title: $t('page.statistics.totalCount'),
    dataIndex: 'totalCount',
  },
  {
    align: 'right' as const,
    title: $t('page.statistics.contractCount'),
    dataIndex: 'contractCount',
  },
  {
    align: 'right' as const,
    customRender: ({ text }: any) => `${Number(text).toFixed(2)}%`,
    title: $t('page.statistics.conversionRate'),
    dataIndex: 'conversionRate',
  },
];

const detailColumns = {
  industry: [
    ...baseColumns($t('page.statistics.industryCol')),
    {
      align: 'right' as const,
      customRender: ({ text }: any) => formatCurrency(text),
      title: $t('page.statistics.contractAmount'),
      dataIndex: 'contractAmount',
    },
  ],
  source: baseColumns($t('page.statistics.source')),
  type: baseColumns($t('page.statistics.customerTypeCol')),
};
</script>

<template>
  <Page>
    <div class="space-y-4 p-4 pb-8">
      <!-- 页头 -->
      <div class="fade-up">
        <h2 class="mb-1 text-xl font-bold">
          {{ $t('page.statistics.customerAnalysis') }}
        </h2>
        <p class="text-sm text-gray-400">
          {{ $t('page.statistics.funnelSubtitle') }}
        </p>
      </div>

      <!-- 时间筛选 -->
      <TimeFilter class="fade-up" style="animation-delay: 40ms" @change="handleTimeChange" />

      <!-- KPI 指标带 -->
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <div
          v-for="(kpi, i) in kpiCards"
          :key="kpi.title"
          class="fade-up kpi-card rounded-xl border bg-card p-5"
          :style="{ animationDelay: `${80 + i * 60}ms` }"
        >
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-400">{{ kpi.title }}</span>
            <span
              class="flex h-9 w-9 items-center justify-center rounded-lg"
              :style="{ backgroundColor: kpi.bg, color: kpi.accent }"
            >
              <component :is="kpi.icon" class="h-4.5 w-4.5" />
            </span>
          </div>
          <div class="num-font mt-2 text-[28px] font-bold leading-none">
            {{ kpi.prefix }}{{ kpi.value }}{{ kpi.suffix }}
          </div>
        </div>
      </div>

      <!-- 过程诊断：漏斗 + 分层转化 -->
      <Row :gutter="16">
        <Col :lg="15" :sm="24" class="fade-up" style="animation-delay: 320ms">
          <Card :bordered="false" class="chart-card">
            <template #title>
              <div class="card-head">
                <span>{{ $t('page.statistics.funnelTitle') }}</span>
                <Tag color="blue">{{ $t('page.statistics.funnelSubtitle') }}</Tag>
              </div>
            </template>
            <div class="h-[300px] w-full">
              <EchartsUI ref="funnelRef" />
            </div>
          </Card>
        </Col>
        <Col :lg="9" :sm="24" class="fade-up" style="animation-delay: 380ms">
          <Card :bordered="false" class="chart-card">
            <template #title>
              <div class="flex flex-col">
                <span>{{ $t('page.statistics.stageConversion') }}</span>
                <span class="text-xs font-normal text-gray-400">
                  {{ $t('page.statistics.stageConversionSub') }}
                </span>
              </div>
            </template>
            <div class="flex flex-col justify-center gap-4 px-1 py-2">
              <div v-for="(stage, i) in stageRates" :key="stage.label">
                <div class="mb-1 flex items-center justify-between text-sm">
                  <span>{{ stage.label }}</span>
                  <span class="flex items-center gap-2">
                    <span class="num-font text-base font-semibold">{{
                      `${stage.rate.toFixed(1)}%`
                    }}</span>
                    <Tag v-if="i === bottleneckIdx" color="warning" class="mr-0">
                      流失重点
                    </Tag>
                  </span>
                </div>
                <Progress
                  :percent="Number(stage.rate.toFixed(1))"
                  :show-info="false"
                  :stroke-color="
                    i === bottleneckIdx ? '#f59e0b' : '#2563eb'
                  "
                />
              </div>
              <div
                class="mt-1 flex items-center justify-between rounded-lg bg-blue-50 px-3 py-2 dark:bg-blue-950"
              >
                <span class="text-sm text-gray-500">{{
                  $t('page.statistics.fullChain')
                }}</span>
                <span class="num-font text-lg font-bold text-blue-600 dark:text-blue-400">
                  {{ `${fullChainRate.toFixed(1)}%` }}
                </span>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <!-- 结构洞察：类型构成 / 来源效益 / 行业价值 -->
      <Row :gutter="16">
        <Col :lg="8" :md="24" class="fade-up" style="animation-delay: 420ms">
          <Card :bordered="false" class="chart-card">
            <template #title>{{ $t('page.statistics.chartTypeDist') }}</template>
            <div class="h-[280px] w-full">
              <EchartsUI ref="typeChartRef" />
            </div>
          </Card>
        </Col>
        <Col :lg="8" :md="24" class="fade-up" style="animation-delay: 460ms">
          <Card :bordered="false" class="chart-card">
            <template #title>{{
              $t('page.statistics.chartSourceEffect')
            }}</template>
            <div class="h-[280px] w-full">
              <EchartsUI ref="sourceChartRef" />
            </div>
          </Card>
        </Col>
        <Col :lg="8" :md="24" class="fade-up" style="animation-delay: 500ms">
          <Card :bordered="false" class="chart-card">
            <template #title>{{
              $t('page.statistics.chartIndustryValue')
            }}</template>
            <div class="h-[280px] w-full">
              <EchartsUI ref="industryChartRef" />
            </div>
          </Card>
        </Col>
      </Row>

      <!-- 明细数据 -->
      <Card
        :bordered="false"
        class="fade-up chart-card"
        style="animation-delay: 540ms"
      >
        <template #title>{{ $t('page.statistics.detailData') }}</template>
        <Tabs size="small">
          <TabPane :tab="$t('page.statistics.typeTab')" key="type">
            <Table
              :columns="detailColumns.type"
              :data-source="customerTypeData"
              :pagination="false"
              row-key="name"
              size="small"
            />
          </TabPane>
          <TabPane :tab="$t('page.statistics.sourceTab')" key="source">
            <Table
              :columns="detailColumns.source"
              :data-source="customerSourceData"
              :pagination="false"
              row-key="name"
              size="small"
            />
          </TabPane>
          <TabPane :tab="$t('page.statistics.industryTab')" key="industry">
            <Table
              :columns="detailColumns.industry"
              :data-source="customerIndustryData"
              :pagination="false"
              row-key="name"
              size="small"
            />
          </TabPane>
        </Tabs>
      </Card>
    </div>
  </Page>
</template>

<style scoped>
/* 数字字体栈：Windows 用 Bahnschrift，macOS 用 DIN Alternate，皆系统自带 */
.num-font {
  font-family:
    'DIN Alternate',
    Bahnschrift,
    'Segoe UI Variable Display',
    sans-serif;
  font-variant-numeric: tabular-nums;
}

.kpi-card {
  transition:
    transform 0.25s ease,
    box-shadow 0.25s ease;
}
.kpi-card:hover {
  box-shadow: 0 10px 28px -14px rgb(37 99 235 / 35%);
  transform: translateY(-3px);
}

.chart-card {
  border-radius: 12px;
}

.card-head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.fade-up {
  animation: fade-up 0.5s ease forwards;
  opacity: 0;
}
@keyframes fade-up {
  from {
    transform: translateY(12px);
    opacity: 0;
  }
  to {
    opacity: 1;
    transform: none;
  }
}
</style>
