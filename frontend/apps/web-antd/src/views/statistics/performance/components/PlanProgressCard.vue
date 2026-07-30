<script lang="ts" setup>
import { computed, onMounted, ref, watch } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Card, Empty, Progress, Spin, Statistic, Tag } from 'ant-design-vue';

import { getPlanProgressSummaryApi } from '#/api/core/statistics';

interface Props {
  year: number;
}

const props = defineProps<Props>();

const loading = ref(false);
const summary = ref<any>({});

function formatCurrency(val: any) {
  const num = Number(val);
  if (!num || Number.isNaN(num)) return '¥0';
  if (num >= 100000000) return `¥${(num / 100000000).toFixed(2)}亿`;
  if (num >= 10000) return `¥${(num / 10000).toFixed(2)}万`;
  return `¥${num.toLocaleString()}`;
}

function formatPercent(val: any) {
  const num = Number(val);
  if (!num || Number.isNaN(num)) return '0.00';
  return num.toFixed(2);
}

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getPlanProgressSummaryApi({ year: props.year });
    summary.value = res?.data || res || {};
  } catch {
    summary.value = {};
  } finally {
    loading.value = false;
  }
}

const personal = computed(() => summary.value?.personal || {});
const team = computed(() => summary.value?.team || {});
// 有下属时 team.memberCount > 1
const hasTeam = computed(() => Number(team.value?.memberCount || 0) > 1);

const personalRate = computed(() => Number(personal.value?.completionRate || 0));
const teamRate = computed(() => Number(team.value?.completionRate || 0));

watch(
  () => props.year,
  () => loadData(),
);

onMounted(() => loadData());
</script>

<template>
  <Card :body-style="{ padding: '16px' }">
    <template #title>
      <div class="flex items-center gap-2">
        <IconifyIcon icon="lucide:target" class="text-lg text-primary" />
        <span>销售计划进度</span>
      </div>
    </template>
    <template #extra>
      <span class="text-xs text-gray-400">{{ year }} 年</span>
    </template>

    <Spin :spinning="loading">
      <div v-if="!personal.targetAmount && !team.targetAmount" class="py-6">
        <Empty description="暂无计划数据" />
      </div>

      <div v-else class="grid gap-4" :class="hasTeam ? 'md:grid-cols-2' : 'grid-cols-1'">
        <!-- 个人进度 -->
        <div class="p-4 rounded-lg bg-blue-50/60">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-semibold text-gray-700">我的计划</span>
            <Tag color="blue">个人</Tag>
          </div>
          <div class="grid grid-cols-2 gap-3 mb-3">
            <Statistic
              title="目标金额"
              :value="formatCurrency(personal.targetAmount)"
            />
            <Statistic
              title="实际金额"
              :value="formatCurrency(personal.actualAmount)"
              :value-style="{ color: '#1890ff' }"
            />
          </div>
          <div class="mb-1 text-xs text-gray-500 flex justify-between">
            <span>完成率</span>
            <span>{{ formatPercent(personalRate) }}%</span>
          </div>
          <Progress
            :percent="Math.min(personalRate, 100)"
            :stroke-color="personalRate >= 100 ? '#52c41a' : '#1890ff'"
            :show-info="false"
            size="small"
          />
        </div>

        <!-- 团队进度（仅有下属时显示） -->
        <div v-if="hasTeam" class="p-4 rounded-lg bg-purple-50/60">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-semibold text-gray-700">团队汇总</span>
            <Tag color="purple">
              {{ team.memberCount }} 人 · 已通过 {{ team.approvedCount || 0 }}
            </Tag>
          </div>
          <div class="grid grid-cols-2 gap-3 mb-3">
            <Statistic
              title="目标金额"
              :value="formatCurrency(team.targetAmount)"
            />
            <Statistic
              title="实际金额"
              :value="formatCurrency(team.actualAmount)"
              :value-style="{ color: '#722ed1' }"
            />
          </div>
          <div class="mb-1 text-xs text-gray-500 flex justify-between">
            <span>完成率</span>
            <span>{{ formatPercent(teamRate) }}%</span>
          </div>
          <Progress
            :percent="Math.min(teamRate, 100)"
            :stroke-color="teamRate >= 100 ? '#52c41a' : '#722ed1'"
            :show-info="false"
            size="small"
          />
        </div>
      </div>
    </Spin>
  </Card>
</template>
