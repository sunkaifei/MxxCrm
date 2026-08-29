<script lang="ts" setup>
// 本周工作负载卡（二期 M1 抽取：原 index.vue 内联实现独立成卡，注册卡 workspace_week_load）
import { computed, onMounted, ref } from 'vue';

import { Card, Empty, Spin } from 'ant-design-vue';

import { getWeekWorkloadApi } from '#/api';

defineOptions({ name: 'WorkspaceWeekLoadCard' });

const weekLoading = ref(false);
const weekWorkload = ref<Array<{ count: number; day: string }>>([]);

const weekMaxCount = computed(() => {
  return Math.max(1, ...weekWorkload.value.map((w) => w.count || 0));
});

async function loadWeekWorkload() {
  weekLoading.value = true;
  try {
    const res: any = await getWeekWorkloadApi();
    if (Array.isArray(res)) {
      weekWorkload.value = res;
    } else if (Array.isArray(res?.items)) {
      weekWorkload.value = res.items;
    } else {
      weekWorkload.value = [];
    }
  } catch {
    weekWorkload.value = [];
  } finally {
    weekLoading.value = false;
  }
}

onMounted(() => {
  loadWeekWorkload();
});

defineExpose({ reload: loadWeekWorkload });
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
        <span>{{ $t('page.dashboard.weekWorkload') }}</span>
      </div>
    </template>
    <Spin :spinning="weekLoading">
      <div
        v-if="weekWorkload.length > 0"
        class="flex h-40 items-end justify-between gap-2 px-2"
      >
        <div
          v-for="(w, idx) in weekWorkload"
          :key="idx"
          class="flex flex-1 flex-col items-center gap-1"
        >
          <div class="text-xs text-gray-500">{{ w.count || 0 }}</div>
          <div
            class="w-full rounded-t transition-all duration-300"
            :style="{
              height: `${Math.max(4, ((w.count || 0) / weekMaxCount) * 110)}px`,
              background: 'linear-gradient(180deg, #1890ff, #69c0ff)',
            }"
          ></div>
          <div class="text-xs text-gray-600">{{ w.day }}</div>
        </div>
      </div>
      <Empty
        v-else
        :image="Empty.PRESENTED_IMAGE_SIMPLE"
        description="暂无数据"
        class="py-8"
      />
    </Spin>
  </Card>
</template>
