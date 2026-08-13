<script lang="ts" setup>
import { computed, nextTick, onMounted, reactive, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import {
  Button,
  Card,
  Drawer,
  Empty,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Modal,
  Select,
  Switch,
  Table,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import {
  getSchedulerJobListApi,
  getSchedulerLogListApi,
  toggleSchedulerJobApi,
  triggerSchedulerJobApi,
  updateSchedulerJobApi,
} from '#/api/core/system/scheduler';
import { $t } from '#/locales';

// ===== 任务列表 =====
// 后端 SchedulerJobVO 已启用 camelCase 重命名
const list = ref<any[]>([]);
const loading = ref(false);
const queryParams = reactive({
  jobCode: '',
  jobName: '',
  enabled: undefined as number | undefined,
  page: 1,
  pageSize: 20,
});
const total = ref(0);

const columns = [
  { title: $t('page.system.scheduler.column.jobCode'), dataIndex: 'jobCode', width: 200 },
  { title: $t('page.system.scheduler.column.jobName'), dataIndex: 'jobName', width: 180 },
  { title: $t('page.system.scheduler.column.cronExpression'), dataIndex: 'cronExpression', width: 160 },
  { title: $t('page.system.scheduler.column.handler'), dataIndex: 'handler', width: 160 },
  { title: $t('page.system.scheduler.column.status'), dataIndex: 'enabled', width: 100 },
  { title: $t('page.system.scheduler.column.lastRun'), dataIndex: 'lastRun', width: 200 },
  { title: $t('page.system.scheduler.column.lastRunResult'), dataIndex: 'lastRunResult', ellipsis: true },
  {
    title: $t('page.system.scheduler.column.action'),
    dataIndex: 'operation',
    width: 220,
    fixed: 'right' as const,
  },
];

async function loadList() {
  loading.value = true;
  try {
    const res: any = await getSchedulerJobListApi(queryParams);
    const data = res?.data || res;
    if (Array.isArray(data)) {
      list.value = data;
      total.value = data.length;
    } else if (data) {
      list.value = data?.items || data?.list || [];
      total.value = data?.total || list.value.length;
    } else {
      list.value = [];
      total.value = 0;
    }
  } catch (e: any) {
    message.error(e?.message || $t('page.system.scheduler.message.loadFailed'));
    list.value = [];
    total.value = 0;
  } finally {
    loading.value = false;
  }
}

function handleReset() {
  queryParams.jobCode = '';
  queryParams.jobName = '';
  queryParams.enabled = undefined;
  queryParams.page = 1;
  loadList();
}

// 启用/禁用
async function handleToggle(record: any, enabled: any) {
  const isEnabled = enabled === true;
  try {
    await toggleSchedulerJobApi({ id: record.id, enabled: isEnabled ? 1 : 0 });
    message.success(isEnabled ? $t('page.system.scheduler.status.enabled') : $t('page.system.scheduler.status.disabled'));
    loadList();
  } catch (e: any) {
    message.error(e?.message || $t('page.system.scheduler.message.operationFailed'));
  }
}

// 手动触发
function handleTrigger(record: any) {
  Modal.confirm({
    title: $t('page.system.scheduler.message.triggerConfirm'),
    content: $t('page.system.scheduler.message.triggerContent', { name: record.jobName }),
    async onOk() {
      try {
        const res: any = await triggerSchedulerJobApi({ id: record.id });
        message.success(res?.data || res || $t('page.system.scheduler.message.triggerSuccess'));
        loadList();
      } catch (e: any) {
        message.error(e?.message || $t('page.system.scheduler.message.triggerFailed'));
      }
    },
  });
}

// ===== 编辑抽屉 =====
const editVisible = ref(false);
const editLoading = ref(false);
const editForm = reactive({
  id: 0,
  cronExpression: '',
  jobName: '',
  description: '',
});

// ===== Cron 可视化选择器 =====
const cronType = ref('everyMonth');
const cronIntervalMinute = ref(5);
const cronIntervalHour = ref(1);
const cronHour = ref(2);
const cronMinute = ref(0);
const cronDayOfMonth = ref(1);
const cronDayOfWeek = ref(1);

const hourOptions = Array.from({ length: 24 }, (_, i) => ({ label: `${i}`, value: i }));
const minuteOptions = Array.from({ length: 60 }, (_, i) => ({ label: `${i}`, value: i }));
const dayOfMonthOptions = Array.from({ length: 31 }, (_, i) => ({ label: `${i + 1}`, value: i + 1 }));
const weekOptions = [
  { label: $t('page.system.scheduler.cronBuilder.weeks.1'), value: 1 },
  { label: $t('page.system.scheduler.cronBuilder.weeks.2'), value: 2 },
  { label: $t('page.system.scheduler.cronBuilder.weeks.3'), value: 3 },
  { label: $t('page.system.scheduler.cronBuilder.weeks.4'), value: 4 },
  { label: $t('page.system.scheduler.cronBuilder.weeks.5'), value: 5 },
  { label: $t('page.system.scheduler.cronBuilder.weeks.6'), value: 6 },
  { label: $t('page.system.scheduler.cronBuilder.weeks.0'), value: 0 },
];

const cronFromVisual = computed(() => {
  switch (cronType.value) {
    case 'everyMinute': return `0 */${cronIntervalMinute.value} * * * *`;
    case 'everyHour': return `0 0 */${cronIntervalHour.value} * * *`;
    case 'everyDay': return `0 ${cronMinute.value} ${cronHour.value} * * *`;
    case 'everyMonth': return `0 ${cronMinute.value} ${cronHour.value} ${cronDayOfMonth.value} * *`;
    case 'everyWeek': return `0 ${cronMinute.value} ${cronHour.value} * * ${cronDayOfWeek.value}`;
    default: return '';
  }
});

const cronTypeOptions = computed(() => [
  { label: $t('page.system.scheduler.cronBuilder.type.everyMonth'), value: 'everyMonth' },
  { label: $t('page.system.scheduler.cronBuilder.type.everyWeek'), value: 'everyWeek' },
  { label: $t('page.system.scheduler.cronBuilder.type.everyDay'), value: 'everyDay' },
  { label: $t('page.system.scheduler.cronBuilder.type.everyHour'), value: 'everyHour' },
  { label: $t('page.system.scheduler.cronBuilder.type.everyMinute'), value: 'everyMinute' },
]);

const cronHumanReadable = computed(() => {
  switch (cronType.value) {
    case 'everyMinute': return `每 ${cronIntervalMinute.value} 分钟执行一次`;
    case 'everyHour': return `每 ${cronIntervalHour.value} 小时执行一次`;
    case 'everyDay': return `每天 ${String(cronHour.value).padStart(2, '0')}:${String(cronMinute.value).padStart(2, '0')} 执行`;
    case 'everyMonth': return `每月 ${cronDayOfMonth.value} 日 ${String(cronHour.value).padStart(2, '0')}:${String(cronMinute.value).padStart(2, '0')} 执行`;
    case 'everyWeek': {
      const w = $t(`page.system.scheduler.cronBuilder.weeks.${cronDayOfWeek.value}`);
      return `每${w} ${String(cronHour.value).padStart(2, '0')}:${String(cronMinute.value).padStart(2, '0')} 执行`;
    }
    default: return '';
  }
});

// 可视化变化时同步到编辑框（openEdit 初始化期间暂停，防止无法识别的表达式被覆盖）
const cronVisualReady = ref(false);
watch(cronFromVisual, (val) => {
  if (cronVisualReady.value) {
    editForm.cronExpression = val;
  }
});

// 从已有 cron 表达式反解析到可视化状态（打开编辑时调用）
function parseCronToVisual(cron: string): boolean {
  const parts = cron.trim().split(/\s+/);
  if (parts.length !== 6) return false;
  const [s, m, h, dom, mon, dow] = parts;

  // 每N分钟: 0 */N * * * *
  if (s === '0' && m.startsWith('*/') && h === '*' && dom === '*' && mon === '*' && dow === '*') {
    cronType.value = 'everyMinute';
    cronIntervalMinute.value = Number.parseInt(m.slice(2), 10) || 5;
    return true;
  }
  // 每N小时: 0 0 */N * * *
  if (s === '0' && m === '0' && h.startsWith('*/') && dom === '*' && mon === '*' && dow === '*') {
    cronType.value = 'everyHour';
    cronIntervalHour.value = Number.parseInt(h.slice(2), 10) || 1;
    return true;
  }
  // 每天: 0 M H * * *
  if (s === '0' && /^\d+$/.test(m) && /^\d+$/.test(h) && dom === '*' && mon === '*' && dow === '*') {
    cronType.value = 'everyDay';
    cronMinute.value = Number(m);
    cronHour.value = Number(h);
    return true;
  }
  // 每月: 0 M H D * *
  if (s === '0' && /^\d+$/.test(m) && /^\d+$/.test(h) && /^\d+$/.test(dom) && mon === '*' && dow === '*') {
    cronType.value = 'everyMonth';
    cronMinute.value = Number(m);
    cronHour.value = Number(h);
    cronDayOfMonth.value = Number(dom);
    return true;
  }
  // 每周: 0 M H * * W
  if (s === '0' && /^\d+$/.test(m) && /^\d+$/.test(h) && dom === '*' && mon === '*' && /^\d+$/.test(dow)) {
    cronType.value = 'everyWeek';
    cronMinute.value = Number(m);
    cronHour.value = Number(h);
    cronDayOfWeek.value = Number(dow);
    return true;
  }
  return false;
}

function openEdit(record: any) {
  editForm.id = record.id;
  editForm.cronExpression = record.cronExpression || '';
  editForm.jobName = record.jobName || '';
  editForm.description = record.description || '';
  // 尝试从已有 cron 解析到可视化状态；无法识别时保留原始表达式
  cronVisualReady.value = false;
  parseCronToVisual(editForm.cronExpression);
  nextTick(() => {
    cronVisualReady.value = true;
  });
  editVisible.value = true;
}

async function handleSaveEdit() {
  if (!editForm.cronExpression.trim()) {
    message.warning($t('page.system.scheduler.message.cronRequired'));
    return;
  }
  editLoading.value = true;
  try {
    await updateSchedulerJobApi({
      id: editForm.id,
      cronExpression: editForm.cronExpression,
      jobName: editForm.jobName,
      description: editForm.description,
    });
    message.success($t('page.system.scheduler.message.saveSuccess'));
    editVisible.value = false;
    loadList();
  } catch (e: any) {
    message.error(e?.message || $t('page.system.scheduler.message.saveFailed'));
  } finally {
    editLoading.value = false;
  }
}

// ===== 日志抽屉 =====
const logVisible = ref(false);
const logList = ref<any[]>([]);
const logLoading = ref(false);
const logJobName = ref('');

const logColumns = [
  { title: $t('page.system.scheduler.column.triggerType'), dataIndex: 'triggerType', width: 100 },
  { title: $t('page.system.scheduler.column.status'), dataIndex: 'status', width: 80 },
  { title: $t('page.system.scheduler.column.elapsedMs'), dataIndex: 'elapsedMs', width: 100 },
  { title: $t('page.system.scheduler.column.resultMessage'), dataIndex: 'resultMessage', ellipsis: true },
  { title: $t('page.system.scheduler.column.errorMessage'), dataIndex: 'errorMessage', ellipsis: true },
  { title: $t('page.system.scheduler.column.operatorName'), dataIndex: 'operatorName', width: 100 },
  { title: $t('page.system.scheduler.column.startTime'), dataIndex: 'startTime', width: 170 },
];

async function loadLogs(jobId: number) {
  logLoading.value = true;
  try {
    const res: any = await getSchedulerLogListApi({
      jobId,
      page: 1,
      pageSize: 50,
    });
    const data = res?.data || res;
    if (Array.isArray(data)) {
      logList.value = data;
    } else {
      logList.value = data?.items || data?.list || [];
    }
  } catch {
    logList.value = [];
  } finally {
    logLoading.value = false;
  }
}

function openLogs(record: any) {
  logJobName.value = record.jobName || '';
  logVisible.value = true;
  loadLogs(record.id);
}

onMounted(() => {
  loadList();
});
</script>

<template>
  <Page auto-content-height>
    <div class="flex flex-col gap-4">
      <!-- 顶部搜索区 -->
      <Card :bordered="false">
        <div class="flex flex-wrap items-center gap-3">
          <Input
            v-model:value="queryParams.jobCode"
            :placeholder="$t('page.system.scheduler.column.jobCode')"
            style="width: 200px"
            allow-clear
            @press-enter="loadList"
          />
          <Input
            v-model:value="queryParams.jobName"
            :placeholder="$t('page.system.scheduler.column.jobName')"
            style="width: 200px"
            allow-clear
            @press-enter="loadList"
          />
          <Button type="primary" @click="loadList">{{ $t('page.system.common.query') }}</Button>
          <Button @click="handleReset">{{ $t('page.system.common.reset') }}</Button>
        </div>
      </Card>

      <!-- 任务列表 -->
      <Card :bordered="false">
      <div class="mb-3 flex items-center justify-between">
        <span class="text-base font-semibold">{{ $t('page.system.scheduler.listTitle') }}</span>
        <Button @click="loadList">
          <template #icon>
            <IconifyIcon icon="lucide:refresh-cw" />
          </template>
          {{ $t('page.system.common.refresh') }}
        </Button>
      </div>
      <Table
        :data-source="list"
        :columns="columns"
        :loading="loading"
        row-key="id"
        :pagination="{
          current: queryParams.page,
          pageSize: queryParams.pageSize,
          total: total,
          showSizeChanger: true,
          showTotal: (t: number) => $t('page.system.common.total', { count: t }),
          onChange: (p: number, ps: number) => {
            queryParams.page = p;
            queryParams.pageSize = ps;
            loadList();
          },
        }"
        size="middle"
        :scroll="{ x: 1200 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'enabled'">
            <Switch
              :checked="record.enabled === 1"
              :checked-children="$t('page.system.common.enabled')"
              :un-checked-children="$t('page.system.common.disabled')"
              @change="(val: any) => handleToggle(record, val)"
            />
          </template>
          <template v-if="column.dataIndex === 'lastRun'">
            <div v-if="record.lastRunTime">
              <div>{{ record.lastRunTime }}</div>
              <Tag
                v-if="record.lastRunStatus === 1"
                color="green"
                style="margin-top: 4px"
              >
                {{ $t('page.system.scheduler.status.success') }}
              </Tag>
              <Tag
                v-else-if="record.lastRunStatus === 2"
                color="processing"
                style="margin-top: 4px"
              >
                {{ $t('page.system.scheduler.status.running') }}
              </Tag>
              <Tag
                v-else-if="record.lastRunStatus === 3"
                color="orange"
                style="margin-top: 4px"
              >
                {{ $t('page.system.scheduler.status.interrupted') }}
              </Tag>
              <Tag
                v-else-if="record.lastRunStatus === 0"
                color="red"
                style="margin-top: 4px"
              >
                {{ $t('page.system.scheduler.status.failed') }}
              </Tag>
              <Tag v-else color="default" style="margin-top: 4px">
                {{ $t('page.system.scheduler.status.notRun') }}
              </Tag>
            </div>
            <span v-else style="color: #999">{{ $t('page.system.scheduler.status.notRun') }}</span>
          </template>
          <template v-if="column.dataIndex === 'lastRunResult'">
            <Tooltip
              v-if="record.lastRunResult"
              :title="record.lastRunResult"
            >
              <span class="truncate inline-block max-w-[240px] align-bottom">
                {{ record.lastRunResult }}
              </span>
            </Tooltip>
            <span v-else style="color: #999">-</span>
          </template>
          <template v-if="column.dataIndex === 'operation'">
            <Button type="link" size="small" @click="openEdit(record)">
              {{ $t('page.system.common.edit') }}
            </Button>
            <Button
              type="link"
              size="small"
              @click="handleTrigger(record)"
            >
              {{ $t('page.system.scheduler.button.trigger') }}
            </Button>
            <Button type="link" size="small" @click="openLogs(record)">
              {{ $t('page.system.scheduler.button.viewLog') }}
            </Button>
          </template>
        </template>
      </Table>
    </Card>

    <!-- 编辑抽屉 -->
    <Drawer
      v-model:open="editVisible"
      :title="$t('page.system.scheduler.editTitle')"
      width="500px"
      :body-style="{ padding: '16px' }"
    >
      <Form layout="vertical" class="py-4" autocomplete="off">
        <FormItem :label="$t('page.system.scheduler.column.jobName')">
          <Input v-model:value="editForm.jobName" name="jobName" />
        </FormItem>
        <FormItem :label="$t('page.system.scheduler.cronBuilder.title')" required>
          <!-- 频率类型 -->
          <Select
            v-model:value="cronType"
            :options="cronTypeOptions"
            style="width: 100%"
          />

          <!-- 参数行（根据频率类型动态显示） -->
          <div class="mt-3 flex flex-wrap items-center gap-2 text-sm">
            <template v-if="cronType === 'everyMinute'">
              {{ $t('page.system.scheduler.cronBuilder.every') }}
              <InputNumber v-model:value="cronIntervalMinute" :min="1" :max="59" size="small" />
              {{ $t('page.system.scheduler.cronBuilder.minutes') }}
            </template>
            <template v-else-if="cronType === 'everyHour'">
              {{ $t('page.system.scheduler.cronBuilder.every') }}
              <InputNumber v-model:value="cronIntervalHour" :min="1" :max="23" size="small" />
              {{ $t('page.system.scheduler.cronBuilder.hours') }}
            </template>
            <template v-else-if="cronType === 'everyDay'">
              <Select v-model:value="cronHour" :options="hourOptions" size="small" style="width: 80px" />
              {{ $t('page.system.scheduler.cronBuilder.hour') }}
              <Select v-model:value="cronMinute" :options="minuteOptions" size="small" style="width: 80px" />
              {{ $t('page.system.scheduler.cronBuilder.minute') }}
            </template>
            <template v-else-if="cronType === 'everyMonth'">
              <Select v-model:value="cronDayOfMonth" :options="dayOfMonthOptions" size="small" style="width: 80px" />
              {{ $t('page.system.scheduler.cronBuilder.day') }}
              <Select v-model:value="cronHour" :options="hourOptions" size="small" style="width: 80px" />
              {{ $t('page.system.scheduler.cronBuilder.hour') }}
              <Select v-model:value="cronMinute" :options="minuteOptions" size="small" style="width: 80px" />
              {{ $t('page.system.scheduler.cronBuilder.minute') }}
            </template>
            <template v-else-if="cronType === 'everyWeek'">
              <Select v-model:value="cronDayOfWeek" :options="weekOptions" size="small" style="width: 90px" />
              <Select v-model:value="cronHour" :options="hourOptions" size="small" style="width: 80px" />
              {{ $t('page.system.scheduler.cronBuilder.hour') }}
              <Select v-model:value="cronMinute" :options="minuteOptions" size="small" style="width: 80px" />
              {{ $t('page.system.scheduler.cronBuilder.minute') }}
            </template>
          </div>

          <!-- 人类可读描述 -->
          <div class="mt-2 text-sm text-green-600">{{ cronHumanReadable }}</div>

          <!-- 手动输入（高级） -->
          <div class="mt-3 border-t border-gray-100 pt-3">
            <div class="mb-1 text-xs text-gray-400">
              {{ $t('page.system.scheduler.cronBuilder.manual') }}
            </div>
            <Input
              v-model:value="editForm.cronExpression"
              :placeholder="$t('page.system.scheduler.cronPlaceholder')"
              name="cronExpression"
            />
            <div class="mt-1 text-xs text-gray-400">
              {{ $t('page.system.scheduler.cronFormatTip') }}
            </div>
          </div>
        </FormItem>
        <FormItem :label="$t('page.system.scheduler.description')">
          <Input.TextArea
            v-model:value="editForm.description"
            :rows="3"
            :placeholder="$t('page.system.scheduler.descriptionPlaceholder')"
          />
        </FormItem>
      </Form>
      <div class="flex justify-end gap-2">
        <Button @click="editVisible = false">{{ $t('page.system.common.cancel') }}</Button>
        <Button type="primary" :loading="editLoading" @click="handleSaveEdit">
          {{ $t('page.system.common.save') }}
        </Button>
      </div>
    </Drawer>

    <!-- 日志抽屉 -->
    <Drawer
      v-model:open="logVisible"
      :title="logJobName ? $t('page.system.scheduler.logTitleWithName', { name: logJobName }) : $t('page.system.scheduler.logTitle')"
      width="900px"
      :body-style="{ padding: '16px' }"
    >
      <Table
        :data-source="logList"
        :columns="logColumns"
        :loading="logLoading"
        row-key="id"
        :pagination="false"
        size="small"
        :scroll="{ x: 800 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'triggerType'">
            <Tag v-if="record.triggerType === 0" color="blue">{{ $t('page.system.scheduler.triggerType.scheduled') }}</Tag>
            <Tag v-else-if="record.triggerType === 1" color="orange">{{ $t('page.system.scheduler.triggerType.manual') }}</Tag>
            <Tag v-else color="purple">{{ $t('page.system.scheduler.triggerType.rerun') }}</Tag>
          </template>
          <template v-if="column.dataIndex === 'status'">
            <Tag v-if="record.status === 1" color="green">{{ $t('page.system.scheduler.status.success') }}</Tag>
            <Tag v-else-if="record.status === 2" color="processing">{{ $t('page.system.scheduler.status.running') }}</Tag>
            <Tag v-else-if="record.status === 3" color="orange">{{ $t('page.system.scheduler.status.interrupted') }}</Tag>
            <Tag v-else color="red">{{ $t('page.system.scheduler.status.failed') }}</Tag>
          </template>
          <template v-if="column.dataIndex === 'elapsedMs'">
            {{ record.elapsedMs != null ? `${record.elapsedMs} ms` : '-' }}
          </template>
        </template>
        <template #emptyText>
          <Empty :description="$t('page.system.scheduler.logEmpty')" />
        </template>
      </Table>
    </Drawer>
    </div>
  </Page>
</template>
