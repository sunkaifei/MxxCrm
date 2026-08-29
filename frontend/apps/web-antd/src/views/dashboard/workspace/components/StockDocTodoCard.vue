<script lang="ts" setup>
// 出入库待办卡（方案 6 节三期岗位卡：聚合接口 stockDocTodo 分区，
// status 0=草稿 1=审核中，biz_type inbound/outbound，点击行跳对应列表）
import { onMounted, ref } from 'vue';

import { Card, Empty, Spin, Tag } from 'ant-design-vue';
import { useRouter } from 'vue-router';

import { getWorkspaceSummaryShared } from '#/api/core/system/workspace-summary';
import type { StockDocTodoSummary } from '#/api/core/system/workspace-summary';

import { useDashboardPermission } from '../config';

defineOptions({ name: 'WorkspaceStockDocTodoCard' });

const router = useRouter();
const { canShow } = useDashboardPermission();

const loading = ref(false);
const summary = ref<StockDocTodoSummary>({
  inboundTotal: 0,
  items: [],
  outboundTotal: 0,
});

// 后端 Decimal 序列化为字符串，统一 Number() 后展示
function toNum(v: null | number | string | undefined): number {
  const n = Number(v ?? 0);
  return Number.isFinite(n) ? n : 0;
}

function bizTextKey(bizType?: null | string): string {
  return `page.dashboard.workspace.cards.stockDocTodo.${
    bizType === 'inbound' ? 'inbound' : 'outbound'
  }`;
}

function bizColor(bizType?: null | string): string {
  return bizType === 'inbound' ? 'blue' : 'green';
}

// 出入库单状态：0=草稿 1=审核中
function statusTextKey(status?: null | number): string {
  return `page.dashboard.workspace.cards.stockDocTodo.${
    Number(status) === 1 ? 'auditing' : 'draft'
  }`;
}

function statusColor(status?: null | number): string {
  return Number(status) === 1 ? 'processing' : 'default';
}

function goDoc(bizType?: null | string) {
  router.push(bizType === 'inbound' ? '/inbound' : '/outbound');
}

async function loadSummary() {
  // 模块权限过滤：无权限不请求，直接空态兜底
  if (!canShow('stockDocTodo')) {
    summary.value = { inboundTotal: 0, items: [], outboundTotal: 0 };
    return;
  }
  loading.value = true;
  try {
    const res = await getWorkspaceSummaryShared();
    summary.value = res?.stockDocTodo || {
      inboundTotal: 0,
      items: [],
      outboundTotal: 0,
    };
  } catch {
    summary.value = { inboundTotal: 0, items: [], outboundTotal: 0 };
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
          class="inline-block size-2 rounded-full bg-blue-500"
          aria-hidden="true"
        ></span>
        <span>
          {{ $t('page.dashboard.workspace.cards.stockDocTodo.title') }}
        </span>
      </div>
    </template>
    <Spin :spinning="loading">
      <div v-if="summary.items.length > 0">
        <div class="mb-2 flex gap-2">
          <div
            class="flex-1 rounded-md bg-blue-50 px-2 py-1.5 text-center dark:bg-blue-950"
          >
            <div class="text-lg font-semibold text-blue-600">
              {{ summary.inboundTotal }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.dashboard.workspace.cards.stockDocTodo.inbound') }}
            </div>
          </div>
          <div
            class="flex-1 rounded-md bg-green-50 px-2 py-1.5 text-center dark:bg-green-950"
          >
            <div class="text-lg font-semibold text-green-600">
              {{ summary.outboundTotal }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.dashboard.workspace.cards.stockDocTodo.outbound') }}
            </div>
          </div>
        </div>
        <div class="flex flex-col gap-2">
          <div
            v-for="(item, idx) in summary.items"
            :key="idx"
            class="flex cursor-pointer items-center justify-between gap-2 rounded-md border border-gray-100 px-2 py-1.5 transition-colors hover:border-blue-300"
            @click="goDoc(item.bizType)"
          >
            <div class="flex min-w-0 items-center gap-2">
              <Tag :color="bizColor(item.bizType)" class="mr-0">
                {{ $t(bizTextKey(item.bizType)) }}
              </Tag>
              <div class="min-w-0">
                <div class="truncate text-xs font-medium">
                  {{ item.orderNo || '-' }}
                </div>
                <div class="truncate text-xs text-gray-500">
                  {{ item.createdByName || '-' }}
                </div>
              </div>
            </div>
            <div class="flex shrink-0 flex-col items-end gap-1">
              <Tag :color="statusColor(item.status)" class="mr-0">
                {{ $t(statusTextKey(item.status)) }}
              </Tag>
              <span class="text-xs text-gray-600">
                {{ $t('page.dashboard.workspace.cards.stockDocTodo.qty') }}
                {{ toNum(item.totalQuantity) }}
              </span>
            </div>
          </div>
        </div>
      </div>
      <Empty
        v-else
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        :description="$t('page.dashboard.workspace.cards.stockDocTodo.empty')"
        class="py-8"
      />
    </Spin>
  </Card>
</template>
