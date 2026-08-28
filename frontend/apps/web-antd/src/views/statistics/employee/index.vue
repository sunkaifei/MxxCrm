<script lang="ts" setup>
import type { EchartsUIType } from '@vben/plugins/echarts';

import { computed, h, onMounted, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import {
  Card,
  Col,
  Progress,
  Row,
  Spin,
  Switch,
  Table,
  Tag,
  TreeSelect,
} from 'ant-design-vue';

import {
  getEmployeeConversionApi,
  getEmployeeCustomerCountApi,
  getEmployeeFollowUpApi,
} from '#/api/core/statistics';
import { getDeptTreeApi } from '#/api/core/system/dept';
import { $t } from '#/locales';

import TimeFilter from '../components/time-filter.vue';

/**
 * 员工统计页（重新设计）
 * 结构：筛选栏 → 团队 KPI 总览 → 金额榜/商机转化图 → 跟进投入/效率洞察 → 员工全景榜
 * 三份接口数据按 employeeId 合并为一张全景表，管理层无需横向比对三张孤立表。
 */

// ---------- 数据加载 ----------
const loading = ref(false);
const customerRows = ref<any[]>([]);
const followRows = ref<any[]>([]);
const conversionRows = ref<any[]>([]);
const timeParams = ref<{
  end_date?: string;
  start_date?: string;
  year?: number;
}>({});
const departmentId = ref<number>();
// 口径开关：true=仅排行「当年已通过年度销售计划」的销售（有计划 ⇒ 销售身份，与业绩计划中心同源）
const onlySales = ref(true);

function extractList(res: any): any[] {
  return Array.isArray(res) ? res : (res?.data ?? []);
}

async function loadData() {
  loading.value = true;
  try {
    const params = {
      ...timeParams.value,
      department_id: departmentId.value,
      only_sales: onlySales.value,
    };
    const [customerRes, followRes, conversionRes] = await Promise.all([
      getEmployeeCustomerCountApi(params),
      getEmployeeFollowUpApi(params),
      getEmployeeConversionApi(params),
    ]);
    customerRows.value = extractList(customerRes);
    followRows.value = extractList(followRes);
    conversionRows.value = extractList(conversionRes);
  } catch (error) {
    console.error($t('page.statistics.loadEmployeeFailed'), error);
  } finally {
    loading.value = false;
  }
}

function handleTimeChange(params: {
  end_date?: string;
  start_date?: string;
  year?: number;
}) {
  timeParams.value = params;
  loadData();
}

function handleDeptSelect(value: any) {
  departmentId.value = value ? Number(value) : undefined;
  loadData();
}

function handleOnlySalesChange(checked: boolean | string | number) {
  onlySales.value = Boolean(checked);
  loadData();
}

onMounted(loadData);

// ---------- 部门筛选 ----------
const deptTreeData = ref<any[]>([]);

onMounted(async () => {
  try {
    const res = await getDeptTreeApi();
    const list = res?.data || [];
    deptTreeData.value = list;
  } catch {
    deptTreeData.value = [];
  }
});

// ---------- 三表合并（员工全景） ----------
interface MergedRow {
  avgContractAmount: number;
  avgFollowInterval: number;
  avgSalesCycleDays: number;
  contractAmount: number;
  contractCustomers: number;
  customerConversionRate: number;
  customerFollowUp: number;
  customersWithoutFollow30Days: number;
  departmentName?: string;
  employeeId?: number;
  employeeName?: string;
  lostOpportunities: number;
  newCustomers: number;
  openOpportunities: number;
  opportunityFollowUp: number;
  opportunityWinRate: number;
  totalContracts: number;
  totalCustomers: number;
  totalFollowUp: number;
  totalOpportunities: number;
  wonOpportunities: number;
}

const mergedRows = computed<MergedRow[]>(() => {
  const map = new Map<number | string, MergedRow>();
  const keyOf = (r: any) => r.employeeId ?? r.employeeName ?? '';

  for (const c of customerRows.value) {
    map.set(keyOf(c), {
      avgContractAmount: 0,
      avgFollowInterval: 0,
      avgSalesCycleDays: 0,
      contractAmount: 0,
      contractCustomers: Number(c.contractCustomers) || 0,
      customerConversionRate: Number(c.customerConversionRate) || 0,
      customerFollowUp: 0,
      customersWithoutFollow30Days: 0,
      departmentName: c.departmentName,
      employeeId: c.employeeId,
      employeeName: c.employeeName,
      lostOpportunities: 0,
      newCustomers: Number(c.newCustomersThisMonth) || 0,
      openOpportunities: 0,
      opportunityFollowUp: 0,
      opportunityWinRate: 0,
      totalContracts: 0,
      totalCustomers: Number(c.totalCustomers) || 0,
      totalFollowUp: 0,
      totalOpportunities: 0,
      wonOpportunities: 0,
    });
  }
  for (const f of followRows.value) {
    const k = keyOf(f);
    const row =
      map.get(k) ??
      ({
        avgContractAmount: 0,
        avgFollowInterval: 0,
        avgSalesCycleDays: 0,
        contractAmount: 0,
        contractCustomers: 0,
        customerConversionRate: 0,
        customerFollowUp: 0,
        customersWithoutFollow30Days: 0,
        departmentName: f.departmentName,
        employeeId: f.employeeId,
        employeeName: f.employeeName,
        lostOpportunities: 0,
        newCustomers: 0,
        openOpportunities: 0,
        opportunityFollowUp: 0,
        opportunityWinRate: 0,
        totalContracts: 0,
        totalCustomers: 0,
        totalFollowUp: 0,
        totalOpportunities: 0,
        wonOpportunities: 0,
      } as MergedRow);
    row.customerFollowUp = Number(f.customerFollowUp) || 0;
    row.opportunityFollowUp = Number(f.opportunityFollowUp) || 0;
    row.totalFollowUp = Number(f.totalFollowUp) || 0;
    row.avgFollowInterval = Number(f.avgFollowInterval) || 0;
    row.customersWithoutFollow30Days =
      Number(f.customersWithoutFollow30Days) || 0;
    map.set(k, row);
  }
  for (const v of conversionRows.value) {
    const k = keyOf(v);
    const row =
      map.get(k) ??
      ({
        avgContractAmount: 0,
        avgFollowInterval: 0,
        avgSalesCycleDays: 0,
        contractAmount: 0,
        contractCustomers: 0,
        customerConversionRate: 0,
        customerFollowUp: 0,
        customersWithoutFollow30Days: 0,
        departmentName: v.departmentName,
        employeeId: v.employeeId,
        employeeName: v.employeeName,
        lostOpportunities: 0,
        newCustomers: 0,
        openOpportunities: 0,
        opportunityFollowUp: 0,
        opportunityWinRate: 0,
        totalContracts: 0,
        totalCustomers: 0,
        totalFollowUp: 0,
        totalOpportunities: 0,
        wonOpportunities: 0,
      } as MergedRow);
    const total = Number(v.totalOpportunities) || 0;
    const won = Number(v.wonOpportunities) || 0;
    const lost = Number(v.lostOpportunities) || 0;
    row.totalOpportunities = total;
    row.wonOpportunities = won;
    row.lostOpportunities = lost;
    row.openOpportunities = Math.max(total - won - lost, 0);
    row.opportunityWinRate = Number(v.opportunityWinRate) || 0;
    row.totalContracts = Number(v.totalContracts) || 0;
    row.contractAmount = Number(v.contractAmount) || 0;
    row.avgContractAmount = Number(v.avgContractAmount) || 0;
    row.avgSalesCycleDays = Number(v.avgSalesCycleDays) || 0;
    map.set(k, row);
  }
  return [...map.values()].toSorted(
    (a, b) => b.contractAmount - a.contractAmount,
  );
});

// ---------- 团队 KPI ----------
const headcount = computed(() => mergedRows.value.length || 1);

const kpis = computed(() => {
  const sum = (fn: (r: MergedRow) => number) =>
    mergedRows.value.reduce((acc, r) => acc + fn(r), 0);
  const totalOpp = sum((r) => r.totalOpportunities);
  const totalWon = sum((r) => r.wonOpportunities);
  return {
    totalCustomers: sum((r) => r.totalCustomers),
    newCustomers: sum((r) => r.newCustomers),
    contractCustomers: sum((r) => r.contractCustomers),
    totalFollowUp: sum((r) => r.totalFollowUp),
    wonRate: totalOpp > 0 ? (totalWon / totalOpp) * 100 : 0,
    contractAmount: sum((r) => r.contractAmount),
  };
});

const kpiCards = computed(() => [
  {
    accent: '#0d9488',
    label: $t('page.statistics.totalCustomers'),
    display: String(kpis.value.totalCustomers),
    sub: `${$t('page.statistics.employeeView.avgPer')} ${(kpis.value.totalCustomers / headcount.value).toFixed(1)}`,
  },
  {
    accent: '#0284c7',
    label: $t('page.statistics.newCustomers'),
    display: String(kpis.value.newCustomers),
    sub: `${$t('page.statistics.employeeView.avgPer')} ${(kpis.value.newCustomers / headcount.value).toFixed(1)}`,
  },
  {
    accent: '#7c3aed',
    label: $t('page.statistics.contractCustomers'),
    display: String(kpis.value.contractCustomers),
    sub: `${$t('page.statistics.employeeView.avgPer')} ${(kpis.value.contractCustomers / headcount.value).toFixed(1)}`,
  },
  {
    accent: '#d97706',
    label: $t('page.statistics.employeeView.followTotal'),
    display: String(kpis.value.totalFollowUp),
    sub: `${$t('page.statistics.employeeView.avgPer')} ${(kpis.value.totalFollowUp / headcount.value).toFixed(0)}`,
  },
  {
    accent: '#059669',
    label: $t('page.statistics.employeeView.wonRate'),
    display: `${kpis.value.wonRate.toFixed(1)}%`,
    sub: `${mergedRows.value.length} ${$t('page.statistics.employeeView.headUnit')}`,
  },
  {
    accent: '#dc2626',
    label: $t('page.statistics.contractAmount'),
    display: formatCurrency(kpis.value.contractAmount),
    sub: `${$t('page.statistics.employeeView.avgPer')} ${formatCurrency(kpis.value.contractAmount / headcount.value)}`,
  },
]);

// ---------- 效率洞察 ----------
const efficiency = computed(() => {
  const rows = mergedRows.value;
  if (rows.length === 0)
    return [
      { key: 'interval', unit: '', value: '-' },
      { key: 'cycle', unit: '', value: '-' },
      { key: 'deal', unit: '', value: '-' },
      { key: 'noFollow', unit: '', value: '-' },
    ];
  const avg = (fn: (r: MergedRow) => number) =>
    rows.reduce((acc, r) => acc + fn(r), 0) / rows.length;
  const noFollow = rows.reduce(
    (acc, r) => acc + r.customersWithoutFollow30Days,
    0,
  );
  return [
    {
      key: 'interval',
      unit: $t('page.statistics.employeeView.dayUnit'),
      value: avg((r) => r.avgFollowInterval).toFixed(1),
    },
    {
      key: 'cycle',
      unit: $t('page.statistics.employeeView.dayUnit'),
      value: avg((r) => r.avgSalesCycleDays).toFixed(1),
    },
    {
      key: 'deal',
      unit: '',
      value: formatCurrency(avg((r) => r.avgContractAmount)),
    },
    { key: 'noFollow', unit: '', value: String(noFollow) },
  ];
});

// ---------- 图表 ----------
const topChartRef = ref<EchartsUIType>();
const oppChartRef = ref<EchartsUIType>();
const followChartRef = ref<EchartsUIType>();
const { renderEcharts: renderTop } = useEcharts(topChartRef);
const { renderEcharts: renderOpp } = useEcharts(oppChartRef);
const { renderEcharts: renderFollow } = useEcharts(followChartRef);

const emptyHint = $t('page.statistics.noData');

watch(
  mergedRows,
  (rows) => {
    // 图表 1：合同金额榜 TOP 8（横向条形，冠军在最上）
    const top = rows
      .toSorted((a, b) => b.contractAmount - a.contractAmount)
      .slice(0, 8)
      .toReversed();
    renderTop({
      grid: { bottom: 8, containLabel: true, left: 8, right: 28, top: 8 },
      series: [
        {
          data: top.map((r) => r.contractAmount),
          itemStyle: {
            borderRadius: [0, 6, 6, 0],
            color: {
              type: 'linear',
              x: 0,
              x2: 1,
              y: 0,
              y2: 0,
              colorStops: [
                { offset: 0, color: '#99f6e4' },
                { offset: 1, color: '#0d9488' },
              ],
            },
          },
          label: {
            color: '#0f766e',
            formatter: (p: any) => formatCurrency(p.value),
            position: 'right',
            show: true,
          },
          type: 'bar',
          barMaxWidth: 18,
        },
      ],
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'shadow' },
        valueFormatter: (v: any) => formatCurrency(Number(v)),
      },
      xAxis: { type: 'value', splitLine: { lineStyle: { type: 'dashed' } } },
      yAxis: {
        data: top.map((r) => r.employeeName ?? '-'),
        inverse: false,
        type: 'category',
      },
    });

    // 图表 2：商机转化（赢单/输单/进行中 堆叠）
    renderOpp({
      grid: { bottom: 8, containLabel: true, left: 8, right: 8, top: 32 },
      legend: { top: 0 },
      series: [
        {
          barMaxWidth: 22,
          data: rows.map((r) => r.wonOpportunities),
          itemStyle: { borderRadius: 0 },
          name: $t('page.statistics.wonOpportunities'),
          stack: 'opp',
          type: 'bar',
          color: '#10b981',
        },
        {
          barMaxWidth: 22,
          data: rows.map((r) => r.lostOpportunities),
          name: $t('page.statistics.lostOpportunities'),
          stack: 'opp',
          type: 'bar',
          color: '#f87171',
        },
        {
          barMaxWidth: 22,
          data: rows.map((r) => r.openOpportunities),
          name: $t('page.statistics.employeeView.open'),
          stack: 'opp',
          type: 'bar',
          color: '#cbd5e1',
        },
      ],
      tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
      xAxis: {
        axisLabel: { interval: 0, rotate: rows.length > 6 ? 30 : 0 },
        data: rows.map((r) => r.employeeName ?? '-'),
        type: 'category',
      },
      yAxis: { type: 'value', splitLine: { lineStyle: { type: 'dashed' } } },
    });

    // 图表 3：跟进投入（客户跟进 vs 商机跟进 堆叠）
    renderFollow({
      grid: { bottom: 8, containLabel: true, left: 8, right: 8, top: 32 },
      legend: { top: 0 },
      series: [
        {
          barMaxWidth: 22,
          data: rows.map((r) => r.customerFollowUp),
          name: $t('page.statistics.customerFollowUp'),
          stack: 'fu',
          type: 'bar',
          color: '#0ea5e9',
        },
        {
          barMaxWidth: 22,
          data: rows.map((r) => r.opportunityFollowUp),
          name: $t('page.statistics.opportunityFollowUp'),
          stack: 'fu',
          type: 'bar',
          color: '#6366f1',
        },
      ],
      tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
      xAxis: {
        axisLabel: { interval: 0, rotate: rows.length > 6 ? 30 : 0 },
        data: rows.map((r) => r.employeeName ?? '-'),
        type: 'category',
      },
      yAxis: { type: 'value', splitLine: { lineStyle: { type: 'dashed' } } },
    });
  },
  // post：等待 v-else 的 EchartsUI 挂载完成后再渲染
  { deep: true, flush: 'post' },
);

