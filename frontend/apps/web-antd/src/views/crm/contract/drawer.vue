<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import {
  Button,
  DatePicker,
  Input,
  InputNumber,
  Modal,
  Radio,
  Select,
  Table,
  Tabs,
  TabPane,
  Tag,
  Tooltip,
  message,
} from 'ant-design-vue';
import {
  createContractApi,
  deleteContractPaymentPlanApi,
  getContractPaymentPlanApi,
  saveContractPaymentPlanApi,
  updateContractApi,
} from '#/api';
import { requestClient } from '#/api/request';
import { getOrderInfoApi } from '#/api/core/sale/order';
import { getCommissionRuleOptionsApi, previewCommissionApi } from '#/api/core/finance/commission-rule';
import { getUserListApi } from '#/api/core/system/user';
import CustomerSelectModal from '../components/CustomerSelectModal.vue';
import OpportunitySelectModal from '../components/OpportunitySelectModal.vue';
import UserSelectModal from '../components/UserSelectModal.vue';

const props = defineProps<{
  /** 是否为只读模式（查看详情） */
  readonly?: boolean;
  /** 是否从订单创建（客户和订单信息不可修改） */
  fromOrder?: boolean;
}>();

const data = ref();
const loading = ref(false);
const isMaximized = ref(false);

// 是否为只读模式（由外部传入或根据行数据判断）
const isReadonly = computed(() => {
  // 外部强制只读
  if (props.readonly) return true;
  // 已提交审批后不可编辑（approvalStatus > 0 表示已进入审批流程）
  const row = data.value?.row;
  if (row && row.approvalStatus !== undefined && row.approvalStatus !== 0 && row.approvalStatus !== 4) {
    return true;
  }
  return false;
});

// 从订单创建时，客户和商机不可修改
const isFromOrder = computed(() => props.fromOrder || data.value?.fromOrder);

// 投影标题：编辑 vs 查看
const getTitle = computed(() => {
  if (isReadonly.value) {
    return $t('page.crm.contract.detailTitle', { moduleName: $t('page.crm.contract.title') });
  }
  return data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.contract.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.contract.title') });
});

const userOptions = ref<any[]>([]);

// ========== 选择器状态 ==========
// 已选中的客户（显示用）
const selectedCustomer = ref<{ id: number; name: string } | null>(null);
// 已选中的商机（显示用）
const selectedOpportunity = ref<{ id: number; name: string } | null>(null);
// 弹窗可见状态
const customerSelectVisible = ref(false);
const opportunitySelectVisible = ref(false);

// ========== 回款计划相关 ==========
const paymentPlans = ref<any[]>([]);
const activeTabKey = ref('basic');
// 生成方式
const generateMethod = ref<'manual' | 'sign' | 'invoice' | 'settle' | 'ship'>('manual');
// 逾期利率(%)
const overdueRate = ref<number | undefined>(undefined);

const paymentTypeOptions = [
  { label: '预付款', value: 1 },
  { label: '进度款', value: 2 },
  { label: '到货款', value: 3 },
  { label: '验收款', value: 4 },
  { label: '质保金', value: 5 },
  { label: '尾款', value: 6 },
];

const generateMethodOptions = [
  { label: '创建时，手动添加回款计划', value: 'manual' },
  { label: '签订后生成回款计划', value: 'sign' },
  { label: '开票后生成回款计划', value: 'invoice' },
  { label: '结算后生成回款计划', value: 'settle' },
  { label: '发货后生成回款计划', value: 'ship' },
];

const planColumns = [
  { title: '期次', dataIndex: 'sort', width: 60 },
  { title: '期次名称', dataIndex: 'periodName', width: 140 },
  { title: '款项类型', dataIndex: 'paymentType', width: 120 },
  { title: '计划金额', dataIndex: 'plannedAmount', width: 130 },
  { title: '金额占比', dataIndex: 'percentStr', width: 100 },
  { title: '计划回款日期', dataIndex: 'plannedDate', width: 150 },
  { title: '备注', dataIndex: 'remark' },
  { title: '操作', dataIndex: 'action', width: 70 },
];

// ========== 提成配置相关 ==========
const getContractMembersApi = (contractId: number) => {
  return requestClient.get('/api/system/contract/commission-members', { params: { id: contractId } });
};
const saveContractMembersApi = (data: any) => {
  return requestClient.post('/api/system/contract/commission-members/save', data);
};
const setContractRuleApi = (data: any) => {
  return requestClient.post('/api/system/contract/commission-rule/set', data);
};

const commissionRuleOptions = ref<any[]>([]);
const selectedRuleId = ref<number | null>(null);
const commissionMode = ref<number>(1);
const commissionMembers = ref<any[]>([]);
const roleTypeOptions = [
  { value: 1, label: '主签人' },
  { value: 2, label: '参与人' },
  { value: 3, label: '技术支持' },
  { value: 4, label: '其他' },
];
const _commissionModeOptions = [
  { value: 1, label: '按方案自动计算' },
  { value: 2, label: '手动指定分成' },
];
const previewResult = ref<any[]>([]);
const previewVisible = ref(false);
const userSelectVisible = ref(false);
const editingMemberIndex = ref<number | null>(null);

// ========== 订单商品明细（从订单创建合同时展示） ==========
const orderItems = ref<any[]>([]);
const orderInfo = ref<any>(null);

async function loadOrderInfo(orderId: number) {
  try {
    const res: any = await getOrderInfoApi(orderId);
    const data = res?.data ?? res ?? null;
    if (data) {
      orderInfo.value = data;
      orderItems.value = data.items || [];
    }
  } catch {
    orderInfo.value = null;
    orderItems.value = [];
  }
}

