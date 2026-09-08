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
  message,
  Progress,
  Row,
  Segmented,
  Select,
  Spin,
  Table,
  Tabs,
  Tag,
  Tooltip,
  TreeSelect,
} from 'ant-design-vue';
import { useRouter } from 'vue-router';

import {
  exportPerformanceApi,
  getEmployeeConversionApi,
  getEmployeeCustomerCountApi,
  getEmployeeFollowUpApi,
  getMonthlyPerformanceApi,
  getPerformanceComparisonApi,
  getPerformanceConfigApi,
  getPerformanceForecastApi,
  getPerformanceRankingApi,
  getPlanListApi,
} from '#/api/core/statistics';
import { getDeptTreeApi } from '#/api/core/system/dept';
import { $t } from '#/locales';

import BehaviorMetrics from './components/BehaviorMetrics.vue';
import CustomerBreakdown from './components/CustomerBreakdown.vue';
import ForecastCard from './components/ForecastCard.vue';
import MilestoneCard from './components/MilestoneCard.vue';
import PendingApprovalList from './components/PendingApprovalList.vue';
import PersonalGrowth from './components/PersonalGrowth.vue';
import PlanProgressCard from './components/PlanProgressCard.vue';
import ProductBreakdown from './components/ProductBreakdown.vue';
import ProgressAlert from './components/ProgressAlert.vue';
import RegionBreakdown from './components/RegionBreakdown.vue';
import SalesFunnel from './components/SalesFunnel.vue';
import TeamPlanList from './components/TeamPlanList.vue';
import PlanSettingDrawer from './PlanSettingDrawer.vue';

defineOptions({ name: 'PerformanceOverview' });

const userStore = useUserStore();
const router = useRouter();
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
  hasAccessByCodes(['statistics:performance-plan:audit']),
);

// 团队计划页签：manage 或 audit 权限可见，普通员工不可见（方案 §4.6.1）
const canViewTeam = computed(
  () =>
    hasPlanPermission.value &&
    (hasPlanManagePermission.value || hasPlanApprovePermission.value),
);

// 员工对比页签：跨员工业绩数据属管理视角，未授权角色（如业务员）不可见；
// 权限码在角色管理/权限集中分配，无需改代码即可调整可见范围
const canViewEmployeeComparison = computed(() =>
  hasAccessByCodes(['statistics:performance:employee-compare']),
);
const activeTab = ref<'personal' | 'team'>('personal');

// ===== 时间维度 =====
type TimeDimension = 'day' | 'month' | 'year';
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
  const roleCodes = new Set(
    roles.map((r: any) => r.code || r.roleCode || '').filter(Boolean),
  );
  const isBoss =
    roleCodes.has('super_admin') ||
    roleCodes.has('system_admin') ||
    roleCodes.has('boss') ||
    roleCodes.has('gm');
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

// WP1：部门筛选（联动月度趋势与维度拆解榜单）
const selectedDeptId = ref<number>();
const deptTreeData = ref<any[]>([]);

// WP4：未分配负责人金额（数据质量护栏）
const unassignedContractAmount = ref(0);
const unassignedPaymentAmount = ref(0);

// WP3：业绩统计配置（榜单口径，拉取失败时保持默认）
const performanceConfig = ref({
  statScope: 'all_users',
  showZeroTargetRows: false,
  hideOtherActual: false,
  hideOtherTarget: false,
  rankingTopN: 10,
});

const queryParams = computed(() => ({
  year: selectedYear.value,
  month: timeDimension.value === 'year' ? undefined : selectedMonth.value,
  time_dimension: timeDimension.value,
  department_id: selectedDeptId.value,
}));

async function loadDeptTree() {
  try {
    const res: any = await getDeptTreeApi();
    deptTreeData.value = res?.data || res || [];
  } catch {
    deptTreeData.value = [];
  }
}

