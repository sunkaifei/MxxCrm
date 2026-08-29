<script lang="ts" setup>
// 入职引导卡（方案 4.4 三区块设计）：
// A 入职进度（四态徽标 + 进度条 + 主按钮 + 可配置步骤清单）
// B 工作信息汇总（审批摘要 + 公告[可关] + 待办摘要[可关]）
// C 快捷工作入口（底部通栏：个人偏好 > 管理员 quickPreset > 内置默认，无权限入口隐藏不置灰）
// checklist 与 SubmitAuditDrawer 复用个人中心同源逻辑（useAuditChecklist），禁止两套
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import { IconifyIcon } from '@vben/icons';

import { Button, Card, Progress, Spin, Tag } from 'ant-design-vue';

import {
  getMenusRouterApi,
  getMyAuditApi,
  getMyProfileApi,
  getQuickNavPreferenceApi,
  getTodaySummaryApi,
} from '#/api';
import { getMyNoticeListApi } from '#/api/core/system/notice';
import { getOnboardingConfigApi } from '#/api/core/system/onboarding';
import { useAuditChecklist } from '#/composables/use-audit-checklist';
import { $t } from '#/locales';

import SubmitAuditDrawer from '../../../system/user/submit-audit-drawer.vue';
import QuickNavSettingsModal from '../../components/QuickNavSettingsModal.vue';

const props = withDefaults(
  defineProps<{
    // 预览态：配置页弹窗传入五态之一后用模拟数据渲染，不发真实请求（方案 13.3-6）
    previewState?: null | string;
  }>(),
  { previewState: null },
);
const isPreview = computed(() => !!props.previewState);

const emit = defineEmits<{
  'audit-change': [];
  'view-todos': [];
}>();

const router = useRouter();

// ===== 数据源（与个人中心 audit-status.vue 同源，共用 checklist） =====
const loading = ref(false);
const onboarding = ref<any>(null);
const data = ref<any>(null);
const profile = ref<any>(null);

const { checklist, selfRow } = useAuditChecklist(data, profile);

// 抽屉档案完善度四要素（个人中心同源）
const completeness = computed(() =>
  checklist.value.map((i) => ({ label: i.label, done: i.done })),
);

// ===== 状态徽标（后端五态 → 前端四态展示） =====
const stateTag = computed(() => {
  switch (onboarding.value?.state) {
    case 'passed': {
      return {
        color: 'success',
        label: $t('page.dashboard.onboarding.statePassed'),
      };
    }
    case 'in_approval': {
      return {
        color: 'processing',
        label: $t('page.dashboard.onboarding.statePending'),
      };
    }
    case 'rejected': {
      return {
        color: 'error',
        label: $t('page.dashboard.onboarding.stateRejected'),
      };
    }
    default: {
      return {
        color: 'warning',
        label: $t('page.dashboard.onboarding.stateNone'),
      };
    }
  }
});

// 进度 = 完成内置步骤/内置总数（后端聚合，自定义步骤不计入防失真）
const progressDone = computed(() => Number(onboarding.value?.progress?.done ?? 0));
const progressTotal = computed(() =>
  Number(onboarding.value?.progress?.total ?? 0),
);
const progressPercent = computed(() =>
  progressTotal.value > 0
    ? Math.round((progressDone.value / progressTotal.value) * 100)
    : 100,
);

// 步骤清单（管理员可配置，按 sort_order 展示）
const sortedSteps = computed(() => {
  const steps = Array.isArray(onboarding.value?.steps)
    ? onboarding.value.steps
    : [];
  return [...steps].toSorted((a, b) => (a?.sortOrder ?? 0) - (b?.sortOrder ?? 0));
});

// ===== 主按钮（四态：去完善/提交审批/修改后重新提交/审核中禁用显节点） =====
const primaryBtn = computed<null | {
  danger: boolean;
  disabled: boolean;
  key: string;
  text: string;
}>(() => {
  switch (onboarding.value?.state) {
    case 'pending_profile': {
      return {
        danger: false,
        disabled: false,
        key: 'go_complete',
        text: $t('page.dashboard.onboarding.goComplete'),
      };
    }
    case 'ready_submit': {
      return {
        danger: false,
        disabled: false,
        key: 'submit',
        text: $t('page.dashboard.onboarding.submitAudit'),
      };
    }
    case 'rejected': {
      return {
        danger: true,
        disabled: false,
        key: 'resubmit',
        text: $t('page.dashboard.onboarding.resubmit'),
      };
    }
    case 'in_approval': {
      return {
        danger: false,
        disabled: true,
        key: 'in_approval',
        text: $t('page.dashboard.onboarding.inApprovalText'),
      };
    }
    case 'passed': {
      return {
        danger: false,
        disabled: true,
        key: 'passed',
        text: $t('page.dashboard.onboarding.passedText'),
      };
    }
    default: {
      return null;
    }
  }
});

