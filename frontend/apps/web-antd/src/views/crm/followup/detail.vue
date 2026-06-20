<script lang="ts" setup>
import { ref, watch } from 'vue';

import { LucidePhone, LucideMail, LucideMessageCircle, LucideCalendar, LucideTimer, LucideUser, LucideFileText, LucideTarget, LucideBuilding2 } from '@vben/icons';

import { Descriptions, Tag, Card, Spin, Empty, Timeline, message } from 'ant-design-vue';

import { getFollowupInfoApi } from '#/api';

const props = defineProps<{ id: number }>();

const loading = ref(false);
const followup = ref<any>(null);

// 跟进方式映射
const activityLabelMap: Record<string, string> = {
  call: '电话', visit: '拜访', email: '邮件', meeting: '会议',
  demo: '演示', whatsapp: 'WhatsApp', wechat: '微信', other: '其他',
};
const activityColorMap: Record<string, string> = {
  call: 'blue', visit: 'cyan', email: 'purple', meeting: 'orange',
  demo: 'green', whatsapp: 'lime', wechat: 'lime', other: 'default',
};
const activityIconMap: Record<string, any> = {
  call: LucidePhone, visit: LucideBuilding2, email: LucideMail, meeting: LucideUser,
  demo: LucideTarget, whatsapp: LucideMessageCircle, wechat: LucideMessageCircle, other: LucideFileText,
};

async function fetchDetail() {
  if (!props.id) return;
  loading.value = true;
  try {
    const res = await getFollowupInfoApi(props.id);
    followup.value = res;
  } catch {
    message.error('获取跟进记录详情失败');
  } finally {
    loading.value = false;
  }
}

watch(() => props.id, fetchDetail, { immediate: true });
</script>

<template>
  <div class="p-4">
    <Spin :spinning="loading">
      <Empty v-if="!followup && !loading" description="暂无数据" />
      <template v-else-if="followup">
        <!-- 头部卡片 -->
        <Card :bordered="false" class="mb-4" size="small">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <component
                :is="activityIconMap[followup.activityType] || LucideFileText"
                class="h-5 w-5"
                :style="{ color: activityColorMap[followup.activityType] ? `var(--ant-color-${activityColorMap[followup.activityType]})` : undefined }"
              />
              <span class="text-xl font-bold text-gray-800">{{ followup.subject || '跟进记录' }}</span>
              <Tag :color="activityColorMap[followup.activityType] || 'default'">
                {{ activityLabelMap[followup.activityType] || followup.activityType || '-' }}
              </Tag>
              <Tag v-if="followup.result" color="green">有结果</Tag>
            </div>
            <span class="text-sm text-gray-400">{{ followup.createdAt }}</span>
          </div>
        </Card>

        <!-- 统计卡片 -->
        <div class="mb-4 grid grid-cols-4 gap-3">
          <Card :bordered="false" size="small" class="text-center">
            <div class="text-xs text-gray-400">跟进方式</div>
            <div class="text-sm font-bold">{{ activityLabelMap[followup.activityType] || followup.activityType || '-' }}</div>
          </Card>
          <Card :bordered="false" size="small" class="text-center">
            <div class="text-xs text-gray-400">耗时</div>
            <div class="text-sm font-bold">
              <LucideTimer class="mr-1 inline-block h-3.5 w-3.5" />
              {{ followup.durationMinutes ? `${followup.durationMinutes} 分钟` : '-' }}
            </div>
          </Card>
          <Card :bordered="false" size="small" class="text-center">
            <div class="text-xs text-gray-400">下次跟进</div>
            <div class="text-sm font-semibold">
              <LucideCalendar class="mr-1 inline-block h-3.5 w-3.5" />
              {{ followup.nextFollowDate || '-' }}
            </div>
          </Card>
          <Card :bordered="false" size="small" class="text-center">
            <div class="text-xs text-gray-400">跟进人</div>
            <div class="text-sm font-semibold">{{ followup.assignee || '-' }}</div>
          </Card>
        </div>

        <!-- 关联信息 -->
        <Card title="关联信息" :bordered="false" size="small" class="mb-4">
          <Descriptions :column="3" size="small" :colon="false">
            <Descriptions.Item label="线索ID">{{ followup.leadId || '-' }}</Descriptions.Item>
            <Descriptions.Item label="客户ID">{{ followup.customerId || '-' }}</Descriptions.Item>
            <Descriptions.Item label="商机ID">{{ followup.opportunityId || '-' }}</Descriptions.Item>
          </Descriptions>
        </Card>

        <!-- 跟进内容 -->
        <Card title="跟进内容" :bordered="false" size="small" class="mb-4">
          <div class="text-sm whitespace-pre-wrap leading-relaxed text-gray-700">{{ followup.content || '无内容' }}</div>
        </Card>

        <!-- 跟进结果 -->
        <Card v-if="followup.result" title="跟进结果" :bordered="false" size="small" class="mb-4">
          <div class="text-sm whitespace-pre-wrap leading-relaxed text-gray-700">{{ followup.result }}</div>
        </Card>

        <!-- 时间线 -->
        <Card title="操作时间线" :bordered="false" size="small">
          <Timeline>
            <Timeline.Item>
              <span class="text-xs text-gray-400">创建时间</span>
              <div class="text-sm">{{ followup.createdAt || '-' }}</div>
            </Timeline.Item>
            <Timeline.Item v-if="followup.nextFollowDate">
              <span class="text-xs text-gray-400">下次跟进计划</span>
              <div class="text-sm">{{ followup.nextFollowDate }}</div>
            </Timeline.Item>
            <Timeline.Item v-if="followup.updatedAt && followup.updatedAt !== followup.createdAt" color="blue">
              <span class="text-xs text-gray-400">最近更新</span>
              <div class="text-sm">{{ followup.updatedAt }}</div>
            </Timeline.Item>
          </Timeline>
        </Card>
      </template>
    </Spin>
  </div>
</template>