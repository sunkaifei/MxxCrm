<script lang="ts" setup>
import type { CustomerEditLogVO } from '#/api/core/crm/customer-edit-log';
import type {
  BankAccount,
  CustomerFinancialVO,
} from '#/api/core/crm/customer-financial';

import { computed, h, reactive, ref, watch } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import {
  LucideBuilding2,
  LucideChevronDown,
  LucideChevronUp,
  LucideFilePenLine,
  LucideGlobe,
  LucideMail,
  LucideMapPin,
  LucideMoreHorizontal,
  LucidePhone,
  LucidePlus,
  LucideUserPlus,
} from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import { useDebounceFn } from '@vueuse/core';
import {
  Avatar,
  Button,
  Card,
  Cascader,
  Col,
  DatePicker,
  Descriptions,
  Divider,
  Drawer,
  Dropdown,
  Empty,
  Form,
  Input,
  InputNumber,
  Menu,
  MenuItem,
  message,
  Modal,
  Pagination,
  Popconfirm,
  Row,
  Select,
  Skeleton,
  Space,
  Spin,
  Table,
  Tabs,
  Tag,
  Timeline,
  Tooltip,
} from 'ant-design-vue';

import {
  checkCustomerNameApi,
  createCustomerApi,
  deleteOpportunityApi,
  getContractListApi,
  getCountriesApi,
  getCustomerAssignHistoryApi,
  getCustomerContactsApi,
  getCustomerInfoApi,
  getCustomerMailLogApi,
  getExpenseListApi,
  getExpenseTypeListApi,
  getOpportunityListApi,
  getOrderListApi,
  getPaymentListApi,
  getRefundListApi,
  updateCustomerApi,
} from '#/api';
import {
  getBackgroundCheckDetailApi,
  getBackgroundCheckTimelineApi,
  getLatestBackgroundCheckByCompanyApi,
  getLatestBackgroundCheckByCompanyIdApi,
  performBackgroundCheckApi,
} from '#/api/core/crm/ai';
import { getCustomerEditLogApi } from '#/api/core/crm/customer-edit-log';
import {
  getCustomerFinancialApi,
  updateCustomerFinancialApi,
} from '#/api/core/crm/customer-financial';
import { addCustomerToPoolApi } from '#/api/core/crm/customer-pool';
import { createFollowupApi } from '#/api/core/crm/followup';
import { requestClient } from '#/api/request';

import SendMailModal from '../components/SendMailModal.vue';
import TagSelector from '../components/TagSelector.vue';
import ContactDrawer from '../contact/drawer.vue';
import OpportunityDetail from '../opportunity/detail.vue';

const props = defineProps<{
  customerType?: number; // 新建模式时传入：1=企业, 2=个人
  id?: number | string;
}>();

const emit = defineEmits<{
  (e: 'created', id: number | string): void;
}>();

function toCamelCase(obj: any): any {
  if (obj === null || obj === undefined) return obj;
  if (Array.isArray(obj)) return obj.map((item) => toCamelCase(item));
  if (typeof obj !== 'object') return obj;
  const result: any = {};
  for (const key of Object.keys(obj)) {
    const camelKey = key.replaceAll(/_([a-z])/g, (_, c) => c.toUpperCase());
    result[camelKey] = toCamelCase(obj[key]);
  }
  return result;
}

function normalizeBgReport(rawReportData: any): any {
  if (!rawReportData || typeof rawReportData !== 'object') return rawReportData;
  const normalized = { ...rawReportData };
  if (rawReportData.company_info && !rawReportData.basic_info) {
    normalized.basic_info = rawReportData.company_info;
  }
  if (
    rawReportData.cooperation_suggestion === null ||
    rawReportData.cooperation_suggestion === undefined
  ) {
    normalized.cooperation_suggestion = {
      suitable: rawReportData.suitable ? '是' : '否',
      notes: rawReportData.notes || '',
      suggestion: rawReportData.summary || '',
    };
  }
  if (
    rawReportData.basic_info &&
    !normalized.basic_info.company_name &&
    rawReportData.company_name
  ) {
    normalized.basic_info.company_name = rawReportData.company_name;
  }
  return normalized;
}

const loading = ref(true);
const customer = ref<any>({});
const contacts = ref<any[]>([]);
const historyContacts = ref<any[]>([]);
const assignHistory = ref<any[]>([]);
const activeTab = ref('basic');
const bgExpanded = ref(false);
const bgLoading = ref(false);
const bgReport = ref<any>(null);
const bgPrevReport = ref<any>(null);
const bgChangedFields = ref<Set<string>>(new Set());
const bgCorrecting = ref(false);
const editLogs = ref<CustomerEditLogVO[]>([]);
const editLogLoading = ref(false);
const financialEditLogs = ref<CustomerEditLogVO[]>([]);
const financialEditLogLoading = ref(false);
const formSaving = ref(false);

// 订单列表
const orderList = ref<any[]>([]);
const orderLoading = ref(false);
const orderPage = ref(1);
const orderPageSize = ref(10);
const orderTotal = ref(0);

const orderStatusMap: Record<number, { color: string; label: string }> = {
  1: { label: '草稿', color: 'default' },
  2: { label: '已确认', color: 'blue' },
  3: { label: '待发货', color: 'gold' },
  4: { label: '部分发货', color: 'orange' },
  5: { label: '已发货', color: 'cyan' },
  6: { label: '已完成', color: 'green' },
  7: { label: '已取消', color: 'red' },
  8: { label: '已关闭', color: 'default' },
  9: { label: '已签收', color: 'green' },
  10: { label: '备货中', color: 'processing' },
};

const paymentStatusMap: Record<number, { color: string; label: string }> = {
  1: { label: '未支付', color: 'red' },
  2: { label: '部分支付', color: 'orange' },
  3: { label: '已支付', color: 'green' },
  4: { label: '已退款', color: 'default' },
};

function getOrderStatusInfo(status: number) {
  return orderStatusMap[status] || { label: '未知', color: 'default' };
}

function getPaymentStatusInfo(status: number) {
  return paymentStatusMap[status] || { label: '未知', color: 'default' };
}