// ---------- 全景榜表格 ----------
function formatCurrency(val: number) {
  return `¥${(val / 10_000).toFixed(1)}${$t('page.statistics.currencyFormat')}`;
}

const num = (r: MergedRow, key: keyof MergedRow) => Number(r[key]) || 0;

const boardColumns = computed(() => [
  {
    title: $t('page.statistics.rank'),
    width: 64,
    align: 'center' as const,
    customRender: ({ index }: any) =>
      index < 3 ? rankBadge(index + 1) : String(index + 1),
  },
  {
    title: $t('page.statistics.employee'),
    dataIndex: 'employeeName',
    width: 160,
  },
  {
    title: $t('page.statistics.department'),
    dataIndex: 'departmentName',
    width: 110,
    customRender: ({ text }: any) =>
      text ? h(Tag, { color: 'geekblue' }, () => text) : '-',
  },
  {
    title: $t('page.statistics.totalCustomers'),
    align: 'right' as const,
    dataIndex: 'totalCustomers',
    sorter: (a: any, b: any) =>
      num(a, 'totalCustomers') - num(b, 'totalCustomers'),
  },
  {
    title: $t('page.statistics.newCustomers'),
    align: 'right' as const,
    dataIndex: 'newCustomers',
    sorter: (a: any, b: any) => num(a, 'newCustomers') - num(b, 'newCustomers'),
  },
  {
    title: $t('page.statistics.contractCustomers'),
    align: 'right' as const,
    dataIndex: 'contractCustomers',
    sorter: (a: any, b: any) =>
      num(a, 'contractCustomers') - num(b, 'contractCustomers'),
  },
  {
    title: $t('page.statistics.conversionRate'),
    align: 'center' as const,
    dataIndex: 'customerConversionRate',
    width: 140,
    sorter: (a: any, b: any) =>
      num(a, 'customerConversionRate') - num(b, 'customerConversionRate'),
    customRender: ({ text }: any) =>
      h(Progress, {
        percent: Math.min(Number(text) || 0, 100),
        size: 'small' as const,
        strokeColor: '#0d9488',
        format: (p?: number) => `${(p ?? 0).toFixed(1)}%`,
      }),
  },
  {
    title: $t('page.statistics.employeeView.followUpTotalCol'),
    align: 'right' as const,
    dataIndex: 'totalFollowUp',
    sorter: (a: any, b: any) =>
      num(a, 'totalFollowUp') - num(b, 'totalFollowUp'),
  },
  {
    title: $t('page.statistics.opportunityWinRate'),
    align: 'center' as const,
    dataIndex: 'opportunityWinRate',
    width: 140,
    sorter: (a: any, b: any) =>
      num(a, 'opportunityWinRate') - num(b, 'opportunityWinRate'),
    customRender: ({ text }: any) =>
      h(Progress, {
        percent: Math.min(Number(text) || 0, 100),
        size: 'small' as const,
        strokeColor: '#10b981',
        format: (p?: number) => `${(p ?? 0).toFixed(1)}%`,
      }),
  },
  {
    title: $t('page.statistics.contractAmount'),
    align: 'right' as const,
    dataIndex: 'contractAmount',
    defaultSortOrder: 'descend' as const,
    sorter: (a: any, b: any) =>
      num(a, 'contractAmount') - num(b, 'contractAmount'),
    customRender: ({ text }: any) =>
      h(
        'span',
        { class: 'font-semibold text-teal-700' },
        formatCurrency(Number(text) || 0),
      ),
  },
  {
    title: $t('page.statistics.avgDealSize'),
    align: 'right' as const,
    dataIndex: 'avgContractAmount',
    sorter: (a: any, b: any) =>
      num(a, 'avgContractAmount') - num(b, 'avgContractAmount'),
    customRender: ({ text }: any) => formatCurrency(Number(text) || 0),
  },
  {
    title: $t('page.statistics.employeeView.avgCycleCol'),
    align: 'right' as const,
    dataIndex: 'avgSalesCycleDays',
    sorter: (a: any, b: any) =>
      num(a, 'avgSalesCycleDays') - num(b, 'avgSalesCycleDays'),
  },
]);