const memberColumns = [
  { title: '序号', key: 'index', width: 60, customRender: ({ index }: any) => index + 1 },
  { title: '人员姓名', key: 'userName', dataIndex: 'userName' },
  { title: '角色类型', key: 'roleType', dataIndex: 'roleType', width: 140 },
  { title: '分成比例(%)', key: 'shareRatio', dataIndex: 'shareRatio', width: 140 },
  { title: '排序', key: 'sort', dataIndex: 'sort', width: 100 },
  { title: '操作', key: 'action', width: 80 },
];

// 订单商品明细列定义
const orderItemColumns = [
  { title: '商品名称', dataIndex: 'productName', width: 180 },
  { title: '规格', dataIndex: 'spec', width: 120 },
  { title: '单位', dataIndex: 'unit', width: 60 },
  { title: '数量', dataIndex: 'quantity', width: 80 },
  { title: '单价', dataIndex: 'unitPrice', width: 100 },
  { title: '税率', dataIndex: 'taxRate', width: 80 },
  { title: '金额', dataIndex: 'amount', width: 120 },
];

// 回款计划汇总计算
const planSummary = computed(() => {
  const totalAmount = paymentPlans.value.reduce(
    (sum, p) => sum + (Number(p.plannedAmount) || 0),
    0,
  );
  const count = paymentPlans.value.length;
  return { totalAmount, count };
});

// 剩余未分配金额（基于合同总金额）
const remainingAmount = computed(() => {
  return contractTotalAmount.value - planSummary.value.totalAmount;
});

// 剩余占比
const remainingPercent = computed(() => {
  if (contractTotalAmount.value <= 0) return 0;
  return (remainingAmount.value / contractTotalAmount.value) * 100;
});

// 合同总金额缓存
const contractTotalAmount = ref(0);

async function syncContractTotal() {
  try {
    const values = await baseFormApi.getValues();
    contractTotalAmount.value = Number(values.totalAmount) || 0;
  } catch {
    contractTotalAmount.value = 0;
  }
}

function addPlan() {
  const sort = paymentPlans.value.length + 1;
  paymentPlans.value.push({
    periodName: '',
    paymentType: 1,
    plannedAmount: 0,
    plannedDate: undefined,
    remark: '',
    sort,
  });
}

function removePlan(index: number) {
  paymentPlans.value.splice(index, 1);
  paymentPlans.value.forEach((p, i) => {
    p.sort = i + 1;
  });
}

/** 平均分配 */
function distributeRemaining() {
  const remaining = remainingAmount.value;
  if (remaining <= 0) {
    message.warning('没有剩余可分配的金额');
    return;
  }
  if (paymentPlans.value.length === 0) {
    message.warning('请先添加至少一行回款计划');
    return;
  }
  const avg = remaining / paymentPlans.value.length;
  paymentPlans.value.forEach((p) => {
    p.plannedAmount = Number(Number(p.plannedAmount || 0 + avg).toFixed(2));
  });
  message.success('已平均分配剩余金额');
}

async function loadCommissionRuleOptions() {
  try {
    const res: any = await getCommissionRuleOptionsApi();
    const list = res?.data?.data || res?.data || res?.items || res || [];
    if (Array.isArray(list)) {
      commissionRuleOptions.value = list.map((item: any) => ({
        value: item.id,
        label: item.name || item.ruleName || item.planName || item.title,
      }));
    }
  } catch (e) {
    console.error('Failed to load commission rule options:', e);
  }
}

async function loadCommissionData(contractId: number) {
  try {
    const res: any = await getContractMembersApi(contractId);
    const data = res?.data?.data || res?.data || res || {};
    selectedRuleId.value = data.ruleId || data.commissionRuleId || null;
    commissionMode.value = data.mode || data.commissionMode || 1;
    const members = data.members || data.commissionMembers || [];
    if (Array.isArray(members)) {
      commissionMembers.value = members.map((m: any) => ({
        id: m.id,
        userId: m.userId,
        userName: m.userName || m.name || '',
        roleType: m.roleType ?? 1,
        shareRatio: m.shareRatio ? Number(m.shareRatio) * 100 : 0,
        sort: m.sort ?? 0,
      }));
    }
  } catch (e) {
    console.error('Failed to load commission data:', e);
  }
}

function _handleModeChange(value: number) {
  commissionMode.value = value;
}

function addCommissionMember() {
  const sort = commissionMembers.value.length + 1;
  commissionMembers.value.push({
    userId: undefined,
    userName: '',
    roleType: 1,
    shareRatio: 0,
    sort,
  });
}

function handleAddMemberBySelect() {
  editingMemberIndex.value = null;
  userSelectVisible.value = true;
}

function handleSelectUser(row: any) {
  const userId = row.id;
  const userName = row.nickName || row.realName || row.name || row.userName || '';

  if (editingMemberIndex.value === null) {
    const exists = commissionMembers.value.some((m) => m.userId === userId);
    if (exists) {
      message.warning('该员工已在分成列表中');
      return;
    }
    const sort = commissionMembers.value.length + 1;
    commissionMembers.value.push({
      userId,
      userName,
      roleType: 1,
      shareRatio: 0,
      sort,
    });
  } else {
    const index = editingMemberIndex.value;
    const exists = commissionMembers.value.some((m, i) => i !== index && m.userId === userId);
    if (exists) {
      message.warning('该员工已在分成列表中');
      return;
    }
    commissionMembers.value[index].userId = userId;
    commissionMembers.value[index].userName = userName;
  }

  userSelectVisible.value = false;
  editingMemberIndex.value = null;
}

function handleSelectMemberUser(index: number) {
  editingMemberIndex.value = index;
  userSelectVisible.value = true;
}

function removeCommissionMember(index: number) {
  commissionMembers.value.splice(index, 1);
  commissionMembers.value.forEach((m, i) => {
    m.sort = i + 1;
  });
}

