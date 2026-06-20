<script lang="ts" setup>
import { ref, computed, watch } from 'vue';
import { Card, Descriptions, Tag, Button, Timeline, Row, Col, Skeleton, Progress, Divider } from 'ant-design-vue';
import { LucideFilePenLine, LucideTrendingUp, LucideUser, LucideCalendar, LucideDollarSign, LucideTag, LucideLink, LucideFileText } from '@vben/icons';
import { getOpportunityInfoApi } from '#/api';
import { message } from 'ant-design-vue';

const props = defineProps<{ id?: number | string }>();
const emit = defineEmits<{
  (e: 'edit', opportunity: any): void;
}>();

const loading = ref(false);
const opp = ref<any>({});

const stageMap: Record<string, { label: string; color: string }> = {
  qualification: { label: '资格审查', color: 'blue' },
  needs_analysis: { label: '需求分析', color: 'cyan' },
  proposal: { label: '方案报价', color: 'gold' },
  negotiation: { label: '商务谈判', color: 'orange' },
  won: { label: '已成交', color: 'green' },
  lost: { label: '已输单', color: 'red' },
};

const sourceMap: Record<string, string> = {
  website: '官网', exhibition: '展会', social: '社交媒体', referral: '客户转介',
  cold_call: '陌生拜访', customs: '海关数据', email: '邮件营销', alibaba: '阿里国际站',
  amazon: 'Amazon', tiktok: 'TikTok', wechat: '微信', other: '其他',
};

const stageInfo = computed(() => stageMap[opp.value.stage] || { label: opp.value.stage || '-', color: 'default' });
const sourceLabel = computed(() => sourceMap[opp.value.source] || opp.value.source || '-');

const amountText = computed(() => {
  if (opp.value.amount == null) return '-';
  const num = Number(opp.value.amount).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  return `${opp.value.currency || ''} ${num}`;
});

const probabilityNum = computed(() => Number(opp.value.probability ?? 0));

// 阶段进度
const stageList = ['qualification', 'needs_analysis', 'proposal', 'negotiation', 'won'];
const currentStageIndex = computed(() => {
  if (opp.value.stage === 'lost') return -1;
  return stageList.indexOf(opp.value.stage);
});

const loadData = async () => {
  if (!props.id) return;
  loading.value = true;
  try {
    const result = await getOpportunityInfoApi(Number(props.id));
    opp.value = result || {};
  } finally { loading.value = false; }
};

const handleEdit = () => emit('edit', opp.value);

watch(() => props.id, () => { if (props.id) loadData(); }, { immediate: true });
</script>

