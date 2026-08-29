<script lang="ts" setup>
// 入职引导配置页（方案 4.1 / 13.2-1 / 13.3-6）：
// 三页签：步骤管理（内置启停排序 + 自定义增删改）、入口预设、区块开关；
// 顶部：刷新 + 预览引导卡（弹窗五态模拟，复用 OnboardingCard previewState）；
// 保存：全量覆盖式 POST /onboarding/save，version 乐观锁，409 由全局拦截器提示后本地重拉。
import { computed, onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import {
  Button,
  Card,
  Empty,
  Input,
  message,
  Modal,
  Popconfirm,
  Select,
  Spin,
  Switch,
  Table,
  Tabs,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { getOnboardingConfigApi, saveOnboardingConfigApi } from '#/api/core/system/onboarding';
import { $t } from '#/locales';

import OnboardingCard from '../../dashboard/workspace/components/OnboardingCard.vue';

const loading = ref(false);
const saving = ref(false);
const version = ref(0);
const activeTab = ref('steps');

// ===== 页签 1：步骤管理（编辑副本，保存时全量提交） =====
const steps = ref<any[]>([]);
const sortedSteps = computed(() =>
  [...steps.value].toSorted((a, b) => (a?.sortOrder ?? 0) - (b?.sortOrder ?? 0)),
);

const stepModal = reactive({ visible: false, editingIndex: -1 });
const stepForm = reactive({ stepDesc: '', stepName: '', linkUrl: '' });

// ===== 页签 2：入口预设（引导卡区块 C 的管理员默认集合） =====
const quickPreset = ref<any[]>([]);
const entryModal = reactive({ visible: false, editingIndex: -1 });
const entryForm = reactive({
  color: '#1890ff',
  icon: 'lucide:link',
  label: '',
  path: '',
});

// ===== 页签 3：区块开关 =====
const configForm = reactive({
  announceEnabled: 1,
  quickEnabled: 1,
  todoEnabled: 1,
});

// ===== 预览弹窗（13.3-6：五态模拟，不依赖真实账号） =====
const previewVisible = ref(false);
const previewState = ref('pending_profile');
const PREVIEW_STATE_OPTIONS = [
  { label: $t('page.dashboard.onboarding.stateNone'), value: 'pending_profile' },
  { label: $t('page.system.onboarding.previewStateReady'), value: 'ready_submit' },
  { label: $t('page.dashboard.onboarding.statePending'), value: 'in_approval' },
  { label: $t('page.dashboard.onboarding.stateRejected'), value: 'rejected' },
  { label: $t('page.dashboard.onboarding.statePassed'), value: 'passed' },
];

// ===== 加载（管理员态：GET /onboarding/config 追加 admin 全量数据） =====
async function load() {
  loading.value = true;
  try {
    const res: any = await getOnboardingConfigApi();
    const admin = res?.admin;
    if (!admin) {
      message.warning($t('page.system.onboarding.noAdminData'));
      return;
    }
    version.value = Number(admin.version ?? 0);
    steps.value = (Array.isArray(admin.steps) ? admin.steps : []).map((s: any) => ({ ...s }));
    const cfg = res?.config || {};
    configForm.announceEnabled = Number(cfg.announceEnabled ?? 1) === 1 ? 1 : 0;
    configForm.todoEnabled = Number(cfg.todoEnabled ?? 1) === 1 ? 1 : 0;
    configForm.quickEnabled = Number(cfg.quickEnabled ?? 1) === 1 ? 1 : 0;
    quickPreset.value = Array.isArray(cfg.quickPreset)
      ? cfg.quickPreset.map((x: any) => ({ ...x }))
      : [];
  } finally {
    loading.value = false;
  }
}

// ===== 13.4-1 站内相对路径校验（前端侧，与后端双重校验对齐） =====
function isValidInternalPath(url: null | string | undefined): boolean {
  if (!url || !url.startsWith('/') || url.startsWith('//')) return false;
  return /^[/a-zA-Z0-9_?=&.-]*$/.test(url);
}

// ===== 步骤管理：上移/下移（按 sortOrder 升序展示，相邻交换后若冲突则全量重排） =====
function moveStep(index: number, dir: -1 | 1) {
  const j = index + dir;
  if (j < 0 || j >= sortedSteps.value.length) return;
  const a = sortedSteps.value[index];
  const b = sortedSteps.value[j];
  if (a.sortOrder === b.sortOrder) {
    sortedSteps.value.forEach((s, i) => (s.sortOrder = (i + 1) * 10));
  } else {
    const tmp = a.sortOrder;
    a.sortOrder = b.sortOrder;
    b.sortOrder = tmp;
  }
}

function openAddStep() {
  stepModal.editingIndex = -1;
  stepForm.stepName = '';
  stepForm.stepDesc = '';
  stepForm.linkUrl = '';
  stepModal.visible = true;
}

function openEditStep(step: any) {
  const idx = steps.value.findIndex((s) => s.stepCode === step.stepCode);
  if (idx < 0) return;
  stepModal.editingIndex = idx;
  stepForm.stepName = step.stepName || '';
  stepForm.stepDesc = step.stepDesc || '';
  stepForm.linkUrl = step.linkUrl || '';
  stepModal.visible = true;
}

function removeStep(step: any) {
  steps.value = steps.value.filter((s) => s.stepCode !== step.stepCode);
}

function submitStepModal() {
  if (!stepForm.stepName.trim()) {
    message.error($t('page.system.onboarding.nameRequired'));
    return;
  }
  const link = stepForm.linkUrl.trim();
  if (link && !isValidInternalPath(link)) {
    message.error($t('page.system.onboarding.pathInvalid'));
    return;
  }
  if (stepModal.editingIndex >= 0) {
    const target = steps.value[stepModal.editingIndex];
    if (target) {
      target.stepName = stepForm.stepName.trim();
      target.stepDesc = stepForm.stepDesc.trim() || null;
      target.linkUrl = link || null;
    }
  } else {
    const maxSort = steps.value.reduce((m, s) => Math.max(m, Number(s.sortOrder ?? 0)), 0);
    steps.value.push({
      stepCode: `custom_${Date.now().toString(36)}`,
      stepName: stepForm.stepName.trim(),
      stepDesc: stepForm.stepDesc.trim() || null,
      stepType: 2,
      linkUrl: link || null,
      sortOrder: maxSort + 10,
      status: 1,
    });
  }
  stepModal.visible = false;
}

// ===== 入口预设管理 =====
function moveEntry(index: number, dir: -1 | 1) {
  const j = index + dir;
  if (j < 0 || j >= quickPreset.value.length) return;
  const list = quickPreset.value;
  [list[index], list[j]] = [list[j], list[index]];
}

function openAddEntry() {
  entryModal.editingIndex = -1;
  entryForm.label = '';
  entryForm.icon = 'lucide:link';
  entryForm.color = '#1890ff';
  entryForm.path = '';
  entryModal.visible = true;
}

function openEditEntry(entry: any) {
  const idx = quickPreset.value.indexOf(entry);
  if (idx < 0) return;
  entryModal.editingIndex = idx;
  entryForm.label = entry.label || '';
  entryForm.icon = entry.icon || 'lucide:link';
  entryForm.color = entry.color || '#1890ff';
  entryForm.path = entry.path || '';
  entryModal.visible = true;
}

function removeEntry(entry: any) {
  quickPreset.value = quickPreset.value.filter((e) => e !== entry);
}

function submitEntryModal() {
  if (!entryForm.label.trim()) {
    message.error($t('page.system.onboarding.labelRequired'));
    return;
  }
  if (!isValidInternalPath(entryForm.path.trim())) {
    message.error($t('page.system.onboarding.pathInvalid'));
    return;
  }
  const item = {
    code: entryModal.editingIndex >= 0
      ? (quickPreset.value[entryModal.editingIndex]?.code || `entry_${Date.now().toString(36)}`)
      : `entry_${Date.now().toString(36)}`,
    label: entryForm.label.trim(),
    icon: entryForm.icon.trim() || 'lucide:link',
    color: entryForm.color.trim() || '#1890ff',
    path: entryForm.path.trim(),
  };
  if (entryModal.editingIndex >= 0) {
    quickPreset.value[entryModal.editingIndex] = item;
  } else {
    quickPreset.value.push(item);
  }
  entryModal.visible = false;
}

// ===== 保存（全量覆盖式 + version 乐观锁；409 由全局拦截器提示，此处重拉） =====
async function save() {
  const codes = new Set<string>();
  for (const s of steps.value) {
    const code = (s.stepCode || '').trim();
    if (!code) {
      message.error($t('page.system.onboarding.codeRequired'));
      return;
    }
    if (codes.has(code)) {
      message.error($t('page.system.onboarding.codeDup'));
      return;
    }
    codes.add(code);
    if (!(s.stepName || '').trim()) {
      message.error($t('page.system.onboarding.nameRequired'));
      return;
    }
    if (s.linkUrl && !isValidInternalPath(s.linkUrl)) {
      message.error($t('page.system.onboarding.pathInvalid'));
      return;
    }
  }
  for (const p of quickPreset.value) {
    if (!(p.label || '').trim()) {
      message.error($t('page.system.onboarding.labelRequired'));
      return;
    }
    if (!isValidInternalPath(p.path)) {
      message.error($t('page.system.onboarding.pathInvalid'));
      return;
    }
  }

  saving.value = true;
  try {
    const body = {
      steps: steps.value.map((s) => ({
        id: s.id || undefined,
        stepCode: (s.stepCode || '').trim(),
        stepName: (s.stepName || '').trim(),
        stepDesc: s.stepDesc || null,
        linkUrl: s.linkUrl || null,
        sortOrder: Number(s.sortOrder ?? 0),
        status: Number(s.status ?? 1),
      })),
      config: {
        announceEnabled: configForm.announceEnabled,
        todoEnabled: configForm.todoEnabled,
        quickEnabled: configForm.quickEnabled,
        quickPreset: quickPreset.value,
        version: version.value,
      },
      version: version.value,
    };
    const nv: any = await saveOnboardingConfigApi(body);
    version.value = Number(nv ?? version.value + 1);
    message.success($t('page.system.onboarding.saveSuccess'));
  } catch {
    await load();
  } finally {
    saving.value = false;
  }
}

const stepColumns = computed(() => [
  { dataIndex: 'sortOrder', title: $t('page.system.onboarding.colSort'), width: 70 },
  { dataIndex: 'stepCode', title: $t('page.system.onboarding.colCode'), width: 160 },
  { dataIndex: 'stepName', title: $t('page.system.onboarding.colName'), width: 160 },
  { dataIndex: 'stepDesc', title: $t('page.system.onboarding.colDesc') },
  { dataIndex: 'linkUrl', title: $t('page.system.onboarding.colLink'), width: 180 },
  { dataIndex: 'stepType', title: $t('page.system.onboarding.colType'), width: 90 },
  { dataIndex: 'status', title: $t('page.system.onboarding.colStatus'), width: 80 },
  { dataIndex: 'actions', title: $t('page.system.onboarding.colActions'), width: 150 },
]);

onMounted(() => {
  load();
});
</script>

<template>
  <Page :title="$t('page.system.onboarding.title')" auto-content-height>
    <Card :body-style="{ padding: '12px 16px' }">
      <div class="flex flex-wrap items-center gap-3">
        <Tag color="blue">
          {{ $t('page.system.onboarding.versionLabel') }}: {{ version }}
        </Tag>
        <span class="text-xs text-gray-400">
          {{ $t('page.system.onboarding.builtinTip') }}
        </span>
        <div class="ml-auto flex items-center gap-2">
          <Button :disabled="loading" @click="load">
            {{ $t('page.system.onboarding.reload') }}
          </Button>
          <Button
            type="primary"
            ghost
            @click="previewVisible = true"
          >
            {{ $t('page.system.onboarding.previewTitle') }}
          </Button>
          <Button type="primary" :loading="saving" @click="save">
            {{ $t('page.system.onboarding.saveAction') }}
          </Button>
        </div>
      </div>
    </Card>

    <Spin :spinning="loading">
      <Card class="mt-3" :body-style="{ padding: '8px 16px 16px' }">
        <Tabs v-model:active-key="activeTab">
          <!-- 页签 1：步骤管理 -->
          <Tabs.TabPane
            key="steps"
            :tab="$t('page.system.onboarding.tabSteps')"
          >
            <div class="mb-3 flex items-center">
              <span class="text-xs text-gray-400">
                {{ $t('page.system.onboarding.builtinTip') }}
              </span>
              <Button
                class="ml-auto"
                size="small"
                type="primary"
                @click="openAddStep"
              >
                {{ $t('page.system.onboarding.addStep') }}
              </Button>
            </div>
            <Table
              :columns="stepColumns"
              :data-source="sortedSteps"
              :pagination="false"
              row-key="stepCode"
              size="small"
            >
              <template #bodyCell="{ column, record, index }">
                <template v-if="column.dataIndex === 'stepName'">
                  {{ record.stepName }}
                </template>
                <template v-else-if="column.dataIndex === 'stepDesc'">
                  <span class="text-xs text-gray-500">{{ record.stepDesc || '-' }}</span>
                </template>
                <template v-else-if="column.dataIndex === 'linkUrl'">
                  <span class="text-xs">{{ record.linkUrl || '-' }}</span>
                </template>
                <template v-else-if="column.dataIndex === 'stepType'">
                  <Tag v-if="record.stepType === 1" color="default">
                    {{ $t('page.system.onboarding.typeBuiltIn') }}
                  </Tag>
                  <Tag v-else color="purple">
                    {{ $t('page.system.onboarding.typeCustom') }}
                  </Tag>
                </template>
                <template v-else-if="column.dataIndex === 'status'">
                  <Switch
                    v-model:checked="record.status"
                    :checked-value="1"
                    :disabled="record.stepType !== 1"
                    :un-checked-value="0"
                    size="small"
                  />
                </template>
                <template v-else-if="column.dataIndex === 'actions'">
                  <div class="flex items-center gap-1">
                    <Tooltip :title="$t('page.system.onboarding.moveUp')">
                      <Button
                        :disabled="index === 0"
                        size="small"
                        type="text"
                        @click="moveStep(index, -1)"
                      >
                        <IconifyIcon icon="lucide:arrow-up" class="size-4" />
                      </Button>
                    </Tooltip>
                    <Tooltip :title="$t('page.system.onboarding.moveDown')">
                      <Button
                        :disabled="index === sortedSteps.length - 1"
                        size="small"
                        type="text"
                        @click="moveStep(index, 1)"
                      >
                        <IconifyIcon icon="lucide:arrow-down" class="size-4" />
                      </Button>
                    </Tooltip>
                    <template v-if="record.stepType !== 1">
                      <Button
                        size="small"
                        type="text"
                        @click="openEditStep(record)"
                      >
                        <IconifyIcon icon="lucide:pencil" class="size-4" />
                      </Button>
                      <Popconfirm
                        :title="$t('page.system.onboarding.delStepConfirm')"
                        @confirm="removeStep(record)"
                      >
                        <Button danger size="small" type="text">
                          <IconifyIcon icon="lucide:trash-2" class="size-4" />
                        </Button>
                      </Popconfirm>
                    </template>
                  </div>
                </template>
              </template>
            </Table>
          </Tabs.TabPane>

          <!-- 页签 2：入口预设 -->
          <Tabs.TabPane
            key="preset"
            :tab="$t('page.system.onboarding.tabPreset')"
          >
            <div class="mb-3 flex items-center">
              <span class="text-xs text-gray-400">
                {{ $t('page.system.onboarding.presetTip') }}
              </span>
              <Button
                class="ml-auto"
                size="small"
                type="primary"
                @click="openAddEntry"
              >
                {{ $t('page.system.onboarding.addEntry') }}
              </Button>
            </div>
            <div
              v-if="quickPreset.length === 0"
              class="rounded-md border border-dashed border-gray-200 py-8 text-center"
            >
              <Empty :description="$t('page.system.onboarding.presetEmpty')" />
            </div>
            <div v-else class="flex flex-col gap-2">
              <div
                v-for="(entry, index) in quickPreset"
                :key="entry.code || index"
                class="flex items-center gap-3 rounded-md border border-gray-100 px-3 py-2"
              >
                <IconifyIcon
                  :icon="entry.icon || 'lucide:link'"
                  class="size-4 shrink-0"
                  :style="{ color: entry.color || '#1890ff' }"
                />
                <span class="text-sm font-medium text-gray-700">
                  {{ entry.label }}
                </span>
                <span class="truncate text-xs text-gray-400">{{ entry.path }}</span>
                <div class="ml-auto flex shrink-0 items-center gap-1">
                  <Button
                    :disabled="index === 0"
                    size="small"
                    type="text"
                    @click="moveEntry(index, -1)"
                  >
                    <IconifyIcon icon="lucide:arrow-up" class="size-4" />
                  </Button>
                  <Button
                    :disabled="index === quickPreset.length - 1"
                    size="small"
                    type="text"
                    @click="moveEntry(index, 1)"
                  >
                    <IconifyIcon icon="lucide:arrow-down" class="size-4" />
                  </Button>
                  <Button size="small" type="text" @click="openEditEntry(entry)">
                    <IconifyIcon icon="lucide:pencil" class="size-4" />
                  </Button>
                  <Popconfirm
                    :title="$t('page.system.onboarding.delStepConfirm')"
                    @confirm="removeEntry(entry)"
                  >
                    <Button danger size="small" type="text">
                      <IconifyIcon icon="lucide:trash-2" class="size-4" />
                    </Button>
                  </Popconfirm>
                </div>
              </div>
            </div>
          </Tabs.TabPane>

          <!-- 页签 3：区块开关 -->
          <Tabs.TabPane
            key="switches"
            :tab="$t('page.system.onboarding.tabSwitches')"
          >
            <div class="flex max-w-2xl flex-col gap-4 pt-2">
              <div class="flex items-center justify-between rounded-md bg-gray-50 px-4 py-3">
                <div>
                  <div class="text-sm font-medium text-gray-700">
                    {{ $t('page.system.onboarding.switchAnnounce') }}
                  </div>
                  <div class="mt-0.5 text-xs text-gray-400">
                    {{ $t('page.system.onboarding.switchAnnounceTip') }}
                  </div>
                </div>
                <Switch
                  v-model:checked="configForm.announceEnabled"
                  :checked-value="1"
                  :un-checked-value="0"
                />
              </div>
              <div class="flex items-center justify-between rounded-md bg-gray-50 px-4 py-3">
                <div>
                  <div class="text-sm font-medium text-gray-700">
                    {{ $t('page.system.onboarding.switchTodo') }}
                  </div>
                  <div class="mt-0.5 text-xs text-gray-400">
                    {{ $t('page.system.onboarding.switchTodoTip') }}
                  </div>
                </div>
                <Switch
                  v-model:checked="configForm.todoEnabled"
                  :checked-value="1"
                  :un-checked-value="0"
                />
              </div>
              <div class="flex items-center justify-between rounded-md bg-gray-50 px-4 py-3">
                <div>
                  <div class="text-sm font-medium text-gray-700">
                    {{ $t('page.system.onboarding.switchQuick') }}
                  </div>
                  <div class="mt-0.5 text-xs text-gray-400">
                    {{ $t('page.system.onboarding.switchQuickTip') }}
                  </div>
                </div>
                <Switch
                  v-model:checked="configForm.quickEnabled"
                  :checked-value="1"
                  :un-checked-value="0"
                />
              </div>
            </div>
          </Tabs.TabPane>
        </Tabs>
      </Card>
    </Spin>

    <!-- 新增/编辑自定义步骤 -->
    <Modal
      v-model:visible="stepModal.visible"
      :title="stepModal.editingIndex >= 0
        ? $t('page.system.onboarding.editStep')
        : $t('page.system.onboarding.addStep')"
      @ok="submitStepModal"
    >
      <div class="flex flex-col gap-3 py-2">
        <div>
          <label class="mb-1 block text-xs text-gray-500">
            {{ $t('page.system.onboarding.stepName') }}
            <span class="text-red-500">*</span>
          </label>
          <Input
            v-model:value="stepForm.stepName"
            :maxlength="50"
            :placeholder="$t('page.system.onboarding.stepName')"
          />
        </div>
        <div>
          <label class="mb-1 block text-xs text-gray-500">
            {{ $t('page.system.onboarding.stepDesc') }}
          </label>
          <Input
            v-model:value="stepForm.stepDesc"
            :maxlength="200"
            :placeholder="$t('page.system.onboarding.stepDesc')"
          />
        </div>
        <div>
          <label class="mb-1 block text-xs text-gray-500">
            {{ $t('page.system.onboarding.stepLink') }}
          </label>
          <Input
            v-model:value="stepForm.linkUrl"
            :placeholder="$t('page.system.onboarding.linkTip')"
          />
          <div class="mt-1 text-xs text-gray-400">
            {{ $t('page.system.onboarding.linkTip') }}
          </div>
        </div>
      </div>
    </Modal>

    <!-- 新增/编辑入口预设 -->
    <Modal
      v-model:visible="entryModal.visible"
      :title="entryModal.editingIndex >= 0
        ? $t('page.system.onboarding.editEntry')
        : $t('page.system.onboarding.addEntry')"
      @ok="submitEntryModal"
    >
      <div class="flex flex-col gap-3 py-2">
        <div>
          <label class="mb-1 block text-xs text-gray-500">
            {{ $t('page.system.onboarding.entryLabel') }}
            <span class="text-red-500">*</span>
          </label>
          <Input
            v-model:value="entryForm.label"
            :maxlength="30"
            :placeholder="$t('page.system.onboarding.entryLabel')"
          />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="mb-1 block text-xs text-gray-500">
              {{ $t('page.system.onboarding.entryIcon') }}
            </label>
            <div class="flex items-center gap-2">
              <IconifyIcon
                :icon="entryForm.icon || 'lucide:link'"
                class="size-4 shrink-0"
                :style="{ color: entryForm.color }"
              />
              <Input v-model:value="entryForm.icon" :maxlength="50" />
            </div>
          </div>
          <div>
            <label class="mb-1 block text-xs text-gray-500">
              {{ $t('page.system.onboarding.entryColor') }}
            </label>
            <Input v-model:value="entryForm.color" :maxlength="20" />
          </div>
        </div>
        <div>
          <label class="mb-1 block text-xs text-gray-500">
            {{ $t('page.system.onboarding.entryPath') }}
            <span class="text-red-500">*</span>
          </label>
          <Input
            v-model:value="entryForm.path"
            :placeholder="$t('page.system.onboarding.linkTip')"
          />
          <div class="mt-1 text-xs text-gray-400">
            {{ $t('page.system.onboarding.linkTip') }}
          </div>
        </div>
      </div>
    </Modal>

    <!-- 预览引导卡（13.3-6：五态模拟） -->
    <Modal
      v-model:visible="previewVisible"
      :footer="null"
      :title="$t('page.system.onboarding.previewTitle')"
      width="860px"
    >
      <div class="mb-4 flex items-center gap-2">
        <span class="text-sm text-gray-500">
          {{ $t('page.system.onboarding.previewStateLabel') }}
        </span>
        <Select
          v-model:value="previewState"
          :options="PREVIEW_STATE_OPTIONS"
          class="w-48"
        />
      </div>
      <OnboardingCard :preview-state="previewState" />
    </Modal>
  </Page>
</template>