function getTotalShareRatio() {
  const total = commissionMembers.value.reduce(
    (sum, m) => sum + (Number(m.shareRatio) || 0),
    0,
  );
  return total / 100;
}

async function previewCommission() {
  const row = data.value?.row;
  if (!row?.id) {
    message.warning('请先保存合同基本信息');
    return;
  }
  try {
    const res: any = await previewCommissionApi(row.id);
    const list = res?.data?.data || res?.data || res?.items || res || [];
    previewResult.value = Array.isArray(list) ? list : [];
    previewVisible.value = true;
  } catch (e) {
    console.error('Failed to preview commission:', e);
  }
}

const drawerClass = computed(() =>
  isMaximized.value ? 'w-[95vw]' : 'w-[75vw]',
);

// ========== 合同状态映射（系统自动驱动，非用户选择）==========
const contractStatusMap: Record<number, { label: string; color: string; description: string }> = {
  0: { label: '草稿', color: 'default', description: '已创建，待提交审批' },
  1: { label: '待审批', color: 'processing', description: '已提交，等待审批人处理' },
  2: { label: '审批中', color: 'warning', description: '正在多级审批流转中' },
  3: { label: '执行中', color: 'success', description: '审批通过，合同生效执行中' },
  4: { label: '已完成', color: 'cyan', description: '合同全部履行完毕' },
  5: { label: '已终止', color: 'error', description: '合同被终止作废' },
};

// 当前行的审批状态
const currentApprovalStatus = computed(() => data.value?.row?.approvalStatus ?? 0);
const currentStatusInfo = computed(() => contractStatusMap[currentApprovalStatus.value] || contractStatusMap[0]);

const contractTypeList = computed(() => [
  { value: 1, label: '销售合同' },
  { value: 2, label: '采购合同' },
  { value: 3, label: '服务合同' },
  { value: 4, label: '合作协议' },
  { value: 5, label: '其他' },
]);

const currencyList = computed(() => [
  { value: 1, label: 'CNY - 人民币' },
  { value: 2, label: 'USD - 美元' },
  { value: 3, label: 'EUR - 欧元' },
  { value: 4, label: 'GBP - 英镑' },
  { value: 5, label: 'JPY - 日元' },
  { value: 6, label: 'HKD - 港币' },
  { value: 7, label: 'AUD - 澳元' },
]);

const paymentMethodTypeList = computed(() => [
  { value: 1, label: '一次性收款' },
  { value: 2, label: '分期收款' },
]);

// ========== 客户/商机选择 ==========
function openCustomerSelect() {
  if (isReadonly.value) return;
  customerSelectVisible.value = true;
}

/** 客户选择回调 */
async function handleSelectCustomer(row: any) {
  selectedCustomer.value = { id: row.id, name: row.companyName || row.name };
  // 同步设置表单字段值，用于验证
  await baseFormApi.setValues({ _customerDisplay: row.companyName || row.name });
  // 清除之前选择的商机（因为换了客户）
  selectedOpportunity.value = null;
  await baseFormApi.setValues({ _opportunityDisplay: '' });
  customerSelectVisible.value = false;
}

async function handleClearCustomer() {
  selectedCustomer.value = null;
  selectedOpportunity.value = null;
  // 清除表单字段值
  await baseFormApi.setValues({ _customerDisplay: '', _opportunityDisplay: '' });
}

function openOpportunitySelect() {
  if (isReadonly.value) return;
  opportunitySelectVisible.value = true;
}

/** 商机选择回调 */
async function handleSelectOpportunity(row: any) {
  selectedOpportunity.value = { id: row.id, name: row.title || row.name };
  // 同步设置表单字段值，用于验证
  await baseFormApi.setValues({ _opportunityDisplay: row.title || row.name });
  // 选择商机后自动填充金额信息
  if (row.amount) {
    const v = await baseFormApi.getValues();
    if (!v.amount) {
      await baseFormApi.setValues({ amount: row.amount });
      calculateTotalAmount();
    }
  }
  opportunitySelectVisible.value = false;
}

async function handleClearOpportunity() {
  selectedOpportunity.value = null;
  // 清除表单字段值
  await baseFormApi.setValues({ _opportunityDisplay: '' });
}

async function loadUserOptions() {
  try {
    const result = await getUserListApi({ page: 1, pageSize: 1000 });
    if (result.data && result.data.items) {
      userOptions.value = result.data.items.map((item: any) => ({
        value: item.id,
        label: item.realName || item.userName,
      }));
    }
  } catch (e) {
    console.error('Failed to load user options:', e);
  }
}

