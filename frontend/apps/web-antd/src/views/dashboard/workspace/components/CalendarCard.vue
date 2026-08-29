<script lang="ts" setup>
// 任务日历卡（注册卡 workspace_calendar）：整月月历 + 有任务日期按类型打点，
// 点击日期在卡片下方显示当天任务明细（跟进提醒/待回款/合同到期，数据源 GET /todo/calendar）
import { computed, onMounted, ref } from 'vue';

import { Card, Empty, Spin, Tag } from 'ant-design-vue';
import { useRouter } from 'vue-router';

import { getTodoCalendarApi } from '#/api';

defineOptions({ name: 'WorkspaceCalendarCard' });

interface CalendarTaskItem {
  amount: null | number | string;
  businessId: number;
  date: string;
  taskType: string;
  title: string;
}

const router = useRouter();

const calendarLoading = ref(false);
const calendarTasks = ref<CalendarTaskItem[]>([]);
const currentYear = ref(new Date().getFullYear());
const currentMonth = ref(new Date().getMonth() + 1);
const selectedDate = ref('');

const monthKey = computed(
  () =>
    `${currentYear.value}-${String(currentMonth.value).padStart(2, '0')}`,
);

const todayStr = (() => {
  const n = new Date();
  return `${n.getFullYear()}-${String(n.getMonth() + 1).padStart(2, '0')}-${String(n.getDate()).padStart(2, '0')}`;
})();

// 当月天数 / 1 号前置偏移（周一起始）
const daysInMonth = computed(
  () => new Date(currentYear.value, currentMonth.value, 0).getDate(),
);
const firstDayOffset = computed(
  () =>
    (new Date(currentYear.value, currentMonth.value - 1, 1).getDay() + 6) % 7,
);

// 按日期分组（打点与点击过滤共用）
const tasksByDate = computed(() => {
  const map = new Map<string, CalendarTaskItem[]>();
  for (const t of calendarTasks.value) {
    const list = map.get(t.date);
    if (list) list.push(t);
    else map.set(t.date, [t]);
  }
  return map;
});

// 月历格子：前置空位 + 日期格
const calendarCells = computed(() => {
  const cells: Array<null | { date: string; day: number }> = [];
  for (let i = 0; i < firstDayOffset.value; i++) cells.push(null);
  for (let d = 1; d <= daysInMonth.value; d++) {
    cells.push({
      date: `${monthKey.value}-${String(d).padStart(2, '0')}`,
      day: d,
    });
  }
  return cells;
});

const monthTaskTotal = computed(() => calendarTasks.value.length);
const selectedTasks = computed(
  () => tasksByDate.value.get(selectedDate.value) || [],
);

const WEEKDAY_KEYS = [
  'page.dashboard.workspace.cards.calendar.mon',
  'page.dashboard.workspace.cards.calendar.tue',
  'page.dashboard.workspace.cards.calendar.wed',
  'page.dashboard.workspace.cards.calendar.thu',
  'page.dashboard.workspace.cards.calendar.fri',
  'page.dashboard.workspace.cards.calendar.sat',
  'page.dashboard.workspace.cards.calendar.sun',
];

// 类型徽标：跟进=蓝 回款=金 合同=绿 商机=紫 发票=红 订单=青
function taskTypeMeta(type: string): { color: string; key: string } {
  switch (type) {
    case 'payment': {
      return { color: 'gold', key: 'payment' };
    }
    case 'contract': {
      return { color: 'green', key: 'contract' };
    }
    case 'opportunity': {
      return { color: 'purple', key: 'opportunity' };
    }
    case 'invoice': {
      return { color: 'red', key: 'invoice' };
    }
    case 'order': {
      return { color: 'cyan', key: 'order' };
    }
    default: {
      return { color: 'blue', key: 'followUp' };
    }
  }
}

// Decimal 字符串金额格式化（千分位 + 两位小数）
function fmtAmount(v: null | number | string | undefined): string {
  const n = Number(v ?? 0);
  if (!Number.isFinite(n)) return '0.00';
  return n.toLocaleString('en-US', {
    maximumFractionDigits: 2,
    minimumFractionDigits: 2,
  });
}

// 按任务类型跳转对应业务页
function goTask(item: CalendarTaskItem) {
  switch (item.taskType) {
    case 'payment': {
      router.push('/sale/payment-plan');
      break;
    }
    case 'contract': {
      router.push('/sale/contract');
      break;
    }
    case 'lead': {
      router.push('/crm/lead');
      break;
    }
    case 'opportunity': {
      router.push('/sale/opportunity');
      break;
    }
    case 'invoice': {
      router.push('/sale/invoice');
      break;
    }
    case 'order': {
      router.push('/sale/order');
      break;
    }
    default: {
      router.push('/crm/customer');
    }
  }
}

function switchMonth(delta: number) {
  let year = currentYear.value;
  let month = currentMonth.value + delta;
  if (month < 1) {
    month = 12;
    year -= 1;
  } else if (month > 12) {
    month = 1;
    year += 1;
  }
  currentYear.value = year;
  currentMonth.value = month;
  loadCalendarTasks();
}

async function loadCalendarTasks() {
  calendarLoading.value = true;
  try {
    const res: any = await getTodoCalendarApi({ month: monthKey.value });
    calendarTasks.value = Array.isArray(res)
      ? res
      : Array.isArray(res?.items)
        ? res.items
        : [];
    // 默认选中：当月优先今天，其次首个有任务日期（无任务置空显示引导）
    if (tasksByDate.value.has(todayStr)) {
      selectedDate.value = todayStr;
    } else {
      const firstTask = calendarTasks.value[0]?.date;
      selectedDate.value = firstTask ? firstTask : '';
    }
  } catch {
    calendarTasks.value = [];
    selectedDate.value = '';
  } finally {
    calendarLoading.value = false;
  }
}

