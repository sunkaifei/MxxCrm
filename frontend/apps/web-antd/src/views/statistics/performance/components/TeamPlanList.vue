<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, ref, watch } from 'vue';

import { useAccess } from '@vben/access';
import { useUserStore } from '@vben/stores';

import {
  Alert,
  Button,
  Card,
  Input,
  message,
  Modal,
  Radio,
  RadioGroup,
  Select,
  Tag,
  Textarea,
  TreeSelect,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  approvePlanApi,
  getPlanCoverageApi,
  getPlanListApi,
  rejectPlanApi,
  remindPlanApi,
} from '#/api/core/statistics/performance-plan';
import { getDeptListApi } from '#/api/core/system/dept';
import { $t } from '#/locales';

import PlanSettingDrawer from '../PlanSettingDrawer.vue';
import PlanMonthlyCompare from './PlanMonthlyCompare.vue';

defineOptions({ name: 'TeamPlanList' });

const props = defineProps<{
  year: number;
}>();

const { hasAccessByCodes } = useAccess();
// 代建/代改走管理动作；审批走审批动作
const hasManage = computed(() => hasAccessByCodes(['statistics:performance-plan:manage']));
const hasAudit = computed(() => hasAccessByCodes(['statistics:performance-plan:audit']));
const userStore = useUserStore();
const currentUserId = computed(() =>
  Number(userStore.userInfo?.userId ?? userStore.userInfo?.id ?? 0),
);

// ===== 筛选状态 =====
const filterYear = ref(props.year);
const filterDeptId = ref<number>();
const filterStatusList = ref<number[]>([]);
const filterKeyword = ref('');
const yearOptions = computed(() => {
  const current = new Date().getFullYear();
  return Array.from({ length: 5 }, (_, i) => ({
    value: current - i,
    label: $t('page.statistics.performancePlan.yearOption', { n: current - i }),
  }));
});

watch(
  () => props.year,
  (v) => {
    if (v && v !== filterYear.value) {
      filterYear.value = v;
      loadCoverage();
      gridApi.query();
    }
  },
);

// ===== 覆盖度提示条 =====
interface CoverageItem {
  employeeId: number;
  name?: null | string;
  deptName?: null | string;
  hasPlan: boolean;
  approved: boolean;
  planStatus?: null | number;
}
interface CoverageSummary {
  year: number;
  totalEmployees: number;
  withPlanCount: number;
  approvedCount: number;
  coverageRate?: null | string;
  items: CoverageItem[];
}
const coverage = ref<CoverageSummary>();
const coverageExpanded = ref(false);
const uncoveredList = computed(() =>
  (coverage.value?.items ?? []).filter((i) => !i.hasPlan),
);
const loadCoverage = async () => {
  try {
    coverage.value = await getPlanCoverageApi({ year: filterYear.value });
  } catch {
    coverage.value = undefined;
  }
};

// ===== 部门树 =====
interface DeptNode {
  id: number;
  deptName?: null | string;
  parentId?: null | number;
}
interface TreeNode {
  value: number;
  title: string;
  children: TreeNode[];
}
const deptTree = ref<TreeNode[]>([]);
const buildDeptTree = (list: DeptNode[]): TreeNode[] => {
  const map = new Map<number, TreeNode>();
  list.forEach((d) => {
    map.set(
      d.id,
      {
        value: d.id,
        title:
          d.deptName ?? $t('page.statistics.performancePlan.deptFallback', { n: d.id }),
        children: [],
      },
    );
  });
  const roots: TreeNode[] = [];
  list.forEach((d) => {
    const node = map.get(d.id)!;
    const parent = d.parentId ? map.get(d.parentId) : undefined;
    if (parent && parent.value !== d.id) {
      parent.children.push(node);
    } else {
      roots.push(node);
    }
  });
  return roots;
};
const loadDepts = async () => {
  try {
    const list: DeptNode[] = await getDeptListApi();
    deptTree.value = buildDeptTree(Array.isArray(list) ? list : []);
  } catch {
    deptTree.value = [];
  }
};
loadDepts();

// ===== 状态映射 =====
const statusConfig: Record<number, { color: string; text: string }> = {
  0: { color: 'orange', text: $t('page.statistics.performancePlan.draft') },
  1: { color: 'blue', text: $t('page.statistics.performancePlan.pending') },
  2: { color: 'green', text: $t('page.statistics.performancePlan.approved') },
  3: { color: 'red', text: $t('page.statistics.performancePlan.rejected') },
};
const statusOptions = Object.entries(statusConfig).map(([value, cfg]) => ({
  value: Number(value),
  label: cfg.text,
}));

