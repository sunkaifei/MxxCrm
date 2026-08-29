<script lang="ts" setup>
// 公司公告卡（方案 6 节三期岗位卡：聚合接口 announcement 分区，含未读数；
// 公告卡不设权限码，可见性由第一层角色分配控制；空态为本期验收项 11.4）
import { onMounted, ref } from 'vue';

import { Card, Empty, Spin, Tag } from 'ant-design-vue';
import { useRouter } from 'vue-router';

import { getWorkspaceSummaryShared } from '#/api/core/system/workspace-summary';
import type { AnnouncementSummary } from '#/api/core/system/workspace-summary';

defineOptions({ name: 'WorkspaceAnnouncementCard' });

const router = useRouter();

const loading = ref(false);
const summary = ref<AnnouncementSummary>({ items: [], total: 0, unread: 0 });

function fmtDate(v?: null | string): string {
  return typeof v === 'string' ? v.slice(0, 10) : '';
}

// 公告用户侧去向：用户消息中心（后端菜单提供路由）
function goList() {
  router.push('/company/message');
}

async function loadSummary() {
  loading.value = true;
  try {
    const res = await getWorkspaceSummaryShared();
    summary.value = res?.announcement || { items: [], total: 0, unread: 0 };
  } catch {
    summary.value = { items: [], total: 0, unread: 0 };
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
          class="inline-block size-2 rounded-full bg-amber-500"
          aria-hidden="true"
        ></span>
        <span>{{ $t('page.dashboard.workspace.cards.announcement.title') }}</span>
        <Tag v-if="summary.unread > 0" color="red" class="ml-1">
          {{ $t('page.dashboard.workspace.cards.announcement.unread') }}
          {{ summary.unread }}
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
          :key="item.id ?? idx"
          class="flex cursor-pointer items-center justify-between gap-2 rounded-md border border-gray-100 px-2 py-1.5 transition-colors hover:border-amber-300"
          @click="goList"
        >
          <div class="flex min-w-0 items-center gap-2">
            <span
              v-if="Number(item.isRead) === 0"
              class="inline-block size-1.5 shrink-0 rounded-full bg-red-500"
              aria-hidden="true"
            ></span>
            <div class="min-w-0">
              <div
                class="truncate text-xs"
                :class="
                  Number(item.isRead) === 0
                    ? 'font-medium text-gray-800 dark:text-gray-100'
                    : 'text-gray-600 dark:text-gray-300'
                "
              >
                {{ item.title || '-' }}
              </div>
              <div class="truncate text-xs text-gray-500">
                {{ item.publishName || '-' }}
              </div>
            </div>
          </div>
          <span class="shrink-0 text-xs text-gray-400">
            {{ fmtDate(item.publishTime) }}
          </span>
        </div>
      </div>
      <Empty
        v-else
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        :description="$t('page.dashboard.workspace.cards.announcement.empty')"
        class="py-8"
      />
    </Spin>
  </Card>
</template>
