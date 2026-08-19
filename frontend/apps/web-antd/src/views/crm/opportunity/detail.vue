<script lang="ts" setup>
import type { SalesFlowMode } from '#/api';

import { computed, reactive, ref, watch } from 'vue';

import { LucidePhone, LucidePlus } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import {
  Avatar,
  Button,
  DatePicker,
  Form,
  Input,
  InputNumber,
  message,
  Modal,
  Radio,
  Select,
  Spin,
  Tag,
  Upload,
} from 'ant-design-vue';

import {
  convertOpportunityToOrderApi,
  convertOpportunityToQuotationApi,
  createFollowupApi,
  createOpportunityApi,
  getContactListApi,
  getCustomerContactsApi,
  getCustomerListApi,
  getFollowupListApi,
  getOpportunityInfoApi,
  getSalesFlowModeApi,
  updateOpportunityApi,
} from '#/api';

const props = defineProps<{
  customerId?: number | string;
  customerName?: string;
  id?: number | string;
}>();
const emit = defineEmits<{
  (e: 'converted', quotationId: number | string): void;
  (e: 'created', id: number | string): void;
}>();

const isCreate = computed(() => !props.id);
const saving = ref(false);
const converting = ref(false);
const opp = ref<any>({});

// 销售流程模式：A=仅标准(转报价单) B=仅简易(转订单) both=两种都允许
const flowMode = ref<SalesFlowMode>('both');
const loadFlowMode = async () => {
  try {
    flowMode.value = await getSalesFlowModeApi();
  } catch {
    flowMode.value = 'both';
  }
};
// 第 4 步显示文案与点击行为根据模式联动
const step4Label = computed(() => {
  if (flowMode.value === 'A') return '已报价';
  if (flowMode.value === 'B') return '已下单';
  return '已报价/下单';
});
const canStep4ToQuotation = computed(
  () => flowMode.value === 'A' || flowMode.value === 'both',
);
const canStep4ToOrder = computed(
  () => flowMode.value === 'B' || flowMode.value === 'both',
);
loadFlowMode();
// 新建模式下默认展示在「初步沟通」选项卡；编辑模式按数据加载
const activeTab = ref<string>(isCreate.value ? '1' : '3');

const currencyLabelMap: Record<number, string> = {
  1: '¥',
  2: '$',
  3: '€',
  4: '£',
  5: '¥',
  6: 'HK$',
  7: 'A$',
};

