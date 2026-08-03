<script lang="ts" setup>
import { computed, onMounted, ref, watch } from 'vue';

import { useAccess } from '@vben/access';
import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';
import { useUserStore } from '@vben/stores';

import {
  Badge,
  Button,
  Card,
  Col,
  Empty,
  Progress,
  Row,
  Segmented,
  Select,
  Spin,
  Table,
  Tabs,
  Tag,
  Tooltip,
  message,
} from 'ant-design-vue';

import {
  exportPerformanceApi,
  getMonthlyPerformanceApi,
  getPerformanceComparisonApi,
  getPerformanceForecastApi,
  getPerformanceRankingApi,
  getPlanListApi,
} from '#/api/core/statistics';

import BehaviorMetrics from './components/BehaviorMetrics.vue';
import CustomerBreakdown from './components/CustomerBreakdown.vue';
import ForecastCard from './components/ForecastCard.vue';
import MilestoneCard from './components/MilestoneCard.vue';
import PersonalGrowth from './components/PersonalGrowth.vue';
import ProductBreakdown from './components/ProductBreakdown.vue';
import ProgressAlert from './components/ProgressAlert.vue';
import RegionBreakdown from './components/RegionBreakdown.vue';
import SalesFunnel from './components/SalesFunnel.vue';
import PlanProgressCard from './components/PlanProgressCard.vue';
import PlanSettingDrawer from './PlanSettingDrawer.vue';
import PendingApprovalList from './components/PendingApprovalList.vue';

defineOptions({ name: 'PerformanceOverview' });

const userStore = useUserStore();
const { hasAccessByCodes } = useAccess();

// 销售计划权限：没有权限则不渲染计划进度卡片、不调用计划相关 API
const hasPlanPermission = computed(() =>
  hasAccessByCodes(['statistics:performance-plan:view']),
);
// 销售计划管理权限：仅有 view 权限的用户（如业务员）看不到设置按钮，无法填写
const hasPlanManagePermission = computed(() =>
  hasAccessByCodes(['statistics:performance-plan:manage']),
);
// 销售计划审批权限：有此权限的用户可审批下属计划
const hasPlanApprovePermission = computed(() =>
  hasAccessByCodes(['statistics:performance-plan:approve']),
);

// ===== 时间维度 =====
type TimeDimension = 'year' | 'month' | 'day';
const timeDimension = ref<TimeDimension>('year');
const selectedYear = ref(new Date().getFullYear());
const selectedMonth = ref(new Date().getMonth() + 1);

const yearOptions = computed(() => {
  const current = new Date().getFullYear();
  return Array.from({ length: 5 }, (_, i) => ({
    value: current - i,
    label: `${current - i}年`,
  }));
});

const monthOptions = Array.from({ length: 12 }, (_, i) => ({
  value: i + 1,
  label: `${i + 1}月`,
}));

const dimensionOptions = [
  { label: '按年', value: 'year' },
  { label: '按月', value: 'month' },
  { label: '按日', value: 'day' },
];

// ===== 角色判断 =====
// data_scope: 1=全部数据, 3=本部门, 4=本部门及以下, 5=仅本人
const userRole = computed(() => {
  const scope = userStore.userInfo?.dataScope;
  const roles = userStore.userInfo?.roles || [];
  const roleCodes = roles
    .map((r: any) => r.code || r.roleCode || '')
    .filter(Boolean);
  const isBoss =
    roleCodes.includes('super_admin') ||
    roleCodes.includes('system_admin') ||
    roleCodes.includes('boss') ||
    roleCodes.includes('gm');
  if (isBoss || scope === 1) return 'company';
  if (scope === 3 || scope === 4) return 'dept';
  return 'personal';
});

const isCompanyView = computed(() => userRole.value === 'company');
const isDeptView = computed(() => userRole.value === 'dept');
const isPersonalView = computed(() => userRole.value === 'personal');

// ===== 数据加载 =====
const loading = ref(false);
const exporting = ref(false);
const monthlyData = ref<any[]>([]);
const rankingData = ref<any[]>([]);
const personalData = ref<any>({});
const comparisonData = ref<any>({});
const forecastData = ref<any>({});

const queryParams = computed(() => ({
  year: selectedYear.value,
  month: timeDimension.value !== 'year' ? selectedMonth.value : undefined,
  time_dimension: timeDimension.value,
}));

