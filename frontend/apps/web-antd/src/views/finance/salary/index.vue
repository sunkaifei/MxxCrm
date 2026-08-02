<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, onMounted, reactive, ref } from 'vue';

import { useAccess } from '@vben/access';
import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import {
  Alert,
  Button,
  Card,
  Col,
  Drawer,
  Dropdown,
  Empty,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Modal,
  Row,
  Select,
  Spin,
  Statistic,
  Table,
  Tag,
  Tooltip,
} from 'ant-design-vue';
import { RefreshCw } from 'lucide-vue-next';
import { useRouter } from 'vue-router';
import { UserPickerModal } from '#/components/UserPickerModal';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  approveSalaryApi,
  batchApproveSalaryApi,
  batchPaySalaryApi,
  calculateSalaryApi,
  confirmSalaryApi,
  deleteSalaryConfigApi,
  exportSalaryApi,
  exportSalaryXlsxApi,
  exportTaxApi,
  exportTaxXlsxApi,
  getPendingConfirmsApi,
  getSalaryCalcLogListApi,
  getSalaryConfigListApi,
  getSalaryListApi,
  getSalarySummaryApi,
  handleConfirmApi,
  paySalaryApi,
  submitSalaryApprovalApi,
  syncSalaryApprovalApi,
  upsertSalaryConfigApi,
} from '#/api/core/finance';
import { $t } from '#/locales';
import { PageUsageGuide } from '#/components/PageUsageGuide';

const guideStepCount = 5;

const router = useRouter();

// ===== 数据权限分级 =====
const { hasAccessByRoles } = useAccess();
// 全量权限：超级管理员、财务、总经理、老板
const isFullScope = computed(() =>
  hasAccessByRoles([
    'super_admin',
    'admin',
    'finance',
    'general_manager',
    'boss',
    'cw',
  ]),
);
// 管理权限：有下属的管理岗（保留批量审核，但不显示核算/配置/日志）
const isManagerScope = computed(
  () =>
    !isFullScope.value &&
    hasAccessByRoles(['manager', 'dept_leader', 'sales_director']),
);
// 是否显示财务专属按钮（执行核算/底薪配置/核算日志）
const showFinanceButtons = computed(() => isFullScope.value);

// 日期格式化器（使用 Intl.DateTimeFormat，避免硬编码格式）
const dateTimeFormatter = new Intl.DateTimeFormat('zh-CN', {
  year: 'numeric',
  month: '2-digit',
  day: '2-digit',
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
  hour12: false,
});

const statusMap: Record<number, { label: string; color: string }> = {
  0: { label: $t('page.finance.salary.status.pending'), color: 'blue' },
  1: { label: $t('page.finance.salary.status.approved'), color: 'orange' },
  2: { label: $t('page.finance.salary.status.paid'), color: 'green' },
};

const statusOptions = [
  { value: 0, label: $t('page.finance.salary.status.pending') },
  { value: 1, label: $t('page.finance.salary.status.approved') },
  { value: 2, label: $t('page.finance.salary.status.paid') },
];

const monthOptions = Array.from({ length: 12 }, (_, i) => ({
  value: i + 1,
  label: $t('page.finance.salary.format.monthShort', { month: i + 1 }),
}));

const now = new Date();

// ===== 核算弹窗 =====
const calcVisible = ref(false);
const calcLoading = ref(false);
const calcForm = reactive({
  year: now.getFullYear(),
  month: now.getMonth() + 1,
});

// ===== 月度汇总 =====
const summaryData = ref<any>({});
const summaryLoading = ref(false);
const summaryYear = ref(now.getFullYear());
const summaryMonth = ref(now.getMonth() + 1);

async function loadSummary() {
  summaryLoading.value = true;
  try {
    const res: any = await getSalarySummaryApi({
      year: summaryYear.value,
      month: summaryMonth.value,
    });
    summaryData.value = res?.data || res || {};
  } catch {
    summaryData.value = {};
  } finally {
    summaryLoading.value = false;
  }
}

// ===== 底薪配置抽屉 =====
const configVisible = ref(false);
const configLoading = ref(false);
const configList = ref<any[]>([]);
const configFormVisible = ref(false);
const configForm = reactive({
  id: undefined as number | undefined,
  employeeId: undefined as number | undefined,
  employeeName: '',
  year: now.getFullYear(),
  month: undefined as number | undefined,
  baseSalary: 0,
  positionAllowance: undefined as number | undefined,
  performanceBase: undefined as number | undefined,
  performanceCoefficient: undefined as number | undefined,
});
const configFormSubmitting = ref(false);

async function loadConfigList() {
  configLoading.value = true;
  try {
    const res: any = await getSalaryConfigListApi({
      year: summaryYear.value,
    });
    configList.value = Array.isArray(res) ? res : res?.data || [];
  } catch {
    configList.value = [];
  } finally {
    configLoading.value = false;
  }
}

function openConfigForm(record?: any) {
  if (record) {
    configForm.id = record.id;
    configForm.employeeId = record.employeeId;
    configForm.employeeName = record.employeeName || '';
    configForm.year = record.year;
    configForm.month = record.month;
    configForm.baseSalary = Number(record.baseSalary || 0);
    configForm.positionAllowance = record.positionAllowance
      ? Number(record.positionAllowance)
      : undefined;
    configForm.performanceBase = record.performanceBase
      ? Number(record.performanceBase)
      : undefined;
    configForm.performanceCoefficient = record.performanceCoefficient
      ? Number(record.performanceCoefficient)
      : undefined;
  } else {
    configForm.id = undefined;
    configForm.employeeId = undefined;
    configForm.employeeName = '';
    configForm.year = now.getFullYear();
    configForm.month = undefined;
    configForm.baseSalary = 0;
    configForm.positionAllowance = undefined;
    configForm.performanceBase = undefined;
    configForm.performanceCoefficient = undefined;
  }
  configFormVisible.value = true;
}

