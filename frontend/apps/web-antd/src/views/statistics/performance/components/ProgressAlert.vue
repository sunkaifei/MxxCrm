<script lang="ts" setup>
import { computed } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Tooltip } from 'ant-design-vue';

interface Props {
  /** 时间进度百分比（0-100） */
  timeProgress: number;
  /** 业绩进度百分比（0-100+） */
  performanceProgress: number;
}

const props = withDefaults(defineProps<Props>(), {
  timeProgress: 0,
  performanceProgress: 0,
});

/**
 * 三态判断：
 * - 领先（绿）：业绩 > 时间 × 1.1
 * - 持平（黄）：业绩 ∈ [时间 × 0.9, 时间 × 1.1]
 * - 落后（红）：业绩 < 时间 × 0.9
 */
const status = computed(() => {
  const tp = props.timeProgress;
  const pp = props.performanceProgress;
  if (pp >= tp * 1.1) return 'lead';
  if (pp >= tp * 0.9) return 'even';
  return 'behind';
});

const statusConfig = computed(() => {
  const map = {
    lead: {
      color: '#52c41a',
      bg: '#f6ffed',
      border: '#b7eb8f',
      icon: 'lucide:trending-up',
      text: '领先',
      desc: '业绩进度领先于时间进度，请继续保持',
    },
    even: {
      color: '#faad14',
      bg: '#fffbe6',
      border: '#ffe58f',
      icon: 'lucide:minus',
      text: '持平',
      desc: '业绩进度与时间进度持平，需加速',
    },
    behind: {
      color: '#ff4d4f',
      bg: '#fff2f0',
      border: '#ffccc7',
      icon: 'lucide:trending-down',
      text: '落后',
      desc: '业绩进度落后于时间进度，需重点关注',
    },
  } as const;
  return map[status.value];
});

const gap = computed(() =>
  Math.abs(props.performanceProgress - props.timeProgress).toFixed(2),
);
</script>

<template>
  <div
    class="progress-alert"
    :style="{
      background: statusConfig.bg,
      borderColor: statusConfig.border,
      color: statusConfig.color,
    }"
  >
    <div class="flex items-center gap-2 mb-2">
      <IconifyIcon :icon="statusConfig.icon" class="text-base" />
      <span class="font-semibold">{{ statusConfig.text }}</span>
      <span class="text-xs opacity-70">| {{ statusConfig.desc }}</span>
      <span class="ml-auto text-xs opacity-70">差距 {{ gap }}%</span>
    </div>
    <div class="grid grid-cols-2 gap-4">
      <Tooltip :title="`时间进度 ${timeProgress.toFixed(2)}%`">
        <div class="progress-row">
          <div class="text-xs text-gray-500 mb-1">⏰ 时间进度</div>
          <div class="progress-track">
            <div
              class="progress-fill"
              :style="{
                width: `${Math.min(timeProgress, 100)}%`,
                background: 'linear-gradient(90deg, #1890ff 0%, #69c0ff 100%)',
              }"
            />
          </div>
          <div class="text-xs mt-1 text-blue-600 font-medium">
            {{ timeProgress.toFixed(2) }}%
          </div>
        </div>
      </Tooltip>
      <Tooltip :title="`业绩进度 ${performanceProgress.toFixed(2)}%`">
        <div class="progress-row">
          <div class="text-xs text-gray-500 mb-1">🎯 业绩进度</div>
          <div class="progress-track">
            <div
              class="progress-fill"
              :style="{
                width: `${Math.min(performanceProgress, 100)}%`,
                background: `linear-gradient(90deg, ${statusConfig.color} 0%, ${statusConfig.color}aa 100%)`,
              }"
            />
          </div>
          <div
            class="text-xs mt-1 font-medium"
            :style="{ color: statusConfig.color }"
          >
            {{ performanceProgress.toFixed(2) }}%
          </div>
        </div>
      </Tooltip>
    </div>
  </div>
</template>

<style scoped>
.progress-alert {
  padding: 14px 16px;
  border: 1px solid;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.progress-row {
  display: flex;
  flex-direction: column;
}

.progress-track {
  position: relative;
  width: 100%;
  height: 10px;
  background: rgb(255 255 255 / 60%);
  border-radius: 5px;
  overflow: hidden;
  box-shadow: inset 0 1px 2px rgb(0 0 0 / 6%);
}

.progress-fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  border-radius: 5px;
  transition: width 0.6s cubic-bezier(0.16, 1, 0.3, 1);
}
</style>