async function loadData() {
  loading.value = true;
  try {
    const params: any = { ...queryParams.value };
    const [monthlyRes, rankingRes, comparisonRes, forecastRes] =
      await Promise.all([
        getMonthlyPerformanceApi(params),
        getPerformanceRankingApi({
          ...params,
          order_by: 'contract_amount',
        }),
        getPerformanceComparisonApi(params).catch(() => ({})),
        getPerformanceForecastApi(params).catch(() => ({})),
      ]);

    monthlyData.value = monthlyRes?.data?.months || monthlyRes?.months || [];
    rankingData.value =
      rankingRes?.data?.map((item: any) => ({
        rank: item.rank,
        employeeName: item.employeeName || item.employee_name,
        departmentName: item.departmentName || item.department_name,
        contractAmount: item.contractAmount || item.contract_amount || 0,
        contractTarget: item.contractTarget || item.contract_target || 0,
        paymentAmount: item.paymentAmount || item.payment_amount || 0,
        paymentTarget: item.paymentTarget || item.payment_target || 0,
        completionRate:
          item.contractCompletionRate || item.contract_completion_rate || 0,
        monthOnMonth: item.monthOnMonth || item.month_on_month || 0,
      })) || [];

    comparisonData.value = comparisonRes?.data || comparisonRes || {};
    forecastData.value = forecastRes?.data || forecastRes || {};

    // 个人视图额外加载自己的数据
    if (isPersonalView.value) {
      personalData.value =
        rankingRes?.data?.find(
          (r: any) => r.employeeId === userStore.userInfo?.userId,
        ) || {};
    }
  } catch (e) {
    console.error('加载业绩数据失败', e);
    monthlyData.value = [];
    rankingData.value = [];
  } finally {
    loading.value = false;
  }
}

onMounted(() => loadData());

// ===== 汇总计算 =====
// 后端 Decimal 经 msgpack 序列化后为字符串，统一用 Number() 转换避免字符串拼接
const totalContractTarget = computed(() =>
  monthlyData.value.reduce(
    (s, m) => s + Number(m.contractTarget || m.contract_target || 0),
    0,
  ),
);
const totalContractActual = computed(() =>
  monthlyData.value.reduce(
    (s, m) => s + Number(m.contractActual || m.contract_actual || 0),
    0,
  ),
);
const totalPaymentTarget = computed(() =>
  monthlyData.value.reduce(
    (s, m) => s + Number(m.paymentTarget || m.payment_target || 0),
    0,
  ),
);
const totalPaymentActual = computed(() =>
  monthlyData.value.reduce(
    (s, m) => s + Number(m.paymentActual || m.payment_actual || 0),
    0,
  ),
);
const contractCount = computed(() =>
  monthlyData.value.reduce(
    (s, m) => s + Number(m.contractCount || m.contract_count || 0),
    0,
  ),
);
const avgDealSize = computed(() => {
  const cnt = contractCount.value || 0;
  return cnt > 0 ? totalContractActual.value / cnt : 0;
});

function getRate(target: any, actual: any) {
  const t = Number(target);
  const a = Number(actual);
  return t > 0 ? Math.min((a / t) * 100, 100).toFixed(2) : '0';
}

// 百分比统一格式化：小数点后两位
function formatPercent(val: any) {
  const n = Number(val);
  if (!n || Number.isNaN(n)) return '0';
  return n.toFixed(2);
}

function formatCurrency(val: any) {
  const num = Number(val);
  if (!num || Number.isNaN(num)) return '¥0';
  if (num >= 100000000) return `¥${(num / 100000000).toFixed(2)}亿`;
  if (num >= 10000) return `¥${(num / 10000).toFixed(1)}万`;
  return `¥${num.toLocaleString()}`;
}

// ===== 进度预警条 =====
const timeProgress = computed(() => {
  const now = new Date();
  if (timeDimension.value === 'year') {
    const start = new Date(now.getFullYear(), 0, 1).getTime();
    const end = new Date(now.getFullYear() + 1, 0, 1).getTime();
    return ((now.getTime() - start) / (end - start)) * 100;
  }
  if (timeDimension.value === 'month') {
    const start = new Date(now.getFullYear(), now.getMonth(), 1).getTime();
    const end = new Date(now.getFullYear(), now.getMonth() + 1, 1).getTime();
    return ((now.getTime() - start) / (end - start)) * 100;
  }
  // day
  return 100;
});

