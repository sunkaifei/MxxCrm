<script lang="ts" setup>
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';

import {
  Alert,
  Button,
  Card,
  Descriptions,
  DescriptionsItem,
  Input,
  InputNumber,
  message,
  Modal,
  Switch,
  Table,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { useAccessStore } from '@vben/stores';

import {
  cleanExecuteApi,
  cleanPreviewApi,
  deleteBackupApi,
  downloadBackupApi,
  getBackupConfigApi,
  getBackupListApi,
  restoreBackupApi,
  sendBackupOtpApi,
  triggerBackupApi,
  updateBackupConfigApi,
} from '#/api/core/system/backup';
import { $t } from '#/locales';

const accessStore = useAccessStore();
const hasAccess = (code: string) => accessStore.hasAccessCode(code);

// ===== 备份设置 =====
const configLoading = ref(false);
const configSaving = ref(false);
const config = reactive({
  jobId: 0,
  cronExpression: '',
  enabled: 1,
  keepDays: 14,
  outputDir: '',
  pgDumpPath: '',
  lastBackupTime: '',
  lastBackupStatus: undefined as number | undefined,
});

async function loadConfig() {
  configLoading.value = true;
  try {
    const res: any = await getBackupConfigApi();
    const d = res?.data || res;
    if (d) {
      config.jobId = d.jobId ?? 0;
      config.cronExpression = d.cronExpression ?? '';
      config.enabled = d.enabled ?? 1;
      config.keepDays = d.keepDays ?? 14;
      config.outputDir = d.outputDir ?? '';
      config.pgDumpPath = d.pgDumpPath ?? '';
      config.lastBackupTime = d.lastBackupTime ?? '';
      config.lastBackupStatus = d.lastBackupStatus;
    }
  } catch (error: any) {
    message.error(error?.message || $t('page.system.backup.message.loadFailed'));
  } finally {
    configLoading.value = false;
  }
}

async function handleSaveConfig() {
  configSaving.value = true;
  try {
    await updateBackupConfigApi({
      keepDays: config.keepDays,
      cronExpression: config.cronExpression,
      enabled: config.enabled,
    });
    message.success($t('page.system.backup.message.saveSuccess'));
    loadConfig();
  } catch (error: any) {
    message.error(error?.message || $t('page.system.backup.message.saveFailed'));
  } finally {
    configSaving.value = false;
  }
}

// 启用/停用定时备份：点击开关立即保存并动态重载调度任务（只提交 enabled，不影响正在编辑的 cron/keepDays）
async function handleToggleEnabled(val: any) {
  config.enabled = val ? 1 : 0;
  configSaving.value = true;
  try {
    await updateBackupConfigApi({ enabled: config.enabled });
    message.success($t('page.system.backup.message.saveSuccess'));
    loadConfig();
  } catch (error: any) {
    message.error(error?.message || $t('page.system.backup.message.saveFailed'));
    // 保存失败回滚开关状态
    loadConfig();
  } finally {
    configSaving.value = false;
  }
}

// ===== 立即备份 =====
const triggering = ref(false);
async function handleTrigger() {
  triggering.value = true;
  try {
    const res: any = await triggerBackupApi();
    message.success(res?.data || res || $t('page.system.backup.message.backupDone'));
    loadList();
    loadConfig();
  } catch (error: any) {
    message.error(error?.message || $t('page.system.backup.message.backupFailed'));
  } finally {
    triggering.value = false;
  }
}

// ===== 备份记录列表 =====
const list = ref<any[]>([]);
const loading = ref(false);
const queryParams = reactive({ page: 1, pageSize: 20 });
const total = ref(0);

const columns = [
  { title: $t('page.system.backup.column.fileName'), dataIndex: 'fileName', width: 260 },
  { title: $t('page.system.backup.column.operateType'), dataIndex: 'operateType', width: 90 },
  { title: $t('page.system.backup.column.status'), dataIndex: 'status', width: 90 },
  { title: $t('page.system.backup.column.fileSize'), dataIndex: 'fileSize', width: 110 },
  { title: $t('page.system.backup.column.tableCount'), dataIndex: 'tableCount', width: 90 },
  { title: $t('page.system.backup.column.costMs'), dataIndex: 'costMs', width: 100 },
  { title: $t('page.system.backup.column.createTime'), dataIndex: 'createTime', width: 170 },
  { title: $t('page.system.backup.column.errorMessage'), dataIndex: 'errorMessage', ellipsis: true },
  {
    title: $t('page.system.backup.column.action'),
    dataIndex: 'operation',
    width: 200,
    fixed: 'right' as const,
  },
];

async function loadList() {
  loading.value = true;
  try {
    const res: any = await getBackupListApi(queryParams);
    const d = res?.data || res;
    list.value = Array.isArray(d) ? d : (d?.items ?? []);
    total.value = d?.total ?? list.value.length;
  } catch (error: any) {
    message.error(error?.message || $t('page.system.backup.message.loadFailed'));
    list.value = [];
    total.value = 0;
  } finally {
    loading.value = false;
  }
}

function formatSize(size: any) {
  const n = Number(size ?? 0);
  if (n <= 0) return '-';
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(2)} MB`;
}

// ===== 高危操作（下载/删除/还原）：仅超管 + 登录密码 + 邮箱验证码三重验证 =====
const otpVisible = ref(false);
const otpLoading = ref(false);
const otpSending = ref(false);
const otpPassword = ref('');
const otpCode = ref('');
const otpAction = ref(''); // download / delete / restore
const otpTarget = ref<any>(null);
const otpMaskedEmail = ref('');
const otpCountdown = ref(0);
let otpTimer: ReturnType<typeof setInterval> | null = null;

function clearOtpTimer() {
  if (otpTimer) {
    clearInterval(otpTimer);
    otpTimer = null;
  }
  otpCountdown.value = 0;
}

function openOtp(action: 'download' | 'delete' | 'restore', record: any) {
  otpAction.value = action;
  otpTarget.value = record;
  otpPassword.value = '';
  otpCode.value = '';
  otpMaskedEmail.value = '';
  clearOtpTimer();
  otpVisible.value = true;
}

const otpTitle = computed(() => {
  if (otpAction.value === 'delete') {
    return $t('page.system.backup.deleteTitle');
  }
  if (otpAction.value === 'download') {
    return $t('page.system.backup.downloadTitle');
  }
  return $t('page.system.backup.restoreTitle');
});

const otpWarning = computed(() => {
  if (otpAction.value === 'delete') {
    return $t('page.system.backup.deleteModalWarning');
  }
  if (otpAction.value === 'download') {
    return $t('page.system.backup.downloadModalWarning');
  }
  return $t('page.system.backup.restoreModalWarning');
});

async function handleSendOtp() {
  if (otpCountdown.value > 0) return;
  otpSending.value = true;
  try {
    const res: any = await sendBackupOtpApi(otpAction.value);
    const d = res?.data ?? res;
    otpMaskedEmail.value = typeof d === 'string' ? d : (d?.email ?? '');
    message.success($t('page.system.backup.otpSentTo', { email: otpMaskedEmail.value }));
    otpCountdown.value = 60;
    otpTimer = setInterval(() => {
      otpCountdown.value -= 1;
      if (otpCountdown.value <= 0) {
        clearOtpTimer();
      }
    }, 1000);
  } catch (error: any) {
    message.error(error?.message || $t('page.system.backup.otpSendFailed'));
  } finally {
    otpSending.value = false;
  }
}

async function handleOtpConfirm() {
  if (!otpTarget.value) return;
  if (!otpPassword.value.trim()) {
    message.error($t('page.system.backup.otpPasswordPlaceholder'));
    return;
  }
  if (!otpCode.value.trim()) {
    message.error($t('page.system.backup.otpCodePlaceholder'));
    return;
  }
  otpLoading.value = true;
  const action = otpAction.value;
  const target = otpTarget.value;
  try {
    if (action === 'delete') {
      const res: any = await deleteBackupApi(
        target.id,
        otpPassword.value,
        otpCode.value,
      );
      message.success(res?.data || res || $t('page.system.backup.message.deleteDone'));
      otpVisible.value = false;
      loadList();
    } else if (action === 'restore') {
      const res: any = await restoreBackupApi(
        target.id,
        otpPassword.value,
        otpCode.value,
      );
      otpVisible.value = false;
      Modal.success({
        title: $t('page.system.backup.message.restoreDone'),
        content: res?.data || res || '',
      });
      loadList();
    } else if (action === 'download') {
      const res: any = await downloadBackupApi(
        target.id,
        otpPassword.value,
        otpCode.value,
      );
      const blob = res instanceof Blob ? res : new Blob([res]);
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = target.fileName || `backup_${target.id}.dump`;
      document.body.append(a);
      a.click();
      a.remove();
      URL.revokeObjectURL(url);
      otpVisible.value = false;
    }
  } catch (error: any) {
    const msg =
      error?.message ||
      (action === 'delete'
        ? $t('page.system.backup.message.deleteFailed')
        : action === 'download'
          ? $t('page.system.backup.message.downloadFailed')
          : $t('page.system.backup.message.restoreFailed'));
    message.error(msg);
  } finally {
    otpLoading.value = false;
  }
}

function handleOtpCancel() {
  clearOtpTimer();
  otpVisible.value = false;
}

onUnmounted(() => {
  clearOtpTimer();
});

onMounted(() => {
  loadConfig();
  loadList();
});

// ===== 数据初始化（超管 + 登录密码 + 一次性确认码双重验证） =====
const cleanVisible = ref(false);
const cleanLoading = ref(false);
const cleanExecuting = ref(false);
const cleanPassword = ref('');
const cleanConfirmCode = ref('');
const cleanPreviewData = ref<any>(null);

async function handleCleanPreview() {
  cleanLoading.value = true;
  try {
    const res: any = await cleanPreviewApi();
    cleanPreviewData.value = res?.data || res;
  } catch (error: any) {
    message.error(
      error?.message || $t('page.system.backup.cleanPreviewFailed'),
    );
  } finally {
    cleanLoading.value = false;
  }
}

function openCleanExecute() {
  cleanPassword.value = '';
  cleanConfirmCode.value = '';
  cleanVisible.value = true;
}

async function handleCleanExecute() {
  if (!cleanPassword.value.trim()) {
    message.error($t('page.system.backup.cleanPasswordPlaceholder'));
    return;
  }
  if (!cleanConfirmCode.value.trim()) {
    message.error($t('page.system.backup.cleanCodePlaceholder'));
    return;
  }
  cleanExecuting.value = true;
  try {
    const res: any = await cleanExecuteApi(
      cleanPassword.value,
      cleanConfirmCode.value,
    );
    cleanVisible.value = false;
    cleanPreviewData.value = null;
    cleanPassword.value = '';
    cleanConfirmCode.value = '';
    Modal.success({
      title: $t('page.system.backup.cleanDone'),
      content:
        res?.data?.backupMessage ||
        res?.data ||
        res ||
        '',
    });
    loadList();
    loadConfig();
  } catch (error: any) {
    message.error(error?.message || $t('page.system.backup.cleanFailed'));
  } finally {
    cleanExecuting.value = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <div class="flex flex-col gap-4">
      <!-- 备份设置 -->
      <Card :bordered="false" :loading="configLoading">
        <template #title>{{ $t('page.system.backup.settingTitle') }}</template>
        <div class="flex flex-wrap items-end gap-6">
          <div>
            <div class="mb-1 text-sm text-gray-500">
              {{ $t('page.system.backup.cron') }}
            </div>
            <Input
              v-model:value="config.cronExpression"
              style="width: 200px"
              :disabled="!hasAccess('system:backup:update')"
            />
          </div>
          <div>
            <div class="mb-1 text-sm text-gray-500">
              {{ $t('page.system.backup.keepDays') }}
            </div>
            <InputNumber
              v-model:value="config.keepDays"
              :min="1"
              :max="365"
              :disabled="!hasAccess('system:backup:update')"
            />
          </div>
          <div>
            <div class="mb-1 text-sm text-gray-500">
              {{ $t('page.system.backup.enabled') }}
            </div>
            <Switch
              :checked="config.enabled === 1"
              :checked-children="$t('page.system.common.enabled')"
              :un-checked-children="$t('page.system.common.disabled')"
              :disabled="!hasAccess('system:backup:update') || configSaving"
              @change="handleToggleEnabled"
            />
          </div>
          <div class="flex gap-2">
            <Button
              v-if="hasAccess('system:backup:update')"
              type="primary"
              :loading="configSaving"
              @click="handleSaveConfig"
            >
              {{ $t('page.system.common.save') }}
            </Button>
            <Button
              v-if="hasAccess('system:backup:save')"
              :loading="triggering"
              @click="handleTrigger"
            >
              {{ $t('page.system.backup.button.backupNow') }}
            </Button>
          </div>
        </div>

        <Descriptions
          class="mt-4"
          :column="2"
          size="small"
          :title="$t('page.system.backup.envTitle')"
        >
          <DescriptionsItem :label="$t('page.system.backup.outputDir')">
            {{ config.outputDir }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.system.backup.pgDumpPath')">
            {{ config.pgDumpPath }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.system.backup.lastBackup')">
            <template v-if="config.lastBackupTime">
              {{ config.lastBackupTime }}
              <Tag
                :color="config.lastBackupStatus === 1 ? 'green' : 'red'"
                style="margin-left: 8px"
              >
                {{
                  config.lastBackupStatus === 1
                    ? $t('page.system.backup.status.success')
                    : $t('page.system.backup.status.failed')
                }}
              </Tag>
            </template>
            <span v-else style="color: #999">-</span>
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.system.backup.cronTip')">
            {{ $t('page.system.backup.cronTipContent') }}
          </DescriptionsItem>
        </Descriptions>
      </Card>

      <!-- 恢复警告 -->
      <Alert
        type="warning"
        show-icon
        :message="$t('page.system.backup.restoreWarning')"
      />

      <!-- 数据初始化（仅超管可执行，需登录密码 + 一次性确认码验证） -->
      <Card v-if="hasAccess('system:backup:clean')" :bordered="false">
        <template #title>
          <span class="text-red-600">{{ $t('page.system.backup.cleanTitle') }}</span>
        </template>
        <p class="mb-3 text-sm text-gray-500">
          {{ $t('page.system.backup.cleanDesc') }}
        </p>
        <Alert
          type="error"
          show-icon
          class="mb-4"
          :message="$t('page.system.backup.cleanWarning')"
        />
        <div class="flex items-center gap-3">
          <Button danger :loading="cleanLoading" @click="handleCleanPreview">
            {{ $t('page.system.backup.button.cleanPreview') }}
          </Button>
          <Button
            v-if="cleanPreviewData"
            danger
            type="primary"
            @click="openCleanExecute"
          >
            {{ $t('page.system.backup.button.cleanExecute') }}
          </Button>
        </div>

        <!-- 预览结果 -->
        <template v-if="cleanPreviewData">
          <div class="mt-4 flex flex-wrap items-center gap-x-6 gap-y-1 text-sm">
            <span>
              {{ $t('page.system.backup.cleanTotalTables') }}:
              <b>{{ cleanPreviewData.totalTables }}</b>
            </span>
            <span>
              {{ $t('page.system.backup.cleanTotalRows') }}:
              <b>{{ cleanPreviewData.totalRows }}</b>
            </span>
            <span v-if="cleanPreviewData.confirmCode" class="text-red-600">
              {{ $t('page.system.backup.cleanCodeLabel') }}:
              <b class="text-base tracking-widest">{{ cleanPreviewData.confirmCode }}</b>
            </span>
          </div>
          <p v-if="cleanPreviewData.confirmCode" class="mt-1 text-xs text-gray-400">
            {{ $t('page.system.backup.cleanCodeTip') }}
          </p>
          <div class="mt-3">
            <div class="mb-2 text-sm font-medium">
              {{ $t('page.system.backup.cleanGroupsTitle') }}
            </div>
            <div class="flex flex-wrap gap-3">
              <div
                v-for="g in cleanPreviewData.groups || []"
                :key="g.name"
                class="rounded border border-gray-200 px-3 py-2 text-sm"
              >
                <div class="font-medium">{{ g.name }}</div>
                <div class="text-gray-500">
                  {{ g.tables?.length || 0 }}
                  {{ $t('page.system.backup.cleanTotalTables') }} /
                  {{ g.rows }} {{ $t('page.system.backup.cleanRowsUnit') }}
                </div>
              </div>
            </div>
          </div>
        </template>
      </Card>

      <!-- 备份记录 -->
      <Card :bordered="false">
        <template #title>{{ $t('page.system.backup.listTitle') }}</template>
        <Table
          :data-source="list"
          :columns="columns"
          :loading="loading"
          row-key="id"
          :pagination="{
            current: queryParams.page,
            pageSize: queryParams.pageSize,
            total,
            showSizeChanger: true,
            showTotal: (t: number) =>
              $t('page.system.common.total', { count: t }),
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
            <template v-if="column.dataIndex === 'fileName'">
              <Tooltip v-if="record.fileName" :title="record.fileName">
                <span class="truncate inline-block max-w-[240px] align-bottom">
                  {{ record.fileName }}
                </span>
              </Tooltip>
              <span v-else style="color: #999">-</span>
            </template>
            <template v-if="column.dataIndex === 'operateType'">
              <Tag :color="record.operateType === 1 ? 'orange' : 'blue'">
                {{
                  record.operateType === 1
                    ? $t('page.system.backup.operateType.restore')
                    : $t('page.system.backup.operateType.backup')
                }}
              </Tag>
            </template>
            <template v-if="column.dataIndex === 'status'">
              <Tag :color="record.status === 1 ? 'green' : 'red'">
                {{
                  record.status === 1
                    ? $t('page.system.backup.status.success')
                    : $t('page.system.backup.status.failed')
                }}
              </Tag>
            </template>
            <template v-if="column.dataIndex === 'fileSize'">
              {{ formatSize(record.fileSize) }}
            </template>
            <template v-if="column.dataIndex === 'tableCount'">
              {{ record.tableCount || '-' }}
            </template>
            <template v-if="column.dataIndex === 'costMs'">
              {{ record.costMs != null ? `${record.costMs} ms` : '-' }}
            </template>
            <template v-if="column.dataIndex === 'errorMessage'">
              <Tooltip v-if="record.errorMessage" :title="record.errorMessage">
                <span class="truncate inline-block max-w-[200px] align-bottom text-red-500">
                  {{ record.errorMessage }}
                </span>
              </Tooltip>
              <span v-else style="color: #999">-</span>
            </template>
            <template v-if="column.dataIndex === 'operation'">
              <Button
                v-if="
                  hasAccess('system:backup:export') &&
                  record.operateType === 0 &&
                  record.status === 1
                "
                type="link"
                size="small"
                @click="openOtp('download', record)"
              >
                {{ $t('page.system.backup.button.download') }}
              </Button>
              <Button
                v-if="
                  hasAccess('system:backup:restore') &&
                  record.operateType === 0 &&
                  record.status === 1
                "
                type="link"
                size="small"
                danger
                @click="openOtp('restore', record)"
              >
                {{ $t('page.system.backup.button.restore') }}
              </Button>
              <Button
                v-if="hasAccess('system:backup:delete')"
                type="link"
                size="small"
                danger
                @click="openOtp('delete', record)"
              >
                {{ $t('page.system.backup.button.delete') }}
              </Button>
            </template>
          </template>
        </Table>
      </Card>

      <!-- 高危操作验证弹窗（下载/删除/还原共用：仅超管 + 登录密码 + 邮箱验证码三重验证） -->
      <Modal
        v-model:open="otpVisible"
        :title="otpTitle"
        :confirm-loading="otpLoading"
        :ok-button-props="{ danger: true }"
        :ok-text="$t('page.system.backup.button.confirm')"
        :cancel-text="$t('page.system.common.cancel')"
        @ok="handleOtpConfirm"
        @cancel="handleOtpCancel"
      >
        <Alert type="error" show-icon class="mb-4" :message="otpWarning" />
        <p class="mb-3">
          {{ $t('page.system.backup.restoreTarget') }}:
          {{ otpTarget?.fileName }}
        </p>
        <p v-if="otpAction === 'restore'" class="mb-3 text-sm text-gray-500">
          {{ $t('page.system.backup.restoreTip') }}
        </p>
        <div class="mb-3">
          <div class="mb-1 text-sm font-medium">
            {{ $t('page.system.backup.otpPasswordLabel') }}
          </div>
          <Input.Password
            v-model:value="otpPassword"
            :placeholder="$t('page.system.backup.otpPasswordPlaceholder')"
            @press-enter="handleOtpConfirm"
          />
        </div>
        <div>
          <div class="mb-1 text-sm font-medium">
            {{ $t('page.system.backup.otpCodeLabel') }}
          </div>
          <div class="flex gap-2">
            <Input
              v-model:value="otpCode"
              :placeholder="$t('page.system.backup.otpCodePlaceholder')"
              @press-enter="handleOtpConfirm"
            />
            <Button
              :loading="otpSending"
              :disabled="otpCountdown > 0"
              style="white-space: nowrap"
              @click="handleSendOtp"
            >
              {{
                otpCountdown > 0
                  ? $t('page.system.backup.otpResendCountdown', {
                      s: otpCountdown,
                    })
                  : otpMaskedEmail
                    ? $t('page.system.backup.otpSendAgain')
                    : $t('page.system.backup.otpSend')
              }}
            </Button>
          </div>
          <p v-if="otpMaskedEmail" class="mt-1 text-xs text-gray-400">
            {{ $t('page.system.backup.otpSentTo', { email: otpMaskedEmail }) }}
          </p>
        </div>
      </Modal>

      <!-- 数据初始化确认弹窗（超管 + 登录密码 + 一次性确认码） -->
      <Modal
        v-model:open="cleanVisible"
        :title="$t('page.system.backup.cleanTitle')"
        :confirm-loading="cleanExecuting"
        :ok-button-props="{ danger: true }"
        :ok-text="$t('page.system.backup.button.cleanExecute')"
        :cancel-text="$t('page.system.common.cancel')"
        @ok="handleCleanExecute"
      >
        <Alert
          type="error"
          show-icon
          class="mb-4"
          :message="$t('page.system.backup.cleanModalWarning')"
        />
        <div class="mb-3">
          <div class="mb-1 text-sm font-medium">
            {{ $t('page.system.backup.cleanPasswordLabel') }}
          </div>
          <Input.Password
            v-model:value="cleanPassword"
            :placeholder="$t('page.system.backup.cleanPasswordPlaceholder')"
            @press-enter="handleCleanExecute"
          />
        </div>
        <div>
          <div class="mb-1 text-sm font-medium">
            {{ $t('page.system.backup.cleanCodeLabel') }}
          </div>
          <Input
            v-model:value="cleanConfirmCode"
            :placeholder="$t('page.system.backup.cleanCodePlaceholder')"
            @press-enter="handleCleanExecute"
          />
        </div>
      </Modal>
    </div>
  </Page>
</template>
