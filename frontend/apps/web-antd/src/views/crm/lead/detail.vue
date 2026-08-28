<script lang="ts" setup>
import { computed, reactive, ref, watch } from 'vue';

import { LucidePhone, LucidePlus } from '@vben/icons';
import { useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  DatePicker,
  Empty,
  Form,
  Input,
  InputNumber,
  message,
  Select,
  Spin,
  Tag,
} from 'ant-design-vue';

import {
  createLeadApi,
  getCountriesApi,
  getLatestBackgroundCheckApi,
  getLeadInfoApi,
  performBackgroundCheckApi,
  saveFollowupApi,
  updateLeadApi,
} from '#/api';

import TagSelector from '../components/TagSelector.vue';

const props = defineProps<{
  create?: boolean;
  id: null | number;
}>();

const emit = defineEmits<{
  (e: 'saved', id?: number): void;
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

const isCreate = computed(() => props.create || !props.id);

// ============ 数据 ============
const loading = ref(false);
const saving = ref(false);
const followupSaving = ref(false);
const correctingName = ref(false);
const lead = ref<any>({});
const followupRecords = ref<any[]>([]);
const tagSelectorRef = ref<InstanceType<typeof TagSelector>>();
const activeSection = ref<'background' | 'followup' | 'info'>('info');

// 企业背调数据
const bgLoading = ref(false);
const bgReport = ref<any>(null);
const bgActiveTab = ref<'basic' | 'business' | 'risk' | 'suggestion'>('basic');

// ============ 枚举映射 ============
const statusLabelMap: Record<string, string> = {
  6: '未核查',
  7: '核查中',
  4: '无效线索',
  8: '线索池',
  1: '新线索',
  2: '跟进中',
  3: '已转化',
  5: '已回收',
};
const statusColorMap: Record<string, string> = {
  6: 'default',
  7: 'processing',
  4: 'default',
  8: 'success',
  1: 'blue',
  2: 'cyan',
  3: 'green',
  5: 'warning',
};
const sourceLabelMap: Record<string, string> = {
  website: '官网',
  exhibition: '展会',
  social: '社交媒体',
  referral: '客户转介',
  cold_call: '陌生拜访',
  customs: '海关数据',
  email: '邮件营销',
  alibaba: '阿里国际站',
  amazon: 'Amazon',
  tiktok: 'TikTok',
  wechat: '微信',
  other: '其他',
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
const levelLabelMap: Record<string, string> = {
  1: '无级别',
  2: '重点客户',
  3: '优质客户',
  4: '普通客户',
  5: '其他',
};
const levelColorMap: Record<string, string> = {
  1: 'default',
  2: 'red',
  3: 'orange',
  4: 'blue',
  5: 'green',
};

const followMethodOptions = [
  { label: '电话', value: 1, color: '#1890ff' },
  { label: '拜访', value: 2, color: '#13c2c2' },
  { label: '邮件', value: 3, color: '#722ed1' },
  { label: '会议', value: 4, color: '#fa8c16' },
  { label: 'WhatsApp', value: 5, color: '#25b864' },
  { label: '微信', value: 6, color: '#52c41a' },
  { label: '其他', value: 7, color: '#8c8c8c' },
];

const followStatusOptions = [
  { label: '新客', value: 1 },
  { label: '跟进中', value: 2 },
  { label: '已成交', value: 3 },
  { label: '无效线索', value: 4 },
  { label: '已回收', value: 5 },
  { label: '未核查', value: 6 },
  { label: '核查中', value: 7 },
  { label: '线索池', value: 8 },
];

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
const levelOptions = [
  { label: '无级别', value: 1 },
  { label: '重点客户', value: 2 },
  { label: '优质客户', value: 3 },
  { label: '普通客户', value: 4 },
  { label: '其他', value: 5 },
];
const sourceOptions = [
  { label: '官网', value: 'website' },
  { label: '展会', value: 'exhibition' },
  { label: '社交媒体', value: 'social' },
  { label: '客户转介', value: 'referral' },
  { label: '陌生拜访', value: 'cold_call' },
  { label: '海关数据', value: 'customs' },
  { label: '邮件营销', value: 'email' },
  { label: '阿里国际站', value: 'alibaba' },
  { label: 'Amazon', value: 'amazon' },
  { label: 'TikTok', value: 'tiktok' },
  { label: '微信', value: 'wechat' },
  { label: '其他', value: 'other' },
];
const currencyOptions = [
  { label: 'CNY', value: 'CNY' },
  { label: 'USD', value: 'USD' },
  { label: 'EUR', value: 'EUR' },
  { label: 'GBP', value: 'GBP' },
  { label: 'JPY', value: 'JPY' },
  { label: 'KRW', value: 'KRW' },
  { label: 'HKD', value: 'HKD' },
  { label: 'AUD', value: 'AUD' },
];

// ============ 表单 ============
const formRef = ref();
const form = reactive({
  companyName: '',
  contactName: '',
  title: '',
  email: '',
  phone: '',
  mobile: '',
  website: '',
  industry: undefined as number | undefined,
  level: undefined as number | undefined,
  source: undefined as string | undefined,
  sourceDetail: '',
  country: undefined as string | undefined,
  region: '',
  address: '',
  budget: undefined as number | undefined,
  currency: 'CNY',
  nextFollowAt: undefined as any,
  description: '',
});

const followupForm = reactive({
  content: '',
  nextFollowAt: undefined as any,
  status: 2,
  method: 1,
});

const countryOptions = ref<{ label: string; value: string }[]>([]);
const countryLoading = ref(false);

async function loadCountries() {
  if (countryOptions.value.length > 0) return;
  countryLoading.value = true;
  try {
    const res = await getCountriesApi();
    const items = Array.isArray(res) ? res : [];
    countryOptions.value = items.map((item: any) => ({
      label: item.name,
      value: item.name,
    }));
  } catch {
    /* ignore */
  } finally {
    countryLoading.value = false;
  }
}

// ============ 加载数据 ============
const sortedFollowupRecords = computed(() =>
  followupRecords.value.toSorted(
    (a, b) =>
      new Date(b.createTime).getTime() - new Date(a.createTime).getTime(),
  ),
);

const followupCount = computed(() => followupRecords.value.length);

const budgetText = computed(() => {
  const val = form.budget;
  if (val === null || val === undefined) return '-';
  return `${form.currency || 'CNY'} ${Number(val).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 2 })}`;
});

const editHistory = ref<any[]>([
  {
    time: '2024-12-15 14:30',
    user: '张三',
    action: '创建线索',
    changes: [
      { field: '公司名称', newValue: '深圳市科技有限公司' },
      { field: '联系人', newValue: '李四' },
      { field: '手机', newValue: '13800138000' },
      { field: '来源', newValue: '展会' },
      { field: '行业', newValue: '电商' },
    ],
  },
  {
    time: '2024-12-16 10:15',
    user: '王五',
    action: '更新信息',
    changes: [
      { field: '职位', newValue: '销售经理' },
      { field: '邮箱', newValue: 'lisi@example.com' },
      { field: '客户级别', newValue: '重点客户' },
    ],
  },
  {
    time: '2024-12-18 16:45',
    user: '张三',
    action: '更新信息',
    changes: [
      { field: '预算金额', newValue: 'CNY 500,000' },
      { field: '下次跟进时间', newValue: '2024-12-25' },
    ],
  },
  {
    time: '2024-12-20 09:20',
    user: '赵六',
    action: '更新信息',
    changes: [
      { field: '详细地址', newValue: '广东省深圳市南山区科技园' },
      { field: '网站', newValue: 'https://example.com' },
    ],
  },
]);

async function fetchDetail() {
  if (!props.id) return;
  loading.value = true;
  try {
    const res = await getLeadInfoApi(props.id);
    lead.value = res;
    followupRecords.value = res?.followups || [];

    // 填充表单
    Object.keys(form).forEach((key) => {
      if (res[key] !== undefined && res[key] !== null) {
        (form as any)[key] = res[key];
      }
    });

    // 加载背调数据
    await fetchBackgroundCheck();
  } catch {
    message.error('获取线索详情失败');
  } finally {
    loading.value = false;
  }
}

async function fetchBackgroundCheck() {
  if (!props.id) return;
  bgLoading.value = true;
  try {
    const res = await getLatestBackgroundCheckApi(props.id);
    if (res) {
      const rawReportData = res.reportData || res.report_data;
      bgReport.value = toCamelCase(res);
      if (bgReport.value) {
        bgReport.value.reportData = normalizeBgReport(rawReportData);
        if (!bgReport.value.riskScore && rawReportData) {
          bgReport.value.riskScore =
            rawReportData.risk_score ||
            rawReportData.riskScore ||
            rawReportData.risk_assessment?.risk_score ||
            rawReportData.riskAssessment?.riskScore ||
            50;
        }
        if (!bgReport.value.riskLevel && rawReportData) {
          const score = bgReport.value.riskScore;
          bgReport.value.riskLevel =
            rawReportData.risk_level ||
            rawReportData.riskLevel ||
            rawReportData.risk_assessment?.risk_level ||
            rawReportData.riskAssessment?.riskLevel ||
            getRiskLevelByScore(score);
        }
      }
    } else {
      bgReport.value = null;
    }
  } catch (error) {
    console.error('[背调] 加载失败:', error);
  } finally {
    bgLoading.value = false;
  }
}

async function handleRunBackgroundCheck() {
  if (!form.companyName || !form.companyName.trim()) {
    message.error('请先填写公司名称');
    return;
  }
  bgLoading.value = true;
  try {
    const res = await performBackgroundCheckApi({
      company_name: form.companyName,
      lead_id: props.id ?? undefined,
    });
    if (res) {
      const rawReportData = res.reportData || res.report_data;
      bgReport.value = toCamelCase(res);
      if (bgReport.value) {
        bgReport.value.reportData = normalizeBgReport(rawReportData);
        if (!bgReport.value.riskScore && rawReportData) {
          bgReport.value.riskScore =
            rawReportData.risk_score ||
            rawReportData.riskScore ||
            rawReportData.risk_assessment?.risk_score ||
            rawReportData.riskAssessment?.riskScore ||
            50;
        }
        if (!bgReport.value.riskLevel && rawReportData) {
          const score = bgReport.value.riskScore;
          bgReport.value.riskLevel =
            rawReportData.risk_level ||
            rawReportData.riskLevel ||
            rawReportData.risk_assessment?.risk_level ||
            rawReportData.riskAssessment?.riskLevel ||
            getRiskLevelByScore(score);
        }
      }
    } else {
      bgReport.value = null;
    }
    message.success('企业背调完成，评估结果已保存');
    activeSection.value = 'background';
  } catch (error: any) {
    const msg =
      error?.message || error?.msg || '评估失败，请检查API配置是否正常';
    console.error('[背调] 评估失败:', error);
    message.error(msg);
  } finally {
    bgLoading.value = false;
  }
}

// ============ 一键更正公司名称 ============
const correctCompanyName = computed(() => {
  return (
    bgReport.value?.reportData?.basic_info?.company_name ||
    bgReport.value?.reportData?.company_info?.company_name ||
    bgReport.value?.companyName ||
    bgReport.value?.company_name ||
    bgReport.value?.reportData?.companyName ||
    ''
  );
});

async function handleCorrectCompanyName() {
  const correctName = correctCompanyName.value;

  if (!correctName || !correctName.trim()) {
    message.warning('背调报告中未找到工商注册的公司名称');
    return;
  }

  if (correctName.trim() === form.companyName?.trim()) {
    message.info('公司名称已是最新，无需更正');
    return;
  }

  if (isCreate.value) {
    form.companyName = correctName.trim();
    message.success('公司名称已更新为工商注册名称');
    return;
  }

  correctingName.value = true;
  try {
    const payload: any = {
      ...form,
      companyName: correctName.trim(),
      assignedTo: lead.value?.assignedTo || undefined,
      id: props.id,
    };
    await updateLeadApi(payload);
    form.companyName = correctName.trim();
    lead.value.companyName = correctName.trim();
    message.success('公司名称已更正为工商注册名称');
  } catch {
    // 错误由全局拦截器处理
  } finally {
    correctingName.value = false;
  }
}

// ============ 保存线索 ============
async function handleSave() {
  try {
    await formRef.value?.validate();
  } catch {
    return;
  }

  saving.value = true;
  try {
    const payload: any = { ...form };
    if (isCreate.value) {
      payload.status = 6;
      payload.assignedTo = Number(useUserStore().userInfo?.userId) || undefined;
      const result: any = await createLeadApi(payload);
      const newId = result?.id || result?.data?.id;
      message.success('线索创建成功');
      if (newId && tagSelectorRef.value) {
        await tagSelectorRef.value.saveToEntity(newId);
      }
      emit('saved', newId);
      if (newId) {
        // 切换到编辑模式，刷新数据
        emit('saved', newId);
      }
    } else {
      payload.id = props.id;
      payload.assignedTo = lead.value?.assignedTo || undefined;
      await updateLeadApi(payload);
      message.success('保存成功');
      await fetchDetail();
      emit('saved');
    }
  } catch {
    // 错误由全局拦截器处理
  } finally {
    saving.value = false;
  }
}

// ============ 保存跟进记录 ============
async function handleSaveFollowup() {
  if (!followupForm.content.trim()) {
    message.warning('请填写跟进内容');
    return;
  }
  if (isCreate.value) {
    message.warning('请先保存线索基本信息');
    return;
  }

  followupSaving.value = true;
  try {
    await saveFollowupApi({
      leadId: Number(props.id),
      content: followupForm.content,
      nextFollowDate: followupForm.nextFollowAt,
      activityType: Number(followupForm.method),
      leadStatus: Number(followupForm.status),
    });
    message.success('跟进记录已保存');
    followupForm.content = '';
    followupForm.nextFollowAt = undefined;
    followupForm.status = 2;
    followupForm.method = 1;
    await fetchDetail();
  } catch {
    // 错误由全局拦截器处理
  } finally {
    followupSaving.value = false;
  }
}

function getMethodOption(value: any) {
  return followMethodOptions.find((o) => o.value === value);
}

function getRiskClass(score: number | undefined): string {
  if (!score) return 'risk-default';
  if (score <= 30) return 'risk-high';
  if (score <= 50) return 'risk-medium';
  if (score <= 70) return 'risk-low';
  return 'risk-safe';
}

// 根据风险评分推导风险等级文案
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

// ============ 生命周期 ============
watch(
  () => props.id,
  (val) => {
    if (val) {
      fetchDetail();
      activeSection.value = 'info';
    } else {
      resetForm();
    }
  },
  { immediate: true },
);

watch(
  () => props.create,
  (val) => {
    if (val) {
      resetForm();
      activeSection.value = 'info';
      loadCountries();
    }
  },
  { immediate: true },
);

function resetForm() {
  lead.value = {};
  followupRecords.value = [];
  Object.keys(form).forEach((key) => {
    if (key === 'currency') (form as any)[key] = 'CNY';
    else if (key === 'status') (form as any)[key] = undefined;
    else
      (form as any)[key] =
        typeof (form as any)[key] === 'number' ? undefined : '';
  });
  form.budget = undefined;
  form.industry = undefined;
  form.level = undefined;
  form.source = undefined;
  form.country = undefined;
  form.nextFollowAt = undefined;
  followupForm.content = '';
  followupForm.nextFollowAt = undefined;
  followupForm.status = 2;
  followupForm.method = 1;
  loadCountries();
}
</script>

<template>
  <div class="lead-detail">
    <!-- 线索信息卡（整合标题、联系信息与业务详情） -->
    <div v-if="!isCreate" class="lead-info-card">
      <div class="lead-info-main">
        <div class="lead-info-title-row">
          <span class="lead-company">{{
            isCreate ? '新建线索' : form.companyName || '未命名公司'
          }}</span>
          <Tag
            v-if="!isCreate"
            :color="statusColorMap[lead.status] || 'default'"
            class="lead-no-tag"
          >
            {{ statusLabelMap[lead.status] || lead.status }}
          </Tag>
          <Tag
            v-if="!isCreate && lead.level"
            :color="levelColorMap[lead.level] || 'default'"
            class="lead-no-tag"
          >
            {{ levelLabelMap[lead.level] || lead.level }}
          </Tag>
        </div>
        <div class="lead-info-desc-row">
          <span class="lead-info-desc-item">
            <span class="lead-info-label">联系人</span>
            <span class="lead-info-value">{{ lead.contactName || '-' }}</span>
            <span v-if="lead.title" class="lead-info-sub"
              >({{ lead.title }})</span
            >
          </span>
          <span class="lead-info-desc-sep">|</span>
          <span class="lead-info-desc-item">
            <span class="lead-info-label">邮箱</span>
            <span class="lead-info-value">{{ lead.email || '-' }}</span>
          </span>
          <span class="lead-info-desc-sep">|</span>
          <span class="lead-info-desc-item">
            <span class="lead-info-label">手机</span>
            <span class="lead-info-value">{{ lead.mobile || '-' }}</span>
          </span>
        </div>
        <div class="lead-info-detail-grid">
          <div class="lead-info-detail-item">
            <div class="lead-info-detail-label">行业</div>
            <div class="lead-info-detail-value">
              {{ industryLabelMap[form.industry!] || form.industry || '-' }}
            </div>
          </div>
          <div class="lead-info-detail-item">
            <div class="lead-info-detail-label">来源</div>
            <div class="lead-info-detail-value">
              {{ sourceLabelMap[form.source!] || form.source || '-' }}
            </div>
          </div>
          <div class="lead-info-detail-item">
            <div class="lead-info-detail-label">国家/地区</div>
            <div class="lead-info-detail-value">{{ form.country || '-' }}</div>
          </div>
          <div class="lead-info-detail-item">
            <div class="lead-info-detail-label">区域</div>
            <div class="lead-info-detail-value">
              {{ lead.region || form.region || '-' }}
            </div>
          </div>
          <div class="lead-info-detail-item">
            <div class="lead-info-detail-label">下次跟进</div>
            <div class="lead-info-detail-value">
              {{ lead.nextFollowAt || '-' }}
            </div>
          </div>
          <div class="lead-info-detail-item">
            <div class="lead-info-detail-label">跟进次数</div>
            <div class="lead-info-detail-value">
              {{ isCreate ? '0次' : `${followupCount}次` }}
            </div>
          </div>
        </div>
      </div>
      <div class="lead-info-extra">
        <div class="lead-info-amount-label">预算</div>
        <div class="lead-info-amount-value">
          {{ isCreate ? '-' : budgetText }}
        </div>
      </div>
    </div>

    <!-- 区域切换 Tab -->
    <div class="lead-nav" :class="{ 'lead-nav-create': isCreate }">
      <span
        class="lead-nav-item"
        :class="{ active: activeSection === 'info' }"
        @click="activeSection = 'info'"
      >
        {{ isCreate ? '填写线索信息' : '编辑信息' }}
      </span>
      <span
        v-if="!isCreate"
        class="lead-nav-item"
        :class="{ active: activeSection === 'followup' }"
        @click="activeSection = 'followup'"
      >
        跟进记录
        <span v-if="followupCount > 0" class="lead-nav-badge">{{
          followupCount
        }}</span>
      </span>
      <span
        v-if="!isCreate"
        class="lead-nav-item"
        :class="{ active: activeSection === 'background' }"
        @click="activeSection = 'background'"
      >
        企业背调
      </span>
    </div>

    <!-- 主体：左右布局 -->
    <div class="lead-body">
      <!-- 左栏：表单 / 跟进 -->
      <div class="lead-main">
        <Spin :spinning="loading" style="width: 100%">
          <!-- 线索信息表单 -->
          <div v-show="activeSection === 'info'" class="lead-tab-content">
            <div
              class="lead-edit-layout"
              :class="{ 'lead-edit-create': isCreate }"
            >
              <!-- 左侧：编辑历史时间轴 -->
              <div v-if="!isCreate" class="lead-edit-history">
                <div class="lead-edit-history-title">编辑历史</div>
                <div class="lead-edit-timeline">
                  <div
                    v-for="(item, idx) in editHistory"
                    :key="idx"
                    class="lead-edit-tl-item"
                  >
                    <div class="lead-edit-tl-dot"></div>
                    <div class="lead-edit-tl-body">
                      <div class="lead-edit-tl-time">
                        <span>{{ item.time }}</span>
                        <span class="lead-edit-tl-user">· {{ item.user }}</span>
                      </div>
                      <div class="lead-edit-tl-action">{{ item.action }}</div>
                      <div class="lead-edit-tl-changes">
                        <div
                          v-for="(change, cidx) in item.changes"
                          :key="cidx"
                          class="lead-edit-tl-change"
                        >
                          <span class="lead-edit-tl-field">{{
                            change.field
                          }}</span>
                          <span class="lead-edit-tl-arrow">→</span>
                          <span class="lead-edit-tl-new">{{
                            change.newValue
                          }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 右侧：编辑表单 -->
              <div class="lead-edit-form">
                <div class="lead-form-header">
                  <span class="lead-form-title">{{
                    isCreate ? '填写线索信息' : '编辑线索信息'
                  }}</span>
                </div>
                <Form
                  ref="formRef"
                  :model="form"
                  layout="vertical"
                  class="lead-form"
                >
                  <Form.Item
                    label="公司名称"
                    name="companyName"
                    :rules="[{ required: true, message: '请输入公司名称' }]"
                  >
                    <Input
                      v-model:value="form.companyName"
                      placeholder="请输入公司名称"
                      allow-clear
                    />
                  </Form.Item>
                  <div class="lead-form-row">
                    <Form.Item
                      label="联系人"
                      name="contactName"
                      :rules="[{ required: true, message: '请输入联系人' }]"
                      class="lead-form-item"
                    >
                      <Input
                        v-model:value="form.contactName"
                        placeholder="请输入联系人"
                        allow-clear
                      />
                    </Form.Item>
                    <Form.Item label="职位" name="title" class="lead-form-item">
                      <Input
                        v-model:value="form.title"
                        placeholder="请输入职位"
                        allow-clear
                      />
                    </Form.Item>
                  </div>
                  <div class="lead-form-row">
                    <Form.Item label="邮箱" name="email" class="lead-form-item">
                      <Input
                        v-model:value="form.email"
                        placeholder="请输入邮箱"
                        allow-clear
                      />
                    </Form.Item>
                    <Form.Item
                      label="手机"
                      name="mobile"
                      class="lead-form-item"
                    >
                      <Input
                        v-model:value="form.mobile"
                        placeholder="请输入手机号码"
                        allow-clear
                      />
                    </Form.Item>
                  </div>
                  <div class="lead-form-row">
                    <Form.Item label="电话" name="phone" class="lead-form-item">
                      <Input
                        v-model:value="form.phone"
                        placeholder="请输入固定电话"
                        allow-clear
                      />
                    </Form.Item>
                    <Form.Item
                      label="网站"
                      name="website"
                      class="lead-form-item"
                    >
                      <Input
                        v-model:value="form.website"
                        placeholder="https://"
                        allow-clear
                      />
                    </Form.Item>
                  </div>

                  <!-- 业务信息分隔 -->
                  <div class="lead-section-divider">
                    <span class="lead-section-divider-text">业务信息</span>
                  </div>

                  <div class="lead-form-row">
                    <Form.Item
                      label="行业"
                      name="industry"
                      class="lead-form-item"
                    >
                      <Select
                        v-model:value="form.industry"
                        placeholder="请选择行业"
                        allow-clear
                        :options="industryOptions"
                      />
                    </Form.Item>
                    <Form.Item
                      label="客户级别"
                      name="level"
                      class="lead-form-item"
                    >
                      <Select
                        v-model:value="form.level"
                        placeholder="请选择级别"
                        allow-clear
                        :options="levelOptions"
                      />
                    </Form.Item>
                  </div>
                  <div class="lead-form-row">
                    <Form.Item
                      label="来源"
                      name="source"
                      class="lead-form-item"
                    >
                      <Select
                        v-model:value="form.source"
                        placeholder="请选择来源"
                        allow-clear
                        :options="sourceOptions"
                      />
                    </Form.Item>
                    <Form.Item
                      label="来源详情"
                      name="sourceDetail"
                      class="lead-form-item"
                    >
                      <Input
                        v-model:value="form.sourceDetail"
                        placeholder="补充说明来源"
                        allow-clear
                      />
                    </Form.Item>
                  </div>
                  <div class="lead-form-row">
                    <Form.Item
                      label="国家/地区"
                      name="country"
                      class="lead-form-item"
                    >
                      <Select
                        v-model:value="form.country"
                        placeholder="请选择国家"
                        allow-clear
                        show-search
                        :filter-option="
                          (input: string, option: any) =>
                            (option?.label ?? '')
                              .toLowerCase()
                              .includes(input.toLowerCase())
                        "
                        :options="countryOptions"
                        :loading="countryLoading"
                        @focus="loadCountries"
                      />
                    </Form.Item>
                    <Form.Item
                      label="区域"
                      name="region"
                      class="lead-form-item"
                    >
                      <Input
                        v-model:value="form.region"
                        placeholder="省/州"
                        allow-clear
                      />
                    </Form.Item>
                  </div>
                  <Form.Item label="详细地址" name="address">
                    <Input
                      v-model:value="form.address"
                      placeholder="请输入详细地址"
                      allow-clear
                    />
                  </Form.Item>
                  <div class="lead-form-row">
                    <Form.Item
                      label="预算金额"
                      name="budget"
                      class="lead-form-item"
                    >
                      <InputNumber
                        v-model:value="form.budget"
                        :min="0"
                        :precision="2"
                        placeholder="请输入预算"
                        style="width: 100%"
                      />
                    </Form.Item>
                    <Form.Item
                      label="币种"
                      name="currency"
                      class="lead-form-item"
                    >
                      <Select
                        v-model:value="form.currency"
                        :options="currencyOptions"
                      />
                    </Form.Item>
                  </div>
                  <Form.Item label="下次跟进时间" name="nextFollowAt">
                    <DatePicker
                      v-model:value="form.nextFollowAt"
                      placeholder="选择日期"
                      style="width: 100%"
                      value-format="YYYY-MM-DD"
                      allow-clear
                    />
                  </Form.Item>
                  <Form.Item label="备注描述" name="description">
                    <Input.TextArea
                      v-model:value="form.description"
                      placeholder="线索备注、需求概述等"
                      :rows="3"
                      :maxlength="2000"
                      show-count
                    />
                  </Form.Item>

                  <!-- 标签（仅编辑模式） -->
                  <div v-if="!isCreate" class="lead-section-divider">
                    <span class="lead-section-divider-text">标签</span>
                  </div>
                  <div v-if="!isCreate">
                    <TagSelector
                      ref="tagSelectorRef"
                      entity-type="lead"
                      :entity-id="id"
                    />
                  </div>
                </Form>
                <div class="lead-form-footer">
                  <Button
                    type="primary"
                    class="lead-submit-btn"
                    :loading="saving"
                    @click="handleSave"
                  >
                    {{ isCreate ? '创建线索' : '保存' }}
                  </Button>
                </div>
              </div>
            </div>
          </div>

          <!-- 跟进记录 Tab -->
          <div v-show="activeSection === 'followup'" class="lead-tab-content">
            <div class="lead-followup-layout">
              <!-- 左侧：跟进记录列表 -->
              <div class="lead-followup-list">
                <div class="lead-form-header">
                  <span class="lead-form-title">跟进记录</span>
                </div>
                <div
                  v-if="sortedFollowupRecords.length === 0"
                  class="lead-empty-followup"
                >
                  <Empty description="暂无跟进记录" />
                </div>
                <div v-else class="lead-timeline">
                  <div
                    v-for="(record, idx) in sortedFollowupRecords"
                    :key="record.id || idx"
                    class="lead-tl-item"
                  >
                    <div
                      class="lead-tl-dot"
                      :style="{
                        backgroundColor:
                          getMethodOption(record.activityType)?.color ||
                          '#8c8c8c',
                      }"
                    ></div>
                    <div class="lead-tl-body">
                      <div class="lead-tl-time">
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
                          class="lead-tl-stage-tag"
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
                          class="lead-tl-user-name"
                          >· {{ record.createdByName }}</span
                        >
                      </div>
                      <div class="lead-tl-content">
                        {{ record.content || '-' }}
                      </div>
                      <div v-if="record.nextFollowDate" class="lead-tl-next">
                        <LucidePhone
                          :size="11"
                          style="margin-right: 3px"
                        />下次联系：{{ record.nextFollowDate }}
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 右侧：添加跟进表单 -->
              <div class="lead-followup-form-wrap">
                <div class="lead-right-section">
                  <div class="lead-right-title">添加跟进</div>
                  <div class="lead-followup-form">
                    <div class="lead-fu-field">
                      <label class="lead-fu-label"
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
                    <div class="lead-fu-field">
                      <label class="lead-fu-label">下次联系时间</label>
                      <DatePicker
                        v-model:value="followupForm.nextFollowAt as any"
                        placeholder="选择日期"
                        style="width: 100%"
                        value-format="YYYY-MM-DD"
                        allow-clear
                      />
                    </div>
                    <div class="lead-fu-row">
                      <div class="lead-fu-field lead-fu-half">
                        <label class="lead-fu-label">跟进状态</label>
                        <Select
                          v-model:value="followupForm.status"
                          :options="followStatusOptions"
                          placeholder="选择状态"
                          size="small"
                        />
                      </div>
                      <div class="lead-fu-field lead-fu-half">
                        <label class="lead-fu-label">跟进方式</label>
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
                      class="lead-fu-submit"
                      :loading="followupSaving"
                      :disabled="isCreate"
                      @click="handleSaveFollowup"
                    >
                      保存跟进记录
                    </Button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 企业背调 Tab -->
          <div v-show="activeSection === 'background'" class="lead-tab-content">
            <div class="lead-bg-container">
              <Spin :spinning="bgLoading" style="width: 100%">
                <div v-if="!bgReport" class="lead-bg-empty">
                  <div class="lead-bg-empty-icon">
                    <svg
                      width="64"
                      height="64"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.5"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      class="text-gray-400"
                    >
                      <path
                        d="M20.38 3.4a1 1 0 0 1 1.22.7v15.4a1 1 0 0 1-1.22.7l-4.78-1.4a1 1 0 0 1-.78-.26l-.32-.26a1 1 0 0 0-.78.26l-.32.26a1 1 0 0 1-.78.26l-4.78-1.4a1 1 0 0 1-.78-1.71L9.5 14.89l2.9-2.22a1 1 0 0 1 .78-.26h2.57a1 1 0 0 1 .78.26l2.9 2.22-.89 2.71a1 1 0 0 0 .78 1.34l3.59 1.05V6.1a1 1 0 0 1 .78-.98l3.59-1.05a1 1 0 0 1 .82.02z"
                      />
                      <circle cx="12" cy="12" r="3" />
                    </svg>
                  </div>
                  <div class="lead-bg-empty-title">暂无企业背调报告</div>
                  <div class="lead-bg-empty-desc">
                    通过AI获取企业工商信息和风险评估报告
                  </div>
                  <Button
                    type="primary"
                    :loading="bgLoading"
                    @click="handleRunBackgroundCheck"
                  >
                    <LucidePlus :size="16" /> 一键评估
                  </Button>
                </div>

                <div v-else class="lead-bg-report">
                  <div class="lead-bg-header">
                    <div class="lead-bg-header-left">
                      <h2 class="lead-bg-company-name">
                        {{ form.companyName }} 综合评估
                      </h2>
                      <span class="lead-bg-report-time"
                        >评估时间：{{
                          bgReport.createdAt
                            ? formatDateTime(bgReport.createdAt)
                            : '-'
                        }}</span
                      >
                    </div>
                    <Button
                      type="primary"
                      ghost
                      @click="handleRunBackgroundCheck"
                    >
                      <LucidePlus :size="14" /> 重新评估
                    </Button>
                  </div>

                  <div class="lead-bg-info-row">
                    <div class="lead-bg-info-item">
                      <span class="lead-bg-info-label">统一社会信用代码</span>
                      <span class="lead-bg-info-value">{{
                        bgReport.reportData?.basic_info?.credit_code || '-'
                      }}</span>
                    </div>
                    <div class="lead-bg-info-item">
                      <span class="lead-bg-info-label">法人</span>
                      <span class="lead-bg-info-value">{{
                        bgReport.reportData?.basic_info?.legal_person || '-'
                      }}</span>
                    </div>
                    <div class="lead-bg-info-item">
                      <span class="lead-bg-info-label">企业类型</span>
                      <span class="lead-bg-info-value">{{
                        bgReport.reportData?.basic_info?.company_type || '-'
                      }}</span>
                    </div>
                    <div class="lead-bg-info-item">
                      <span class="lead-bg-info-label">成立日期</span>
                      <span class="lead-bg-info-value">{{
                        bgReport.reportData?.basic_info?.establish_date || '-'
                      }}</span>
                    </div>
                  </div>
                  <div class="lead-bg-info-row">
                    <div class="lead-bg-info-item">
                      <span class="lead-bg-info-label">注册资本</span>
                      <span class="lead-bg-info-value">{{
                        bgReport.reportData?.basic_info?.registered_capital ||
                        '-'
                      }}</span>
                    </div>
                    <div class="lead-bg-info-item">
                      <span class="lead-bg-info-label">注册地址</span>
                      <span class="lead-bg-info-value">{{
                        bgReport.reportData?.basic_info?.registered_address ||
                        '-'
                      }}</span>
                    </div>
                    <div class="lead-bg-info-item">
                      <span class="lead-bg-info-label">参保人数</span>
                      <span class="lead-bg-info-value">{{
                        bgReport.reportData?.business_analysis?.insured_count ||
                        '-'
                      }}</span>
                    </div>
                    <div class="lead-bg-info-item">
                      <span class="lead-bg-info-label">经营状态</span>
                      <span class="lead-bg-info-value">{{
                        bgReport.reportData?.business_analysis
                          ?.business_status || '-'
                      }}</span>
                    </div>
                  </div>

                  <div class="lead-bg-score-row">
                    <div class="lead-bg-score-card">
                      <div class="lead-bg-score-label">综合风险评分</div>
                      <div
                        class="lead-bg-score-value"
                        :class="getRiskClass(bgReport.riskScore)"
                      >
                        {{ bgReport.riskScore || '-' }}
                      </div>
                      <div class="lead-bg-score-bar">
                        <div
                          class="lead-bg-score-bar-fill"
                          :class="getRiskClass(bgReport.riskScore)"
                          :style="{
                            width: `${(bgReport.riskScore || 50) * 1.2}%`,
                          }"
                        ></div>
                      </div>
                      <Tag
                        :color="getRiskTagColor(bgReport.riskLevel)"
                        class="lead-bg-score-tag"
                      >
                        {{ bgReport.riskLevel || '-' }}
                      </Tag>
                    </div>
                    <div class="lead-bg-suggestion-card">
                      <div class="lead-bg-suggestion-label">合作建议</div>
                      <div class="lead-bg-suggestion-content">
                        {{
                          bgReport.reportData?.cooperation_suggestion
                            ?.suggestion || '-'
                        }}
                      </div>
                    </div>
                  </div>

                  <div class="lead-bg-tabs">
                    <div class="lead-bg-tab-nav">
                      <span
                        class="lead-bg-tab-item"
                        :class="{ active: bgActiveTab === 'basic' }"
                        @click="bgActiveTab = 'basic'"
                        >工商信息</span
                      >
                      <span
                        class="lead-bg-tab-item"
                        :class="{ active: bgActiveTab === 'business' }"
                        @click="bgActiveTab = 'business'"
                        >经营分析</span
                      >
                      <span
                        class="lead-bg-tab-item"
                        :class="{ active: bgActiveTab === 'risk' }"
                        @click="bgActiveTab = 'risk'"
                        >风险评估</span
                      >
                      <span
                        class="lead-bg-tab-item"
                        :class="{ active: bgActiveTab === 'suggestion' }"
                        @click="bgActiveTab = 'suggestion'"
                        >销售建议</span
                      >
                    </div>

                    <div class="lead-bg-tab-content">
                      <!-- 工商信息 -->
                      <div
                        v-show="bgActiveTab === 'basic'"
                        class="lead-bg-tab-panel"
                      >
                        <div class="lead-bg-detail-grid">
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">公司全称</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.basic_info?.company_name ||
                              '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label"
                              >统一社会信用代码</span
                            >
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.basic_info?.credit_code ||
                              '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">法定代表人</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.basic_info?.legal_person ||
                              '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">企业类型</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.basic_info?.company_type ||
                              '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">成立日期</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.basic_info?.establish_date ||
                              '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">注册资本</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.basic_info
                                ?.registered_capital || '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">注册地址</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.basic_info
                                ?.registered_address || '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">经营范围</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.basic_info?.business_scope ||
                              '-'
                            }}</span>
                          </div>
                        </div>
                      </div>

                      <!-- 经营分析 -->
                      <div
                        v-show="bgActiveTab === 'business'"
                        class="lead-bg-tab-panel"
                      >
                        <div class="lead-bg-detail-grid">
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">参保人数</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.business_analysis
                                ?.insured_count || '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">经营状态</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.business_analysis
                                ?.business_status || '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">分支机构</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.business_analysis
                                ?.has_branches || '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label">行业地位</span>
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.business_analysis
                                ?.industry_position || '-'
                            }}</span>
                          </div>
                        </div>
                      </div>

                      <!-- 风险评估 -->
                      <div
                        v-show="bgActiveTab === 'risk'"
                        class="lead-bg-tab-panel"
                      >
                        <div class="lead-bg-risk-cards">
                          <div class="lead-bg-risk-card">
                            <div class="lead-bg-risk-icon">
                              <svg
                                width="20"
                                height="20"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                class="text-blue-500"
                              >
                                <circle cx="12" cy="12" r="10" />
                                <polyline points="12 6 12 12 16 14" />
                              </svg>
                            </div>
                            <div class="lead-bg-risk-title">资金履约风险</div>
                            <div class="lead-bg-risk-content">
                              {{
                                bgReport.reportData?.risk_assessment
                                  ?.payment_risk || '-'
                              }}
                            </div>
                          </div>
                          <div class="lead-bg-risk-card">
                            <div class="lead-bg-risk-icon">
                              <svg
                                width="20"
                                height="20"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                class="text-orange-500"
                              >
                                <circle cx="12" cy="12" r="10" />
                                <line x1="12" y1="8" x2="12" y2="12" />
                                <line x1="12" y1="16" x2="12.01" y2="16" />
                              </svg>
                            </div>
                            <div class="lead-bg-risk-title">合规风险</div>
                            <div class="lead-bg-risk-content">
                              {{
                                bgReport.reportData?.risk_assessment
                                  ?.compliance_risk || '-'
                              }}
                            </div>
                          </div>
                          <div class="lead-bg-risk-card">
                            <div class="lead-bg-risk-icon">
                              <svg
                                width="20"
                                height="20"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                class="text-purple-500"
                              >
                                <path
                                  d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"
                                />
                              </svg>
                            </div>
                            <div class="lead-bg-risk-title">业务稳定性风险</div>
                            <div class="lead-bg-risk-content">
                              {{
                                bgReport.reportData?.risk_assessment
                                  ?.stability_risk || '-'
                              }}
                            </div>
                          </div>
                        </div>
                      </div>

                      <!-- 销售建议 -->
                      <div
                        v-show="bgActiveTab === 'suggestion'"
                        class="lead-bg-tab-panel"
                      >
                        <div class="lead-bg-detail-grid">
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label"
                              >是否适合合作</span
                            >
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.cooperation_suggestion
                                ?.suitable || '-'
                            }}</span>
                          </div>
                          <div class="lead-bg-detail-item">
                            <span class="lead-bg-detail-label"
                              >合作注意事项</span
                            >
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.cooperation_suggestion
                                ?.notes || '-'
                            }}</span>
                          </div>
                          <div
                            class="lead-bg-detail-item"
                            style="grid-column: span 2"
                          >
                            <span class="lead-bg-detail-label"
                              >建议合作方式</span
                            >
                            <span class="lead-bg-detail-value">{{
                              bgReport.reportData?.cooperation_suggestion
                                ?.suggestion || '-'
                            }}</span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div class="lead-bg-footer">
                    <Button
                      type="primary"
                      :loading="correctingName"
                      :disabled="!correctCompanyName"
                      @click="handleCorrectCompanyName"
                    >
                      一键更正公司名称
                    </Button>
                    <Button>生成风险备注</Button>
                    <Button>打印报告</Button>
                  </div>
                </div>
              </Spin>
            </div>
          </div>
        </Spin>
      </div>
    </div>
  </div>
