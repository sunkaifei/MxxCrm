<script lang="ts" setup>
import type { Dayjs } from 'dayjs';

import { computed, ref } from 'vue';

import { RangePicker, Segmented } from 'ant-design-vue';
import dayjs from 'dayjs';

import { $t } from '#/locales';

/**
 * 统计页共用时间筛选组件。
 * 参考主流 CRM 惯例：默认展示"本月"，支持 本月/上月/本季度/上季度/今年/全部 快捷切换，
 * 以及自定义日期范围。变更时通过 change 事件抛出查询参数。
 */

type ShortcutKey =
  | 'all'
  | 'custom'
  | 'lastMonth'
  | 'lastQuarter'
  | 'month'
  | 'quarter'
  | 'year';

const emit = defineEmits<{
  change: [params: { end_date?: string; start_date?: string; year?: number }];
}>();

const shortcut = ref<ShortcutKey>('month');
const customRange = ref<[Dayjs, Dayjs] | null>(null);

/** 季度起止（q: 1-4） */
function quarterRange(year: number, q: number): [Dayjs, Dayjs] {
  const startMonth = (q - 1) * 3;
  const start = dayjs(`${year}-${String(startMonth + 1).padStart(2, '0')}-01`);
  const end = start.add(3, 'month').subtract(1, 'day');
  return [start, end];
}

const options = computed(() => [
  { label: $t('page.statistics.timeFilter.month'), value: 'month' },
  { label: $t('page.statistics.timeFilter.lastMonth'), value: 'lastMonth' },
  { label: $t('page.statistics.timeFilter.quarter'), value: 'quarter' },
  { label: $t('page.statistics.timeFilter.lastQuarter'), value: 'lastQuarter' },
  { label: $t('page.statistics.timeFilter.year'), value: 'year' },
  { label: $t('page.statistics.timeFilter.all'), value: 'all' },
  { label: $t('page.statistics.timeFilter.custom'), value: 'custom' },
]);

/** 根据快捷选项计算起止日期 */
function resolveRange(): [Dayjs | null, Dayjs | null] {
  const now = dayjs();
  switch (shortcut.value) {
    case 'custom': {
      return customRange.value
        ? [customRange.value[0], customRange.value[1]]
        : [null, null];
    }
    case 'lastMonth': {
      const lm = now.subtract(1, 'month');
      return [lm.startOf('month'), lm.endOf('month')];
    }
    case 'lastQuarter': {
      const curQ = Math.floor(now.month() / 3) + 1;
      return curQ === 1
        ? quarterRange(now.year() - 1, 4)
        : quarterRange(now.year(), curQ - 1);
    }
    case 'month': {
      return [now.startOf('month'), now.endOf('month')];
    }
    case 'quarter': {
      const q = Math.floor(now.month() / 3) + 1;
      return quarterRange(now.year(), q);
    }
    case 'year': {
      return [now.startOf('year'), now.endOf('year')];
    }
    default: {
      return [null, null];
    }
  }
}

function emitChange() {
  const [start, end] = resolveRange();
  if (shortcut.value === 'custom' && (!start || !end)) {
    // 自定义未选完整时不触发查询
    return;
  }
  if (start && end) {
    emit('change', {
      start_date: start.format('YYYY-MM-DD'),
      end_date: end.format('YYYY-MM-DD'),
      year: start.year(),
    });
  } else {
    emit('change', {});
  }
}

function handleShortcutChange(value: any) {
  shortcut.value = value as ShortcutKey;
  if (value !== 'custom') {
    emitChange();
  }
}

function handleCustomChange(dates: any) {
  customRange.value = dates ?? null;
  if (dates && dates[0] && dates[1]) {
    emitChange();
  }
}

// 初始触发一次（默认本月）
emitChange();
</script>

<template>
  <div class="mb-4 flex flex-wrap items-center gap-3">
    <Segmented
      :options="options"
      :value="shortcut"
      size="small"
      @change="handleShortcutChange"
    />
    <RangePicker
      v-if="shortcut === 'custom'"
      :allow-clear="false"
      :value="customRange as any"
      @change="handleCustomChange"
    />
  </div>
</template>
