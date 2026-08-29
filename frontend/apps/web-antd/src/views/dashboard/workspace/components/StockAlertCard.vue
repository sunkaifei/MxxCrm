<script lang="ts" setup>
// 库存预警卡（方案 6 节三期岗位卡：聚合接口 stockAlert 分区，权限码 product:alert:list）
import { onMounted, ref } from 'vue';

import { Card, Empty, Spin, Tag } from 'ant-design-vue';
import { useRouter } from 'vue-router';

import { getWorkspaceSummaryShared } from '#/api/core/system/workspace-summary';
import type { StockAlertSummary } from '#/api/core/system/workspace-summary';

import { useDashboardPermission } from '../config';

defineOptions({ name: 'WorkspaceStockAlertCard' });

const router = useRouter();
const { canShow } = useDashboardPermission();

const loading = ref(false);
const summary = ref<StockAlertSummary>({ items: [], total: 0 });

// 后端 Decimal 序列化为字符串，统一 Number() 后展示
function toNum(v: null | number | string | undefined): number {
  const n = Number(v ?? 0);
  return Number.isFinite(n) ? n : 0;
}

// alert_type 值域（inventory_service）：low_stock 低于下限 / high_stock 高于上限 / stale 呆滞
const ALERT_TYPE_META: Record<string, { color: string; key: string }> = {
  high_stock: { color: 'orange', key: 'highStock' },
  low_stock: { color: 'red', key: 'lowStock' },
  stale: { color: 'purple', key: 'stale' },
};

function alertTextKey(alertType?: null | string): string {
  const key = ALERT_TYPE_META[alertType || '']?.key || 'lowStock';
  return `page.dashboard.workspace.cards.stockAlert.${key}`;
}

function alertColor(alertType?: null | string): string {
  return ALERT_TYPE_META[alertType || '']?.color || 'red';
}

function goList() {
  router.push('/inventory-alert');
}

async function loadSummary() {
  // 模块权限过滤（三因子第三因子）：无权限不请求，直接空态兜底
  if (!canShow('stockAlert')) {
    summary.value = { items: [], total: 0 };
    return;
  }
  loading.value = true;
  try {
    const res = await getWorkspaceSummaryShared();
    summary.value = res?.stockAlert || { items: [], total: 0 };
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
          class="inline-block size-2 rounded-full bg-red-500"
          aria-hidden="true"
        ></span>
        <span>{{ $t('page.dashboard.workspace.cards.stockAlert.title') }}</span>
        <Tag v-if="summary.total > 0" color="red" class="ml-1">
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
          class="flex items-center justify-between gap-2 rounded-md border border-gray-100 px-2 py-1.5"
        >
          <div class="min-w-0 flex-1">
            <div class="truncate text-xs font-medium">
              {{ item.productName || '-' }}
            </div>
            <div class="truncate text-xs text-gray-500">
              {{ item.warehouseName || '-' }}
            </div>
          </div>
          <div class="flex shrink-0 flex-col items-end gap-1">
            <Tag :color="alertColor(item.alertType)" class="mr-0">
              {{ $t(alertTextKey(item.alertType)) }}
            </Tag>
            <span class="text-xs text-gray-600">
              {{ $t('page.dashboard.workspace.cards.stockAlert.quantity') }}
              {{ toNum(item.quantity) }}
            </span>
          </div>
        </div>
      </div>
      <Empty
        v-else
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        :description="$t('page.dashboard.workspace.cards.stockAlert.empty')"
        class="py-8"
      />
    </Spin>
  </Card>
</template>
