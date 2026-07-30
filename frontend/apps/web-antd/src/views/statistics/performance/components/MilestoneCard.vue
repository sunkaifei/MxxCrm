<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Card, Empty, Spin, Tag } from 'ant-design-vue';

import { getPerformanceMilestoneApi } from '#/api';

interface Milestone {
  label: string;
  amount: number;
  achieved: boolean;
  achievedDate?: string | null;
}

interface Props {
  year: number;
  currentAmount?: number;
}

const props = defineProps<Props>();

const loading = ref(false);
const data = ref<any>({});

function formatCurrency(val?: number) {
  if (!val) return '¥0';
  if (val >= 100000000) return `¥${(val / 100000000).toFixed(2)}亿`;
  if (val >= 10000) return `¥${(val / 10000).toFixed(1)}万`;
  return `¥${val.toLocaleString()}`;
}

const milestones = computed<Milestone[]>(() => data.value?.milestones || []);

const currentMilestone = computed(() => data.value?.currentMilestone);
const nextMilestone = computed(() => data.value?.nextMilestone);
const futureMilestone = computed(() => data.value?.futureMilestone);

// 计算距离下一档的进度
const progressPercent = computed(() => {
  if (!futureMilestone.value?.achieved && futureMilestone.value?.remaining !== undefined) {
    const next = nextMilestone.value;
    const future = futureMilestone.value;
    if (!next || !future) return 0;
    const range = future.amount - next.amount;
    if (range <= 0) return 0;
    const achieved = (props.currentAmount || 0) - next.amount;
    return Math.min(Math.max((achieved / range) * 100, 0), 100);
  }
  return 100;
});

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getPerformanceMilestoneApi({ year: props.year });
    data.value = res?.data || res || {};
  } catch {
    data.value = {};
  } finally {
    loading.value = false;
  }
}

onMounted(() => loadData());
</script>

<template>
  <Card :body-style="{ padding: '16px' }">
    <template #title>
      <div class="flex items-center gap-2">
        <IconifyIcon icon="lucide:trophy" class="text-lg text-yellow-500" />
        <span>业绩里程碑</span>
      </div>
    </template>
    <template #extra>
      <span class="text-xs text-gray-400">达成目标，开启新里程碑</span>
    </template>

    <Spin :spinning="loading">
      <div v-if="milestones.length === 0" class="py-8">
        <Empty description="暂无里程碑数据" />
      </div>

      <div v-else>
        <!-- 当前里程碑大卡片 -->
        <div
          v-if="futureMilestone"
          class="current-milestone-card mb-4"
          :class="{
            'achieved': futureMilestone.achieved,
            'in-progress': !futureMilestone.achieved,
          }"
        >
          <div class="flex items-center justify-between">
            <div>
              <div class="text-xs text-gray-500 mb-1">下一里程碑</div>
              <div class="text-2xl font-bold text-yellow-600">
                🏆 {{ futureMilestone.label }}
              </div>
              <div class="text-xs text-gray-500 mt-1">
                目标金额：{{ formatCurrency(futureMilestone.amount) }}
              </div>
            </div>
            <div class="text-right">
              <div class="text-xs text-gray-500 mb-1">还差</div>
              <div class="text-xl font-bold text-orange-500">
                {{ formatCurrency(futureMilestone.remaining) }}
              </div>
              <div class="text-xs text-gray-500 mt-1">加油！</div>
            </div>
          </div>

          <!-- 进度条 -->
          <div class="mt-3">
            <div class="progress-track">
              <div
                class="progress-fill"
                :style="{ width: `${progressPercent}%` }"
              />
            </div>
            <div class="flex justify-between mt-1 text-xs text-gray-500">
              <span>{{ nextMilestone?.label || '起点' }}</span>
              <span>{{ progressPercent.toFixed(2) }}%</span>
              <span>{{ futureMilestone.label }}</span>
            </div>
          </div>
        </div>

        <!-- 所有里程碑列表 -->
        <div class="milestone-list">
          <div
            v-for="m in milestones"
            :key="m.label"
            class="milestone-item"
            :class="{ 'achieved': m.achieved }"
          >
            <div class="flex items-center gap-3">
              <div class="medal" :class="{ 'achieved': m.achieved }">
                <IconifyIcon
                  :icon="m.achieved ? 'lucide:award' : 'lucide:circle'"
                  class="text-xl"
                />
              </div>
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-semibold">{{ m.label }}</span>
                  <Tag v-if="m.achieved" color="success" style="margin: 0">已达成</Tag>
                  <Tag v-else color="default" style="margin: 0">未达成</Tag>
                </div>
                <div class="text-xs text-gray-500 mt-1">
                  目标：{{ formatCurrency(m.amount) }}
                  <template v-if="m.achieved && m.achievedDate">
                    · 达成时间：{{ m.achievedDate }}
                  </template>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Spin>
  </Card>
</template>

<style scoped>
.current-milestone-card {
  padding: 16px;
  border-radius: 10px;
  background: linear-gradient(135deg, #fffbe6 0%, #fff7e6 100%);
  border: 1px solid #ffe58f;
}

.current-milestone-card.achieved {
  background: linear-gradient(135deg, #f6ffed 0%, #f0fff0 100%);
  border-color: #b7eb8f;
}

.progress-track {
  position: relative;
  width: 100%;
  height: 12px;
  background: rgb(255 255 255 / 60%);
  border-radius: 6px;
  overflow: hidden;
  box-shadow: inset 0 1px 2px rgb(0 0 0 / 6%);
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #faad14 0%, #ffd666 100%);
  border-radius: 6px;
  transition: width 0.6s cubic-bezier(0.16, 1, 0.3, 1);
}

.milestone-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.milestone-item {
  padding: 12px;
  border-radius: 8px;
  background: #fafafa;
  transition: all 0.3s;
}

.milestone-item.achieved {
  background: linear-gradient(90deg, #f6ffed 0%, #fafafa 100%);
}

.milestone-item:hover {
  transform: translateX(2px);
  box-shadow: 0 2px 8px rgb(0 0 0 / 6%);
}

.medal {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: #f0f0f0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #bfbfbf;
}

.medal.achieved {
  background: linear-gradient(135deg, #ffd666 0%, #faad14 100%);
  color: white;
  box-shadow: 0 2px 8px rgb(250 173 20 / 40%);
}
</style>