// ========== Form Schema =========
const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    // ---- 基本信息区（一行两列） ----
    {
      component: 'Input',
      fieldName: 'title',
      label: '合同标题',
      rules: 'required',
    },
    {
      component: 'Input',
      fieldName: 'contractNo',
      label: '合同编号',
      componentProps: {
        disabled: true,
        placeholder: '保存后由系统根据编码规则自动生成',
        style: { width: '100%' },
      },
    },
    {
      component: 'Input',
      fieldName: '_customerDisplay',
      label: '客户',
      rules: 'required',
    },
    {
      component: 'Input',
      fieldName: '_opportunityDisplay',
      label: '关联商机',
    },

    // ---- 财务信息区 ----
    {
      component: 'InputNumber',
      fieldName: 'amount',
      label: '合同金额',
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
        min: 0,
        precision: 2,
        style: { width: '100%' },
        onChange: async () => {
          await calculateTotalAmount();
        },
      },
    },
    {
      component: 'Select',
      fieldName: 'currency',
      label: '币种',
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        options: currencyList,
      },
    },

    // ---- 收款方式 ----
    {
      component: 'RadioGroup',
      fieldName: 'paymentMethodType',
      defaultValue: 1,
      label: '收款方式',
      formItemClass: 'col-span-2',
      componentProps: {
        optionType: 'button',
        class: 'flex flex-wrap',
        options: paymentMethodTypeList,
        onChange: async (val: number) => {
          if (val === 2) activeTabKey.value = 'paymentPlan';
          await syncContractTotal();
        },
      },
    },

    // ---- 日期与条款 ----
    {
      component: 'DatePicker',
      fieldName: 'startDate',
      label: '合同开始日期',
      rules: 'required',
      componentProps: { placeholder: '选择日期', valueFormat: 'YYYY-MM-DD' },
    },
    {
      component: 'DatePicker',
      fieldName: 'endDate',
      label: '合同结束日期',
      rules: 'required',
      componentProps: { placeholder: '选择日期', valueFormat: 'YYYY-MM-DD' },
    },
    {
      component: 'DatePicker',
      fieldName: 'signDate',
      label: '签署日期',
      componentProps: { placeholder: '选择日期', valueFormat: 'YYYY-MM-DD' },
    },

    // ---- 负责人与文件 ----
    {
      component: 'Select',
      fieldName: 'assignedTo',
      label: '负责人',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          option.label.toLowerCase().includes(input.toLowerCase()),
        options: userOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'ourSignerId',
      label: '我方签署人',
      componentProps: {
        placeholder: '默认为订单创建人（业务员）',
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          option.label.toLowerCase().includes(input.toLowerCase()),
        options: userOptions,
      },
    },
    {
      component: 'Input',
      fieldName: 'theirSignerName',
      label: '对方签署人',
      componentProps: {
        placeholder: '默认为订单联系人',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'theirSignerPhone',
      label: '对方签署电话',
      componentProps: {
        placeholder: '对方签署人联系电话',
        allowClear: true,
      },
    },
    {
      component: 'Upload',
      fieldName: 'contractFile',
      label: '合同文件',
      formItemClass: 'col-span-2',
      componentProps: {
        accept: '.pdf,.doc,.docx',
        maxCount: 1,
        showUploadList: true,
        listType: 'text',
        beforeUpload: () => false,
      },
    },
    {
      component: 'Upload',
      fieldName: 'contractImages',
      label: '合同扫描件',
      formItemClass: 'col-span-2',
      componentProps: {
        accept: 'image/*',
        maxCount: 9,
        multiple: true,
        showUploadList: true,
        listType: 'picture-card',
        beforeUpload: () => false,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'remark',
      label: '备注',
      formItemClass: 'col-span-2',
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true, rows: 2 },
    },
  ],
});

async function calculateTotalAmount() {
  const values = await baseFormApi.getValues();
  const amount = Number(values.amount) || 0;
  const taxAmount = Number(values.taxAmount) || 0;
  const total = amount + taxAmount;
  await baseFormApi.setValues({ totalAmount: total });
  contractTotalAmount.value = total;
}

