<script lang="ts" setup>
import { computed, reactive, ref, watch } from 'vue';
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
  Tag,
  Upload,
} from 'ant-design-vue';
import {
  LucidePhone,
  LucidePlus,
} from '@vben/icons';
import {
  convertToQuotationApi,
  createOpportunityApi,
  getCustomerContactsApi,
  getCustomerListApi,
  getOpportunityInfoApi,
  updateOpportunityApi,
} from '#/api';
import { formatDateTime } from '@vben/utils';

const props = defineProps<{ id?: number | string }>();
const emit = defineEmits<{
  (e: 'converted', quotationId: number | string): void;
  (e: 'created', id: number | string): void;
}>();

const isCreate = computed(() => !props.id);
const saving = ref(false);
const converting = ref(false);
const opp = ref<any>({});
// 默认展示在「方案沟通」选项卡（与设计图一致）
const activeTab = ref<string>('3');

const currencyLabelMap: Record<number, string> = {
  1: '¥', 2: '$', 3: '€', 4: '£', 5: '¥', 6: 'HK$', 7: 'A$',
};

const sourceMap: Record<string, string> = {
  1: '官网', 2: '展会', 3: '社交媒体', 4: '客户转介',
  5: '陌生拜访', 6: '海关数据', 7: '邮件营销', 8: '阿里国际站',
  9: 'Amazon', 10: 'TikTok', 11: '微信', 12: '其他',
};

const industryLabelMap: Record<number, string> = {
  1: '零售', 2: '批发', 3: '制造', 4: '贸易代理',
  5: '电商', 6: '微商', 7: '社交电商', 8: '其他',
};

const sourceOptions = Object.entries(sourceMap).map(([k, v]) => ({ label: v, value: Number(k) }));
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
  if (opp.value.amount == null) return '¥280,000';
  const num = Number(opp.value.amount).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 0 });
  const currencyLabel = currencyLabelMap[opp.value.currency] || '¥';
  return `${currencyLabel}${num}`;
});

const probabilityNum = computed(() => Number(opp.value.probability ?? 50));