async function submitConfigForm() {
  if (!configForm.employeeId) {
    message.warning($t('page.finance.salary.config.message.employeeIdRequired'));
    return;
  }
  if (configForm.baseSalary < 0) {
    message.warning($t('page.finance.salary.config.message.baseSalaryNegative'));
    return;
  }
  configFormSubmitting.value = true;
  try {
    await upsertSalaryConfigApi({
      employeeId: configForm.employeeId,
      year: configForm.year,
      month: configForm.month,
      baseSalary: configForm.baseSalary,
      positionAllowance: configForm.positionAllowance,
      performanceBase: configForm.performanceBase,
      performanceCoefficient: configForm.performanceCoefficient,
    });
    message.success($t('page.finance.common.saveSuccess'));
    configFormVisible.value = false;
    await loadConfigList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.saveFailed'));
  } finally {
    configFormSubmitting.value = false;
  }
}

async function deleteConfig(id: number) {
  Modal.confirm({
    title: $t('page.finance.salary.config.message.deleteTitle'),
    content: $t('page.finance.salary.config.message.deleteContent'),
    okText: $t('page.finance.common.delete'),
    okType: 'danger',
    cancelText: $t('page.finance.common.cancel'),
    async onOk() {
      try {
        await deleteSalaryConfigApi(id);
        message.success($t('page.finance.common.deleteSuccess'));
        await loadConfigList();
      } catch (e: any) {
        message.error(e?.message || $t('page.finance.common.deleteFailed'));
      }
    },
  });
}

// ===== 核算日志 =====
const logVisible = ref(false);
const logLoading = ref(false);
const logList = ref<any[]>([]);

async function loadLogList() {
  logLoading.value = true;
  try {
    const res: any = await getSalaryCalcLogListApi({
      year: summaryYear.value,
      month: summaryMonth.value,
      page: 1,
      pageSize: 50,
    });
    const data = res?.data || res;
    logList.value = Array.isArray(data) ? data : data?.items || [];
  } catch {
    logList.value = [];
  } finally {
    logLoading.value = false;
  }
}

const logTriggerMap: Record<number, { text: string; color: string }> = {
  0: { text: $t('page.finance.salary.log.trigger.manual'), color: 'blue' },
  1: { text: $t('page.finance.salary.log.trigger.auto'), color: 'purple' },
};

const logResultMap: Record<number, { text: string; color: string }> = {
  0: { text: $t('page.finance.salary.log.result.failed'), color: 'red' },
  1: { text: $t('page.finance.salary.log.result.success'), color: 'green' },
};

function formatMoney(val: any) {
  if (val === null || val === undefined || val === '') return '-';
  return `¥${Number(val).toLocaleString()}`;
}

function formatMoneyShort(val: any) {
  const num = Number(val || 0);
  if (num >= 100000000) return $t('page.finance.salary.format.yi', { value: (num / 100000000).toFixed(2) });
  if (num >= 10000) return $t('page.finance.salary.format.wan', { value: (num / 10000).toFixed(1) });
  return `¥${num.toLocaleString()}`;
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'InputNumber',
      fieldName: 'year',
      label: $t('page.finance.salary.config.label.year'),
      defaultValue: now.getFullYear(),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        min: 2020,
        max: 2099,
        style: { width: '100%' },
      },
    },
    {
      component: 'Select',
      fieldName: 'month',
      label: $t('page.finance.salary.config.column.month'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: monthOptions,
      },
    },
    {
      component: 'Input',
      fieldName: 'employeeName',
      label: $t('page.finance.salary.column.employeeName'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.finance.salary.column.status'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: statusOptions,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  pagerConfig: {},
  checkboxConfig: {
    highlight: true,
  },
  stripe: true,
  rowConfig: {
    isHover: true,
  },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        };
        // 同步汇总年份月份
        if (formValues.year) summaryYear.value = formValues.year;
        if (formValues.month) summaryMonth.value = formValues.month;
        const res = await getSalaryListApi(params);
        // 查询后刷新汇总
        loadSummary();
        return res;
      },
    },
  },

  columns: [
    {
      type: 'checkbox',
      width: 50,
    },
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: $t('page.finance.salary.column.employeeName'),
      field: 'employeeName',
      minWidth: 120,
    },
    {
      title: $t('page.finance.salary.column.department'),
      field: 'deptName',
      minWidth: 120,
    },
    {
      title: $t('page.finance.salary.column.yearMonth'),
      field: 'year',
      minWidth: 120,
      slots: { default: 'yearMonth' },
    },
    {
      title: $t('page.finance.salary.column.baseSalary'),
      field: 'baseSalary',
      width: 120,
      slots: { default: 'baseSalary' },
    },
    {
      title: $t('page.finance.salary.column.commissionAmount'),
      field: 'commissionAmount',
      width: 120,
      slots: { default: 'commissionAmount' },
    },
    {
      title: $t('page.finance.salary.column.performanceBonus'),
      field: 'performanceBonus',
      width: 120,
      slots: { default: 'performanceBonus' },
    },
    {
      title: $t('page.finance.salary.column.deductionAmount'),
      field: 'deduction',
      width: 120,
      slots: { default: 'deduction' },
    },
    {
      title: $t('page.finance.salary.column.totalSalary'),
      field: 'totalAmount',
      width: 140,
      slots: { default: 'totalAmount' },
    },
    {
      title: $t('page.finance.salary.column.socialInsurancePersonal'),
      field: 'socialInsurancePersonal',
      width: 100,
      slots: { default: 'socialInsurancePersonal' },
    },
    {
      title: $t('page.finance.salary.column.housingFundPersonal'),
      field: 'housingFundPersonal',
      width: 110,
      slots: { default: 'housingFundPersonal' },
    },
    {
      title: $t('page.finance.salary.column.taxAmount'),
      field: 'taxAmount',
      width: 100,
      slots: { default: 'taxAmount' },
    },
    {
      title: $t('page.finance.salary.column.netSalary'),
      field: 'netSalary',
      width: 120,
      slots: { default: 'netSalary' },
    },
    {
      title: $t('page.finance.salary.column.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('page.finance.salary.column.employeeConfirmed'),
      field: 'employeeConfirmed',
      width: 100,
      slots: { default: 'employeeConfirmed' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 280,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function goDetail(row: any) {
  void router.push({ path: `/finance/salary/detail/${row.id}` });
}

async function handleApprove(row: any) {
  row.pending = true;
  try {
    await approveSalaryApi(row.id);
    message.success($t('page.finance.salary.message.approveSuccess'));
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.salary.message.approveFailed'));
  } finally {
    row.pending = false;
  }
}

async function handlePay(row: any) {
  row.pending = true;
  try {
    await paySalaryApi(row.id);
    message.success($t('page.finance.salary.message.paySuccess'));
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.salary.message.payFailed'));
  } finally {
    row.pending = false;
  }
}

function handleAdjust(row: any) {
  goDetail(row);
}

async function handleCalculate() {
  calcLoading.value = true;
  try {
    const res: any = await calculateSalaryApi({
      year: calcForm.year,
      month: calcForm.month,
    });
    const count = res?.data ?? res;
    message.success(
      $t('page.finance.salary.modal.calcSuccess', { count }),
    );
    calcVisible.value = false;
    gridApi.query();
    loadSummary();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.salary.message.calcFailed'));
  } finally {
    calcLoading.value = false;
  }
}

async function handleBatchApprove() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    message.warning($t('page.finance.salary.message.batchApproveSelectRequired'));
    return;
  }
  const ids = records.map((r: any) => r.id);
  try {
    await batchApproveSalaryApi(ids);
    message.success($t('page.finance.salary.message.batchApproveSuccess'));
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.salary.message.batchApproveFailed'));
  }
}

async function handleBatchPay() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    message.warning($t('page.finance.salary.message.batchPaySelectRequired'));
    return;
  }
  const ids = records.map((r: any) => r.id);
  try {
    await batchPaySalaryApi(ids);
    message.success($t('page.finance.salary.message.batchPaySuccess'));
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.salary.message.batchPayFailed'));
  }
}

