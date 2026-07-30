<script lang="ts" setup>
import { onMounted, ref, watch } from 'vue';

import { formatDateTime } from '@vben/utils';

import { Card, Empty, Spin, Tag } from 'ant-design-vue';

import { getTodayWorkLogApi } from '#/api';
import { $t } from '#/locales';

defineOptions({
  name: 'WorkLogCard',
});

const props = defineProps<{
  /** 刷新触发 key，变化时重新加载 */
  refreshKey?: number;
}>();

const loading = ref(false);
const logs = ref<any[]>([]);

// 结果标签颜色映射（成功=绿色 / 驳回=橙色 / 其它=蓝色）
function getResultTag(result: any): { color: string; text: string } {
  const text = String(result ?? '').toUpperCase();
  if (
    text.includes('SUCCESS') ||
    text.includes('OK') ||
    text.includes('通过') ||
    text.includes('成功') ||
    text.includes('已处理') ||
    text.includes('已记录')
  ) {
    return { color: 'success', text: result || $t('page.dashboard.successTag') };
  }
  if (
    text.includes('REJECT') ||
    text.includes('FAIL') ||
    text.includes('驳回') ||
    text.includes('失败')
  ) {
    return { color: 'warning', text: result || $t('page.dashboard.rejectTag') };
  }
  return { color: 'processing', text: result || '--' };
}

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getTodayWorkLogApi();
    // 兼容数组或 { items: [] } 两种返回格式
    if (Array.isArray(res)) {
      logs.value = res;
    } else if (Array.isArray(res?.items)) {
      logs.value = res.items;
    } else {
      logs.value = [];
    }
  } catch {
    logs.value = [];
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.refreshKey,
  () => {
    loadData();
  },
);

onMounted(() => {
  loadData();
});
</script>

<template>
  <Card class="work-log-card">
    <template #title>
      <div class="flex items-center gap-2">
        <span
          class="inline-block size-2 rounded-full bg-green-500"
          aria-hidden="true"
        ></span>
        <span>{{ $t('page.dashboard.workLog') }}</span>
      </div>
    </template>
    <Spin :spinning="loading">
      <div v-if="logs.length > 0" class="work-log-list">
        <div
          v-for="(log, idx) in logs"
          :key="log.id ?? idx"
          class="work-log-item flex items-start gap-3 py-2"
        >
          <div class="flex flex-col items-center pt-1">
            <span
              class="inline-block size-2 rounded-full bg-green-500"
              aria-hidden="true"
            ></span>
          </div>
          <div class="min-w-0 flex-1">
            <div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
              <span class="font-mono text-xs text-gray-500">
                {{ formatDateTime(log.createTime || log.create_time) }}
              </span>
              <span class="text-gray-700">
                {{ log.actionName || log.action || '--' }}
              </span>
              <span class="text-gray-400">·</span>
              <span class="truncate text-gray-800">
                {{ log.businessTitle || log.title || '--' }}
              </span>
            </div>
          </div>
          <Tag
            v-if="log.result || log.status"
            :color="getResultTag(log.result || log.status).color"
            class="ml-2 shrink-0"
          >
            {{ getResultTag(log.result || log.status).text }}
          </Tag>
        </div>
      </div>
      <Empty
        v-else
        :description="$t('page.dashboard.noWorkLog')"
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        class="py-8"
      />
    </Spin>
  </Card>
</template>

<style scoped>
.work-log-card :deep(.ant-card-head-title) {
  padding: 12px 0;
}

.work-log-list {
  max-height: 320px;
  overflow-y: auto;
}

.work-log-item + .work-log-item {
  border-top: 1px dashed #f0f0f0;
}
</style>