function formatMoney(val: any): string {
  const num = Number(val) || 0;
  return `¥${num.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

async function loadOrderList() {
  if (!props.id) return;
  orderLoading.value = true;
  try {
    const result: any = await getOrderListApi({
      customerId: Number(props.id),
      pageNum: orderPage.value,
      pageSize: orderPageSize.value,
    } as any);
    const data = result?.data ?? result ?? {};
    orderList.value = data?.list ?? data?.items ?? [];
    orderTotal.value = data?.total ?? 0;
  } catch {
    orderList.value = [];
    orderTotal.value = 0;
  } finally {
    orderLoading.value = false;
  }
}

function handleOrderPageChange(page: number, pageSize: number) {
  orderPage.value = page;
  orderPageSize.value = pageSize;
  loadOrderList();
}

// 合同列表
const contractList = ref<any[]>([]);
const contractLoading = ref(false);
const contractPage = ref(1);
const contractPageSize = ref(10);
const contractTotal = ref(0);

// 合同状态映射：1=草稿, 2=已签订, 3=执行中, 4=已完成, 5=已终止
const contractStatusMap: Record<number, { color: string; label: string }> = {
  1: { label: '草稿', color: 'default' },
  2: { label: '已签订', color: 'blue' },
  3: { label: '执行中', color: 'processing' },
  4: { label: '已完成', color: 'green' },
  5: { label: '已终止', color: 'red' },
};

// 审批状态映射：0=草稿, 1=待审批, 2=审批中, 3=已通过, 4=已驳回
const contractApprovalStatusMap: Record<
  number,
  { color: string; label: string }
> = {
  0: { label: '草稿', color: 'default' },
  1: { label: '待审批', color: 'gold' },
  2: { label: '审批中', color: 'processing' },
  3: { label: '已通过', color: 'green' },
  4: { label: '已驳回', color: 'red' },
};

function getContractStatusInfo(status: any) {
  const num = Number(status);
  if (!Number.isNaN(num))
    return contractStatusMap[num] || { label: '未知', color: 'default' };
  // 兼容后端枚举字符串
  const labelMap: Record<string, { color: string; label: string }> = {
    Draft: { label: '草稿', color: 'default' },
    Signed: { label: '已签订', color: 'blue' },
    Executing: { label: '执行中', color: 'processing' },
    Completed: { label: '已完成', color: 'green' },
    Terminated: { label: '已终止', color: 'red' },
  };
  return (
    labelMap[String(status)] || {
      label: String(status ?? '未知'),
      color: 'default',
    }
  );
}

function getContractApprovalStatusInfo(status: number) {
  return (
    contractApprovalStatusMap[status] || { label: '未知', color: 'default' }
  );
}

function formatDate(val: any): string {
  if (!val) return '—';
  const s = String(val);
  return s.length >= 10 ? s.slice(0, 10) : s;
}

async function loadContractList() {
  if (!props.id) return;
  contractLoading.value = true;
  try {
    const result: any = await getContractListApi({
      customerId: Number(props.id),
      page: contractPage.value,
      pageSize: contractPageSize.value,
    } as any);
    const data = result?.data ?? result ?? {};
    contractList.value = data?.list ?? data?.items ?? [];
    contractTotal.value = data?.total ?? 0;
  } catch {
    contractList.value = [];
    contractTotal.value = 0;
  } finally {
    contractLoading.value = false;
  }
}

function handleContractPageChange(page: number, pageSize: number) {
  contractPage.value = page;
  contractPageSize.value = pageSize;
  loadContractList();
}

// ========== 回款列表 ==========
const paymentList = ref<any[]>([]);
const paymentLoading = ref(false);
const paymentPage = ref(1);
const paymentPageSize = ref(10);
const paymentTotal = ref(0);

// 回款付款方式：1=银行转账, 2=支付宝, 3=微信支付, 4=现金, 5=支票, 6=其他
const paymentMethodMap: Record<number, { color: string; label: string }> = {
  1: { label: '银行转账', color: 'blue' },
  2: { label: '支付宝', color: 'cyan' },
  3: { label: '微信支付', color: 'green' },
  4: { label: '现金', color: 'orange' },
  5: { label: '支票', color: 'purple' },
  6: { label: '其他', color: 'default' },
};

// 回款状态：1=待确认, 2=已确认, 3=已驳回, 4=已取消
const paymentConfirmStatusMap: Record<
  number,
  { color: string; label: string }
> = {
  1: { label: '待确认', color: 'default' },
  2: { label: '已确认', color: 'green' },
  3: { label: '已驳回', color: 'red' },
  4: { label: '已取消', color: 'default' },
};

function getPaymentMethodInfo(v: number) {
  return paymentMethodMap[v] || { label: '未知', color: 'default' };
}
function getPaymentConfirmStatusInfo(v: number) {
  return paymentConfirmStatusMap[v] || { label: '未知', color: 'default' };
}

async function loadPaymentList() {
  if (!props.id) return;
  paymentLoading.value = true;
  try {
    const result: any = await getPaymentListApi({
      customerId: Number(props.id),
      page: paymentPage.value,
      pageSize: paymentPageSize.value,
    } as any);
    const data = result?.data ?? result ?? {};
    paymentList.value = data?.list ?? data?.items ?? [];
    paymentTotal.value = data?.total ?? 0;
  } catch {
    paymentList.value = [];
    paymentTotal.value = 0;
  } finally {
    paymentLoading.value = false;
  }
}

function handlePaymentPageChange(page: number, pageSize: number) {
  paymentPage.value = page;
  paymentPageSize.value = pageSize;
  loadPaymentList();
}

// ========== 退货记录列表 ==========
const refundList = ref<any[]>([]);
const refundLoading = ref(false);
const refundPage = ref(1);
const refundPageSize = ref(10);
const refundTotal = ref(0);

// 退货状态：1=草稿,2=待审批,3=审批通过,4=待收货,5=已收货,6=质检中,7=已完成,8=已驳回,9=已取消
const refundStatusMap: Record<number, { color: string; label: string }> = {
  1: { label: '草稿', color: 'default' },
  2: { label: '待审批', color: 'processing' },
  3: { label: '审批通过', color: 'success' },
  4: { label: '待收货', color: 'warning' },
  5: { label: '已收货', color: 'cyan' },
  6: { label: '质检中', color: 'orange' },
  7: { label: '已完成', color: 'green' },
  8: { label: '已驳回', color: 'error' },
  9: { label: '已取消', color: 'default' },
};

// 审批状态：0=草稿,1=待审批,2=审批中,3=已通过,4=已驳回
const refundApprovalStatusMap: Record<
  number,
  { color: string; label: string }
> = {
  0: { label: '草稿', color: 'default' },
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
};

// 退货类型：1=整单退货, 2=部分退货
const refundTypeMap: Record<number, { color: string; label: string }> = {
  1: { label: '整单退货', color: 'orange' },
  2: { label: '部分退货', color: 'blue' },
};

function getRefundStatusInfo(v: number) {
  return refundStatusMap[v] || { label: '未知', color: 'default' };
}
function getRefundApprovalStatusInfo(v: number) {
  return refundApprovalStatusMap[v] || { label: '未知', color: 'default' };
}
function getRefundTypeInfo(v: number) {
  return refundTypeMap[v] || { label: '未知', color: 'default' };
}

async function loadRefundList() {
  if (!props.id) return;
  refundLoading.value = true;
  try {
    const result: any = await getRefundListApi({
      customerId: Number(props.id),
      pageNum: refundPage.value,
      pageSize: refundPageSize.value,
    });
    const data = result?.data ?? result ?? {};
    refundList.value = data?.list ?? data?.items ?? [];
    refundTotal.value = data?.total ?? 0;
  } catch {
    refundList.value = [];
    refundTotal.value = 0;
  } finally {
    refundLoading.value = false;
  }
}

function handleRefundPageChange(page: number, pageSize: number) {
  refundPage.value = page;
  refundPageSize.value = pageSize;
  loadRefundList();
}

// ========== 费用记录列表 ==========
const expenseList = ref<any[]>([]);
const expenseLoading = ref(false);
const expensePage = ref(1);
const expensePageSize = ref(10);
const expenseTotal = ref(0);
const expenseTypeMap = ref<Record<number, { color: string; name: string }>>({});

// 费用状态：1=草稿,2=待审批,3=审批中,4=已通过,5=已驳回,6=已打款
const expenseStatusMap: Record<number, { color: string; label: string }> = {
  1: { label: '草稿', color: 'default' },
  2: { label: '待审批', color: 'processing' },
  3: { label: '审批中', color: 'warning' },
  4: { label: '已通过', color: 'success' },
  5: { label: '已驳回', color: 'error' },
  6: { label: '已打款', color: 'green' },
};

function getExpenseStatusInfo(v: number) {
  return expenseStatusMap[v] || { label: '未知', color: 'default' };
}

async function loadExpenseTypes() {
  if (Object.keys(expenseTypeMap.value).length > 0) return;
  try {
    const res: any = await getExpenseTypeListApi({
      page: 1,
      pageSize: 100,
      enabled: 1,
    });
    const data = res?.data ?? res ?? {};
    const list = data.list || data.items || data.rows || data || [];
    const arr = Array.isArray(list) ? list : [];
    const map: Record<number, { color: string; name: string }> = {};
    arr.forEach((t: any) => {
      map[t.id] = {
        name: t.typeName || t.name || '',
        color: t.color || 'blue',
      };
    });
    expenseTypeMap.value = map;
  } catch {
    expenseTypeMap.value = {};
  }
}

async function loadExpenseList() {
  if (!props.id) return;
  expenseLoading.value = true;
  try {
    await loadExpenseTypes();
    const result: any = await getExpenseListApi({
      customerId: Number(props.id),
      pageNum: expensePage.value,
      pageSize: expensePageSize.value,
    });
    const data = result?.data ?? result ?? {};
    expenseList.value = data?.list ?? data?.items ?? [];
    expenseTotal.value = data?.total ?? 0;
  } catch {
    expenseList.value = [];
    expenseTotal.value = 0;
  } finally {
    expenseLoading.value = false;
  }
}

function handleExpensePageChange(page: number, pageSize: number) {
  expensePage.value = page;
  expensePageSize.value = pageSize;
  loadExpenseList();
}

// 三个只读列表的表格列定义
const paymentColumns = [
  { title: '回款编号', dataIndex: 'paymentNo', width: 180, ellipsis: true },
  { title: '回款日期', dataIndex: 'paymentDate', width: 120 },
  {
    title: '回款金额',
    dataIndex: 'amount',
    width: 130,
    align: 'right' as const,
  },
  {
    title: '已核销金额',
    dataIndex: 'appliedAmount',
    width: 130,
    align: 'right' as const,
  },
  { title: '付款方式', dataIndex: 'paymentMethod', width: 110 },
  { title: '状态', dataIndex: 'status', width: 100 },
  { title: '确认时间', dataIndex: 'confirmTime', width: 170 },
];

const refundColumns = [
  { title: '退货编号', dataIndex: 'refundNo', width: 170, ellipsis: true },
  { title: '标题', dataIndex: 'title', width: 200, ellipsis: true },
  { title: '退货类型', dataIndex: 'refundType', width: 100 },
  {
    title: '退货金额',
    dataIndex: 'refundAmount',
    width: 120,
    align: 'right' as const,
  },
  {
    title: '退款金额',
    dataIndex: 'refundedAmount',
    width: 120,
    align: 'right' as const,
  },
  { title: '状态', dataIndex: 'refundStatus', width: 100 },
  { title: '审批状态', dataIndex: 'approvalStatus', width: 100 },
  { title: '创建时间', dataIndex: 'createTime', width: 170 },
];

const expenseColumns = [
  { title: '费用编号', dataIndex: 'expenseNo', width: 170, ellipsis: true },
  { title: '标题', dataIndex: 'title', width: 200, ellipsis: true },
  { title: '费用类型', dataIndex: 'expenseType', width: 120 },
  {
    title: '金额',
    dataIndex: 'totalAmount',
    width: 130,
    align: 'right' as const,
  },
  { title: '申请人', dataIndex: 'applicantName', width: 100 },
  { title: '状态', dataIndex: 'status', width: 100 },
  { title: '申请日期', dataIndex: 'applyDate', width: 120 },
];

const [ContactEditDrawer, contactEditDrawerApi] = useVbenDrawer({
  connectedComponent: ContactDrawer,
  onClosed() {
    if (contactEditDrawerApi.getData()?.needRefresh) loadContacts();
  },
});

// 商机详情抽屉（新建/编辑共用 OpportunityDetail 组件）
const oppDetailVisible = ref(false);
const oppDetailId = ref<number | string | undefined>(undefined);
const oppDetailCustomerId = ref<number | string | undefined>(undefined);
const oppDetailCustomerName = ref<string>('');
const oppDetailTitle = computed(() =>
  oppDetailId.value ? '编辑商机' : '新建商机',
);

function closeOppDetail() {
  oppDetailVisible.value = false;
  oppDetailId.value = undefined;
  oppDetailCustomerId.value = undefined;
  oppDetailCustomerName.value = '';
  loadOpportunities();
}

const levelColor = computed(() => {
  const map: Record<string, string> = {
    1: 'default',
    2: 'red',
    3: 'orange',
    4: 'blue',
    5: 'green',
  };
  return map[customer.value.level] || 'blue';
});

const levelLabel = computed(() => {
  const map: Record<string, string> = {
    1: '无级别',
    2: '重点客户',
    3: '优质客户',
    4: '普通客户',
    5: '其他',
  };
  return map[customer.value.level] || customer.value.level || '-';
});

// 客户类型：1=企业 2=个人
const isPersonal = computed(() => Number(customer.value?.customerType) === 2);
const displayName = computed(() => {
  if (isPersonal.value) {
    return customer.value?.personName || customer.value?.companyName || '-';
  }
  return customer.value?.companyName || customer.value?.personName || '-';
});

// 新建模式：无 id 时为新建
const isCreate = computed(() => !props.id);

// 名称查重错误持久化（防止 Zod 通过后被清除）
const nameDuplicateError = ref<string | undefined>(undefined);

const initials = computed(() => {
  const name = displayName.value || customer.value?.shortName || '?';
  return name.slice(0, 2).toUpperCase();
});

const roleLabel: Record<number, string> = {
  0: '决策人',
  1: '影响者',
  2: '使用者',
  3: '其他',
};
const roleColor: Record<number, string> = {
  0: 'red',
  1: 'blue',
  2: 'green',
  3: 'default',
};

const industryLabelMap: Record<number, string> = {
  1: '零售',
  2: '批发',
  3: '制造',
  4: '贸易代理',
  5: '电商',
  6: '微商',
  7: '社交电商',
  8: '其他',
};

const sourceLabelMap: Record<number, string> = {
  1: '官网',
  2: '展会',
  3: '社交媒体',
  4: '客户转介',
  5: '陌生拜访',
  6: '海关数据',
  7: '邮件营销',
  8: '阿里国际站',
  9: 'Amazon',
  10: 'TikTok',
  11: '微信',
  12: '其他',
};

const levelLabelMap: Record<number, string> = {
  1: '无级别',
  2: '重点客户',
  3: '优质客户',
  4: '普通客户',
  5: '其他',
};
const currencyLabelMap: Record<number, string> = {
  1: '人民币',
  2: '美元',
  3: '欧元',
  4: '英镑',
  5: '日元',
  6: '港币',
  7: '澳元',
};

const countryOptions = ref<{ label: string; value: string }[]>([]);

// 中国省市区三级联动数据（Cascader 格式）
const chinaAreaOptions = ref<any[]>([]);
// Cascader 选中路径（数组形式：[省, 市, 区]）
const regionPath = ref<string[]>([]);

// 是否中国：country 字段存的是国家名称
const isChina = computed(() => {
  const c = (form.country || '').trim();
  return c === '中国' || c === 'China' || c === 'CN';
});

// 将"省/市/区"文本回填到 regionPath（编辑模式回显）
function syncRegionPathFromText() {
  if (!form.region) {
    regionPath.value = [];
    return;
  }
  // 兼容 " / "、"/" 分隔符
  regionPath.value = form.region.split(/\s*\/\s*/).filter(Boolean);
}

// Cascader 选中变化时同步到 form.region（存为"省/市/区"文本）
function handleRegionCascaderChange(value: any) {
  form.region = Array.isArray(value) && value.length > 0 ? value.join('/') : '';
}

// 加载中国省市区三级联动数据
// 使用 /api/system/region/treelist 接口（数据量小，响应快）
async function loadChinaArea() {
  if (chinaAreaOptions.value.length > 0) return; // 已加载则跳过
  try {
    const result = await requestClient.get('/api/system/region/treelist');
    const tree = Array.isArray(result) ? result : ((result as any)?.data ?? []);
    // RegionTreeVO { id, parentId, title, regionName, sort, status, children }
    // → Cascader { value, label, children }
    chinaAreaOptions.value = convertRegionTreeToCascader(tree);
  } catch {
    /* ignore */
  }
}

// 将 region 树转为 Cascader 选项（value/label 用 regionName，便于回填"省/市/区"文本）
function convertRegionTreeToCascader(nodes: any[]): any[] {
  if (!Array.isArray(nodes)) return [];
  return nodes
    .filter((n) => n && (n.regionName || n.title))
    .map((n) => {
      const label = n.regionName || n.title || '';
      const node: any = { value: label, label };
      if (Array.isArray(n.children) && n.children.length > 0) {
        node.children = convertRegionTreeToCascader(n.children);
      }
      return node;
    });
}

// 国家选择变化时处理 region 字段
function handleCountryChange() {
  // 国家切换后清空省/州，避免与新国家不匹配
  form.region = '';
  regionPath.value = [];
  if (isChina.value) {
    loadChinaArea();
  }
}

const industryOptions = [
  { label: '零售', value: 1 },
  { label: '批发', value: 2 },
  { label: '制造', value: 3 },
  { label: '贸易代理', value: 4 },
  { label: '电商', value: 5 },
  { label: '微商', value: 6 },
  { label: '社交电商', value: 7 },
  { label: '其他', value: 8 },
];

const sourceOptions = [
  { label: '官网', value: 1 },
  { label: '展会', value: 2 },
  { label: '社交媒体', value: 3 },
  { label: '客户转介', value: 4 },
  { label: '陌生拜访', value: 5 },
  { label: '海关数据', value: 6 },
  { label: '邮件营销', value: 7 },
  { label: '阿里国际站', value: 8 },
  { label: 'Amazon', value: 9 },
  { label: 'TikTok', value: 10 },
  { label: '微信', value: 11 },
  { label: '其他', value: 12 },
];

const levelOptions = [
  { label: '无级别', value: 1 },
  { label: '重点客户', value: 2 },
  { label: '优质客户', value: 3 },
  { label: '普通客户', value: 4 },
  { label: '其他', value: 5 },
];

const currencyOptions = [
  { label: '人民币', value: 1 },
  { label: '美元', value: 2 },
  { label: '欧元', value: 3 },
  { label: '英镑', value: 4 },
  { label: '日元', value: 5 },
  { label: '港币', value: 6 },
  { label: '澳元', value: 7 },
];

// 个人客户相关选项
const genderOptions = [
  { label: '男', value: 1 },
  { label: '女', value: 2 },
  { label: '未知', value: 0 },
];

const form = reactive({
  customerType: 1 as number,
  companyName: '',
  shortName: '',
  customerNo: '',
  level: undefined as number | undefined,
  source: undefined as number | undefined,
  industry: undefined as number | undefined,
  country: '',
  region: '',
  address: '',
  website: '',
  currency: 1,
  creditLimit: undefined as number | undefined,
  creditDays: undefined as number | undefined,
  cooperatedAt: undefined as string | undefined,
  description: '',
  // 个人客户字段
  personName: '',
  gender: undefined as number | undefined,
  birthday: undefined as string | undefined,
  wechat: '',
  qq: '',
  personalMobile: '',
  personalEmail: '',
  nickname: '',
  occupation: '',
});

function fillFormFromCustomer() {
  form.customerType = Number(customer.value.customerType) || 1;
  form.companyName = customer.value.companyName || '';
  form.shortName = customer.value.shortName || '';
  form.customerNo = customer.value.customerNo || '';
  form.level =
    customer.value.level === null || customer.value.level === undefined
      ? undefined
      : Number(customer.value.level);
  form.source =
    customer.value.source === null || customer.value.source === undefined
      ? undefined
      : Number(customer.value.source);
  form.industry =
    customer.value.industry === null || customer.value.industry === undefined
      ? undefined
      : Number(customer.value.industry);
  form.country = customer.value.country || '';
  form.region = customer.value.region || '';
  // 回显 Cascader 选中路径（中国时生效）
  syncRegionPathFromText();
  // 编辑模式若为国家为中国，预加载中国省市区数据
  if (isChina.value) {
    loadChinaArea();
  }
  form.address = customer.value.address || '';
  form.website = customer.value.website || '';
  form.currency = customer.value.currency || 1;
  form.creditLimit = customer.value.creditLimit;
  form.creditDays = customer.value.creditDays;
  form.cooperatedAt = customer.value.cooperatedAt;
  form.description = customer.value.description || '';
  // 个人客户字段
  form.personName = customer.value.personName || '';
  form.gender = customer.value.gender;
  form.birthday = customer.value.birthday;
  form.wechat = customer.value.wechat || '';
  form.qq = customer.value.qq || '';
  form.personalMobile = customer.value.personalMobile || '';
  form.personalEmail = customer.value.personalEmail || '';
  form.nickname = customer.value.nickname || '';
  form.occupation = customer.value.occupation || '';
}

function getFieldValueLabel(
  field: string,
  value: null | string | undefined,
): string {
  if (value === null || value === undefined || value === '') return '';
  const numVal = Number(value);
  if (field === 'level') return levelLabelMap[numVal] || value;
  if (field === 'industry') return industryLabelMap[numVal] || value;
  if (field === 'source') return sourceLabelMap[numVal] || value;
  if (field === 'currency') return currencyLabelMap[numVal] || value;
  return value;
}

const statCards = computed(() => [
  {
    label: '成交总额',
    value: `¥${((customer.value.stats?.totalRevenue ?? 0) / 10_000).toFixed(1)}万`,
    color: 'text-blue-600',
    bg: 'bg-blue-50',
  },
  {
    label: '成交笔数',
    value: customer.value.stats?.orderCount ?? 0,
    color: 'text-green-600',
    bg: 'bg-green-50',
  },
  {
    label: '联系人',
    value: contacts.value.length,
    color: 'text-purple-600',
    bg: 'bg-purple-50',
  },
  {
    label: '商机数',
    value: oppPagination.total ?? 0,
    color: 'text-orange-600',
    bg: 'bg-orange-50',
  },
  {
    label: '信用额度',
    value: `¥${((customer.value.creditLimit ?? 0) / 10_000).toFixed(1)}万`,
    color: 'text-red-500',
    bg: 'bg-red-50',
  },
  {
    label: '账期',
    value: `${customer.value.creditDays ?? 0}天`,
    color: 'text-cyan-600',
    bg: 'bg-cyan-50',
  },
]);

function getRiskClass(score: number | undefined): string {
  if (!score && score !== 0) return 'risk-default';
  if (score <= 30) return 'risk-high';
  if (score <= 50) return 'risk-medium';
  if (score <= 70) return 'risk-low';
  return 'risk-safe';
}

function getRiskLevelByScore(score: number): string {
  if (score <= 30) return '高风险';
  if (score <= 50) return '中风险';
  if (score <= 70) return '低风险';
  return '安全';
}

function getRiskTagColor(level: string | undefined): string {
  if (!level) return 'default';
  if (level.includes('高')) return 'red';
  if (level.includes('中')) return 'orange';
  if (level.includes('低')) return 'green';
  return 'blue';
}

const correctCompanyName = computed(() => {
  return (
    bgReport.value?.reportData?.basic_info?.company_name ||
    bgReport.value?.reportData?.company_info?.company_name ||
    bgReport.value?.companyName ||
    ''
  );
});

async function fetchBackgroundCheck() {
  if (!customer.value.companyName && !props.id) return;
  bgLoading.value = true;
  bgPrevReport.value = null;
  bgChangedFields.value = new Set();
  try {
    let res: any = null;
    if (props.id) {
      try {
        res = await getLatestBackgroundCheckByCompanyIdApi(Number(props.id));
      } catch {
        res = null;
      }
    }
    if (!res && customer.value.companyName) {
      res = await getLatestBackgroundCheckByCompanyApi(
        customer.value.companyName,
      );
    }
    if (res) {
      const rawReportData = res.reportData || res.report_data;
      bgReport.value = toCamelCase(res);
      if (bgReport.value && rawReportData) {
        bgReport.value.reportData = normalizeBgReport(rawReportData);
        if (!bgReport.value.riskScore) {
          bgReport.value.riskScore =
            rawReportData.risk_score ||
            rawReportData.riskScore ||
            rawReportData.risk_assessment?.risk_score ||
            rawReportData.riskAssessment?.riskScore ||
            50;
        }
        if (!bgReport.value.riskLevel) {
          const score = bgReport.value.riskScore;
          bgReport.value.riskLevel =
            rawReportData.risk_level ||
            rawReportData.riskLevel ||
            rawReportData.risk_assessment?.risk_level ||
            rawReportData.riskAssessment?.riskLevel ||
            getRiskLevelByScore(score);
        }
      }
      // 获取历史版本对比差异
      await fetchBgCompare(
        customer.value.companyName || bgReport.value.companyName || '',
      );
    } else {
      bgReport.value = null;
    }
  } catch (error) {
    console.error('[客户背调] 加载失败:', error);
    bgReport.value = null;
  } finally {
    bgLoading.value = false;
  }
}

async function fetchBgCompare(companyName: string) {
  if (!companyName) return;
  try {
    const historyResp: any = await getBackgroundCheckTimelineApi(companyName);
    const items: any[] = historyResp?.data ?? historyResp ?? [];
    if (items.length <= 1 || !bgReport.value?.id) return;
    const sorted = items.toSorted(
      (a: any, b: any) =>
        new Date(b.createdAt || 0).getTime() -
        new Date(a.createdAt || 0).getTime(),
    );
    const curIdx = sorted.findIndex((x: any) => x.id === bgReport.value.id);
    if (curIdx === -1 || curIdx >= sorted.length - 1) return;
    // 取上一个版本的详情
    const prevResp: any = await getBackgroundCheckDetailApi(
      sorted[curIdx + 1].id,
    );
    const prevData = prevResp?.data ?? prevResp ?? null;
    if (!prevData?.reportData) return;
    const prevNorm = normalizeBgReport(prevData.reportData);
    bgPrevReport.value = prevNorm;
    // 逐字段比较
    const changes = new Set<string>();
    deepCompareFields(bgReport.value.reportData, prevNorm, changes);
    bgChangedFields.value = changes;
  } catch {
    /* optional */
  }
}

function deepCompareFields(
  latest: any,
  prev: any,
  changes: Set<string>,
  prefix = '',
) {
  if (!latest || !prev) return;
  for (const key of Object.keys(latest)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    const lv = latest[key];
    const pv = prev[key];
    if (
      typeof lv === 'object' &&
      lv !== null &&
      typeof pv === 'object' &&
      pv !== null &&
      !Array.isArray(lv)
    ) {
      deepCompareFields(lv, pv, changes, fullKey);
    } else if (JSON.stringify(lv) !== JSON.stringify(pv)) {
      changes.add(fullKey);
    }
  }
}

async function handleRunBackgroundCheck() {
  if (!customer.value.companyName?.trim()) {
    message.error('客户名称为空，无法进行背调');
    return;
  }
  bgLoading.value = true;
  try {
    const res = await performBackgroundCheckApi({
      company_name: customer.value.companyName,
      company_id: Number(props.id),
    });
    if (res) {
      const rawReportData = res.reportData || res.report_data;
      bgReport.value = toCamelCase(res);
      if (bgReport.value && rawReportData) {
        bgReport.value.reportData = normalizeBgReport(rawReportData);
        if (!bgReport.value.riskScore) {
          bgReport.value.riskScore =
            rawReportData.risk_score || rawReportData.riskScore || 50;
        }
        if (!bgReport.value.riskLevel) {
          const score = bgReport.value.riskScore;
          bgReport.value.riskLevel =
            rawReportData.risk_level ||
            rawReportData.riskLevel ||
            getRiskLevelByScore(score);
        }
      }
      message.success('企业背调完成');
      bgExpanded.value = true;
    }
  } catch (error: any) {
    const msg = error?.message || error?.msg || '评估失败，请检查API配置';
    message.error(msg);
  } finally {
    bgLoading.value = false;
  }
}

async function handleCorrectCompanyName() {
  const correctName = correctCompanyName.value;
  if (!correctName?.trim()) {
    message.warning('背调报告中未找到工商注册名称');
    return;
  }
  if (correctName.trim() === form.companyName?.trim()) {
    message.info('公司名称已是最新');
    return;
  }
  bgCorrecting.value = true;
  try {
    const payload = {
      ...form,
      companyName: correctName.trim(),
      id: Number(props.id),
    };
    await updateCustomerApi(payload);
    form.companyName = correctName.trim();
    customer.value.companyName = correctName.trim();
    message.success('公司名称已更正');
  } catch {
    // 全局拦截处理
  } finally {
    bgCorrecting.value = false;
  }
}

const loadData = async () => {
  // 新建模式：初始化空数据 + 按 prop 设置客户类型 + 加载国家选项
  if (!props.id) {
    const initType = Number(props.customerType) === 2 ? 2 : 1;
    customer.value = { customerType: initType };
    form.customerType = initType;
    loading.value = false;
    await loadCountries();
    return;
  }
  loading.value = true;
  try {
    const result = await getCustomerInfoApi(Number(props.id));
    customer.value = result || {};
    fillFormFromCustomer();
    await Promise.all([
      loadContacts(),
      loadAssignHistory(),
      fetchBackgroundCheck(),
      loadCountries(),
      loadEditLogs(),
      loadOpportunities(),
    ]);
  } finally {
    loading.value = false;
  }
};

async function loadCountries() {
  try {
    const result = await getCountriesApi();
    const items = Array.isArray(result) ? result : [];
    countryOptions.value = items.map((item: any) => ({
      label: item.name,
      value: item.name,
    }));
  } catch {
    /* ignore */
  }
}

const loadAssignHistory = async () => {
  if (!props.id) return;
  try {
    const result = await getCustomerAssignHistoryApi(Number(props.id));
    const rawList = Array.isArray(result) ? result : result?.data;
    const list = Array.isArray(rawList) ? rawList : [];
    assignHistory.value = list;
  } catch {
    /* ignore */
  }
};

const loadContacts = async () => {
  if (!props.id) return;
  try {
    const result = await getCustomerContactsApi(Number(props.id));
    contacts.value = result.current || [];
    historyContacts.value = result.history || [];
  } catch {
    /* ignore */
  }
};

const handleAddContact = () => {
  contactEditDrawerApi.setData({ create: true, customerId: Number(props.id) });
  contactEditDrawerApi.open();
};

const handleViewContact = (_contactId: number) => {
  // TODO: 跳转联系人详情
};

const handleUnbind = async (_contactId: number) => {
  message.success('解绑成功');
  loadContacts();
};

// 切换到基本信息 Tab（替代原"编辑"按钮，基本信息 Tab 内表单可直接编辑）
const handleEdit = () => {
  activeTab.value = 'basic';
};

// 发邮件
const sendMailVisible = ref(false);
const handleSendMail = () => {
  sendMailVisible.value = true;
};

// 邮件记录
const mailLogs = ref<any[]>([]);
const mailLogsLoading = ref(false);
async function loadMailLogs() {
  if (!props.id) return;
  mailLogsLoading.value = true;
  try {
    const res: any = await getCustomerMailLogApi(Number(props.id));
    if (Array.isArray(res)) {
      mailLogs.value = res;
    } else if (res && Array.isArray(res.items)) {
      mailLogs.value = res.items;
    } else {
      mailLogs.value = [];
    }
  } catch {
    mailLogs.value = [];
  } finally {
    mailLogsLoading.value = false;
  }
}

const handleReturnToPool = () => {
  Modal.confirm({
    title: '退回公海',
    content: `确定将客户"${customer.value.companyName}"退回公海吗？`,
    onOk: async () => {
      try {
        await addCustomerToPoolApi(Number(props.id));
        message.success('已退回公海');
        loadData();
      } catch {
        message.error('退回公海失败');
      }
    },
  });
};

const followups = computed(() => customer.value?.followups || []);

const sortedFollowupRecords = computed(() =>
  followups.value.toSorted(
    (a: any, b: any) =>
      new Date(b.createTime).getTime() - new Date(a.createTime).getTime(),
  ),
);

const followMethodOptions = [
  { label: '电话', value: 1, color: '#1890ff' },
  { label: '拜访', value: 2, color: '#13c2c2' },
  { label: '邮件', value: 3, color: '#722ed1' },
  { label: '会议', value: 4, color: '#fa8c16' },
  { label: 'WhatsApp', value: 5, color: '#25b864' },
  { label: '微信', value: 6, color: '#52c41a' },
  { label: '其他', value: 7, color: '#8c8c8c' },
];

const followupForm = reactive({
  content: '',
  nextFollowAt: undefined as any,
  status: 2,
  method: 1,
});
const followupSaving = ref(false);

function getMethodOption(value: any) {
  return followMethodOptions.find((o) => o.value === value);
}

async function handleSaveFollowup() {
  if (!followupForm.content.trim()) {
    message.warning('请填写跟进内容');
    return;
  }
  if (!props.id) {
    message.warning('客户ID为空');
    return;
  }
  followupSaving.value = true;
  try {
    await createFollowupApi({
      customerId: Number(props.id),
      content: followupForm.content,
      nextFollowDate: followupForm.nextFollowAt,
      activityType: Number(followupForm.method),
    });
    message.success('跟进记录已保存');
    followupForm.content = '';
    followupForm.nextFollowAt = undefined;
    followupForm.status = 2;
    followupForm.method = 1;
    await loadData();
  } catch {
    // 全局拦截器处理
  } finally {
    followupSaving.value = false;
  }
}

const loadEditLogs = async () => {
  if (!props.id) return;
  editLogLoading.value = true;
  try {
    // 不过滤 logType，加载全部日志（基本信息 logType=0 + 转移日志 logType=2）
    const result = await getCustomerEditLogApi({
      customerId: Number(props.id),
      page: 1,
      pageSize: 50,
    });
    editLogs.value = (result as any)?.items || [];
  } catch {
    /* ignore */
  } finally {
    editLogLoading.value = false;
  }
};

const loadFinancialEditLogs = async () => {
  if (!props.id) return;
  financialEditLogLoading.value = true;
  try {
    const result = await getCustomerEditLogApi({
      customerId: Number(props.id),
      page: 1,
      pageSize: 50,
      logType: 1,
    });
    financialEditLogs.value = (result as any)?.items || [];
  } catch {
    /* ignore */
  } finally {
    financialEditLogLoading.value = false;
  }
};

// 防抖查重（提交时使用）
const checkNameDebounced = useDebounceFn(
  async (val: string, type: 1 | 2, excludeId?: number): Promise<boolean> => {
    try {
      const res = await checkCustomerNameApi({
        customerType: type,
        name: val,
        excludeId,
      });
      return !!(res as any)?.exists;
    } catch {
      return false;
    }
  },
  400,
);

// 公司名称输入时实时查重（防抖）
const checkCompanyNameOnInput = useDebounceFn(async (val: string) => {
  if (!val?.trim()) {
    nameDuplicateError.value = undefined;
    return;
  }
  try {
    const res = await checkCustomerNameApi({
      customerType: 1,
      name: val,
      excludeId: props.id ? Number(props.id) : undefined,
    });
    nameDuplicateError.value = (res as any)?.exists
      ? '该公司名称已存在'
      : undefined;
  } catch {
    /* ignore */
  }
}, 500);

// 个人姓名输入时实时查重（防抖）
const checkPersonNameOnInput = useDebounceFn(async (val: string) => {
  if (!val?.trim()) {
    nameDuplicateError.value = undefined;
    return;
  }
  try {
    const res = await checkCustomerNameApi({
      customerType: 2,
      name: val,
      excludeId: props.id ? Number(props.id) : undefined,
    });
    nameDuplicateError.value = (res as any)?.exists
      ? '该个人姓名已存在'
      : undefined;
  } catch {
    /* ignore */
  }
}, 500);

async function handleSaveForm() {
  if (isPersonal.value) {
    if (!form.personName?.trim()) {
      message.error('请输入姓名');
      return;
    }
    // 个人姓名查重
    const excludeId = props.id ? Number(props.id) : undefined;
    const exists = await checkNameDebounced(
      form.personName.trim(),
      2,
      excludeId,
    );
    if (exists) {
      message.error('该个人姓名已存在');
      return;
    }
  } else {
    if (!form.companyName?.trim()) {
      message.error('请输入公司名称');
      return;
    }
    // 公司名称查重
    const excludeId = props.id ? Number(props.id) : undefined;
    const exists = await checkNameDebounced(
      form.companyName.trim(),
      1,
      excludeId,
    );
    if (exists) {
      message.error('该公司名称已存在');
      return;
    }
  }
  formSaving.value = true;
  try {
    const payload = {
      ...form,
      customerType: Number(customer.value?.customerType) || 1,
    };
    if (isCreate.value) {
      // 新建客户
      const res: any = await createCustomerApi(payload);
      const newId = res?.id ?? res?.data?.id;
      message.success('创建成功');
      if (newId) emit('created', newId);
    } else {
      // 更新客户
      await updateCustomerApi({ ...payload, id: Number(props.id) });
      Object.assign(customer.value, payload);
      message.success('保存成功');
      loadEditLogs();
    }
  } catch {
    // 全局拦截处理
  } finally {
    formSaving.value = false;
  }
}

// ========== 财务信息 ==========
const financialLoading = ref(false);
const financialSaving = ref(false);
const financialData = ref<CustomerFinancialVO | null>(null);
const financialForm = reactive({
  taxId: '',
  invoiceTitle: '',
  registeredAddress: '',
  registeredPhone: '',
  financePhone: '',
  bankAccounts: [] as BankAccount[],
});

function fillFinancialForm(data: CustomerFinancialVO | null) {
  if (data) {
    financialForm.taxId = data.taxId || '';
    financialForm.invoiceTitle = data.invoiceTitle || '';
    financialForm.registeredAddress = data.registeredAddress || '';
    financialForm.registeredPhone = data.registeredPhone || '';
    financialForm.financePhone = data.financePhone || '';
    financialForm.bankAccounts = Array.isArray(data.bankAccounts)
      ? [...data.bankAccounts]
      : [];
  } else {
    financialForm.taxId = '';
    financialForm.invoiceTitle = '';
    financialForm.registeredAddress = '';
    financialForm.registeredPhone = '';
    financialForm.financePhone = '';
    financialForm.bankAccounts = [];
  }
}

function addBankAccount() {
  financialForm.bankAccounts.push({
    accountName: '',
    bankName: '',
    accountNumber: '',
    isDefault: financialForm.bankAccounts.length === 0,
  });
}

function removeBankAccount(index: number) {
  if (financialForm.bankAccounts.length <= 1) return;
  const removed = financialForm.bankAccounts[index];
  financialForm.bankAccounts.splice(index, 1);
  // 如果删除了默认账号，将第一个设为默认
  if (removed?.isDefault && financialForm.bankAccounts.length > 0) {
    const firstAccount = financialForm.bankAccounts[0];
    if (firstAccount) {
      firstAccount.isDefault = true;
    }
  }
}

function setDefaultBankAccount(index: number) {
  financialForm.bankAccounts.forEach((acct, i) => {
    acct.isDefault = i === index;
  });
}

async function loadFinancialData() {
  if (!props.id) return;
  financialLoading.value = true;
  try {
    const resp: any = await getCustomerFinancialApi(Number(props.id));
    const data = resp?.data ?? resp ?? null;
    financialData.value = data;
    fillFinancialForm(data);
  } catch {
    financialData.value = null;
    fillFinancialForm(null);
  } finally {
    financialLoading.value = false;
  }
}

async function handleSaveFinancial() {
  if (!props.id) return;

  // 校验纳税人识别号（统一社会信用代码为18位）
  if (financialForm.taxId && !/^[0-9A-Za-z]{18}$/.test(financialForm.taxId)) {
    message.error('纳税人识别号格式不正确，统一社会信用代码应为18位');
    return;
  }

  // 校验注册电话必须为数字
  if (
    financialForm.registeredPhone &&
    !/^\d+$/.test(financialForm.registeredPhone)
  ) {
    message.error('注册电话只能包含数字');
    return;
  }

  // 校验财务电话必须为数字
  if (financialForm.financePhone && !/^\d+$/.test(financialForm.financePhone)) {
    message.error('财务电话只能包含数字');
    return;
  }

  // 校验银行账户
  for (let i = 0; i < financialForm.bankAccounts.length; i++) {
    const acct = financialForm.bankAccounts[i];
    if (!acct) continue;
    if (!acct.bankName?.trim()) {
      message.error(`第 ${i + 1} 个银行账户的「开户行」不能为空`);
      return;
    }
    if (!acct.accountNumber?.trim()) {
      message.error(`第 ${i + 1} 个银行账户的「银行账号」不能为空`);
      return;
    }
    if (!/^\d+$/.test(acct.accountNumber.trim())) {
      message.error(`第 ${i + 1} 个银行账户的「银行账号」只能包含数字`);
      return;
    }
  }
  financialSaving.value = true;
  try {
    await updateCustomerFinancialApi({
      customerId: Number(props.id),
      taxId: financialForm.taxId,
      invoiceTitle: financialForm.invoiceTitle,
      registeredAddress: financialForm.registeredAddress,
      registeredPhone: financialForm.registeredPhone,
      financePhone: financialForm.financePhone,
      bankAccounts: financialForm.bankAccounts,
    });
    message.success('财务信息保存成功');
    await loadFinancialData();
    await loadFinancialEditLogs();
  } catch {
    // 全局拦截处理
  } finally {
    financialSaving.value = false;
  }
}

// ========== 商机列表 ==========
const opportunities = ref<any[]>([]);
const oppLoading = ref(false);
const oppPagination = reactive({ page: 1, pageSize: 10, total: 0 });

const stageLabelMap: Record<number, string> = {
  1: '初步沟通',
  2: '需求确认',
  3: '方案沟通',
  4: '已报价',
  5: '成交/丢单',
};
const stageColorMap: Record<number, string> = {
  1: 'blue',
  2: 'cyan',
  3: 'gold',
  4: 'orange',
  5: 'green',
};

async function loadOpportunities() {
  if (!props.id) return;
  oppLoading.value = true;
  try {
    const resp: any = await getOpportunityListApi({
      page: oppPagination.page,
      pageSize: oppPagination.pageSize,
      customerId: Number(props.id),
      listType: 'customer',
    });
    const items = resp?.items || resp?.data?.items || [];
    opportunities.value = items;
    oppPagination.total = resp?.total ?? resp?.data?.total ?? items.length;
  } catch {
    opportunities.value = [];
  } finally {
    oppLoading.value = false;
  }
}

function handleOppPageChange(page: number) {
  oppPagination.page = page;
  loadOpportunities();
}

function openCreateOpportunity() {
  oppDetailId.value = undefined;
  oppDetailCustomerId.value = Number(props.id);
  oppDetailCustomerName.value = customer.value?.companyName || '';
  oppDetailVisible.value = true;
}

function openEditOpportunity(row: any) {
  oppDetailId.value = row.id;
  oppDetailCustomerId.value = undefined;
  oppDetailCustomerName.value = '';
  oppDetailVisible.value = true;
}

async function handleDeleteOpportunity(row: any) {
  const id = row.id ?? row.id_;
  if (!id) return;
  try {
    await deleteOpportunityApi([Number(id)]);
    message.success('商机已删除');
    loadOpportunities();
  } catch {
    // 全局拦截处理
  }
}

watch(
  () => activeTab.value,
  (tab) => {
    if (tab === 'basic' && editLogs.value.length === 0) loadEditLogs();
    if (tab === 'financial') {
      loadFinancialData();
      if (financialEditLogs.value.length === 0) loadFinancialEditLogs();
    }
    if (tab === 'opportunities') loadOpportunities();
    if (tab === 'orders') loadOrderList();
    if (tab === 'contracts') loadContractList();
    if (tab === 'payments') loadPaymentList();
    if (tab === 'refunds') loadRefundList();
    if (tab === 'expenses') loadExpenseList();
    if (tab === 'mailLogs') loadMailLogs();
  },
);

watch(
  () => props.id,
  () => {
    loadData();
  },
  { immediate: true },
);
</script>

<template>
  <div class="p-4">
    <Skeleton :loading="loading" active>
      <!-- 头部信息卡片（仅编辑模式） -->
      <Card
        v-if="!isCreate"
        class="rounded-lg shadow-sm"
        :body-style="{ padding: '24px' }"
        style="margin-bottom: 16px"
      >
        <div class="flex items-start justify-between">
          <div class="flex items-start gap-5">
            <Avatar
              :size="64"
              :style="{
                backgroundColor: isPersonal
                  ? customer.gender === 2
                    ? '#eb2f96'
                    : '#1677ff'
                  : '#1677ff',
                fontSize: '24px',
                fontWeight: 600,
              }"
            >
              {{ initials }}
            </Avatar>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-3 mb-3">
                <Tag :color="isPersonal ? 'green' : 'blue'" class="text-sm">
                  {{ isPersonal ? '个人' : '企业' }}
                </Tag>
                <h2 class="text-xl font-bold text-gray-800 m-0">
                  {{ displayName }}
                </h2>
                <Tag :color="levelColor" class="text-sm">{{ levelLabel }}</Tag>
                <Tag
                  v-if="customer.customerNo"
                  color="default"
                  class="text-xs text-gray-400"
                >
                  {{ customer.customerNo }}
                </Tag>
              </div>
              <div
                class="flex items-center gap-5 text-sm text-gray-500 flex-wrap mb-3"
              >
                <!-- 企业客户副信息 -->
                <template v-if="!isPersonal">
                  <span
                    v-if="customer.industry"
                    class="flex items-center gap-1.5"
                  >
                    <LucideBuilding2 :size="14" class="text-gray-400" />{{
                      industryLabelMap[customer.industry] || customer.industry
                    }}
                  </span>
                  <span
                    v-if="customer.country"
                    class="flex items-center gap-1.5"
                  >
                    <LucideMapPin :size="14" class="text-gray-400" />{{
                      customer.country
                    }}
                  </span>
                  <span
                    v-if="customer.website"
                    class="flex items-center gap-1.5"
                  >
                    <LucideGlobe :size="14" class="text-gray-400" /><a
                      :href="customer.website"
                      target="_blank"
                      class="text-blue-500 hover:text-blue-600 hover:underline"
                      >{{ customer.website }}</a
                    >
                  </span>
                </template>
                <!-- 个人客户副信息 -->
                <template v-else>
                  <span
                    v-if="customer.personalMobile"
                    class="flex items-center gap-1.5"
                  >
                    <LucidePhone :size="14" class="text-gray-400" />{{
                      customer.personalMobile
                    }}
                  </span>
                  <span
                    v-if="customer.personalEmail"
                    class="flex items-center gap-1.5"
                  >
                    <LucideMail :size="14" class="text-gray-400" />{{
                      customer.personalEmail
                    }}
                  </span>
                  <span
                    v-if="customer.occupation"
                    class="flex items-center gap-1.5"
                  >
                    <LucideBuilding2 :size="14" class="text-gray-400" />{{
                      customer.occupation
                    }}
                  </span>
                  <span
                    v-if="customer.country"
                    class="flex items-center gap-1.5"
                  >
                    <LucideMapPin :size="14" class="text-gray-400" />{{
                      customer.country
                    }}
                  </span>
                </template>
                <span
                  v-if="customer.assignedToName"
                  class="flex items-center gap-1.5"
                >
                  <LucideUserPlus :size="14" class="text-gray-400" />{{
                    customer.assignedToName
                  }}
                </span>
                <span
                  v-if="!isPersonal && customer.cooperatedAt"
                  class="flex items-center gap-1.5"
                >
                  <span class="text-gray-400">合作:</span
                  >{{ customer.cooperatedAt }}
                </span>
              </div>
              <div class="flex items-center gap-2">
                <TagSelector
                  entity-type="customer"
                  :entity-id="Number(props.id)"
                />
              </div>
            </div>
          </div>
          <div class="flex items-center gap-3">
            <!-- 工商背调仅企业客户显示 -->
            <Tooltip
              v-if="!isPersonal"
              :title="bgExpanded ? '收起工商信息' : '展开工商信息'"
            >
              <Button
                :icon="h(bgExpanded ? LucideChevronUp : LucideChevronDown)"
                @click="bgExpanded = !bgExpanded"
              />
            </Tooltip>
            <Button
              type="primary"
              :icon="h(LucideFilePenLine)"
              @click="handleEdit"
            >
              编辑
            </Button>
            <Button :icon="h(LucideMail)" @click="handleSendMail">
              发邮件
            </Button>
            <Dropdown>
              <Button :icon="h(LucideMoreHorizontal)" />
              <template #overlay>
                <Menu>
                  <MenuItem key="transfer">转移负责人</MenuItem>
                  <MenuItem key="merge">合并客户</MenuItem>
                  <MenuItem key="returnToPool" @click="handleReturnToPool">
                    退回公海
                  </MenuItem>
                </Menu>
              </template>
            </Dropdown>
          </div>
        </div>

        <!-- 工商信息展开面板（仅企业客户） -->
        <Transition name="bg-expand">
          <div
            v-if="!isPersonal && bgExpanded"
            class="mt-5 pt-5"
            style="border-top: 1px solid var(--border-color-base, #f0f0f0)"
          >
            <Spin :spinning="bgLoading">
              <div v-if="!bgReport" class="text-center py-8">
                <div class="text-gray-400 mb-3">暂无企业工商背调信息</div>
                <Button
                  type="primary"
                  :icon="h(LucidePlus)"
                  @click="handleRunBackgroundCheck"
                >
                  一键获取企业背调
                </Button>
              </div>
              <div v-else>
                <div class="flex items-center justify-between mb-4">
                  <div class="flex items-center gap-3">
                    <span class="text-base font-semibold text-gray-800"
                      >企业工商信息</span
                    >
                    <span
                      class="flex items-center gap-2 text-xs text-gray-400 flex-wrap"
                    >
                      <span
                        >评估时间：{{
                          bgReport.createdAt
                            ? formatDateTime(bgReport.createdAt)
                            : '-'
                        }}</span
                      >
                      <span v-if="bgReport.createdBy"
                        >操作人：{{ bgReport.createdBy }}</span
                      >
                      <Tag
                        v-if="bgPrevReport"
                        color="orange"
                        size="small"
                        class="!mr-0"
                        >信息已更新</Tag
                      >
                    </span>
                  </div>
                  <Space>
                    <Button
                      size="small"
                      type="primary"
                      ghost
                      :loading="bgCorrecting"
                      :disabled="!correctCompanyName"
                      @click="handleCorrectCompanyName"
                    >
                      一键更正公司名称
                    </Button>
                    <Button
                      size="small"
                      :icon="h(LucidePlus)"
                      @click="handleRunBackgroundCheck"
                    >
                      重新评估
                    </Button>
                  </Space>
                </div>

                <!-- 基础工商信息 -->
                <Descriptions
                  :column="2"
                  bordered
                  size="small"
                  style="margin-bottom: 15px"
                >
                  <Descriptions.Item label="公司全称">
                    {{
                      bgReport.reportData?.basic_info?.company_name ||
                      bgReport.companyName ||
                      '-'
                    }}
                  </Descriptions.Item>
                  <Descriptions.Item label="统一社会信用代码">
                    {{ bgReport.reportData?.basic_info?.credit_code || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="法定代表人">
                    {{ bgReport.reportData?.basic_info?.legal_person || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="企业类型">
                    {{ bgReport.reportData?.basic_info?.company_type || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="成立日期">
                    {{ bgReport.reportData?.basic_info?.establish_date || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="注册资本">
                    {{
                      bgReport.reportData?.basic_info?.registered_capital || '-'
                    }}
                  </Descriptions.Item>
                  <Descriptions.Item label="经营状态">
                    {{
                      bgReport.reportData?.business_analysis?.business_status ||
                      '-'
                    }}
                  </Descriptions.Item>
                  <Descriptions.Item label="参保人数">
                    {{
                      bgReport.reportData?.business_analysis?.insured_count ||
                      '-'
                    }}
                  </Descriptions.Item>
                  <Descriptions.Item label="注册地址" :span="2">
                    {{
                      bgReport.reportData?.basic_info?.registered_address || '-'
                    }}
                  </Descriptions.Item>
                </Descriptions>

                <!-- 风险评分 -->
                <Row :gutter="16">
                  <Col :span="10">
                    <div class="bg-risk-card">
                      <div class="bg-risk-label">综合风险评分</div>
                      <div
                        class="bg-risk-value"
                        :class="getRiskClass(bgReport.riskScore)"
                      >
                        {{ bgReport.riskScore || '-' }}
                      </div>
                      <div class="bg-risk-bar">
                        <div
                          class="bg-risk-bar-fill"
                          :class="getRiskClass(bgReport.riskScore)"
                          :style="{
                            width: `${Math.min((bgReport.riskScore || 50) * 1.2, 100)}%`,
                          }"
                        ></div>
                      </div>
                      <Tag
                        :color="getRiskTagColor(bgReport.riskLevel)"
                        class="mt-2"
                      >
                        {{ bgReport.riskLevel || '-' }}
                      </Tag>
                    </div>
                  </Col>
                  <Col :span="14">
                    <div class="bg-suggestion-card">
                      <div class="bg-suggestion-label">AI合作建议</div>
                      <div class="bg-suggestion-content">
                        {{
                          bgReport.reportData?.cooperation_suggestion
                            ?.suggestion ||
                          bgReport.reportData?.summary ||
                          '-'
                        }}
                      </div>
                    </div>
                  </Col>
                </Row>
              </div>
            </Spin>
          </div>
        </Transition>
      </Card>

      <!-- KPI 统计卡片（仅编辑模式） -->
      <Row v-if="!isCreate" :gutter="16" style="margin-bottom: 16px">
        <Col v-for="stat in statCards" :key="stat.label" :span="4">
          <Card
            size="small"
            class="text-center rounded-lg hover:shadow-md transition-shadow"
            :body-style="{ padding: '20px 16px', backgroundColor: stat.bg }"
          >
            <div class="text-2xl font-bold" :class="stat.color">
              {{ stat.value }}
            </div>
            <div class="text-xs text-gray-500 mt-2">{{ stat.label }}</div>
          </Card>
        </Col>
      </Row>

      <!-- Tab 内容区 -->
      <Card class="overflow-hidden" :body-style="{ padding: '0' }">
        <Tabs
          v-model:active-key="activeTab"
          :tab-bar-style="{ paddingLeft: '30px' }"
          class="pt-4"
        >
          <Tabs.TabPane key="basic" tab="基本信息">
            <div
              class="p-4"
              style="display: flex; gap: 16px; min-height: 500px"
            >
              <!-- 左侧：修改记录时间轴（仅编辑模式） -->
              <div v-if="!isCreate" class="basic-left">
                <div class="basic-left-header">修改记录</div>
                <Skeleton
                  :loading="editLogLoading"
                  active
                  :paragraph="{ rows: 3 }"
                >
                  <div class="basic-timeline-wrap">
                    <Timeline v-if="editLogs.length > 0">
                      <Timeline.Item
                        v-for="log in editLogs"
                        :key="log.id"
                        :color="log.logType === 2 ? 'purple' : 'blue'"
                      >
                        <div class="flex items-start justify-between mb-1">
                          <div class="flex items-center gap-2">
                            <Avatar
                              size="small"
                              :style="{
                                backgroundColor:
                                  log.logType === 2 ? '#722ed1' : '#1677ff',
                              }"
                            >
                              {{ log.editorName?.charAt(0) || '?' }}
                            </Avatar>
                            <span class="font-medium text-sm">{{
                              log.editorName || '未知'
                            }}</span>
                            <Tag
                              v-if="log.logType === 2"
                              color="purple"
                              size="small"
                              style="margin: 0; font-size: 11px"
                            >
                              转移
                            </Tag>
                          </div>
                          <span class="text-xs text-gray-400">{{
                            log.editTime ? formatDateTime(log.editTime) : '-'
                          }}</span>
                        </div>
                        <!-- 转移日志（logType=2）：content 是 JSON 对象 -->
                        <div v-if="log.logType === 2" class="mt-1 space-y-2">
                          <div class="transfer-log-card">
                            <div class="transfer-flow">
                              <Tag color="default" size="small">
                                {{ (log as any).content?.['原负责人'] || '-' }}
                              </Tag>
                              <span class="transfer-arrow">→</span>
                              <Tag color="purple" size="small">
                                {{ (log as any).content?.['新负责人'] || '-' }}
                              </Tag>
                            </div>
                            <div class="transfer-reason">
                              <span class="reason-label">交接原因：</span>
                              <span class="reason-value">{{
                                (log as any).content?.['交接原因'] || '-'
                              }}</span>
                            </div>
                            <div
                              v-if="(log as any).content?.['备注']"
                              class="transfer-remark"
                            >
                              <span class="remark-label">备注：</span>
                              <span class="remark-value">{{
                                (log as any).content?.['备注']
                              }}</span>
                            </div>
                            <div
                              v-if="(log as any).content?.['受影响资源']"
                              class="transfer-affected"
                            >
                              <span class="affected-label">受影响资源：</span>
                              <div class="affected-tags">
                                <Tag
                                  v-for="(val, key) in (log as any).content?.[
                                    '受影响资源'
                                  ]"
                                  :key="key"
                                  :color="val > 0 ? 'blue' : 'default'"
                                  size="small"
                                  style="margin: 2px 4px 2px 0; font-size: 11px"
                                >
                                  {{ key }} × {{ val }}
                                </Tag>
                              </div>
                            </div>
                          </div>
                        </div>
                        <!-- 普通修改日志（logType=0）：content 是 EditLogItem 数组 -->
                        <div v-else class="mt-1 space-y-1">
                          <div
                            v-for="(item, idx) in log.content"
                            :key="idx"
                            class="text-xs flex items-center gap-1 py-1 px-2 rounded bg-gray-50 flex-wrap"
                          >
                            <Tag
                              color="blue"
                              size="small"
                              class="!mr-0"
                              style="font-size: 11px"
                            >
                              {{ item.fieldLabel }}
                            </Tag>
                            <template
                              v-if="item.old !== null && item.new !== null"
                            >
                              <span class="text-gray-400 line-through">{{
                                getFieldValueLabel(item.field, item.old)
                              }}</span>
                              <span class="text-gray-400">→</span>
                              <span class="text-green-600 font-medium">{{
                                getFieldValueLabel(item.field, item.new)
                              }}</span>
                            </template>
                            <template v-else-if="item.new === null">
                              <span class="text-red-500"
                                >删除：{{
                                  getFieldValueLabel(item.field, item.old)
                                }}</span
                              >
                            </template>
                            <template v-else>
                              <span class="text-green-600 font-medium">{{
                                getFieldValueLabel(item.field, item.new)
                              }}</span>
                            </template>
                          </div>
                        </div>
                      </Timeline.Item>
                    </Timeline>
                    <Empty
                      v-else
                      description="暂无修改记录"
                      :image-style="{ height: '40px' }"
                    />
                  </div>
                </Skeleton>
              </div>

              <!-- 右侧：编辑表单 -->
              <div class="basic-right">
                <div class="basic-right-header">
                  <span class="text-sm font-semibold text-gray-700"
                    >客户信息</span
                  >
                </div>
                <Form
                  :model="form"
                  layout="vertical"
                  class="customer-basic-form"
                >
                  <Row :gutter="16">
                    <!-- 企业客户：公司名称（必填） -->
                    <Col v-if="!isPersonal" :span="12">
                      <Form.Item
                        label="公司名称"
                        required
                        :validate-status="nameDuplicateError ? 'error' : ''"
                        :help="nameDuplicateError"
                      >
                        <Input
                          v-model:value="form.companyName"
                          placeholder="请输入公司名称"
                          allow-clear
                          @change="
                            (e: any) =>
                              isCreate &&
                              checkCompanyNameOnInput(e?.target?.value || '')
                          "
                        />
                      </Form.Item>
                    </Col>
                    <!-- 个人客户：姓名（必填） -->
                    <Col v-if="isPersonal" :span="12">
                      <Form.Item
                        label="姓名"
                        required
                        :validate-status="nameDuplicateError ? 'error' : ''"
                        :help="nameDuplicateError"
                      >
                        <Input
                          v-model:value="form.personName"
                          placeholder="请输入姓名"
                          allow-clear
                          @change="
                            (e: any) =>
                              isCreate &&
                              checkPersonNameOnInput(e?.target?.value || '')
                          "
                        />
                      </Form.Item>
                    </Col>
                    <Col :span="12">
                      <Form.Item label="客户编号">
                        <Input
                          v-model:value="form.customerNo"
                          placeholder="保存时自动生成"
                          disabled
                        />
                      </Form.Item>
                    </Col>
                    <!-- 个人客户：性别 / 出生日期 -->
                    <Col v-if="isPersonal" :span="12">
                      <Form.Item label="性别">
                        <Select
                          v-model:value="form.gender"
                          :options="genderOptions"
                          placeholder="请选择性别"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <Col v-if="isPersonal" :span="12">
                      <Form.Item label="出生日期">
                        <DatePicker
                          v-model:value="form.birthday"
                          placeholder="选择日期"
                          style="width: 100%"
                          value-format="YYYY-MM-DD"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <!-- 个人客户：手机号 / 邮箱 -->
                    <Col v-if="isPersonal" :span="12">
                      <Form.Item label="手机号">
                        <Input
                          v-model:value="form.personalMobile"
                          placeholder="请输入手机号"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <Col v-if="isPersonal" :span="12">
                      <Form.Item label="个人邮箱">
                        <Input
                          v-model:value="form.personalEmail"
                          placeholder="请输入邮箱"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <!-- 个人客户：微信 / QQ -->
                    <Col v-if="isPersonal" :span="12">
                      <Form.Item label="微信">
                        <Input
                          v-model:value="form.wechat"
                          placeholder="请输入微信号"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <Col v-if="isPersonal" :span="12">
                      <Form.Item label="QQ">
                        <Input
                          v-model:value="form.qq"
                          placeholder="请输入QQ号"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <!-- 个人客户：职业 / 昵称 -->
                    <Col v-if="isPersonal" :span="12">
                      <Form.Item label="职业">
                        <Input
                          v-model:value="form.occupation"
                          placeholder="请输入职业"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <Col v-if="isPersonal" :span="12">
                      <Form.Item label="昵称">
                        <Input
                          v-model:value="form.nickname"
                          placeholder="请输入昵称"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <!-- 企业客户：简称 / 等级 -->
                    <Col v-if="!isPersonal" :span="12">
                      <Form.Item label="简称">
                        <Input
                          v-model:value="form.shortName"
                          placeholder="请输入公司简称"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <Col :span="12">
                      <Form.Item label="客户等级">
                        <Select
                          v-model:value="form.level"
                          :options="levelOptions"
                          placeholder="请选择客户等级"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <Col :span="12">
                      <Form.Item label="客户来源">
                        <Select
                          v-model:value="form.source"
                          :options="sourceOptions"
                          placeholder="请选择来源"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <!-- 企业客户：行业 -->
                    <Col v-if="!isPersonal" :span="12">
                      <Form.Item label="行业">
                        <Select
                          v-model:value="form.industry"
                          :options="industryOptions"
                          placeholder="请选择行业"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <Col :span="12">
                      <Form.Item label="国家">
                        <Select
                          v-model:value="form.country"
                          :options="countryOptions"
                          placeholder="请选择国家"
                          allow-clear
                          show-search
                          :filter-option="
                            (input: string, option: any) =>
                              option.label
                                ?.toLowerCase()
                                .includes(input.toLowerCase())
                          "
                          @change="handleCountryChange"
                        />
                      </Form.Item>
                    </Col>
                    <Col :span="12">
                      <Form.Item label="省/州">
                        <Cascader
                          v-if="isChina"
                          v-model:value="regionPath"
                          :options="chinaAreaOptions"
                          placeholder="请选择省/市/区"
                          change-on-select
                          allow-clear
                          @change="handleRegionCascaderChange"
                        />
                        <Input
                          v-else
                          v-model:value="form.region"
                          placeholder="省/州"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <Col :span="24">
                      <Form.Item label="详细地址">
                        <Input
                          v-model:value="form.address"
                          placeholder="详细地址"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <!-- 企业客户：网站 -->
                    <Col v-if="!isPersonal" :span="12">
                      <Form.Item label="网站">
                        <Input
                          v-model:value="form.website"
                          placeholder="https://"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <Col :span="12">
                      <Form.Item label="币种">
                        <Select
                          v-model:value="form.currency"
                          :options="currencyOptions"
                          placeholder="请选择币种"
                        />
                      </Form.Item>
                    </Col>
                    <!-- 企业客户：信用额度 / 账期 / 合作起始 -->
                    <Col v-if="!isPersonal" :span="12">
                      <Form.Item label="信用额度">
                        <InputNumber
                          v-model:value="form.creditLimit"
                          placeholder="信用额度"
                          :min="0"
                          style="width: 100%"
                        />
                      </Form.Item>
                    </Col>
                    <Col v-if="!isPersonal" :span="12">
                      <Form.Item label="账期（天）">
                        <InputNumber
                          v-model:value="form.creditDays"
                          placeholder="账期天数"
                          :min="0"
                          style="width: 100%"
                        />
                      </Form.Item>
                    </Col>
                    <Col v-if="!isPersonal" :span="12">
                      <Form.Item label="合作起始日期">
                        <DatePicker
                          v-model:value="form.cooperatedAt"
                          placeholder="选择日期"
                          style="width: 100%"
                          value-format="YYYY-MM-DD"
                          allow-clear
                        />
                      </Form.Item>
                    </Col>
                    <Col :span="24">
                      <Form.Item label="备注">
                        <Input.TextArea
                          v-model:value="form.description"
                          placeholder="备注信息"
                          :rows="3"
                        />
                      </Form.Item>
                    </Col>
                  </Row>
                </Form>
                <div class="basic-right-footer">
                  <Button
                    type="primary"
                    :loading="formSaving"
                    @click="handleSaveForm"
                  >
                    {{ isCreate ? '创建客户' : '保存' }}
                  </Button>
                </div>
              </div>
            </div>
          </Tabs.TabPane>
          <Tabs.TabPane v-if="!isCreate" key="financial" tab="财务信息">
            <div
              class="p-4"
              style="display: flex; gap: 16px; min-height: 500px"
            >
              <!-- 左侧：修改记录时间轴（只显示财务字段变更，不显示时间） -->
              <div class="basic-left">
                <div class="basic-left-header">修改记录</div>
                <Skeleton
                  :loading="financialEditLogLoading"
                  active
                  :paragraph="{ rows: 3 }"
                >
                  <div class="basic-timeline-wrap">
                    <Timeline v-if="financialEditLogs.length > 0">
                      <Timeline.Item
                        v-for="log in financialEditLogs"
                        :key="log.id"
                        color="blue"
                      >
                        <div class="flex items-start justify-between mb-1">
                          <div class="flex items-center gap-2">
                            <Avatar
                              size="small"
                              :style="{ backgroundColor: '#1677ff' }"
                            >
                              {{ log.editorName?.charAt(0) || '?' }}
                            </Avatar>
                            <span class="font-medium text-sm">{{
                              log.editorName || '未知'
                            }}</span>
                          </div>
                          <!-- 不显示修改时间，只显示修改内容 -->
                        </div>
                        <div class="mt-1 space-y-1">
                          <div
                            v-for="(item, idx) in log.content"
                            :key="idx"
                            class="text-xs flex items-center gap-1 py-1 px-2 rounded bg-gray-50 flex-wrap"
                          >
                            <Tag
                              color="blue"
                              size="small"
                              class="!mr-0"
                              style="font-size: 11px"
                            >
                              {{ item.fieldLabel }}
                            </Tag>
                            <template
                              v-if="item.old !== null && item.new !== null"
                            >
                              <span class="text-gray-400 line-through">{{
                                item.old
                              }}</span>
                              <span class="text-gray-400">→</span>
                              <span class="text-green-600 font-medium">{{
                                item.new
                              }}</span>
                            </template>
                            <template v-else-if="item.new === null">
                              <span class="text-red-500"
                                >删除：{{ item.old }}</span
                              >
                            </template>
                            <template v-else>
                              <span class="text-green-600 font-medium">{{
                                item.new
                              }}</span>
                            </template>
                          </div>
                        </div>
                      </Timeline.Item>
                    </Timeline>
                    <Empty
                      v-else
                      description="暂无修改记录"
                      :image-style="{ height: '40px' }"
                    />
                  </div>
                </Skeleton>
              </div>
              <!-- 右侧：编辑表单 -->
              <div class="basic-right">
                <div class="basic-right-header">
                  <span class="text-sm font-semibold text-gray-700">
                    {{ isPersonal ? '个人银行卡' : '税务信息' }}
                  </span>
                </div>
                <Spin :spinning="financialLoading">
                  <Form layout="vertical" class="customer-basic-form">
                    <!-- 企业客户：税务信息字段 -->
                    <Row v-if="!isPersonal" :gutter="16">
                      <Col :span="12">
                        <Form.Item label="纳税人识别号">
                          <Input
                            v-model:value="financialForm.taxId"
                            placeholder="统一社会信用代码"
                            allow-clear
                          />
                        </Form.Item>
                      </Col>
                      <Col :span="12">
                        <Form.Item label="发票抬头">
                          <Input
                            v-model:value="financialForm.invoiceTitle"
                            placeholder="发票抬头名称"
                            allow-clear
                          />
                        </Form.Item>
                      </Col>
                      <Col :span="12">
                        <Form.Item label="注册地址">
                          <Input
                            v-model:value="financialForm.registeredAddress"
                            placeholder="税务登记注册地址"
                            allow-clear
                          />
                        </Form.Item>
                      </Col>
                      <Col :span="12">
                        <Form.Item label="注册电话">
                          <Input
                            v-model:value="financialForm.registeredPhone"
                            placeholder="税务登记电话"
                            allow-clear
                          />
                        </Form.Item>
                      </Col>
                      <Col :span="12">
                        <Form.Item label="财务电话">
                          <Input
                            v-model:value="financialForm.financePhone"
                            placeholder="财务部门联系电话"
                            allow-clear
                          />
                        </Form.Item>
                      </Col>
                    </Row>

                    <div class="flex items-center justify-between mt-5 mb-3">
                      <span class="text-sm font-semibold text-gray-700">
                        {{ isPersonal ? '银行卡信息' : '银行账户信息' }}
                      </span>
                      <Button
                        size="small"
                        type="dashed"
                        :icon="h(LucidePlus)"
                        @click="addBankAccount"
                      >
                        {{ isPersonal ? '添加银行卡' : '添加账户' }}
                      </Button>
                    </div>

                    <div
                      v-for="(acct, idx) in financialForm.bankAccounts"
                      :key="idx"
                      class="bank-account-card"
                    >
                      <div class="flex items-center justify-between mb-2">
                        <Tag v-if="acct.isDefault" color="blue">默认账户</Tag>
                        <span v-else class="text-xs text-gray-400"
                          >账户 {{ idx + 1 }}</span
                        >
                        <Space size="small">
                          <Button
                            v-if="!acct.isDefault"
                            size="small"
                            type="link"
                            @click="setDefaultBankAccount(idx)"
                          >
                            设为默认
                          </Button>
                          <Popconfirm
                            title="确认删除该银行账户？"
                            ok-text="确认"
                            cancel-text="取消"
                            @confirm="removeBankAccount(idx)"
                          >
                            <Button size="small" type="link" danger>
                              删除
                            </Button>
                          </Popconfirm>
                        </Space>
                      </div>
                      <Row :gutter="16">
                        <Col :span="24">
                          <Form.Item label="账户名称">
                            <Input
                              v-model:value="acct.accountName"
                              placeholder="如：北京心月狐科技有限公司"
                              allow-clear
                            />
                          </Form.Item>
                        </Col>
                        <Col :span="24">
                          <Form.Item label="开户行">
                            <Input
                              v-model:value="acct.bankName"
                              placeholder="如：中国银行北京分行"
                              allow-clear
                            />
                          </Form.Item>
                        </Col>
                        <Col :span="24">
                          <Form.Item label="银行账号">
                            <Input
                              v-model:value="acct.accountNumber"
                              placeholder="银行账号"
                              allow-clear
                            />
                          </Form.Item>
                        </Col>
                      </Row>
                    </div>

                    <Empty
                      v-if="financialForm.bankAccounts.length === 0"
                      description="暂无银行账户，点击上方添加"
                      :image-style="{ height: '50px' }"
                      class="my-4"
                    />
                  </Form>
                  <div class="basic-right-footer">
                    <Button
                      type="primary"
                      :loading="financialSaving"
                      @click="handleSaveFinancial"
                    >
                      保存
                    </Button>
                  </div>
                </Spin>
              </div>
            </div>
          </Tabs.TabPane>

          <Tabs.TabPane
            v-if="!isCreate"
            key="contacts"
            :tab="`联系人 (${contacts.length})`"
          >
            <div class="flex items-center justify-between mb-4 mt-2 px-2">
              <span class="text-sm font-semibold text-gray-600">当前在职</span>
              <Button
                size="small"
                type="primary"
                ghost
                :icon="h(LucideUserPlus)"
                @click="handleAddContact"
              >
                添加联系人
              </Button>
            </div>
            <div
              v-if="contacts.length === 0"
              class="text-gray-400 text-center py-16 text-sm"
            >
              暂无联系人
            </div>
            <div class="space-y-3 px-2">
              <Card
                v-for="c in contacts"
                :key="c.id"
                size="small"
                hoverable
                class="border-l-4 rounded-lg transition-shadow hover:shadow-sm"
                :class="
                  c.isPrimary ? 'border-l-blue-500' : 'border-l-transparent'
                "
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-4 flex-1">
                    <Avatar
                      :size="40"
                      :style="{
                        backgroundColor: c.isPrimary ? '#1677ff' : '#d9d9d9',
                      }"
                      class="flex-shrink-0"
                    >
                      {{ c.name?.slice(0, 1) || '?' }}
                    </Avatar>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2.5 flex-wrap mb-1.5">
                        <span
                          class="font-semibold text-gray-800 cursor-pointer hover:text-blue-600 truncate"
                          @click="handleViewContact(c.id)"
                          >{{ c.name }}</span
                        >
                        <Tag :color="roleColor[c.roleType]" size="small">
                          {{ roleLabel[c.roleType] || c.roleType }}
                        </Tag>
                        <Tag v-if="c.isPrimary" color="gold" size="small">
                          首要
                        </Tag>
                        <Tag v-if="c.isBilling" color="purple" size="small">
                          账单
                        </Tag>
                        <Tag v-if="c.isShipping" color="cyan" size="small">
                          收货
                        </Tag>
                      </div>
                      <div
                        class="text-xs text-gray-500 flex items-center gap-4 flex-wrap"
                      >
                        <span v-if="c.title" class="text-gray-600">{{
                          c.title
                        }}</span>
                        <Tooltip v-if="c.email" :title="c.email">
                          <span class="flex items-center gap-1"
                            ><LucideMail :size="12" class="text-gray-400" />{{
                              c.email
                            }}</span
                          >
                        </Tooltip>
                        <Tooltip v-if="c.mobile" :title="c.mobile">
                          <span class="flex items-center gap-1"
                            ><LucidePhone :size="12" class="text-gray-400" />{{
                              c.mobile
                            }}</span
                          >
                        </Tooltip>
                      </div>
                    </div>
                  </div>
                  <Space size="small" class="flex-shrink-0">
                    <Button
                      size="small"
                      type="link"
                      @click="handleViewContact(c.id)"
                    >
                      详情
                    </Button>
                    <Popconfirm
                      title="确认解绑该联系人？"
                      ok-text="确认"
                      cancel-text="取消"
                      @confirm="handleUnbind(c.id)"
                    >
                      <Button size="small" type="link" danger>解绑</Button>
                    </Popconfirm>
                  </Space>
                </div>
              </Card>
            </div>

            <template v-if="historyContacts.length > 0">
              <Divider class="!my-5" />
              <div class="flex items-center gap-2 mb-4 px-2">
                <span class="text-sm font-semibold text-gray-400"
                  >历史联系人</span
                >
                <Tag size="small" color="default" class="text-gray-400">
                  {{ historyContacts.length }}人
                </Tag>
              </div>
              <div class="space-y-3 px-2">
                <Card
                  v-for="c in historyContacts"
                  :key="c.id"
                  size="small"
                  class="opacity-75 rounded-lg"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-4 flex-1">
                      <Avatar
                        :size="36"
                        :style="{ backgroundColor: '#d9d9d9' }"
                        class="flex-shrink-0"
                      >
                        {{ c.name?.slice(0, 1) || '?' }}
                      </Avatar>
                      <div class="flex-1">
                        <div class="flex items-center gap-2 mb-1">
                          <span
                            class="font-medium text-gray-600 cursor-pointer hover:text-blue-600"
                            @click="handleViewContact(c.id)"
                            >{{ c.name }}</span
                          >
                          <span class="text-xs text-gray-400">{{
                            c.title
                          }}</span>
                        </div>
                        <div class="text-xs text-gray-400">
                          {{ c.boundAt }} ~ {{ c.unboundAt }}
                          <span v-if="c.notes" class="ml-2"
                            >| {{ c.notes }}</span
                          >
                        </div>
                      </div>
                    </div>
                    <Button
                      size="small"
                      type="link"
                      @click="handleViewContact(c.id)"
                    >
                      详情
                    </Button>
                  </div>
                </Card>
              </div>
            </template>
          </Tabs.TabPane>

          <Tabs.TabPane
            v-if="!isCreate"
            key="opportunities"
            :tab="`商机 (${oppPagination.total || 0})`"
          >
            <div class="opp-container">
              <!-- 头部操作栏 -->
              <div class="opp-header">
                <div class="opp-header-left">
                  <span class="opp-title">企业商机</span>
                  <span v-if="oppPagination.total > 0" class="opp-count"
                    >共 {{ oppPagination.total }} 个商机</span
                  >
                </div>
                <Button
                  type="primary"
                  class="opp-create-btn"
                  @click="openCreateOpportunity"
                >
                  <template #icon><LucidePlus :size="14" /></template>
                  新建商机
                </Button>
              </div>

              <!-- 加载状态 -->
              <Spin :spinning="oppLoading">
                <!-- 空状态 -->
                <div
                  v-if="!oppLoading && opportunities.length === 0"
                  class="opp-empty"
                >
                  <Empty description="暂无商机记录">
                    <template #extra>
                      <Button
                        type="primary"
                        size="small"
                        @click="openCreateOpportunity"
                      >
                        新建第一个商机
                      </Button>
                    </template>
                  </Empty>
                </div>

                <!-- 商机列表卡片 -->
                <div v-else class="opp-list">
                  <div
                    v-for="opp in opportunities"
                    :key="opp.id"
                    class="opp-card"
                  >
                    <div class="opp-card-main">
                      <div class="opp-card-top">
                        <div class="opp-card-title-row">
                          <span class="opp-card-title">{{
                            opp.title || '未命名商机'
                          }}</span>
                          <Tag
                            :color="stageColorMap[opp.stage] || 'default'"
                            size="small"
                          >
                            {{ stageLabelMap[opp.stage] || '-' }}
                          </Tag>
                        </div>
                        <div class="opp-card-actions">
                          <Tooltip title="编辑">
                            <Button
                              type="link"
                              size="small"
                              @click.stop="openEditOpportunity(opp)"
                            >
                              <LucideFilePenLine :size="14" />
                            </Button>
                          </Tooltip>
                          <Popconfirm
                            title="确定删除该商机？"
                            @confirm="handleDeleteOpportunity(opp)"
                            ok-text="确认"
                            cancel-text="取消"
                          >
                            <Tooltip title="删除">
                              <Button type="link" size="small" danger>
                                <svg
                                  xmlns="http://www.w3.org/2000/svg"
                                  width="14"
                                  height="14"
                                  viewBox="0 0 24 24"
                                  fill="none"
                                  stroke="currentColor"
                                  stroke-width="2"
                                  stroke-linecap="round"
                                  stroke-linejoin="round"
                                >
                                  <path d="M3 6h18" />
                                  <path
                                    d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"
                                  />
                                  <path
                                    d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"
                                  />
                                </svg>
                              </Button>
                            </Tooltip>
                          </Popconfirm>
                        </div>
                      </div>
                      <div class="opp-card-details">
                        <div class="opp-detail-item">
                          <span class="opp-detail-label">金额</span>
                          <span class="opp-detail-value opp-amount"
                            >{{ opp.currency != null ? '¥' : '-'
                            }}{{
                              opp.amount != null
                                ? Number(opp.amount).toLocaleString()
                                : ''
                            }}</span
                          >
                        </div>
                        <div class="opp-detail-item">
                          <span class="opp-detail-label">概率</span>
                          <span class="opp-detail-value">{{
                            opp.probability != null
                              ? `${opp.probability}%`
                              : '-'
                          }}</span>
                        </div>
                        <div class="opp-detail-item">
                          <span class="opp-detail-label">来源</span>
                          <span class="opp-detail-value">{{
                            sourceLabelMap[Number(opp.source)] || '-'
                          }}</span>
                        </div>
                        <div class="opp-detail-item">
                          <span class="opp-detail-label">负责人</span>
                          <span class="opp-detail-value">{{
                            opp.createdByName || opp.assignedTo || '-'
                          }}</span>
                        </div>
                        <div class="opp-detail-item">
                          <span class="opp-detail-label">创建时间</span>
                          <span class="opp-detail-value">{{
                            opp.createTime
                              ? formatDateTime(opp.createTime)
                              : '-'
                          }}</span>
                        </div>
                        <div class="opp-detail-item">
                          <span class="opp-detail-label">预计成交</span>
                          <span class="opp-detail-value">{{
                            opp.expectedCloseDate || '-'
                          }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </Spin>

              <!-- 分页 -->
              <div
                v-if="oppPagination.total > oppPagination.pageSize"
                class="opp-pagination"
              >
                <div class="opp-pagination-info">
                  第
                  {{ (oppPagination.page - 1) * oppPagination.pageSize + 1 }}-{{
                    Math.min(
                      oppPagination.page * oppPagination.pageSize,
                      oppPagination.total,
                    )
                  }}
                  条，共 {{ oppPagination.total }} 条
                </div>
                <div class="opp-pagination-btns">
                  <Button
                    size="small"
                    :disabled="oppPagination.page <= 1"
                    @click="handleOppPageChange(oppPagination.page - 1)"
                  >
                    上一页
                  </Button>
                  <span class="opp-page-indicator"
                    >{{ oppPagination.page }} /
                    {{
                      Math.ceil(oppPagination.total / oppPagination.pageSize)
                    }}</span
                  >
                  <Button
                    size="small"
                    :disabled="
                      oppPagination.page >=
                      Math.ceil(oppPagination.total / oppPagination.pageSize)
                    "
                    @click="handleOppPageChange(oppPagination.page + 1)"
                  >
                    下一页
                  </Button>
                </div>
              </div>
            </div>
          </Tabs.TabPane>
          <Tabs.TabPane
            v-if="!isCreate"
            key="orders"
            :tab="`订单 (${customer.stats?.orderCount || 0})`"
          >
            <div class="order-list-wrap">
              <Spin :spinning="orderLoading">
                <div v-if="orderList.length > 0" class="order-cards">
                  <div
                    v-for="order in orderList"
                    :key="order.id"
                    class="order-card"
                  >
                    <div class="order-card-header">
                      <div class="order-card-header-left">
                        <span class="order-no">{{
                          order.orderNo || `#${order.id}`
                        }}</span>
                        <Tag
                          :color="getOrderStatusInfo(order.orderStatus).color"
                          class="!mr-0"
                        >
                          {{ getOrderStatusInfo(order.orderStatus).label }}
                        </Tag>
                        <Tag
                          :color="
                            getPaymentStatusInfo(order.paymentStatus).color
                          "
                          class="!mr-0"
                        >
                          {{ getPaymentStatusInfo(order.paymentStatus).label }}
                        </Tag>
                      </div>
                      <span class="order-date">{{
                        formatDateTime(order.orderDate)
                      }}</span>
                    </div>
                    <div class="order-card-body">
                      <div class="order-title">{{ order.title || '—' }}</div>
                      <div class="order-meta">
                        <span class="order-meta-item">
                          <span class="order-meta-label">订单金额</span>
                          <span class="order-amount">{{
                            formatMoney(order.totalAmount)
                          }}</span>
                        </span>
                        <span class="order-meta-item">
                          <span class="order-meta-label">已付</span>
                          <span class="order-paid">{{
                            formatMoney(order.paidAmount)
                          }}</span>
                        </span>
                        <span class="order-meta-item">
                          <span class="order-meta-label">未付</span>
                          <span class="order-unpaid">{{
                            formatMoney(order.unpaidAmount)
                          }}</span>
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
                <Empty v-else description="暂无订单记录" />
                <div v-if="orderTotal > 0" class="order-pagination">
                  <Pagination
                    v-model:current="orderPage"
                    v-model:page-size="orderPageSize"
                    :total="orderTotal"
                    :page-size-options="['10', '20', '50']"
                    show-size-changer
                    show-quick-jumper
                    :show-total="(t: number) => `共 ${t} 条`"
                    @change="handleOrderPageChange"
                  />
                </div>
              </Spin>
            </div>
          </Tabs.TabPane>
          <Tabs.TabPane
            v-if="!isCreate"
            key="contracts"
            :tab="`合同 (${contractTotal || 0})`"
          >
            <div class="contract-list-wrap">
              <Spin :spinning="contractLoading">
                <div v-if="contractList.length > 0" class="contract-cards">
                  <div
                    v-for="contract in contractList"
                    :key="contract.id"
                    class="contract-card"
                  >
                    <div class="contract-card-header">
                      <div class="contract-card-header-left">
                        <span class="contract-no">{{
                          contract.contractNo || `#${contract.id}`
                        }}</span>
                        <Tag
                          :color="getContractStatusInfo(contract.status).color"
                          class="!mr-0"
                        >
                          {{ getContractStatusInfo(contract.status).label }}
                        </Tag>
                        <Tag
                          :color="
                            getContractApprovalStatusInfo(
                              contract.approvalStatus,
                            ).color
                          "
                          class="!mr-0"
                        >
                          审批:
                          {{
                            getContractApprovalStatusInfo(
                              contract.approvalStatus,
                            ).label
                          }}
                        </Tag>
                      </div>
                      <span class="contract-date">
                        {{ formatDate(contract.startDate) }} ~
                        {{ formatDate(contract.endDate) }}
                      </span>
                    </div>
                    <div class="contract-card-body">
                      <div class="contract-title">
                        {{ contract.title || '—' }}
                      </div>
                      <div class="contract-meta">
                        <span class="contract-meta-item">
                          <span class="contract-meta-label">合同金额</span>
                          <span class="contract-amount">{{
                            formatMoney(contract.totalAmount ?? contract.amount)
                          }}</span>
                        </span>
                        <span class="contract-meta-item">
                          <span class="contract-meta-label">不含税</span>
                          <span class="contract-exclude-tax">{{
                            formatMoney(contract.amount)
                          }}</span>
                        </span>
                        <span class="contract-meta-item">
                          <span class="contract-meta-label">合同类型</span>
                          <span class="contract-type">{{
                            contract.contractType || '—'
                          }}</span>
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
                <Empty v-else description="暂无合同记录" />
                <div v-if="contractTotal > 0" class="contract-pagination">
                  <Pagination
                    v-model:current="contractPage"
                    v-model:page-size="contractPageSize"
                    :total="contractTotal"
                    :page-size-options="['10', '20', '50']"
                    show-size-changer
                    show-quick-jumper
                    :show-total="(t: number) => `共 ${t} 条`"
                    @change="handleContractPageChange"
                  />
                </div>
              </Spin>
            </div>
          </Tabs.TabPane>
          <Tabs.TabPane
            v-if="!isCreate"
            key="payments"
            :tab="`回款 (${paymentTotal || 0})`"
          >
            <div class="record-list-wrap">
              <Spin :spinning="paymentLoading">
                <Table
                  :columns="paymentColumns"
                  :data-source="paymentList"
                  :pagination="false"
                  row-key="id"
                  size="middle"
                  :scroll="{ x: 940 }"
                >
                  <template #bodyCell="{ column, record }">
                    <template v-if="column.dataIndex === 'amount'">
                      <span class="font-medium text-blue-600"
                        >¥{{ Number(record.amount || 0).toFixed(2) }}</span
                      >
                    </template>
                    <template v-else-if="column.dataIndex === 'appliedAmount'">
                      <span
                        :class="
                          Number(record.appliedAmount || 0) > 0
                            ? 'text-green-600 font-medium'
                            : 'text-gray-400'
                        "
                        >¥{{
                          Number(record.appliedAmount || 0).toFixed(2)
                        }}</span
                      >
                    </template>
                    <template v-else-if="column.dataIndex === 'paymentMethod'">
                      <Tag
                        :color="
                          getPaymentMethodInfo(record.paymentMethod).color
                        "
                      >
                        {{ getPaymentMethodInfo(record.paymentMethod).label }}
                      </Tag>
                    </template>
                    <template v-else-if="column.dataIndex === 'status'">
                      <Tag
                        :color="
                          getPaymentConfirmStatusInfo(record.status).color
                        "
                      >
                        {{ getPaymentConfirmStatusInfo(record.status).label }}
                      </Tag>
                    </template>
                    <template v-else-if="column.dataIndex === 'confirmTime'">
                      {{
                        record.confirmTime
                          ? formatDateTime(record.confirmTime)
                          : '—'
                      }}
                    </template>
                  </template>
                </Table>
                <div v-if="paymentTotal > 0" class="record-pagination">
                  <Pagination
                    v-model:current="paymentPage"
                    v-model:page-size="paymentPageSize"
                    :total="paymentTotal"
                    :page-size-options="['10', '20', '50']"
                    show-size-changer
                    show-quick-jumper
                    :show-total="(t: number) => `共 ${t} 条`"
                    @change="handlePaymentPageChange"
                  />
                </div>
              </Spin>
            </div>
          </Tabs.TabPane>
          <Tabs.TabPane
            v-if="!isCreate"
            key="refunds"
            :tab="`退货记录 (${refundTotal || 0})`"
          >
            <div class="record-list-wrap">
              <Spin :spinning="refundLoading">
                <Table
                  :columns="refundColumns"
                  :data-source="refundList"
                  :pagination="false"
                  row-key="id"
                  size="middle"
                  :scroll="{ x: 1080 }"
                >
                  <template #bodyCell="{ column, record }">
                    <template v-if="column.dataIndex === 'refundType'">
                      <Tag :color="getRefundTypeInfo(record.refundType).color">
                        {{ getRefundTypeInfo(record.refundType).label }}
                      </Tag>
                    </template>
                    <template v-else-if="column.dataIndex === 'refundAmount'">
                      <span class="font-medium text-red-500"
                        >¥{{
                          Number(record.refundAmount || 0).toFixed(2)
                        }}</span
                      >
                    </template>
                    <template v-else-if="column.dataIndex === 'refundedAmount'">
                      <span
                        :class="
                          Number(record.refundedAmount || 0) > 0
                            ? 'text-orange-600 font-medium'
                            : 'text-gray-400'
                        "
                        >¥{{
                          Number(record.refundedAmount || 0).toFixed(2)
                        }}</span
                      >
                    </template>
                    <template v-else-if="column.dataIndex === 'refundStatus'">
                      <Tag
                        :color="getRefundStatusInfo(record.refundStatus).color"
                      >
                        {{ getRefundStatusInfo(record.refundStatus).label }}
                      </Tag>
                    </template>
                    <template v-else-if="column.dataIndex === 'approvalStatus'">
                      <Tag
                        :color="
                          getRefundApprovalStatusInfo(record.approvalStatus)
                            .color
                        "
                      >
                        {{
                          getRefundApprovalStatusInfo(record.approvalStatus)
                            .label
                        }}
                      </Tag>
                    </template>
                    <template v-else-if="column.dataIndex === 'createTime'">
                      {{
                        record.createTime
                          ? formatDateTime(record.createTime)
                          : '—'
                      }}
                    </template>
                  </template>
                </Table>
                <div v-if="refundTotal > 0" class="record-pagination">
                  <Pagination
                    v-model:current="refundPage"
                    v-model:page-size="refundPageSize"
                    :total="refundTotal"
                    :page-size-options="['10', '20', '50']"
                    show-size-changer
                    show-quick-jumper
                    :show-total="(t: number) => `共 ${t} 条`"
                    @change="handleRefundPageChange"
                  />
                </div>
              </Spin>
            </div>
          </Tabs.TabPane>
          <Tabs.TabPane
            v-if="!isCreate"
            key="expenses"
            :tab="`费用记录 (${expenseTotal || 0})`"
          >
            <div class="record-list-wrap">
              <Spin :spinning="expenseLoading">
                <Table
                  :columns="expenseColumns"
                  :data-source="expenseList"
                  :pagination="false"
                  row-key="id"
                  size="middle"
                  :scroll="{ x: 940 }"
                >
                  <template #bodyCell="{ column, record }">
                    <template v-if="column.dataIndex === 'expenseType'">
                      <Tag
                        :color="
                          expenseTypeMap[record.expenseType]?.color || 'blue'
                        "
                      >
                        {{
                          expenseTypeMap[record.expenseType]?.name ||
                          record.expenseTypeName ||
                          '-'
                        }}
                      </Tag>
                    </template>
                    <template v-else-if="column.dataIndex === 'totalAmount'">
                      <span class="font-medium text-red-500"
                        >¥{{
                          Number(
                            record.totalAmount ?? record.amount ?? 0,
                          ).toFixed(2)
                        }}</span
                      >
                    </template>
                    <template v-else-if="column.dataIndex === 'status'">
                      <Tag :color="getExpenseStatusInfo(record.status).color">
                        {{ getExpenseStatusInfo(record.status).label }}
                      </Tag>
                    </template>
                  </template>
                </Table>
                <div v-if="expenseTotal > 0" class="record-pagination">
                  <Pagination
                    v-model:current="expensePage"
                    v-model:page-size="expensePageSize"
                    :total="expenseTotal"
                    :page-size-options="['10', '20', '50']"
                    show-size-changer
                    show-quick-jumper
                    :show-total="(t: number) => `共 ${t} 条`"
                    @change="handleExpensePageChange"
                  />
                </div>
              </Spin>
            </div>
          </Tabs.TabPane>
          <Tabs.TabPane v-if="!isCreate" key="followups" tab="跟进记录">
            <div class="followup-layout">
              <!-- 左侧：跟进记录时间轴 -->
              <div class="followup-list">
                <div class="followup-list-header">
                  <span class="text-sm font-semibold text-gray-700"
                    >跟进记录</span
                  >
                  <span
                    v-if="sortedFollowupRecords.length > 0"
                    class="text-xs text-gray-400"
                    >共 {{ sortedFollowupRecords.length }} 条</span
                  >
                </div>
                <div
                  v-if="sortedFollowupRecords.length === 0"
                  class="followup-empty"
                >
                  <Empty description="暂无跟进记录" />
                </div>
                <div v-else class="followup-timeline">
                  <div
                    v-for="(record, idx) in sortedFollowupRecords"
                    :key="record.id || idx"
                    class="followup-tl-item"
                  >
                    <div
                      class="followup-tl-dot"
                      :style="{
                        backgroundColor:
                          getMethodOption(record.activityType)?.color ||
                          '#8c8c8c',
                      }"
                    ></div>
                    <div class="followup-tl-body">
                      <div class="followup-tl-time">
                        <Tag
                          v-if="getMethodOption(record.activityType)"
                          size="small"
                          :color="
                            getMethodOption(record.activityType)!.value <= 2
                              ? 'blue'
                              : getMethodOption(record.activityType)!.value <= 4
                                ? 'purple'
                                : 'default'
                          "
                          class="followup-tl-tag"
                        >
                          {{ getMethodOption(record.activityType)!.label }}
                        </Tag>
                        <span>{{
                          record.createTime
                            ? formatDateTime(record.createTime)
                            : '-'
                        }}</span>
                        <span
                          v-if="record.createdByName"
                          class="followup-tl-user"
                          >· {{ record.createdByName }}</span
                        >
                      </div>
                      <div class="followup-tl-content">
                        {{ record.content || '-' }}
                      </div>
                      <div
                        v-if="record.nextFollowDate"
                        class="followup-tl-next"
                      >
                        下次联系：{{ record.nextFollowDate }}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <!-- 右侧：添加跟进表单 -->
              <div class="followup-form-wrap">
                <div class="followup-form-card">
                  <div class="followup-form-title">添加跟进</div>
                  <div class="followup-form-body">
                    <div class="followup-field">
                      <label class="followup-label"
                        ><span class="text-red-500">*</span> 跟进内容</label
                      >
                      <Input.TextArea
                        v-model:value="followupForm.content"
                        placeholder="请输入跟进内容..."
                        :rows="4"
                        :maxlength="2000"
                        show-count
                      />
                    </div>
                    <div class="followup-field">
                      <label class="followup-label">下次联系时间</label>
                      <DatePicker
                        v-model:value="followupForm.nextFollowAt as any"
                        placeholder="选择日期"
                        style="width: 100%"
                        value-format="YYYY-MM-DD"
                        allow-clear
                      />
                    </div>
                    <div class="followup-row">
                      <div class="followup-field followup-half">
                        <label class="followup-label">跟进方式</label>
                        <Select
                          v-model:value="followupForm.method"
                          :options="followMethodOptions"
                          placeholder="选择方式"
                          size="small"
                        />
                      </div>
                    </div>
                    <Button
                      type="primary"
                      class="followup-submit"
                      :loading="followupSaving"
                      @click="handleSaveFollowup"
                    >
                      保存跟进记录
                    </Button>
                  </div>
                </div>
              </div>
            </div>
          </Tabs.TabPane>
          <Tabs.TabPane
            v-if="!isCreate"
            key="assignHistory"
            :tab="`负责人记录 (${assignHistory.length})`"
          >
            <div class="p-4">
              <Timeline v-if="assignHistory.length > 0">
                <Timeline.Item
                  v-for="(item, index) in assignHistory"
                  :key="item.id || index"
                  :color="item.endTime ? 'blue' : 'green'"
                >
                  <div class="flex items-start justify-between">
                    <div class="flex items-center gap-2">
                      <Avatar
                        size="small"
                        :style="{
                          backgroundColor: item.endTime ? '#d9d9d9' : '#52c41a',
                        }"
                      >
                        {{ item.adminName?.charAt(0) || '?' }}
                      </Avatar>
                      <div>
                        <span class="font-medium">{{
                          item.adminName || '未知'
                        }}</span>
                        <Tag
                          v-if="!item.endTime"
                          color="green"
                          size="small"
                          class="ml-2"
                        >
                          服务中
                        </Tag>
                        <Tag v-else color="default" size="small" class="ml-2">
                          已结束
                        </Tag>
                      </div>
                    </div>
                  </div>
                  <div class="mt-2 text-sm text-gray-500">
                    <span>{{ formatDateTime(item.startTime) }}</span>
                    <span v-if="item.endTime">
                      ~ {{ formatDateTime(item.endTime) }}</span
                    >
                    <span v-else class="text-green-500"> ~ 至今</span>
                  </div>
                  <div v-if="item.remark" class="mt-1 text-xs text-gray-400">
                    {{ item.remark }}
                  </div>
                </Timeline.Item>
              </Timeline>
              <Empty v-else description="暂无负责人记录" />
            </div>
          </Tabs.TabPane>
          <Tabs.TabPane
            v-if="!isCreate"
            key="mailLogs"
            :tab="`邮件记录 (${mailLogs.length})`"
          >
            <div class="p-4">
              <Spin :spinning="mailLogsLoading">
                <Timeline v-if="mailLogs.length > 0">
                  <Timeline.Item
                    v-for="(item, index) in mailLogs"
                    :key="item.id || index"
                    :color="item.status === 1 ? 'green' : 'red'"
                  >
                    <div
                      class="flex items-start justify-between flex-wrap gap-2"
                    >
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2 flex-wrap">
                          <span class="font-medium">{{
                            item.subject || '(无主题)'
                          }}</span>
                          <Tag :color="item.status === 1 ? 'success' : 'error'">
                            {{ item.status === 1 ? '成功' : '失败' }}
                          </Tag>
                        </div>
                        <div class="mt-1 text-sm text-gray-600">
                          <span class="text-gray-400">收件人：</span>
                          <span>{{
                            Array.isArray(item.toEmails)
                              ? item.toEmails.join('; ')
                              : item.toEmails || '-'
                          }}</span>
                        </div>
                        <div
                          v-if="
                            item.ccEmails &&
                            (Array.isArray(item.ccEmails)
                              ? item.ccEmails.length
                              : item.ccEmails)
                          "
                          class="mt-1 text-sm text-gray-600"
                        >
                          <span class="text-gray-400">抄送：</span>
                          <span>{{
                            Array.isArray(item.ccEmails)
                              ? item.ccEmails.join('; ')
                              : item.ccEmails
                          }}</span>
                        </div>
                        <div
                          v-if="item.errorMsg"
                          class="mt-1 text-xs text-red-500"
                        >
                          错误：{{ item.errorMsg }}
                        </div>
                      </div>
                      <div
                        class="text-right text-xs text-gray-400 whitespace-nowrap"
                      >
                        <div v-if="item.senderName">
                          发送人：{{ item.senderName }}
                        </div>
                        <div v-if="item.sendTime">
                          发送时间：{{ formatDateTime(item.sendTime) }}
                        </div>
                        <div v-else-if="item.createTime">
                          创建时间：{{ formatDateTime(item.createTime) }}
                        </div>
                      </div>
                    </div>
                  </Timeline.Item>
                </Timeline>
                <Empty v-else description="暂无邮件记录" />
              </Spin>
            </div>
          </Tabs.TabPane>
        </Tabs>
      </Card>
      <ContactEditDrawer />
      <Drawer
        v-model:open="oppDetailVisible"
        :width="1200"
        placement="right"
        :destroy-on-close="true"
        :mask-closable="true"
        :closable="true"
        :title="oppDetailTitle"
        :body-style="{
          padding: 0,
          maxHeight: 'calc(100vh - 110px)',
          overflow: 'auto',
        }"
        @close="closeOppDetail"
      >
        <OpportunityDetail
          :id="oppDetailId"
          :customer-id="oppDetailCustomerId"
          :customer-name="oppDetailCustomerName"
          @created="loadOpportunities"
        />
      </Drawer>

      <SendMailModal
        v-model:visible="sendMailVisible"
        :customer-id="Number(props.id)"
        :customer-name="customer?.companyName"
        @success="
          () => {
            if (activeTab === 'mailLogs') loadMailLogs();
          }
        "
      />
    </Skeleton>
  </div>
