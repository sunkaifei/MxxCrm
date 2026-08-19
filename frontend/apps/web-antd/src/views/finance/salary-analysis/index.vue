<script lang="ts" setup>
import type { EchartsUIType } from '@vben/plugins/echarts';

import { computed, onMounted, reactive, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { EchartsUI, useEcharts } from '@vben/plugins/echarts';

import {
  Button,
  Card,
  Col,
  Empty,
  Input,
  InputNumber,
  message,
  Row,
  Select,
  Spin,
  Statistic,
  Table,
} from 'ant-design-vue';
import { RefreshCw } from 'lucide-vue-next';

import {
  getSalaryTrendDepartmentApi,
  getSalaryTrendEmployeeApi,
  getSalaryTrendMonthlyApi,
  getSalaryTrendSummaryApi,
} from '#/api/core/finance';
import { $t } from '#/locales';

// ===== 查询条件 =====
const currentYear = new Date().getFullYear();
const queryParams = reactive({
  yearStart: currentYear - 2,
  yearEnd: currentYear,
  monthStart: undefined as number | undefined,
  monthEnd: undefined as number | undefined,
  departmentName: undefined as string | undefined,
  employeeName: undefined as string | undefined,
});

const monthOptions = Array.from({ length: 12 }, (_, i) => ({
  label: `${i + 1}${$t('page.finance.salaryAnalysis.unit.month')}`,
  value: i + 1,
}));

// ===== 汇总卡片 =====
const summaryLoading = ref(false);
const summary = ref<any>({});

async function loadSummary() {
  summaryLoading.value = true;
  try {
    summary.value = await getSalaryTrendSummaryApi({
      yearStart: queryParams.yearStart,
      yearEnd: queryParams.yearEnd,
      monthStart: queryParams.monthStart,
      monthEnd: queryParams.monthEnd,
      departmentName: queryParams.departmentName,
      employeeName: queryParams.employeeName,
    });
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salaryAnalysis.message.loadFailed'),
    );
  } finally {
    summaryLoading.value = false;
  }
}

// ===== 月度趋势折线图 =====
const monthlyChartRef = ref<EchartsUIType>();
const { renderEcharts: renderMonthlyChart } = useEcharts(monthlyChartRef);
const monthlyLoading = ref(false);

// 用户选择展示的工资科目
const selectedMetrics = ref<string[]>([
  'totalNet',
  'totalGross',
  'totalCommission',
]);
const metricOptions = computed(() => [
  {
    label: $t('page.finance.salaryAnalysis.metric.totalGross'),
    value: 'totalGross',
  },
  {
    label: $t('page.finance.salaryAnalysis.metric.totalNet'),
    value: 'totalNet',
  },
  {
    label: $t('page.finance.salaryAnalysis.metric.totalBase'),
    value: 'totalBase',
  },
  {
    label: $t('page.finance.salaryAnalysis.metric.totalCommission'),
    value: 'totalCommission',
  },
  {
    label: $t('page.finance.salaryAnalysis.metric.totalPerformance'),
    value: 'totalPerformance',
  },
  {
    label: $t('page.finance.salaryAnalysis.metric.totalTeamCommission'),
    value: 'totalTeamCommission',
  },
  {
    label: $t('page.finance.salaryAnalysis.metric.totalTax'),
    value: 'totalTax',
  },
  {
    label: $t('page.finance.salaryAnalysis.metric.avgNet'),
    value: 'avgNet',
  },
]);

const metricColorMap: Record<string, string> = {
  totalGross: '#5ab1ef',
  totalNet: '#019680',
  totalBase: '#faad14',
  totalCommission: '#ff6b6b',
  totalPerformance: '#a55eea',
  totalTeamCommission: '#26c281',
  totalTax: '#f7a04a',
  avgNet: '#4a90e2',
};

const metricLabelMap = computed<Record<string, string>>(() => ({
  totalGross: $t('page.finance.salaryAnalysis.metric.totalGross'),
  totalNet: $t('page.finance.salaryAnalysis.metric.totalNet'),
  totalBase: $t('page.finance.salaryAnalysis.metric.totalBase'),
  totalCommission: $t('page.finance.salaryAnalysis.metric.totalCommission'),
  totalPerformance: $t('page.finance.salaryAnalysis.metric.totalPerformance'),
  totalTeamCommission: $t(
    'page.finance.salaryAnalysis.metric.totalTeamCommission',
  ),
  totalTax: $t('page.finance.salaryAnalysis.metric.totalTax'),
  avgNet: $t('page.finance.salaryAnalysis.metric.avgNet'),
}));