async function loadPerformanceConfig() {
  try {
    const res: any = await getPerformanceConfigApi();
    const cfg = res?.data || res || {};
    const topN = Number(cfg.rankingTopN ?? cfg.ranking_top_n);
    performanceConfig.value = {
      statScope: cfg.statScope ?? cfg.stat_scope ?? 'all_users',
      showZeroTargetRows: Boolean(
        cfg.showZeroTargetRows ?? cfg.show_zero_target_rows,
      ),
      hideOtherActual: Boolean(cfg.hideOtherActual ?? cfg.hide_other_actual),
      hideOtherTarget: Boolean(cfg.hideOtherTarget ?? cfg.hide_other_target),
      rankingTopN: topN > 0 ? topN : 10,
    };
  } catch {
    // 保持默认口径，页面可用性优先
  }
}

// ===== 排行榜排序维度（方案 §4.5.4：支持完成率排序） =====
const rankingOrderBy = ref<
  | 'contract_amount'
  | 'contract_completion_rate'
  | 'payment_amount'
  | 'payment_completion_rate'
>('contract_amount');
const rankingOrderByOptions = [
  { label: '按合同额', value: 'contract_amount' },
  { label: '按回款额', value: 'payment_amount' },
  { label: '按合同完成率', value: 'contract_completion_rate' },
  { label: '按回款完成率', value: 'payment_completion_rate' },
];

watch(rankingOrderBy, () => {
  loadRanking();
});

// 排行榜独立加载（方案 §4.5.4：order_by 支持完成率排序，切换时仅重拉排行榜）
// 内部自带失败兜底，永不 reject，可安全地浮空调用
async function loadRanking() {
  try {
    const rankingRes = await getPerformanceRankingApi({
      ...queryParams.value,
      order_by: rankingOrderBy.value,
    });
    // WP4：后端返回 { list, unassigned_contract_amount, unassigned_payment_amount }
    const payload: any = rankingRes?.data ?? rankingRes ?? {};
    const rankingList: any[] = Array.isArray(payload)
      ? payload
      : (payload.list ?? []);
    unassignedContractAmount.value =
      Number(
        payload?.unassigned_contract_amount ??
          payload?.unassignedContractAmount ??
          0,
      ) || 0;
    unassignedPaymentAmount.value =
      Number(
        payload?.unassigned_payment_amount ??
          payload?.unassignedPaymentAmount ??
          0,
      ) || 0;
    // WP3：后端对打码行金额置 null（未打码行恒为数值），此处保留 null 语义
    rankingData.value =
      rankingList.map((item: any) => ({
        rank: item.rank,
        employeeId: item.employeeId ?? item.employee_id,
        employeeName: item.employeeName || item.employee_name,
        departmentName: item.departmentName || item.department_name,
        contractAmount: item.contractAmount ?? item.contract_amount ?? null,
        contractTarget: item.contractTarget ?? item.contract_target ?? null,
        paymentAmount: item.paymentAmount ?? item.payment_amount ?? null,
        paymentTarget: item.paymentTarget ?? item.payment_target ?? null,
        completionRate:
          item.contractCompletionRate || item.contract_completion_rate || 0,
        monthOnMonth: item.monthOnMonth || item.month_on_month || 0,
        // WP2：目标缺失引导标记
        hasApprovedPlan: item.hasApprovedPlan ?? item.has_approved_plan,
      })) || [];

    // 个人视图额外加载自己的数据（取归一化后的行，避免字段命名差异）
    if (isPersonalView.value) {
      personalData.value =
        rankingData.value.find(
          (r: any) =>
            String(r.employeeId) === String(userStore.userInfo?.userId),
        ) || {};
    }
  } catch (error) {
    console.error('加载排行榜数据失败', error);
    rankingData.value = [];
    if (isPersonalView.value) {
      personalData.value = {};
    }
  }
}

