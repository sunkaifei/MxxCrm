<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

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
  message,
  Modal,
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

function openEdit(record: any) {
  editForm.id = record.id;
  editForm.cronExpression = record.cronExpression || '';
  editForm.jobName = record.jobName || '';
  editForm.description = record.description || '';
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
    <!-- 顶部搜索区 -->
    <Card :bordered="false" class="mb-4">
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
        <FormItem :label="$t('page.system.scheduler.column.cronExpression')" required>
          <Input
            v-model:value="editForm.cronExpression"
            :placeholder="$t('page.system.scheduler.cronPlaceholder')"
            name="cronExpression"
          />
          <div class="mt-1 text-xs text-gray-400">
            {{ $t('page.system.scheduler.cronFormatTip') }}
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
            <Tag v-else color="orange">{{ $t('page.system.scheduler.triggerType.manual') }}</Tag>
          </template>
          <template v-if="column.dataIndex === 'status'">
            <Tag v-if="record.status === 1" color="green">{{ $t('page.system.scheduler.status.success') }}</Tag>
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
  </Page>
</template>