async function loadMonthlyChart() {
  monthlyLoading.value = true;
  let monthly: any[] = [];
  try {
    monthly = await getSalaryTrendMonthlyApi({
      yearStart: queryParams.yearStart,
      yearEnd: queryParams.yearEnd,
      monthStart: queryParams.monthStart,
      monthEnd: queryParams.monthEnd,
      departmentName: queryParams.departmentName,
      employeeName: queryParams.employeeName,
    });
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salaryAnalysis.message.loadFailed'),
    );
    monthlyLoading.value = false;
    return;
  } finally {
    monthlyLoading.value = false;
  }

  const xData = monthly.map((m: any) => m.period);
  const metrics = selectedMetrics.value;

  const series = metrics.map((key) => ({
    name: metricLabelMap.value[key] || key,
    type: 'line' as const,
    smooth: true,
    areaStyle: metrics.length === 1 ? {} : undefined,
    data: monthly.map((m: any) => Number((m as any)[key] ?? 0)),
    itemStyle: { color: metricColorMap[key] || '#999' },
  }));

  renderMonthlyChart({
    grid: {
      top: 40,
      left: '2%',
      right: '2%',
      bottom: 10,
      containLabel: true,
    },
    legend: { data: series.map((s) => s.name), top: 5 },
    tooltip: {
      trigger: 'axis',
      valueFormatter: (val: any) => `¥${Number(val).toLocaleString()}`,
    },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: xData,
      axisTick: { show: false },
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        formatter: (val: number) =>
          val >= 10_000 ? `${(val / 10_000).toFixed(1)}万` : `${val}`,
      },
    },
    series,
  });
}

// ===== 部门对比柱状图 =====
const deptChartRef = ref<EchartsUIType>();
const { renderEcharts: renderDeptChart } = useEcharts(deptChartRef);
const deptLoading = ref(false);
const deptData = ref<any[]>([]);

async function loadDeptChart() {
  deptLoading.value = true;
  try {
    deptData.value = await getSalaryTrendDepartmentApi({
      yearStart: queryParams.yearStart,
      yearEnd: queryParams.yearEnd,
      monthStart: queryParams.monthStart,
      monthEnd: queryParams.monthEnd,
      departmentName: queryParams.departmentName,
      employeeName: queryParams.employeeName,
    });
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salaryAnalysis.message.loadFailed'),
    );
    deptLoading.value = false;
    return;
  } finally {
    deptLoading.value = false;
  }

  const top = [...deptData.value].slice(0, 15);
  const xData = top.map((d: any) => d.departmentName);
  renderDeptChart({
    grid: {
      top: 30,
      left: '2%',
      right: '2%',
      bottom: 10,
      containLabel: true,
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      valueFormatter: (val: any) => `¥${Number(val).toLocaleString()}`,
    },
    legend: {
      data: [
        $t('page.finance.salaryAnalysis.metric.totalNet'),
        $t('page.finance.salaryAnalysis.metric.totalGross'),
      ],
      top: 0,
    },
    xAxis: {
      type: 'category',
      data: xData,
      axisLabel: { interval: 0, rotate: xData.length > 6 ? 30 : 0 },
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        formatter: (val: number) =>
          val >= 10_000 ? `${(val / 10_000).toFixed(1)}万` : `${val}`,
      },
    },
    series: [
      {
        name: $t('page.finance.salaryAnalysis.metric.totalGross'),
        type: 'bar',
        data: top.map((d: any) => Number(d.totalGross ?? 0)),
        itemStyle: { color: '#5ab1ef' },
      },
      {
        name: $t('page.finance.salaryAnalysis.metric.totalNet'),
        type: 'bar',
        data: top.map((d: any) => Number(d.totalNet ?? 0)),
        itemStyle: { color: '#019680' },
      },
    ],
  });
}

