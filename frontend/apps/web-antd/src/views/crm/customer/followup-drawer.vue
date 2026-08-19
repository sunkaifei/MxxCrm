<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import {
  LucideBuilding2,
  LucideChevronDown,
  LucideChevronUp,
  LucideGlobe,
  LucideMail,
  LucidePhone,
  LucideUser,
} from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Card,
  DatePicker,
  Empty,
  message,
  Select,
  Tag,
  Textarea,
  Timeline,
} from 'ant-design-vue';
import dayjs from 'dayjs';

import { getCustomerInfoApi, saveFollowupApi } from '#/api';

const props = defineProps<{ id: number }>();
const emit = defineEmits<{ refresh: [] }>();

const loading = ref(false);
const customer = ref<any>(null);
const showMoreInfo = ref(false);
const submitting = ref(false);

const followupForm = ref({
  content: '',
  nextFollowAt: null,
  method: 1,
});

const followupRecords = ref<any[]>([]);

const sourceLabelMap: Record<string, string> = {
  Website: '官网',
  Exhibition: '展会',
  Social: '社交媒体',
  Referral: '客户转介',
  ColdCall: '陌生拜访',
  Customs: '海关数据',
  Email: '邮件营销',
  Alibaba: '阿里国际站',
  Amazon: 'Amazon',
  Tiktok: 'TikTok',
  Wechat: '微信',
  Other: '其他',
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
  { label: '电话', value: 1, color: 'blue' },
  { label: '拜访', value: 2, color: 'cyan' },
  { label: '邮件', value: 3, color: 'purple' },
  { label: '会议', value: 4, color: 'orange' },
  { label: 'WhatsApp', value: 5, color: 'lime' },
  { label: '微信', value: 6, color: 'geekblue' },
  { label: '其他', value: 7, color: 'default' },
];

function getMethodOption(value: any) {
  return followMethodOptions.find((o) => o.value === value);
}

// 跟进记录倒序（最新在前）
const reversedFollowupRecords = computed(() =>
  followupRecords.value.toSorted(
    (a, b) =>
      new Date(b.createTime).getTime() - new Date(a.createTime).getTime(),
  ),
);

async function fetchDetail() {
  if (!props.id) return;
  loading.value = true;
  try {
    const res = await getCustomerInfoApi(props.id);
    customer.value = res;
    followupRecords.value = res?.followups || [];
  } catch {
    message.error('获取客户详情失败');
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
      customerId: Number(props.id),
      content: followupForm.value.content,
      nextFollowDate: nextDate,
      activityType: Number(followupForm.value.method),
    });
    message.success('跟进记录已保存');
    emit('refresh');
    await fetchDetail();
    followupForm.value = { content: '', nextFollowAt: null, method: 1 };
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
        <Card size="small" title="客户信息" class="flex-shrink-0">
          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <LucideBuilding2 class="w-5 h-5 text-gray-400" />
                <span class="font-bold text-lg">{{
                  customer?.companyName || '-'
                }}</span>
                <Tag
                  v-if="customer?.level"
                  :color="levelColorMap[customer.level] || 'blue'"
                >
                  {{ levelLabelMap[customer.level] || customer.level }}
                </Tag>
              </div>
              <Button type="link" @click="toggleMoreInfo" class="text-blue-600">
                <LucideChevronDown
                  v-if="!showMoreInfo"
                  class="inline w-4 h-4 mr-1"
                />
                <LucideChevronUp v-else class="inline w-4 h-4 mr-1" />
                {{ showMoreInfo ? '隐藏信息' : '显示更多' }}
              </Button>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div class="flex items-center gap-2 text-sm">
                <LucideUser class="w-4 h-4 text-gray-400" />
                <span class="text-gray-400">联系人：</span>
                <span>{{ customer?.contactName || '-' }}</span>
              </div>
              <div class="flex items-center gap-2 text-sm">
                <LucidePhone class="w-4 h-4 text-gray-400" />
                <span>{{ customer?.mobile || customer?.phone || '-' }}</span>
              </div>
              <div class="flex items-center gap-2 text-sm">
                <LucideMail class="w-4 h-4 text-gray-400" />
                <span>{{ customer?.email || '-' }}</span>
              </div>
              <div class="text-sm">
                <span class="text-gray-400">来源：</span>
                <span>{{
                  sourceLabelMap[customer?.source] || customer?.source || '-'
                }}</span>
              </div>
            </div>

            <div v-if="showMoreInfo" class="border-t pt-3 mt-2 space-y-2">
              <div class="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <span class="text-gray-400">行业：</span>
                  <span>{{
                    industryLabelMap[customer?.industry] ||
                    customer?.industry ||
                    '-'
                  }}</span>
                </div>
                <div>
                  <span class="text-gray-400">级别：</span>
                  <span>{{
                    levelLabelMap[customer?.level] || customer?.level || '-'
                  }}</span>
                </div>
                <div>
                  <span class="text-gray-400">国家：</span>
                  <span>{{ customer?.country || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">地区：</span>
                  <span>{{ customer?.region || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">地址：</span>
                  <span>{{ customer?.address || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">网站：</span>
                  <a
                    v-if="customer?.website"
                    :href="customer.website"
                    target="_blank"
                    class="text-blue-600 hover:underline"
                  >
                    <LucideGlobe class="inline w-3.5 h-3.5 mr-0.5" />{{
                      customer.website
                    }}
                  </a>
                  <span v-else>-</span>
                </div>
                <div>
                  <span class="text-gray-400">负责人：</span>
                  <span>{{ customer?.assignedTo?.name || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">币种：</span>
                  <span>{{ customer?.currency || '-' }}</span>
                </div>
              </div>
            </div>
          </div>
        </Card>

        <div class="flex-1 overflow-auto">
          <Card size="small" title="跟进记录">
            <Empty
              v-if="reversedFollowupRecords.length === 0"
              description="暂无跟进记录"
            />
            <Timeline v-else>
              <Timeline.Item
                v-for="(record, index) in reversedFollowupRecords"
                :key="record.id || index"
                :color="getMethodOption(record.activityType)?.color || 'gray'"
              >
                <div class="flex flex-wrap items-center gap-2 mb-1">
                  <Tag
                    :color="
                      getMethodOption(record.activityType)?.color || 'default'
                    "
                  >
                    {{ getMethodOption(record.activityType)?.label || '未知' }}
                  </Tag>
                  <span class="text-xs text-gray-400">{{
                    formatDateTime(record.createTime)
                  }}</span>
                  <span
                    v-if="record.createdByName"
                    class="text-xs text-gray-400"
                  >
                    · {{ record.createdByName }}
                  </span>
                </div>
                <div
                  class="text-sm text-gray-800 whitespace-pre-wrap break-all"
                >
                  {{ record.content }}
                </div>
                <div
                  v-if="record.nextFollowDate"
                  class="mt-1 text-xs text-orange-500"
                >
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
              <label class="block text-sm font-medium text-gray-700 mb-1"
                >下次联系时间</label
              >
              <DatePicker
                v-model:value="followupForm.nextFollowAt as any"
                class="w-full"
                placeholder="选择日期"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1"
                >跟进方式</label
              >
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