const performanceProgress = computed(() => {
  if (totalContractTarget.value === 0) return 0;
  return (totalContractActual.value / totalContractTarget.value) * 100;
});

// ===== 维度拆解 Tab =====
const breakdownTab = ref<
  'dept' | 'employee' | 'customer' | 'product' | 'region'
>('dept');

// ===== 部门排名聚合 =====
const deptRanking = computed(() => {
  const deptMap = new Map<string, any>();
  rankingData.value.forEach((item) => {
    const dept = item.departmentName || '未分配';
    if (!deptMap.has(dept)) {
      deptMap.set(dept, {
        ...item,
        employeeName: dept,
        contractAmount: 0,
        contractTarget: 0,
        paymentAmount: 0,
        paymentTarget: 0,
      });
    }
    const d = deptMap.get(dept);
    d.contractAmount += item.contractAmount || 0;
    d.contractTarget += item.contractTarget || 0;
    d.paymentAmount += item.paymentAmount || 0;
    d.paymentTarget += item.paymentTarget || 0;
  });
  const arr = Array.from(deptMap.values());
  arr.forEach((d, i) => {
    d.rank = i + 1;
    d.completionRate = getRate(d.contractTarget, d.contractAmount);
  });
  return arr;
});

function rankingColumnsFor(type: 'dept' | 'employee') {
  return [
    { title: '排名', dataIndex: 'rank', width: 70, fixed: 'left' as const },
    {
      title: type === 'dept' ? '部门' : '销售员',
      dataIndex: 'employeeName',
      width: 120,
    },
    ...(type === 'employee'
      ? [{ title: '部门', dataIndex: 'departmentName', width: 120 }]
      : []),
    {
      title: '合同金额',
      dataIndex: 'contractAmount',
      align: 'right' as const,
      width: 120,
    },
    {
      title: '合同目标',
      dataIndex: 'contractTarget',
      align: 'right' as const,
      width: 120,
    },
    {
      title: '完成率',
      dataIndex: 'completionRate',
      align: 'right' as const,
      width: 100,
    },
    {
      title: '回款金额',
      dataIndex: 'paymentAmount',
      align: 'right' as const,
      width: 120,
    },
  ];
}

// ===== 个人销售计划抽屉 =====
const planDrawerVisible = ref(false);
const planStatus = ref<'none' | 'draft' | 'pending' | 'approved' | 'rejected'>(
  'none',
);

// ===== 待我审批抽屉 =====
const pendingApprovalVisible = ref(false);
const pendingApprovalCount = ref(0);

async function checkPlanStatus() {
  // 无销售计划查看权限时直接跳过，避免 403 报错
  if (!hasPlanPermission.value) {
    planStatus.value = 'none';
    return;
  }
  try {
    const employeeId = userStore.userInfo?.userId || userStore.userInfo?.id;
    const res: any = await getPlanListApi({ year: selectedYear.value, employeeId });
    // requestClient 已配置 responseReturn: 'data'，res 本身就是 plans 数组
    const plans = Array.isArray(res) ? res : (res?.data || []);
    if (plans.length === 0) {
      planStatus.value = 'none';
    } else {
      const statusNum = Number(plans[0].status);
      planStatus.value =
        (['none', 'draft', 'pending', 'approved', 'rejected'] as const)[statusNum] ||
        'none';
    }
    // 同时加载待审批数量
    if (hasPlanApprovePermission.value) {
      await loadPendingCount();
    }
  } catch {
    planStatus.value = 'none';
  }
}

// 加载待我审批的计划数量
async function loadPendingCount() {
  try {
    const res: any = await getPlanListApi({
      year: selectedYear.value,
      pendingMyApproval: true,
    });
    const plans = Array.isArray(res) ? res : (res?.data || []);
    pendingApprovalCount.value = plans.length;
  } catch {
    pendingApprovalCount.value = 0;
  }
}

onMounted(() => checkPlanStatus());

// 年份切换时重新检查计划状态
watch(
  () => selectedYear.value,
  () => checkPlanStatus(),
);

function openPlanDrawer() {
  planDrawerVisible.value = true;
}

