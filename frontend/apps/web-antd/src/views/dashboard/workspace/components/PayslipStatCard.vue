<script lang="ts" setup>
// 工资条概况卡（方案 6 节三期岗位卡：聚合接口 payslipStat 分区，
// 后端按当月工资条统计：total 总数 / sent 已发送 / read 已读 / unread 已发送未读）
import { onMounted, ref } from 'vue';

import { Card, Empty, Spin } from 'ant-design-vue';
import { useRouter } from 'vue-router';

import { getWorkspaceSummaryShared } from '#/api/core/system/workspace-summary';
import type { PayslipStatSummary } from '#/api/core/system/workspace-summary';

import { useDashboardPermission } from '../config';

defineOptions({ name: 'WorkspacePayslipStatCard' });

const router = useRouter();
const { canShow } = useDashboardPermission();

const loading = ref(false);
const summary = ref<PayslipStatSummary>({
  readCount: 0,
  sentCount: 0,
  totalCount: 0,
  unreadCount: 0,
});

function goList() {
  router.push('/finance/payslip');
}

async function loadSummary() {
  // 模块权限过滤：无权限不请求，直接空态兜底
  if (!canShow('payslipStat')) {
    summary.value = {
      readCount: 0,
      sentCount: 0,
      totalCount: 0,
      unreadCount: 0,
    };
    return;
  }
  loading.value = true;
  try {
    const res = await getWorkspaceSummaryShared();
    summary.value = res?.payslipStat || {
      readCount: 0,
      sentCount: 0,
      totalCount: 0,
      unreadCount: 0,
    };
  } catch {
    summary.value = {
      readCount: 0,
      sentCount: 0,
      totalCount: 0,
      unreadCount: 0,
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
          class="inline-block size-2 rounded-full bg-green-500"
          aria-hidden="true"
        ></span>
        <span>{{ $t('page.dashboard.workspace.cards.payslipStat.title') }}</span>
      </div>
    </template>
    <template #extra>
      <a class="text-xs" @click="goList">
        {{ $t('page.dashboard.workspace.cards.more') }}
      </a>
    </template>
    <Spin :spinning="loading">
      <div
        v-if="summary.totalCount > 0"
        class="grid grid-cols-2 gap-2"
      >
        <div
          class="cursor-pointer rounded-md bg-gray-50 px-3 py-2 transition-colors hover:bg-gray-100 dark:bg-gray-800 dark:hover:bg-gray-700"
          @click="goList"
        >
          <div class="text-xl font-semibold text-gray-800 dark:text-gray-100">
            {{ summary.totalCount }}
          </div>
          <div class="text-xs text-gray-500">
            {{ $t('page.dashboard.workspace.cards.payslipStat.monthTotal') }}
          </div>
        </div>
        <div
          class="cursor-pointer rounded-md bg-blue-50 px-3 py-2 transition-colors hover:bg-blue-100 dark:bg-blue-950 dark:hover:bg-blue-900"
          @click="goList"
        >
          <div class="text-xl font-semibold text-blue-600">
            {{ summary.sentCount }}
          </div>
          <div class="text-xs text-gray-500">
            {{ $t('page.dashboard.workspace.cards.payslipStat.sent') }}
          </div>
        </div>
        <div
          class="cursor-pointer rounded-md bg-green-50 px-3 py-2 transition-colors hover:bg-green-100 dark:bg-green-950 dark:hover:bg-green-900"
          @click="goList"
        >
          <div class="text-xl font-semibold text-green-600">
            {{ summary.readCount }}
          </div>
          <div class="text-xs text-gray-500">
            {{ $t('page.dashboard.workspace.cards.payslipStat.read') }}
          </div>
        </div>
        <div
          class="cursor-pointer rounded-md bg-orange-50 px-3 py-2 transition-colors hover:bg-orange-100 dark:bg-orange-950 dark:hover:bg-orange-900"
          @click="goList"
        >
          <div class="text-xl font-semibold text-orange-500">
            {{ summary.unreadCount }}
          </div>
          <div class="text-xs text-gray-500">
            {{ $t('page.dashboard.workspace.cards.payslipStat.unread') }}
          </div>
        </div>
      </div>
      <Empty
        v-else
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        :description="$t('page.dashboard.workspace.cards.payslipStat.empty')"
        class="py-8"
      />
    </Spin>
  </Card>
</template>
