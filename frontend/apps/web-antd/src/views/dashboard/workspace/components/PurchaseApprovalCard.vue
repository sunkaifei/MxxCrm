<script lang="ts" setup>
// 采购审批卡（方案 6 节三期岗位卡：聚合接口 purchaseApproval 分区，我的审批待办）
// 注意：urgency 后端 DB 层为字符串、前端以数字比较，统一 Number() 归一后映射
import { onMounted, ref } from 'vue';

import { Card, Empty, Spin, Tag } from 'ant-design-vue';
import { useRouter } from 'vue-router';

import { getWorkspaceSummaryShared } from '#/api/core/system/workspace-summary';
import type { PurchaseApprovalSummary } from '#/api/core/system/workspace-summary';

import { useDashboardPermission } from '../config';

defineOptions({ name: 'WorkspacePurchaseApprovalCard' });

const router = useRouter();
const { canShow } = useDashboardPermission();

const loading = ref(false);
const summary = ref<PurchaseApprovalSummary>({ items: [], total: 0 });

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

// urgency 值域：0=普通 1=紧急 2=非常紧急（与采购请购单 drawer 的 urgencyOptions 一致）
const URGENCY_META: Record<number, { color: string; key: string }> = {
  0: { color: 'default', key: 'urgencyNormal' },
  1: { color: 'warning', key: 'urgencyHigh' },
  2: { color: 'red', key: 'urgencyCritical' },
};

function urgencyTextKey(v?: null | string): string {
  const n = Number(v ?? 0);
  const key = URGENCY_META[n]?.key || 'urgencyNormal';
  return `page.dashboard.workspace.cards.purchaseApproval.${key}`;
}

function urgencyColor(v?: null | string): string {
  const n = Number(v ?? 0);
  return URGENCY_META[n]?.color || 'default';
}

function fmtDate(v?: null | string): string {
  return typeof v === 'string' ? v.slice(0, 10) : '';
}

function goList() {
  router.push('/purchase/requisition');
}

async function loadSummary() {
  // 模块权限过滤：无权限不请求，直接空态兜底
  if (!canShow('purchaseApproval')) {
    summary.value = { items: [], total: 0 };
    return;
  }
  loading.value = true;
  try {
    const res = await getWorkspaceSummaryShared();
    summary.value = res?.purchaseApproval || { items: [], total: 0 };
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
          class="inline-block size-2 rounded-full bg-orange-500"
          aria-hidden="true"
        ></span>
        <span>
          {{ $t('page.dashboard.workspace.cards.purchaseApproval.title') }}
        </span>
        <Tag v-if="summary.total > 0" color="orange" class="ml-1">
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
          class="flex cursor-pointer items-center justify-between gap-2 rounded-md border border-gray-100 px-2 py-1.5 transition-colors hover:border-orange-300 hover:bg-orange-50/60 dark:border-gray-800 dark:hover:border-orange-700 dark:hover:bg-orange-950/40"
          @click="goList"
        >
          <div class="min-w-0 flex-1">
            <div class="truncate text-xs font-medium">
              {{ item.prNo || '-' }}
            </div>
            <div class="truncate text-xs text-gray-500">
              {{ item.title || '-' }}
            </div>
            <div class="truncate text-xs text-gray-400">
              {{ fmtDate(item.createTime) }}
            </div>
          </div>
          <div class="flex shrink-0 flex-col items-end gap-1">
            <Tag :color="urgencyColor(item.urgency)" class="mr-0">
              {{ $t(urgencyTextKey(item.urgency)) }}
            </Tag>
            <span class="text-xs font-medium text-gray-700">
              ¥ {{ fmtAmount(item.totalAmount) }}
            </span>
          </div>
        </div>
      </div>
      <Empty
        v-else
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        :description="
          $t('page.dashboard.workspace.cards.purchaseApproval.empty')
        "
        class="py-8"
      />
    </Spin>
  </Card>
</template>