// ========== Drawer ==========
const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    // 只读模式下不显示确认按钮，但防御性处理
    if (isReadonly.value) {
      drawerApi.close();
      return;
    }

    const validate = await baseFormApi.validate();
    if (!validate.valid) {
      activeTabKey.value = 'basic';
      return;
    }

    // 校验必填：客户和合同编号
    const values = await baseFormApi.getValues();

    if (!values.title?.trim()) {
      message.warning('请输入合同标题');
      activeTabKey.value = 'basic';
      return;
    }
    if (!selectedCustomer.value?.id) {
      message.warning('请选择客户');
      activeTabKey.value = 'basic';
      return;
    }

    // 将客户ID和商机ID从 ref 同步到提交数据
    values.customerId = selectedCustomer.value.id;
    values.customerName = selectedCustomer.value.name;
    if (selectedOpportunity.value) {
      values.opportunityId = selectedOpportunity.value.id;
      values.opportunityName = selectedOpportunity.value.name;
    }

    // 分期收款校验
    const paymentMethodType = values.paymentMethodType ?? 1;
    if (paymentMethodType === 2) {
      if (paymentPlans.value.length === 0) {
        message.warning('分期收款模式下请至少添加一条回款计划');
        activeTabKey.value = 'paymentPlan';
        return;
      }
      for (let i = 0; i < paymentPlans.value.length; i++) {
        const p = paymentPlans.value[i];
        if (!p.periodName?.trim()) {
          message.warning(`第${i + 1}行：期次名称不能为空`);
          activeTabKey.value = 'paymentPlan';
          return;
        }
        if (!p.plannedAmount || Number(p.plannedAmount) <= 0) {
          message.warning(`第${i + 1}行：计划金额必须大于0`);
          activeTabKey.value = 'paymentPlan';
          return;
        }
      }
      const planTotal = paymentPlans.value.reduce(
        (s, p) => s + (Number(p.plannedAmount) || 0), 0,
      );
      const contractTotal = Number(values.totalAmount) || 0;
      if (contractTotal > 0 && Math.abs(planTotal - contractTotal) > 0.01) {
        const diff = Math.abs(planTotal - contractTotal).toFixed(2);
        const res = await new Promise<boolean>((resolve) => {
          Modal.confirm({
            title: '金额不匹配',
            content: `回款计划总额 ¥${planTotal.toFixed(2)} 与合同总金额 ¥${contractTotal.toFixed(2)} 相差 ¥${diff}，是否继续保存？`,
            okText: '继续保存',
            cancelText: '返回修改',
            onOk: () => resolve(true),
            onCancel: () => resolve(false),
          });
        });
        if (!res) return;
      }
    }

    // 手动分成模式校验
    if (commissionMode.value === 2) {
      if (commissionMembers.value.length === 0) {
        message.warning('手动分成模式下请至少添加一位分成人员');
        activeTabKey.value = 'commission';
        return;
      }
      const total = getTotalShareRatio();
      if (Math.abs(total - 1) > 0.001) {
        message.warning(`分成比例合计必须为100%，当前为${(total * 100).toFixed(2)}%`);
        activeTabKey.value = 'commission';
        return;
      }
      for (let i = 0; i < commissionMembers.value.length; i++) {
        const m = commissionMembers.value[i];
        if (!m.userName?.trim()) {
          message.warning(`第${i + 1}行：人员姓名不能为空`);
          activeTabKey.value = 'commission';
          return;
        }
      }
    }

    // 将提成配置同步到提交数据
    values.commissionRuleId = selectedRuleId.value;
    values.commissionMode = commissionMode.value;

    setLoading(true);
    const isCreate = data.value?.create;

    try {
      const result: any = isCreate
        ? await createContractApi(values)
        : await updateContractApi({ ...values, id: data.value.row.id });

      const contractId = isCreate
        ? result?.data?.id || result?.data?.data?.id || result?.id
        : data.value.row.id;

      if (paymentMethodType === 2) {
        if (contractId && paymentPlans.value.length > 0) {
          await saveContractPaymentPlanApi({
            contractId,
            plans: paymentPlans.value.map((plan) => ({
              stageName: plan.periodName,
              paymentType: plan.paymentType,
              plannedAmount: Number(plan.plannedAmount) || 0,
              plannedDate: plan.plannedDate,
              remark: plan.remark,
              sort: plan.sort,
            })),
          });
        }
      } else if (paymentMethodType === 1 && !isCreate && contractId) {
        await deleteContractPaymentPlanApi(contractId);
      }

      // 保存提成配置
      if (contractId) {
        if (commissionMode.value === 2 && commissionMembers.value.length > 0) {
          await saveContractMembersApi({
            contractId,
            members: commissionMembers.value.map((m) => ({
              id: m.id,
              userId: m.userId,
              userName: m.userName,
              roleType: m.roleType,
              shareRatio: (Number(m.shareRatio) || 0) / 100,
              sort: m.sort,
            })),
          });
        }
      }

      message.success(
        isCreate ? $t('ui.notification.create_success') : $t('ui.notification.update_success'),
      );
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } catch {
      // 全局拦截器处理
    } finally {
      setLoading(false);
    }
  },

  async onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};

      // 重置
      paymentPlans.value = [];
      activeTabKey.value = 'basic';
      generateMethod.value = 'manual';
      overdueRate.value = undefined;
      contractTotalAmount.value = 0;
      selectedCustomer.value = null;
      selectedOpportunity.value = null;
      commissionRuleOptions.value = [];
      selectedRuleId.value = null;
      commissionMode.value = 1;
      commissionMembers.value = [];
      previewResult.value = [];
      previewVisible.value = false;
      orderItems.value = [];
      orderInfo.value = null;

      await Promise.all([loadUserOptions(), loadCommissionRuleOptions()]);

      // 恢复已选择的客户和商机显示
      if (row.customerId) {
        selectedCustomer.value = { id: row.customerId, name: row.customerName || '' };
        // 同步到表单字段用于验证
        row._customerDisplay = row.customerName || '';
      }
      if (row.opportunityId) {
        selectedOpportunity.value = { id: row.opportunityId, name: row.opportunityName || row.opportunityTitle || '' };
        // 同步到表单字段用于验证
        row._opportunityDisplay = row.opportunityName || row.opportunityTitle || '';
      }

      if (data.value?.create) {
        row.status = 1; // 默认草稿
        row.paymentMethodType = 1;
      } else {
        // 加载已有回款计划
        if (row.id) {
          try {
            const res: any = await getContractPaymentPlanApi(row.id);
            const plans = res?.data?.data || res?.data || res?.items || res || [];
            if (Array.isArray(plans) && plans.length > 0) {
              paymentPlans.value = plans.map((plan: any) => ({
                id: plan.id,
                periodName: plan.stageName || plan.periodName || '',
                paymentType: plan.paymentType ?? 1,
                plannedAmount: Number(plan.planAmount ?? plan.plannedAmount ?? 0),
                plannedDate: plan.planDate ?? plan.plannedDate ?? undefined,
                remark: plan.remark || '',
                sort: plan.sort ?? 0,
              }));
              row.paymentMethodType = 2;
            }
          } catch { /* ignore */ }
        }
        // 加载提成配置
        if (row.id) {
          try {
            await loadCommissionData(row.id);
          } catch { /* ignore */ }
        }
        contractTotalAmount.value = Number(row.totalAmount) || 0;
      }

      // 如果关联了订单，加载订单商品明细
      if (row.orderId) {
        await loadOrderInfo(Number(row.orderId));
      }

      baseFormApi.setValues(row);
      setLoading(false);
    }
  },
});

function setLoading(loadingState: boolean) {
  loading.value = loadingState;
  drawerApi.setState({ loading: loadingState });
}

function toggleMaximize() {
  isMaximized.value = !isMaximized.value;
}
</script>

