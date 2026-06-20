<script lang="ts" setup>
import { ref, watch, computed } from 'vue';

import {
  LucidePhone, LucideMail, LucideUser, LucideTag, LucideEdit,
  LucideMoreHorizontal, LucideLoader2, LucideClock, LucidePlusCircle,
  LucideCheckCircle, LucideXCircle,
} from '@vben/icons';

import {
  Descriptions, Tag, Card, Spin, Empty, message, Button,
  Tabs, Timeline, Avatar, Modal, Dropdown,
} from 'ant-design-vue';

import { getLeadInfoApi, updateLeadStatusApi, addLeadToPoolApi } from '#/api';
import TagSelector from '../components/TagSelector.vue';

const props = defineProps<{ id: number }>();
const emit = defineEmits(['edit']);

const loading = ref(false);
const lead = ref<any>(null);
const activeTab = ref('info');

const statusLabelMap: Record<string, string> = {
  unchecked: '未审查',
  checking: '审查中',
  invalid: '无效线索',
  valid: '有效',
  new: '新线索',
  following: '跟进中',
  converted: '已转化',
  recycled: '已回收',
};

const statusColorMap: Record<string, string> = {
  unchecked: 'blue',
  checking: 'cyan',
  invalid: 'default',
  valid: 'green',
  new: 'blue',
  following: 'cyan',
  converted: 'green',
  recycled: 'orange',
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

const industryLabelMap: Record<string, string> = {
  retail: '零售',
  wholesale: '批发',
  manufacturer: '制造',
  trade_agent: '贸易代理',
  ecommerce: '电商',
  wechat_business: '微商',
  social: '社交电商',
  other: '其他',
};

const levelLabelMap: Record<string, string> = {
  hot: '热门',
  warm: '温',
  cold: '冷',
  a: 'A级',
  b: 'B级',
  c: 'C级',
};

const canAddToPool = computed(() => lead.value?.status !== 'valid' && lead.value?.status !== 'invalid');

async function fetchDetail() {
  if (!props.id) return;
  loading.value = true;
  try {
    const res = await getLeadInfoApi(props.id);
    lead.value = res;
  } catch {
    message.error('获取线索详情失败');
  } finally {
    loading.value = false;
  }
}

async function handleStartCheck() {
  try {
    await updateLeadStatusApi(props.id, 'checking');
    message.success('已开始审查');
    fetchDetail();
  } catch {
    message.error('操作失败');
  }
}

async function handleApprove() {
  try {
    await updateLeadStatusApi(props.id, 'valid');
    message.success('审核通过，已加入线索池');
    fetchDetail();
  } catch {
    message.error('操作失败');
  }
}

async function handleReject() {
  try {
    await updateLeadStatusApi(props.id, 'invalid');
    message.success('已标记为无效线索');
    fetchDetail();
  } catch {
    message.error('操作失败');
  }
}

async function handleAddToPool() {
  Modal.confirm({
    title: '加入线索池',
    content: `确定将线索"${lead.value?.companyName}"加入线索池吗？`,
    onOk: async () => {
      try {
        await addLeadToPoolApi(props.id);
        message.success('已加入线索池');
        fetchDetail();
      } catch {
        message.error('操作失败');
      }
    },
  });
}

watch(() => props.id, fetchDetail, { immediate: true });
</script>

<template>
  <div class="flex flex-col h-full">
    <Spin :spinning="loading">
      <Empty v-if="!lead && !loading" description="暂无数据" />
      <template v-else-if="lead">
        <div class="bg-gradient-to-r from-blue-600 to-blue-700 text-white px-6 py-4">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <span class="text-xl font-bold">{{ lead.companyName || '未命名公司' }}</span>
                <Tag :color="statusColorMap[lead.status] || 'default'" class="text-white">
                  {{ statusLabelMap[lead.status] || lead.status }}
                </Tag>
                <Tag v-if="lead.level" :color="lead.level === 'hot' ? 'red' : lead.level === 'warm' ? 'orange' : 'blue'" class="text-white">
                  {{ levelLabelMap[lead.level] || lead.level }}
                </Tag>
              </div>
              <div class="flex flex-wrap items-center gap-3 text-sm text-blue-100">
                <span v-if="lead.contactName">
                  <LucideUser class="mr-1 inline-block h-3.5 w-3.5" />{{ lead.contactName }}
                  <span v-if="lead.title" class="ml-1">({{ lead.title }})</span>
                </span>
                <span v-if="lead.email" class="flex items-center">
                  <LucideMail class="mr-1 h-3.5 w-3.5" />{{ lead.email }}
                </span>
                <span v-if="lead.mobile" class="flex items-center">
                  <LucidePhone class="mr-1 h-3.5 w-3.5" />{{ lead.mobile }}
                </span>
              </div>
            </div>
            <div class="flex items-center gap-2 ml-4">
              <Button ghost type="primary" @click="emit('edit', lead)">
                <LucideEdit class="mr-1 h-3.5 w-3.5" />编辑
              </Button>
              <Dropdown>
                <Button ghost>
                  <LucideMoreHorizontal class="h-3.5 w-3.5" />
                </Button>
              </Dropdown>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-4 gap-3 px-6 py-3 bg-white border-b">
          <div class="text-center">
            <div class="text-xs text-gray-400">预算</div>
            <div class="text-base font-bold text-blue-600">
              {{ lead.budget ? `${lead.currency || ''} ${Number(lead.budget).toLocaleString()}` : '-' }}
            </div>
          </div>
          <div class="text-center">
            <div class="text-xs text-gray-400">来源</div>
            <div class="text-sm font-medium">{{ sourceLabelMap[lead.source] || lead.source || '-' }}</div>
          </div>
          <div class="text-center">
            <div class="text-xs text-gray-400">下次跟进</div>
            <div class="text-sm font-medium">{{ lead.nextFollowAt || '-' }}</div>
          </div>
          <div class="text-center">
            <div class="text-xs text-gray-400">负责人</div>
            <div class="text-sm font-medium">{{ lead.assignee || '-' }}</div>
          </div>
        </div>

        <div v-if="lead.status === 'unchecked' || lead.status === 'checking'" class="px-6 py-2 bg-yellow-50 border-b">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <LucideLoader2 v-if="lead.status === 'checking'" class="h-3.5 w-3.5 text-yellow-600 animate-spin" />
              <LucideClock v-else class="h-3.5 w-3.5 text-blue-600" />
              <span class="text-sm font-medium text-gray-700">
                {{ lead.status === 'checking' ? '正在审查中...' : '等待审查' }}
              </span>
            </div>
            <div class="flex items-center gap-2">
              <Button v-if="lead.status === 'unchecked'" size="small" type="primary" @click="handleStartCheck">
                <LucideLoader2 class="mr-1 h-3 w-3" />开始审查
              </Button>
              <Button v-if="lead.status === 'checking'" size="small" type="success" @click="handleApprove">
                <LucideCheckCircle class="mr-1 h-3 w-3" />审核通过
              </Button>
              <Button v-if="lead.status === 'checking'" size="small" danger @click="handleReject">
                <LucideXCircle class="mr-1 h-3 w-3" />审核不通过
              </Button>
              <Button v-if="canAddToPool" size="small" ghost @click="handleAddToPool">
                <LucidePlusCircle class="mr-1 h-3 w-3" />加入线索池
              </Button>
            </div>
          </div>
        </div>

        <div class="flex-1 overflow-auto p-6 bg-gray-50">
          <Tabs v-model:activeKey="activeTab" type="card" class="bg-white rounded-lg">
            <Tabs.TabPane key="info" tab="基本信息">
              <Card :bordered="false" size="small" class="mb-3">
                <Descriptions :column="2" size="small" :colon="false">
                  <Descriptions.Item label="公司名称">
                    <span class="font-medium">{{ lead.companyName || '-' }}</span>
                  </Descriptions.Item>
                  <Descriptions.Item label="联系人">
                    {{ lead.contactName || '-' }}
                    <span v-if="lead.title" class="text-gray-400">({{ lead.title }})</span>
                  </Descriptions.Item>
                  <Descriptions.Item label="行业">
                    {{ industryLabelMap[lead.industry] || lead.industry || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="国家">
                    {{ lead.country || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="区域">
                    {{ lead.region || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="地址">
                    {{ lead.address || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="来源">
                    {{ sourceLabelMap[lead.source] || lead.source || '-' }}
                    <span v-if="lead.sourceDetail" class="ml-2 text-gray-400 text-sm">({{ lead.sourceDetail }})</span>
                  </Descriptions.Item>
                  <Descriptions.Item label="币种">
                    {{ lead.currency || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="网站">
                    <a v-if="lead.website" :href="lead.website" target="_blank" class="text-blue-600 hover:underline">
                      {{ lead.website }}
                    </a>
                    <span v-else>-</span>
                  </Descriptions.Item>
                  <Descriptions.Item label="创建人">
                    {{ lead.createdBy || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="创建时间">
                    {{ lead.createdAt || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="更新时间">
                    {{ lead.updatedAt || '-' }}
                  </Descriptions.Item>
                </Descriptions>
              </Card>

              <Card :bordered="false" size="small" class="mb-3">
                <template #title>联系方式</template>
                <Descriptions :column="2" size="small" :colon="false">
                  <Descriptions.Item label="邮箱">
                    <a :href="`mailto:${lead.email}`" class="text-blue-600">{{ lead.email || '-' }}</a>
                  </Descriptions.Item>
                  <Descriptions.Item label="电话">
                    {{ lead.phone || '-' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="手机">
                    <a :href="`tel:${lead.mobile}`" class="text-blue-600">{{ lead.mobile || '-' }}</a>
                  </Descriptions.Item>
                  <Descriptions.Item label="网站">
                    <a v-if="lead.website" :href="lead.website" target="_blank" class="text-blue-600 hover:underline">
                      {{ lead.website }}
                    </a>
                    <span v-else>-</span>
                  </Descriptions.Item>
                </Descriptions>
              </Card>

              <Card :bordered="false" size="small">
                <template #title>标签</template>
                <TagSelector
                  entity-type="lead"
                  :entity-id="lead.id"
                />
              </Card>
            </Tabs.TabPane>

            <Tabs.TabPane key="followup" tab="跟进记录">
              <Card :bordered="false" size="small" class="mb-3">
                <template #extra>
                  <Button type="primary" size="small">
                    <LucidePlusCircle class="mr-1 h-3 w-3" />新建跟进
                  </Button>
                </template>
                <Timeline v-if="lead.followups && lead.followups.length > 0">
                  <Timeline.Item v-for="(item, index) in lead.followups" :key="index">
                    <div class="flex items-start justify-between">
                      <div class="flex items-center gap-2">
                        <Avatar size="small">{{ item.createdBy?.charAt(0) || '?' }}</Avatar>
                        <span class="font-medium">{{ item.createdBy || '-' }}</span>
                      </div>
                      <span class="text-sm text-gray-400">{{ item.createdAt || '-' }}</span>
                    </div>
                    <p class="mt-1 text-sm text-gray-600">{{ item.content || '-' }}</p>
                  </Timeline.Item>
                </Timeline>
                <Empty v-else description="暂无跟进记录" />
              </Card>
            </Tabs.TabPane>

            <Tabs.TabPane key="tags" tab="标签管理">
              <Card :bordered="false" size="small">
                <TagSelector
                  entity-type="lead"
                  :entity-id="lead.id"
                />
              </Card>
            </Tabs.TabPane>

            <Tabs.TabPane key="timeline" tab="操作日志">
              <Card :bordered="false" size="small">
                <Timeline>
                  <Timeline.Item>
                    <div class="flex items-center justify-between">
                      <span class="font-medium">线索创建</span>
                      <span class="text-sm text-gray-400">{{ lead.createdAt || '-' }}</span>
                    </div>
                    <p class="mt-1 text-sm text-gray-600">创建人：{{ lead.createdBy || '-' }}</p>
                  </Timeline.Item>
                  <Timeline.Item>
                    <div class="flex items-center justify-between">
                      <span class="font-medium">状态变更</span>
                      <span class="text-sm text-gray-400">{{ lead.updatedAt || '-' }}</span>
                    </div>
                    <p class="mt-1 text-sm text-gray-600">当前状态：{{ statusLabelMap[lead.status] || lead.status }}</p>
                  </Timeline.Item>
                </Timeline>
              </Card>
            </Tabs.TabPane>
          </Tabs>
        </div>
      </template>
    </Spin>
  </div>
</template>
