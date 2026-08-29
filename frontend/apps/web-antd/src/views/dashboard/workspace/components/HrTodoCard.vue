<script lang="ts" setup>
// 人事待办卡（方案 6 节三期岗位卡：聚合接口 hrTodo 分区，
// 待我审批的入职（business_type=user）与离职（resign）在途实例）
import { onMounted, ref } from 'vue';

import { Card, Empty, Spin, Tag } from 'ant-design-vue';
import { useRouter } from 'vue-router';

import { getWorkspaceSummaryShared } from '#/api/core/system/workspace-summary';
import type { HrTodoSummary } from '#/api/core/system/workspace-summary';

import { useDashboardPermission } from '../config';

defineOptions({ name: 'WorkspaceHrTodoCard' });

const router = useRouter();
const { canShow } = useDashboardPermission();

const loading = ref(false);
const summary = ref<HrTodoSummary>({
  items: [],
  onboarding: 0,
  resign: 0,
  total: 0,
});

function fmtDate(v?: null | string): string {
  return typeof v === 'string' ? v.slice(0, 10) : '';
}

function typeTextKey(businessType?: string): string {
  return `page.dashboard.workspace.cards.hrTodo.${
    businessType === 'resign' ? 'resign' : 'onboarding'
  }`;
}

function typeColor(businessType?: string): string {
  return businessType === 'resign' ? 'orange' : 'blue';
}

function goList() {
  router.push('/system/approval/todo');
}

async function loadSummary() {
  // 模块权限过滤：无权限不请求，直接空态兜底
  if (!canShow('hrTodo')) {
    summary.value = { items: [], onboarding: 0, resign: 0, total: 0 };
    return;
  }
  loading.value = true;
  try {
    const res = await getWorkspaceSummaryShared();
    summary.value = res?.hrTodo || {
      items: [],
      onboarding: 0,
      resign: 0,
      total: 0,
    };
  } catch {
    summary.value = { items: [], onboarding: 0, resign: 0, total: 0 };
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
          class="inline-block size-2 rounded-full bg-purple-500"
          aria-hidden="true"
        ></span>
        <span>{{ $t('page.dashboard.workspace.cards.hrTodo.title') }}</span>
        <Tag v-if="summary.total > 0" color="purple" class="ml-1">
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
      <div v-if="summary.items.length > 0">
        <div class="mb-2 flex gap-2">
          <div
            class="flex-1 rounded-md bg-blue-50 px-2 py-1.5 text-center dark:bg-blue-950"
          >
            <div class="text-lg font-semibold text-blue-600">
              {{ summary.onboarding }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.dashboard.workspace.cards.hrTodo.onboarding') }}
            </div>
          </div>
          <div
            class="flex-1 rounded-md bg-orange-50 px-2 py-1.5 text-center dark:bg-orange-950"
          >
            <div class="text-lg font-semibold text-orange-500">
              {{ summary.resign }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.dashboard.workspace.cards.hrTodo.resign') }}
            </div>
          </div>
        </div>
        <div class="flex flex-col gap-2">
          <div
            v-for="(item, idx) in summary.items"
            :key="idx"
            class="flex items-center justify-between gap-2 rounded-md border border-gray-100 px-2 py-1.5"
          >
            <div class="flex min-w-0 items-center gap-2">
              <Tag :color="typeColor(item.businessType)" class="mr-0">
                {{ $t(typeTextKey(item.businessType)) }}
              </Tag>
              <div class="min-w-0">
                <div class="truncate text-xs font-medium">
                  {{ item.businessTitle || '-' }}
                </div>
                <div class="truncate text-xs text-gray-500">
                  {{ item.submitterName || '-' }}
                </div>
              </div>
            </div>
            <span class="shrink-0 text-xs text-gray-400">
              {{ fmtDate(item.submittedAt) }}
            </span>
          </div>
        </div>
      </div>
      <Empty
        v-else
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        :description="$t('page.dashboard.workspace.cards.hrTodo.empty')"
        class="py-8"
      />
    </Spin>
  </Card>
</template>