async function loadData() {
  loading.value = true;
  try {
    const params: any = { ...queryParams.value };
    // 排行榜内部自带失败兜底；不并入 Promise.all 元组，避免 TS 对 .catch 链的元组推断污染
    const rankingPromise = loadRanking();
    const [monthlyRes, comparisonRes, forecastRes] = await Promise.all([
      getMonthlyPerformanceApi(params),
      getPerformanceComparisonApi(params).catch(() => ({})),
      getPerformanceForecastApi(params).catch(() => ({})),
    ]);
    await rankingPromise;

    monthlyData.value = monthlyRes?.data?.months || monthlyRes?.months || [];
    comparisonData.value = comparisonRes?.data || comparisonRes || {};
    forecastData.value = forecastRes?.data || forecastRes || {};
  } catch (error) {
    console.error('加载业绩数据失败', error);
    monthlyData.value = [];
    rankingData.value = [];
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadData();
  loadDeptTree();
  loadPerformanceConfig();
});

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
  if (num >= 100_000_000) return `¥${(num / 100_000_000).toFixed(2)}亿`;
  if (num >= 10_000) return `¥${(num / 10_000).toFixed(1)}万`;
  return `¥${num.toLocaleString()}`;
}

// ===== WP3：榜单口径（隐私打码 + TopN 截断） =====
const rankingTopN = computed(
  () => Number(performanceConfig.value.rankingTopN) || 0,
);
const rankingDisplayData = computed(() =>
  rankingTopN.value > 0
    ? rankingData.value.slice(0, rankingTopN.value)
    : rankingData.value,
);

// 金额打码仅对个人视角的他人行生效，与后端口径一致
function isMaskedRow(record: any, kind: 'actual' | 'target') {
  if (!isPersonalView.value) return false;
  if (
    record.employeeId &&
    String(record.employeeId) === String(userStore.userInfo?.userId)
  ) {
    return false;
  }
  // 后端打码行金额恒为 null（未打码行恒为数值），兜底配置拉取失败的场景
  const nullMasked =
    kind === 'actual' ? record.contractAmount == null : record.contractTarget == null;
  return (
    nullMasked ||
    (kind === 'actual'
      ? performanceConfig.value.hideOtherActual
      : performanceConfig.value.hideOtherTarget)
  );
}

function formatRankAmount(
  record: any,
  field: string,
  kind: 'actual' | 'target',
) {
  return isMaskedRow(record, kind)
    ? $t('page.statistics.masked')
    : formatCurrency(record[field]);
}

// ===== WP4：未分配负责人警示（跳转合同列表处理） =====
const showUnassignedBadge = computed(
  () =>
    unassignedContractAmount.value > 0 || unassignedPaymentAmount.value > 0,
);