const sourceMap: Record<string, string> = {
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

const sourceOptions = Object.entries(sourceMap).map(([k, v]) => ({
  label: v,
  value: Number(k),
}));
const currencyOptions = [
  { label: '人民币 (CNY)', value: 1 },
  { label: '美元 (USD)', value: 2 },
  { label: '欧元 (EUR)', value: 3 },
  { label: '英镑 (GBP)', value: 4 },
  { label: '日元 (JPY)', value: 5 },
  { label: '港币 (HKD)', value: 6 },
  { label: '澳元 (AUD)', value: 7 },
];

const reqTypeOptions = [
  { label: '功能需求', value: 1 },
  { label: '性能需求', value: 2 },
  { label: '安全需求', value: 3 },
  { label: '其他需求', value: 4 },
];

const budgetOptions = [
  { label: '10万以下', value: 1 },
  { label: '10-50万', value: 2 },
  { label: '50-100万', value: 3 },
  { label: '100-500万', value: 4 },
  { label: '500万以上', value: 5 },
];

const solutionTypeOptions = [
  { label: '标准方案', value: 1 },
  { label: '定制方案', value: 2 },
  { label: '混合方案', value: 3 },
];

const demoTypeOptions = [
  { label: '线上演示', value: 1 },
  { label: '现场演示', value: 2 },
  { label: '视频会议', value: 3 },
];

const amountText = computed(() => {
  if (opp.value.amount === null || opp.value.amount === undefined)
    return '¥280,000';
  const num = Number(opp.value.amount).toLocaleString('en-US', {
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  });
  const currencyLabel = currencyLabelMap[opp.value.currency] || '¥';
  return `${currencyLabel}${num}`;
});

const probabilityNum = computed(() => Number(opp.value.probability ?? 50));

const sortedFollowUpRecords = computed(() => {
  return followUpRecords.value.toSorted((a, b) => {
    return new Date(b.time).getTime() - new Date(a.time).getTime();
  });
});

// 跟进记录：按设计图 4 条
const followUpRecords = ref<any[]>([
  {
    stage: 1,
    stageLabel: '初步沟通',
    time: '2024-12-15 10:30',
    user: '张伟',
    color: '#52c41a',
    content:
      '创建商机，完善基础信息：商机名称、所属客户、预算金额、预计成交日期、商机来源、赢单概率等。',
    tags: [{ text: '新建商机', color: 'green' }],
  },
  {
    stage: 3,
    stageLabel: '方案沟通',
    time: '2025-01-05 15:00',
    user: '张伟',
    color: '#7c3aed',
    content:
      '向客户演示了标准版ERP升级方案的PPT，重点展示了数据迁移的方案和系统架构设计。客户对整体升级方案表示认可，但对报价存在一定疑虑。',
    tags: [
      { text: '演示', color: 'purple' },
      { text: '方案v1', color: 'blue' },
    ],
  },
  {
    stage: 3,
    stageLabel: '方案沟通',
    time: '2024-12-28 11:00',
    user: '王芳',
    color: '#5b8ff9',
    content:
      '发送了初步方案文档（V1.0），包含三个模块的详细功能规划和技术架构图，客户反馈将在元旦后安排内部评审。',
    tags: [{ text: '附件', color: 'default' }],
  },
  {
    stage: 2,
    stageLabel: '需求确认',
    time: '2024-12-25 16:30',
    user: '张伟',
    color: '#5b8ff9',
    content:
      '与客户技术负责人线上进行了线上会议，确认了技术对接方案，客户现有数据库采用Oracle，需要考虑兼容性问题。',
    tags: [{ text: '线上版', color: 'green' }],
  },
  {
    stage: 2,
    stageLabel: '需求确认',
    time: '2024-12-22 10:00',
    user: '张伟',
    color: '#5b8ff9',
    content:
      '完成需求确认面谈后，开始整理方案框架，计划两周内完成初步方案文档。',
    tags: [{ text: '待跟进', color: 'orange' }],
  },
]);

// 关键联系人：按设计图 3 条
const contactList = ref<any[]>([
  {
    name: '李明辉',
    title: '总经理',
    mobile: '138-0000-1234',
    avatarColor: '#5b8ff9',
    tags: [{ text: '主要', color: 'orange' }],
  },
  {
    name: '王芳',
    title: 'IT部经理',
    mobile: '139-0000-5678',
    avatarColor: '#5b8ff9',
    tags: [{ text: '技术对接', color: 'blue' }],
  },
  {
    name: '陈志强',
    title: '财务部主管',
    mobile: '137-0000-9012',
    avatarColor: '#5ad8a6',
    tags: [{ text: '费用对接', color: 'purple' }],
  },
]);

// 联系人角色选项
const contactRoleOptions = [
  { label: '主要', value: 'primary' },
  { label: '技术对接', value: 'tech' },
  { label: '费用对接', value: 'finance' },
  { label: '商务对接', value: 'business' },
  { label: '其他', value: 'other' },
];

// 角色颜色映射
const contactRoleColorMap: Record<string, string> = {
  primary: 'orange',
  tech: 'blue',
  finance: 'purple',
  business: 'green',
  other: 'default',
};

// 角色文字映射
const contactRoleLabelMap: Record<string, string> = {
  primary: '主要',
  tech: '技术对接',
  finance: '费用对接',
  business: '商务对接',
  other: '其他',
};

// 联系人选择弹窗
const contactPickerVisible = ref(false);
const contactPickerLoading = ref(false);
const customerContactOptions = ref<any[]>([]);
const selectedContactIds = ref<string[]>([]);
const contactRoleMap = reactive<Record<string, string>>({});

function openContactPicker() {
  if (!opp.value.customerId) {
    message.warning('请先选择客户');
    return;
  }
  selectedContactIds.value = [];
  Object.keys(contactRoleMap).forEach((k) => delete contactRoleMap[k]);
  loadCustomerContacts();
  contactPickerVisible.value = true;
}

async function loadCustomerContacts() {
  contactPickerLoading.value = true;
  try {
    const res: any = await getContactListApi({
      page: 1,
      pageSize: 100,
      customerId: Number(opp.value.customerId),
    });
    const list: any[] = res?.items || res?.data?.list || res?.data?.items || [];
    customerContactOptions.value = list.map((c: any) => ({
      id: c.id,
      name: c.name || c.contactName,
      position: c.position || c.title,
      mobile: c.mobile,
      phone: c.phone,
      email: c.email,
    }));
  } catch {
    customerContactOptions.value = [];
  } finally {
    contactPickerLoading.value = false;
  }
}

function toggleContactSelection(id: number | string) {
  const idStr = String(id);
  const idx = selectedContactIds.value.indexOf(idStr);
  if (idx === -1) {
    selectedContactIds.value.push(idStr);
    if (!contactRoleMap[idStr]) {
      contactRoleMap[idStr] = 'tech';
    }
  } else {
    selectedContactIds.value.splice(idx, 1);
  }
}

const avatarColors = [
  '#5b8ff9',
  '#5ad8a6',
  '#f6bd16',
  '#ff9845',
  '#6ec8fc',
  '#7262fd',
  '#78d3f0',
  '#ff99c3',
];

function handleConfirmAddContacts() {
  if (selectedContactIds.value.length === 0) {
    message.warning('请至少选择一个联系人');
    return;
  }
  const existingIds = new Set(contactList.value.map((c: any) => String(c.id)));
  let added = 0;
  selectedContactIds.value.forEach((idStr) => {
    if (existingIds.has(idStr)) return;
    const contact = customerContactOptions.value.find(
      (c) => String(c.id) === idStr,
    );
    if (!contact) return;
    const role = contactRoleMap[idStr] || 'other';
    const colorIdx = contactList.value.length % avatarColors.length;
    contactList.value.push({
      id: contact.id,
      name: contact.name,
      title: contact.position || '-',
      mobile: contact.mobile,
      avatarColor: avatarColors[colorIdx],
      tags: [
        {
          text: contactRoleLabelMap[role] || '其他',
          color: contactRoleColorMap[role] || 'default',
        },
      ],
    });
    added++;
  });
  if (added > 0) {
    message.success(`已添加 ${added} 个关键联系人`);
  } else {
    message.info('所选联系人已在关键联系人列表中');
  }
  contactPickerVisible.value = false;
}

// ============ 表单数据 ============
const baseFormRef = ref();
const baseForm = reactive({
  title: '',
  customerId: undefined as number | string | undefined,
  contactId: undefined as number | undefined,
  assignedTo: undefined as number | undefined,
  amount: undefined as number | undefined,
  currency: 1,
  probability: undefined as number | undefined,
  source: undefined as number | undefined,
  expectedCloseDate: undefined as string | undefined,
  description: '',
});

const reqForm = reactive({
  reqType: undefined as number | undefined,
  reqDesc: '',
  priority: 'mid' as 'high' | 'low' | 'mid',
  expectDate: undefined as string | undefined,
  budgetRange: undefined as number | undefined,
});

const solForm = reactive({
  solutionType: undefined as number | undefined,
  solutionOverview: '',
  solutionHighlights: '',
  estimatedDuration: '',
  quoteAmount: undefined as number | undefined,
  demoDate: undefined as string | undefined,
  demoType: undefined as number | undefined,
});

const contactOptions = ref<{ label: string; value: number }[]>([]);

async function loadContacts(customerId: number) {
  try {
    const res: any = await getCustomerContactsApi(customerId);
    const items: any[] = res?.current || res?.data?.current || [];
    contactOptions.value = items.map((c: any) => ({
      label: c.name || c.contactName || '',
      value: Number(c.id || c.contactId),
    }));
  } catch {
    contactOptions.value = [];
  }
}

// 联系人选择弹窗（基本信息表单专用）
const contactPickerBasicVisible = ref(false);
const contactPickerBasicLoading = ref(false);
const contactPickerBasicOptions = ref<any[]>([]);
const selectedContactName = ref('');

function openContactPickerBasic() {
  if (!baseForm.customerId) {
    message.warning('请先选择所属企业');
    return;
  }
  contactPickerBasicLoading.value = true;
  contactPickerBasicVisible.value = true;
  getCustomerContactsApi(Number(baseForm.customerId))
    .then((res: any) => {
      const items: any[] = res?.current || res?.data?.current || [];
      contactPickerBasicOptions.value = items.map((c: any) => ({
        id: c.id || c.contactId,
        name: c.name || c.contactName || '',
        position: c.position || c.title || '',
        mobile: c.mobile || '',
        phone: c.phone || '',
        email: c.email || '',
      }));
    })
    .catch(() => {
      contactPickerBasicOptions.value = [];
    })
    .finally(() => {
      contactPickerBasicLoading.value = false;
    });
}

function selectContactFromPicker(contact: any) {
  baseForm.contactId = Number(contact.id);
  selectedContactName.value = contact.name;
  contactPickerBasicVisible.value = false;
}

function clearSelectedContact() {
  baseForm.contactId = undefined;
  selectedContactName.value = '';
}

// ============ 所属企业选择弹窗 ============
const customerPickerVisible = ref(false);
const customerPickerKeyword = ref('');
const customerPickerOptions = ref<any[]>([]);
const customerPickerLoading = ref(false);

async function loadCustomerPickerOptions(keyword?: string) {
  customerPickerLoading.value = true;
  try {
    const res: any = await getCustomerListApi({
      page: 1,
      pageSize: 20,
      ...(keyword ? { companyName: keyword } : {}),
    });
    customerPickerOptions.value = res?.items || [];
  } catch {
    customerPickerOptions.value = [];
  } finally {
    customerPickerLoading.value = false;
  }
}

function openCustomerPicker() {
  customerPickerVisible.value = true;
  customerPickerKeyword.value = '';
  loadCustomerPickerOptions('');
}

function handleCustomerPickerSearch() {
  loadCustomerPickerOptions(customerPickerKeyword.value);
}

function selectCustomerFromPicker(customer: any) {
  baseForm.customerId = customer.id;
  opp.value.customerName = customer.companyName;
  opp.value.customerId = customer.id;
  baseForm.contactId = undefined;
  contactOptions.value = [];
  if (customer.id) {
    loadContacts(Number(customer.id));
  }
  // 自动带出客户负责人作为商机负责人
  if (customer.assignedTo) {
    baseForm.assignedTo = customer.assignedTo;
  }
  customerPickerVisible.value = false;
}

function clearSelectedCustomer() {
  baseForm.customerId = undefined;
  opp.value.customerName = '';
  opp.value.customerId = undefined;
  baseForm.contactId = undefined;
  contactOptions.value = [];
}

const resetForm = () => {
  opp.value = {};
  activeTab.value = isCreate.value ? '1' : '3';
  baseForm.title = '';
  baseForm.customerId = undefined;
  baseForm.contactId = undefined;
  baseForm.assignedTo = undefined;
  selectedContactName.value = '';
  baseForm.amount = undefined;
  baseForm.currency = 1;
  baseForm.probability = undefined;
  baseForm.source = undefined;
  baseForm.expectedCloseDate = undefined;
  baseForm.description = '';
  reqForm.reqType = undefined;
  reqForm.reqDesc = '';
  reqForm.priority = 'mid';
  reqForm.expectDate = undefined;
  reqForm.budgetRange = undefined;
  solForm.solutionType = undefined;
  solForm.solutionOverview = '';
  solForm.solutionHighlights = '';
  solForm.estimatedDuration = '';
  solForm.quoteAmount = undefined;
  solForm.demoDate = undefined;
  solForm.demoType = undefined;
  contactOptions.value = [];
  // 从客户列表新建：预填充所属企业
  if (isCreate.value && props.customerId) {
    baseForm.customerId = Number(props.customerId);
    opp.value.customerId = Number(props.customerId);
    opp.value.customerName = props.customerName || '';
    loadContacts(Number(props.customerId));
  }
};

const loadData = async () => {
  if (!props.id) {
    resetForm();
    return;
  }
  try {
    const result: any = await getOpportunityInfoApi(Number(props.id));
    const data = result?.data || result || {};
    opp.value = data;

    const stage = Number(data.stage);
    activeTab.value =
      stage >= 1 && stage <= 5 ? String(Math.min(stage, 3)) : '3';

    baseForm.title = data.title || '';
    baseForm.customerId =
      data.customerId === null || data.customerId === undefined
        ? undefined
        : Number(data.customerId);
    baseForm.contactId =
      data.contactId === null || data.contactId === undefined
        ? undefined
        : Number(data.contactId);
    baseForm.assignedTo =
      data.assignedTo === null || data.assignedTo === undefined
        ? undefined
        : Number(data.assignedTo);
    baseForm.amount =
      data.amount === null || data.amount === undefined
        ? undefined
        : Number(data.amount);
    baseForm.currency =
      data.currency === null || data.currency === undefined
        ? 1
        : Number(data.currency);
    baseForm.probability =
      data.probability === null || data.probability === undefined
        ? undefined
        : Number(data.probability);
    baseForm.source =
      data.source === null || data.source === undefined
        ? undefined
        : Number(data.source);
    baseForm.expectedCloseDate = data.expectedCloseDate || undefined;
    baseForm.description = data.description || '';

    reqForm.reqDesc = data.requirementSummary || '';
    solForm.solutionOverview = data.solutionSummary || '';

    if (data.customerId) {
      loadContacts(Number(data.customerId));
    }

    await loadFollowupRecords();
  } catch {
    /* ignore */
  }
};

// 跟进方式颜色映射
const activityColorMap: Record<number, string> = {
  1: '#52c41a',
  2: '#1890ff',
  3: '#722ed1',
  4: '#fa8c16',
  5: '#25b864',
  6: '#52c41a',
  7: '#8c8c8c',
};

const activityLabelMap: Record<number, string> = {
  1: '电话',
  2: '拜访',
  3: '邮件',
  4: '会议',
  5: 'WhatsApp',
  6: '微信',
  7: '其他',
};

// 加载跟进记录
async function loadFollowupRecords() {
  if (!props.id) return;
  try {
    const res: any = await getFollowupListApi({
      page: 1,
      pageSize: 50,
      opportunityId: Number(props.id),
    });
    const list: any[] = res?.items || res?.data?.list || res?.data?.items || [];
    if (list.length > 0) {
      followUpRecords.value = list.map((item: any) => {
        const actType = Number(item.activityType) || 7;
        let actTagColor = 'default';
        switch (actType) {
          case 1: {
            actTagColor = 'green';
            break;
          }
          case 2: {
            actTagColor = 'blue';
            break;
          }
          case 3: {
            actTagColor = 'purple';
            break;
          }
          case 4: {
            actTagColor = 'orange';
            break;
          }
        }
        return {
          id: item.id,
          stage: 0,
          stageLabel: activityLabelMap[actType] || '其他',
          time: item.createTime ? formatDateTime(item.createTime) : '',
          user: item.createdByName || '未知',
          color: activityColorMap[actType] || '#8c8c8c',
          content: item.content || '',
          tags: [
            { text: activityLabelMap[actType] || '其他', color: actTagColor },
          ],
        };
      });
    }
  } catch {
    /* ignore */
  }
}

// 跟进方式选项
const activityTypeOptions = [
  { label: '电话', value: 1 },
  { label: '拜访', value: 2 },
  { label: '邮件', value: 3 },
  { label: '会议', value: 4 },
  { label: 'WhatsApp', value: 5 },
  { label: '微信', value: 6 },
  { label: '其他', value: 7 },
];

// 添加跟进记录弹窗
const followupModalVisible = ref(false);
const followupSaving = ref(false);
const followupFormRef = ref();
const followupForm = reactive({
  activityType: undefined as number | undefined,
  content: '',
  nextFollowDate: undefined as string | undefined,
  durationMinutes: undefined as number | undefined,
  result: '',
  stage: 1,
});

const stageLabelMap: Record<number, string> = {
  1: '初步沟通',
  2: '需求确认',
  3: '方案沟通',
};

const followupModalTitle = computed(() => {
  return `添加${stageLabelMap[followupForm.stage] || ''}跟进记录`;
});

const handleSaveFollowup = async () => {
  try {
    await followupFormRef.value?.validate();
  } catch {
    return;
  }
  followupSaving.value = true;
  try {
    await createFollowupApi({
      opportunityId: Number(props.id),
      customerId: opp.value.customerId,
      leadId: opp.value.leadId,
      activityType: followupForm.activityType,
      content: followupForm.content,
      nextFollowDate: followupForm.nextFollowDate,
      durationMinutes: followupForm.durationMinutes,
      result: followupForm.result,
    });
    message.success('跟进记录添加成功');
    followupModalVisible.value = false;
    await loadFollowupRecords();
  } catch {
    /* ignore */
  } finally {
    followupSaving.value = false;
  }
};

const handleSaveBase = async () => {
  try {
    await baseFormRef.value?.validate();
  } catch {
    return;
  }
  saving.value = true;
  try {
    const payload = {
      title: baseForm.title,
      customerId:
        baseForm.customerId === null || baseForm.customerId === undefined
          ? undefined
          : Number(baseForm.customerId),
      contactId: baseForm.contactId,
      assignedTo: baseForm.assignedTo,
      amount: baseForm.amount,
      currency: baseForm.currency,
      probability: baseForm.probability,
      source: baseForm.source,
      expectedCloseDate: baseForm.expectedCloseDate,
      description: baseForm.description,
      stage: 1,
    };

    if (isCreate.value) {
      const res: any = await createOpportunityApi(payload);
      const newId = res?.data?.id || res?.id;
      message.success('商机创建成功');
      if (newId !== null && newId !== undefined) {
        emit('created', newId);
      }
    } else {
      await updateOpportunityApi({ ...payload, id: Number(props.id) });
      try {
        await createFollowupApi({
          opportunityId: Number(props.id),
          customerId:
            baseForm.customerId === null || baseForm.customerId === undefined
              ? undefined
              : Number(baseForm.customerId),
          activityType: 7,
          content: `更新初步沟通信息：商机名称、客户、预算金额、预计成交日期等基础信息`,
        });
      } catch {
        /* ignore */
      }
      message.success('保存成功');
      await loadData();
    }
  } catch {
    /* ignore */
  } finally {
    saving.value = false;
  }
};

const handleSaveReq = async () => {
  saving.value = true;
  try {
    await updateOpportunityApi({
      id: Number(props.id),
      requirementSummary: reqForm.reqDesc,
      stage: 2,
    });
    try {
      await createFollowupApi({
        opportunityId: Number(props.id),
        customerId: opp.value.customerId,
        activityType: 7,
        content: `更新需求确认信息：${reqForm.reqDesc || '需求描述'}`,
      });
    } catch {
      /* ignore */
    }
    message.success('保存成功');
    await loadData();
  } catch {
    /* ignore */
  } finally {
    saving.value = false;
  }
};

const handleSubmitSolution = async () => {
  saving.value = true;
  try {
    await updateOpportunityApi({
      id: Number(props.id),
      solutionSummary: solForm.solutionOverview,
      stage: 3,
    });
    try {
      await createFollowupApi({
        opportunityId: Number(props.id),
        customerId: opp.value.customerId,
        activityType: 7,
        content: `提交方案沟通：${solForm.solutionOverview || '方案概述'}`,
      });
    } catch {
      /* ignore */
    }
    message.success('方案已提交');
    await loadData();
  } catch {
    /* ignore */
  } finally {
    saving.value = false;
  }
};

const handleConvertToQuotation = (): void => {
  handleStep4ToQuotation();
};

const handleStep4ToQuotation = (): void => {
  Modal.confirm({
    title: '转报价单',
    content: '确定要将该商机转为报价单吗？转换后将进入报价单新建页面。',
    okText: '确定',
    cancelText: '取消',
    onOk: async () => {
      converting.value = true;
      try {
        const res: any = await convertOpportunityToQuotationApi(
          Number(props.id),
        );
        message.success('已转为报价单');
        const quotationId =
          res?.data?.id ||
          res?.id ||
          res?.data?.quotationId ||
          res?.quotationId;
        if (quotationId !== null && quotationId !== undefined) {
          emit('converted', quotationId);
        }
        await loadData();
      } catch {
        /* ignore */
      } finally {
        converting.value = false;
      }
    },
  });
};

const handleStep4ToOrder = (): void => {
  Modal.confirm({
    title: '转订单',
    content:
      '确定要将该商机直接转为订单吗？转换后将创建订单草稿，可在订单页面继续完善明细。',
    okText: '确定',
    cancelText: '取消',
    onOk: async () => {
      converting.value = true;
      try {
        const res: any = await convertOpportunityToOrderApi(Number(props.id));
        message.success('已转为订单');
        const orderId =
          res?.data?.id || res?.id || res?.data?.orderId || res?.orderId;
        if (orderId !== null && orderId !== undefined) {
          emit('converted', orderId);
        }
        await loadData();
      } catch {
        /* ignore */
      } finally {
        converting.value = false;
      }
    },
  });
};

// 第 4 步点击：根据模式决定行为
const handleStep4Click = (): void => {
  if (isCreate.value) return;
  if (canStep4ToQuotation.value) {
    handleStep4ToQuotation();
  } else if (canStep4ToOrder.value) {
    handleStep4ToOrder();
  }
};

// 客户字段锁定：商机已转报价单或已转订单时锁定客户字段
const isCustomerLocked = computed(() => {
  if (isCreate.value) return false;
  return opp.value?.quoteStatus === 1 || opp.value?.orderStatus === 1;
});

// 引用以避免 noUnusedLocals 警告
void handleConvertToQuotation;

watch(
  () => props.id,
  () => {
    loadData();
  },
  { immediate: true },
);
</script>

<template>
  <div class="opp-detail">
    <!-- 商机信息卡（编辑模式才显示，新建时隐藏） -->
    <div v-if="!isCreate" class="opp-info-card">
      <div class="opp-info-main">
        <div class="opp-info-title-row">
          <span class="opp-company">{{ opp.customerName || 'XX科技' }}</span>
          <span class="opp-project">
            - {{ opp.title || 'ERP系统升级项目' }}</span
          >
          <Tag v-if="opp.opportunityNo" color="default" class="opp-no-tag">
            {{ opp.opportunityNo }}
          </Tag>
          <Tag v-else color="default" class="opp-no-tag"> OPP-2024-0092 </Tag>
        </div>
        <div class="opp-info-desc-row">
          <span class="opp-info-desc-item">
            <span class="opp-info-label">客户</span>
            <span class="opp-info-value">{{
              opp.customerName || 'XX科技有限公司'
            }}</span>
          </span>
          <span class="opp-info-desc-sep">|</span>
          <span class="opp-info-desc-item">
            <span class="opp-info-label">负责人</span>
            <span class="opp-info-value">{{ opp.assignee || '张伟' }}</span>
          </span>
          <span class="opp-info-desc-sep">|</span>
          <span class="opp-info-desc-item">
            <span class="opp-info-label">创建时间</span>
            <span class="opp-info-value">{{
              opp.createTime ? formatDateTime(opp.createTime) : '2024-12-15'
            }}</span>
          </span>
        </div>
        <div class="opp-info-detail-row">
          <div class="opp-info-detail-item">
            <div class="opp-info-detail-label">行业</div>
            <div class="opp-info-detail-value">
              {{ industryLabelMap[Number(opp.customerIndustry)] || '制造业' }}
            </div>
          </div>
          <div class="opp-info-detail-item">
            <div class="opp-info-detail-label">来源</div>
            <div class="opp-info-detail-value">
              {{ sourceMap[opp.source] || '展会' }}
            </div>
          </div>
          <div class="opp-info-detail-item">
            <div class="opp-info-detail-label">预计成交日期</div>
            <div class="opp-info-detail-value">
              {{ opp.expectedCloseDate || '2025-03-31' }}
            </div>
          </div>
          <div class="opp-info-detail-item">
            <div class="opp-info-detail-label">商机阶段</div>
            <Tag color="purple">方案沟通</Tag>
          </div>
          <div class="opp-info-detail-item">
            <div class="opp-info-detail-label">赢单概率</div>
            <div class="opp-info-detail-value prob">{{ probabilityNum }}%</div>
          </div>
          <div class="opp-info-detail-item">
            <div class="opp-info-detail-label">跟进次数</div>
            <div class="opp-info-detail-value">12次</div>
          </div>
        </div>
      </div>
      <div class="opp-info-extra">
        <div class="opp-info-amount-label">预计金额</div>
        <div class="opp-info-amount-value">{{ amountText }}</div>
      </div>
    </div>

    <!-- 5步进度条（编辑模式才显示，新建时隐藏） -->
    <div v-if="!isCreate" class="opp-steps">
      <div class="opp-step step-done">
        <div class="opp-step-number">1</div>
        <div class="opp-step-label">初步沟通</div>
      </div>
      <div class="opp-step-line line-done"></div>
      <div class="opp-step step-done">
        <div class="opp-step-number">2</div>
        <div class="opp-step-label">需求确认</div>
      </div>
      <div class="opp-step-line line-done"></div>
      <div class="opp-step step-current-purple">
        <div class="opp-step-number">3</div>
        <div class="opp-step-label">方案沟通</div>
      </div>
      <div class="opp-step-line">
        <span class="opp-step-arrow">›</span>
      </div>
      <div
        class="opp-step"
        :class="{
          'step-clickable':
            !isCreate && (canStep4ToQuotation || canStep4ToOrder),
        }"
        @click="handleStep4Click"
      >
        <div class="opp-step-number">4</div>
        <div class="opp-step-label">{{ step4Label }}</div>
      </div>
      <div class="opp-step-line">
        <span class="opp-step-arrow">›</span>
      </div>
      <div class="opp-step">
        <div class="opp-step-number">5</div>
        <div class="opp-step-label">成交/丢单</div>
      </div>
    </div>

    <!-- 新建模式下的标题 -->
    <div v-if="isCreate" class="opp-create-header">
      <span class="opp-create-title">新建商机</span>
      <span class="opp-create-subtitle"
        >填写商机基础信息，保存后可继续完善需求、方案等阶段内容</span
      >
    </div>

    <!-- 阶段 Tab 切换 -->
    <div class="opp-nav">
      <span
        class="opp-nav-item"
        :class="{ active: activeTab === '1' }"
        @click="activeTab = '1'"
        >初步沟通</span
      >
      <span
        class="opp-nav-item"
        :class="{ active: activeTab === '2' }"
        @click="activeTab = '2'"
        >需求确认</span
      >
      <span
        class="opp-nav-item nav-purple"
        :class="{ active: activeTab === '3' }"
        @click="activeTab = '3'"
        >方案沟通</span
      >
    </div>

    <!-- 主体：左右布局 -->
    <div class="opp-body">
      <!-- 左栏：阶段Tab + 表单 -->
      <div class="opp-main">
        <!-- Tab1: 初步沟通 -->
        <div v-show="activeTab === '1'" class="opp-tab-content">
          <div class="opp-form-header">
            <span class="opp-form-title">初步沟通记录</span>
          </div>
          <Form
            ref="baseFormRef"
            :model="baseForm"
            layout="vertical"
            class="opp-form"
          >
            <Form.Item
              label="商机名称"
              name="title"
              :rules="[{ required: true, message: '请输入商机名称' }]"
            >
              <Input
                v-model:value="baseForm.title"
                placeholder="请输入商机名称"
              />
            </Form.Item>
            <div class="opp-form-row">
              <Form.Item
                label="所属企业"
                name="customerId"
                class="opp-form-item"
                :rules="[{ required: true, message: '请选择所属企业' }]"
              >
                <div class="opp-customer-picker">
                  <Input
                    :value="baseForm.customerId ? opp.customerName || '' : ''"
                    :placeholder="
                      isCustomerLocked
                        ? '已转下游单据，客户不可修改'
                        : '点击右侧按钮选择客户'
                    "
                    readonly
                    class="opp-customer-picker-input"
                  />
                  <Button
                    type="primary"
                    size="small"
                    class="opp-customer-picker-btn"
                    :disabled="isCustomerLocked"
                    @click="openCustomerPicker"
                  >
                    选择客户
                  </Button>
                  <Button
                    v-if="baseForm.customerId && !isCustomerLocked"
                    type="link"
                    size="small"
                    danger
                    class="opp-customer-picker-clear"
                    @click="clearSelectedCustomer"
                  >
                    清除
                  </Button>
                </div>
              </Form.Item>
              <Form.Item
                label="联系人"
                name="contactId"
                class="opp-form-item"
                :rules="[{ required: true, message: '请选择联系人' }]"
              >
                <div class="opp-customer-picker">
                  <Input
                    :value="baseForm.contactId ? selectedContactName : ''"
                    :placeholder="
                      baseForm.customerId
                        ? '点击右侧按钮选择联系人'
                        : '请先选择所属企业'
                    "
                    readonly
                    class="opp-customer-picker-input"
                  />
                  <Button
                    type="primary"
                    size="small"
                    class="opp-customer-picker-btn"
                    :disabled="!baseForm.customerId"
                    @click="openContactPickerBasic"
                  >
                    选择联系人
                  </Button>
                  <Button
                    v-if="baseForm.contactId"
                    type="link"
                    size="small"
                    danger
                    class="opp-customer-picker-clear"
                    @click="clearSelectedContact"
                  >
                    清除
                  </Button>
                </div>
              </Form.Item>
            </div>
            <div class="opp-form-row">
              <Form.Item label="商机金额" name="amount" class="opp-form-item">
                <InputNumber
                  v-model:value="baseForm.amount"
                  :min="0"
                  :precision="2"
                  placeholder="请输入商机金额"
                  style="width: 100%"
                />
              </Form.Item>
              <Form.Item label="币种" name="currency" class="opp-form-item">
                <Select
                  v-model:value="baseForm.currency"
                  :options="currencyOptions"
                />
              </Form.Item>
            </div>
            <div class="opp-form-row">
              <Form.Item
                label="赢单概率"
                name="probability"
                class="opp-form-item"
              >
                <InputNumber
                  v-model:value="baseForm.probability"
                  :min="0"
                  :max="100"
                  placeholder="0-100"
                  style="width: 100%"
                >
                  <template #addonAfter>%</template>
                </InputNumber>
              </Form.Item>
              <Form.Item label="商机来源" name="source" class="opp-form-item">
                <Select
                  v-model:value="baseForm.source"
                  placeholder="请选择来源"
                  allow-clear
                  :options="sourceOptions"
                />
              </Form.Item>
            </div>
            <Form.Item label="预计成交日期" name="expectedCloseDate">
              <DatePicker
                v-model:value="baseForm.expectedCloseDate"
                placeholder="请选择预计成交日期"
                style="width: 100%"
                value-format="YYYY-MM-DD"
              />
            </Form.Item>
            <Form.Item label="商机描述" name="description">
              <Input.TextArea
                v-model:value="baseForm.description"
                placeholder="详细描述商机背景、客户需求、价值主张等"
                :rows="4"
                :maxlength="2000"
                show-count
              />
            </Form.Item>
          </Form>
          <div class="opp-form-footer">
            <Button type="primary" :loading="saving" @click="handleSaveBase">
              {{ isCreate ? '创建商机' : '保存' }}
            </Button>
          </div>
        </div>

        <!-- Tab2: 需求确认 -->
        <div v-show="activeTab === '2'" class="opp-tab-content">
          <div class="opp-form-header">
            <span class="opp-form-title">需求确认记录</span>
          </div>
          <Form :model="reqForm" layout="vertical" class="opp-form">
            <Form.Item label="需求类型" name="reqType">
              <Select
                v-model:value="reqForm.reqType"
                placeholder="请选择需求类型"
                allow-clear
                :options="reqTypeOptions"
              />
            </Form.Item>
            <Form.Item label="需求描述" name="reqDesc">
              <Input.TextArea
                v-model:value="reqForm.reqDesc"
                placeholder="请输入需求描述..."
                :rows="8"
                :maxlength="2000"
                show-count
              />
            </Form.Item>
            <Form.Item label="优先级" name="priority">
              <Radio.Group v-model:value="reqForm.priority">
                <Radio value="high">高</Radio>
                <Radio value="mid">中</Radio>
                <Radio value="low">低</Radio>
              </Radio.Group>
            </Form.Item>
            <Form.Item label="期望交付时间" name="expectDate">
              <DatePicker
                v-model:value="reqForm.expectDate"
                placeholder="年 / 月 / 日"
                style="width: 100%"
                value-format="YYYY-MM-DD"
              />
            </Form.Item>
            <Form.Item label="预算范围" name="budgetRange">
              <Select
                v-model:value="reqForm.budgetRange"
                placeholder="请选择预算范围"
                allow-clear
                :options="budgetOptions"
              />
            </Form.Item>
            <Form.Item label="需求文档">
              <Upload>
                <Button> <LucidePlus /> 点击或拖拽文件上传 </Button>
                <template #tip>
                  <div class="ant-upload-hint">
                    支持 PDF、Word、Excel 等格式，单个文件不超过 10MB
                  </div>
                </template>
              </Upload>
            </Form.Item>
          </Form>
          <div class="opp-form-footer">
            <Button type="primary" :loading="saving" @click="handleSaveReq">
              保存
            </Button>
          </div>
        </div>

        <!-- Tab3: 方案沟通 -->
        <div v-show="activeTab === '3'" class="opp-tab-content">
          <div class="opp-form-header">
            <span class="opp-form-title">方案沟通记录</span>
          </div>
          <Form :model="solForm" layout="vertical" class="opp-form">
            <Form.Item label="方案类型" name="solutionType">
              <Select
                v-model:value="solForm.solutionType"
                placeholder="请选择方案类型"
                allow-clear
                :options="solutionTypeOptions"
              />
            </Form.Item>
            <Form.Item label="方案概述" name="solutionOverview">
              <Input.TextArea
                v-model:value="solForm.solutionOverview"
                placeholder="请输入方案概述..."
                :rows="8"
                :maxlength="2000"
                show-count
              />
            </Form.Item>
            <Form.Item label="方案亮点">
              <Input
                v-model:value="solForm.solutionHighlights"
                placeholder="请输入方案亮点"
              />
              <div class="opp-highlight-tags">
                <Tag color="green">已发送文件</Tag>
              </div>
            </Form.Item>
            <div class="opp-form-row">
              <Form.Item label="预计工期" class="opp-form-item">
                <Input
                  v-model:value="solForm.estimatedDuration"
                  placeholder="请填写预计工期"
                  prefix="约"
                  suffix="周"
                />
              </Form.Item>
              <Form.Item label="报价金额" class="opp-form-item">
                <InputNumber
                  v-model:value="solForm.quoteAmount"
                  :min="0"
                  :precision="2"
                  placeholder="请输入报价金额"
                  style="width: 100%"
                >
                  <template #addonBefore>¥</template>
                </InputNumber>
              </Form.Item>
            </div>
            <Form.Item label="方案文档">
              <Upload>
                <div class="opp-upload-box">
                  <div class="opp-upload-icon">
                    <LucidePlus :size="20" />
                  </div>
                  <div class="opp-upload-text">点击或拖拽文件上传</div>
                  <div class="opp-upload-hint">
                    支持 PDF、Word、Excel 等格式，单个文件不超过 10MB
                  </div>
                </div>
              </Upload>
            </Form.Item>
            <div class="opp-form-demo">
              <div class="opp-demo-title">演示安排</div>
              <div class="opp-form-row">
                <Form.Item label="演示日期" class="opp-form-item">
                  <DatePicker
                    v-model:value="solForm.demoDate"
                    placeholder="年 / 月 / 日"
                    style="width: 100%"
                    value-format="YYYY-MM-DD"
                  />
                </Form.Item>
                <Form.Item label="演示方式" class="opp-form-item">
                  <Select
                    v-model:value="solForm.demoType"
                    placeholder="请选择演示方式"
                    allow-clear
                    :options="demoTypeOptions"
                  />
                </Form.Item>
              </div>
            </div>
          </Form>
          <div class="opp-form-footer">
            <Button
              type="primary"
              class="opp-submit-btn"
              :loading="saving"
              @click="handleSubmitSolution"
            >
              提交方案
            </Button>
          </div>
        </div>
      </div>

      <!-- 右栏：跟进记录 + 关键联系人 -->
      <div class="opp-side">
        <!-- 跟进记录 -->
        <div class="opp-right-section">
          <div class="opp-right-title">跟进记录</div>
          <div class="opp-timeline">
            <div
              v-for="(record, idx) in sortedFollowUpRecords"
              :key="idx"
              class="opp-tl-item"
            >
              <div
                class="opp-tl-dot"
                :style="{ backgroundColor: record.color }"
              ></div>
              <div class="opp-tl-body">
                <div class="opp-tl-time">
                  <Tag
                    v-if="record.stageLabel"
                    size="small"
                    :color="
                      record.stage === 1
                        ? 'green'
                        : record.stage === 2
                          ? 'blue'
                          : 'purple'
                    "
                    class="opp-tl-stage-tag"
                  >
                    {{ record.stageLabel }}
                  </Tag>
                  <span>{{ record.time }}</span>
                </div>
                <div class="opp-tl-user">
                  <Avatar
                    :size="20"
                    :style="{ backgroundColor: record.color, color: '#fff' }"
                  >
                    {{ record.user.charAt(0) }}
                  </Avatar>
                  <span class="opp-tl-name">{{ record.user }}</span>
                </div>
                <div class="opp-tl-content">{{ record.content }}</div>
                <div
                  v-if="record.tags && record.tags.length > 0"
                  class="opp-tl-tags"
                >
                  <Tag
                    v-for="t in record.tags"
                    :key="t.text"
                    :color="t.color"
                    size="small"
                    class="opp-tl-tag"
                  >
                    {{ t.text }}
                  </Tag>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 关键联系人 -->
        <div class="opp-right-section">
          <div class="opp-right-title">
            <span>关键联系人</span>
            <a
              v-if="!isCreate && opp.customerId"
              class="opp-add-contact"
              @click="openContactPicker"
              >+ 添加</a
            >
          </div>
          <div>
            <div
              v-for="(c, idx) in contactList"
              :key="idx"
              class="opp-contact-item"
            >
              <Avatar
                :size="36"
                :style="{ backgroundColor: c.avatarColor, color: '#fff' }"
              >
                {{ c.name.charAt(0) }}
              </Avatar>
              <div class="opp-contact-info">
                <div class="opp-contact-name">
                  {{ c.name }}
                  <Tag
                    v-for="t in c.tags"
                    :key="t.text"
                    :color="t.color"
                    size="small"
                    class="opp-tl-tag"
                  >
                    {{ t.text }}
                  </Tag>
                </div>
                <div class="opp-contact-title">{{ c.title }}</div>
              </div>
              <div class="opp-contact-links">
                <div v-if="c.mobile" class="opp-contact-link">
                  <LucidePhone :size="12" /> {{ c.mobile }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 选择联系人弹窗 -->
    <Modal
      v-model:open="contactPickerVisible"
      title="选择关键联系人"
      :width="600"
      :confirm-loading="contactPickerLoading"
      ok-text="确定添加"
      cancel-text="取消"
      @ok="handleConfirmAddContacts"
    >
      <div class="opp-contact-picker">
        <div class="opp-picker-tip">
          从「{{
            opp.customerName || '该客户'
          }}」的联系人中选择要标记为关键联系人的人员：
        </div>
        <div class="opp-picker-list">
          <div
            v-for="item in customerContactOptions"
            :key="item.id"
            class="opp-picker-item"
            :class="{ active: selectedContactIds.includes(String(item.id)) }"
            @click="toggleContactSelection(item.id)"
          >
            <div class="opp-picker-checkbox">
              <div
                class="opp-picker-check-inner"
                v-if="selectedContactIds.includes(String(item.id))"
              >
                ✓
              </div>
            </div>
            <Avatar :size="32" class="opp-picker-avatar">
              {{ item.name?.charAt(0) || '?' }}
            </Avatar>
            <div class="opp-picker-info">
              <div class="opp-picker-name">{{ item.name }}</div>
              <div class="opp-picker-meta">
                {{ item.position || item.title || '-' }} ·
                {{ item.mobile || item.phone || '-' }}
              </div>
            </div>
            <Select
              v-model:value="contactRoleMap[String(item.id)]"
              :options="contactRoleOptions"
              size="small"
              placeholder="选择角色"
              style="width: 100px"
              @click.stop
            />
          </div>
        </div>
        <div
          v-if="customerContactOptions.length === 0 && !contactPickerLoading"
          class="opp-picker-empty"
        >
          该客户下暂无联系人
        </div>
      </div>
    </Modal>

    <!-- 联系人选择弹窗（基本信息表单） -->
    <Modal
      v-model:open="contactPickerBasicVisible"
      title="选择联系人"
      :width="600"
      :footer="null"
      :destroy-on-close="true"
    >
      <Spin :spinning="contactPickerBasicLoading">
        <div class="opp-customer-picker-modal">
          <div class="opp-picker-tip">
            从「{{ opp.customerName || '该企业' }}」中选择联系人：
          </div>
          <div class="opp-customer-picker-list">
            <div
              v-for="item in contactPickerBasicOptions"
              :key="item.id"
              class="opp-customer-picker-row"
              :class="{
                active: String(baseForm.contactId) === String(item.id),
              }"
              @click="selectContactFromPicker(item)"
            >
              <div class="opp-customer-picker-info">
                <div class="opp-customer-picker-name">
                  {{ item.name || '未命名' }}
                </div>
                <div class="opp-customer-picker-meta">
                  <span v-if="item.position">职位: {{ item.position }}</span>
                  <span v-if="item.mobile">手机: {{ item.mobile }}</span>
                  <span v-if="item.phone">电话: {{ item.phone }}</span>
                  <span v-if="item.email">邮箱: {{ item.email }}</span>
                </div>
              </div>
              <Tag
                v-if="String(baseForm.contactId) === String(item.id)"
                color="green"
                class="opp-customer-picker-checked"
              >
                已选
              </Tag>
            </div>
            <div
              v-if="
                contactPickerBasicOptions.length === 0 &&
                !contactPickerBasicLoading
              "
              class="opp-customer-picker-empty"
            >
              该企业下暂无联系人
            </div>
          </div>
        </div>
      </Spin>
    </Modal>

    <!-- 添加跟进记录弹窗 -->
    <Modal
      v-model:open="followupModalVisible"
      :title="followupModalTitle"
      :width="560"
      :confirm-loading="followupSaving"
      ok-text="保存"
      cancel-text="取消"
      @ok="handleSaveFollowup"
    >
      <Form
        ref="followupFormRef"
        :model="followupForm"
        layout="vertical"
        class="opp-followup-form"
      >
        <Form.Item
          label="跟进方式"
          name="activityType"
          :rules="[{ required: true, message: '请选择跟进方式' }]"
        >
          <Select
            v-model:value="followupForm.activityType"
            placeholder="请选择跟进方式"
            :options="activityTypeOptions"
          />
        </Form.Item>
        <Form.Item
          label="跟进内容"
          name="content"
          :rules="[{ required: true, message: '请输入跟进内容' }]"
        >
          <Input.TextArea
            v-model:value="followupForm.content"
            :rows="5"
            placeholder="请输入跟进内容详情"
            allow-clear
          />
        </Form.Item>
        <div class="opp-form-row">
          <Form.Item label="下次跟进日期" class="opp-form-item">
            <DatePicker
              v-model:value="followupForm.nextFollowDate"
              placeholder="选择日期"
              style="width: 100%"
              value-format="YYYY-MM-DD"
            />
          </Form.Item>
          <Form.Item label="沟通时长（分钟）" class="opp-form-item">
            <InputNumber
              v-model:value="followupForm.durationMinutes"
              :min="0"
              placeholder="分钟"
              style="width: 100%"
            />
          </Form.Item>
        </div>
        <Form.Item label="沟通结果">
          <Input.TextArea
            v-model:value="followupForm.result"
            :rows="3"
            placeholder="请输入沟通结果（选填）"
            allow-clear
          />
        </Form.Item>
      </Form>
    </Modal>

    <!-- 所属企业选择弹窗 -->
    <Modal
      v-model:open="customerPickerVisible"
      title="选择所属企业"
      :width="640"
      :footer="null"
      :destroy-on-close="true"
    >
      <div class="opp-customer-picker-modal">
        <div class="opp-customer-picker-search">
          <Input
            v-model:value="customerPickerKeyword"
            placeholder="输入客户名称搜索"
            allow-clear
            class="opp-customer-picker-search-input"
            @press-enter="handleCustomerPickerSearch"
          />
          <Button type="primary" @click="handleCustomerPickerSearch">
            搜索
          </Button>
        </div>
        <Spin :spinning="customerPickerLoading">
          <div class="opp-customer-picker-list">
            <div
              v-for="item in customerPickerOptions"
              :key="item.id"
              class="opp-customer-picker-row"
              :class="{
                active: String(baseForm.customerId) === String(item.id),
              }"
              @click="selectCustomerFromPicker(item)"
            >
              <div class="opp-customer-picker-info">
                <div class="opp-customer-picker-name">
                  {{ item.companyName || '未命名客户' }}
                </div>
                <div class="opp-customer-picker-meta">
                  <span v-if="item.contactName"
                    >联系人: {{ item.contactName }}</span
                  >
                  <span v-if="item.mobile">手机: {{ item.mobile }}</span>
                  <span v-if="item.industry"
                    >行业:
                    {{
                      industryLabelMap[Number(item.industry)] ||
                      item.industry ||
                      '-'
                    }}</span
                  >
                </div>
              </div>
              <Tag
                v-if="String(baseForm.customerId) === String(item.id)"
                color="green"
                class="opp-customer-picker-checked"
              >
                已选
              </Tag>
            </div>
            <div
              v-if="
                customerPickerOptions.length === 0 && !customerPickerLoading
              "
              class="opp-customer-picker-empty"
            >
              暂无客户数据
            </div>
          </div>
        </Spin>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
