<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

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
  Popconfirm,
  Switch,
  Table,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { useAccessStore } from '@vben/stores';

import {
  deleteBackupApi,
  downloadBackupApi,
  getBackupConfigApi,
  getBackupListApi,
  restoreBackupApi,
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

// ===== 下载 =====
async function handleDownload(record: any) {
  try {
    const res: any = await downloadBackupApi(record.id);
    const blob = res instanceof Blob ? res : new Blob([res]);
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = record.fileName || `backup_${record.id}.dump`;
    document.body.append(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
  } catch {
    message.error($t('page.system.backup.message.downloadFailed'));
  }
}

// ===== 删除 =====
async function handleDelete(record: any) {
  try {
    const res: any = await deleteBackupApi(record.id);
    message.success(res?.data || res || $t('page.system.backup.message.deleteDone'));
    loadList();
  } catch (error: any) {
    message.error(error?.message || $t('page.system.backup.message.deleteFailed'));
  }
}

// ===== 数据恢复（危险操作：输入 RESTORE 确认） =====
const restoreVisible = ref(false);
const restoreLoading = ref(false);
const restoreConfirmText = ref('');
const restoreTarget = ref<any>(null);

function openRestore(record: any) {
  restoreTarget.value = record;
  restoreConfirmText.value = '';
  restoreVisible.value = true;
}

async function handleRestore() {
  if (!restoreTarget.value) return;
  restoreLoading.value = true;
  try {
    const res: any = await restoreBackupApi(
      restoreTarget.value.id,
      restoreConfirmText.value,
    );
    restoreVisible.value = false;
    Modal.success({
      title: $t('page.system.backup.message.restoreDone'),
      content: res?.data || res || '',
    });
    loadList();
  } catch (error: any) {
    message.error(error?.message || $t('page.system.backup.message.restoreFailed'));
  } finally {
    restoreLoading.value = false;
  }
}

onMounted(() => {
  loadConfig();
  loadList();
});
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
              :disabled="!hasAccess('system:backup:update')"
              @change="(val: any) => (config.enabled = val ? 1 : 0)"
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
                @click="handleDownload(record)"
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
                @click="openRestore(record)"
              >
                {{ $t('page.system.backup.button.restore') }}
              </Button>
              <Popconfirm
                v-if="hasAccess('system:backup:delete')"
                :title="$t('page.system.backup.message.deleteConfirm')"
                @confirm="handleDelete(record)"
              >
                <Button type="link" size="small" danger>
                  {{ $t('page.system.backup.button.delete') }}
                </Button>
              </Popconfirm>
            </template>
          </template>
        </Table>
      </Card>

      <!-- 恢复确认弹窗 -->
      <Modal
        v-model:open="restoreVisible"
        :title="$t('page.system.backup.restoreTitle')"
        :confirm-loading="restoreLoading"
        :ok-button-props="{ danger: true }"
        :ok-text="$t('page.system.backup.button.restoreConfirm')"
        :cancel-text="$t('page.system.common.cancel')"
        @ok="handleRestore"
      >
        <Alert
          type="error"
          show-icon
          class="mb-4"
          :message="$t('page.system.backup.restoreModalWarning')"
        />
        <p class="mb-2">
          {{ $t('page.system.backup.restoreTarget') }}:
          {{ restoreTarget?.fileName }}
        </p>
        <p class="mb-2 text-sm text-gray-500">
          {{ $t('page.system.backup.restoreTip') }}
        </p>
        <Input
          v-model:value="restoreConfirmText"
          placeholder="RESTORE"
          @press-enter="handleRestore"
        />
      </Modal>
    </div>
  </Page>
</template>