function goUnassignedContracts() {
  router.push('/sale/contract');
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
  | 'customer'
  | 'dept'
  | 'employee'
  | 'employee-comparison'
  | 'product'
  | 'region'
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
  const arr = [...deptMap.values()];
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

// ===== 员工对比 Tab =====
const employeeComparisonLoading = ref(false);
const employeeComparisonData = ref<any[]>([]);
const employeeComparisonLoaded = ref(false);

const employeeComparisonColumns = [
  {
    title: '员工',
    dataIndex: 'employeeName',
    width: 100,
    fixed: 'left' as const,
  },
  { title: '部门', dataIndex: 'departmentName', width: 100 },
  {
    title: '客户总数',
    dataIndex: 'totalCustomers',
    align: 'right' as const,
    width: 90,
  },
  {
    title: '合同客户',
    dataIndex: 'contractCustomers',
    align: 'right' as const,
    width: 90,
  },
  {
    title: '转化率',
    dataIndex: 'customerConversionRate',
    align: 'right' as const,
    width: 90,
  },
  {
    title: '跟进次数',
    dataIndex: 'totalFollowUp',
    align: 'right' as const,
    width: 90,
  },
  {
    title: '商机跟进',
    dataIndex: 'opportunityFollowUp',
    align: 'right' as const,
    width: 90,
  },
  {
    title: '商机数',
    dataIndex: 'totalOpportunities',
    align: 'right' as const,
    width: 80,
  },
  {
    title: '成交商机',
    dataIndex: 'wonOpportunities',
    align: 'right' as const,
    width: 90,
  },
  {
    title: '胜率',
    dataIndex: 'opportunityWinRate',
    align: 'right' as const,
    width: 80,
  },
  {
    title: '合同数',
    dataIndex: 'totalContracts',
    align: 'right' as const,
    width: 80,
  },
  {
    title: '合同金额',
    dataIndex: 'contractAmount',
    align: 'right' as const,
    width: 120,
  },
  {
    title: '客单价',
    dataIndex: 'avgContractAmount',
    align: 'right' as const,
    width: 110,
  },
];

async function loadEmployeeComparison() {
  if (employeeComparisonLoaded.value) return;
  employeeComparisonLoading.value = true;
  try {
    const params = {
      year: selectedYear.value,
      month: timeDimension.value === 'year' ? undefined : selectedMonth.value,
    };
    const [customerRes, followUpRes, conversionRes] = await Promise.all([
      getEmployeeCustomerCountApi(),
      getEmployeeFollowUpApi(params),
      getEmployeeConversionApi(params),
    ]);

    const customerList: any[] = customerRes?.data || customerRes || [];
    const followUpList: any[] = followUpRes?.data || followUpRes || [];
    const conversionList: any[] = conversionRes?.data || conversionRes || [];

    const mergeMap = new Map<string, any>();
    const getKey = (item: any) =>
      String(item.employeeId ?? item.employee_id ?? '');

    customerList.forEach((item: any) => {
      const key = getKey(item);
      mergeMap.set(key, {
        employeeName: item.employeeName ?? item.employee_name ?? '-',
        departmentName: item.departmentName ?? item.department_name ?? '-',
        totalCustomers: Number(
          item.totalCustomers ?? item.total_customers ?? 0,
        ),
        contractCustomers: Number(
          item.contractCustomers ?? item.contract_customers ?? 0,
        ),
        customerConversionRate: Number(
          item.customerConversionRate ?? item.customer_conversion_rate ?? 0,
        ),
      });
    });

    followUpList.forEach((item: any) => {
      const key = getKey(item);
      const row =
        mergeMap.get(key) ||
        (mergeMap.set(key, {
          employeeName: item.employeeName ?? item.employee_name ?? '-',
          departmentName: item.departmentName ?? item.department_name ?? '-',
        }),
        mergeMap.get(key));
      row.totalFollowUp = Number(
        item.totalFollowUp ?? item.total_follow_up ?? 0,
      );
      row.opportunityFollowUp = Number(
        item.opportunityFollowUp ?? item.opportunity_follow_up ?? 0,
      );
    });

    conversionList.forEach((item: any) => {
      const key = getKey(item);
      const row =
        mergeMap.get(key) ||
        (mergeMap.set(key, {
          employeeName: item.employeeName ?? item.employee_name ?? '-',
          departmentName: item.departmentName ?? item.department_name ?? '-',
        }),
        mergeMap.get(key));
      row.totalOpportunities = Number(
        item.totalOpportunities ?? item.total_opportunities ?? 0,
      );
      row.wonOpportunities = Number(
        item.wonOpportunities ?? item.won_opportunities ?? 0,
      );
      row.opportunityWinRate = Number(
        item.opportunityWinRate ?? item.opportunity_win_rate ?? 0,
      );
      row.totalContracts = Number(
        item.totalContracts ?? item.total_contracts ?? 0,
      );
      row.contractAmount = Number(
        item.contractAmount ?? item.contract_amount ?? 0,
      );
      row.avgContractAmount = Number(
        item.avgContractAmount ?? item.avg_contract_amount ?? 0,
      );
    });

    employeeComparisonData.value = [...mergeMap.values()].toSorted(
      (a, b) => (b.contractAmount || 0) - (a.contractAmount || 0),
    );
    employeeComparisonLoaded.value = true;
  } catch (error) {
    console.error('加载员工对比数据失败', error);
    employeeComparisonData.value = [];
  } finally {
    employeeComparisonLoading.value = false;
  }
}

watch(breakdownTab, (val) => {
  if (val === 'employee-comparison') {
    loadEmployeeComparison();
  }
});

// 筛选条件变化时重置员工对比缓存，下次切换 Tab 重新加载
watch(
  () => [
    selectedYear.value,
    selectedMonth.value,
    timeDimension.value,
    selectedDeptId.value,
  ],
  () => {
    employeeComparisonLoaded.value = false;
  },
);

// WP1：部门切换立即联动月度趋势与榜单（comparison/forecast 后端不支持部门过滤，自动忽略该参数）
watch(selectedDeptId, () => {
  loadData();
});

// ===== 个人销售计划抽屉 =====
const planDrawerVisible = ref(false);
const planStatus = ref<'approved' | 'draft' | 'none' | 'pending' | 'rejected'>(
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
    const res: any = await getPlanListApi({
      year: selectedYear.value,
      employeeId,
    });
    // requestClient 已配置 responseReturn: 'data'，res 为分页对象 { total, page, pageSize, items }
    const plans = res?.items || [];
    if (plans.length === 0) {
      planStatus.value = 'none';
    } else {
      const statusNum = Number(plans[0].status);
      planStatus.value =
        (['draft', 'pending', 'approved', 'rejected'] as const)[statusNum] ||
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
    const plans = res?.items || [];
    pendingApprovalCount.value = Number(res?.total ?? plans.length) || 0;
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
    case 'approved': {
      // 审批通过后隐藏入口（用户选择"隐藏入口仅留查看"）
      return {
        text: $t('page.statistics.viewPlan'),
        color: '#52c41a',
        icon: 'lucide:eye',
        show: true,
      };
    }
    case 'draft': {
      return {
        text: $t('page.statistics.editPlanDraft'),
        color: '#faad14',
        icon: 'lucide:edit',
        show: true,
      };
    }
    case 'none': {
      return {
        text: $t('page.statistics.setSalesPlan'),
        color: '#ff4d4f',
        icon: 'lucide:alert-circle',
        show: true,
      };
    }
    case 'pending': {
      return {
        text: $t('page.statistics.viewPlanPending'),
        color: '#1890ff',
        icon: 'lucide:clock',
        show: true,
      };
    }
    case 'rejected': {
      return {
        text: $t('page.statistics.resubmitPlan'),
        color: '#ff4d4f',
        icon: 'lucide:rotate-ccw',
        show: true,
      };
    }
    default: {
      return {
        text: $t('page.statistics.setSalesPlan'),
        color: '#1890ff',
        icon: 'lucide:target',
        show: true,
      };
    }
  }
});