onMounted(() => {
  loadCalendarTasks();
});

defineExpose({ reload: loadCalendarTasks });
</script>

<template>
  <Card
    class="flex h-full flex-col overflow-hidden"
    :body-style="{ flex: '1 1 0', minHeight: 0, overflowY: 'auto' }"
  >
    <template #title>
      <div class="flex items-center gap-2">
        <span
          class="inline-block size-2 rounded-full bg-violet-500"
          aria-hidden="true"
        ></span>
        <span>
          {{ $t('page.dashboard.workspace.cards.calendar.title') }}
        </span>
        <Tag v-if="monthTaskTotal > 0" color="purple" class="ml-1">
          {{ monthTaskTotal }}
        </Tag>
      </div>
    </template>
    <template #extra>
      <div class="flex items-center gap-1">
        <a class="px-1 text-xs" @click="switchMonth(-1)">
          <span class="text-base leading-none">‹</span>
        </a>
        <span class="min-w-[72px] text-center text-xs font-medium">
          {{ monthKey }}
        </span>
        <a class="px-1 text-xs" @click="switchMonth(1)">
          <span class="text-base leading-none">›</span>
        </a>
      </div>
    </template>
    <Spin :spinning="calendarLoading">
      <!-- 月历网格 -->
      <div class="grid grid-cols-7 gap-y-0.5 text-center">
        <div
          v-for="w in WEEKDAY_KEYS"
          :key="w"
          class="py-1 text-xs text-gray-400"
        >
          {{ $t(w) }}
        </div>
        <template v-for="(cell, idx) in calendarCells" :key="idx">
          <div v-if="!cell"></div>
          <div
            v-else
            class="cal-cell mx-0.5 cursor-pointer rounded-md py-0.5 transition-colors hover:bg-violet-50 dark:hover:bg-violet-950/40"
            :class="{
              'cal-cell-selected bg-violet-500 text-white hover:bg-violet-500 dark:bg-violet-600':
                cell.date === selectedDate,
              'cal-cell-today ring-1 ring-violet-400':
                cell.date === todayStr && cell.date !== selectedDate,
            }"
            @click="selectedDate = cell.date"
          >
            <div class="text-xs leading-5">{{ cell.day }}</div>
            <div class="flex h-1.5 items-center justify-center gap-0.5">
              <template
                v-if="(tasksByDate.get(cell.date) || []).length > 0"
              >
                <span
                  v-for="(t, ti) in (tasksByDate.get(cell.date) || []).slice(
                    0,
                    3,
                  )"
                  :key="ti"
                  class="inline-block size-1.5 rounded-full"
                  :class="{
                    'bg-violet-200': cell.date === selectedDate,
                    'bg-blue-400':
                      cell.date !== selectedDate && t.taskType === 'followUp',
                    'bg-amber-400':
                      cell.date !== selectedDate && t.taskType === 'payment',
                    'bg-green-500':
                      cell.date !== selectedDate && t.taskType === 'contract',
                    'bg-purple-500':
                      cell.date !== selectedDate &&
                      t.taskType === 'opportunity',
                    'bg-red-400':
                      cell.date !== selectedDate && t.taskType === 'invoice',
                    'bg-cyan-500':
                      cell.date !== selectedDate && t.taskType === 'order',
                  }"
                ></span>
              </template>
            </div>
          </div>
        </template>
      </div>

      <!-- 当日任务明细 -->
      <div class="mt-3 border-t border-gray-100 pt-2 dark:border-gray-800">
        <div v-if="selectedDate" class="mb-1.5 text-xs font-medium text-gray-600">
          {{
            $t('page.dashboard.workspace.cards.calendar.dayTitle', {
              date: selectedDate.slice(5),
              n: selectedTasks.length,
            })
          }}
        </div>
        <div
          v-if="selectedDate && selectedTasks.length > 0"
          class="flex flex-col gap-1.5"
        >
          <div
            v-for="(item, idx) in selectedTasks"
            :key="idx"
            class="flex cursor-pointer items-center justify-between gap-2 rounded-md border border-gray-100 px-2 py-1.5 transition-colors hover:border-violet-300 hover:bg-violet-50/60 dark:border-gray-800 dark:hover:border-violet-700 dark:hover:bg-violet-950/40"
            @click="goTask(item)"
          >
            <div class="min-w-0 flex-1">
              <div class="truncate text-xs font-medium">
                {{ item.title }}
              </div>
            </div>
            <div class="flex shrink-0 items-center gap-2">
              <span
                v-if="item.amount != null && Number(item.amount) > 0"
                class="text-xs text-gray-600"
              >
                ¥ {{ fmtAmount(item.amount) }}
              </span>
              <Tag :color="taskTypeMeta(item.taskType).color" class="mr-0">
                {{ $t(`page.dashboard.workspace.cards.calendar.${taskTypeMeta(item.taskType).key}`) }}
              </Tag>
            </div>
          </div>
        </div>
        <Empty
          v-else-if="selectedDate"
          :image="Empty.PRESENTED_IMAGE_SIMPLE"
          :description="$t('page.dashboard.workspace.cards.calendar.emptyDay')"
          class="py-4"
        />
        <Empty
          v-else
          :image="Empty.PRESENTED_IMAGE_SIMPLE"
          :description="$t('page.dashboard.workspace.cards.calendar.pickTip')"
          class="py-4"
        />
      </div>
    </Spin>
  </Card>
</template>

<style scoped>
.cal-cell-selected {
  color: #fff;
}
.cal-cell-selected .text-xs {
  color: inherit;
}
</style>