.opp-detail {
  min-height: 100%;
  padding: 0 0 24px;
  background: hsl(var(--background));
}

/* 商机信息卡 */
.opp-info-card {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: flex-start;
  justify-content: space-between;
  padding: 20px 24px;
  margin: 16px 24px 0;
  background: hsl(var(--muted) / 40%);
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  box-shadow: 0 1px 2px rgb(0 0 0 / 3%);
}

.opp-info-main {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

.opp-info-title-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.opp-company {
  font-size: 16px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.opp-project {
  font-size: 16px;
  font-weight: 500;
  color: hsl(var(--muted-foreground));
}

.opp-no-tag {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  border-color: hsl(var(--border));
  border-radius: 4px;
  transform: scale(0.9);
  transform-origin: left center;
}

.opp-info-desc-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  font-size: 12px;
}

.opp-info-desc-item {
  display: flex;
  gap: 4px;
  align-items: center;
}

.opp-info-desc-sep {
  color: hsl(var(--border));
}

.opp-info-label {
  color: hsl(var(--muted-foreground));
}

.opp-info-value {
  color: hsl(var(--card-foreground) / 80%);
}

.opp-info-detail-row {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  align-items: center;
  margin-top: 2px;
}

.opp-info-detail-item {
  display: flex;
  gap: 6px;
  align-items: center;
  font-size: 12px;
}

.opp-info-detail-label {
  color: hsl(var(--muted-foreground));
}

.opp-info-detail-value {
  color: hsl(var(--card-foreground) / 80%);
}

.opp-info-detail-value.prob {
  font-weight: 500;
  color: #fa8c16;
}

.opp-info-extra {
  display: flex;
  flex-direction: column;
  gap: 4px;
  align-items: flex-end;
  justify-content: center;
}

.opp-info-amount-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.opp-info-amount-value {
  font-size: 18px;
  font-weight: 600;
  color: #fa8c16;
}

/* 5步进度条 */
.opp-steps {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 20px 40px 16px;
  margin: 12px 24px 0;
  background: hsl(var(--card));
  border: none;
  border-radius: 8px;
}

.opp-step {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
  gap: 4px;
  align-items: center;
  min-width: 60px;
}

.opp-step.step-clickable {
  cursor: pointer;
}

.opp-step.step-clickable:hover .opp-step-number {
  box-shadow: 0 2px 8px rgb(0 0 0 / 15%);
  transform: scale(1.1);
  transition: all 0.2s;
}

.opp-step-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  font-size: 12px;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--border));
  border-radius: 50%;
}

