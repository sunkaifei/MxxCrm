<script lang="ts" setup>
// 销售业绩卡（方案 6 节三期岗位卡：聚合接口 salesPerformance 分区，
// 当月新增客户/跟进次数/新增商机/成交金额（合同签订口径，Decimal 字符串需 Number()））
import { computed, onMounted, ref } from 'vue';

import { Card, Empty, Spin } from 'ant-design-vue';

import { getWorkspaceSummaryShared } from '#/api/core/system/workspace-summary';
import type { SalesPerformanceSummary } from '#/api/core/system/workspace-summary';

import { useDashboardPermission } from '../config';

defineOptions({ name: 'WorkspaceSalesPerformanceCard' });

const { canShow } = useDashboardPermission();

const loading = ref(false);
const summary = ref<SalesPerformanceSummary>({
  dealAmount: 0,
  followUps: 0,
  newCustomers: 0,
  newOpportunities: 0,
});

function toNum(v: null | number | string | undefined): number {
  const n = Number(v ?? 0);
  return Number.isFinite(n) ? n : 0;
}

// Decimal 字符串金额格式化（千分位 + 两位小数）
const dealAmountText = computed(() => {
  return toNum(summary.value.dealAmount).toLocaleString('en-US', {
    maximumFractionDigits: 2,
    minimumFractionDigits: 2,
  });
});

// 四项指标全为 0 视为本月暂无业绩，展示空态
const hasData = computed(() => {
  return (
    summary.value.newCustomers > 0 ||
    summary.value.followUps > 0 ||
    summary.value.newOpportunities > 0 ||
    toNum(summary.value.dealAmount) > 0
  );
});

async function loadSummary() {
  // 模块权限过滤：无权限不请求，直接空态兜底
  if (!canShow('salesPerformance')) {
    summary.value = {
      dealAmount: 0,
      followUps: 0,
      newCustomers: 0,
      newOpportunities: 0,
    };
    return;
  }
  loading.value = true;
  try {
    const res = await getWorkspaceSummaryShared();
    summary.value = res?.salesPerformance || {
      dealAmount: 0,
      followUps: 0,
      newCustomers: 0,
      newOpportunities: 0,
    };
  } catch {
    summary.value = {
      dealAmount: 0,
      followUps: 0,
      newCustomers: 0,
      newOpportunities: 0,
    };
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
          class="inline-block size-2 rounded-full bg-indigo-500"
          aria-hidden="true"
        ></span>
        <span>
          {{ $t('page.dashboard.workspace.cards.salesPerformance.title') }}
        </span>
      </div>
    </template>
    <Spin :spinning="loading">
      <div v-if="hasData" class="flex flex-col gap-2">
        <div class="grid grid-cols-2 gap-2">
          <div
            class="rounded-md bg-blue-50 px-3 py-2 dark:bg-blue-950"
          >
            <div class="text-xl font-semibold text-blue-600">
              {{ summary.newCustomers }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.dashboard.workspace.cards.salesPerformance.newCustomers') }}
            </div>
          </div>
          <div
            class="rounded-md bg-cyan-50 px-3 py-2 dark:bg-cyan-950"
          >
            <div class="text-xl font-semibold text-cyan-600">
              {{ summary.followUps }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.dashboard.workspace.cards.salesPerformance.followUps') }}
            </div>
          </div>
          <div
            class="rounded-md bg-purple-50 px-3 py-2 dark:bg-purple-950"
          >
            <div class="text-xl font-semibold text-purple-600">
              {{ summary.newOpportunities }}
            </div>
            <div class="text-xs text-gray-500">
              {{
                $t(
                  'page.dashboard.workspace.cards.salesPerformance.newOpportunities',
                )
              }}
            </div>
          </div>
          <div
            class="rounded-md bg-orange-50 px-3 py-2 dark:bg-orange-950"
          >
            <div class="text-base font-semibold text-orange-500">
              ¥ {{ dealAmountText }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.dashboard.workspace.cards.salesPerformance.dealAmount') }}
            </div>
          </div>
        </div>
      </div>
      <Empty
        v-else
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        :description="
          $t('page.dashboard.workspace.cards.salesPerformance.empty')
        "
        class="py-8"
      />
    </Spin>
  </Card>
</template>
