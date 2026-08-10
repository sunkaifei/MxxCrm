<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Card, Spin } from 'ant-design-vue';

import { getTodoSummaryApi } from '#/api';
import { $t } from '#/locales';

defineOptions({
  name: 'TodoOverviewCard',
});

const emit = defineEmits<{
  (e: 'click-card', tabKey: string): void;
}>();

const loading = ref(false);
const summary = ref<any>({});

// 6 个指标方块配置
const cards = computed(() => [
  {
    key: 'overdueFollowUp',
    title: $t('page.dashboard.overdueFollowUp'),
    value: summary.value.overdueFollowUp || 0,
    color: '#ff4d4f',
    bg: '#fff2f0',
    icon: 'lucide:bell-ring',
    tabKey: 'followUp',
  },
  {
    key: 'todayFollowUp',
    title: $t('page.dashboard.todayFollowUp'),
    value: summary.value.todayFollowUp || 0,
    color: '#faad14',
    bg: '#fffbe6',
    icon: 'lucide:calendar-check',
    tabKey: 'followUp',
  },
  {
    key: 'pendingApproval',
    title: $t('page.dashboard.pendingApproval'),
    value: summary.value.pendingApproval || 0,
    color: '#1890ff',
    bg: '#e6f7ff',
    icon: 'lucide:file-check',
    tabKey: 'approval',
  },
  {
    key: 'pendingPayment',
    title: $t('page.dashboard.pendingPayment'),
    value: summary.value.pendingPayment || 0,
    color: '#13c2c2',
    bg: '#e6fffb',
    icon: 'lucide:wallet',
    tabKey: 'payment',
  },
  {
    key: 'expiringContract',
    title: $t('page.dashboard.expiringContract'),
    value: summary.value.expiringContract || 0,
    color: '#52c41a',
    bg: '#f6ffed',
    icon: 'lucide:file-text',
    tabKey: 'contract',
  },
  {
    key: 'stagnantOpportunity',
    title: $t('page.dashboard.stagnantOpportunity'),
    value: summary.value.stagnantOpportunity || 0,
    color: '#eb2f96',
    bg: '#fff0f6',
    icon: 'lucide:alert-triangle',
    tabKey: 'opportunity',
  },
  {
    key: 'pendingPlanApproval',
    title: $t('page.dashboard.pendingPlanApproval'),
    value: summary.value.pendingPlanApproval || 0,
    color: '#722ed1',
    bg: '#f9f0ff',
    icon: 'lucide:clipboard-check',
    tabKey: 'planApproval',
  },
  {
    key: 'unreadCc',
    title: $t('page.dashboard.unreadCc'),
    value: summary.value.unreadCc || 0,
    color: '#fa8c16',
    bg: '#fff7e6',
    icon: 'lucide:mail',
    tabKey: 'cc',
  },
]);

async function loadData() {
  loading.value = true;
  try {
    summary.value = await getTodoSummaryApi();
  } catch {
    // 忽略加载错误，保持空数据
  } finally {
    loading.value = false;
  }
}

function handleClick(tabKey: string) {
  emit('click-card', tabKey);
}

onMounted(() => {
  loadData();
});
</script>

<template>
  <Card class="todo-overview-card">
    <template #title>
      <div class="flex items-center gap-2">
        <IconifyIcon icon="lucide:layout-dashboard" class="size-4" />
        <span>{{ $t('page.dashboard.todoOverview') }}</span>
      </div>
    </template>
    <Spin :spinning="loading">
      <div class="grid grid-cols-2 gap-3 md:grid-cols-4">
        <div
          v-for="card in cards"
          :key="card.key"
          class="metric-block group cursor-pointer rounded-lg p-4 transition-all duration-300 hover:-translate-y-1 hover:shadow-lg"
          :style="{ background: card.bg }"
          @click="handleClick(card.tabKey)"
        >
          <div class="flex items-center justify-between">
            <IconifyIcon
              :icon="card.icon"
              class="size-6 transition-transform duration-300 group-hover:scale-110"
              :style="{ color: card.color }"
            />
          </div>
          <div class="mt-2 text-xs text-gray-600">{{ card.title }}</div>
          <div
            class="mt-1 text-2xl font-bold"
            :style="{ color: card.color }"
          >
            {{ card.value }}
          </div>
        </div>
      </div>
    </Spin>
  </Card>
</template>

<style scoped>
.todo-overview-card :deep(.ant-card-head-title) {
  padding: 12px 0;
}

.metric-block {
  min-height: 96px;
}
</style>