const approval = computed(() => onboarding.value?.approval || null);

function handlePrimary() {
  const key = primaryBtn.value?.key;
  if (key === 'go_complete') {
    // 跳第一个未完善项对应个人中心 tab
    const firstUndone = checklist.value.find((i) => !i.done);
    router
      .push(firstUndone ? `/profile?tab=${firstUndone.tab}` : '/profile')
      .catch(() => {});
  } else if (key === 'submit' || key === 'resubmit') {
    submitVisible.value = true;
  }
}

// ===== 步骤跳转（13.4-1 站内相对路径双重校验的前端侧） =====
function isSafeInternalPath(url?: null | string): boolean {
  if (!url || !url.startsWith('/') || url.startsWith('//')) return false;
  return /^[/a-zA-Z0-9_?=&.-]*$/.test(url);
}

function handleStepClick(step: any) {
  if (!isSafeInternalPath(step?.linkUrl)) return;
  router.push(step.linkUrl).catch(() => {});
}

// ===== 区块 B：审批摘要 / 公告（可关） / 待办摘要（可关） =====
const showApprovalSummary = computed(() =>
  ['in_approval', 'rejected'].includes(onboarding.value?.state),
);
const showAnnounce = computed(
  () => Number(onboarding.value?.config?.announceEnabled) === 1,
);
const showTodo = computed(
  () => Number(onboarding.value?.config?.todoEnabled) === 1,
);
const showQuick = computed(
  () => Number(onboarding.value?.config?.quickEnabled) === 1,
);

const notices = ref<any[]>([]);
async function loadNotices() {
  if (!showAnnounce.value) {
    notices.value = [];
    return;
  }
  try {
    const res: any = await getMyNoticeListApi({ page: 1, pageSize: 3 });
    notices.value = (Array.isArray(res) ? res : res?.items || []).slice(0, 3);
  } catch {
    notices.value = [];
  }
}

const todoTotal = ref(0);
const todoProcessed = ref(0);
async function loadTodoSummary() {
  if (!showTodo.value) return;
  try {
    const res: any = await getTodaySummaryApi();
    todoTotal.value = Number(res?.todoTotal ?? 0);
    todoProcessed.value = Number(res?.todoProcessed ?? 0);
  } catch {
    todoTotal.value = 0;
    todoProcessed.value = 0;
  }
}

// ===== 区块 C：快捷工作入口 =====
// 快捷入口条目（icon 恒为 Iconify 字符串；收窄 @vben WorkbenchQuickNavItem 的宽联合类型，供模板 IconifyIcon 直接绑定）
interface QuickEntry {
  color: string;
  icon: string;
  title: string;
  url: string;
}
const quickItems = ref<QuickEntry[]>([]);
const navSettingsVisible = ref(false);
const menuLeaves = ref<any[]>([]);

function flattenMenus(menus: any[]): any[] {
  const result: any[] = [];
  const traverse = (list: any[]) => {
    if (!Array.isArray(list)) return;
    for (const menu of list) {
      if (!menu) continue;
      const children = menu.children || [];
      if (menu.path && children.length === 0) {
        result.push(menu);
      }
      if (children.length > 0) {
        traverse(children);
      }
    }
  };
  traverse(menus);
  return result;
}

async function loadMenus() {
  try {
    const resp: any = await getMenusRouterApi({});
    menuLeaves.value = flattenMenus(resp?.items || resp || []);
  } catch {
    menuLeaves.value = [];
  }
}

// 登录即可访问的静态入口（个人中心系列，不在动态菜单接口中）
const ALWAYS_VISIBLE_PATHS = new Set(['/profile', '/profile?tab=password']);
function isEntryVisible(url: string): boolean {
  if (ALWAYS_VISIBLE_PATHS.has(url)) return true;
  return menuLeaves.value.some((m) => m.path === url);
}

const DEFAULT_QUICK_ENTRIES = computed<QuickEntry[]>(() => [
  {
    color: '#1890ff',
    icon: 'lucide:calendar-check',
    title: $t('page.dashboard.onboarding.entryAttendance'),
    url: '/finance/attendance',
  },
  {
    color: '#eb2f96',
    icon: 'lucide:wallet',
    title: $t('page.dashboard.onboarding.entryPayslip'),
    url: '/finance/payslip',
  },
  {
    color: '#52c41a',
    icon: 'lucide:user',
    title: $t('page.dashboard.onboarding.entryProfile'),
    url: '/profile',
  },
  {
    color: '#722ed1',
    icon: 'lucide:shield',
    title: $t('page.dashboard.onboarding.entrySecurity'),
    url: '/profile?tab=password',
  },
]);