// ===== 员工 TopN 排名 =====
const empChartRef = ref<EchartsUIType>();
const { renderEcharts: renderEmpChart } = useEcharts(empChartRef);
const empLoading = ref(false);
const empTopN = ref(10);

async function loadEmpChart() {
  empLoading.value = true;
  let list: any[];
  try {
    list = await getSalaryTrendEmployeeApi({
      yearStart: queryParams.yearStart,
      yearEnd: queryParams.yearEnd,
      monthStart: queryParams.monthStart,
      monthEnd: queryParams.monthEnd,
      departmentName: queryParams.departmentName,
      employeeName: queryParams.employeeName,
      limit: empTopN.value,
    });
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salaryAnalysis.message.loadFailed'),
    );
    empLoading.value = false;
    return;
  } finally {
    empLoading.value = false;
  }

  // 横向柱状图（倒序排列让最大的在最上方）
  const reversed = list.toReversed();
  const yData = reversed.map((d: any) => d.employeeName);
  renderEmpChart({
    grid: {
      top: 30,
      left: '3%',
      right: '4%',
      bottom: 10,
      containLabel: true,
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      valueFormatter: (val: any) => `¥${Number(val).toLocaleString()}`,
    },
    legend: {
      data: [
        $t('page.finance.salaryAnalysis.metric.totalNet'),
        $t('page.finance.salaryAnalysis.metric.totalCommission'),
      ],
      top: 0,
    },
    xAxis: {
      type: 'value',
      axisLabel: {
        formatter: (val: number) =>
          val >= 10_000 ? `${(val / 10_000).toFixed(1)}万` : `${val}`,
      },
    },
    yAxis: { type: 'category', data: yData },
    series: [
      {
        name: $t('page.finance.salaryAnalysis.metric.totalNet'),
        type: 'bar',
        data: reversed.map((d: any) => Number(d.totalNet ?? 0)),
        itemStyle: { color: '#019680' },
      },
      {
        name: $t('page.finance.salaryAnalysis.metric.totalCommission'),
        type: 'bar',
        data: reversed.map((d: any) => Number(d.totalCommission ?? 0)),
        itemStyle: { color: '#ff6b6b' },
      },
    ],
  });
}

