<script lang="ts" setup>
import { ref, watch, computed } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import {
  Button, Card, Descriptions, message, Tag, DatePicker,
  Select, Textarea, Timeline, Empty,
} from 'ant-design-vue';
import { LucideChevronDown, LucideChevronUp, LucidePhone, LucideMail, LucideUser } from '@vben/icons';
import { formatDateTime } from '@vben/utils';
import dayjs from 'dayjs';
import { getLeadInfoApi, saveFollowupApi } from '#/api';

const props = defineProps<{ id: number }>();

const loading = ref(false);
const lead = ref<any>(null);
const showMoreInfo = ref(false);
const submitting = ref(false);

const followupForm = ref({
  content: '',
  nextFollowAt: null,
  status: 2,
  method: 1,
});

const followupRecords = ref<any[]>([]);

const sourceLabelMap: Record<string, string> = {
  website: '官网', exhibition: '展会', social: '社交媒体', referral: '客户转介',
  cold_call: '陌生拜访', customs: '海关数据', email: '邮件营销', alibaba: '阿里国际站',
  amazon: 'Amazon', tiktok: 'TikTok', wechat: '微信', other: '其他',
};

const industryLabelMap: Record<string, string> = {
  retail: '零售', wholesale: '批发', manufacturer: '制造', trade_agent: '贸易代理',
  ecommerce: '电商', wechat_business: '微商', social: '社交电商', other: '其他',
};

const levelLabelMap: Record<string, string> = {
  1: '无级别', 2: '重点客户', 3: '优质客户', 4: '普通客户', 5: '其他',
};

const followStatusOptions = [
  { label: '新客', value: 1 },
  { label: '跟进中', value: 2 },
  { label: '已成交', value: 3 },
  { label: '无效线索', value: 4 },
  { label: '已回收', value: 5 },
  { label: '未核查', value: 6 },
  { label: '核查中', value: 7 },
  { label: '有效线索', value: 8 },
];

const followMethodOptions = [
  { label: '电话', value: 1, color: 'blue' },
  { label: '拜访', value: 2, color: 'cyan' },
  { label: '邮件', value: 3, color: 'purple' },
  { label: '会议', value: 4, color: 'orange' },
  { label: 'WhatsApp', value: 5, color: 'lime' },
  { label: '微信', value: 6, color: 'geekblue' },
  { label: '其他', value: 7, color: 'default' },
];

function getMethodOption(value: any) {
  return followMethodOptions.find(o => o.value === value);
}

// 跟进记录倒序（最新在前）
const reversedFollowupRecords = computed(() =>
  [...followupRecords.value].sort((a, b) =>
    new Date(b.createTime).getTime() - new Date(a.createTime).getTime()
  )
);

async function fetchDetail() {
  if (!props.id) return;
  loading.value = true;
  try {
    const res = await getLeadInfoApi(props.id);
    lead.value = res;
    followupRecords.value = res?.followups || [];
  } catch {
    message.error('获取线索详情失败');
  } finally {
    loading.value = false;
  }
}

async function handleSaveFollowup() {
  if (!followupForm.value.content.trim()) {
    message.warning('请填写跟进内容');
    return;
  }

  submitting.value = true;
  try {
    const nextDate = followupForm.value.nextFollowAt
      ? dayjs(followupForm.value.nextFollowAt).format('YYYY-MM-DD')
      : null;
    await saveFollowupApi({
      leadId: props.id,
      content: followupForm.value.content,
      nextFollowDate: nextDate,
      activityType: followupForm.value.method,
      leadStatus: followupForm.value.status,
    });
    message.success('跟进记录已保存');
    await fetchDetail();
    followupForm.value = { content: '', nextFollowAt: null, status: 2, method: 1 };
  } catch {
    // 错误提示由全局拦截器统一处理，这里不需要重复提示
  } finally {
    submitting.value = false;
  }
}

function toggleMoreInfo() {
  showMoreInfo.value = !showMoreInfo.value;
}

