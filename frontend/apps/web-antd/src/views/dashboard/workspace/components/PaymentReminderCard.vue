<script lang="ts" setup>
// 待收款提醒卡（方案 6 节三期岗位卡：聚合接口 paymentReminder 分区，
// status 0=未开始 1=部分回款（后端仅返回未完成计划），按 plan_date 升序）
import { onMounted, ref } from 'vue';

import { Card, Empty, Spin, Tag } from 'ant-design-vue';
import { useRouter } from 'vue-router';

import { getWorkspaceSummaryShared } from '#/api/core/system/workspace-summary';
import type { PaymentReminderSummary } from '#/api/core/system/workspace-summary';

import { useDashboardPermission } from '../config';

defineOptions({ name: 'WorkspacePaymentReminderCard' });

const router = useRouter();
const { canShow } = useDashboardPermission();

const loading = ref(false);
const summary = ref<PaymentReminderSummary>({ items: [], total: 0 });

function toNum(v: null | number | string | undefined): number {
  const n = Number(v ?? 0);
  return Number.isFinite(n) ? n : 0;
}

// Decimal 字符串金额格式化（千分位 + 两位小数）
function fmtAmount(v: null | number | string | undefined): string {
  const n = toNum(v);
  return n.toLocaleString('en-US', {
    maximumFractionDigits: 2,
    minimumFractionDigits: 2,
  });
}

function fmtDate(v?: null | string): string {
  return typeof v === 'string' ? v.slice(0, 10) : '';
}

// 回款计划状态：0=未开始 1=部分回款
function statusTextKey(status?: null | number): string {
  return `page.dashboard.workspace.cards.paymentReminder.${
    Number(status) === 1 ? 'statusPartial' : 'statusNotStarted'
  }`;
}

function statusColor(status?: null | number): string {
  return Number(status) === 1 ? 'processing' : 'default';
}

function goList() {
  router.push('/sale/payment-plan');
}

async function loadSummary() {
  // 模块权限过滤：无权限不请求，直接空态兜底
  if (!canShow('paymentReminder')) {
    summary.value = { items: [], total: 0 };
    return;
  }
  loading.value = true;
  try {
    const res = await getWorkspaceSummaryShared();
    summary.value = res?.paymentReminder || { items: [], total: 0 };
  } catch {
    summary.value = { items: [], total: 0 };
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadSummary();
});

defineExpose({ reload: loadSummary });
</script>

<template>
  <Card
    class="flex h-full flex-col overflow-hidden"
    :body-style="{ flex: '1 1 0', minHeight: 0, overflowY: 'auto' }"
  >
    <template #title>
      <div class="flex items-center gap-2">
        <span
          class="inline-block size-2 rounded-full bg-cyan-500"
          aria-hidden="true"
        ></span>
        <span>
          {{ $t('page.dashboard.workspace.cards.paymentReminder.title') }}
        </span>
        <Tag v-if="summary.total > 0" color="cyan" class="ml-1">
          {{ summary.total }}
        </Tag>
      </div>
    </template>
    <template #extra>
      <a class="text-xs" @click="goList">
        {{ $t('page.dashboard.workspace.cards.more') }}
      </a>
    </template>
    <Spin :spinning="loading">
      <div v-if="summary.items.length > 0" class="flex flex-col gap-2">
        <div
          v-for="(item, idx) in summary.items"
          :key="idx"
          class="flex cursor-pointer items-center justify-between gap-2 rounded-md border border-gray-100 px-2 py-1.5 transition-colors hover:border-cyan-300 hover:bg-cyan-50/60 dark:border-gray-800 dark:hover:border-cyan-700 dark:hover:bg-cyan-950/40"
          @click="goList"
        >
          <div class="min-w-0 flex-1">
            <div class="truncate text-xs font-medium">
              {{ item.stageName || '-' }}
            </div>
            <div class="truncate text-xs text-gray-500">
              {{ fmtDate(item.planDate) }}
            </div>
          </div>
          <div class="flex shrink-0 flex-col items-end gap-1">
            <Tag :color="statusColor(item.status)" class="mr-0">
              {{ $t(statusTextKey(item.status)) }}
            </Tag>
            <span class="text-xs text-gray-600">
              {{ $t('page.dashboard.workspace.cards.paymentReminder.planAmount') }}
              ¥ {{ fmtAmount(item.planAmount) }}
            </span>
            <span class="text-xs text-green-600">
              {{ $t('page.dashboard.workspace.cards.paymentReminder.receivedAmount') }}
              ¥ {{ fmtAmount(item.receivedAmount) }}
            </span>
          </div>
        </div>
      </div>
      <Empty
        v-else
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        :description="$t('page.dashboard.workspace.cards.paymentReminder.empty')"
        class="py-8"
      />
    </Spin>
  </Card>
</template>