const planButtonConfig = computed(() => {
  switch (planStatus.value) {
    case 'none':
      return { text: '设置销售计划', color: '#ff4d4f', icon: 'lucide:alert-circle', show: true };
    case 'draft':
      return { text: '编辑计划（草稿）', color: '#faad14', icon: 'lucide:edit', show: true };
    case 'pending':
      return { text: '查看计划（审批中）', color: '#1890ff', icon: 'lucide:clock', show: true };
    case 'approved':
      // 审批通过后隐藏入口（用户选择"隐藏入口仅留查看"）
      return { text: '查看计划', color: '#52c41a', icon: 'lucide:eye', show: true };
    case 'rejected':
      return { text: '重新提交计划', color: '#ff4d4f', icon: 'lucide:rotate-ccw', show: true };
    default:
      return { text: '设置销售计划', color: '#1890ff', icon: 'lucide:target', show: true };
  }
});

// 审批通过后顶部按钮隐藏（仅保留在计划进度卡片中显示状态）
const showPlanButton = computed(() =>
  planButtonConfig.value.show && planStatus.value !== 'approved',
);

// 处理待审批抽屉刷新
function handlePendingRefresh() {
  loadPendingCount();
  checkPlanStatus();
}

// ===== 同比环比箭头 =====
function trendArrow(yoy?: number, mom?: number) {
  if (yoy === undefined && mom === undefined) return null;
  const value = yoy ?? mom ?? 0;
  if (value > 0) return { color: '#52c41a', icon: 'lucide:trending-up', text: `↑${formatPercent(value)}%` };
  if (value < 0) return { color: '#ff4d4f', icon: 'lucide:trending-down', text: `↓${formatPercent(Math.abs(value))}%` };
  return { color: '#8c8c8c', icon: 'lucide:minus', text: '0%' };
}

// ===== 8 个 KPI 卡片配置（带同比环比） =====
const kpiCards = computed(() => {
  const comp = comparisonData.value || {};
  return [
    {
      title: '合同目标',
      value: formatCurrency(totalContractTarget.value),
      icon: 'lucide:target',
      color: '#1890ff',
      bg: '#e6f7ff',
      trend: null,
    },
    {
      title: '合同实际',
      value: formatCurrency(totalContractActual.value),
      sub: `完成 ${getRate(totalContractTarget.value, totalContractActual.value)}%`,
      progress: Number(getRate(totalContractTarget.value, totalContractActual.value)),
      icon: 'lucide:file-check',
      color: '#52c41a',
      bg: '#f6ffed',
      trend: trendArrow(comp?.contract?.yoy, comp?.contract?.mom),
    },
    {
      title: '回款目标',
      value: formatCurrency(totalPaymentTarget.value),
      icon: 'lucide:wallet',
      color: '#722ed1',
      bg: '#f9f0ff',
      trend: null,
    },
    {
      title: '回款实际',
      value: formatCurrency(totalPaymentActual.value),
      sub: `完成 ${getRate(totalPaymentTarget.value, totalPaymentActual.value)}%`,
      progress: Number(getRate(totalPaymentTarget.value, totalPaymentActual.value)),
      icon: 'lucide:dollar-sign',
      color: '#fa8c16',
      bg: '#fff7e6',
      trend: trendArrow(comp?.payment?.yoy, comp?.payment?.mom),
    },
    {
      title: '客单价',
      value: formatCurrency(avgDealSize.value),
      icon: 'lucide:receipt',
      color: '#13c2c2',
      bg: '#e6fffb',
      trend: trendArrow(comp?.avgDealSize?.yoy, comp?.avgDealSize?.mom),
    },
    {
      title: '合同数',
      value: `${contractCount.value} 个`,
      icon: 'lucide:files',
      color: '#eb2f96',
      bg: '#fff0f6',
      trend: trendArrow(comp?.contractCount?.yoy, comp?.contractCount?.mom),
    },
    {
      title: '在途商机',
      value: formatCurrency(forecastData.value?.pipelineAmount || 0),
      icon: 'lucide:pipeline',
      color: '#2f54eb',
      bg: '#f0f5ff',
      trend: null,
    },
    {
      title: 'Pipeline覆盖率',
      value: `${(forecastData.value?.pipelineCoverage || 0).toFixed(2)} 倍`,
      icon: 'lucide:shield-check',
      color: '#a0d911',
      bg: '#fcffe6',
      trend: null,
    },
  ];
});