</template>

<style scoped>
.lead-detail {
  min-height: 100%;
  padding: 0 0 24px;
  background: hsl(var(--background));
}

/* 线索信息卡 */
.lead-info-card {
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

.lead-info-main {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

.lead-info-title-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.lead-company {
  font-size: 16px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.lead-no-tag {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  border-color: hsl(var(--border));
  border-radius: 4px;
  transform: scale(0.9);
  transform-origin: left center;
}

.lead-info-desc-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  font-size: 12px;
}

.lead-info-desc-item {
  display: flex;
  gap: 4px;
  align-items: center;
}

.lead-info-desc-sep {
  color: hsl(var(--border));
}

.lead-info-label {
  color: hsl(var(--muted-foreground));
}

.lead-info-value {
  color: hsl(var(--card-foreground) / 80%);
}

.lead-info-sub {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.lead-info-detail-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px 40px;
  margin-top: 2px;
}

.lead-info-detail-item {
  display: flex;
  gap: 6px;
  align-items: center;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 12px;
  white-space: nowrap;
}

.lead-info-detail-label {
  flex-shrink: 0;
  color: hsl(var(--muted-foreground));
}

.lead-info-detail-value {
  overflow: hidden;
  text-overflow: ellipsis;
  color: hsl(var(--card-foreground) / 80%);
  white-space: nowrap;
}

.lead-info-extra {
  display: flex;
  flex-direction: column;
  gap: 4px;
  align-items: flex-end;
  justify-content: center;
}

.lead-info-amount-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.lead-info-amount-value {
  font-size: 18px;
  font-weight: 600;
  color: #fa8c16;
}

/* ====== Tab 切换 ====== */
.lead-nav {
  display: flex;
  gap: 28px;
  align-items: center;
  padding: 0 24px;
  margin: 12px 24px 0;
  font-size: 13px;
  background: hsl(var(--card));
  border-bottom: 1px solid hsl(var(--border));
}

.lead-nav-item {
  display: flex;
  gap: 6px;
  align-items: center;
  padding: 10px 0;
  margin-bottom: -1px;
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  user-select: none;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.lead-nav-item:hover {
  color: hsl(var(--card-foreground));
}

.lead-nav-item.active {
  font-weight: 600;
  color: #2563eb;
  border-bottom-color: #2563eb;
}

.lead-nav-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  font-size: 11px;
  font-weight: 600;
  line-height: 1;
  color: #fff;
  background: #2563eb;
  border-radius: 9px;
}

/* ====== 主体布局 ====== */
.lead-body {
  display: flex;
  gap: 16px;
  align-items: flex-start;
  margin: 14px 24px 0;
}

.lead-main {
  width: 100%;
  min-width: 0;
}

.lead-side {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
  gap: 12px;
  width: 360px;
}

.lead-tab-content {
  width: 100%;
  padding: 0;
}

/* ====== 表单 ====== */
.lead-form-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.lead-form-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.lead-form-row {
  display: flex;
  gap: 16px;
}

.lead-form-item {
  flex: 1;
}

.lead-form-footer {
  display: flex;
  justify-content: center;
  padding: 16px 0 20px;
  margin-top: 8px;
  border-top: 1px solid hsl(var(--border));
}

.lead-submit-btn {
  min-width: 120px;
  font-weight: 500;
  background: #2563eb;
  border-color: #2563eb;
}

.lead-submit-btn:hover,
.lead-submit-btn:focus {
  background: #1d4ed8;
  border-color: #1d4ed8;
}

/* 分隔 */
.lead-section-divider {
  position: relative;
  margin: 20px 0 16px;
  text-align: center;
}

.lead-section-divider::before {
  position: absolute;
  top: 50%;
  left: 0;
  width: 100%;
  height: 1px;
  content: '';
  background: hsl(var(--border));
}

.lead-section-divider-text {
  position: relative;
  padding: 0 12px;
  font-size: 12px;
  font-weight: 500;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted) / 40%);
}