async function loadQuickEntries() {
  const items: QuickEntry[] = [];
  // 1) 个人偏好优先（与工作台快捷导航同一份偏好数据，不建第二套）
  let savedPref: any[] = [];
  try {
    const prefResp: any = await getQuickNavPreferenceApi();
    savedPref = Array.isArray(prefResp) ? prefResp : prefResp?.items || [];
  } catch {
    savedPref = [];
  }
  if (savedPref.length > 0) {
    const sorted = savedPref.toSorted((a, b) => (a?.sort ?? 0) - (b?.sort ?? 0));
    for (const pref of sorted) {
      if (items.length >= 6) break;
      const menu = menuLeaves.value.find((m) => m.id === pref.menuId);
      if (!menu) continue; // 无权限菜单隐藏不置灰
      const meta = menu.meta || {};
      const rawTitle = meta.title || menu.name || menu.path;
      const title =
        typeof rawTitle === 'string' && rawTitle.startsWith('page.')
          ? $t(rawTitle)
          : rawTitle;
      items.push({
        color: '#1890ff',
        icon: meta.icon || 'lucide:menu',
        title: typeof title === 'string' ? title : String(title || ''),
        url: menu.path,
      });
    }
  }
  // 2) 管理员入口预设 quickPreset（前端跳转前再次校验站内路径）
  if (items.length === 0) {
    const preset = Array.isArray(onboarding.value?.config?.quickPreset)
      ? onboarding.value.config.quickPreset
      : [];
    for (const p of preset) {
      if (items.length >= 6) break;
      const path = typeof p?.path === 'string' ? p.path : '';
      if (!isSafeInternalPath(path) || !isEntryVisible(path)) continue;
      items.push({
        color: typeof p?.color === 'string' ? p.color : '#1890ff',
        icon: typeof p?.icon === 'string' && p.icon ? p.icon : 'lucide:link',
        title: typeof p?.label === 'string' && p.label ? p.label : path,
        url: path,
      });
    }
  }
  // 3) 内置默认集合（无权限入口隐藏不置灰）
  if (items.length === 0) {
    for (const entry of DEFAULT_QUICK_ENTRIES.value) {
      if (!entry.url || !isEntryVisible(entry.url)) continue;
      items.push(entry);
    }
  }
  quickItems.value = items;
}

function navTo(item: QuickEntry) {
  if (!isSafeInternalPath(item.url)) return;
  router.push(item.url).catch(() => {});
}

// 个人偏好保存后重算（个人一旦保存即以个人为准）
function onQuickNavSaved() {
  loadQuickEntries();
}

// ===== 提交入职审批（复用个人中心抽屉） =====
const submitVisible = ref(false);
async function handleAuditSuccess() {
  await loadAll();
  emit('audit-change');
}

// ===== 预览态（配置页弹窗）：模拟数据渲染五态，不发真实请求（方案 13.3-6） =====
function applyPreview(state: string) {
  const doneCount = ['in_approval', 'passed'].includes(state)
    ? 4
    : ['ready_submit', 'rejected'].includes(state)
      ? 3
      : 1;
  const timeText = new Date().toLocaleString('zh-CN', { hour12: false });
  onboarding.value = {
    state,
    steps: [
      { stepCode: 'base_info', stepName: $t('page.dashboard.onboarding.previewStepBase'), stepType: 1, linkUrl: '/profile', done: doneCount >= 1, sortOrder: 10 },
      { stepCode: 'resume', stepName: $t('page.dashboard.onboarding.previewStepResume'), stepType: 1, linkUrl: '/profile?tab=resume', done: doneCount >= 2, sortOrder: 20 },
      { stepCode: 'id_finance', stepName: $t('page.dashboard.onboarding.previewStepIdfinance'), stepType: 1, linkUrl: '/profile?tab=idfinance', done: doneCount >= 3, sortOrder: 30 },
      { stepCode: 'bank_card', stepName: $t('page.dashboard.onboarding.previewStepBank'), stepType: 1, linkUrl: '/profile', done: doneCount >= 4, sortOrder: 40 },
    ],
    progress: { done: doneCount, total: 4 },
    config: { announceEnabled: 1, todoEnabled: 1, quickEnabled: 1, quickPreset: null },
    approval:
      state === 'in_approval'
        ? { currentNodeName: $t('page.dashboard.onboarding.previewNodeName'), submittedAt: timeText, rejectReason: null }
        : state === 'rejected'
          ? { currentNodeName: null, submittedAt: timeText, rejectReason: $t('page.dashboard.onboarding.previewRejectReason') }
          : null,
  };
  data.value = null;
  profile.value = null;
  notices.value = [
    { id: 'preview', title: $t('page.dashboard.onboarding.previewNoticeTitle'), publishTime: timeText },
  ];
  todoTotal.value = 3;
  todoProcessed.value = 1;
  quickItems.value = [...DEFAULT_QUICK_ENTRIES.value];
  loading.value = false;
}