// 个人卡片（仅普通销售显示）
const personalCards = computed(() => [
  {
    title: '本月目标',
    value: formatCurrency(personalData.value?.contractTarget || 0),
    icon: 'lucide:target',
    color: '#1890ff',
  },
  {
    title: '本月实际',
    value: formatCurrency(personalData.value?.contractAmount || 0),
    sub: `完成 ${formatPercent(personalData.value?.completionRate || 0)}%`,
    icon: 'lucide:check-circle',
    color: '#52c41a',
  },
  {
    title: '全公司排名',
    value: `第 ${personalData.value?.rank || '-'} 名`,
    icon: 'lucide:trophy',
    color: '#fa8c16',
  },
  {
    title: '本部门排名',
    value: `第 ${personalData.value?.deptRank || '-'} 名`,
    icon: 'lucide:users',
    color: '#eb2f96',
  },
]);

// 月度趋势图（CSS 实现）
const maxMonthlyValue = computed(() => {
  return Math.max(
    ...monthlyData.value.map((m) =>
      Math.max(
        Number(m.contractTarget || m.contract_target || 0),
        Number(m.contractActual || m.contract_actual || 0),
      ),
    ),
    1,
  );
});

function barHeight(val: number) {
  return `${(val / maxMonthlyValue.value) * 180}px`;
}