// 底薪配置表格列
const configColumns = computed(() => [
  { title: $t('page.finance.salary.config.column.employeeId'), dataIndex: 'employeeId', width: 80 },
  {
    title: $t('page.finance.salary.config.column.year'),
    dataIndex: 'year',
    width: 80,
    customRender: ({ text }: any) => $t('page.finance.salary.format.year', { year: text }),
  },
  {
    title: $t('page.finance.salary.config.column.month'),
    dataIndex: 'month',
    width: 80,
    customRender: ({ text }: any) => (text ? $t('page.finance.salary.format.month', { month: text }) : $t('page.finance.salary.config.fullYear')),
  },
  {
    title: $t('page.finance.salary.config.column.baseSalary'),
    dataIndex: 'baseSalary',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.salary.config.column.positionAllowance'),
    dataIndex: 'positionAllowance',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.salary.config.column.performanceBase'),
    dataIndex: 'performanceBase',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.salary.config.column.performanceCoefficient'),
    dataIndex: 'performanceCoefficient',
    width: 100,
    customRender: ({ text }: any) => (text ? `${Number(text).toFixed(2)}` : $t('page.finance.salary.config.auto')),
  },
  { title: $t('page.finance.salary.config.column.action'), key: 'action', width: 140, fixed: 'right' as const },
]);

// 核算日志表格列
const logColumns = computed(() => [
  {
    title: $t('page.finance.salary.log.column.yearMonth'),
    key: 'yearMonth',
    width: 120,
    customRender: ({ record }: any) => `${$t('page.finance.salary.format.year', { year: record.year })}${$t('page.finance.salary.format.month', { month: record.month })}`,
  },
  {
    title: $t('page.finance.salary.log.column.triggerType'),
    dataIndex: 'triggerType',
    width: 100,
    customRender: ({ text }: any) => {
      const m = logTriggerMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.text) : text;
    },
  },
  {
    title: $t('page.finance.salary.log.column.result'),
    dataIndex: 'result',
    width: 80,
    customRender: ({ text }: any) => {
      const m = logResultMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.text) : '-';
    },
  },
  {
    title: $t('page.finance.salary.log.column.generatedCount'),
    dataIndex: 'generatedCount',
    width: 100,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.salary.log.column.elapsed'),
    dataIndex: 'elapsedMs',
    width: 100,
    align: 'right' as const,
    customRender: ({ text }: any) => (text ? `${text}ms` : '-'),
  },
  {
    title: $t('page.finance.salary.log.column.operator'),
    dataIndex: 'operatorName',
    width: 100,
  },
  {
    title: $t('page.finance.salary.log.column.errorMessage'),
    dataIndex: 'errorMessage',
    ellipsis: true,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.salary.log.column.executeTime'),
    dataIndex: 'createTime',
    width: 170,
    customRender: ({ text }: any) => {
      if (!text) return '-';
      try {
        return dateTimeFormatter.format(new Date(text));
      } catch {
        return '-';
      }
    },
  },
]);

// ===== 工资确认/申诉（员工侧） =====
const confirmVisible = ref(false);
const confirmLoading = ref(false);
const confirmTarget = ref<any>(null);

function openConfirmModal(record: any) {
  confirmTarget.value = record;
  confirmVisible.value = true;
}

async function handleConfirmSalary() {
  if (!confirmTarget.value) return;
  confirmLoading.value = true;
  try {
    await confirmSalaryApi({
      salaryRecordId: confirmTarget.value.id,
      action: 1,
    });
    message.success($t('page.finance.salary.message.confirmSuccess'));
    confirmVisible.value = false;
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.salary.message.confirmFailed'));
  } finally {
    confirmLoading.value = false;
  }
}

// 申请重新核算
const appealVisible = ref(false);
const appealLoading = ref(false);
const appealTarget = ref<any>(null);
const appealForm = reactive({ reason: '' });

function openAppealModal(record: any) {
  appealTarget.value = record;
  appealForm.reason = '';
  appealVisible.value = true;
}

