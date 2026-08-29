<script lang="ts" setup>
import { computed, onMounted, ref, watch } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Button, Card, Empty, Progress, Spin, Statistic, Tag } from 'ant-design-vue';

import { getPlanProgressSummaryApi } from '#/api/core/statistics';
import { $t } from '#/locales';

interface Props {
  year: number;
  planStatus?: 'approved' | 'draft' | 'none' | 'pending' | 'rejected';
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'viewTeam'): void;
}>();

const loading = ref(false);
const summary = ref<any>({});

// 状态 Tag 与 index.vue checkPlanStatus 联动（§4.6.2）
const planStatusTag = computed(() => {
  const map: Record<string, { color: string; text: string }> = {
    none: { color: 'default', text: $t('page.statistics.performancePlan.none') },
    draft: { color: 'gold', text: $t('page.statistics.performancePlan.draft') },
    pending: { color: 'processing', text: $t('page.statistics.performancePlan.pendingBadge') },
    approved: { color: 'success', text: $t('page.statistics.performancePlan.approved') },
    rejected: { color: 'error', text: $t('page.statistics.performancePlan.rejected') },
  };
  return props.planStatus ? map[props.planStatus] : undefined;
});

function formatCurrency(val: any) {
  const num = Number(val);
  if (!num || Number.isNaN(num)) return '¥0';
  if (num >= 100_000_000)
    return `¥${(num / 100_000_000).toFixed(2)}${$t('page.statistics.performancePlan.yiUnit')}`;
  if (num >= 10_000)
    return `¥${(num / 10_000).toFixed(2)}${$t('page.statistics.performancePlan.wanUnit')}`;
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

const personalRate = computed(() =>
  Number(personal.value?.completionRate || 0),
);
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
        <span>{{ $t('page.statistics.performancePlan.planProgressTitle') }}</span>
        <Tag v-if="planStatusTag" :color="planStatusTag.color">
          {{ planStatusTag.text }}
        </Tag>
      </div>
    </template>
    <template #extra>
      <span class="text-xs text-gray-400">{{
        $t('page.statistics.performancePlan.yearValue', { n: year })
      }}</span>
    </template>

    <Spin :spinning="loading">
      <div v-if="!personal.targetAmount && !team.targetAmount" class="py-6">
        <Empty :description="$t('page.statistics.performancePlan.noPlanData')" />
      </div>

      <div
        v-else
        class="grid gap-4"
        :class="hasTeam ? 'md:grid-cols-2' : 'grid-cols-1'"
      >
        <!-- 个人进度 -->
        <div class="p-4 rounded-lg bg-blue-50/60">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-semibold text-gray-700">{{
              $t('page.statistics.performancePlan.myPlan')
            }}</span>
            <Tag color="blue">
{{
              $t('page.statistics.performancePlan.personal')
            }}
</Tag>
          </div>
          <div class="grid grid-cols-2 gap-3 mb-3">
            <Statistic
              :title="$t('page.statistics.performancePlan.targetAmount')"
              :value="formatCurrency(personal.targetAmount)"
            />
            <Statistic
              :title="$t('page.statistics.performancePlan.actualAmount')"
              :value="formatCurrency(personal.actualAmount)"
              :value-style="{ color: '#1890ff' }"
            />
          </div>
          <div class="mb-1 text-xs text-gray-500 flex justify-between">
            <span>{{ $t('page.statistics.performancePlan.completionRate') }}</span>
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
            <span class="text-sm font-semibold text-gray-700">{{
              $t('page.statistics.performancePlan.teamSummary')
            }}</span>
            <Tag color="purple">
              {{
                $t('page.statistics.performancePlan.teamMemberTag', {
                  n: team.memberCount,
                  m: team.approvedCount || 0,
                })
              }}
            </Tag>
          </div>
          <div class="grid grid-cols-2 gap-3 mb-3">
            <Statistic
              :title="$t('page.statistics.performancePlan.targetAmount')"
              :value="formatCurrency(team.targetAmount)"
            />
            <Statistic
              :title="$t('page.statistics.performancePlan.actualAmount')"
              :value="formatCurrency(team.actualAmount)"
              :value-style="{ color: '#722ed1' }"
            />
          </div>
          <div class="mb-1 text-xs text-gray-500 flex justify-between">
            <span>{{ $t('page.statistics.performancePlan.completionRate') }}</span>
            <span>{{ formatPercent(teamRate) }}%</span>
          </div>
          <Progress
            :percent="Math.min(teamRate, 100)"
            :stroke-color="teamRate >= 100 ? '#52c41a' : '#722ed1'"
            :show-info="false"
            size="small"
          />
          <div class="mt-2 text-right">
            <Button type="link" size="small" @click="emit('viewTeam')">
              {{ $t('page.statistics.performancePlan.viewTeamDetail') }}
            </Button>
          </div>
        </div>
      </div>
    </Spin>
  </Card>
</template>