// ===== 格式化 =====
const toNum = (v: null | number | string | undefined) => {
  const n = typeof v === 'string' ? Number.parseFloat(v) : (v ?? 0);
  return Number.isFinite(n) ? n : 0;
};
const fmtAmount = (v: null | number | string | undefined) => {
  const num = toNum(v);
  if (num >= 100_000_000)
    return `¥${(num / 100_000_000).toFixed(2)}${$t('page.statistics.yiUnit')}`;
  if (num >= 10_000)
    return `¥${(num / 10_000).toFixed(1)}${$t('page.statistics.currencyFormat')}`;
  return `¥${num.toFixed(2)}`;
};
const rateColor = (v: null | number | string | undefined) => {
  const n = toNum(v);
  if (n >= 100) return 'green';
  if (n >= 60) return 'blue';
  return 'red';
};
// P2-2 低达成红旗预警：完成率 <60% 的行整行红色底色高亮（方案 §5 第三期）
const rowRedFlag = ({ row }: { row: any }) =>
  toNum(row.completionRate) < 60 ? 'row-red-flag' : '';

// ===== 远程排序：vxe 字段（camelCase）→ 后端 order_by（snake_case）白名单 =====
const sortFieldMap: Record<string, string> = {
  employeeName: 'employee_name',
  status: 'status',
  totalContractTarget: 'total_contract_target',
  completionRate: 'completion_rate',
  paymentCompletionRate: 'payment_completion_rate',
  version: 'version',
  submitTime: 'submit_time',
};
const orderBy = ref<string>();
const orderDir = ref<'asc' | 'desc'>();
const onSortChange = ({ field, order }: { field?: string; order?: null | string }) => {
  orderBy.value = field && sortFieldMap[field] ? sortFieldMap[field] : undefined;
  orderDir.value = order === 'asc' ? 'asc' : (order === 'desc' ? 'desc' : undefined);
  gridApi.query();
};