<template>
  <div class="p-4">
    <Skeleton :loading="loading" active>
      <!-- 头部信息 -->
      <Card class="mb-4" :body-style="{ padding: '20px 24px' }">
        <div class="flex items-start justify-between">
          <div>
            <div class="flex items-center gap-3 mb-2">
              <Tag :color="stageInfo.color" class="text-base px-3 py-1">{{ stageInfo.label }}</Tag>
              <span v-if="opp.opportunityNo" class="text-gray-400 text-sm">商机编号: {{ opp.opportunityNo }}</span>
            </div>
            <h2 class="text-2xl font-bold m-0 mb-3">{{ opp.title || '商机详情' }}</h2>
            <div class="flex items-center gap-6 text-sm">
              <div class="flex items-center gap-2">
                <LucideDollarSign :size="16" class="text-green-600" />
                <span class="text-2xl font-bold text-green-600">{{ amountText }}</span>
              </div>
              <div class="flex items-center gap-2">
                <LucideTrendingUp :size="16" class="text-orange-500" />
                <span class="text-gray-500">赢单概率</span>
                <span class="font-bold text-orange-500">{{ probabilityNum }}%</span>
              </div>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <Button type="primary" @click="handleEdit">
              <template #icon><LucideFilePenLine /></template>编辑
            </Button>
          </div>
        </div>
      </Card>

      <!-- 阶段进度 -->
      <Card v-if="currentStageIndex >= 0" size="small" class="mb-4" :body-style="{ padding: '16px 24px' }">
        <template #title><span class="text-sm font-medium">销售阶段进度</span></template>
        <div class="flex items-center justify-between mb-3">
          <div
            v-for="(stage, index) in stageList"
            :key="stage"
            class="flex-1 flex items-center"
          >
            <div class="flex flex-col items-center flex-1">
              <div
                class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium transition-colors"
                :class="index <= currentStageIndex
                  ? (index === currentStageIndex ? 'bg-blue-500 text-white' : 'bg-blue-100 text-blue-600')
                  : 'bg-gray-100 text-gray-400'"
              >
                {{ index + 1 }}
              </div>
              <div
                class="text-xs mt-1"
                :class="index <= currentStageIndex ? 'text-blue-600 font-medium' : 'text-gray-400'"
              >
                {{ stageMap[stage].label }}
              </div>
            </div>
            <div
              v-if="index < stageList.length - 1"
              class="h-0.5 flex-1 mx-1"
              :class="index < currentStageIndex ? 'bg-blue-500' : 'bg-gray-200'"
            />
          </div>
        </div>
        <Progress
          :percent="Math.min(100, Math.round(((currentStageIndex + 1) / stageList.length) * 100))"
          :show-info="false"
          :stroke-color="stageInfo.color === 'default' ? '#1677ff' : undefined"
        />
      </Card>

      <Row :gutter="16">
        <!-- 左侧：基本信息 -->
        <Col :span="16">
          <Card size="small" class="mb-4">
            <template #title>基本信息</template>
            <Descriptions :column="2" bordered size="small">
              <Descriptions.Item label="商机编号">{{ opp.opportunityNo || '-' }}</Descriptions.Item>
              <Descriptions.Item label="商机名称" :span="2">{{ opp.title || '-' }}</Descriptions.Item>
              <Descriptions.Item label="销售阶段">
                <Tag :color="stageInfo.color">{{ stageInfo.label }}</Tag>
              </Descriptions.Item>
              <Descriptions.Item label="商机来源">
                <Tag>{{ sourceLabel }}</Tag>
              </Descriptions.Item>
              <Descriptions.Item label="商机金额">
                <span class="font-bold text-green-600">{{ amountText }}</span>
              </Descriptions.Item>
              <Descriptions.Item label="赢单概率">
                <Progress :percent="probabilityNum" :size="'small'" :stroke-color="probabilityNum >= 70 ? '#52c41a' : probabilityNum >= 30 ? '#faad14' : '#ff4d4f'" />
              </Descriptions.Item>
              <Descriptions.Item label="预计成交日期">{{ opp.expectedCloseDate || '-' }}</Descriptions.Item>
              <Descriptions.Item label="负责人ID">{{ opp.assignedTo ?? '-' }}</Descriptions.Item>
            </Descriptions>
          </Card>

          <!-- 商机描述 -->
          <Card size="small" class="mb-4" v-if="opp.description">
            <template #title>
              <span class="flex items-center gap-1"><LucideFileText :size="14" />商机描述</span>
            </template>
            <div class="text-sm text-gray-700 whitespace-pre-wrap leading-relaxed">{{ opp.description }}</div>
          </Card>
        </Col>

        <!-- 右侧：关联信息 -->
        <Col :span="8">
          <Card size="small" class="mb-4">
            <template #title>
              <span class="flex items-center gap-1"><LucideLink :size="14" />关联信息</span>
            </template>
            <div class="flex flex-col gap-3">
              <div class="flex items-center justify-between text-sm">
                <span class="text-gray-500">客户ID</span>
                <span class="font-medium text-blue-600 cursor-pointer hover:underline">{{ opp.customerId ?? '-' }}</span>
              </div>
              <Divider class="!my-0" />
              <div class="flex items-center justify-between text-sm">
                <span class="text-gray-500">联系人ID</span>
                <span class="font-medium">{{ opp.contactId ?? '-' }}</span>
              </div>
              <Divider class="!my-0" />
              <div class="flex items-center justify-between text-sm">
                <span class="text-gray-500">线索ID</span>
                <span class="font-medium">{{ opp.leadId ?? '-' }}</span>
              </div>
            </div>
          </Card>

          <Card size="small">
            <template #title>
              <span class="flex items-center gap-1"><LucideUser :size="14" />负责人</span>
            </template>
            <div class="text-center py-4">
              <div class="text-base font-medium">{{ opp.assignedTo ?? '未分配' }}</div>
              <div class="text-xs text-gray-400 mt-1">负责人ID</div>
            </div>
          </Card>
        </Col>
      </Row>
    </Skeleton>
  </div>
</template>