<template>
  <Drawer :title="getTitle" :class="drawerClass">
    <template #extra>
      <!-- 最大化按钮 -->
      <button
        type="button"
        class="w-8 h-8 flex items-center justify-center text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-md transition-colors"
        @click="toggleMaximize"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
        </svg>
      </button>
    </template>

    <!-- ====== 只读模式：顶部状态条 ====== -->
    <div v-if="isReadonly || !data?.create" class="mb-4 flex items-center gap-3">
      <span class="text-sm text-gray-500">当前状态：</span>
      <Tag :color="currentStatusInfo.color" class="text-sm px-3 py-0.5">
        {{ currentStatusInfo.label }}
      </Tag>
      <span class="text-xs text-gray-400">{{ currentStatusInfo.description }}</span>
      <!-- 审批中锁定提示 -->
      <Tag v-if="isReadonly && currentApprovalStatus > 0 && currentApprovalStatus !== 4" color="warning" class="text-xs">
        🔒 审批流程中，信息不可修改
      </Tag>
    </div>

    <!-- ====== Tabs 导航 ====== -->
    <Tabs v-model:activeKey="activeTabKey" class="contract-drawer-tabs">
      <!-- ====== Tab 1: 基本信息 ====== -->
      <TabPane key="basic" tab="基本信息" force-render>
        <BaseForm>
          <!-- 客户选择 slot -->
          <template #_customerDisplay="{ model }">
            <div class="flex items-center gap-2 w-full">
              <Input
                v-if="!isReadonly && !isFromOrder"
                :value="selectedCustomer?.name || ''"
                placeholder="点击选择客户"
                readonly
                class="flex-1 cursor-pointer select-modal-input"
                @click="openCustomerSelect"
              >
                <template #suffix>
                  <Button type="link" size="small" class="!p-0 !text-blue-600 font-medium" @click.stop="openCustomerSelect">选择</Button>
                </template>
              </Input>
              <span v-else class="flex-1 text-gray-800 truncate">{{ selectedCustomer?.name || '-' }}</span>
              <Button v-if="!isReadonly && !isFromOrder && selectedCustomer" type="link" danger size="small" class="shrink-0 !p-0" @click.stop="handleClearCustomer">清除</Button>
            </div>
          </template>

          <!-- 商机选择 slot -->
          <template #_opportunityDisplay="{ model }">
            <div class="flex items-center gap-2 w-full" @click="openOpportunitySelect">
              <Input
                v-if="!isReadonly"
                :value="selectedOpportunity?.name || ''"
                placeholder="点击选择商机"
                readonly
                class="flex-1 cursor-pointer select-modal-input"
              >
                <template #suffix>
                  <Button type="link" size="small" class="!p-0 !text-blue-600 font-medium" @click.stop="openOpportunitySelect">选择</Button>
                </template>
              </Input>
              <span v-else class="flex-1 text-gray-800 truncate">{{ selectedOpportunity?.name || '-' }}</span>
              <Button v-if="!isReadonly && selectedOpportunity" type="link" danger size="small" class="shrink-0 !p-0" @click.stop="handleClearOpportunity">清除</Button>
            </div>
          </template>
        </BaseForm>

        <!-- 订单商品明细（从订单创建合同时展示） -->
        <div v-if="orderItems.length > 0" class="order-items-section">
          <div class="order-items-header">
            <span class="text-sm font-semibold text-gray-700">订单商品明细</span>
            <span v-if="orderInfo?.orderNo" class="text-xs text-gray-400 ml-2">订单号：{{ orderInfo.orderNo }}</span>
          </div>
          <Table
            :columns="orderItemColumns"
            :data-source="orderItems"
            :pagination="false"
            size="small"
            bordered
            :row-key="(_record: any, index: number) => `item_${index}`"
            class="order-items-table"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.dataIndex === 'unitPrice'">
                ¥{{ Number(record.unitPrice || 0).toFixed(2) }}
              </template>
              <template v-else-if="column.dataIndex === 'amount'">
                ¥{{ Number(record.amount || 0).toFixed(2) }}
              </template>
              <template v-else-if="column.dataIndex === 'taxRate'">
                {{ record.taxRate != null ? Number(record.taxRate).toFixed(0) + '%' : '-' }}
              </template>
            </template>
          </Table>
          <div v-if="orderInfo" class="order-items-summary">
            <span class="text-xs text-gray-500">
              商品总额：¥{{ Number(orderInfo.productAmount || 0).toFixed(2) }}
            </span>
            <span class="text-xs text-gray-500 ml-4">
              税额：¥{{ Number(orderInfo.taxAmount || 0).toFixed(2) }}
            </span>
            <span class="text-xs font-semibold text-gray-700 ml-4">
              订单总额：¥{{ Number(orderInfo.totalAmount || 0).toFixed(2) }}
            </span>
          </div>
        </div>
      </TabPane>

      <!-- ====== Tab 2: 回款计划 ====== -->
      <TabPane key="paymentPlan" tab="回款计划">
        <div class="w-full space-y-4">
          <!-- 提示 -->
          <div class="flex items-start gap-2 text-sm text-gray-500 bg-blue-50/50 rounded-lg px-4 py-3 border border-blue-100">
            <svg class="w-4 h-4 mt-0.5 text-blue-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span>请添加合同的回款计划，用于智能提醒到期、逾期情况</span>
            <Tooltip title="系统会根据回款计划自动计算应收账款，并在到期前提醒相关人员。支持按签订、开票、结算、发货等节点触发。">
              <span class="ml-auto text-blue-500 cursor-help underline decoration-dotted">如何自动生成？</span>
            </Tooltip>
          </div>

          <!-- 工具栏：只读时隐藏操作项 -->
          <div v-if="!isReadonly" class="flex items-center gap-4 flex-wrap">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium text-gray-600 whitespace-nowrap">生成方式:</span>
              <Select v-model:value="generateMethod" :options="generateMethodOptions" style="width: 240px" />
            </div>
            <div class="flex items-center gap-2 ml-auto">
              <span class="text-sm text-gray-500 whitespace-nowrap">设定逾期利率:</span>
              <InputNumber v-model:value="overdueRate" :min="0" :max="100" :precision="2" addon-after="%" placeholder="可选" style="width: 140px" />
            </div>
          </div>

          <!-- 表格 -->
          <Table
            :columns="planColumns"
            :data-source="paymentPlans"
            :pagination="false"
            bordered
            size="small"
            :row-key="(_record: any, index: number) => `plan_${index}`"
            :scroll="{ x: 900 }"
            class="payment-plan-table"
          >
            <template #bodyCell="{ column, record, index }">
              <template v-if="column.dataIndex === 'sort'">
                <span class="text-gray-400 text-sm">{{ index + 1 }}</span>
              </template>
              <template v-else-if="column.dataIndex === 'periodName'">
                <Input v-if="!isReadonly" v-model:value="record.periodName" placeholder="如：第1期" size="small" />
                <span v-else>{{ record.periodName || '-' }}</span>
              </template>
              <template v-else-if="column.dataIndex === 'paymentType'">
                <Select v-if="!isReadonly" v-model:value="record.paymentType" :options="paymentTypeOptions" placeholder="选择" size="small" style="width: 100%" />
                <span v-else>{{ paymentTypeOptions.find(o => o.value === record.paymentType)?.label || '-' }}</span>
              </template>
              <template v-else-if="column.dataIndex === 'plannedAmount'">
                <InputNumber v-if="!isReadonly" v-model:value="record.plannedAmount" :min="0" :precision="2" size="small" style="width: 100%" placeholder="金额" />
                <span v-else>¥{{ Number(record.plannedAmount || 0).toLocaleString('zh-CN', { minimumFractionDigits: 2 }) }}</span>
              </template>
              <template v-else-if="column.dataIndex === 'percentStr'">
                <span class="text-sm" :class="contractTotalAmount > 0 && record.plannedAmount ? 'text-gray-700' : 'text-gray-300'">
                  {{ contractTotalAmount > 0 && record.plannedAmount ? ((record.plannedAmount / contractTotalAmount) * 100).toFixed(1) + '%' : '-' }}
                </span>
              </template>
              <template v-else-if="column.dataIndex === 'plannedDate'">
                <DatePicker v-if="!isReadonly" v-model:value="record.plannedDate" value-format="YYYY-MM-DD" placeholder="选择日期" size="small" style="width: 100%" />
                <span v-else>{{ record.plannedDate || '-' }}</span>
              </template>
              <template v-else-if="column.dataIndex === 'remark'">
                <Input v-if="!isReadonly" v-model:value="record.remark" placeholder="备注" size="small" />
                <span v-else>{{ record.remark || '-' }}</span>
              </template>
              <template v-else-if="column.dataIndex === 'action'">
                <Button v-if="!isReadonly" type="link" danger size="small" @click="removePlan(index)">删除</Button>
              </template>
            </template>
          </Table>

          <!-- 添加行（仅编辑模式） -->
          <div v-if="!isReadonly" class="flex justify-start">
            <Button type="dashed" block class="w-40" @click="addPlan">+ 添加行</Button>
          </div>

          <!-- 空状态 -->
          <div v-if="isReadonly && paymentPlans.length === 0" class="text-center py-8 text-gray-400 text-sm">
            暂无回款计划
          </div>

          <!-- 底部汇总栏 -->
          <div v-if="paymentPlans.length > 0" class="rounded-lg border px-4 py-3" :class="
            Math.abs(remainingPercent) <= 1 ? 'border-green-200 bg-green-50/50'
            : remainingPercent > 0 ? 'border-orange-200 bg-orange-50/50'
            : 'border-red-200 bg-red-50/50'
          ">
            <div class="flex items-center gap-6 flex-wrap text-sm">
              <div>
                <span class="text-gray-500">计划总额：</span>
                <span class="font-semibold text-gray-800">¥{{ planSummary.totalAmount.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}</span>
              </div>
              <div>
                <span class="text-gray-500">合同总额：</span>
                <span class="font-semibold text-gray-800">¥{{ contractTotalAmount.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}</span>
              </div>
              <div>
                <span class="text-gray-500">剩余</span>
                <span class="font-semibold ml-1" :class="
                  Math.abs(remainingPercent) <= 1 ? 'text-green-600'
                  : remainingPercent > 0 ? 'text-orange-600' : 'text-red-600'
                ">{{ remainingPercent > 0 ? '+' : '' }}{{ remainingPercent.toFixed(1) }}%</span>
                <span class="text-gray-500 ml-1">（¥{{ Math.abs(remainingAmount).toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}）</span>
              </div>
              <Button v-if="!isReadonly && remainingPercent > 1" type="link" size="small" class="ml-auto" @click="distributeRemaining">平均分配剩余金额</Button>
              <span v-else-if="Math.abs(remainingPercent) <= 1" class="ml-auto text-green-600 text-xs font-medium">✓ 金额匹配</span>
              <span v-else class="ml-auto text-red-500 text-xs font-medium">⚠ 超出合同金额</span>
            </div>
          </div>
        </div>
      </TabPane>

      <!-- ====== Tab 3: 提成配置 ====== -->
      <TabPane key="commission" tab="提成配置" force-render>
        <div class="commission-config">
          <div class="form-row">
            <div class="form-label">提成方案</div>
            <div class="form-content">
              <Select
                v-model:value="selectedRuleId"
                :options="commissionRuleOptions"
                :disabled="isReadonly"
                placeholder="请选择提成方案（不选则用默认方案）"
                allowClear
                style="width: 320px"
              />
              <Button type="link" size="small" class="ml-2" @click="previewCommission">试算提成</Button>
            </div>
          </div>

          <div class="form-row">
            <div class="form-label">提成模式</div>
            <div class="form-content">
              <Radio.Group v-model:value="commissionMode" :disabled="isReadonly">
                <Radio :value="1">按方案自动计算</Radio>
                <Radio :value="2">手动指定分成</Radio>
              </Radio.Group>
            </div>
          </div>

          <div v-if="commissionMode === 2" class="mt-4">
            <div class="flex items-center mb-2">
              <span class="font-medium">分成人员配置</span>
              <span class="ml-2 text-sm" :class="Math.abs(getTotalShareRatio() - 1) <= 0.001 ? 'text-green-600' : 'text-red-500'">
                合计：{{ (getTotalShareRatio() * 100).toFixed(2) }}%
              </span>
              <Button v-if="!isReadonly" type="primary" size="small" class="ml-auto" @click="handleAddMemberBySelect">添加人员</Button>
            </div>

            <Table
              :columns="memberColumns"
              :data-source="commissionMembers"
              :pagination="false"
              size="small"
              bordered
              :row-key="(_record: any, index: number) => `member_${index}`"
            >
              <template #bodyCell="{ column, record, index }">
                <template v-if="column.key === 'userName'">
                  <div v-if="!isReadonly" class="flex items-center gap-2">
                    <span class="flex-1 truncate">{{ record.userName || '-' }}</span>
                    <Button type="link" size="small" class="!p-0 shrink-0" @click="handleSelectMemberUser(index)">
                      {{ record.userName ? '更换' : '选择' }}
                    </Button>
                  </div>
                  <span v-else>{{ record.userName || '-' }}</span>
                </template>
                <template v-else-if="column.key === 'roleType'">
                  <Select v-model:value="record.roleType" :disabled="isReadonly" :options="roleTypeOptions" style="width: 120px" />
                </template>
                <template v-else-if="column.key === 'shareRatio'">
                  <InputNumber v-model:value="record.shareRatio" :disabled="isReadonly" :min="0" :max="100" :step="1" style="width: 100px" />
                  <span class="ml-1">%</span>
                </template>
                <template v-else-if="column.key === 'sort'">
                  <InputNumber v-model:value="record.sort" :disabled="isReadonly" :min="0" style="width: 80px" />
                </template>
                <template v-else-if="column.key === 'action'">
                  <Button v-if="!isReadonly" type="link" danger size="small" @click="removeCommissionMember(index)">删除</Button>
                </template>
              </template>
            </Table>
          </div>
        </div>
      </TabPane>
    </Tabs>

    <!-- ====== 客户选择弹窗（独立组件） ====== -->
    <CustomerSelectModal
      v-model:visible="customerSelectVisible"
      @select="handleSelectCustomer"
    />

    <!-- ====== 商机选择弹窗（独立组件） ====== -->
    <OpportunitySelectModal
      v-model:visible="opportunitySelectVisible"
      :customer-id="selectedCustomer?.id"
      @select="handleSelectOpportunity"
    />

    <!-- ====== 员工选择弹窗（独立组件） ====== -->
    <UserSelectModal
      v-model:visible="userSelectVisible"
      @select="handleSelectUser"
    />

    <!-- ====== 提成试算弹窗 ====== -->
    <Modal v-model:open="previewVisible" title="提成试算结果" width="700px" :footer="null">
      <Table :data-source="previewResult" :pagination="false" size="small">
        <Table.Column title="人员" dataIndex="userName" key="userName" />
        <Table.Column title="方案类型" data-index="ruleType" key="ruleType" />
        <Table.Column title="计算基数" data-index="calcBaseAmount" key="calcBaseAmount" />
        <Table.Column title="提成比例" data-index="commissionRate" key="commissionRate" />
        <Table.Column title="分成比例" data-index="shareRatio" key="shareRatio" />
        <Table.Column title="提成金额" data-index="commissionAmount" key="commissionAmount" />
      </Table>
    </Modal>
  </Drawer>
</template>

<style scoped>
.contract-drawer-tabs :deep(.ant-tabs-nav) {
  margin-bottom: 16px;
  padding: 0 4px;
}
.contract-drawer-tabs :deep(.ant-tabs-tab) {
  font-size: 15px;
  padding: 8px 20px;
  font-weight: 500;
}
/* 选择输入框样式 */
.select-modal-input :deep(.ant-input) {
  background-color: #fafafa;
  border-color: #d9d9d9;
  cursor: pointer;
}
.select-modal-input :deep(.ant-input:hover) {
  border-color: #1677ff;
}
/* 选择弹窗中的行悬停效果 */
:deep(.ant-table-tbody > tr.ant-row-hover:hover > td) {
  background-color: #e6f4ff !important;
}
/* 提成配置样式 */
.commission-config {
  padding: 8px 4px;
}
.form-row {
  display: flex;
  align-items: flex-start;
  margin-bottom: 16px;
}
.form-label {
  width: 100px;
  flex-shrink: 0;
  padding-top: 6px;
  color: #666;
  text-align: right;
  padding-right: 12px;
}
.form-content {
  flex: 1;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
}

/* 回款计划表格 - 无数据时高度150px，有数据时按内容自适应 */
.payment-plan-table :deep(.ant-table) {
  height: auto;
}
/* 无数据时设置占位高度 */
.payment-plan-table :deep(.ant-table-placeholder) {
  min-height: 150px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.payment-plan-table :deep(.ant-table-placeholder .ant-empty) {
  margin: 0;
}
/* 有数据时移除固定高度，按内容自适应 */
.payment-plan-table :deep(.ant-table-body) {
  max-height: none !important;
  overflow-y: visible !important;
}
/* 表格内容区域 */
.payment-plan-table :deep(.ant-table-content) {
  min-height: auto;
}

/* 订单商品明细区域 */
.order-items-section {
  margin-top: 16px;
  padding: 12px 16px;
  background: var(--background-color-light, #fafafa);
  border: 1px solid var(--border-color-base, #f0f0f0);
  border-radius: 8px;
}
.order-items-header {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color-base, #f0f0f0);
}
.order-items-table :deep(.ant-table) {
  height: auto;
}
.order-items-table :deep(.ant-table-body) {
  max-height: none !important;
  overflow-y: visible !important;
}
.order-items-summary {
  display: flex;
  align-items: center;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--border-color-base, #f0f0f0);
}
</style>