// ===== 导出 =====
async function handleExport(format: 'excel' | 'pdf') {
  exporting.value = true;
  try {
    const blob: any = await exportPerformanceApi({
      format,
      ...queryParams.value,
    });
    const url = window.URL.createObjectURL(
      new Blob([blob], {
        type:
          format === 'excel'
            ? 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
            : 'application/pdf',
      }),
    );
    const link = document.createElement('a');
    link.href = url;
    link.download = `业绩概览_${selectedYear.value}.${
      format === 'excel' ? 'xlsx' : 'pdf'
    }`;
    link.click();
    window.URL.revokeObjectURL(url);
    message.success('导出成功');
  } catch (e: any) {
    message.error(e?.message || '导出失败');
  } finally {
    exporting.value = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <Spin :spinning="loading">
      <!-- ============ 模块1：顶部工具栏 ============ -->
      <Card class="mb-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <IconifyIcon icon="lucide:bar-chart-3" class="text-xl text-primary" />
            <span class="text-lg font-semibold">
              {{
                isCompanyView
                  ? '全公司业绩概览'
                  : isDeptView
                    ? '本部门业绩概览'
                    : '我的业绩概览'
              }}
            </span>
            <Tag v-if="isCompanyView" color="blue">全公司可见</Tag>
            <Tag v-else-if="isDeptView" color="cyan">本部门可见</Tag>
            <Tag v-else color="orange">仅本人可见</Tag>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <Segmented
              v-model:value="timeDimension"
              :options="dimensionOptions"
              size="small"
            />
            <Select
              v-model:value="selectedYear"
              :options="yearOptions"
              size="small"
              style="width: 100px"
            />
            <Select
              v-if="timeDimension !== 'year'"
              v-model:value="selectedMonth"
              :options="monthOptions"
              size="small"
              style="width: 80px"
            />
            <Button type="primary" size="small" @click="loadData">
              <template #icon>
                <IconifyIcon icon="lucide:refresh-cw" />
              </template>
              刷新
            </Button>
            <Button size="small" :loading="exporting" @click="handleExport('excel')">
              <template #icon>
                <IconifyIcon icon="lucide:file-spreadsheet" />
              </template>
              导出 Excel
            </Button>
            <Button size="small" :loading="exporting" @click="handleExport('pdf')">
              <template #icon>
                <IconifyIcon icon="lucide:file-text" />
              </template>
              导出 PDF
            </Button>
            <!-- 待我审批按钮（有审批权限且有待审计划时显示） -->
            <Badge
              v-if="hasPlanApprovePermission && pendingApprovalCount > 0"
              :count="pendingApprovalCount"
              :offset="[-4, 4]"
            >
              <Button
                type="primary"
                size="small"
                ghost
                @click="pendingApprovalVisible = true"
              >
                <template #icon>
                  <IconifyIcon icon="lucide:clipboard-check" />
                </template>
                待我审批
              </Button>
            </Badge>

            <!-- 个人计划设置/查看按钮（审批通过后隐藏入口） -->
            <Button
              v-if="hasPlanManagePermission && showPlanButton"
              :type="planStatus === 'none' && isPersonalView ? 'primary' : 'default'"
              :danger="planStatus === 'none' && isPersonalView"
              size="small"
              @click="openPlanDrawer"
            >
              <template #icon>
                <IconifyIcon :icon="planButtonConfig.icon" />
              </template>
              {{ planButtonConfig.text }}
            </Button>
          </div>
        </div>
      </Card>

      <!-- ============ 模块2：进度预警条 ============ -->
      <div class="mb-4 mt-4">
        <ProgressAlert
          :time-progress="timeProgress"
          :performance-progress="performanceProgress"
        />
      </div>

      <!-- ============ 模块3：8 个 KPI 卡片（带同比环比箭头） ============ -->
      <Row :gutter="[16, 16]" class="mb-4">
        <Col
          v-for="card in kpiCards"
          :key="card.title"
          :xs="24"
          :sm="12"
          :md="6"
        >
          <Card :body-style="{ padding: '20px' }" class="h-full kpi-card">
            <div class="flex items-start justify-between">
              <div>
                <div class="text-gray-500 text-sm mb-2">{{ card.title }}</div>
                <div class="text-2xl font-bold" :style="{ color: card.color }">
                  {{ card.value }}
                </div>
                <div v-if="card.sub" class="text-xs text-gray-400 mt-1">
                  {{ card.sub }}
                </div>
                <!-- 同比环比箭头 -->
                <div v-if="card.trend" class="text-xs mt-1" :style="{ color: card.trend.color }">
                  <IconifyIcon :icon="card.trend.icon" class="mr-1" />
                  同比 {{ card.trend.text }}
                </div>
              </div>
              <div
                class="flex items-center justify-center rounded-lg"
                :style="{ width: '48px', height: '48px', background: card.bg }"
              >
                <IconifyIcon
                  :icon="card.icon"
                  :style="{ color: card.color, fontSize: '24px' }"
                />
              </div>
            </div>
            <Progress
              v-if="card.progress !== undefined"
              :percent="card.progress"
              :stroke-color="card.color"
              :show-info="false"
              size="small"
              class="mt-3"
            />
          </Card>
        </Col>
      </Row>

      <!-- ============ 模块4：业绩预测卡片 ============ -->
      <div class="mb-4">
        <ForecastCard :loading="loading" :data="forecastData" />
      </div>

      <!-- ============ 模块4.5：销售计划进度（个人+团队） ============ -->
      <div v-if="hasPlanPermission" class="mb-4">
        <PlanProgressCard :year="selectedYear" />
      </div>

      <!-- ============ 模块5+6：月度趋势 + 完成率环形 ============ -->
      <Row :gutter="[16, 16]" class="mb-4">
        <Col :xs="24" :lg="16">
          <Card title="月度业绩趋势（目标 vs 实际 vs 预测）">
            <div v-if="monthlyData.length === 0" class="py-8">
              <Empty description="暂无数据" />
            </div>
            <div v-else class="flex items-end justify-around gap-2" style="height: 240px">
              <div
                v-for="m in monthlyData"
                :key="m.month"
                class="flex flex-col items-center gap-1"
                style="flex: 1"
              >
                <Tooltip :title="`实际: ${formatCurrency(m.contractActual || m.contract_actual || 0)}`">
                  <div
                    class="rounded-t transition-all duration-500 hover:opacity-80"
                    :style="{
                      width: '18px',
                      height: barHeight(m.contractActual || m.contract_actual || 0),
                      background: 'linear-gradient(180deg, #52c41a 0%, #95de64 100%)',
                    }"
                  />
                </Tooltip>
                <Tooltip :title="`目标: ${formatCurrency(m.contractTarget || m.contract_target || 0)}`">
                  <div
                    class="rounded-t transition-all duration-500 hover:opacity-80"
                    :style="{
                      width: '18px',
                      height: barHeight(m.contractTarget || m.contract_target || 0),
                      background: 'linear-gradient(180deg, #1890ff 0%, #69c0ff 100%)',
                    }"
                  />
                </Tooltip>
                <div class="text-xs text-gray-500 mt-1">{{ m.month }}月</div>
              </div>
            </div>
            <div class="flex justify-center gap-4 mt-3">
              <span class="flex items-center gap-1">
                <span class="w-3 h-3 rounded" style="background: #1890ff" /> 合同目标
              </span>
              <span class="flex items-center gap-1">
                <span class="w-3 h-3 rounded" style="background: #52c41a" /> 合同实际
              </span>
            </div>
          </Card>
        </Col>
        <Col :xs="24" :lg="8">
          <Card title="完成率分析">
            <div class="flex flex-col items-center gap-4 py-4">
              <Progress
                type="circle"
                :percent="Number(getRate(totalContractTarget, totalContractActual))"
                :stroke-color="'#52c41a'"
                :width="120"
              >
                <template #format="{ percent }">
                  <div>
                    <div class="text-xl font-bold text-green-600">{{ formatPercent(percent) }}%</div>
                    <div class="text-xs text-gray-400">合同完成率</div>
                  </div>
                </template>
              </Progress>
              <Progress
                type="circle"
                :percent="Number(getRate(totalPaymentTarget, totalPaymentActual))"
                :stroke-color="'#fa8c16'"
                :width="120"
              >
                <template #format="{ percent }">
                  <div>
                    <div class="text-xl font-bold text-orange-500">{{ formatPercent(percent) }}%</div>
                    <div class="text-xs text-gray-400">回款完成率</div>
                  </div>
                </template>
              </Progress>
            </div>
          </Card>
        </Col>
      </Row>

      <!-- ============ 模块7：销售漏斗 ============ -->
      <div class="mb-4">
        <SalesFunnel
          :year="selectedYear"
          :month="selectedMonth"
          :time-dimension="timeDimension"
        />
      </div>

      <!-- ============ 模块8：维度拆解 Tabs（部门排名/销售员排名/客户/产品/区域） ============ -->
      <Card class="mb-4">
        <Tabs v-model:active-key="breakdownTab">
          <Tabs.TabPane key="dept" tab="部门排名" />
          <Tabs.TabPane key="employee" tab="销售员排名" />
          <Tabs.TabPane key="customer" tab="客户维度" />
          <Tabs.TabPane key="product" tab="产品维度" />
          <Tabs.TabPane key="region" tab="区域维度" />
        </Tabs>

        <!-- 部门排名 -->
        <Table
          v-if="breakdownTab === 'dept'"
          :columns="rankingColumnsFor('dept')"
          :data-source="deptRanking"
          :pagination="{ pageSize: 10, showSizeChanger: true }"
          row-key="rank"
          size="middle"
          :scroll="{ x: 800 }"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.dataIndex === 'rank'">
              <div
                class="flex items-center justify-center w-7 h-7 rounded-full text-xs font-bold"
                :class="{
                  'bg-yellow-100 text-yellow-700': record.rank === 1,
                  'bg-gray-100 text-gray-600': record.rank === 2,
                  'bg-orange-100 text-orange-700': record.rank === 3,
                  'bg-gray-50 text-gray-400': record.rank > 3,
                }"
              >
                {{ record.rank }}
              </div>
            </template>
            <template v-else-if="column.dataIndex === 'contractAmount'">
              {{ formatCurrency(record.contractAmount) }}
            </template>
            <template v-else-if="column.dataIndex === 'contractTarget'">
              <span class="text-gray-500">{{ formatCurrency(record.contractTarget) }}</span>
            </template>
            <template v-else-if="column.dataIndex === 'completionRate'">
              <Progress
                :percent="Number(formatPercent(record.completionRate))"
                :stroke-color="
                  Number(record.completionRate) >= 100
                    ? '#52c41a'
                    : Number(record.completionRate) >= 60
                      ? '#faad14'
                      : '#ff4d4f'
                "
                size="small"
              />
            </template>
            <template v-else-if="column.dataIndex === 'paymentAmount'">
              {{ formatCurrency(record.paymentAmount) }}
            </template>
          </template>
        </Table>

        <!-- 销售员排名 -->
        <Table
          v-else-if="breakdownTab === 'employee'"
          :columns="rankingColumnsFor('employee')"
          :data-source="rankingData"
          :pagination="{ pageSize: 10, showSizeChanger: true }"
          row-key="rank"
          size="middle"
          :scroll="{ x: 800 }"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.dataIndex === 'rank'">
              <div
                class="flex items-center justify-center w-7 h-7 rounded-full text-xs font-bold"
                :class="{
                  'bg-yellow-100 text-yellow-700': record.rank === 1,
                  'bg-gray-100 text-gray-600': record.rank === 2,
                  'bg-orange-100 text-orange-700': record.rank === 3,
                  'bg-gray-50 text-gray-400': record.rank > 3,
                }"
              >
                {{ record.rank }}
              </div>
            </template>
            <template v-else-if="column.dataIndex === 'contractAmount'">
              {{ formatCurrency(record.contractAmount) }}
            </template>
            <template v-else-if="column.dataIndex === 'contractTarget'">
              <span class="text-gray-500">{{ formatCurrency(record.contractTarget) }}</span>
            </template>
            <template v-else-if="column.dataIndex === 'completionRate'">
              <Progress
                :percent="Number(formatPercent(record.completionRate))"
                :stroke-color="
                  Number(record.completionRate) >= 100
                    ? '#52c41a'
                    : Number(record.completionRate) >= 60
                      ? '#faad14'
                      : '#ff4d4f'
                "
                size="small"
              />
            </template>
            <template v-else-if="column.dataIndex === 'paymentAmount'">
              {{ formatCurrency(record.paymentAmount) }}
            </template>
          </template>
        </Table>

        <!-- 客户维度 -->
        <CustomerBreakdown
          v-else-if="breakdownTab === 'customer'"
          :year="selectedYear"
          :month="selectedMonth"
          :time-dimension="timeDimension"
        />

        <!-- 产品维度 -->
        <ProductBreakdown
          v-else-if="breakdownTab === 'product'"
          :year="selectedYear"
          :month="selectedMonth"
          :time-dimension="timeDimension"
        />

        <!-- 区域维度 -->
        <RegionBreakdown
          v-else-if="breakdownTab === 'region'"
          :year="selectedYear"
          :month="selectedMonth"
          :time-dimension="timeDimension"
        />
      </Card>

      <!-- ============ 模块9：行为指标 ============ -->
      <div class="mb-4 mt-4">
        <BehaviorMetrics
          :year="selectedYear"
          :month="selectedMonth"
          :time-dimension="timeDimension"
        />
      </div>

      <!-- ============ 模块10：个人业绩卡（仅普通销售显示） ============ -->
      <Card v-if="isPersonalView" title="我的业绩" class="mb-4">
        <Row :gutter="[16, 16]">
          <Col v-for="card in personalCards" :key="card.title" :xs="12" :md="6">
            <div class="p-4 rounded-lg bg-gray-50 text-center">
              <IconifyIcon
                :icon="card.icon"
                class="text-2xl mb-2"
                :style="{ color: card.color }"
              />
              <div class="text-gray-500 text-sm">{{ card.title }}</div>
              <div class="text-xl font-bold mt-1" :style="{ color: card.color }">
                {{ card.value }}
              </div>
              <div v-if="card.sub" class="text-xs text-gray-400 mt-1">
                {{ card.sub }}
              </div>
            </div>
          </Col>
        </Row>
      </Card>

      <!-- ============ 模块6扩展：里程碑激励（普通销售显示） ============ -->
      <div v-if="isPersonalView" class="mb-4">
        <MilestoneCard
          :year="selectedYear"
          :current-amount="totalContractActual"
        />
      </div>

      <!-- ============ 模块11：个人成长档案（仅普通销售显示） ============ -->
      <div v-if="isPersonalView" class="mb-4">
        <PersonalGrowth />
      </div>

      <!-- 个人销售计划设置抽屉 -->
      <PlanSettingDrawer
        v-model:visible="planDrawerVisible"
        :year="selectedYear"
        @success="loadData"
      />

      <!-- 待我审批抽屉 -->
      <PendingApprovalList
        v-model:visible="pendingApprovalVisible"
        :year="selectedYear"
        @refresh="handlePendingRefresh"
      />
    </Spin>
  </Page>
</template>

<style scoped>
.kpi-card {
  transition: all 0.3s ease;
}

.kpi-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgb(0 0 0 / 10%);
}
</style>