// 审批通过后顶部按钮隐藏（仅保留在计划进度卡片中显示状态）
const showPlanButton = computed(
  () => planButtonConfig.value.show && planStatus.value !== 'approved',
);

// 处理待审批抽屉刷新
function handlePendingRefresh() {
  loadPendingCount();
  checkPlanStatus();
}

// ===== WP2：目标缺失引导（个人视角无生效年度计划时） =====
const personalHasApprovedPlan = computed(() => {
  const v = personalData.value?.hasApprovedPlan;
  if (v !== undefined && v !== null) return v === true;
  // 无业绩数据（不在榜单）时兜底用计划状态判断
  return planStatus.value === 'approved';
});
const showPlanGuideCard = computed(
  () => isPersonalView.value && !personalHasApprovedPlan.value,
);
const planGuideText = computed(() => {
  switch (planStatus.value) {
    case 'draft': {
      return $t('page.statistics.planGuideDraft');
    }
    case 'pending': {
      return $t('page.statistics.planGuidePending');
    }
    case 'rejected': {
      return $t('page.statistics.planGuideRejected');
    }
    default: {
      return $t('page.statistics.planGuideNone');
    }
  }
});
const planGuideActionText = computed(() =>
  planStatus.value === 'draft'
    ? $t('page.statistics.planGuideEdit')
    : $t('page.statistics.planGuideAction'),
);
// 完成率 UI 仅在有生效目标时展示，避免无目标用户看到 0%
const hideCompletionHint = computed(
  () => isPersonalView.value && !personalHasApprovedPlan.value,
);