watch(
  () => props.previewState,
  (s) => {
    if (s) applyPreview(s);
  },
);

// ===== 主加载（聚合配置 + 审核状态 + 档案 + 菜单并行，入口串行） =====
async function loadAll() {
  if (props.previewState) {
    applyPreview(props.previewState);
    return;
  }
  loading.value = true;
  try {
    const [cfg, auditRes, profileRes] = await Promise.all([
      getOnboardingConfigApi().catch(() => null),
      getMyAuditApi().catch(() => null),
      getMyProfileApi().catch(() => null),
      loadMenus(),
    ]);
    onboarding.value = cfg ?? null;
    data.value = auditRes?.data?.data ?? auditRes?.data ?? auditRes ?? null;
    profile.value = profileRes ?? null;
    await loadQuickEntries();
    loadNotices();
    loadTodoSummary();
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadAll();
});
</script>

<template>
  <Card class="onboarding-card">
    <template #title>
      <div class="flex items-center gap-2">
        <span
          class="inline-block size-2 rounded-full bg-purple-500"
          aria-hidden="true"
        ></span>
        <span>{{ $t('page.dashboard.onboarding.cardTitle') }}</span>
        <Tag v-if="onboarding" :color="stateTag.color" class="ml-1">
          {{ stateTag.label }}
        </Tag>
      </div>
    </template>
    <Spin :spinning="loading">
      <div class="flex flex-col gap-5 lg:flex-row">
        <!-- 区块 A：入职进度（60%） -->
        <div class="w-full lg:w-3/5">
          <div class="text-sm text-gray-500">
            {{ $t('page.dashboard.onboarding.welcome') }}
          </div>
          <div class="mt-3 flex items-center gap-3">
            <Progress
              class="flex-1"
              :percent="progressPercent"
              :show-info="false"
              :stroke-width="8"
            />
            <span class="shrink-0 text-sm font-medium text-gray-700">
              {{ progressDone }}/{{ progressTotal }}
            </span>
          </div>
          <Button
            v-if="primaryBtn"
            class="mt-3"
            :type="primaryBtn.disabled ? 'default' : 'primary'"
            :danger="primaryBtn.danger"
            :disabled="primaryBtn.disabled"
            @click="handlePrimary"
          >
            {{
              primaryBtn.key === 'in_approval' && approval?.currentNodeName
                ? `${primaryBtn.text}：${approval.currentNodeName}`
                : primaryBtn.text
            }}
          </Button>
          <!-- 步骤清单（内置步骤计入完成度；自定义步骤跳链接不计入） -->
          <div class="mt-4">
            <div
              v-for="step in sortedSteps"
              :key="step.stepCode"
              class="flex items-center gap-2 py-1.5 text-sm"
              :class="{ 'cursor-pointer': !!step.linkUrl }"
              @click="handleStepClick(step)"
            >
              <IconifyIcon
                v-if="step.stepType === 1 && step.done"
                icon="lucide:circle-check"
                class="size-4 shrink-0 text-green-500"
              />
              <IconifyIcon
                v-else-if="step.stepType === 1"
                icon="lucide:circle"
                class="size-4 shrink-0 text-gray-300"
              />
              <IconifyIcon
                v-else
                icon="lucide:external-link"
                class="size-4 shrink-0 text-blue-400"
              />
              <span
                :class="
                  step.stepType === 1 && step.done
                    ? 'text-gray-400 line-through'
                    : 'text-gray-700'
                "
              >
                {{ step.stepName }}
              </span>
              <Tag v-if="step.stepType !== 1" class="ml-1 shrink-0">
                {{ $t('page.dashboard.onboarding.stepCustom') }}
              </Tag>
            </div>
            <div
              v-if="sortedSteps.some((s) => s.stepType !== 1)"
              class="mt-1 text-xs text-gray-400"
            >
              {{ $t('page.dashboard.onboarding.customStepsTip') }}
            </div>
          </div>
        </div>
        <!-- 区块 B：工作信息汇总（40%） -->
        <div class="flex w-full flex-col gap-4 lg:w-2/5">
          <!-- 审批摘要（审核中/已驳回时展示） -->
          <div v-if="showApprovalSummary" class="rounded-md bg-gray-50 p-3">
            <div class="text-sm font-medium text-gray-700">
              {{ $t('page.dashboard.onboarding.approvalSummary') }}
            </div>
            <template v-if="approval">
              <div
                v-if="approval.currentNodeName"
                class="mt-2 text-xs text-gray-500"
              >
                {{ $t('page.dashboard.onboarding.currentNode') }}：
                {{ approval.currentNodeName }}
              </div>
              <div v-if="approval.submittedAt" class="mt-1 text-xs text-gray-500">
                {{ $t('page.dashboard.onboarding.submittedAt') }}：
                {{ approval.submittedAt }}
              </div>
              <div v-if="approval.rejectReason" class="mt-1 text-xs text-red-500">
                {{ $t('page.dashboard.onboarding.rejectReason') }}：
                {{ approval.rejectReason }}
              </div>
            </template>
          </div>
          <div
            v-else-if="onboarding?.state === 'ready_submit'"
            class="rounded-md bg-gray-50 p-3 text-xs text-gray-500"
          >
            {{ $t('page.dashboard.onboarding.approvalWaitingTip') }}
          </div>
          <!-- 公告最近 3 条（可关） -->
          <div v-if="showAnnounce">
            <div class="text-sm font-medium text-gray-700">
              {{ $t('page.dashboard.onboarding.announcements') }}
            </div>
            <div v-if="notices.length > 0" class="mt-2">
              <div
                v-for="n in notices"
                :key="n.id"
                class="flex items-center gap-2 py-1 text-xs text-gray-600"
              >
                <span
                  class="inline-block size-1.5 shrink-0 rounded-full bg-blue-400"
                  aria-hidden="true"
                ></span>
                <span class="truncate">{{ n.title }}</span>
                <span class="ml-auto shrink-0 text-gray-400">
                  {{ n.publishTime || n.createTime || '' }}
                </span>
              </div>
            </div>
            <div v-else class="mt-2 text-xs text-gray-400">
              {{ $t('page.dashboard.onboarding.noAnnouncement') }}
            </div>
          </div>
          <!-- 待办摘要（可关）→ 待办总览 -->
          <div v-if="showTodo" class="rounded-md bg-gray-50 p-3">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-gray-700">
                {{ $t('page.dashboard.onboarding.todoSummary') }}
              </span>
              <Button type="link" size="small" @click="emit('view-todos')">
                {{ $t('page.dashboard.onboarding.viewTodos') }}
              </Button>
            </div>
            <div class="mt-1 text-xs text-gray-500">
              {{ todoTotal }}
              {{ $t('page.dashboard.onboarding.todoTotalUnit') }} ·
              {{ todoProcessed }}
              {{ $t('page.dashboard.onboarding.todoProcessedUnit') }}
            </div>
          </div>
        </div>
      </div>
      <!-- 区块 C：快捷工作入口（底部通栏） -->
      <div v-if="showQuick" class="mt-5 border-t border-gray-100 pt-4">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium text-gray-700">
            {{ $t('page.dashboard.onboarding.quickEntries') }}
          </span>
          <Button
            v-if="!isPreview"
            type="link"
            size="small"
            @click="navSettingsVisible = true"
          >
            {{ $t('page.dashboard.settings') }}
          </Button>
        </div>
        <div class="mt-3 flex flex-wrap gap-2">
          <button
            v-for="item in quickItems"
            :key="item.url"
            class="flex items-center gap-2 rounded-md border border-gray-100 px-3 py-2 text-sm text-gray-700 transition hover:bg-gray-50"
            type="button"
            @click="navTo(item)"
          >
            <IconifyIcon
              :icon="item.icon"
              class="size-4"
              :style="{ color: item.color }"
            />
            <span>{{ item.title }}</span>
          </button>
        </div>
      </div>
    </Spin>
  </Card>

  <!-- 个人快捷入口增删（复用工作台快捷导航设置，同一份偏好数据） -->
  <QuickNavSettingsModal
    v-model:visible="navSettingsVisible"
    @saved="onQuickNavSaved"
  />

  <!-- 提交入职审批（复用个人中心抽屉，同源 selfRow/checklist） -->
  <SubmitAuditDrawer
    v-model:visible="submitVisible"
    :row="selfRow"
    :completeness="completeness"
    @success="handleAuditSuccess"
  />
</template>