</template>

<style scoped>
.bg-expand-enter-active,
.bg-expand-leave-active {
  overflow: hidden;
  transition: all 0.3s ease;
}

.bg-expand-enter-from,
.bg-expand-leave-to {
  max-height: 0;
  padding-top: 0 !important;
  margin-top: 0 !important;
  opacity: 0;
}

.bg-expand-enter-to,
.bg-expand-leave-from {
  max-height: 800px;
  opacity: 1;
}

.bg-risk-card {
  padding: 16px;
  text-align: center;
  background-color: var(--background-color-light, #fafafa);
  border: 1px solid var(--border-color-base, #f0f0f0);
  border-radius: 6px;
}

.bg-risk-label {
  margin-bottom: 8px;
  font-size: 13px;
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
}

.bg-risk-value {
  font-size: 36px;
  font-weight: 700;
  line-height: 1.2;
}

.bg-risk-value.risk-high {
  color: #f5222d;
}

.bg-risk-value.risk-medium {
  color: #fa8c16;
}

.bg-risk-value.risk-low {
  color: #52c41a;
}

.bg-risk-value.risk-safe {
  color: #1890ff;
}

.bg-risk-value.risk-default {
  color: var(--text-color-secondary, rgb(0 0 0 / 45%));
}

.bg-risk-bar {
  height: 6px;
  margin: 8px 16px;
  overflow: hidden;
  background-color: var(--border-color-base, #f0f0f0);
  border-radius: 3px;
}

.bg-risk-bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.6s ease;
}

.bg-risk-bar-fill.risk-high {
  background-color: #f5222d;
}

.bg-risk-bar-fill.risk-medium {
  background-color: #fa8c16;
}

.bg-risk-bar-fill.risk-low {
  background-color: #52c41a;
}

.bg-risk-bar-fill.risk-safe {
  background-color: #1890ff;
}

.bg-risk-bar-fill.risk-default {
  background-color: var(--text-color-secondary, rgb(0 0 0 / 25%));
}

.bg-suggestion-card {
  height: 100%;
  padding: 16px;
  background-color: var(--background-color-light, #fafafa);
  border: 1px solid var(--border-color-base, #f0f0f0);
  border-radius: 6px;
}

.bg-suggestion-label {
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
}

.bg-suggestion-content {
  font-size: 13px;
  line-height: 1.7;
  color: var(--heading-color, rgb(0 0 0 / 88%));
}

.basic-left {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
  width: 40%;
  min-width: 300px;
  max-width: 420px;
  overflow: hidden;
  background-color: var(--background-color-light, #fafafa);
  border: 1px solid var(--border-color-base, #f0f0f0);
  border-radius: 6px;
}

.basic-left-header {
  padding: 12px 16px;
  font-size: 14px;
  font-weight: 600;
  color: var(--heading-color, rgb(0 0 0 / 88%));
  background-color: var(--component-background, #fff);
  border-bottom: 1px solid var(--border-color-base, #f0f0f0);
}

.basic-timeline-wrap {
  flex: 1;
  max-height: 560px;
  padding: 16px;
  overflow-y: auto;
}

.basic-left-footer {
  padding: 8px 16px;
  text-align: center;
  background-color: var(--component-background, #fff);
  border-top: 1px solid var(--border-color-base, #f0f0f0);
}

/* 转移日志卡片样式 */
.transfer-log-card {
  padding: 10px 12px;
  font-size: 12px;
  background: linear-gradient(135deg, #f9f0ff 0%, #f5f0ff 100%);
  border: 1px solid #d3adf7;
  border-radius: 6px;
}

.transfer-flow {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 6px;
}

.transfer-arrow {
  font-size: 14px;
  font-weight: 600;
  color: #722ed1;
}

.transfer-reason,
.transfer-remark {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
  margin-top: 4px;
}

.reason-label,
.remark-label,
.affected-label {
  font-size: 11px;
  color: #8c8c8c;
}

.reason-value,
.remark-value {
  font-size: 12px;
  color: #262626;
}

.transfer-affected {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: flex-start;
  margin-top: 6px;
}

.affected-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
}

.basic-right {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
  background-color: var(--component-background, #fff);
  border: 1px solid var(--border-color-base, #f0f0f0);
  border-radius: 6px;
}

.basic-right-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-color-base, #f0f0f0);
}

.customer-basic-form {
  flex: 1;
  padding: 16px 20px;
  overflow-y: auto;
}

.customer-basic-form :deep(.ant-form-item) {
  margin-bottom: 16px;
}

@media (max-width: 1024px) {
  .basic-left {
    width: 100%;
    min-width: 0;
    max-width: none;
  }

  .basic-right {
    width: 100%;
  }

  :deep(.p-4) > div[style*='display: flex'] {
    flex-direction: column !important;
  }
}
</style>

<style scoped>
.basic-right-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  background-color: var(--component-background, #fff);
  border-top: 1px solid var(--border-color-base, #f0f0f0);
}

.bank-account-card {
  padding: 16px;
  margin-bottom: 12px;
  background-color: var(--background-color-light, #fafafa);
  border: 1px solid var(--border-color-base, #f0f0f0);
  border-radius: 8px;
  transition: all 0.2s ease;
}

.bank-account-card:hover {
  border-color: var(--border-color-base, #d9d9d9);
  box-shadow: 0 1px 4px rgb(0 0 0 / 6%);
}

.bg-change-highlight {
  padding: 0 2px;
  background-color: rgb(250 173 20 / 8%) !important;
  border-radius: 2px;
}

/* ====== 跟进记录布局 ====== */
.followup-layout {
  display: flex;
  gap: 20px;
  align-items: flex-start;
  width: 100%;
  padding: 16px 24px;
}

.followup-list {
  flex: 1;
  min-width: 0;
  padding: 20px 24px 8px;
  background: var(--background-color-light, #fafafa);
  border: 1px solid var(--border-color-base, #f0f0f0);
  border-radius: 8px;
}

.followup-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 10px;
  margin-bottom: 16px;
  border-bottom: 1px solid var(--border-color-base, #f0f0f0);
}

.followup-empty {
  padding: 60px 0;
  text-align: center;
}

.followup-timeline {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.followup-tl-item {
  position: relative;
  display: flex;
  gap: 12px;
  padding-bottom: 20px;
}

.followup-tl-dot {
  position: relative;
  z-index: 1;
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  margin-top: 5px;
  border-radius: 50%;
}

.followup-tl-item::before {
  position: absolute;
  top: 14px;
  bottom: 0;
  left: 3.5px;
  width: 1px;
  content: '';
  background: var(--border-color-base, #f0f0f0);
}

.followup-tl-item:last-child::before {
  display: none;
}

.followup-tl-body {
  flex: 1;
  min-width: 0;
}

.followup-tl-time {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 4px;
  font-size: 11px;
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
}

.followup-tl-tag {
  margin: 0;
  border-radius: 3px;
  transform: scale(0.9);
  transform-origin: left center;
}

.followup-tl-user {
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
}

.followup-tl-content {
  font-size: 13px;
  line-height: 1.7;
  color: var(--text-color, rgb(0 0 0 / 88%));
  word-break: break-all;
  white-space: pre-wrap;
}

.followup-tl-next {
  margin-top: 6px;
  font-size: 11px;
  color: #f59e0b;
}

.followup-form-wrap {
  position: sticky;
  top: 0;
  flex-shrink: 0;
  width: 320px;
}

.followup-form-card {
  padding: 14px 16px;
  background: var(--background-color-light, #fafafa);
  border: 1px solid var(--border-color-base, #f0f0f0);
  border-radius: 8px;
}

.followup-form-title {
  padding-bottom: 8px;
  margin-bottom: 12px;
  font-size: 13px;
  font-weight: 600;
  color: var(--heading-color, rgb(0 0 0 / 88%));
  border-bottom: 1px solid var(--border-color-base, #f0f0f0);
}

.followup-form-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.followup-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.followup-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
}

.followup-row {
  display: flex;
  gap: 10px;
}

.followup-half {
  flex: 1;
}

.followup-submit {
  width: 100%;
  margin-top: 4px;
}

@media (max-width: 900px) {
  .followup-layout {
    flex-direction: column;
  }

  .followup-list {
    order: 2;
    width: 100%;
  }

  .followup-form-wrap {
    position: static;
    order: 1;
    width: 100%;
  }
}

/* ====== 商机列表 ====== */
.opp-container {
  max-width: 1060px;
  padding: 16px 24px;
}

.opp-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  margin-bottom: 16px;
  border-bottom: 1px solid var(--border-color-base, #f0f0f0);
}

.opp-header-left {
  display: flex;
  gap: 10px;
  align-items: baseline;
}

.opp-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--heading-color, rgb(0 0 0 / 88%));
}

.opp-count {
  font-size: 12px;
  color: var(--text-color-secondary, rgb(0 0 0 / 45%));
}

.opp-create-btn {
  display: inline-flex;
  gap: 4px;
  align-items: center;
}

.opp-empty {
  padding: 60px 0;
  text-align: center;
}

.opp-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
  overflow: hidden;
  background: var(--border-color-base, #f0f0f0);
  border: 1px solid var(--border-color-base, #f0f0f0);
  border-radius: 8px;
}

.opp-card {
  background: var(--background-color-secondary, #fff);
  transition: background 0.15s;
}

.opp-card:hover {
  background: var(--background-color-light, #fafafa);
}

.opp-card-main {
  padding: 16px 20px;
}

.opp-card-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.opp-card-title-row {
  display: flex;
  gap: 10px;
  align-items: center;
}

.opp-card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--heading-color, rgb(0 0 0 / 88%));
}

.opp-card-actions {
  display: flex;
  gap: 2px;
  align-items: center;
  opacity: 0;
  transition: opacity 0.15s;
}

.opp-card:hover .opp-card-actions {
  opacity: 1;
}

.opp-card-details {
  display: flex;
  flex-wrap: wrap;
  gap: 0;
}

.opp-detail-item {
  display: flex;
  flex: 0 0 33.33%;
  gap: 6px;
  align-items: center;
  min-width: 150px;
  padding: 4px 0;
}

.opp-detail-label {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--text-color-secondary, rgb(0 0 0 / 45%));
}

.opp-detail-value {
  font-size: 12px;
  color: var(--text-color, rgb(0 0 0 / 88%));
}

.opp-amount {
  font-weight: 600;
  color: var(--color-primary, #1677ff);
}

.opp-pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  margin-top: 16px;
}

.opp-pagination-info {
  font-size: 12px;
  color: var(--text-color-secondary, rgb(0 0 0 / 45%));
}

.opp-pagination-btns {
  display: flex;
  gap: 8px;
  align-items: center;
}

.opp-page-indicator {
  min-width: 60px;
  font-size: 12px;
  color: var(--text-color-secondary, rgb(0 0 0 / 45%));
  text-align: center;
}

/* ========== 订单列表样式 ========== */
.order-list-wrap {
  padding: 16px 24px;
}

.order-cards {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.order-card {
  position: relative;
  padding: 16px 20px;
  overflow: hidden;
  background: #fff;
  border: 1px solid #f0f0f0;
  border-radius: 10px;
  transition: all 0.25s ease;
}

.order-card::before {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  width: 3px;
  content: '';
  background: linear-gradient(180deg, #1890ff, #69b1ff);
  opacity: 0;
  transition: opacity 0.25s ease;
}

.order-card:hover {
  border-color: #d6e4ff;
  box-shadow: 0 4px 16px rgb(24 144 255 / 10%);
  transform: translateY(-1px);
}

.order-card:hover::before {
  opacity: 1;
}

.order-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 10px;
  margin-bottom: 10px;
  border-bottom: 1px dashed #f0f0f0;
}

.order-card-header-left {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.order-no {
  font-family: 'SF Mono', Menlo, Monaco, Consolas, monospace;
  font-size: 13px;
  font-weight: 600;
  color: #262626;
  letter-spacing: 0.3px;
}

.order-date {
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: #8c8c8c;
}

.order-card-body {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
}

.order-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 14px;
  font-weight: 500;
  color: #595959;
  white-space: nowrap;
}

.order-meta {
  display: flex;
  flex-shrink: 0;
  gap: 24px;
}

.order-meta-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 80px;
}

.order-meta-label {
  font-size: 11px;
  color: #8c8c8c;
}

.order-amount {
  font-size: 15px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: #262626;
}

.order-paid {
  font-size: 13px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: #52c41a;
}

.order-unpaid {
  font-size: 13px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: #fa8c16;
}

.order-pagination {
  display: flex;
  justify-content: flex-end;
  padding-top: 12px;
  margin-top: 16px;
  border-top: 1px solid #f0f0f0;
}

/* ========== 合同列表样式 ========== */
.contract-list-wrap {
  padding: 16px 24px;
}

.contract-cards {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.contract-card {
  position: relative;
  padding: 16px 20px;
  overflow: hidden;
  background: #fff;
  border: 1px solid #f0f0f0;
  border-radius: 10px;
  transition: all 0.25s ease;
}

.contract-card::before {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  width: 3px;
  content: '';
  background: linear-gradient(180deg, #722ed1, #b37feb);
  opacity: 0;
  transition: opacity 0.25s ease;
}

.contract-card:hover {
  border-color: #d3adf7;
  box-shadow: 0 4px 16px rgb(114 46 209 / 10%);
  transform: translateY(-1px);
}

.contract-card:hover::before {
  opacity: 1;
}

.contract-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 10px;
  margin-bottom: 10px;
  border-bottom: 1px dashed #f0f0f0;
}

.contract-card-header-left {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.contract-no {
  font-family: 'SF Mono', Menlo, Monaco, Consolas, monospace;
  font-size: 13px;
  font-weight: 600;
  color: #262626;
  letter-spacing: 0.3px;
}

.contract-date {
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: #8c8c8c;
}

.contract-card-body {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
}

.contract-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 14px;
  font-weight: 500;
  color: #595959;
  white-space: nowrap;
}

.contract-meta {
  display: flex;
  flex-shrink: 0;
  gap: 24px;
}

.contract-meta-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 90px;
}

.contract-meta-label {
  font-size: 11px;
  color: #8c8c8c;
}

.contract-amount {
  font-size: 15px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: #262626;
}

.contract-exclude-tax {
  font-size: 13px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: #8c8c8c;
}

.contract-type {
  font-size: 13px;
  font-weight: 500;
  color: #722ed1;
}

.contract-pagination {
  display: flex;
  justify-content: flex-end;
  padding-top: 12px;
  margin-top: 16px;
  border-top: 1px solid #f0f0f0;
}

/* ========== 回款/退货/费用 只读列表样式 ========== */
.record-list-wrap {
  padding: 16px 24px;
}

.record-pagination {
  display: flex;
  justify-content: flex-end;
  padding-top: 12px;
  margin-top: 16px;
  border-top: 1px solid #f0f0f0;
}
</style>