async function handleAppealSalary() {
  if (!appealTarget.value) return;
  if (!appealForm.reason.trim()) {
    message.warning($t('page.finance.salary.modal.recalcReasonRequired'));
    return;
  }
  appealLoading.value = true;
  try {
    await confirmSalaryApi({
      salaryRecordId: appealTarget.value.id,
      action: 2,
      reason: appealForm.reason,
    });
    message.success($t('page.finance.salary.message.recalcSubmitted'));
    appealVisible.value = false;
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.salary.message.submitFailed'));
  } finally {
    appealLoading.value = false;
  }
}

// ===== 待处理申诉（财务侧） =====
const pendingDrawerVisible = ref(false);
const pendingList = ref<any[]>([]);
const pendingLoading = ref(false);
// V8-6: 申诉列表筛选条件
const pendingFilter = reactive({
  employeeId: undefined as number | undefined,
  year: undefined as number | undefined,
  month: undefined as number | undefined,
  status: undefined as number | undefined,
});
const pendingStatusOptions = [
  { value: 1, label: $t('page.finance.salary.confirmStatus.confirmed') },
  { value: 2, label: $t('page.finance.salary.confirmStatus.recalcRequested') },
  { value: 3, label: $t('page.finance.salary.confirmStatus.appealing') },
];
const pendingColumns = [
  { title: $t('page.finance.salary.pending.column.employee'), dataIndex: 'employeeName', width: 100 },
  {
    title: $t('page.finance.salary.pending.column.yearMonth'),
    dataIndex: 'year',
    width: 100,
    customRender: ({ record }: any) => `${record.year}-${record.month}`,
  },
  { title: $t('page.finance.salary.pending.column.reason'), dataIndex: 'reason', ellipsis: true },
  { title: $t('page.finance.salary.pending.column.createTime'), dataIndex: 'createTime', width: 170 },
  { title: $t('page.finance.salary.pending.column.action'), dataIndex: 'operation', width: 160, fixed: 'right' as const },
];

async function loadPendingList() {
  pendingLoading.value = true;
  try {
    const res: any = await getPendingConfirmsApi({
      page: 1,
      pageSize: 100,
      employeeId: pendingFilter.employeeId,
      year: pendingFilter.year,
      month: pendingFilter.month,
      status: pendingFilter.status,
    });
    const data = res?.data || res;
    pendingList.value = Array.isArray(data) ? data : data?.items || [];
  } catch {
    pendingList.value = [];
  } finally {
    pendingLoading.value = false;
  }
}

function resetPendingFilter() {
  pendingFilter.employeeId = undefined;
  pendingFilter.year = undefined;
  pendingFilter.month = undefined;
  pendingFilter.status = undefined;
  loadPendingList();
}

function openPendingDrawer() {
  pendingDrawerVisible.value = true;
  loadPendingList();
}

// ===== V8-5: 导出工资单/个税（V9: 支持 CSV 与 XLSX 双格式）=====
const exportingSalary = ref(false);
const exportingTax = ref(false);

function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

// V9: format: 'csv' | 'xlsx'
async function handleExportSalary(format: 'csv' | 'xlsx' = 'xlsx') {
  exportingSalary.value = true;
  try {
    const params = { year: summaryYear.value, month: summaryMonth.value };
    const res: any = format === 'xlsx'
      ? await exportSalaryXlsxApi(params)
      : await exportSalaryApi(params);
    const mime = format === 'xlsx'
      ? 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
      : 'text/csv;charset=utf-8';
    const blob = res instanceof Blob ? res : new Blob([res as any], { type: mime });
    downloadBlob(blob, `salary_${summaryYear.value}-${summaryMonth.value}.${format}`);
    message.success($t('page.finance.common.success'));
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.failed'));
  } finally {
    exportingSalary.value = false;
  }
}

async function handleExportTax(format: 'csv' | 'xlsx' = 'xlsx') {
  exportingTax.value = true;
  try {
    const params = { year: summaryYear.value, month: summaryMonth.value };
    const res: any = format === 'xlsx'
      ? await exportTaxXlsxApi(params)
      : await exportTaxApi(params);
    const mime = format === 'xlsx'
      ? 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
      : 'text/csv;charset=utf-8';
    const blob = res instanceof Blob ? res : new Blob([res as any], { type: mime });
    downloadBlob(blob, `tax_${summaryYear.value}-${summaryMonth.value}.${format}`);
    message.success($t('page.finance.common.success'));
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.failed'));
  } finally {
    exportingTax.value = false;
  }
}

// V9: Dropdown 菜单选项
const salaryExportMenu = computed(() => [
  { key: 'xlsx', label: $t('page.finance.salary.export.xlsx') },
  { key: 'csv', label: $t('page.finance.salary.export.csv') },
]);
const taxExportMenu = computed(() => [
  { key: 'xlsx', label: $t('page.finance.salary.export.xlsx') },
  { key: 'csv', label: $t('page.finance.salary.export.csv') },
]);
function onSalaryExportMenu(e: { key: any }) {
  handleExportSalary(e.key as 'csv' | 'xlsx');
}
function onTaxExportMenu(e: { key: any }) {
  handleExportTax(e.key as 'csv' | 'xlsx');
}

// ===== V8-1: 工资单审批流对接 =====
const submittingApproval = ref(false);
const syncingApproval = ref(false);

async function handleSubmitApproval() {
  Modal.confirm({
    title: $t('page.finance.salary.modal.submitApprovalTitle'),
    content: $t('page.finance.salary.modal.submitApprovalContent', {
      year: summaryYear.value,
      month: summaryMonth.value,
    }),
    okText: $t('page.finance.common.confirm'),
    cancelText: $t('page.finance.common.cancel'),
    async onOk() {
      submittingApproval.value = true;
      try {
        const res: any = await submitSalaryApprovalApi({
          year: summaryYear.value,
          month: summaryMonth.value,
        });
        const data = res?.data || res;
        message.success(data?.message || $t('page.finance.common.success'));
        gridApi.query();
      } catch (e: any) {
        message.error(e?.message || $t('page.finance.common.failed'));
      } finally {
        submittingApproval.value = false;
      }
    },
  });
}