/** 排名徽章（前三金银铜） */
function rankBadge(rank: number) {
  let cls = 'rank-3';
  if (rank === 1) {
    cls = 'rank-1';
  } else if (rank === 2) {
    cls = 'rank-2';
  }
  return h('span', { class: ['rank-badge', cls] }, String(rank));
}
</script>

<template>
  <Page auto-content-height content-class="overflow-y-auto">
    <div class="employee-stats p-4 lg:p-6">
      <!-- 头部：标题 -->
      <div class="mb-4">
        <h2 class="text-xl font-bold tracking-tight">
          {{ $t('page.statistics.employeeStats') }}
        </h2>
        <p class="mt-1 text-xs text-gray-400">
          {{ $t('page.statistics.employeeView.subtitle') }}
        </p>
      </div>

      <!-- 筛选卡片：部门 + 时间 统一收纳 -->
      <Card
        class="filter-card fade-up mb-4"
        :body-style="{ padding: '12px 16px' }"
      >
        <div class="flex flex-wrap items-center gap-x-6 gap-y-2">
          <div class="flex items-center gap-2">
            <span class="filter-label">{{
              $t('page.statistics.department')
            }}</span>
            <TreeSelect
              :allow-clear="true"
              :dropdown-style="{ maxHeight: '320px', overflow: 'auto' }"
              :field-names="{
                children: 'children',
                label: 'label',
                value: 'value',
              }"
              :placeholder="$t('page.statistics.employeeView.deptAll')"
              :tree-data="deptTreeData"
              :value="departmentId"
              class="min-w-44"
              tree-default-expand-all
              @update:value="handleDeptSelect"
            />
          </div>
          <div class="filter-divider hidden lg:block"></div>
          <div class="flex items-center gap-2">
            <span class="filter-label">{{
              $t('page.statistics.employeeView.timeRange')
            }}</span>
            <TimeFilter @change="handleTimeChange" />
          </div>
          <div class="filter-divider hidden lg:block"></div>
          <!-- 口径切换：仅销售 = 当年已通过年度销售计划的员工（与业绩计划中心同一事实源） -->
          <div class="flex items-center gap-2">
            <span class="filter-label">统计口径</span>
            <Switch
              :checked="onlySales"
              checked-children="仅销售"
              un-checked-children="全员"
              @change="handleOnlySalesChange"
            />
            <span class="hidden text-xs text-gray-400 xl:inline">
              仅统计当年已通过年度销售计划的销售
            </span>
          </div>
        </div>
      </Card>

      <Spin :spinning="loading">
        <!-- KPI 总览 -->
        <div class="kpi-grid">
          <div
            v-for="(kpi, i) in kpiCards"
            :key="kpi.label"
            :style="{ '--accent': kpi.accent, animationDelay: `${i * 60}ms` }"
            class="kpi-card fade-up"
          >
            <div class="kpi-label">{{ kpi.label }}</div>
            <div class="kpi-value">{{ kpi.display }}</div>
            <div class="kpi-sub">{{ kpi.sub }}</div>
          </div>
        </div>

        <!-- 图表行 1：金额榜 + 商机转化 -->
        <Row :gutter="[16, 16]" class="mt-4">
          <Col :xs="24" :lg="10">
            <Card
              class="chart-card fade-up"
              :style="{ animationDelay: '120ms' }"
            >
              <template #title>
                <span class="card-kicker">{{
                  $t('page.statistics.employeeView.topPerformers')
                }}</span>
              </template>
              <div v-if="mergedRows.length === 0" class="chart-empty">
                {{ emptyHint }}
              </div>
              <EchartsUI v-else ref="topChartRef" height="320px" />
            </Card>
          </Col>
          <Col :xs="24" :lg="14">
            <Card
              class="chart-card fade-up"
              :style="{ animationDelay: '180ms' }"
            >
              <template #title>
                <span class="card-kicker">{{
                  $t('page.statistics.employeeView.oppConversion')
                }}</span>
              </template>
              <div v-if="mergedRows.length === 0" class="chart-empty">
                {{ emptyHint }}
              </div>
              <EchartsUI v-else ref="oppChartRef" height="320px" />
            </Card>
          </Col>
        </Row>

        <!-- 图表行 2：跟进投入 + 效率洞察 -->
        <Row :gutter="[16, 16]" class="mt-4">
          <Col :xs="24" :lg="14">
            <Card
              class="chart-card fade-up"
              :style="{ animationDelay: '240ms' }"
            >
              <template #title>
                <span class="card-kicker">{{
                  $t('page.statistics.employeeView.followInvest')
                }}</span>
              </template>
              <div v-if="mergedRows.length === 0" class="chart-empty">
                {{ emptyHint }}
              </div>
              <EchartsUI v-else ref="followChartRef" height="280px" />
            </Card>
          </Col>
          <Col :xs="24" :lg="10">
            <Card
              class="chart-card fade-up"
              :style="{ animationDelay: '300ms' }"
            >
              <template #title>
                <span class="card-kicker">{{
                  $t('page.statistics.employeeView.efficiency')
                }}</span>
              </template>
              <div class="eff-grid">
                <div v-for="e in efficiency" :key="e.key" class="eff-item">
                  <template v-if="e.key === 'interval'">
                    <div class="eff-label">
                      {{
                        $t(
                          'page.statistics.employeeView.avgFollowIntervalShort',
                        )
                      }}
                    </div>
                    <div class="eff-value">
                      {{ e.value }}<span class="eff-unit">{{ e.unit }}</span>
                    </div>
                  </template>
                  <template v-else-if="e.key === 'cycle'">
                    <div class="eff-label">
                      {{
                        $t('page.statistics.employeeView.avgSalesCycleShort')
                      }}
                    </div>
                    <div class="eff-value">
                      {{ e.value }}<span class="eff-unit">{{ e.unit }}</span>
                    </div>
                  </template>
                  <template v-else-if="e.key === 'deal'">
                    <div class="eff-label">
                      {{ $t('page.statistics.avgDealSize') }}
                    </div>
                    <div class="eff-value">{{ e.value }}</div>
                  </template>
                  <template v-else>
                    <div class="eff-label">
                      {{ $t('page.statistics.employeeView.noFollow30d') }}
                    </div>
                    <div class="eff-value eff-warn">{{ e.value }}</div>
                  </template>
                </div>
              </div>
            </Card>
          </Col>
        </Row>

        <!-- 员工全景榜 -->
        <!-- mt-6：上方图表行带垂直 gutter，antd 会写入行内 margin-bottom:-8px 抵消一半上边距 -->
        <Card
          class="mt-6 board-card fade-up"
          :style="{ animationDelay: '360ms' }"
        >
          <template #title>
            <span class="card-kicker">{{
              $t('page.statistics.employeeView.rankBoard')
            }}</span>
          </template>
          <Table
            :columns="boardColumns"
            :data-source="mergedRows"
            :pagination="false"
            :scroll="{ x: 1200 }"
            row-key="employeeId"
            size="middle"
          />
        </Card>
      </Spin>
    </div>
  </Page>
