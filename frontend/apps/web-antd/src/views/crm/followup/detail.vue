<script lang="ts" setup>
import { computed, h, reactive, ref, watch } from 'vue';

import {
  LucideArrowRight,
  LucideBuilding2,
  LucideCalendar,
  LucideFilePenLine,
  LucideMail,
  LucidePhone,
  LucidePlus,
  LucideSend,
  LucideTarget,
  LucideUser,
  LucideUsers,
  SvgBellIcon,
} from '@vben/icons';
import { formatDate, formatDateTime } from '@vben/utils';

import {
  Avatar,
  Badge,
  Button,
  Card,
  DatePicker,
  Drawer,
  Empty,
  Input,
  message,
  Modal,
  Popconfirm,
  Select,
  SelectOption,
  Spin,
  Switch,
  Tabs,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import {
  convertLeadToCustomerApi,
  createContactApi,
  createFollowupApi,
  deleteContactApi,
  getCustomerContactsApi,
  getCustomerInfoApi,
  getFollowupInfoApi,
  getLeadInfoApi,
  getOpportunityListApi,
  updateContactApi,
} from '#/api';

import OpportunityDetail from '../opportunity/detail.vue';

const props = defineProps<{ id: number }>();

const loading = ref(false);
const followup = ref<any>(null);
const subject = ref<any>(null);
const submitting = ref(false);
const activeTab = ref('followup');

const sourceType = computed(() => followup.value?.sourceType ?? 0);
const isLead = computed(() => sourceType.value === 1);
const isCustomer = computed(
  () => sourceType.value === 2 || sourceType.value === 3,
);

// 跟进方式映射
const activityLabelMap: Record<number, string> = {
  1: '电话',
  2: '拜访',
  3: '邮件',
  4: '会议',
  5: 'WhatsApp',
  6: '微信',
  7: '其他',
};
const activityColorMap: Record<number, string> = {
  1: 'blue',
  2: 'cyan',
  3: 'purple',
  4: 'orange',
  5: 'lime',
  6: 'lime',
  7: 'default',
};

// 销售阶段映射
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

// 行业映射
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

// 线索状态映射
const leadStatusLabelMap: Record<number, string> = {
  1: '新客',
  2: '跟进中',
  3: '已成交',
  4: '无效线索',
  5: '已回收',
  6: '未核查',
  7: '核查中',
  8: '有效线索',
};
const leadStatusColorMap: Record<number, string> = {
  1: 'blue',
  2: 'processing',
  3: 'success',
  4: 'default',
  5: 'default',
  6: 'warning',
  7: 'processing',
  8: 'success',
};

// 客户/商机来源映射（线索后端返回字符串标识，客户后端返回数值）
const sourceStringMap: Record<string, string> = {
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

function getSourceLabel(source: any): string {
  if (source === null || source === undefined || source === '') return '-';
  // 线索返回字符串标识（如 "alibaba"）
  if (typeof source === 'string') {
    return sourceStringMap[source.toLowerCase()] || source;
  }
  // 客户/商机返回数值
  return sourceLabelMap[source as number] || String(source);
}

// 跟进表单
const followupForm = ref({
  content: '',
  activityType: 1,
  nextFollowDate: undefined as any,
  enableReminder: true,
});

// 跟进历史
const followupHistory = computed(() => {
  if (!subject.value?.followups) return [];
  return subject.value.followups.toSorted(
    (a: any, b: any) =>
      new Date(b.createTime || b.createdAt || 0).getTime() -
      new Date(a.createTime || a.createdAt || 0).getTime(),
  );
});

// 联系人
const contacts = ref<any[]>([]);
const contactsLoading = ref(false);
const contactDrawerVisible = ref(false);
const contactEditing = ref<any>(null);
const contactForm = reactive({
  name: '',
  title: '',
  mobile: '',
  email: '',
  phone: '',
  isPrimary: false,
});

// 商机
const opportunities = ref<any[]>([]);
const oppLoading = ref(false);
const oppPagination = reactive({ page: 1, pageSize: 10, total: 0 });
const oppTotalAmount = computed(() =>
  opportunities.value.reduce((sum, o) => sum + (Number(o.amount) || 0), 0),
);

async function fetchDetail() {
  if (!props.id) return;
  loading.value = true;
  try {
    const res: any = await getFollowupInfoApi(props.id);
    followup.value = res;
    const sId = res?.subjectId || res?.customerId || res?.leadId;
    const sType = res?.sourceType;
    if (sId && sType) {
      if (sType === 1) {
        const lead: any = await getLeadInfoApi(sId);
        subject.value = lead;
      } else if (sType === 2 || sType === 3) {
        const customer: any = await getCustomerInfoApi(sId);
        subject.value = customer;
      }
      if (isCustomer.value) {
        loadContacts();
        loadOpportunities();
      }
    }
  } catch {
    message.error('获取跟进详情失败');
  } finally {
    loading.value = false;
  }
}

async function loadContacts() {
  if (!subject.value?.id) return;
  contactsLoading.value = true;
  try {
    const res: any = await getCustomerContactsApi(subject.value.id);
    // 后端返回 { current: [...], history: [...] } 结构
    // current 与 history 可能包含同一联系人（解绑后重新绑定），按 id 去重，优先保留 current
    if (Array.isArray(res)) {
      contacts.value = res;
    } else if (res && (res.current || res.history)) {
      const current = Array.isArray(res.current) ? res.current : [];
      const history = Array.isArray(res.history) ? res.history : [];
      const seen = new Set<unknown>();
      const merged: any[] = [];
      for (const c of current) {
        const key = c?.id;
        if (key !== null && key !== undefined) {
          if (seen.has(key)) continue;
          seen.add(key);
        }
        merged.push(c);
      }
      for (const c of history) {
        const key = c?.id;
        if (key !== null && key !== undefined) {
          if (seen.has(key)) continue;
          seen.add(key);
        }
        merged.push(c);
      }
      contacts.value = merged;
    } else if (res && Array.isArray(res.items)) {
      contacts.value = res.items;
    } else {
      contacts.value = [];
    }
  } catch {
    contacts.value = [];
  } finally {
    contactsLoading.value = false;
  }
}

async function loadOpportunities() {
  if (!subject.value?.id) return;
  oppLoading.value = true;
  try {
    const res: any = await getOpportunityListApi({
      customerId: subject.value.id,
      page: oppPagination.page,
      pageSize: oppPagination.pageSize,
    });
    opportunities.value = res?.items ?? [];
    oppPagination.total = res?.total ?? opportunities.value.length;
  } catch {
    opportunities.value = [];
    oppPagination.total = 0;
  } finally {
    oppLoading.value = false;
  }
}

function handleOppPageChange(page: number) {
  oppPagination.page = page;
  loadOpportunities();
}

function openAddContact() {
  contactEditing.value = null;
  contactForm.name = '';
  contactForm.title = '';
  contactForm.mobile = '';
  contactForm.email = '';
  contactForm.phone = '';
  contactForm.isPrimary = false;
  contactDrawerVisible.value = true;
}

function openEditContact(contact: any) {
  contactEditing.value = contact;
  contactForm.name = contact.name || '';
  contactForm.title = contact.title || '';
  contactForm.mobile = contact.mobile || '';
  contactForm.email = contact.email || '';
  contactForm.phone = contact.phone || '';
  contactForm.isPrimary = contact.isPrimary === 1 || contact.isPrimary === true;
  contactDrawerVisible.value = true;
}

async function handleSaveContact() {
  if (!contactForm.name.trim()) {
    message.warning('请输入联系人姓名');
    return;
  }
  try {
    const payload: any = {
      customerId: subject.value.id,
      name: contactForm.name,
      title: contactForm.title,
      mobile: contactForm.mobile,
      email: contactForm.email,
      phone: contactForm.phone,
      isPrimary: contactForm.isPrimary ? 1 : 0,
    };
    if (contactEditing.value) {
      payload.id = contactEditing.value.id;
      await updateContactApi(payload);
      message.success('联系人已更新');
    } else {
      await createContactApi(payload);
      message.success('联系人已添加');
    }
    contactDrawerVisible.value = false;
    await loadContacts();
  } catch {
    message.error('保存联系人失败');
  }
}

async function handleDeleteContact(contact: any) {
  try {
    await deleteContactApi([contact.id]);
    message.success('联系人已删除');
    await loadContacts();
  } catch {
    message.error('删除联系人失败');
  }
}

// 商机操作（跳转到商机详情页或弹窗）
const opportunityDrawerVisible = ref(false);
const opportunityEditing = ref<any>(null);

function openCreateOpportunity() {
  opportunityEditing.value = null;
  opportunityDrawerVisible.value = true;
}

function openEditOpportunity(opp: any) {
  opportunityEditing.value = opp;
  opportunityDrawerVisible.value = true;
}

function handleOpportunitySuccess() {
  opportunityDrawerVisible.value = false;
  opportunityEditing.value = null;
  loadOpportunities();
}

// 线索转客户
async function handleConvertToCustomer() {
  if (!subject.value?.id) return;
  Modal.confirm({
    title: '确认转化',
    content:
      '确定将该线索转为客户吗？转换后将自动创建客户和联系人记录，且不可撤销。',
    okText: '确认转化',
    cancelText: '取消',
    onOk: async () => {
      try {
        await convertLeadToCustomerApi(subject.value.id);
        message.success('线索已转为客户');
        await fetchDetail();
      } catch {
        message.error('转化失败');
      }
    },
  });
}

// 快速发布跟进
async function handleSubmitFollowup() {
  if (!followupForm.value.content.trim()) {
    message.warning('请输入跟进内容');
    return;
  }
  submitting.value = true;
  try {
    const payload: any = {
      content: followupForm.value.content,
      activityType: followupForm.value.activityType,
      nextFollowDate: followupForm.value.nextFollowDate || undefined,
    };
    if (isLead.value && subject.value?.id) {
      payload.leadId = subject.value.id;
    } else if (isCustomer.value && subject.value?.id) {
      payload.customerId = subject.value.id;
    }
    await createFollowupApi(payload);
    message.success('跟进发布成功');
    followupForm.value.content = '';
    followupForm.value.nextFollowDate = undefined;
    await fetchDetail();
  } catch {
    message.error('跟进发布失败');
  } finally {
    submitting.value = false;
  }
}

watch(activeTab, (tab) => {
  if (
    tab === 'opportunities' &&
    opportunities.value.length === 0 &&
    oppPagination.total === 0
  ) {
    loadOpportunities();
  }
});

watch(() => props.id, fetchDetail, { immediate: true });
</script>

<template>
  <div class="followup-detail-page">
    <Spin :spinning="loading">
      <Empty v-if="!followup && !loading" description="暂无数据" />

      <template v-else-if="followup">
        <!-- ============ 顶部信息卡（按类型区分）============ -->
        <!-- 线索信息卡 -->
        <div v-if="isLead" class="profile-card profile-card-lead">
          <div class="profile-main">
            <div class="profile-title-row">
              <LucideBuilding2 class="profile-title-icon" />
              <span class="profile-title">{{
                subject?.companyName || subject?.contactName || '未命名线索'
              }}</span>
              <Tag
                :color="leadStatusColorMap[subject?.status] || 'default'"
                class="profile-tag"
              >
                {{ leadStatusLabelMap[subject?.status] || '未核查' }}
              </Tag>
              <Tag
                v-if="subject?.industry"
                class="profile-tag profile-tag-soft"
              >
                {{ industryLabelMap[subject.industry] || subject.industry }}
              </Tag>
            </div>

            <div
              v-if="
                subject?.contactName ||
                subject?.mobile ||
                subject?.email ||
                subject?.phone
              "
              class="profile-contact-row"
            >
              <span v-if="subject?.contactName" class="profile-contact-item">
                <LucideUser class="profile-contact-icon" />
                <span class="profile-contact-label">联系人</span>
                <span class="profile-contact-value">{{
                  subject.contactName
                }}</span>
                <span v-if="subject?.title" class="profile-contact-sub"
                  >({{ subject.title }})</span
                >
              </span>
              <span v-if="subject?.mobile" class="profile-contact-item">
                <LucidePhone class="profile-contact-icon" />
                <span class="profile-contact-label">手机</span>
                <span class="profile-contact-value">{{ subject.mobile }}</span>
              </span>
              <span v-if="subject?.email" class="profile-contact-item">
                <LucideMail class="profile-contact-icon" />
                <span class="profile-contact-label">邮箱</span>
                <span class="profile-contact-value">{{ subject.email }}</span>
              </span>
              <span v-if="subject?.phone" class="profile-contact-item">
                <span class="profile-contact-icon-text">☎</span>
                <span class="profile-contact-label">电话</span>
                <span class="profile-contact-value">{{ subject.phone }}</span>
              </span>
            </div>

            <div class="profile-grid">
              <div class="profile-grid-item">
                <div class="profile-grid-label">来源</div>
                <div class="profile-grid-value">
                  {{ getSourceLabel(subject?.source) }}
                </div>
              </div>
              <div class="profile-grid-item">
                <div class="profile-grid-label">国家/地区</div>
                <div class="profile-grid-value">
                  {{ subject?.country || '-'
                  }}{{ subject?.region ? ` · ${subject.region}` : '' }}
                </div>
              </div>
              <div class="profile-grid-item">
                <div class="profile-grid-label">下次跟进</div>
                <div class="profile-grid-value profile-grid-value-accent">
                  <SvgBellIcon
                    v-if="subject?.nextFollowDate || followup?.nextFollowDate"
                    class="profile-bell-icon"
                  />
                  {{
                    subject?.nextFollowDate || followup?.nextFollowDate || '-'
                  }}
                </div>
              </div>
              <div class="profile-grid-item">
                <div class="profile-grid-label">创建时间</div>
                <div class="profile-grid-value">
                  {{ subject?.createTime || subject?.createdAt || '-' }}
                </div>
              </div>
            </div>

            <!-- 全长行：地址、官网、描述 -->
            <div v-if="subject?.address" class="profile-full-row">
              <span class="profile-full-label">详细地址</span>
              <span class="profile-full-value">{{ subject.address }}</span>
            </div>
            <div v-if="subject?.website" class="profile-full-row">
              <span class="profile-full-label">官网</span>
              <a
                class="profile-full-link"
                :href="subject.website"
                target="_blank"
                rel="noopener noreferrer"
                >{{ subject.website }}</a
              >
            </div>
            <div v-if="subject?.description" class="profile-full-row">
              <span class="profile-full-label">描述</span>
              <span class="profile-full-value">{{ subject.description }}</span>
            </div>

            <!-- 创建人 -->
            <div class="profile-footer-row">
              <LucideUser class="profile-footer-icon" />
              <span class="profile-footer-label">创建人</span>
              <span class="profile-footer-value">{{
                subject?.createdByName || '-'
              }}</span>
            </div>
          </div>

          <div class="profile-extra">
            <div class="profile-stat-block">
              <div class="profile-stat-label">跟进次数</div>
              <div class="profile-stat-value">
                {{ followupHistory.length
                }}<span class="profile-stat-unit">次</span>
              </div>
            </div>
            <Button
              v-if="!subject?.convertedToCustomerId"
              type="primary"
              size="large"
              class="profile-cta"
              @click="handleConvertToCustomer"
            >
              <template #icon><LucideArrowRight :size="16" /></template>
              立即转客户
            </Button>
            <Tag v-else color="success" class="profile-converted-tag">
              已转客户
            </Tag>
          </div>
        </div>

        <!-- 客户信息卡 -->
        <div v-else-if="isCustomer" class="profile-card profile-card-customer">
          <div class="profile-main">
            <div class="profile-title-row">
              <LucideBuilding2
                v-if="subject?.customerType === 1"
                class="profile-title-icon"
              />
              <LucideUser v-else class="profile-title-icon" />
              <span class="profile-title">
                {{
                  subject?.customerType === 1
                    ? subject?.companyName
                    : subject?.personName || '未命名客户'
                }}
              </span>
              <Tag
                :color="subject?.customerType === 1 ? 'blue' : 'green'"
                class="profile-tag"
              >
                {{ subject?.customerType === 1 ? '企业' : '个人' }}
              </Tag>
              <Tag
                v-if="subject?.customerNo"
                class="profile-tag profile-tag-soft"
              >
                {{ subject.customerNo }}
              </Tag>
              <Tag
                v-if="subject?.industry"
                class="profile-tag profile-tag-soft"
              >
                {{ industryLabelMap[subject.industry] || subject.industry }}
              </Tag>
            </div>

            <div
              v-if="subject?.address || subject?.website"
              class="profile-contact-row"
            >
              <span v-if="subject?.address" class="profile-contact-item">
                <LucideBuilding2 class="profile-contact-icon" />
                <span class="profile-contact-label">地址</span>
                <span class="profile-contact-value"
                  >{{ subject.country ? `${subject.country} · ` : ''
                  }}{{ subject.address }}</span
                >
              </span>
              <span v-if="subject?.website" class="profile-contact-item">
                <LucideArrowRight class="profile-contact-icon" />
                <span class="profile-contact-label">官网</span>
                <span class="profile-contact-value">{{ subject.website }}</span>
              </span>
            </div>

            <div class="profile-grid">
              <div class="profile-grid-item">
                <div class="profile-grid-label">客户来源</div>
                <div class="profile-grid-value">
                  {{ getSourceLabel(subject?.source) }}
                </div>
              </div>
              <div class="profile-grid-item">
                <div class="profile-grid-label">国家/地区</div>
                <div class="profile-grid-value">
                  {{ subject?.country || '-'
                  }}{{ subject?.region ? ` · ${subject.region}` : '' }}
                </div>
              </div>
              <div class="profile-grid-item">
                <div class="profile-grid-label">负责人</div>
                <div class="profile-grid-value">
                  {{ subject?.assignedToName || subject?.ownerUserName || '-' }}
                </div>
              </div>
              <div class="profile-grid-item">
                <div class="profile-grid-label">创建时间</div>
                <div class="profile-grid-value">
                  {{ subject?.createTime || subject?.createdAt || '-' }}
                </div>
              </div>
            </div>

            <!-- 全长行：详细地址、描述 -->
            <div v-if="subject?.address" class="profile-full-row">
              <span class="profile-full-label">详细地址</span>
              <span class="profile-full-value"
                >{{ subject.country ? `${subject.country} · ` : ''
                }}{{ subject.region ? `${subject.region} · ` : ''
                }}{{ subject.address }}</span
              >
            </div>
            <div v-if="subject?.description" class="profile-full-row">
              <span class="profile-full-label">描述</span>
              <span class="profile-full-value">{{ subject.description }}</span>
            </div>
          </div>

          <div class="profile-extra profile-extra-customer">
            <div class="profile-kpi-row">
              <div class="profile-kpi-item">
                <div class="profile-kpi-label">联系人</div>
                <div class="profile-kpi-value">
                  {{ contacts.length }}<span class="profile-kpi-unit">人</span>
                </div>
              </div>
              <div class="profile-kpi-divider"></div>
              <div class="profile-kpi-item">
                <div class="profile-kpi-label">商机</div>
                <div class="profile-kpi-value">
                  {{ oppPagination.total
                  }}<span class="profile-kpi-unit">个</span>
                </div>
              </div>
              <div class="profile-kpi-divider"></div>
              <div class="profile-kpi-item">
                <div class="profile-kpi-label">商机总额</div>
                <div class="profile-kpi-value profile-kpi-value-accent">
                  ¥{{ oppTotalAmount.toLocaleString() }}
                </div>
              </div>
            </div>
            <Button
              type="primary"
              size="large"
              class="profile-cta"
              @click="openCreateOpportunity"
            >
              <template #icon><LucidePlus :size="16" /></template>
              新建商机
            </Button>
          </div>
        </div>

        <!-- ============ 选项卡区域 ============ -->
        <Card :bordered="false" class="content-card">
          <Tabs
            v-model:active-key="activeTab"
            class="workbench-tabs"
            :tabbarstyle="{ marginBottom: '16px' }"
          >
            <!-- ====== 跟进记录（左右分栏）====== -->
            <Tabs.TabPane
              key="followup"
              :tab="`跟进记录${followupHistory.length > 0 ? ` (${followupHistory.length})` : ''}`"
            >
              <div class="followup-layout">
                <div class="followup-list">
                  <div class="followup-list-header">
                    <span class="followup-list-title">跟进历史</span>
                    <span
                      v-if="followupHistory.length > 0"
                      class="followup-list-count"
                      >共 {{ followupHistory.length }} 条</span
                    >
                  </div>
                  <div
                    v-if="followupHistory.length === 0"
                    class="followup-empty"
                  >
                    <Empty description="暂无跟进记录" />
                  </div>
                  <div v-else class="followup-timeline">
                    <div
                      v-for="(item, idx) in followupHistory"
                      :key="item.id || idx"
                      class="followup-tl-item"
                    >
                      <div
                        class="followup-tl-dot"
                        :style="{
                          backgroundColor: activityColorMap[
                            item.activityType || item.method
                          ]
                            ? `var(--ant-color-${activityColorMap[item.activityType || item.method]}-6)`
                            : '#8c8c8c',
                        }"
                      ></div>
                      <div class="followup-tl-body">
                        <div class="followup-tl-time">
                          <Tag
                            v-if="item.activityType || item.method"
                            size="small"
                            :color="
                              activityColorMap[
                                item.activityType || item.method
                              ] || 'default'
                            "
                            class="followup-tl-tag"
                          >
                            {{
                              activityLabelMap[
                                item.activityType || item.method
                              ] || '未知'
                            }}
                          </Tag>
                          <span class="followup-tl-date">{{
                            formatDateTime(item.createTime || item.createdAt)
                          }}</span>
                          <span
                            v-if="item.createdByName"
                            class="followup-tl-user"
                            >· {{ item.createdByName }}</span
                          >
                        </div>
                        <div class="followup-tl-content">
                          {{ item.content || '-' }}
                        </div>
                        <div
                          v-if="item.nextFollowDate || item.nextFollowAt"
                          class="followup-tl-next"
                        >
                          <LucideCalendar class="next-icon" />
                          <span
                            >下次跟进:
                            {{
                              formatDate(
                                item.nextFollowDate || item.nextFollowAt,
                              )
                            }}</span
                          >
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="followup-form-wrap">
                  <div class="followup-form-card">
                    <div class="followup-form-title">
                      <LucideSend class="form-icon" />
                      <span>添加跟进</span>
                    </div>
                    <div class="followup-form-body">
                      <div class="followup-field">
                        <label class="followup-label"
                          ><span class="required-mark">*</span> 跟进内容</label
                        >
                        <Input.TextArea
                          v-model:value="followupForm.content"
                          :rows="5"
                          placeholder="请输入跟进内容..."
                          :maxlength="2000"
                          show-count
                        />
                      </div>
                      <div class="followup-row">
                        <div class="followup-field followup-half">
                          <label class="followup-label">跟进方式</label>
                          <Select
                            v-model:value="followupForm.activityType"
                            placeholder="选择方式"
                          >
                            <SelectOption
                              v-for="(label, key) in activityLabelMap"
                              :key="key"
                              :value="Number(key)"
                            >
                              {{ label }}
                            </SelectOption>
                          </Select>
                        </div>
                        <div class="followup-field followup-half">
                          <label class="followup-label">到期提醒</label>
                          <div class="reminder-wrap">
                            <Switch
                              v-model:checked="followupForm.enableReminder"
                              size="small"
                            />
                            <SvgBellIcon
                              class="reminder-bell"
                              :class="{ active: followupForm.enableReminder }"
                            />
                            <span
                              v-if="followupForm.enableReminder"
                              class="reminder-text"
                              >站内信</span
                            >
                          </div>
                        </div>
                      </div>
                      <div class="followup-field">
                        <label class="followup-label">下次跟进时间</label>
                        <DatePicker
                          v-model:value="followupForm.nextFollowDate"
                          show-time
                          format="YYYY-MM-DD HH:mm"
                          placeholder="选择跟进时间"
                          style="width: 100%"
                        />
                      </div>
                      <Button
                        type="primary"
                        block
                        :loading="submitting"
                        :icon="h(LucideSend)"
                        class="followup-submit"
                        @click="handleSubmitFollowup"
                      >
                        发布跟进
                      </Button>
                      <div
                        v-if="
                          followupForm.enableReminder &&
                          followupForm.nextFollowDate
                        "
                        class="reminder-hint"
                      >
                        <SvgBellIcon class="hint-icon" />
                        <span>将在到期时通过站内信提醒</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </Tabs.TabPane>

            <!-- ====== 客户：联系人 ====== -->
            <Tabs.TabPane
              v-if="isCustomer"
              key="contact"
              :tab="`联系人${contacts.length > 0 ? ` (${contacts.length})` : ''}`"
            >
              <div class="contact-section">
                <div class="section-header">
                  <div class="section-header-left">
                    <LucideUsers class="section-icon" />
                    <span class="section-title">联系人管理</span>
                  </div>
                  <Button type="primary" size="small" @click="openAddContact">
                    <template #icon><LucidePlus :size="12" /></template>
                    添加联系人
                  </Button>
                </div>
                <Spin :spinning="contactsLoading">
                  <Empty
                    v-if="!contactsLoading && contacts.length === 0"
                    description="暂无联系人"
                    class="contact-empty"
                  >
                    <template #extra>
                      <Button
                        type="primary"
                        size="small"
                        @click="openAddContact"
                      >
                        添加第一个联系人
                      </Button>
                    </template>
                  </Empty>
                  <div v-else class="contact-grid">
                    <div
                      v-for="(c, idx) in contacts"
                      :key="c.id || idx"
                      class="contact-card"
                    >
                      <div class="contact-card-header">
                        <Avatar class="contact-card-avatar" :size="40">
                          <LucideUser class="contact-avatar-icon" />
                        </Avatar>
                        <div class="contact-card-info">
                          <div class="contact-card-name">
                            {{ c.name || '-' }}
                            <Badge
                              v-if="c.isPrimary === 1 || c.isPrimary"
                              count="主"
                              :number-style="{
                                backgroundColor: '#52c41a',
                                fontSize: '10px',
                                padding: '0 4px',
                              }"
                            />
                          </div>
                          <div class="contact-card-position">
                            {{ c.title || '-' }}
                          </div>
                        </div>
                      </div>
                      <div class="contact-card-body">
                        <div v-if="c.mobile" class="contact-info-line">
                          <LucidePhone class="info-line-icon" />
                          <span class="info-line-text">{{ c.mobile }}</span>
                        </div>
                        <div v-if="c.email" class="contact-info-line">
                          <LucideMail class="info-line-icon" />
                          <span class="info-line-text">{{ c.email }}</span>
                        </div>
                        <div v-if="c.phone" class="contact-info-line">
                          <span class="info-line-icon">☎</span>
                          <span class="info-line-text">{{ c.phone }}</span>
                        </div>
                      </div>
                      <div class="contact-card-actions">
                        <Button
                          type="link"
                          size="small"
                          @click="openEditContact(c)"
                        >
                          <LucideFilePenLine :size="13" /> 编辑
                        </Button>
                        <Popconfirm
                          title="确定删除该联系人？"
                          @confirm="handleDeleteContact(c)"
                          ok-text="确认"
                          cancel-text="取消"
                        >
                          <Button type="link" size="small" danger>删除</Button>
                        </Popconfirm>
                      </div>
                    </div>
                  </div>
                </Spin>
              </div>
            </Tabs.TabPane>

            <!-- ====== 客户：商机 ====== -->
            <Tabs.TabPane
              v-if="isCustomer"
              key="opportunities"
              :tab="`商机${oppPagination.total > 0 ? ` (${oppPagination.total})` : ''}`"
            >
              <div class="opp-section">
                <div class="section-header">
                  <LucideTarget class="section-icon" />
                  <span class="section-title">商机列表</span>
                  <div class="section-actions">
                    <span v-if="oppTotalAmount > 0" class="opp-total-amount">
                      总金额:
                      <strong>¥{{ oppTotalAmount.toLocaleString() }}</strong>
                    </span>
                    <Button
                      type="primary"
                      size="small"
                      @click="openCreateOpportunity"
                    >
                      <template #icon><LucidePlus :size="12" /></template>
                      新建商机
                    </Button>
                  </div>
                </div>

                <Spin :spinning="oppLoading">
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
                          </div>
                        </div>
                        <div class="opp-card-details">
                          <div class="opp-detail-item">
                            <span class="opp-detail-label">金额</span>
                            <span class="opp-detail-value opp-amount">
                              {{
                                opp.amount != null
                                  ? `¥${Number(opp.amount).toLocaleString()}`
                                  : '-'
                              }}
                            </span>
                          </div>
                          <div class="opp-detail-item">
                            <span class="opp-detail-label">成交概率</span>
                            <span class="opp-detail-value">{{
                              opp.probability != null
                                ? `${opp.probability}%`
                                : '-'
                            }}</span>
                          </div>
                          <div class="opp-detail-item">
                            <span class="opp-detail-label">预计成交</span>
                            <span class="opp-detail-value">{{
                              opp.expectedCloseDate || '-'
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
                              opp.createTime ? opp.createTime.slice(0, 10) : '-'
                            }}</span>
                          </div>
                          <div class="opp-detail-item">
                            <span class="opp-detail-label">来源</span>
                            <span class="opp-detail-value">{{
                              getSourceLabel(opp.source)
                            }}</span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </Spin>

                <div
                  v-if="oppPagination.total > oppPagination.pageSize"
                  class="opp-pagination"
                >
                  <div class="opp-pagination-info">
                    第
                    {{
                      (oppPagination.page - 1) * oppPagination.pageSize + 1
                    }}-{{
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
          </Tabs>
        </Card>

        <!-- ====== 联系人编辑抽屉 ====== -->
        <Drawer
          v-model:open="contactDrawerVisible"
          :title="contactEditing ? '编辑联系人' : '添加联系人'"
          placement="right"
          width="min(480px, 92vw)"
          :destroy-on-close="true"
        >
          <div class="contact-drawer-form">
            <div class="contact-form-item">
              <label class="contact-form-label"
                ><span class="required-mark">*</span> 姓名</label
              >
              <Input
                v-model:value="contactForm.name"
                placeholder="请输入姓名"
              />
            </div>
            <div class="contact-form-item">
              <label class="contact-form-label">职位</label>
              <Input
                v-model:value="contactForm.title"
                placeholder="请输入职位"
              />
            </div>
            <div class="contact-form-item">
              <label class="contact-form-label">手机号</label>
              <Input
                v-model:value="contactForm.mobile"
                placeholder="请输入手机号"
              />
            </div>
            <div class="contact-form-item">
              <label class="contact-form-label">邮箱</label>
              <Input
                v-model:value="contactForm.email"
                placeholder="请输入邮箱"
              />
            </div>
            <div class="contact-form-item">
              <label class="contact-form-label">固定电话</label>
              <Input
                v-model:value="contactForm.phone"
                placeholder="请输入固定电话"
              />
            </div>
            <div class="contact-form-item">
              <label class="contact-form-label">主要联系人</label>
              <Switch v-model:checked="contactForm.isPrimary" />
              <span class="form-hint">设为主要联系人后将优先展示</span>
            </div>
            <div class="contact-form-actions">
              <Button @click="contactDrawerVisible = false">取消</Button>
              <Button type="primary" @click="handleSaveContact">保存</Button>
            </div>
          </div>
        </Drawer>

        <!-- ====== 商机编辑抽屉 ====== -->
        <Drawer
          v-model:open="opportunityDrawerVisible"
          :title="opportunityEditing ? '编辑商机' : '新建商机'"
          placement="right"
          width="min(1100px, 94vw)"
          :destroy-on-close="true"
          :body-style="{ paddingBottom: '80px', overflow: 'auto' }"
        >
          <OpportunityDetail
            v-if="opportunityDrawerVisible"
            :id="opportunityEditing?.id"
            :customer-id="subject?.id"
            :customer-name="subject?.companyName || subject?.personName"
            @success="handleOpportunitySuccess"
          />
        </Drawer>
      </template>
    </Spin>
  </div>
</template>

<style scoped>
/* ===== 跟进详情页 - 明暗模式兼容（使用项目 hsl(var(--*)) 令牌体系）===== */
.followup-detail-page {
  min-height: 100%;
  padding: 16px 24px 24px;
  color: hsl(var(--card-foreground));
  background: hsl(var(--background));
}

/* ============ 顶部信息卡（按类型区分）============ */
.profile-card {
  position: relative;
  display: flex;
  gap: 24px;
  align-items: stretch;
  justify-content: space-between;
  padding: 20px 24px;
  margin-bottom: 16px;
  overflow: hidden;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  box-shadow: 0 1px 2px hsl(var(--foreground) / 4%);
}

.profile-card::before {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  width: 3px;
  content: '';
  background: hsl(var(--primary));
}

.profile-card-customer::before {
  background: hsl(var(--success));
}

.profile-main {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 14px;
  min-width: 0;
}

.profile-title-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}

.profile-title-icon {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  color: hsl(var(--primary));
}

.profile-card-customer .profile-title-icon {
  color: hsl(var(--success));
}

.profile-title {
  max-width: 360px;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 17px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
  letter-spacing: 0.2px;
  white-space: nowrap;
}

.profile-tag {
  font-size: 11px;
  border-radius: 4px;
  transform: scale(0.92);
  transform-origin: left center;
}

.profile-tag-soft {
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted));
  border-color: hsl(var(--border));
}