.opp-step-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
}

.opp-step.step-done .opp-step-number {
  color: #fff;
  background: #52c41a;
}

.opp-step.step-done .opp-step-label {
  font-weight: 500;
  color: #52c41a;
}

.opp-step.step-current-purple .opp-step-number {
  color: #fff;
  background: #7c3aed;
}

.opp-step.step-current-purple .opp-step-label {
  font-weight: 500;
  color: #7c3aed;
}

.opp-step-line {
  position: relative;
  flex: 1;
  min-width: 40px;
  max-width: 100px;
  height: 2px;
  margin-top: 12px;
  background: hsl(var(--border));
}

.opp-step-line::after {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 2px;
  content: '';
  background: repeating-linear-gradient(
    to right,
    hsl(var(--border)) 0 4px,
    transparent 4px 8px
  );
  opacity: 0;
}

.opp-step-line:not(.line-done)::after {
  opacity: 1;
}

.opp-step-line:not(.line-done) {
  background: transparent;
}

.opp-step-line .opp-step-arrow {
  position: absolute;
  top: 50%;
  left: 50%;
  z-index: 1;
  display: none;
  padding: 0 4px;
  font-size: 18px;
  line-height: 1;
  color: hsl(var(--border));
  background: hsl(var(--card));
  transform: translate(-50%, -50%);
}