</template>

<style scoped>
.employee-stats {
  --ink: #134e4a;
  --teal: #0d9488;
}

/* ---- 筛选卡片 ---- */
.filter-card {
  border-radius: 10px;
}

.filter-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
  letter-spacing: 0.04em;
  white-space: nowrap;
}

.filter-divider {
  width: 1px;
  height: 20px;
  background: #e2e8f0;
}

/* TimeFilter 内嵌卡片时去除其自带底部间距 */
.filter-card :deep(> .ant-card-body > div > div > div:last-child > div) {
  margin-bottom: 0;
}

/* ---- KPI 卡 ---- */
.kpi-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
  gap: 12px;
}

.kpi-card {
  position: relative;
  padding: 14px 16px;
  overflow: hidden;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 10px;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;
}

.kpi-card::before {
  position: absolute;
  inset: 0 auto 0 0;
  width: 3px;
  content: '';
  background: var(--accent);
}

.kpi-card:hover {
  box-shadow: 0 8px 20px -8px rgb(0 0 0 / 15%);
  transform: translateY(-2px);
}

.kpi-label {
  font-size: 12px;
  color: #6b7280;
}

.kpi-value {
  margin-top: 6px;
  font-size: 22px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: #111827;
  letter-spacing: -0.02em;
}

.kpi-sub {
  margin-top: 4px;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  color: #9ca3af;
}