// ===== 数据表格 =====
const empColumns = computed(() => [
  {
    title: $t('page.finance.salaryAnalysis.column.rank'),
    width: 60,
    key: 'rank',
    customRender: ({ index }: any) => index + 1,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.employeeName'),
    dataIndex: 'employeeName',
    width: 120,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.department'),
    dataIndex: 'departmentName',
    width: 120,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.salaryAnalysis.column.months'),
    dataIndex: 'months',
    width: 90,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.totalBase'),
    dataIndex: 'totalBase',
    width: 130,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.totalCommission'),
    dataIndex: 'totalCommission',
    width: 130,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.totalGross'),
    dataIndex: 'totalGross',
    width: 130,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.totalNet'),
    dataIndex: 'totalNet',
    width: 130,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.avgMonthlyNet'),
    dataIndex: 'avgMonthlyNet',
    width: 140,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
]);

const deptColumns = computed(() => [
  {
    title: $t('page.finance.salaryAnalysis.column.rank'),
    width: 60,
    key: 'rank',
    customRender: ({ index }: any) => index + 1,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.department'),
    dataIndex: 'departmentName',
    width: 160,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.headcount'),
    dataIndex: 'headcount',
    width: 100,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.totalBase'),
    dataIndex: 'totalBase',
    width: 130,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.totalCommission'),
    dataIndex: 'totalCommission',
    width: 130,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.totalGross'),
    dataIndex: 'totalGross',
    width: 130,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.totalNet'),
    dataIndex: 'totalNet',
    width: 130,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.salaryAnalysis.column.avgNet'),
    dataIndex: 'avgNet',
    width: 130,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
]);

// 完整员工排名表（TopN 之外展示更多）
const empTableData = ref<any[]>([]);
const empTableLoading = ref(false);

async function loadEmpTable() {
  empTableLoading.value = true;
  try {
    empTableData.value = await getSalaryTrendEmployeeApi({
      yearStart: queryParams.yearStart,
      yearEnd: queryParams.yearEnd,
      monthStart: queryParams.monthStart,
      monthEnd: queryParams.monthEnd,
      departmentName: queryParams.departmentName,
      employeeName: queryParams.employeeName,
      limit: 100,
    });
  } catch {
    empTableData.value = [];
  } finally {
    empTableLoading.value = false;
  }
}

// ===== 统一加载 / 刷新 =====
const allLoading = computed(
  () =>
    summaryLoading.value ||
    monthlyLoading.value ||
    deptLoading.value ||
    empLoading.value,
);

async function loadAll() {
  await Promise.all([
    loadSummary(),
    loadMonthlyChart(),
    loadDeptChart(),
    loadEmpChart(),
    loadEmpTable(),
  ]);
}

function handleRefresh() {
  loadAll();
}

// 监听 metrics 变化，重绘月度图（不重新拉接口）
watch(selectedMetrics, () => {
  loadMonthlyChart();
});

// 监听 TopN 变化，重新拉接口
watch(empTopN, () => {
  loadEmpChart();
});

onMounted(() => {
  loadAll();
});
</script>

<template>
  <Page :title="$t('page.finance.salaryAnalysis.title')">
    <!-- 筛选区 -->
    <Card class="mb-4" :bordered="false">
      <div class="flex flex-wrap items-center gap-3">
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-600">{{
            $t('page.finance.salaryAnalysis.filter.yearStart')
          }}</span>
          <InputNumber
            v-model:value="queryParams.yearStart"
            :min="2000"
            :max="2100"
            style="width: 110px"
          />
        </div>
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-600">{{
            $t('page.finance.salaryAnalysis.filter.yearEnd')
          }}</span>
          <InputNumber
            v-model:value="queryParams.yearEnd"
            :min="2000"
            :max="2100"
            style="width: 110px"
          />
        </div>
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-600">{{
            $t('page.finance.salaryAnalysis.filter.monthStart')
          }}</span>
          <Select
            v-model:value="queryParams.monthStart"
            :options="monthOptions"
            :placeholder="
              $t('page.finance.salaryAnalysis.filter.monthPlaceholder')
            "
            allow-clear
            style="width: 110px"
          />
        </div>
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-600">{{
            $t('page.finance.salaryAnalysis.filter.monthEnd')
          }}</span>
          <Select
            v-model:value="queryParams.monthEnd"
            :options="monthOptions"
            :placeholder="
              $t('page.finance.salaryAnalysis.filter.monthPlaceholder')
            "
            allow-clear
            style="width: 110px"
          />
        </div>
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-600">{{
            $t('page.finance.salaryAnalysis.filter.department')
          }}</span>
          <Input
            v-model:value="queryParams.departmentName"
            :placeholder="
              $t('page.finance.salaryAnalysis.filter.departmentPlaceholder')
            "
            allow-clear
            style="width: 160px"
          />
        </div>
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-600">{{
            $t('page.finance.salaryAnalysis.filter.employee')
          }}</span>
          <Input
            v-model:value="queryParams.employeeName"
            :placeholder="
              $t('page.finance.salaryAnalysis.filter.employeePlaceholder')
            "
            allow-clear
            style="width: 160px"
          />
        </div>
        <Button type="primary" :loading="allLoading" @click="handleRefresh">
          <template #icon>
            <RefreshCw class="size-4" />
          </template>
          {{ $t('page.finance.salaryAnalysis.button.refresh') }}
        </Button>
      </div>
    </Card>

    <!-- KPI 汇总卡片 -->
    <Spin :spinning="summaryLoading">
      <Row :gutter="16" class="mb-4">
        <Col :xs="24" :sm="12" :md="8" :lg="4">
          <Card>
            <Statistic
              :title="$t('page.finance.salaryAnalysis.summary.totalGross')"
              :value="summary.totalGross || 0"
              :precision="2"
              prefix="¥"
            />
          </Card>
        </Col>
        <Col :xs="24" :sm="12" :md="8" :lg="4">
          <Card>
            <Statistic
              :title="$t('page.finance.salaryAnalysis.summary.totalNet')"
              :value="summary.totalNet || 0"
              :precision="2"
              prefix="¥"
              :value-style="{ color: '#3f8600' }"
            />
          </Card>
        </Col>
        <Col :xs="24" :sm="12" :md="8" :lg="4">
          <Card>
            <Statistic
              :title="$t('page.finance.salaryAnalysis.summary.totalCommission')"
              :value="summary.totalCommission || 0"
              :precision="2"
              prefix="¥"
            />
          </Card>
        </Col>
        <Col :xs="24" :sm="12" :md="8" :lg="4">
          <Card>
            <Statistic
              :title="$t('page.finance.salaryAnalysis.summary.totalTax')"
              :value="summary.totalTax || 0"
              :precision="2"
              prefix="¥"
              :value-style="{ color: '#cf1322' }"
            />
          </Card>
        </Col>
        <Col :xs="24" :sm="12" :md="8" :lg="4">
          <Card>
            <Statistic
              :title="$t('page.finance.salaryAnalysis.summary.totalHeadcount')"
              :value="summary.totalHeadcount || 0"
              :suffix="$t('page.finance.salaryAnalysis.unit.person')"
            />
          </Card>
        </Col>
        <Col :xs="24" :sm="12" :md="8" :lg="4">
          <Card>
            <Statistic
              :title="$t('page.finance.salaryAnalysis.summary.avgMonthlyNet')"
              :value="summary.avgMonthlyNet || 0"
              :precision="2"
              prefix="¥"
            />
          </Card>
        </Col>
      </Row>
    </Spin>

    <!-- 月度趋势折线图 -->
    <Card
      class="mb-4"
      :bordered="false"
      :title="$t('page.finance.salaryAnalysis.chart.monthlyTitle')"
    >
      <template #extra>
        <Select
          v-model:value="selectedMetrics"
          mode="multiple"
          :options="metricOptions"
          :max-tag-count="3"
          style="min-width: 240px"
          :placeholder="
            $t('page.finance.salaryAnalysis.chart.metricPlaceholder')
          "
        />
      </template>
      <Spin :spinning="monthlyLoading">
        <Empty
          v-if="!monthlyLoading && selectedMetrics.length === 0"
          :description="$t('page.finance.salaryAnalysis.chart.empty')"
        />
        <EchartsUI ref="monthlyChartRef" style="height: 360px" />
      </Spin>
    </Card>

    <!-- 部门对比 + 员工排名 -->
    <Row :gutter="16" class="mb-4">
      <Col :xs="24" :lg="12">
        <Card
          :bordered="false"
          :title="$t('page.finance.salaryAnalysis.chart.deptTitle')"
        >
          <Spin :spinning="deptLoading">
            <EchartsUI ref="deptChartRef" style="height: 360px" />
          </Spin>
        </Card>
      </Col>
      <Col :xs="24" :lg="12">
        <Card
          :bordered="false"
          :title="$t('page.finance.salaryAnalysis.chart.empTitle')"
        >
          <template #extra>
            <Select
              v-model:value="empTopN"
              :options="[
                { label: 'Top 5', value: 5 },
                { label: 'Top 10', value: 10 },
                { label: 'Top 20', value: 20 },
                { label: 'Top 50', value: 50 },
              ]"
              style="width: 100px"
            />
          </template>
          <Spin :spinning="empLoading">
            <EchartsUI ref="empChartRef" style="height: 360px" />
          </Spin>
        </Card>
      </Col>
    </Row>

    <!-- 部门明细表 -->
    <Card
      class="mb-4"
      :bordered="false"
      :title="$t('page.finance.salaryAnalysis.table.deptTitle')"
    >
      <Table
        :data-source="deptData"
        :columns="deptColumns"
        :pagination="false"
        row-key="departmentName"
        size="small"
        :scroll="{ x: 1000 }"
      >
        <template #emptyText>
          <Empty
            :description="$t('page.finance.salaryAnalysis.message.empty')"
          />
        </template>
      </Table>
    </Card>

    <!-- 员工排名明细表 -->
    <Card
      :bordered="false"
      :title="$t('page.finance.salaryAnalysis.table.empTitle')"
    >
      <Table
        :data-source="empTableData"
        :columns="empColumns"
        :pagination="{ pageSize: 20, showSizeChanger: true }"
        row-key="employeeId"
        size="small"
        :loading="empTableLoading"
        :scroll="{ x: 1100 }"
      >
        <template #emptyText>
          <Empty
            :description="$t('page.finance.salaryAnalysis.message.empty')"
          />
        </template>
      </Table>
    </Card>
  </Page>
</template>