async function handleSyncApproval() {
  syncingApproval.value = true;
  try {
    const res: any = await syncSalaryApprovalApi({
      year: summaryYear.value,
      month: summaryMonth.value,
    });
    const data = res?.data || res;
    message.success(data?.message || $t('page.finance.common.success'));
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.failed'));
  } finally {
    syncingApproval.value = false;
  }
}

async function handleApproveAppeal(record: any) {
  Modal.confirm({
    title: $t('page.finance.salary.modal.approveRecalcTitle'),
    content: $t('page.finance.salary.modal.approveRecalcContent'),
    async onOk() {
      try {
        await handleConfirmApi({
          confirmId: record.id,
          action: 1,
          remark: $t('page.finance.salary.modal.approveRecalcRemark'),
        });
        message.success($t('page.finance.salary.modal.approveRecalcSuccess'));
        loadPendingList();
        gridApi.query();
      } catch (e: any) {
        message.error(e?.message || $t('page.finance.salary.modal.handleFailed'));
      }
    },
  });
}

async function handleRejectAppeal(record: any) {
  Modal.confirm({
    title: $t('page.finance.salary.modal.rejectAppealTitle'),
    content: $t('page.finance.salary.modal.rejectAppealContent'),
    async onOk() {
      try {
        await handleConfirmApi({
          confirmId: record.id,
          action: 2,
          remark: $t('page.finance.salary.modal.rejectAppealRemark'),
        });
        message.success($t('page.finance.salary.modal.rejectAppealSuccess'));
        loadPendingList();
      } catch (e: any) {
        message.error(e?.message || $t('page.finance.salary.modal.handleFailed'));
      }
    },
  });
}