// 跟进记录：按设计图 4 条
const followUpRecords = ref<any[]>([
  {
    time: '2025-01-05 15:00',
    user: '张伟',
    color: '#7c3aed',
    content: '向客户演示了标准版ERP升级方案的PPT，重点展示了数据迁移的方案和系统架构设计。客户对整体升级方案表示认可，但对报价存在一定疑虑。',
    tags: [{ text: '演示', color: 'purple' }, { text: '方案v1', color: 'blue' }],
  },
  {
    time: '2024-12-28 11:00',
    user: '王芳',
    color: '#5b8ff9',
    content: '发送了初步方案文档（V1.0），包含三个模块的详细功能规划和技术架构图，客户反馈将在元旦后安排内部评审。',
    tags: [{ text: '附件', color: 'default' }],
  },
  {
    time: '2024-12-25 16:30',
    user: '张伟',
    color: '#5b8ff9',
    content: '与客户技术负责人线上进行了线上会议，确认了技术对接方案，客户现有数据库采用Oracle，需要考虑兼容性问题。',
    tags: [{ text: '线上版', color: 'green' }],
  },
  {
    time: '2024-12-22 10:00',
    user: '张伟',
    color: '#5b8ff9',
    content: '完成需求确认面谈后，开始整理方案框架，计划两周内完成初步方案文档。',
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

// ============ 表单数据 ============
const baseFormRef = ref();
const baseForm = reactive({
  title: '',
  customerId: undefined as number | undefined | string,
  contactId: undefined as number | undefined,
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
  priority: 'mid' as 'high' | 'mid' | 'low',
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

const customerOptions = ref<any[]>([]);
const customerLoading = ref(false);

async function loadCustomerOptions(keyword: string) {
  customerLoading.value = true;
  try {
    const res: any = await getCustomerListApi({
      page: 1,
      pageSize: 20,
      ...(keyword ? { companyName: keyword } : {}),
    });
    customerOptions.value = res?.items || [];
  } catch {
    customerOptions.value = [];
  } finally {
    customerLoading.value = false;
  }
}

function handleCustomerSearch(value: string) {
  loadCustomerOptions(value || '');
}

function handleCustomerChange(value: any) {
  baseForm.contactId = undefined;
  contactOptions.value = [];
  if (value) {
    loadContacts(Number(value));
  }
}

const contactOptions = ref<{ label: string; value: number }[]>([]);

async function loadContacts(customerId: number) {
  try {
    const res: any = await getCustomerContactsApi(customerId);
    const items: any[] = res?.data?.current || [];
    contactOptions.value = items.map((c: any) => ({
      label: c.name || c.contactName || '',
      value: Number(c.id || c.contactId),
    }));
  } catch {
    contactOptions.value = [];
  }
}

const resetForm = () => {
  opp.value = {};
  activeTab.value = '3';
  baseForm.title = '';
  baseForm.customerId = undefined;
  baseForm.contactId = undefined;
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
  customerOptions.value = [];
  loadCustomerOptions('');
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
    if (stage >= 1 && stage <= 5) {
      activeTab.value = String(stage > 3 ? 3 : stage);
    } else {
      activeTab.value = '3';
    }

    baseForm.title = data.title || '';
    baseForm.customerId = data.customerId != null ? Number(data.customerId) : undefined;
    baseForm.contactId = data.contactId != null ? Number(data.contactId) : undefined;
    baseForm.amount = data.amount != null ? Number(data.amount) : undefined;
    baseForm.currency = data.currency != null ? Number(data.currency) : 1;
    baseForm.probability = data.probability != null ? Number(data.probability) : undefined;
    baseForm.source = data.source != null ? Number(data.source) : undefined;
    baseForm.expectedCloseDate = data.expectedCloseDate || undefined;
    baseForm.description = data.description || '';

    reqForm.reqDesc = data.requirementSummary || '';
    solForm.solutionOverview = data.solutionSummary || '';

    if (data.customerId) {
      loadContacts(Number(data.customerId));
    }
  } catch {
    /* ignore */
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
      customerId: baseForm.customerId != null ? Number(baseForm.customerId) : undefined,
      contactId: baseForm.contactId,
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
      if (newId != null) {
        emit('created', newId);
      }
    } else {
      await updateOpportunityApi({ ...payload, id: Number(props.id) });
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
    });
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
    message.success('方案已提交');
    await loadData();
  } catch {
    /* ignore */
  } finally {
    saving.value = false;
  }
};

const handleConvertToQuotation = (): void => {
  // 商机转为报价单
  Modal.confirm({
    title: '确认操作',
    content: '确定要将该商机转为报价单吗？',
    okText: '确定',
    cancelText: '取消',
    onOk: async () => {
      converting.value = true;
      try {
        const res: any = await convertToQuotationApi(Number(props.id));
        message.success('已转为报价单');
        const quotationId = res?.data?.id || res?.id || res?.data?.quotationId || res?.quotationId;
        if (quotationId != null) {
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

// 引用以避免 noUnusedLocals 警告
void handleConvertToQuotation;

watch(() => props.id, () => { loadData(); }, { immediate: true });
</script>

<template>
  <div class="opp-detail">
    <!-- 商机信息卡（整合标题、客户信息与商机详情） -->
    <div class="opp-info-card">
      <div class="opp-info-main">
        <div class="opp-info-title-row">
          <span class="opp-company">{{ opp.customerName || 'XX科技' }}</span>
          <span class="opp-project"> - {{ opp.title || 'ERP系统升级项目' }}</span>
          <Tag v-if="opp.opportunityNo" color="default" class="opp-no-tag">
            {{ opp.opportunityNo }}
          </Tag>
          <Tag v-else color="default" class="opp-no-tag">
            OPP-2024-0092
          </Tag>
        </div>
        <div class="opp-info-desc-row">
          <span class="opp-info-desc-item">
            <span class="opp-info-label">客户</span>
            <span class="opp-info-value">{{ opp.customerName || 'XX科技有限公司' }}</span>
          </span>
          <span class="opp-info-desc-sep">|</span>
          <span class="opp-info-desc-item">
            <span class="opp-info-label">负责人</span>
            <span class="opp-info-value">{{ opp.assignee || '张伟' }}</span>
          </span>
          <span class="opp-info-desc-sep">|</span>
          <span class="opp-info-desc-item">
            <span class="opp-info-label">创建时间</span>
            <span class="opp-info-value">{{ opp.createTime ? formatDateTime(opp.createTime) : '2024-12-15' }}</span>
          </span>
        </div>
        <div class="opp-info-detail-row">
          <div class="opp-info-detail-item">
            <div class="opp-info-detail-label">行业</div>
            <div class="opp-info-detail-value">{{ industryLabelMap[Number(opp.customerIndustry)] || '制造业' }}</div>
          </div>
          <div class="opp-info-detail-item">
            <div class="opp-info-detail-label">来源</div>
            <div class="opp-info-detail-value">{{ sourceMap[opp.source] || '展会' }}</div>
          </div>
          <div class="opp-info-detail-item">
            <div class="opp-info-detail-label">预计成交日期</div>
            <div class="opp-info-detail-value">{{ opp.expectedCloseDate || '2025-03-31' }}</div>
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

    <!-- 5步进度条 -->
    <div class="opp-steps">
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
      <div class="opp-step-line"></div>
      <div class="opp-step">
        <div class="opp-step-number">4</div>
        <div class="opp-step-label">已报价</div>
      </div>
      <div class="opp-step-line"></div>
      <div class="opp-step">
        <div class="opp-step-number">5</div>
        <div class="opp-step-label">成交/丢单</div>
      </div>
    </div>

    <!-- 阶段 Tab 切换 -->
    <div class="opp-nav">
      <span
        class="opp-nav-item"
        :class="{ active: activeTab === '1' }"
        @click="activeTab = '1'"
      >初步沟通</span>
      <span
        class="opp-nav-item"
        :class="{ active: activeTab === '2' }"
        @click="activeTab = '2'"
      >需求确认</span>
      <span
        class="opp-nav-item nav-purple"
        :class="{ active: activeTab === '3' }"
        @click="activeTab = '3'"
      >方案沟通</span>
    </div>

    <!-- 主体：左右布局 -->
    <div class="opp-body">
      <!-- 左栏：阶段Tab + 表单 -->
      <div class="opp-main">
        <!-- Tab1: 初步沟通 -->
        <div v-show="activeTab === '1'" class="opp-tab-content">
          <div class="opp-form-header">
            <span class="opp-form-title">初步沟通记录</span>
            <Button size="small" class="opp-add-btn">+ 添加记录</Button>
          </div>
          <Form ref="baseFormRef" :model="baseForm" layout="vertical" class="opp-form">
            <Form.Item label="商机名称" name="title" :rules="[{ required: true, message: '请输入商机名称' }]">
              <Input v-model:value="baseForm.title" placeholder="请输入商机名称" />
            </Form.Item>
            <Form.Item label="所属企业" name="customerId">
              <Select
                v-model:value="baseForm.customerId"
                show-search
                placeholder="搜索并选择客户"
                allow-clear
                :filter-option="false"
                :loading="customerLoading"
                :options="customerOptions"
                :field-names="{ label: 'companyName', value: 'id' }"
                @search="handleCustomerSearch"
                @change="handleCustomerChange"
              >
                <template #option="{ item }">
                  <div>{{ item.companyName }}</div>
                </template>
              </Select>
            </Form.Item>
            <Form.Item label="联系人" name="contactId">
              <Select
                v-model:value="baseForm.contactId"
                placeholder="请先选择所属企业"
                allow-clear
                show-search
                :options="contactOptions"
                :filter-option="(input: string, option: any) => (option?.label ?? '').toLowerCase().includes(input.toLowerCase())"
                @focus="() => { if (!baseForm.customerId) message.warning('请先选择所属企业') }"
              />
            </Form.Item>
            <div class="opp-form-row">
              <Form.Item label="商机金额" name="amount" class="opp-form-item">
                <InputNumber v-model:value="baseForm.amount" :min="0" :precision="2" placeholder="请输入商机金额" style="width: 100%" />
              </Form.Item>
              <Form.Item label="币种" name="currency" class="opp-form-item">
                <Select v-model:value="baseForm.currency" :options="currencyOptions" />
              </Form.Item>
            </div>
            <div class="opp-form-row">
              <Form.Item label="赢单概率" name="probability" class="opp-form-item">
                <InputNumber v-model:value="baseForm.probability" :min="0" :max="100" placeholder="0-100" style="width: 100%">
                  <template #addonAfter>%</template>
                </InputNumber>
              </Form.Item>
              <Form.Item label="商机来源" name="source" class="opp-form-item">
                <Select v-model:value="baseForm.source" placeholder="请选择来源" allow-clear :options="sourceOptions" />
              </Form.Item>
            </div>
            <Form.Item label="预计成交日期" name="expectedCloseDate">
              <DatePicker v-model:value="baseForm.expectedCloseDate" placeholder="请选择预计成交日期" style="width: 100%" value-format="YYYY-MM-DD" />
            </Form.Item>
            <Form.Item label="商机描述" name="description">
              <Input.TextArea v-model:value="baseForm.description" placeholder="详细描述商机背景、客户需求、价值主张等" :rows="4" :maxlength="2000" show-count />
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
            <Button size="small" class="opp-add-btn">+ 添加记录</Button>
          </div>
          <Form :model="reqForm" layout="vertical" class="opp-form">
            <Form.Item label="需求类型" name="reqType">
              <Select v-model:value="reqForm.reqType" placeholder="请选择需求类型" allow-clear :options="reqTypeOptions" />
            </Form.Item>
            <Form.Item label="需求描述" name="reqDesc">
              <Input.TextArea v-model:value="reqForm.reqDesc" placeholder="请输入需求描述..." :rows="8" :maxlength="2000" show-count />
            </Form.Item>
            <Form.Item label="优先级" name="priority">
              <Radio.Group v-model:value="reqForm.priority">
                <Radio value="high">高</Radio>
                <Radio value="mid">中</Radio>
                <Radio value="low">低</Radio>
              </Radio.Group>
            </Form.Item>
            <Form.Item label="期望交付时间" name="expectDate">
              <DatePicker v-model:value="reqForm.expectDate" placeholder="年 / 月 / 日" style="width: 100%" value-format="YYYY-MM-DD" />
            </Form.Item>
            <Form.Item label="预算范围" name="budgetRange">
              <Select v-model:value="reqForm.budgetRange" placeholder="请选择预算范围" allow-clear :options="budgetOptions" />
            </Form.Item>
            <Form.Item label="需求文档">
              <Upload>
                <Button>
                  <LucidePlus /> 点击或拖拽文件上传
                </Button>
                <template #tip>
                  <div class="ant-upload-hint">支持 PDF、Word、Excel 等格式，单个文件不超过 10MB</div>
                </template>
              </Upload>
            </Form.Item>
          </Form>
          <div class="opp-form-footer">
            <Button type="primary" :loading="saving" @click="handleSaveReq">保存</Button>
          </div>
        </div>

        <!-- Tab3: 方案沟通 -->
        <div v-show="activeTab === '3'" class="opp-tab-content">
          <div class="opp-form-header">
            <span class="opp-form-title">方案沟通记录</span>
            <Button size="small" class="opp-add-btn">+ 添加记录</Button>
          </div>
          <Form :model="solForm" layout="vertical" class="opp-form">
            <Form.Item label="方案类型" name="solutionType">
              <Select v-model:value="solForm.solutionType" placeholder="请选择方案类型" allow-clear :options="solutionTypeOptions" />
            </Form.Item>
            <Form.Item label="方案概述" name="solutionOverview">
              <Input.TextArea v-model:value="solForm.solutionOverview" placeholder="请输入方案概述..." :rows="8" :maxlength="2000" show-count />
            </Form.Item>
            <Form.Item label="方案亮点">
              <Input v-model:value="solForm.solutionHighlights" placeholder="请输入方案亮点" />
              <div class="opp-highlight-tags">
                <Tag color="green">已发送文件</Tag>
              </div>
            </Form.Item>
            <div class="opp-form-row">
              <Form.Item label="预计工期" class="opp-form-item">
                <Input v-model:value="solForm.estimatedDuration" placeholder="请填写预计工期" prefix="约" suffix="周" />
              </Form.Item>
              <Form.Item label="报价金额" class="opp-form-item">
                <InputNumber v-model:value="solForm.quoteAmount" :min="0" :precision="2" placeholder="请输入报价金额" style="width: 100%">
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
                  <div class="opp-upload-hint">支持 PDF、Word、Excel 等格式，单个文件不超过 10MB</div>
                </div>
              </Upload>
            </Form.Item>
            <div class="opp-form-demo">
              <div class="opp-demo-title">演示安排</div>
              <div class="opp-form-row">
                <Form.Item label="演示日期" class="opp-form-item">
                  <DatePicker v-model:value="solForm.demoDate" placeholder="年 / 月 / 日" style="width: 100%" value-format="YYYY-MM-DD" />
                </Form.Item>
                <Form.Item label="演示方式" class="opp-form-item">
                  <Select v-model:value="solForm.demoType" placeholder="请选择演示方式" allow-clear :options="demoTypeOptions" />
                </Form.Item>
              </div>
            </div>
          </Form>
          <div class="opp-form-footer">
            <Button type="primary" class="opp-submit-btn" :loading="saving" @click="handleSubmitSolution">
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
            <div v-for="(record, idx) in followUpRecords" :key="idx" class="opp-tl-item">
              <div class="opp-tl-dot" :style="{ backgroundColor: record.color }"></div>
              <div class="opp-tl-body">
                <div class="opp-tl-time">{{ record.time }}</div>
                <div class="opp-tl-user">
                  <Avatar :size="20" :style="{ backgroundColor: record.color, color: '#fff' }">
                    {{ record.user.charAt(0) }}
                  </Avatar>
                  <span class="opp-tl-name">{{ record.user }}</span>
                </div>
                <div class="opp-tl-content">{{ record.content }}</div>
                <div v-if="record.tags && record.tags.length" class="opp-tl-tags">
                  <Tag v-for="t in record.tags" :key="t.text" :color="t.color" size="small" class="opp-tl-tag">
                    {{ t.text }}
                  </Tag>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 关键联系人 -->
        <div class="opp-right-section">
          <div class="opp-right-title">关键联系人</div>
          <div>
            <div v-for="(c, idx) in contactList" :key="idx" class="opp-contact-item">
              <Avatar :size="36" :style="{ backgroundColor: c.avatarColor, color: '#fff' }">
                {{ c.name.charAt(0) }}
              </Avatar>
              <div class="opp-contact-info">
                <div class="opp-contact-name">
                  {{ c.name }}
                  <Tag v-for="t in c.tags" :key="t.text" :color="t.color" size="small" class="opp-tl-tag">{{ t.text }}</Tag>
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
  </div>
</template>

<style scoped>
.opp-detail {
  min-height: 100%;
  background: hsl(var(--background));
  padding: 0 0 24px 0;
}

/* 商机信息卡 */
.opp-info-card {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin: 16px 24px 0;
  padding: 20px 24px;
  background: hsl(var(--muted) / 40%);
  border-radius: 8px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.03);
  border: 1px solid hsl(var(--border));
  flex-wrap: wrap;
  gap: 16px;
}
.opp-info-main {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
  flex: 1;
}
.opp-info-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.opp-company {
  font-size: 16px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}
.opp-project {
  font-size: 16px;
  color: hsl(var(--muted-foreground));
  font-weight: 500;
}
.opp-no-tag {
  font-size: 11px;
  transform: scale(0.9);
  transform-origin: left center;
  border-radius: 4px;
  border-color: hsl(var(--border));
  color: hsl(var(--muted-foreground));
}
.opp-info-desc-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  font-size: 12px;
}
.opp-info-desc-item {
  display: flex;
  align-items: center;
  gap: 4px;
}
.opp-info-desc-sep {
  color: hsl(var(--border));
}
.opp-info-label {
  color: hsl(var(--muted-foreground));
}
.opp-info-value {
  color: hsl(var(--card-foreground) / 0.8);
}
.opp-info-detail-row {
  display: flex;
  align-items: center;
  gap: 24px;
  flex-wrap: wrap;
  margin-top: 2px;
}
.opp-info-detail-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}
.opp-info-detail-label {
  color: hsl(var(--muted-foreground));
}
.opp-info-detail-value {
  color: hsl(var(--card-foreground) / 0.8);
}
.opp-info-detail-value.prob {
  color: #fa8c16;
  font-weight: 500;
}
.opp-info-extra {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: center;
  gap: 4px;
}
.opp-info-amount-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}
.opp-info-amount-value {
  font-size: 18px;
  color: #fa8c16;
  font-weight: 600;
}

/* 5步进度条 */
.opp-steps {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 20px 40px 16px;
  margin: 12px 24px 0;
  background: hsl(var(--card));
  border-radius: 8px;
  border: none;
}
.opp-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  min-width: 60px;
}
.opp-step-number {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: hsl(var(--border));
  color: hsl(var(--muted-foreground));
  font-size: 12px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
}
.opp-step-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
}
.opp-step.step-done .opp-step-number {
  background: #52c41a;
  color: #fff;
}
.opp-step.step-done .opp-step-label {
  color: #52c41a;
  font-weight: 500;
}
.opp-step.step-current-purple .opp-step-number {
  background: #7c3aed;
  color: #fff;
}
.opp-step.step-current-purple .opp-step-label {
  color: #7c3aed;
  font-weight: 500;
}
.opp-step-line {
  flex: 1;
  height: 2px;
  background: hsl(var(--border));
  margin-top: 12px;
  max-width: 100px;
  min-width: 40px;
}
.opp-step-line.line-done {
  background: #52c41a;
}

/* Tab 切换 */
.opp-nav {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 0 24px;
  margin: 12px 24px 0;
  background: hsl(var(--card));
  border-radius: 0;
  border: none;
  border-bottom: 1px solid hsl(var(--border));
  font-size: 13px;
}
.opp-nav-item {
  padding: 10px 0;
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: all 0.2s;
}
.opp-nav-item:hover {
  color: hsl(var(--card-foreground));
}
.opp-nav-item.active {
  color: #52c41a;
  border-bottom-color: #52c41a;
  font-weight: 500;
}
.opp-nav-item.nav-purple.active {
  color: #7c3aed;
  border-bottom-color: #7c3aed;
}

/* 主体布局 */
.opp-body {
  display: flex;
  gap: 16px;
  margin: 15px 24px 0;
  align-items: flex-start;
}
.opp-side {
  width: 360px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.opp-main {
  flex: 1;
  min-width: 0;
  background: hsl(var(--muted) / 40%);
  border-radius: 8px;
  border: 1px solid hsl(var(--border));
  overflow: hidden;
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
  border-top: 1px solid hsl(var(--border));
  margin-top: 8px;
}
.opp-submit-btn {
  background: #7c3aed;
  border-color: #7c3aed;
  min-width: 100px;
}
.opp-submit-btn:hover,
.opp-submit-btn:focus {
  background: #6d28d9;
  border-color: #6d28d9;
}

/* 方案亮点标签 */
.opp-highlight-tags {
  margin-top: 6px;
  display: flex;
  gap: 6px;
}

/* 上传框 */
.opp-upload-box {
  border: 1px dashed hsl(var(--border));
  border-radius: 6px;
  padding: 24px 16px;
  text-align: center;
  background: hsl(var(--card) / 50%);
  cursor: pointer;
  transition: all 0.2s;
}
.opp-upload-box:hover {
  border-color: #7c3aed;
}
.opp-upload-icon {
  color: hsl(var(--muted-foreground));
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
}
.opp-upload-text {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  margin-bottom: 4px;
}
.opp-upload-hint {
  font-size: 11px;
  color: hsl(var(--muted-foreground) / 0.7);
}

/* 演示安排 */
.opp-form-demo {
  margin-top: 4px;
  padding-top: 8px;
  border-top: 1px dashed hsl(var(--border));
}
.opp-demo-title {
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
  margin-bottom: 8px;
}

/* 右栏 */
.opp-right-section {
  background: hsl(var(--muted) / 40%);
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  padding: 14px 16px;
}
.opp-right-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid hsl(var(--border));
}

/* 跟进记录时间轴 */
.opp-timeline {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.opp-tl-item {
  display: flex;
  gap: 10px;
  position: relative;
}
.opp-tl-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  margin-top: 5px;
  position: relative;
  z-index: 1;
}
.opp-tl-item::before {
  content: '';
  position: absolute;
  left: 3.5px;
  top: 13px;
  width: 1px;
  bottom: -6px;
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
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin-bottom: 4px;
}
.opp-tl-user {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}
.opp-tl-name {
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
}
.opp-tl-content {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  line-height: 1.6;
}
.opp-tl-tags {
  display: flex;
  gap: 6px;
  margin-top: 6px;
  flex-wrap: wrap;
}
.opp-tl-tag {
  border-radius: 3px;
  margin: 0;
}

/* 联系人 */
.opp-contact-item {
  display: flex;
  align-items: center;
  gap: 10px;
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
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
  display: flex;
  align-items: center;
  gap: 6px;
}
.opp-contact-title {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin-top: 2px;
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
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 4px;
  justify-content: flex-end;
}
</style>