watch(() => props.id, fetchDetail, { immediate: true });
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex gap-4 flex-1 overflow-hidden p-4">
      <div class="flex-1 flex flex-col gap-4 overflow-hidden">
        <Card size="small" title="线索信息" class="flex-shrink-0">
          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span class="font-bold text-lg">{{ lead?.companyName || '-' }}</span>
                <Tag v-if="lead?.level" color="blue">{{ levelLabelMap[lead.level] || lead.level }}</Tag>
              </div>
              <Button type="link" @click="toggleMoreInfo" class="text-blue-600">
                <LucideChevronDown v-if="!showMoreInfo" class="inline w-4 h-4 mr-1" />
                <LucideChevronUp v-else class="inline w-4 h-4 mr-1" />
                {{ showMoreInfo ? '隐藏信息' : '显示更多' }}
              </Button>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div class="flex items-center gap-2 text-sm">
                <LucideUser class="w-4 h-4 text-gray-400" />
                <span>{{ lead?.contactName || '-' }}</span>
                <span v-if="lead?.title" class="text-gray-400">({{ lead.title }})</span>
              </div>
              <div class="flex items-center gap-2 text-sm">
                <LucidePhone class="w-4 h-4 text-gray-400" />
                <span>{{ lead?.mobile || '-' }}</span>
              </div>
              <div class="flex items-center gap-2 text-sm">
                <LucideMail class="w-4 h-4 text-gray-400" />
                <span>{{ lead?.email || '-' }}</span>
              </div>
              <div class="text-sm">
                <span class="text-gray-400">来源：</span>
                <span>{{ sourceLabelMap[lead?.source] || lead?.source || '-' }}</span>
              </div>
            </div>

            <div v-if="showMoreInfo" class="border-t pt-3 mt-2 space-y-2">
              <div class="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <span class="text-gray-400">行业：</span>
                  <span>{{ industryLabelMap[lead?.industry] || lead?.industry || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">级别：</span>
                  <span>{{ levelLabelMap[lead?.level] || lead?.level || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">国家：</span>
                  <span>{{ lead?.country || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">地区：</span>
                  <span>{{ lead?.region || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">地址：</span>
                  <span>{{ lead?.address || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">电话：</span>
                  <span>{{ lead?.phone || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">网站：</span>
                  <a v-if="lead?.website" :href="lead.website" target="_blank" class="text-blue-600 hover:underline">
                    {{ lead.website }}
                  </a>
                  <span v-else>-</span>
                </div>
                <div>
                  <span class="text-gray-400">预算：</span>
                  <span>{{ lead?.currency || '' }} {{ lead?.budget || '-' }}</span>
                </div>
              </div>
            </div>
          </div>
        </Card>

        <div class="flex-1 overflow-auto">
          <Card size="small" title="跟进记录">
            <Empty v-if="!reversedFollowupRecords.length" description="暂无跟进记录" />
            <Timeline v-else>
              <Timeline.Item
                v-for="(record, index) in reversedFollowupRecords"
                :key="record.id || index"
                :color="getMethodOption(record.activityType)?.color || 'gray'"
              >
                <div class="flex flex-wrap items-center gap-2 mb-1">
                  <Tag :color="getMethodOption(record.activityType)?.color || 'default'">
                    {{ getMethodOption(record.activityType)?.label || '未知' }}
                  </Tag>
                  <span class="text-xs text-gray-400">{{ formatDateTime(record.createTime) }}</span>
                  <span v-if="record.createdByName" class="text-xs text-gray-400">
                    · {{ record.createdByName }}
                  </span>
                </div>
                <div class="text-sm text-gray-800 whitespace-pre-wrap break-all">{{ record.content }}</div>
                <div v-if="record.nextFollowDate" class="mt-1 text-xs text-orange-500">
                  下次联系：{{ record.nextFollowDate }}
                </div>
              </Timeline.Item>
            </Timeline>
          </Card>
        </div>
      </div>

      <div class="w-96 flex-shrink-0">
        <Card size="small" title="跟进操作" class="h-full">
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">
                <span class="text-red-500">*</span> 跟进内容
              </label>
              <Textarea
                v-model:value="followupForm.content"
                placeholder="请输入跟进内容..."
                :rows="4"
                class="w-full"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">下次联系时间</label>
              <DatePicker
                v-model:value="followupForm.nextFollowAt as any"
                class="w-full"
                placeholder="选择日期"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">跟进状态</label>
              <Select
                v-model:value="followupForm.status"
                class="w-full"
                :options="followStatusOptions"
                placeholder="选择状态"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">跟进方式</label>
              <Select
                v-model:value="followupForm.method"
                class="w-full"
                :options="followMethodOptions"
                placeholder="选择方式"
              />
            </div>

            <div class="pt-2">
              <Button
                type="primary"
                class="w-full"
                :loading="submitting"
                @click="handleSaveFollowup"
              >
                立即保存
              </Button>
            </div>
          </div>
        </Card>
      </div>
    </div>
  </div>
</template>