onMounted(() => {
  loadSummary();
});
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.finance.salary.guide.title')"
      :brief="$t('page.finance.salary.guide.brief')"
      :expand-text="$t('page.finance.salary.guide.expand')"
      :collapse-text="$t('page.finance.salary.guide.collapse')"
    >
      <div
        v-for="i in guideStepCount"
        :key="i"
        class="page-guide-step-item"
      >
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.salary.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.salary.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <!-- 月度汇总卡片 -->
    <Card class="mb-4" :bordered="false">
      <Spin :spinning="summaryLoading">
        <div class="mb-3 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <IconifyIcon icon="lucide:trending-up" class="text-lg text-primary" />
            <span class="font-semibold">
              {{ $t('page.finance.salary.summary.title', { year: summaryYear, month: summaryMonth }) }}
            </span>
          </div>
          <Button size="small" :aria-label="$t('page.finance.salary.tooltip.refreshSummary')" @click="loadSummary">
            <template #icon>
              <IconifyIcon icon="lucide:refresh-cw" aria-hidden="true" />
            </template>
            {{ $t('page.finance.salary.summary.refresh') }}
          </Button>
        </div>
        <Row :gutter="16">
          <Col :span="4">
            <Statistic
              :title="$t('page.finance.salary.summary.baseSalaryGrandTotal')"
              :value="summaryData.totalBase || 0"
              :value-style="{ color: '#1890ff' }"
              :formatter="(v: any) => formatMoneyShort(v.value)"
            />
          </Col>
          <Col :span="4">
            <Statistic
              :title="$t('page.finance.salary.summary.commissionGrandTotal')"
              :value="summaryData.totalCommission || 0"
              :value-style="{ color: '#52c41a' }"
              :formatter="(v: any) => formatMoneyShort(v.value)"
            />
          </Col>
          <Col :span="4">
            <Statistic
              :title="$t('page.finance.salary.summary.performanceGrandTotal')"
              :value="summaryData.totalBonus || 0"
              :value-style="{ color: '#722ed1' }"
              :formatter="(v: any) => formatMoneyShort(v.value)"
            />
          </Col>
          <Col :span="4">
            <Statistic
              :title="$t('page.finance.salary.summary.deductionGrandTotal')"
              :value="summaryData.totalDeduction || 0"
              :value-style="{ color: '#ff4d4f' }"
              :formatter="(v: any) => formatMoneyShort(v.value)"
            />
          </Col>
          <Col :span="4">
            <Statistic
              :title="$t('page.finance.salary.summary.totalSalaryGrandTotal')"
              :value="summaryData.totalSalary || 0"
              :value-style="{ color: '#fa8c16', fontWeight: 'bold' }"
              :formatter="(v: any) => formatMoneyShort(v.value)"
            />
          </Col>
          <Col :span="4">
            <Statistic
              :title="$t('page.finance.salary.summary.recordCount')"
              :value="summaryData.count || 0"
              :value-style="{ color: '#13c2c2' }"
            />
          </Col>
          <Col :span="4">
            <Statistic
              :title="$t('page.finance.salary.summary.socialInsuranceTotal')"
              :value="summaryData.totalSocialInsurancePersonal || 0"
              :value-style="{ color: '#faad14' }"
              :formatter="(v: any) => formatMoneyShort(v.value)"
            />
          </Col>
          <Col :span="4">
            <Statistic
              :title="$t('page.finance.salary.summary.taxTotal')"
              :value="summaryData.totalTaxAmount || 0"
              :value-style="{ color: '#eb2f96' }"
              :formatter="(v: any) => formatMoneyShort(v.value)"
            />
          </Col>
          <Col :span="4">
            <Statistic
              :title="$t('page.finance.salary.summary.netSalaryTotal')"
              :value="summaryData.totalNetSalary || 0"
              :value-style="{ color: '#1890ff', fontWeight: 'bold' }"
              :formatter="(v: any) => formatMoneyShort(v.value)"
            />
          </Col>
        </Row>
      </Spin>
    </Card>

    <Grid :table-title="$t('page.finance.salary.title')">
      <template #toolbar-tools>
        <Tooltip
          v-if="showFinanceButtons"
          :title="$t('page.finance.salary.tooltip.calculate')"
        >
          <Button type="primary" class="mr-2" @click="calcVisible = true">
            <template #icon>
              <IconifyIcon icon="lucide:calculator" />
            </template>
            {{ $t('page.finance.salary.button.calculate') }}
          </Button>
        </Tooltip>
        <Button
          v-if="isFullScope || isManagerScope"
          class="mr-2"
          @click="handleBatchApprove"
        >
          <template #icon>
            <IconifyIcon icon="lucide:check-square" />
          </template>
          {{ $t('page.finance.salary.button.batchApprove') }}
        </Button>
        <Button
          v-if="isFullScope || isManagerScope"
          class="mr-2"
          @click="handleBatchPay"
        >
          <template #icon>
            <IconifyIcon icon="lucide:banknote" />
          </template>
          {{ $t('page.finance.salary.button.batchPay') }}
        </Button>
        <Button
          v-if="showFinanceButtons"
          class="mr-2"
          @click="configVisible = true; loadConfigList()"
        >
          <template #icon>
            <IconifyIcon icon="lucide:settings" />
          </template>
          {{ $t('page.finance.salary.button.config') }}
        </Button>
        <Button
          v-if="showFinanceButtons"
          class="mr-2"
          @click="logVisible = true; loadLogList()"
        >
          <template #icon>
            <IconifyIcon icon="lucide:history" />
          </template>
          {{ $t('page.finance.salary.button.calcLog') }}
        </Button>
        <Button
          v-if="showFinanceButtons"
          class="mr-2"
          @click="openPendingDrawer"
        >
          <template #icon>
            <IconifyIcon icon="lucide:alert-circle" />
          </template>
          {{ $t('page.finance.salary.button.pendingConfirm') }}
        </Button>
        <Dropdown
          v-if="isFullScope"
          :menu="{ items: salaryExportMenu, onClick: onSalaryExportMenu }"
          placement="bottomLeft"
        >
          <Button class="mr-2" :loading="exportingSalary">
            <template #icon>
              <IconifyIcon icon="lucide:file-down" />
            </template>
            {{ $t('page.finance.common.exportSalary') }}
          </Button>
        </Dropdown>
        <Button
          v-if="isFullScope"
          class="mr-2"
          :loading="submittingApproval"
          @click="handleSubmitApproval"
        >
          <template #icon>
            <IconifyIcon icon="lucide:git-pull-request" />
          </template>
          {{ $t('page.finance.salary.button.submitApproval') }}
        </Button>
        <Button
          v-if="isFullScope"
          class="mr-2"
          :loading="syncingApproval"
          @click="handleSyncApproval"
        >
          <template #icon>
            <IconifyIcon icon="lucide:refresh-cw" />
          </template>
          {{ $t('page.finance.salary.button.syncApproval') }}
        </Button>
        <Dropdown
          v-if="isFullScope"
          :menu="{ items: taxExportMenu, onClick: onTaxExportMenu }"
          placement="bottomLeft"
        >
          <Button class="mr-2" :loading="exportingTax">
            <template #icon>
              <IconifyIcon icon="lucide:receipt" />
            </template>
            {{ $t('page.finance.common.exportTax') }}
          </Button>
        </Dropdown>
        <Button
          class="mr-2"
          :aria-label="$t('page.finance.salary.tooltip.refreshList')"
          :icon="h(RefreshCw)"
          @click="gridApi.query()"
        >
          {{ $t('page.finance.common.refresh') }}
        </Button>
      </template>

      <template #yearMonth="{ row }">
        {{ $t('page.finance.salary.format.year', { year: row.year }) }}{{ $t('page.finance.salary.format.month', { month: row.month }) }}
      </template>

      <template #baseSalary="{ row }">
        <span :class="{ 'text-gray-400': !row.baseSalary }">
          {{ formatMoney(row.baseSalary) }}
        </span>
      </template>
      <template #commissionAmount="{ row }">
        <span :class="{ 'text-gray-400': !row.commissionAmount }">
          {{ formatMoney(row.commissionAmount) }}
        </span>
      </template>
      <template #performanceBonus="{ row }">
        <span :class="{ 'text-gray-400': !row.performanceBonus }">
          {{ formatMoney(row.performanceBonus) }}
        </span>
      </template>
      <template #deductionAmount="{ row }">
        <span :class="{ 'text-gray-400': !row.deductionAmount }">
          {{ formatMoney(row.deductionAmount) }}
        </span>
      </template>
      <template #totalSalary="{ row }">
        <span class="font-semibold text-primary">
          {{ formatMoney(row.totalSalary) }}
        </span>
      </template>

      <template #socialInsurancePersonal="{ row }">
        <span :class="{ 'text-gray-400': !row.socialInsurancePersonal }">
          {{ formatMoney(row.socialInsurancePersonal) }}
        </span>
      </template>
      <template #housingFundPersonal="{ row }">
        <span :class="{ 'text-gray-400': !row.housingFundPersonal }">
          {{ formatMoney(row.housingFundPersonal) }}
        </span>
      </template>
      <template #taxAmount="{ row }">
        <span :class="{ 'text-gray-400': !row.taxAmount }">
          {{ formatMoney(row.taxAmount) }}
        </span>
      </template>
      <template #netSalary="{ row }">
        <span class="font-semibold" style="color: #1890ff">
          {{ formatMoney(row.netSalary) }}
        </span>
      </template>

      <template #status="{ row }">
        <Tag :color="statusMap[row.status]?.color || 'default'">
          {{ statusMap[row.status]?.label || row.status }}
        </Tag>
      </template>

      <template #employeeConfirmed="{ row }">
        <Tag v-if="row.employeeConfirmed === 0" color="default">{{ $t('page.finance.salary.confirmStatus.unconfirmed') }}</Tag>
        <Tag v-else-if="row.employeeConfirmed === 1" color="green">
          {{ $t('page.finance.salary.confirmStatus.confirmed') }}
        </Tag>
        <Tag v-else-if="row.employeeConfirmed === 2" color="orange">
          {{ $t('page.finance.salary.confirmStatus.appealing') }}
        </Tag>
        <Tag v-else color="default">-</Tag>
      </template>

      <template #action="{ row }">
        <!-- 财务/超管：审核/发放/调整/详情/处理申诉 -->
        <template v-if="isFullScope">
          <Button type="link" size="small" @click="goDetail(row)">{{ $t('page.finance.salary.action.detail') }}</Button>
          <Button
            v-if="row.status === 0"
            type="link"
            size="small"
            :loading="row.pending"
            @click="handleApprove(row)"
          >
            {{ $t('page.finance.salary.action.approve') }}
          </Button>
          <Button
            v-if="row.status === 0"
            type="link"
            size="small"
            @click="handleAdjust(row)"
          >
            {{ $t('page.finance.salary.action.adjust') }}
          </Button>
          <Button
            v-if="row.status === 1"
            type="link"
            size="small"
            :loading="row.pending"
            @click="handlePay(row)"
          >
            {{ $t('page.finance.salary.action.pay') }}
          </Button>
          <Button
            v-if="row.employeeConfirmed === 2"
            type="link"
            size="small"
            @click="openPendingDrawer"
          >
            {{ $t('page.finance.salary.action.handleAppeal') }}
          </Button>
        </template>
        <!-- 管理员：详情 -->
        <template v-else-if="isManagerScope">
          <Button type="link" size="small" @click="goDetail(row)">{{ $t('page.finance.salary.action.detail') }}</Button>
        </template>
        <!-- 普通员工（销售）：确认工资/申请重新核算 -->
        <template v-else>
          <Button
            v-if="row.employeeConfirmed === 0"
            type="link"
            size="small"
            @click="openConfirmModal(row)"
          >
            {{ $t('page.finance.salary.button.confirm') }}
          </Button>
          <Button
            v-if="row.employeeConfirmed !== 2"
            type="link"
            size="small"
            @click="openAppealModal(row)"
          >
            {{ $t('page.finance.salary.button.applyRecalc') }}
          </Button>
        </template>
      </template>
    </Grid>

    <!-- 核算弹窗 -->
    <Modal
      v-model:open="calcVisible"
      :title="$t('page.finance.salary.modal.calcTitle')"
      :confirm-loading="calcLoading"
      @ok="handleCalculate"
    >
      <div class="py-4">
        <div class="mb-3 flex items-center gap-2 rounded bg-blue-50 p-3 text-sm text-blue-600">
          <IconifyIcon icon="lucide:info" />
          <span>{{ $t('page.finance.salary.modal.calcContentManual') }}</span>
        </div>
        <div class="flex items-center gap-3">
          <span>{{ $t('page.finance.salary.modal.calcYearShort') }}</span>
          <InputNumber
            v-model:value="calcForm.year"
            :min="2020"
            :max="2099"
            style="width: 120px"
          />
          <span>{{ $t('page.finance.salary.modal.calcMonthShort') }}</span>
          <Select
            v-model:value="calcForm.month"
            :options="monthOptions"
            style="width: 120px"
          />
        </div>
      </div>
    </Modal>

    <!-- 底薪配置抽屉 -->
    <Drawer
      v-model:open="configVisible"
      :title="$t('page.finance.salary.config.title')"
      width="900px"
      :body-style="{ padding: '16px' }"
    >
      <div class="mb-4 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-500">
          <IconifyIcon icon="lucide:info" />
          <span>{{ $t('page.finance.salary.config.info') }}</span>
        </div>
        <Button type="primary" size="small" @click="openConfigForm()">
          <IconifyIcon icon="lucide:plus" class="mr-1" />
          {{ $t('page.finance.salary.config.addButton') }}
        </Button>
      </div>

      <Table
        :columns="configColumns"
        :data-source="configList"
        :loading="configLoading"
        row-key="id"
        :pagination="false"
        size="middle"
        :scroll="{ x: 800 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'action'">
            <Button type="link" size="small" @click="openConfigForm(record)">
              {{ $t('page.finance.common.edit') }}
            </Button>
            <Button type="link" size="small" danger @click="deleteConfig(record.id)">
              {{ $t('page.finance.common.delete') }}
            </Button>
          </template>
        </template>
      </Table>

      <!-- 配置表单弹窗 -->
      <Modal
        v-model:open="configFormVisible"
        :title="configForm.id ? $t('page.finance.salary.config.editTitle') : $t('page.finance.salary.config.createTitle')"
        :confirm-loading="configFormSubmitting"
        @ok="submitConfigForm"
      >
        <Form layout="vertical" class="py-4" autocomplete="off">
          <FormItem :label="$t('page.finance.salary.config.label.employeeId')" required>
            <UserPickerModal v-model:value="configForm.employeeId" :disabled="!!configForm.id" />
          </FormItem>
          <Row :gutter="16">
            <Col :span="12">
              <FormItem :label="$t('page.finance.salary.config.label.year')" required>
                <InputNumber
                  v-model:value="configForm.year"
                  :min="2020"
                  :max="2099"
                  style="width: 100%"
                  name="year"
                  autocomplete="off"
                />
              </FormItem>
            </Col>
            <Col :span="12">
              <FormItem :label="$t('page.finance.salary.config.label.month')">
                <Select
                  v-model:value="configForm.month"
                  :options="monthOptions"
                  allow-clear
                  :placeholder="$t('page.finance.salary.config.placeholder.month')"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
          </Row>
          <FormItem :label="$t('page.finance.salary.config.label.baseSalary')" required>
            <InputNumber
              v-model:value="configForm.baseSalary"
              :min="0"
              :precision="2"
              style="width: 100%"
              prefix="¥"
              name="baseSalary"
              autocomplete="off"
            />
          </FormItem>
          <Row :gutter="16">
            <Col :span="12">
              <FormItem :label="$t('page.finance.salary.config.label.positionAllowance')">
                <InputNumber
                  v-model:value="configForm.positionAllowance"
                  :min="0"
                  :precision="2"
                  style="width: 100%"
                  prefix="¥"
                  name="positionAllowance"
                  autocomplete="off"
                />
              </FormItem>
            </Col>
            <Col :span="12">
              <FormItem :label="$t('page.finance.salary.config.label.performanceBase')">
                <InputNumber
                  v-model:value="configForm.performanceBase"
                  :min="0"
                  :precision="2"
                  style="width: 100%"
                  prefix="¥"
                  name="performanceBase"
                  autocomplete="off"
                />
              </FormItem>
            </Col>
          </Row>
          <FormItem :label="$t('page.finance.salary.config.label.performanceCoefficient')">
            <InputNumber
              v-model:value="configForm.performanceCoefficient"
              :min="0"
              :max="2"
              :step="0.1"
              :precision="2"
              style="width: 100%"
              :placeholder="$t('page.finance.salary.config.placeholder.performanceCoefficient')"
              name="performanceCoefficient"
              autocomplete="off"
            />
          </FormItem>
        </Form>
      </Modal>
    </Drawer>

    <!-- 核算日志抽屉 -->
    <Drawer
      v-model:open="logVisible"
      :title="$t('page.finance.salary.log.title')"
      width="1000px"
      :body-style="{ padding: '16px' }"
    >
      <div class="mb-3 flex items-center gap-2 text-sm text-gray-500">
        <IconifyIcon icon="lucide:info" />
        <span>{{ $t('page.finance.salary.log.info') }}</span>
      </div>
      <Table
        :columns="logColumns"
        :data-source="logList"
        :loading="logLoading"
        row-key="id"
        :pagination="false"
        size="middle"
        :scroll="{ x: 800 }"
      >
        <template #emptyText>
          <Empty :description="$t('page.finance.salary.log.empty')" />
        </template>
      </Table>
    </Drawer>

    <!-- 确认工资弹窗（员工侧） -->
    <Modal
      v-model:open="confirmVisible"
      :title="$t('page.finance.salary.modal.confirmTitle')"
      :confirm-loading="confirmLoading"
      @ok="handleConfirmSalary"
    >
      <div class="py-4">
        <Alert
          type="info"
          show-icon
          :message="$t('page.finance.salary.modal.confirmAlertMessage')"
          class="mb-4"
        />
        <p>{{ $t('page.finance.salary.label.employee') }}{{ confirmTarget?.employeeName }}</p>
        <p>
          {{ $t('page.finance.salary.label.yearMonth') }}{{ $t('page.finance.salary.format.year', { year: confirmTarget?.year }) }}{{ $t('page.finance.salary.format.month', { month: confirmTarget?.month }) }}
        </p>
        <p>{{ $t('page.finance.salary.label.totalSalary') }}{{ formatMoney(confirmTarget?.totalSalary) }}</p>
      </div>
    </Modal>

    <!-- 申请重新核算弹窗（员工侧） -->
    <Modal
      v-model:open="appealVisible"
      :title="$t('page.finance.salary.modal.recalcTitle')"
      :confirm-loading="appealLoading"
      @ok="handleAppealSalary"
    >
      <div class="py-4">
        <Alert
          type="warning"
          show-icon
          :message="$t('page.finance.salary.modal.recalcAlertMessage')"
          class="mb-4"
        />
        <p class="mb-2">{{ $t('page.finance.salary.label.employee') }}{{ appealTarget?.employeeName }}</p>
        <p class="mb-2">
          {{ $t('page.finance.salary.label.yearMonth') }}{{ $t('page.finance.salary.format.year', { year: appealTarget?.year }) }}{{ $t('page.finance.salary.format.month', { month: appealTarget?.month }) }}
        </p>
        <p class="mb-2">
          {{ $t('page.finance.salary.label.totalSalary') }}{{ formatMoney(appealTarget?.totalSalary) }}
        </p>
        <FormItem :label="$t('page.finance.salary.modal.recalcReason')" required>
          <Input.TextArea
            v-model:value="appealForm.reason"
            :rows="4"
            :placeholder="$t('page.finance.salary.modal.recalcReasonPlaceholderLong')"
            :maxlength="500"
            show-count
          />
        </FormItem>
      </div>
    </Modal>

    <!-- 待处理申诉抽屉（财务侧） -->
    <Drawer
      v-model:open="pendingDrawerVisible"
      :title="$t('page.finance.salary.pending.title')"
      width="900px"
      :body-style="{ padding: '16px' }"
    >
      <!-- V8-6: 申诉列表筛选条件 -->
      <Form layout="inline" class="mb-3" :model="pendingFilter">
        <FormItem :label="$t('page.finance.common.employeeId')">
          <UserPickerModal v-model:value="pendingFilter.employeeId" style="width: 160px" />
        </FormItem>
        <FormItem :label="$t('page.finance.common.year')">
          <InputNumber
            v-model:value="pendingFilter.year"
            :min="2020"
            :max="2099"
            style="width: 100px"
            :placeholder="$t('page.finance.common.yearPlaceholder')"
            allow-clear
          />
        </FormItem>
        <FormItem :label="$t('page.finance.common.month')">
          <Select
            v-model:value="pendingFilter.month"
            :options="monthOptions"
            style="width: 100px"
            :placeholder="$t('page.finance.common.monthPlaceholder')"
            allow-clear
          />
        </FormItem>
        <FormItem :label="$t('page.finance.common.status')">
          <Select
            v-model:value="pendingFilter.status"
            :options="pendingStatusOptions"
            style="width: 140px"
            :placeholder="$t('page.finance.common.statusPlaceholder')"
            allow-clear
          />
        </FormItem>
        <FormItem>
          <Button type="primary" @click="loadPendingList">
            {{ $t('page.finance.common.query') }}
          </Button>
        </FormItem>
        <FormItem>
          <Button @click="resetPendingFilter">
            {{ $t('page.finance.common.reset') }}
          </Button>
        </FormItem>
      </Form>
      <Table
        :columns="pendingColumns"
        :data-source="pendingList"
        :loading="pendingLoading"
        row-key="id"
        :pagination="false"
        size="small"
        :scroll="{ x: 700 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'operation'">
            <Button
              type="link"
              size="small"
              @click="handleApproveAppeal(record)"
            >
              {{ $t('page.finance.salary.pending.approveRecalc') }}
            </Button>
            <Button
              type="link"
              danger
              size="small"
              @click="handleRejectAppeal(record)"
            >
              {{ $t('page.finance.salary.pending.reject') }}
            </Button>
          </template>
        </template>
        <template #emptyText>
          <Empty :description="$t('page.finance.salary.pending.empty')" />
        </template>
      </Table>
    </Drawer>
  </Page>
</template>