// ===== 同比环比箭头 =====
function trendArrow(yoy?: number, mom?: number) {
  if (yoy === undefined && mom === undefined) return null;
  const value = yoy ?? mom ?? 0;
  if (value > 0)
    return {
      color: '#52c41a',
      icon: 'lucide:trending-up',
      text: `↑${formatPercent(value)}%`,
    };
  if (value < 0)
    return {
      color: '#ff4d4f',
      icon: 'lucide:trending-down',
      text: `↓${formatPercent(Math.abs(value))}%`,
    };
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
      sub: hideCompletionHint.value
        ? undefined
        : `完成 ${getRate(totalContractTarget.value, totalContractActual.value)}%`,
      progress: hideCompletionHint.value
        ? undefined
        : Number(
            getRate(totalContractTarget.value, totalContractActual.value),
          ),
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
      sub: hideCompletionHint.value
        ? undefined
        : `完成 ${getRate(totalPaymentTarget.value, totalPaymentActual.value)}%`,
      progress: hideCompletionHint.value
        ? undefined
        : Number(
            getRate(totalPaymentTarget.value, totalPaymentActual.value),
          ),
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
    sub: hideCompletionHint.value
      ? undefined
      : `完成 ${formatPercent(personalData.value?.completionRate || 0)}%`,
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
  } catch (error: any) {
    message.error(error?.message || '导出失败');
  } finally {
    exporting.value = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <Spin :spinning="loading">
      <Tabs v-model:active-key="activeTab" size="large">
        <Tabs.TabPane key="personal" tab="个人视图">
      <!-- ============ 模块1：顶部工具栏 ============ -->
      <Card class="mb-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <IconifyIcon
              icon="lucide:bar-chart-3"
              class="text-xl text-primary"
            />
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
            <Button
              size="small"
              :loading="exporting"
              @click="handleExport('excel')"
            >
              <template #icon>
                <IconifyIcon icon="lucide:file-spreadsheet" />
              </template>
              导出 Excel
            </Button>
            <Button
              size="small"
              :loading="exporting"
              @click="handleExport('pdf')"
            >
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
                {{ $t('page.statistics.pendingApproval') }}
              </Button>
            </Badge>

            <!-- 个人计划设置/查看按钮（审批通过后隐藏入口） -->
            <Button
              v-if="hasPlanManagePermission && showPlanButton"
              :type="
                planStatus === 'none' && isPersonalView ? 'primary' : 'default'
              "
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

      <!-- ============ 模块2.5：目标缺失引导卡（WP2） ============ -->
      <Card v-if="showPlanGuideCard" class="mb-4">
        <div class="flex flex-wrap items-center gap-3">
          <div
            class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-orange-100"
          >
            <IconifyIcon icon="lucide:target" class="text-xl text-orange-500" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="font-medium">
              {{ $t('page.statistics.planGuideTitle') }}
            </div>
            <div class="mt-1 text-sm text-gray-500">
              {{ planGuideText }}
            </div>
          </div>
          <Button
            v-if="hasPlanManagePermission"
            type="primary"
            size="small"
            @click="openPlanDrawer"
          >
            {{ planGuideActionText }}
          </Button>
        </div>
      </Card>

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
                <div
                  v-if="card.trend"
                  class="text-xs mt-1"
                  :style="{ color: card.trend.color }"
                >
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
        <PlanProgressCard
          :year="selectedYear"
          :plan-status="planStatus"
          @view-team="activeTab = 'team'"
        />
      </div>

      <!-- ============ 模块5+6：月度趋势 + 完成率环形 ============ -->
      <Row :gutter="[16, 16]" class="mb-4">
        <Col :xs="24" :lg="hideCompletionHint ? 24 : 16">
          <Card title="月度业绩趋势（目标 vs 实际 vs 预测）">
            <template #extra>
              <TreeSelect
                v-if="!isPersonalView"
                v-model:value="selectedDeptId"
                :allow-clear="true"
                :dropdown-style="{ maxHeight: '320px', overflow: 'auto' }"
                :field-names="{
                  children: 'children',
                  label: 'label',
                  value: 'value',
                }"
                :placeholder="$t('page.statistics.deptAll')"
                :tree-data="deptTreeData"
                size="small"
                style="min-width: 140px"
                tree-default-expand-all
                tree-node-filter-prop="label"
              />
            </template>
            <div v-if="monthlyData.length === 0" class="py-8">
              <Empty description="暂无数据" />
            </div>
            <div
              v-else
              class="flex items-end justify-around gap-2"
              style="height: 240px"
            >
              <div
                v-for="m in monthlyData"
                :key="m.month"
                class="flex flex-col items-center gap-1"
                style="flex: 1"
              >
                <Tooltip
                  :title="`实际: ${formatCurrency(m.contractActual || m.contract_actual || 0)}`"
                >
                  <div
                    class="rounded-t transition-all duration-500 hover:opacity-80"
                    :style="{
                      width: '18px',
                      height: barHeight(
                        m.contractActual || m.contract_actual || 0,
                      ),
                      background:
                        'linear-gradient(180deg, #52c41a 0%, #95de64 100%)',
                    }"
                  ></div>
                </Tooltip>
                <Tooltip
                  :title="`目标: ${formatCurrency(m.contractTarget || m.contract_target || 0)}`"
                >
                  <div
                    class="rounded-t transition-all duration-500 hover:opacity-80"
                    :style="{
                      width: '18px',
                      height: barHeight(
                        m.contractTarget || m.contract_target || 0,
                      ),
                      background:
                        'linear-gradient(180deg, #1890ff 0%, #69c0ff 100%)',
                    }"
                  ></div>
                </Tooltip>
                <div class="text-xs text-gray-500 mt-1">{{ m.month }}月</div>
              </div>
            </div>
            <div class="flex justify-center gap-4 mt-3">
              <span class="flex items-center gap-1">
                <span
                  class="w-3 h-3 rounded"
                  style="background: #1890ff"
                ></span>
                合同目标
              </span>
              <span class="flex items-center gap-1">
                <span
                  class="w-3 h-3 rounded"
                  style="background: #52c41a"
                ></span>
                合同实际
              </span>
            </div>
          </Card>
        </Col>
        <Col v-if="!hideCompletionHint" :xs="24" :lg="8">
          <Card title="完成率分析">
            <div class="flex flex-col items-center gap-4 py-4">
              <Progress
                type="circle"
                :percent="
                  Number(getRate(totalContractTarget, totalContractActual))
                "
                stroke-color="#52c41a"
                :width="120"
              >
                <template #format="{ percent }">
                  <div>
                    <div class="text-xl font-bold text-green-600">
                      {{ formatPercent(percent) }}%
                    </div>
                    <div class="text-xs text-gray-400">合同完成率</div>
                  </div>
                </template>
              </Progress>
              <Progress
                type="circle"
                :percent="
                  Number(getRate(totalPaymentTarget, totalPaymentActual))
                "
                stroke-color="#fa8c16"
                :width="120"
              >
                <template #format="{ percent }">
                  <div>
                    <div class="text-xl font-bold text-orange-500">
                      {{ formatPercent(percent) }}%
                    </div>
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
        <template #extra>
          <TreeSelect
            v-if="!isPersonalView"
            v-model:value="selectedDeptId"
            :allow-clear="true"
            :dropdown-style="{ maxHeight: '320px', overflow: 'auto' }"
            :field-names="{
              children: 'children',
              label: 'label',
              value: 'value',
            }"
            :placeholder="$t('page.statistics.deptAll')"
            :tree-data="deptTreeData"
            size="small"
            style="min-width: 140px"
            tree-default-expand-all
            tree-node-filter-prop="label"
          />
        </template>
        <Tabs v-model:active-key="breakdownTab">
          <Tabs.TabPane key="dept">
            <template #tab>
              <span>{{ $t('page.statistics.deptRanking') }}</span>
              <!-- WP4：未分配负责人金额警示，点击跳转合同列表处理 -->
              <Tooltip
                v-if="showUnassignedBadge && !isPersonalView"
                :title="$t('page.statistics.unassignedBadgeTitle')"
              >
                <Tag
                  color="warning"
                  class="ml-2 cursor-pointer"
                  @click.stop="goUnassignedContracts"
                >
                  <IconifyIcon icon="lucide:alert-triangle" class="mr-1" />
                  <template v-if="unassignedContractAmount > 0">
                    {{
                      $t('page.statistics.unassignedContractWarning', {
                        amount: formatCurrency(unassignedContractAmount),
                      })
                    }}
                  </template>
                  <template v-else>
                    {{
                      $t('page.statistics.unassignedPaymentWarning', {
                        amount: formatCurrency(unassignedPaymentAmount),
                      })
                    }}
                  </template>
                </Tag>
              </Tooltip>
            </template>
          </Tabs.TabPane>
          <Tabs.TabPane key="employee" tab="销售员排名" />
          <Tabs.TabPane key="customer" tab="客户维度" />
          <Tabs.TabPane key="product" tab="产品维度" />
          <Tabs.TabPane key="region" tab="区域维度" />
          <Tabs.TabPane
            v-if="canViewEmployeeComparison"
            key="employee-comparison"
            tab="员工对比"
          />
        </Tabs>

        <!-- 排序维度（方案 §4.5.4：排行榜支持完成率排序） -->
        <div
          v-if="breakdownTab === 'dept' || breakdownTab === 'employee'"
          class="mb-2 text-right"
        >
          <Select
            v-model:value="rankingOrderBy"
            :options="rankingOrderByOptions"
            size="small"
            style="width: 170px"
          />
        </div>

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
              <span class="text-gray-500">{{
                formatCurrency(record.contractTarget)
              }}</span>
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

        <!-- 销售员排名（WP3：按 rankingTopN 截断展示，金额按配置打码） -->
        <Table
          v-else-if="breakdownTab === 'employee'"
          :columns="rankingColumnsFor('employee')"
          :data-source="rankingDisplayData"
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
              <Tooltip
                :title="
                  isMaskedRow(record, 'actual')
                    ? $t('page.statistics.maskedTip')
                    : ''
                "
              >
                {{ formatRankAmount(record, 'contractAmount', 'actual') }}
              </Tooltip>
            </template>
            <template v-else-if="column.dataIndex === 'contractTarget'">
              <Tooltip
                :title="
                  isMaskedRow(record, 'target')
                    ? $t('page.statistics.maskedTip')
                    : ''
                "
              >
                <span class="text-gray-500">{{
                  formatRankAmount(record, 'contractTarget', 'target')
                }}</span>
              </Tooltip>
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
              <Tooltip
                :title="
                  isMaskedRow(record, 'actual')
                    ? $t('page.statistics.maskedTip')
                    : ''
                "
              >
                {{ formatRankAmount(record, 'paymentAmount', 'actual') }}
              </Tooltip>
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

        <!-- 员工对比 -->
        <Spin
          v-else-if="breakdownTab === 'employee-comparison'"
          :spinning="employeeComparisonLoading"
        >
          <Table
            :columns="employeeComparisonColumns"
            :data-source="employeeComparisonData"
            :pagination="{ pageSize: 10, showSizeChanger: true }"
            row-key="employeeName"
            size="middle"
            :scroll="{ x: 1300 }"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.dataIndex === 'customerConversionRate'">
                {{ formatPercent(record.customerConversionRate) }}%
              </template>
              <template v-else-if="column.dataIndex === 'opportunityWinRate'">
                {{ formatPercent(record.opportunityWinRate) }}%
              </template>
              <template v-else-if="column.dataIndex === 'contractAmount'">
                {{ formatCurrency(record.contractAmount) }}
              </template>
              <template v-else-if="column.dataIndex === 'avgContractAmount'">
                {{ formatCurrency(record.avgContractAmount) }}
              </template>
            </template>
          </Table>
        </Spin>
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
              <div
                class="text-xl font-bold mt-1"
                :style="{ color: card.color }"
              >
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
        </Tabs.TabPane>
        <Tabs.TabPane v-if="canViewTeam" key="team" :tab="$t('page.statistics.performancePlan.teamPlanTab')">
          <TeamPlanList :year="selectedYear" />
        </Tabs.TabPane>
      </Tabs>

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
  box-shadow: 0 4px 12px rgb(0 0 0 / 10%);
  transform: translateY(-2px);
}
</style>