/* 联系信息行 */
.profile-contact-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 16px;
  align-items: center;
  padding: 8px 12px;
  font-size: 12px;
  background: hsl(var(--muted) / 50%);
  border: 1px solid hsl(var(--border) / 50%);
  border-radius: 6px;
}

.profile-contact-item {
  display: flex;
  gap: 5px;
  align-items: center;
  min-width: 0;
}

.profile-contact-icon {
  flex-shrink: 0;
  width: 13px;
  height: 13px;
  color: hsl(var(--muted-foreground));
}

.profile-contact-icon-text {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.profile-contact-label {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.profile-contact-value {
  font-weight: 500;
  color: hsl(var(--card-foreground));
  word-break: break-all;
}

.profile-contact-sub {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

/* 业务详情网格 */
.profile-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px 20px;
  padding-top: 4px;
}

.profile-grid-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.profile-grid-label {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.profile-grid-value {
  display: flex;
  gap: 4px;
  align-items: center;
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
  word-break: break-all;
}

.profile-grid-value-accent {
  color: hsl(var(--warning));
}

.profile-bell-icon {
  width: 12px;
  height: 12px;
}

/* 全长行：地址、官网、描述 */
.profile-full-row {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  padding: 6px 0;
  font-size: 13px;
  line-height: 1.6;
}

.profile-full-label {
  flex-shrink: 0;
  min-width: 48px;
  padding-top: 1px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.profile-full-value {
  flex: 1;
  min-width: 0;
  color: hsl(var(--card-foreground));
  word-break: break-all;
}

.profile-full-link {
  flex: 1;
  min-width: 0;
  color: hsl(var(--primary));
  word-break: break-all;
  text-decoration: none;
  transition: opacity 0.2s;
}

.profile-full-link:hover {
  text-decoration: underline;
  opacity: 0.8;
}

/* 创建人/页脚行 */
.profile-footer-row {
  display: flex;
  gap: 6px;
  align-items: center;
  padding-top: 6px;
  margin-top: 6px;
  font-size: 12px;
  border-top: 1px solid hsl(var(--border) / 50%);
}

.profile-footer-icon {
  width: 12px;
  height: 12px;
  color: hsl(var(--muted-foreground));
}

.profile-footer-label {
  color: hsl(var(--muted-foreground));
}

.profile-footer-value {
  font-weight: 500;
  color: hsl(var(--card-foreground));
}

/* 右侧操作区 */
.profile-extra {
  display: flex;
  flex-direction: column;
  gap: 14px;
  align-items: flex-end;
  justify-content: center;
  min-width: 200px;
  padding-left: 24px;
  border-left: 1px solid hsl(var(--border));
}

.profile-stat-block {
  display: flex;
  flex-direction: column;
  gap: 4px;
  align-items: center;
}

.profile-stat-label {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.profile-stat-value {
  font-size: 26px;
  font-weight: 600;
  font-feature-settings: 'tnum';
  line-height: 1;
  color: hsl(var(--primary));
}

.profile-stat-unit {
  margin-left: 2px;
  font-size: 12px;
  font-weight: 400;
  color: hsl(var(--muted-foreground));
}

/* 客户卡 KPI 行 */
.profile-extra-customer {
  min-width: 320px;
}

.profile-kpi-row {
  display: flex;
  gap: 14px;
  align-items: center;
}

.profile-kpi-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
  align-items: center;
}

.profile-kpi-label {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  letter-spacing: 0.3px;
}

.profile-kpi-value {
  font-size: 18px;
  font-weight: 600;
  font-feature-settings: 'tnum';
  line-height: 1;
  color: hsl(var(--card-foreground));
}

.profile-kpi-value-accent {
  font-size: 16px;
  color: hsl(var(--warning));
}

.profile-kpi-unit {
  margin-left: 2px;
  font-size: 11px;
  font-weight: 400;
  color: hsl(var(--muted-foreground));
}

.profile-kpi-divider {
  width: 1px;
  height: 28px;
  background: hsl(var(--border));
}

/* CTA 按钮 */
.profile-cta {
  height: 38px;
  padding: 0 18px;
  font-weight: 500;
  border-radius: 6px;
  box-shadow: 0 2px 6px hsl(var(--primary) / 20%);
}

.profile-cta:hover {
  box-shadow: 0 4px 12px hsl(var(--primary) / 25%);
  transform: translateY(-1px);
}

.profile-converted-tag {
  padding: 4px 12px;
  font-size: 12px;
}

/* ============ 内容卡片 ============ */
.content-card {
  margin-bottom: 16px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  box-shadow: 0 1px 2px hsl(var(--foreground) / 4%);
}

.content-card :deep(.ant-card-body) {
  padding: 4px 24px 20px;
}

/* ============ 选项卡样式 ============ */
.workbench-tabs :deep(.ant-tabs-tab) {
  padding: 14px 0 !important;
  margin-right: 32px !important;
  font-size: 14px;
  color: hsl(var(--muted-foreground));
  letter-spacing: 0.2px;
  transition: color 0.2s ease;
}

.workbench-tabs :deep(.ant-tabs-tab:hover) {
  color: hsl(var(--primary));
}

.workbench-tabs :deep(.ant-tabs-tab-active .ant-tabs-tab-btn) {
  font-size: 15px;
  font-weight: 600;
  color: hsl(var(--primary)) !important;
}

.workbench-tabs :deep(.ant-tabs-ink-bar) {
  height: 2px !important;
  background: hsl(var(--primary)) !important;
}

.workbench-tabs :deep(.ant-tabs-nav) {
  margin: 0 0 20px !important;
  border-bottom: 1px solid hsl(var(--border)) !important;
}

.workbench-tabs :deep(.ant-tabs-nav::before) {
  border-bottom: none !important;
}

/* ============ 通用 section ============ */
.info-section,
.contact-section,
.opp-section {
  margin-bottom: 28px;
}

.info-section:last-child,
.contact-section:last-child,
.opp-section:last-child {
  margin-bottom: 0;
}

.section-header {
  display: flex;
  gap: 10px;
  align-items: center;
  padding-bottom: 10px;
  margin-bottom: 16px;
  border-bottom: 1px solid hsl(var(--border));
}

.section-header-left {
  display: flex;
  gap: 8px;
  align-items: center;
}

.section-icon {
  width: 15px;
  height: 15px;
  color: hsl(var(--primary));
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
  letter-spacing: 0.3px;
}

.section-actions {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-left: auto;
}

.info-desc :deep(.ant-descriptions-item-label) {
  font-size: 13px;
  color: hsl(var(--muted-foreground));
}

.info-desc :deep(.ant-descriptions-item-content) {
  font-size: 13px;
  color: hsl(var(--card-foreground));
}

/* ============ 联系人快卡（线索）============ */
.contact-quick-card {
  display: flex;
  gap: 14px;
  align-items: center;
  padding: 14px 18px;
  background: hsl(var(--muted) / 50%);
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
}

.contact-avatar-lg {
  flex-shrink: 0;
  background: hsl(var(--primary));
}

.contact-avatar-icon {
  width: 24px;
  height: 24px;
  color: #fff;
}

.contact-quick-info {
  flex: 1;
  min-width: 0;
}

.contact-quick-name {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 6px;
  font-size: 15px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.contact-quick-detail {
  display: flex;
  flex-wrap: wrap;
  gap: 18px;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
}

.detail-item {
  display: flex;
  gap: 5px;
  align-items: center;
}

.detail-icon {
  width: 13px;
  height: 13px;
  color: hsl(var(--muted-foreground));
}

/* ============ 线索转化卡 ============ */
.convert-action-card {
  display: flex;
  gap: 18px;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  margin-top: 20px;
  background: hsl(var(--primary) / 4%);
  border: 1px solid hsl(var(--border));
  border-left: 3px solid hsl(var(--primary));
  border-radius: 6px;
}

.convert-action-info {
  flex: 1;
}

.convert-action-title {
  margin-bottom: 4px;
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--primary));
}

.convert-action-desc {
  font-size: 12px;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
}

/* ============ 联系人卡片网格 ============ */
.contact-empty {
  padding: 48px 0;
}

.contact-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(290px, 1fr));
  gap: 14px;
}

.contact-card {
  padding: 14px 16px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.contact-card:hover {
  border-color: hsl(var(--primary) / 40%);
  box-shadow: 0 4px 12px hsl(var(--primary) / 8%);
  transform: translateY(-1px);
}

.contact-card-header {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 10px;
}

.contact-card-avatar {
  flex-shrink: 0;
  background: hsl(var(--primary));
}

.contact-card-info {
  flex: 1;
  min-width: 0;
}

.contact-card-name {
  display: flex;
  gap: 6px;
  align-items: center;
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.contact-card-position {
  margin-top: 2px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.contact-card-body {
  margin-bottom: 8px;
}

.contact-info-line {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 4px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.info-line-icon {
  flex-shrink: 0;
  width: 12px;
  height: 12px;
  color: hsl(var(--muted-foreground));
}

.info-line-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.contact-card-actions {
  display: flex;
  gap: 4px;
  padding-top: 8px;
  margin-top: 4px;
  border-top: 1px solid hsl(var(--border));
}

/* ============ 商机 ============ */
.opp-empty {
  padding: 48px 0;
}

.opp-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.opp-card {
  position: relative;
  padding: 14px 16px;
  overflow: hidden;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.opp-card::before {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  width: 2px;
  content: '';
  background: hsl(var(--primary));
  transform: scaleY(0);
  transform-origin: top;
  transition: transform 0.3s ease;
}

.opp-card:hover {
  border-color: hsl(var(--primary) / 40%);
  box-shadow: 0 4px 12px hsl(var(--primary) / 8%);
}

.opp-card:hover::before {
  transform: scaleY(1);
}

.opp-card-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 10px;
}

.opp-card-title-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.opp-card-title {
  font-size: 15px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.opp-card-actions {
  display: flex;
  gap: 4px;
}

.opp-card-details {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px 16px;
}

.opp-detail-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.opp-detail-label {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.opp-detail-value {
  font-size: 13px;
  color: hsl(var(--card-foreground));
}

.opp-amount {
  font-weight: 600;
  color: hsl(var(--primary));
}

.opp-total-amount {
  font-size: 13px;
  color: hsl(var(--muted-foreground));
}

.opp-total-amount strong {
  color: hsl(var(--primary));
}

.opp-pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 12px;
  margin-top: 14px;
  border-top: 1px solid hsl(var(--border));
}

.opp-pagination-info {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.opp-pagination-btns {
  display: flex;
  gap: 8px;
  align-items: center;
}

.opp-page-indicator {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ============ 跟进记录布局（左右分栏）============ */
.followup-layout {
  display: flex;
  gap: 20px;
  align-items: flex-start;
  width: 100%;
}

.followup-list {
  flex: 1;
  min-width: 0;
  padding: 16px 20px 6px;
  background: hsl(var(--muted) / 30%);
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
}

.followup-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 10px;
  margin-bottom: 14px;
  border-bottom: 1px solid hsl(var(--border));
}

.followup-list-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
}

.followup-list-count {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.followup-empty {
  padding: 48px 0;
  text-align: center;
}

.followup-timeline {
  display: flex;
  flex-direction: column;
}

.followup-tl-item {
  position: relative;
  display: flex;
  gap: 14px;
  padding-bottom: 20px;
}

.followup-tl-dot {
  position: relative;
  z-index: 1;
  flex-shrink: 0;
  width: 10px;
  height: 10px;
  margin-top: 4px;
  border: 2px solid hsl(var(--card));
  border-radius: 50%;
  box-shadow: 0 0 0 2px hsl(var(--primary));
}

.followup-tl-item::before {
  position: absolute;
  top: 18px;
  bottom: 0;
  left: 4px;
  width: 1px;
  content: '';
  background: hsl(var(--border));
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
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-bottom: 6px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.followup-tl-tag {
  margin: 0;
  border-radius: 4px;
  transform: scale(0.92);
  transform-origin: left center;
}

.followup-tl-date {
  font-size: 12px;
  color: hsl(var(--card-foreground) / 80%);
}

.followup-tl-user {
  color: hsl(var(--muted-foreground));
}

.followup-tl-content {
  padding: 10px 12px;
  font-size: 13px;
  line-height: 1.7;
  color: hsl(var(--card-foreground));
  word-break: break-all;
  white-space: pre-wrap;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-left: 2px solid hsl(var(--primary) / 60%);
  border-radius: 4px;
}

.followup-tl-next {
  display: flex;
  gap: 5px;
  align-items: center;
  margin-top: 8px;
  font-size: 12px;
  font-weight: 500;
  color: hsl(var(--warning));
}

.next-icon {
  width: 12px;
  height: 12px;
}

/* ============ 跟进表单（右侧）============ */
.followup-form-wrap {
  position: sticky;
  top: 0;
  flex-shrink: 0;
  width: 340px;
}

.followup-form-card {
  padding: 16px 18px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-top: 2px solid hsl(var(--primary));
  border-radius: 6px;
  box-shadow: 0 1px 2px hsl(var(--foreground) / 4%);
}

.followup-form-title {
  display: flex;
  gap: 8px;
  align-items: center;
  padding-bottom: 10px;
  margin-bottom: 14px;
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--card-foreground));
  border-bottom: 1px solid hsl(var(--border));
}

.form-icon {
  width: 16px;
  height: 16px;
  color: hsl(var(--primary));
}

.followup-form-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.followup-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.followup-label {
  font-size: 12px;
  font-weight: 500;
  color: hsl(var(--muted-foreground));
}

.followup-row {
  display: flex;
  gap: 10px;
}

.followup-half {
  flex: 1;
  min-width: 0;
}

.reminder-wrap {
  display: flex;
  gap: 8px;
  align-items: center;
  height: 32px;
}

.reminder-bell {
  width: 15px;
  height: 15px;
  color: hsl(var(--muted-foreground));
  transition: all 0.2s ease;
}

.reminder-bell.active {
  color: hsl(var(--warning));
  animation: bell-ring 0.6s ease;
}

@keyframes bell-ring {
  0%,
  100% {
    transform: rotate(0);
  }

  20% {
    transform: rotate(-12deg);
  }

  40% {
    transform: rotate(10deg);
  }

  60% {
    transform: rotate(-8deg);
  }

  80% {
    transform: rotate(6deg);
  }
}

.reminder-text {
  font-size: 12px;
  color: hsl(var(--warning));
}

.followup-submit {
  height: 40px;
  margin-top: 4px;
  font-weight: 500;
  border-radius: 6px;
}

.reminder-hint {
  display: flex;
  gap: 5px;
  align-items: center;
  padding: 6px 10px;
  font-size: 12px;
  color: hsl(var(--warning));
  background: hsl(var(--warning) / 8%);
  border-radius: 4px;
}

.hint-icon {
  width: 12px;
  height: 12px;
}

/* ============ 联系人抽屉表单 ============ */
.contact-drawer-form {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.contact-form-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.contact-form-label {
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--card-foreground));
}

.required-mark {
  margin-right: 2px;
  color: hsl(var(--destructive));
}

.form-hint {
  margin-left: 8px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.contact-form-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  padding-top: 16px;
  margin-top: 12px;
  border-top: 1px solid hsl(var(--border));
}

/* ============ 响应式 ============ */
@media (max-width: 1200px) {
  .profile-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 1024px) {
  .followup-layout {
    flex-direction: column;
  }

  .followup-form-wrap {
    position: static;
    width: 100%;
  }
}

@media (max-width: 768px) {
  .followup-detail-page {
    padding: 12px 16px 20px;
  }

  .profile-card {
    flex-direction: column;
    gap: 16px;
    padding: 16px 18px;
  }

  .profile-card::before {
    bottom: auto;
    width: 100%;
    height: 3px;
  }

  .profile-extra {
    align-items: stretch;
    width: 100%;
    min-width: 0;
    padding-top: 16px;
    padding-left: 0;
    border-top: 1px solid hsl(var(--border));
    border-left: none;
  }

  .profile-extra-customer {
    min-width: 0;
  }

  .profile-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .opp-card-details {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