/* ====== 空状态 ====== */
.lead-empty-followup {
  padding: 60px 0;
  text-align: center;
}

/* ====== 时间轴 ====== */
.lead-timeline {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.lead-tl-item {
  position: relative;
  display: flex;
  gap: 12px;
  padding-bottom: 20px;
}

.lead-tl-dot {
  position: relative;
  z-index: 1;
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  margin-top: 5px;
  border-radius: 50%;
}

.lead-tl-item::before {
  position: absolute;
  top: 14px;
  bottom: 0;
  left: 3.5px;
  width: 1px;
  content: '';
  background: hsl(var(--border));
}

.lead-tl-item:last-child::before {
  display: none;
}

.lead-tl-body {
  flex: 1;
  min-width: 0;
}

.lead-tl-time {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 4px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.lead-tl-stage-tag {
  margin: 0;
  border-radius: 3px;
  transform: scale(0.9);
  transform-origin: left center;
}

.lead-tl-user-name {
  color: hsl(var(--muted-foreground));
}

.lead-tl-content {
  font-size: 13px;
  line-height: 1.7;
  color: hsl(var(--card-foreground) / 80%);
  word-break: break-all;
  white-space: pre-wrap;
}

.lead-tl-next {
  display: flex;
  align-items: center;
  margin-top: 6px;
  font-size: 11px;
  color: #f59e0b;
}

/* ====== 右栏 ====== */
.lead-right-section {
  padding: 14px 16px;
  background: hsl(var(--muted) / 40%);
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.lead-right-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 8px;
  margin-bottom: 12px;
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
  border-bottom: 1px solid hsl(var(--border));
}

.lead-toggle-link {
  display: flex;
  gap: 2px;
  align-items: center;
  font-size: 12px;
  font-weight: 400;
  color: #2563eb;
  cursor: pointer;
}

.lead-toggle-link:hover {
  color: #1d4ed8;
}

/* 概要列表 */
.lead-summary-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.lead-summary-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
}

.lead-summary-label {
  flex-shrink: 0;
  color: hsl(var(--muted-foreground));
}

.lead-summary-value {
  color: hsl(var(--card-foreground) / 85%);
  text-align: right;
  word-break: break-all;
}

.lead-summary-link {
  display: flex;
  align-items: center;
  font-size: 12px;
  color: #2563eb;
}

.lead-summary-extra {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-top: 12px;
  margin-top: 12px;
  border-top: 1px dashed hsl(var(--border));
}

/* 跟进操作卡片 */
.lead-followup-card {
  position: sticky;
  top: 0;
}

.lead-followup-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.lead-fu-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.lead-fu-label {
  font-size: 12px;
  font-weight: 500;
  color: hsl(var(--muted-foreground));
}

.lead-fu-row {
  display: flex;
  gap: 10px;
}

.lead-fu-half {
  flex: 1;
}

.lead-fu-submit {
  width: 100%;
  margin-top: 4px;
  background: #2563eb;
  border-color: #2563eb;
}

.lead-fu-submit:hover:not(:disabled),
.lead-fu-submit:focus:not(:disabled) {
  background: #1d4ed8;
  border-color: #1d4ed8;
}

/* 编辑页布局 */
.lead-edit-layout {
  display: flex;
  gap: 20px;
  align-items: flex-start;
  width: 100%;
}

.lead-edit-create {
  justify-content: center;
}

.lead-edit-create .lead-edit-form {
  width: 100%;
  max-width: 720px;
}

.lead-edit-history {
  position: sticky;
  top: 0;
  flex-shrink: 0;
  width: 40%;
  padding: 16px;
  background: hsl(var(--muted) / 40%);
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.lead-edit-history-title {
  padding-bottom: 10px;
  margin-bottom: 16px;
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
  border-bottom: 1px solid hsl(var(--border));
}

.lead-edit-form {
  flex: 1;
  min-width: 0;
  padding: 20px 24px 8px;
  background: hsl(var(--muted) / 40%);
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

/* 编辑历史时间轴 */
.lead-edit-timeline {
  display: flex;
  flex-direction: column;
  gap: 0;
  max-height: calc(100vh - 280px);
  padding-right: 4px;
  overflow-y: auto;
}

.lead-edit-tl-item {
  position: relative;
  display: flex;
  gap: 10px;
  padding-bottom: 18px;
}

.lead-edit-tl-dot {
  position: relative;
  z-index: 1;
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  margin-top: 5px;
  background: #2563eb;
  border-radius: 50%;
}

.lead-edit-tl-item::before {
  position: absolute;
  top: 14px;
  bottom: 0;
  left: 3.5px;
  width: 1px;
  content: '';
  background: hsl(var(--border));
}

.lead-edit-tl-item:last-child::before {
  display: none;
}

.lead-edit-tl-body {
  flex: 1;
  min-width: 0;
}

.lead-edit-tl-time {
  margin-bottom: 2px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.lead-edit-tl-user {
  color: hsl(var(--muted-foreground));
}

.lead-edit-tl-action {
  margin-bottom: 6px;
  font-size: 12px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.lead-edit-tl-changes {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.lead-edit-tl-change {
  display: flex;
  gap: 4px;
  align-items: center;
  font-size: 11px;
  line-height: 1.5;
  color: hsl(var(--muted-foreground));
}

.lead-edit-tl-field {
  color: hsl(var(--muted-foreground));
}

.lead-edit-tl-arrow {
  color: hsl(var(--border));
}

.lead-edit-tl-new {
  font-weight: 500;
  color: hsl(var(--card-foreground) / 80%);
}

/* 跟进记录页布局 */
.lead-followup-layout {
  display: flex;
  gap: 20px;
  align-items: flex-start;
  width: 100%;
}

.lead-followup-list {
  flex: 1;
  min-width: 0;
  padding: 20px 24px 8px;
  background: hsl(var(--muted) / 40%);
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.lead-followup-form-wrap {
  position: sticky;
  top: 0;
  flex-shrink: 0;
  width: 320px;
}

/* 响应式：中等屏幕（抽屉及以下） */
@media (max-width: 1200px) {
  .lead-body {
    margin: 14px 16px 0;
  }

  .lead-info-card {
    padding: 16px 20px;
  }

  .lead-info-detail-grid {
    gap: 8px 24px;
  }

  .lead-edit-history {
    width: 35% !important;
  }
}

/* 响应式：平板及以下 */
@media (max-width: 900px) {
  .lead-info-main {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }

  .lead-info-extra {
    align-items: flex-start;
    width: 100%;
  }

  .lead-info-detail-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 8px 24px;
  }

  .lead-edit-layout {
    flex-direction: column;
  }

  .lead-edit-history {
    position: static;
    order: 2;
    width: 100% !important;
  }

  .lead-edit-form {
    order: 1;
    width: 100%;
  }

  .lead-followup-layout {
    flex-direction: column;
  }

  .lead-followup-list {
    order: 2;
    width: 100%;
  }

  .lead-followup-form-wrap {
    position: static;
    order: 1;
    width: 100%;
  }
}

/* 响应式：手机端 */
@media (max-width: 600px) {
  .lead-body {
    margin: 10px 12px 0;
  }

  .lead-info-card {
    padding: 14px 16px;
  }

  .lead-title-row {
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
  }

  .lead-title {
    font-size: 18px;
  }

  .lead-sub-info {
    flex-direction: column;
    gap: 4px;
    align-items: flex-start;
  }

  .lead-sub-sep {
    display: none;
  }

  .lead-info-detail-grid {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .lead-tab-nav {
    padding: 0 12px;
  }

  .lead-edit-form {
    padding: 16px;
  }

  .lead-edit-history {
    padding: 12px;
  }

  .lead-followup-list {
    padding: 16px;
  }

  .lead-followup-form-wrap :deep(.ant-card-body) {
    padding: 16px;
  }
}

/* 企业背调 */
.lead-bg-container {
  width: 100%;
  min-height: 500px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.lead-bg-empty {
  display: flex;
  flex-direction: column;
  gap: 16px;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
}

.lead-bg-empty-icon {
  margin-bottom: 8px;
}

.lead-bg-empty-title {
  font-size: 16px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.lead-bg-empty-desc {
  font-size: 13px;
  color: hsl(var(--muted-foreground));
}

.lead-bg-report {
  padding: 24px;
}

.lead-bg-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 20px;
}

.lead-bg-header-left {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.lead-bg-company-name {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.lead-bg-report-time {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.lead-bg-info-row {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  margin-bottom: 12px;
}

.lead-bg-info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 180px;
}

.lead-bg-info-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.lead-bg-info-value {
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
}

.lead-bg-score-row {
  display: flex;
  gap: 20px;
  margin: 24px 0;
}

.lead-bg-score-card {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 12px;
  padding: 20px;
  background: hsl(var(--muted) / 40%);
  border-radius: 8px;
}

.lead-bg-score-label {
  font-size: 13px;
  color: hsl(var(--muted-foreground));
}

.lead-bg-score-value {
  font-size: 36px;
  font-weight: 700;
}

.lead-bg-score-value.risk-default {
  color: hsl(var(--muted-foreground));
}

.lead-bg-score-value.risk-high {
  color: #ff4d4f;
}

.lead-bg-score-value.risk-medium {
  color: #fa8c16;
}

.lead-bg-score-value.risk-low {
  color: #faad14;
}

.lead-bg-score-value.risk-safe {
  color: #52c41a;
}

.lead-bg-score-bar {
  height: 6px;
  overflow: hidden;
  background: hsl(var(--border));
  border-radius: 3px;
}

.lead-bg-score-bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s;
}

.lead-bg-score-bar-fill.risk-default {
  background: hsl(var(--border));
}

.lead-bg-score-bar-fill.risk-high {
  background: #ff4d4f;
}

.lead-bg-score-bar-fill.risk-medium {
  background: #fa8c16;
}

.lead-bg-score-bar-fill.risk-low {
  background: #faad14;
}

.lead-bg-score-bar-fill.risk-safe {
  background: #52c41a;
}

.lead-bg-score-tag {
  align-self: flex-start;
  font-size: 12px;
}

.lead-bg-suggestion-card {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 12px;
  padding: 20px;
  background: hsl(var(--muted) / 40%);
  border-radius: 8px;
}

.lead-bg-suggestion-label {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.lead-bg-suggestion-content {
  font-size: 13px;
  line-height: 1.7;
  color: hsl(var(--card-foreground) / 80%);
}

.lead-bg-tabs {
  margin-top: 24px;
}

.lead-bg-tab-nav {
  display: flex;
  gap: 20px;
  padding-bottom: 16px;
  margin-bottom: 20px;
  border-bottom: 2px solid hsl(var(--border));
}

.lead-bg-tab-item {
  padding-bottom: 12px;
  font-size: 14px;
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.lead-bg-tab-item:hover {
  color: hsl(var(--card-foreground));
}

.lead-bg-tab-item.active {
  font-weight: 600;
  color: #2563eb;
  border-bottom-color: #2563eb;
}

.lead-bg-tab-content {
  min-height: 200px;
}

.lead-bg-tab-panel {
  padding: 12px 0;
}

.lead-bg-detail-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.lead-bg-detail-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  background: hsl(var(--muted) / 30%);
  border-radius: 6px;
}

.lead-bg-detail-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.lead-bg-detail-value {
  font-size: 13px;
  line-height: 1.6;
  color: hsl(var(--card-foreground));
}

.lead-bg-risk-cards {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.lead-bg-risk-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 20px;
  background: hsl(var(--muted) / 30%);
  border-radius: 8px;
}

.lead-bg-risk-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: hsl(var(--card));
  border-radius: 8px;
}

.lead-bg-risk-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.lead-bg-risk-content {
  font-size: 13px;
  line-height: 1.6;
  color: hsl(var(--card-foreground) / 80%);
}

.lead-bg-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding-top: 16px;
  margin-top: 24px;
  border-top: 1px solid hsl(var(--border));
}

/* 响应式：企业背调 */
@media (max-width: 900px) {
  .lead-bg-info-row {
    flex-direction: column;
    gap: 12px;
  }

  .lead-bg-info-item {
    min-width: auto;
  }

  .lead-bg-score-row {
    flex-direction: column;
  }

  .lead-bg-detail-grid {
    grid-template-columns: 1fr;
  }

  .lead-bg-risk-cards {
    grid-template-columns: 1fr;
  }
}
</style>