/* ---- 卡片小标题（编辑部风格 kicker） ---- */
.card-kicker {
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
  letter-spacing: 0.06em;
}

.chart-card {
  border-radius: 10px;
}

.chart-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 280px;
  font-size: 13px;
  color: #9ca3af;
}

/* ---- 效率洞察 2x2 ---- */
.eff-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.eff-item {
  padding: 14px;
  background: linear-gradient(145deg, #f8fafc, #f1f5f9);
  border: 1px solid #e2e8f0;
  border-radius: 10px;
}

.eff-label {
  font-size: 12px;
  color: #64748b;
}

.eff-value {
  margin-top: 6px;
  font-size: 20px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: #0f172a;
}

.eff-warn {
  color: #b45309;
}

.eff-unit {
  margin-left: 3px;
  font-size: 12px;
  font-weight: 400;
  color: #94a3b8;
}

/* ---- 排名徽章 ---- */
:deep(.rank-badge) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  font-size: 12px;
  font-weight: 700;
  color: #fff;
  border-radius: 9999px;
}

:deep(.rank-1) {
  background: linear-gradient(135deg, #fbbf24, #d97706);
  box-shadow: 0 2px 8px rgb(217 119 6 / 40%);
}

:deep(.rank-2) {
  background: linear-gradient(135deg, #cbd5e1, #64748b);
}

:deep(.rank-3) {
  background: linear-gradient(135deg, #d6a06a, #92400e);
}

.board-card {
  border-radius: 10px;
}

/* ---- 入场动画 ---- */
.fade-up {
  animation: fade-up 0.45s cubic-bezier(0.22, 1, 0.36, 1) both;
}

@keyframes fade-up {
  from {
    opacity: 0;
    transform: translateY(10px);
  }

  to {
    opacity: 1;
    transform: none;
  }
}
</style>