.opp-step-line:not(.line-done) .opp-step-arrow {
  display: inline-block;
}

.opp-step-line.line-done {
  background: #52c41a;
}

/* Tab 切换 */
.opp-nav {
  display: flex;
  gap: 24px;
  align-items: center;
  padding: 0 24px;
  margin: 12px 24px 0;
  font-size: 13px;
  background: hsl(var(--card));
  border: none;
  border-bottom: 1px solid hsl(var(--border));
  border-radius: 0;
}

.opp-nav-item {
  padding: 10px 0;
  margin-bottom: -1px;
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.opp-nav-item:hover {
  color: hsl(var(--card-foreground));
}

.opp-nav-item.active {
  font-weight: 500;
  color: #52c41a;
  border-bottom-color: #52c41a;
}

.opp-nav-item.nav-purple.active {
  color: #7c3aed;
  border-bottom-color: #7c3aed;
}

/* 主体布局 */
.opp-body {
  display: flex;
  gap: 16px;
  align-items: flex-start;
  margin: 15px 24px 0;
}

.opp-side {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
  gap: 12px;
  width: 360px;
}

.opp-main {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  background: hsl(var(--muted) / 40%);
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.opp-tab-content {
  padding: 20px 24px 8px;
}

/* 表单 */
.opp-form-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.opp-form-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.opp-add-btn {
  border-radius: 4px;
}

.opp-form-actions {
  display: flex;
  gap: 8px;
}

.opp-form {
  max-width: 100%;
}

.opp-form-row {
  display: flex;
  gap: 16px;
}

.opp-form-item {
  flex: 1;
}

.opp-form-footer {
  display: flex;
  justify-content: center;
  padding: 16px 0 20px;
  margin-top: 8px;
  border-top: 1px solid hsl(var(--border));
}

.opp-submit-btn {
  min-width: 100px;
  background: #7c3aed;
  border-color: #7c3aed;
}

.opp-submit-btn:hover,
.opp-submit-btn:focus {
  background: #6d28d9;
  border-color: #6d28d9;
}

/* 方案亮点标签 */
.opp-highlight-tags {
  display: flex;
  gap: 6px;
  margin-top: 6px;
}

/* 上传框 */
.opp-upload-box {
  padding: 24px 16px;
  text-align: center;
  cursor: pointer;
  background: hsl(var(--card) / 50%);
  border: 1px dashed hsl(var(--border));
  border-radius: 6px;
  transition: all 0.2s;
}

.opp-upload-box:hover {
  border-color: #7c3aed;
}

.opp-upload-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
  color: hsl(var(--muted-foreground));
}

.opp-upload-text {
  margin-bottom: 4px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.opp-upload-hint {
  font-size: 11px;
  color: hsl(var(--muted-foreground) / 70%);
}

/* 演示安排 */
.opp-form-demo {
  padding-top: 8px;
  margin-top: 4px;
  border-top: 1px dashed hsl(var(--border));
}

.opp-demo-title {
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
}

/* 右栏 */
.opp-right-section {
  padding: 14px 16px;
  background: hsl(var(--muted) / 40%);
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
}

.opp-right-title {
  padding-bottom: 8px;
  margin-bottom: 12px;
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
  border-bottom: 1px solid hsl(var(--border));
}

/* 跟进记录时间轴 */
.opp-timeline {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.opp-tl-item {
  position: relative;
  display: flex;
  gap: 10px;
}

.opp-tl-dot {
  position: relative;
  z-index: 1;
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  margin-top: 5px;
  border-radius: 50%;
}

.opp-tl-item::before {
  position: absolute;
  top: 13px;
  bottom: -6px;
  left: 3.5px;
  width: 1px;
  content: '';
  background: hsl(var(--border));
}

.opp-tl-item:last-child::before {
  display: none;
}

.opp-tl-body {
  flex: 1;
  min-width: 0;
}

.opp-tl-time {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 4px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.opp-tl-stage-tag {
  margin: 0;
  border-radius: 3px;
  transform: scale(0.9);
  transform-origin: left center;
}

.opp-tl-user {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 4px;
}

.opp-tl-name {
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
}

.opp-tl-content {
  font-size: 12px;
  line-height: 1.6;
  color: hsl(var(--muted-foreground));
}

.opp-tl-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 6px;
}

.opp-tl-tag {
  margin: 0;
  border-radius: 3px;
}

/* 联系人 */
.opp-contact-item {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px dashed hsl(var(--border));
}

.opp-contact-item:last-child {
  border-bottom: none;
}

.opp-contact-info {
  flex: 1;
  min-width: 0;
}

.opp-contact-name {
  display: flex;
  gap: 6px;
  align-items: center;
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
}

.opp-contact-title {
  margin-top: 2px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.opp-contact-links {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  text-align: right;
}

.opp-contact-link {
  display: flex;
  gap: 4px;
  align-items: center;
  justify-content: flex-end;
  white-space: nowrap;
}

.opp-add-contact {
  float: right;
  font-size: 12px;
  font-weight: normal;
  color: #1890ff;
  cursor: pointer;
}

.opp-add-contact:hover {
  color: #40a9ff;
}

/* 联系人选择弹窗 */
.opp-contact-picker {
  max-height: 500px;
  overflow-y: auto;
}

.opp-picker-tip {
  margin-bottom: 12px;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
}

.opp-picker-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.opp-picker-item {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 10px 12px;
  cursor: pointer;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  transition: all 0.2s;
}

.opp-picker-item:hover {
  background: hsl(var(--primary) / 4%);
  border-color: #1890ff;
}

.opp-picker-item.active {
  background: hsl(var(--primary) / 6%);
  border-color: #1890ff;
}

.opp-picker-checkbox {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: 1.5px solid hsl(var(--border));
  border-radius: 3px;
}

.opp-picker-item.active .opp-picker-checkbox {
  background: #1890ff;
  border-color: #1890ff;
}

.opp-picker-check-inner {
  font-size: 11px;
  font-weight: bold;
  line-height: 1;
  color: #fff;
}

.opp-picker-avatar {
  flex-shrink: 0;
}

.opp-picker-info {
  flex: 1;
  min-width: 0;
}

.opp-picker-name {
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
}

.opp-picker-meta {
  margin-top: 2px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.opp-picker-empty {
  padding: 40px 0;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
  text-align: center;
}

/* 新建模式标题 */
.opp-create-header {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 16px 20px;
  margin: 16px 24px 0;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-left: 3px solid #1890ff;
  border-radius: 8px;
}

.opp-create-title {
  font-size: 16px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.opp-create-subtitle {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* 所属企业选择器（表单内） */
.opp-customer-picker {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
}

.opp-customer-picker-input {
  flex: 1;
}

.opp-customer-picker-input :deep(.ant-input-affix-wrapper-disabled),
.opp-customer-picker-input :deep(input[readonly]) {
  cursor: not-allowed;
  background: hsl(var(--muted) / 40%);
}

.opp-customer-picker-btn {
  flex-shrink: 0;
}

.opp-customer-picker-clear {
  flex-shrink: 0;
  padding: 0 4px !important;
}

/* 所属企业选择弹窗 */
.opp-customer-picker-modal {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.opp-customer-picker-search {
  display: flex;
  gap: 8px;
}

.opp-customer-picker-search-input {
  flex: 1;
}

.opp-customer-picker-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 480px;
  overflow-y: auto;
}

.opp-customer-picker-row {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  cursor: pointer;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  transition: all 0.2s;
}

.opp-customer-picker-row:hover {
  background: hsl(var(--primary) / 4%);
  border-color: #1890ff;
}

.opp-customer-picker-row.active {
  background: hsl(122deg 80% 50% / 6%);
  border-color: #52c41a;
}

.opp-customer-picker-info {
  flex: 1;
  min-width: 0;
}

.opp-customer-picker-name {
  margin-bottom: 4px;
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
}

.opp-customer-picker-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.opp-customer-picker-checked {
  flex-shrink: 0;
  margin: 0;
}

.opp-customer-picker-empty {
  padding: 60px 0;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
  text-align: center;
}
</style>