// ===== vxe-grid =====
const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  pagerConfig: { pageSize: 20, pageSizes: [10, 20, 50, 100] },
  sortConfig: { remote: true, trigger: 'cell' } as any,
  cellConfig: { isHover: true } as any,
  rowConfig: { height: 'auto' as any },
  rowClassName: rowRedFlag,
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    response: {
      result: 'items',
      total: 'total',
    },
    ajax: {
      query: async ({ page }) => {
        const result = await getPlanListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          year: filterYear.value || undefined,
          deptId: filterDeptId.value || undefined,
          statusList:
            filterStatusList.value && filterStatusList.value.length > 0
              ? filterStatusList.value
              : undefined,
          keyword: filterKeyword.value?.trim() || undefined,
          orderBy: orderBy.value,
          orderDir: orderDir.value,
        });
        loadCoverage();
        return result;
      },
    },
  },
  columns: [
    { type: 'seq', width: 50, title: $t('page.statistics.performancePlan.seq') },
    {
      field: 'employeeName',
      title: $t('page.statistics.performancePlan.employee'),
      minWidth: 90,
      sortable: true,
    },
    {
      field: 'deptName',
      title: $t('page.statistics.performancePlan.dept'),
      minWidth: 110,
    },
    {
      field: 'status',
      title: $t('page.statistics.performancePlan.status'),
      width: 90,
      sortable: true,
      slots: { default: 'status' },
    },
    {
      field: 'totalContractTarget',
      title: $t('page.statistics.performancePlan.contractTarget'),
      minWidth: 100,
      sortable: true,
      align: 'right',
      formatter: ({ cellValue }) => fmtAmount(cellValue),
    },
    {
      field: 'totalContractActual',
      title: $t('page.statistics.contractActual'),
      minWidth: 100,
      align: 'right',
      formatter: ({ cellValue }) => fmtAmount(cellValue),
    },
    {
      field: 'completionRate',
      title: $t('page.statistics.performancePlan.contractRate'),
      width: 110,
      sortable: true,
      align: 'center',
      slots: { default: 'completionRate' },
    },
    {
      field: 'totalPaymentTarget',
      title: $t('page.statistics.performancePlan.paymentTarget'),
      minWidth: 100,
      align: 'right',
      formatter: ({ cellValue }) => fmtAmount(cellValue),
    },
    {
      field: 'totalPaymentActual',
      title: $t('page.statistics.paymentActual'),
      minWidth: 100,
      align: 'right',
      formatter: ({ cellValue }) => fmtAmount(cellValue),
    },
    {
      field: 'paymentCompletionRate',
      title: $t('page.statistics.performancePlan.paymentRate'),
      width: 110,
      sortable: true,
      align: 'center',
      slots: { default: 'paymentCompletionRate' },
    },
    {
      field: 'version',
      title: $t('page.statistics.performancePlan.version'),
      width: 70,
      sortable: true,
      align: 'center',
    },
    {
      field: 'submitTime',
      title: $t('page.statistics.performancePlan.submitTime'),
      minWidth: 150,
      sortable: true,
    },
    {
      field: 'currentApproverName',
      title: $t('page.statistics.performancePlan.currentApprover'),
      minWidth: 120,
      slots: { default: 'approver' },
    },
    {
      field: 'actions',
      title: $t('page.statistics.performancePlan.operate'),
      width: 200,
      fixed: 'right',
      slots: { default: 'actions' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({
  gridOptions,
  gridEvents: { sortChange: onSortChange } as any,
});

const handleSearch = () => gridApi.query();
const handleReset = () => {
  filterDeptId.value = undefined;
  filterStatusList.value = [];
  filterKeyword.value = '';
  orderBy.value = undefined;
  orderDir.value = undefined;
  gridApi.query();
};

// ===== 行操作：月度明细 / 审批 / 代改 / 代建 =====
const compareVisible = ref(false);
const compareEmployeeId = ref<number>();
const compareEmployeeName = ref('');
const openCompare = (row: any) => {
  compareEmployeeId.value = row.employeeId;
  compareEmployeeName.value = row.employeeName ?? '';
  compareVisible.value = true;
};

const drawerVisible = ref(false);
const targetEmployeeId = ref<number>();
const openDrawerFor = (employeeId: number) => {
  targetEmployeeId.value = employeeId;
  drawerVisible.value = true;
};
const handleDrawerSuccess = () => {
  gridApi.query();
};

const coverageCreate = (item: CoverageItem) => {
  openDrawerFor(item.employeeId);
};

// 一键催办：向未提交计划的员工批量发送站内通知（manage 权限，方案 §4.6.1 三期能力）
const reminding = ref(false);
const remindAll = async () => {
  if (reminding.value || uncoveredList.value.length === 0) return;
  reminding.value = true;
  try {
    const count = await remindPlanApi({ year: filterYear.value });
    message.success(
      $t('page.statistics.performancePlan.remindSuccess', { n: count }),
    );
  } catch (error: any) {
    message.error(
      error?.message || $t('page.statistics.performancePlan.remindFailed'),
    );
  } finally {
    reminding.value = false;
  }
};

// ===== 行内审批直达（方案 §4.6.1：【审批】待审批直达） =====
const auditVisible = ref(false);
const auditRow = ref<any>(null);
const auditAction = ref<'approve' | 'reject'>('approve');
const auditReason = ref('');
const auditSubmitting = ref(false);

const openAudit = (row: any) => {
  auditRow.value = row;
  auditAction.value = 'approve';
  auditReason.value = '';
  auditVisible.value = true;
};

const submitAudit = async () => {
  const row = auditRow.value;
  if (!row?.id) return;
  const reason = auditReason.value.trim();
  if (auditAction.value === 'reject' && !reason) {
    message.warning($t('page.statistics.performancePlan.rejectReasonRequired'));
    return;
  }
  auditSubmitting.value = true;
  try {
    if (auditAction.value === 'approve') {
      await approvePlanApi(row.id, reason || undefined);
      message.success(
        $t('page.statistics.performancePlan.approveEmployeeSuccess', {
          name:
            row.employeeName ||
            $t('page.statistics.performancePlan.thisEmployee'),
        }),
      );
    } else {
      await rejectPlanApi(row.id, reason);
      message.success($t('page.statistics.performancePlan.rejectSuccess'));
    }
    auditVisible.value = false;
    gridApi.query();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.statistics.performancePlan.auditFailed'),
    );
  } finally {
    auditSubmitting.value = false;
  }
};
</script>

<template>
  <div class="flex flex-col gap-4">
    <!-- 覆盖度提示条 -->
    <Alert type="info" show-icon class="team-coverage-alert">
      <template #message>
        <span>
          {{ $t('page.statistics.performancePlan.coverageIntro', { year: filterYear }) }}
          <b>{{ coverage?.totalEmployees ?? '-' }}</b>
          {{ $t('page.statistics.performancePlan.coverageHavePlan') }}
          <b>{{ coverage?.withPlanCount ?? '-' }}</b>
          {{ $t('page.statistics.performancePlan.coverageApproved') }}
          <b>{{ coverage?.approvedCount ?? '-' }}</b>
          {{ $t('page.statistics.performancePlan.coverageUncovered') }}
          <b class="text-red-600">{{ uncoveredList.length }}</b>
          {{ $t('page.statistics.performancePlan.coverageTail') }}
        </span>
        <Button
          type="link"
          size="small"
          @click="coverageExpanded = !coverageExpanded"
        >
          {{ coverageExpanded ? $t('page.statistics.performancePlan.collapseList') : $t('page.statistics.performancePlan.expandList') }}
        </Button>
        <Button
          v-if="hasManage && uncoveredList.length > 0"
          type="link"
          size="small"
          :loading="reminding"
          @click="remindAll"
        >
          {{ $t('page.statistics.performancePlan.remindAllBtn') }}
        </Button>
      </template>
      <template v-if="coverageExpanded" #description>
        <div v-if="uncoveredList.length === 0" class="text-green-600">
          {{ $t('page.statistics.performancePlan.allCovered') }}
        </div>
        <div v-else class="flex flex-wrap gap-2">
          <span
            v-for="item in uncoveredList"
            :key="item.employeeId"
            class="inline-flex items-center gap-1 rounded border border-gray-200 px-2 py-0.5"
          >
            {{ item.name || $t('page.statistics.performancePlan.employeeFallback', { n: item.employeeId }) }}
            <span v-if="item.deptName" class="text-gray-400">({{ item.deptName }})</span>
            <Button
              v-if="hasManage"
              type="link"
              size="small"
              class="!px-1"
              @click="coverageCreate(item)"
            >
              {{ $t('page.statistics.performancePlan.createForEmployee') }}
            </Button>
          </span>
        </div>
      </template>
    </Alert>

    <!-- 筛选栏 -->
    <Card size="small">
      <div class="flex flex-wrap items-center gap-2">
        <Select
          v-model:value="filterYear"
          :options="yearOptions"
          class="w-28"
          @change="handleSearch"
        />
        <TreeSelect
          v-model:value="filterDeptId"
          :tree-data="deptTree"
          allow-clear
          show-search
          tree-node-filter-prop="title"
          :placeholder="$t('page.statistics.performancePlan.filterDeptPlaceholder')"
          class="w-48"
          @change="handleSearch"
        />
        <Select
          v-model:value="filterStatusList"
          :options="statusOptions"
          mode="multiple"
          allow-clear
          :placeholder="$t('page.statistics.performancePlan.filterStatusPlaceholder')"
          class="min-w-48"
          @change="handleSearch"
        />
        <Input
          v-model:value="filterKeyword"
          allow-clear
          :placeholder="$t('page.statistics.performancePlan.keywordPlaceholder')"
          class="w-52"
          @press-enter="handleSearch"
        />
        <Button type="primary" @click="handleSearch">
          {{ $t('page.statistics.performancePlan.search') }}
        </Button>
        <Button @click="handleReset">
          {{ $t('page.statistics.performancePlan.reset') }}
        </Button>
      </div>
    </Card>

    <!-- 团队计划列表 -->
    <Grid>
      <template #status="{ row }">
        <Tag :color="statusConfig[row.status]?.color ?? 'default'">
          {{ statusConfig[row.status]?.text ?? $t('page.statistics.performancePlan.statusFallback', { n: row.status }) }}
        </Tag>
      </template>
      <template #completionRate="{ row }">
        <Tag :color="rateColor(row.completionRate)">
          {{ toNum(row.completionRate).toFixed(2) }}%
        </Tag>
      </template>
      <template #paymentCompletionRate="{ row }">
        <Tag :color="rateColor(row.paymentCompletionRate)">
          {{ toNum(row.paymentCompletionRate).toFixed(2) }}%
        </Tag>
      </template>
      <template #approver="{ row }">
        <template v-if="row.status === 1 && row.currentApproverName">
          {{ row.currentApproverName }}
          <span v-if="row.approvalLevel && row.totalLevels" class="text-gray-400">
            {{ $t('page.statistics.performancePlan.levelSuffix', { level: row.approvalLevel, total: row.totalLevels }) }}
          </span>
        </template>
        <span v-else>—</span>
      </template>
      <template #actions="{ row }">
        <Button type="link" size="small" class="!px-1" @click="openCompare(row)">
          {{ $t('page.statistics.performancePlan.monthlyDetail') }}
        </Button>
        <Button
          v-if="hasAudit && row.status === 1 && row.currentApproverId === currentUserId"
          type="link"
          size="small"
          class="!px-1"
          @click="openAudit(row)"
        >
          {{ $t('page.statistics.performancePlan.audit') }}
        </Button>
        <Button
          v-if="hasManage && (row.status === 0 || row.status === 3)"
          type="link"
          size="small"
          class="!px-1"
          @click="openDrawerFor(row.employeeId)"
        >
          {{ $t('page.statistics.performancePlan.modifyForEmployee') }}
        </Button>
      </template>
    </Grid>

    <!-- 月度明细只读抽屉 -->
    <PlanMonthlyCompare
      v-model:visible="compareVisible"
      :employee-id="compareEmployeeId"
      :employee-name="compareEmployeeName"
      :year="filterYear"
    />

    <!-- 计划详情/代建代改抽屉（targetEmployeeId 由 PlanSettingDrawer 支持） -->
    <PlanSettingDrawer
      v-model:visible="drawerVisible"
      :year="filterYear"
      :target-employee-id="targetEmployeeId"
      @success="handleDrawerSuccess"
    />

    <!-- 审批直达（通过/驳回，方案 §4.6.1） -->
    <Modal
      v-model:open="auditVisible"
      :title="$t('page.statistics.performancePlan.auditTitle', { name: auditRow?.employeeName || '', year: auditRow?.year ?? filterYear })"
      :confirm-loading="auditSubmitting"
      :ok-text="
        auditAction === 'approve'
          ? $t('page.statistics.performancePlan.confirmApprove')
          : $t('page.statistics.performancePlan.confirmReject')
      "
      :ok-type="auditAction === 'approve' ? 'primary' : 'danger'"
      @ok="submitAudit"
    >
      <div class="flex flex-col gap-3 py-2">
        <div class="text-sm text-gray-600">
          {{ $t('page.statistics.performancePlan.contractTarget') }}
          <b>{{ fmtAmount(auditRow?.totalContractTarget) }}</b>
          ·
          {{ $t('page.statistics.performancePlan.paymentTarget') }}
          <b>{{ fmtAmount(auditRow?.totalPaymentTarget) }}</b>
          {{ $t('page.statistics.performancePlan.levelSuffix', { level: auditRow?.approvalLevel ?? 1, total: auditRow?.totalLevels ?? 1 }) }}
        </div>
        <div v-if="auditRow?.applyReason" class="text-sm">
          {{ $t('page.statistics.performancePlan.applyReasonLabel') }}{{ auditRow.applyReason }}
        </div>
        <RadioGroup v-model:value="auditAction">
          <Radio value="approve">
            {{ $t('page.statistics.performancePlan.approve') }}
          </Radio>
          <Radio value="reject">
            {{ $t('page.statistics.performancePlan.reject') }}
          </Radio>
        </RadioGroup>
        <Textarea
          v-model:value="auditReason"
          :rows="3"
          :maxlength="200"
          show-count
          :placeholder="
            auditAction === 'reject'
              ? $t('page.statistics.performancePlan.auditCommentRequired')
              : $t('page.statistics.performancePlan.auditCommentOptional')
          "
        />
      </div>
    </Modal>
  </div>
</template>

<style scoped>
/* P2-2 低达成红旗预警：完成率 <60% 的行整行红色底色（覆盖条纹与悬停背景） */
:deep(.vxe-body--row.row-red-flag) {
  background-color: #fff1f0 !important;
}

:deep(.vxe-body--row.row-red-flag:hover),
:deep(.vxe-body--row.row-red-flag.row--hover) {
  background-color: #ffccc7 !important;
}
</style>
