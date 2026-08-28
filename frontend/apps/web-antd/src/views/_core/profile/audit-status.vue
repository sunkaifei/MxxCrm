<script lang="ts" setup>
// 个人中心「入职审批」（F7）：员工本人查看审批状态/进度、完善档案后提交/重新提交入职审批
// 身份一律取自 JWT（后端 /profile/audit/my + /profile/my），员工无需权限码
//
// 设计方向「入职旅程 · 精确叙事」：
// - 状态 Hero：蓝图点阵底纹 + 状态色渐变，一眼看清当前所处阶段
// - 提交前准备：四要素清单（个人信息/简历/财务/紧急联系人），等宽序号 + 进度条，缺一不可
// - 审批进度：垂直流水线只呈现当前（最新）流程，等宽序号节点 + 状态色连线，当前节点脉冲高亮
// - 审批记录：跨实例聚合时间轴，每轮提交为一组可折叠分段（新→旧，默认展开最新一轮）
import { computed, onMounted, ref } from 'vue';

import {
  Button,
  Empty,
  Input,
  message,
  Modal,
  Spin,
  Tag,
  Timeline,
  TimelineItem,
  Tooltip,
} from 'ant-design-vue';

import { formatDateTime } from '@vben/utils';
import { useUserStore } from '@vben/stores';

import { cancelApprovalApi, getMyAuditApi, getMyProfileApi } from '#/api';
import { sortApprovalNodes } from '#/api/core/system/approval';

import SubmitAuditDrawer from '../../system/user/submit-audit-drawer.vue';

const emit = defineEmits<{
  'switch-tab': [tab: string];
  'audit-change': [];
}>();

const userStore = useUserStore();

const loading = ref(false);
const loadError = ref(false);
const data = ref<any>(null);
const profile = ref<any>(null);
const submitVisible = ref(false);

// 实例状态映射（与审批引擎一致）
const statusMap: Record<number, { color: string; label: string }> = {
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
  5: { label: '已撤回', color: 'default' },
  6: { label: '待修改', color: 'orange' },
};

// 日志动作
const logActionText: Record<number, string> = {
  0: '提交',
  1: '通过',
  2: '驳回',
  3: '转办',
  4: '委派',
  5: '加签',
  6: '退回',
  7: '撤回',
  8: '抄送',
};

const logActionColor: Record<number, string> = {
  0: 'blue',
  1: 'green',
  2: 'red',
  3: 'blue',
  4: 'blue',
  5: 'orange',
  6: 'orange',
  7: 'gray',
  8: 'cyan',
};

// 审批模式映射
const approveModeMap: Record<number, string> = {
  1: '或签',
  2: '会签',
  3: '依次审批',
};

// AntD Tag 色名 → 色值（实例芯片状态点）
const STATUS_HEX: Record<string, string> = {
  processing: '#1677ff',
  warning: '#faad14',
  success: '#52c41a',
  error: '#ff4d4f',
  orange: '#fa8c16',
  default: '#8c8c8c',
};

// 线性图标 path 数据（lucide 风格描边）
const ICON_PATHS: Record<string, string> = {
  check: 'M20 6L9 17l-5-5',
  x: 'M18 6L6 18M6 6l12 12',
  clock: 'M12 8v4l3 3M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
  send: 'M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z',
  rollback: 'M3 9h12a6 6 0 010 12h-3M3 9l4-4M3 9l4 4',
  edit: 'M12 20h9M16.5 3.5a2.1 2.1 0 013 3L7 19l-4 1 1-4L16.5 3.5z',
  user: 'M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2M12 11a4 4 0 100-8 4 4 0 000 8z',
  refresh: 'M1 4v6h6M23 20v-6h-6M20.5 9A9 9 0 005.6 5.6L1 10M3.5 15a9 9 0 0014.9 3.4L23 14',
};

const currentUserId = computed(() =>
  Number(userStore.userInfo?.userId ?? userStore.userInfo?.id ?? 0),
);

// 已通过（auditStatus=1）：父级个人中心已隐藏该 Tab，此处兜底展示
const approved = computed(() => data.value?.auditStatus === 1);

const instances = computed<any[]>(() => data.value?.instances || []);
const latest = computed<any>(() => instances.value.at(-1) || null);

// 审核状态结论（方案 3.2 个人中心）
const auditState = computed(() => {
  if (!data.value) return { type: 'loading', label: '加载中', color: 'default' };
  if (approved.value) return { type: 'approved', label: '已通过', color: 'success' };
  const st = latest.value?.status;
  if (st === 1 || st === 2) return { type: 'pending', label: '审批中', color: 'processing' };
  if (st === 4) return { type: 'rejected', label: '已驳回', color: 'error' };
  if (st === 5) return { type: 'withdrawn', label: '已撤回', color: 'default' };
  if (st === 6) return { type: 'modify', label: '待修改', color: 'warning' };
  return { type: 'none', label: '未提交', color: 'default' };
});

const canSubmit = computed(() =>
  ['none', 'rejected', 'withdrawn', 'modify'].includes(auditState.value.type),
);

// ===== 提交前准备：四要素档案清单（全部完善后才可提交） =====
const checklist = computed(() => {
  const p = profile.value || {};
  const basicDone = !!(p?.basic?.nickName && String(p.basic.nickName).trim());
  const resumeDone = (p?.resume?.length ?? 0) > 0;
  const financeDone = !!(p?.idCard?.masked && p?.bank?.maskedCardNo);
  const contactDone = (p?.emergencyContacts?.length ?? 0) > 0;
  return [
    {
      key: 'basic',
      label: '个人信息',
      desc: '昵称 / 姓名等基础信息',
      tab: 'basic',
      done: basicDone,
    },
    {
      key: 'resume',
      label: '个人简历',
      desc: '教育 / 工作经历至少一条',
      tab: 'resume',
      done: resumeDone,
    },
    {
      key: 'finance',
      label: '财务信息',
      desc: '身份证与工资卡',
      tab: 'idfinance',
      done: financeDone,
    },
    {
      key: 'contact',
      label: '紧急联系人',
      desc: '至少一位紧急联系人',
      tab: 'emergency',
      done: contactDone,
    },
  ];
});
const doneCount = computed(() => checklist.value.filter((i) => i.done).length);
const allDone = computed(() => doneCount.value === checklist.value.length);

// ===== 审批记录聚合：每轮提交为一组分段（新→旧），默认展开最新一轮；进度流水线始终只看最新流程 =====
const rounds = computed<any[]>(() =>
  (instances.value as any[])
    .map((inst: any, idx: number) => {
      const base = inst?.logs || [];
      // 提交节点：日志表无「提交」动作，由实例 submittedAt 组合为首条发起记录
      const logs = inst?.submittedAt
        ? [
            {
              id: `submit-${inst.id}`,
              action: 0,
              nodeName: '提交审批',
              approverName: inst.submitterName || '',
              createTime: inst.submittedAt,
            },
            ...base,
          ]
        : base;
      return { roundNo: idx + 1, key: `round-${idx}`, inst, logs };
    })
    .reverse(),
);

// 折叠展开状态：undefined 时回落到默认规则（仅最新一轮展开）
const openRounds = ref<Record<string, boolean>>({});
const newestRoundKey = computed(() => rounds.value[0]?.key ?? '');

function toggleRound(key: string) {
  openRounds.value[key] = !isOpenKey(key);
}

function isOpenKey(key: string) {
  const v = openRounds.value[key];
  return v === undefined ? key === newestRoundKey.value : v;
}

function dotHex(status: number) {
  return STATUS_HEX[statusMap[status]?.color ?? 'default'] || '#8c8c8c';
}

// 节点状态：0未到达 1审批中 2已通过 3已驳回 4已完成
function nodeStatusLabel(st: number) {
  if (st === 1) return '审批中';
  if (st === 2 || st === 4) return '已通过';
  if (st === 3) return '已驳回';
  return '未到达';
}
function dotClassOf(st: number) {
  if (st === 1) return 'p-dot-active';
  if (st === 2 || st === 4) return 'p-dot-done';
  if (st === 3) return 'p-dot-err';
  return 'p-dot-muted';
}
function lineClassOf(st: number) {
  if (st === 2 || st === 4) return 'p-line-done';
  if (st === 1) return 'p-line-active';
  return 'p-line-muted';
}

// 垂直流水线：发起人 → 审批节点 → 结束（始终展示最新一轮流程）
const flowSteps = computed(() => {
  const inst = latest.value;
  if (!inst) return [];
  const steps: any[] = [];
  steps.push({
    kind: 'user',
    title: '发起人',
    sub: inst.submitterName || '-',
    dotClass: 'p-dot-done',
    lineClass: 'p-line-done',
  });
  const nodes: any[] = sortApprovalNodes(inst.flowNodes, inst.flowEdges).filter(
    (n: any) => n.nodeType === 2,
  );
  nodes.forEach((n: any, i: number) => {
    const st = n.nodeStatus ?? 0;
    steps.push({
      kind: 'num',
      text: String(i + 1),
      title: n.nodeName || `审批节点 ${i + 1}`,
      mode: n.approveMode ? approveModeMap[n.approveMode] : '',
      sub: [n.approverName || '审批人待定', nodeStatusLabel(st)].filter(Boolean).join(' · '),
      dotClass: dotClassOf(st),
      lineClass: lineClassOf(st),
    });
  });
  if (nodes.length > 0) {
    const done = inst.status === 3;
    const rejected = inst.status === 4;
    steps.push({
      kind: 'check',
      title: done ? '审批通过' : rejected ? '已驳回' : '流程结束',
      sub: done
        ? '入职审批已全部通过，账号正式启用'
        : rejected
          ? '审批被驳回，请根据意见完善后重新提交'
          : '审批流程已结束',
      dotClass: done ? 'p-dot-done' : rejected ? 'p-dot-err' : 'p-dot-muted',
      lineClass: '',
    });
  }
  return steps;
});

// ===== 撤销审批（审批进行中，发起人本人撤回） =====
const cancelVisible = ref(false);
const cancelReason = ref('');
const canceling = ref(false);
const canCancel = computed(() => auditState.value.type === 'pending');

function openCancel() {
  cancelReason.value = '';
  cancelVisible.value = true;
}

async function handleCancel() {
  const reason = cancelReason.value.trim();
  if (!reason) {
    message.warning('请填写撤回理由');
    return;
  }
  if (!latest.value?.id) return;
  canceling.value = true;
  try {
    await cancelApprovalApi({
      instanceId: latest.value.id,
      cancelReason: reason,
    });
    message.success('已撤回，可修改档案后重新提交');
    cancelVisible.value = false;
    await loadData();
    emit('audit-change');
  } catch (error: any) {
    message.error(error?.message || '撤回失败');
  } finally {
    canceling.value = false;
  }
}

// ===== 状态 Hero 配置 =====
const hero = computed(() => {
  const t = auditState.value.type;
  if (t === 'approved')
    return {
      icon: 'check',
      tone: 'success',
      title: '入职审批已通过',
      desc: '您的入职审批已全部通过，账号已正常启用。',
      canAction: false,
    };
  if (t === 'pending')
    return {
      icon: 'clock',
      tone: 'primary',
      title: '审批进行中',
      desc: '您的入职审批正在流转中，请耐心等待审批人处理；下方可随时查看审批进度与记录。',
      canAction: false,
    };
  if (t === 'rejected')
    return {
      icon: 'x',
      tone: 'danger',
      title: '审批未通过',
      desc: '本次入职审批被驳回，请根据审批意见完善档案后重新提交。',
      canAction: true,
    };
  if (t === 'withdrawn')
    return {
      icon: 'rollback',
      tone: 'default',
      title: '已撤回',
      desc: '该次入职审批已撤回，可重新提交发起入职审批。',
      canAction: true,
    };
  if (t === 'modify')
    return {
      icon: 'edit',
      tone: 'primary',
      title: '待修改',
      desc: '审批人要求修改档案后重新提交，请完善后再次发起。',
      canAction: true,
    };
  return {
    icon: 'send',
    tone: 'default',
    title: '尚未提交入职审批',
    desc: '完善下方档案清单后即可发起入职审批；审批全部通过后账号正式启用。',
    canAction: true,
  };
});

const btnLabel = computed(() => {
  const t = auditState.value.type;
  if (t === 'modify') return '修改后重新提交';
  if (t === 'none') return '提交入职审批';
  return '重新提交';
});

// 提交抽屉所需的本人 row（复用 SubmitAuditDrawer）
// 完整信息从 /profile/my 的 MyProfileVO 取（部门/岗位/手机号/邮箱/入职时间等），
// userStore.userInfo 仅含登录基础信息，无法提供这些字段
const selfRow = computed(() => {
  const u: any = userStore.userInfo || {};
  const p: any = profile.value || {};
  const basic = p?.basic || {};
  const employ = p?.employ || {};
  return {
    id: currentUserId.value,
    nickName: basic.nickName || u.nickName || u.realName || '',
    userName: employ.userName || u.username || '',
    auditStatus: data.value?.auditStatus ?? 0,
    approvalStatus: latest.value?.status ?? undefined,
    approvalInstanceId: data.value?.latestInstanceId ?? undefined,
    flowCode: latest.value?.flowCode || 'hire_approval',
    deptName: (employ.deptNames || []).join('、'),
    postName: (employ.postNames || []).join('、'),
    roleName: u.roleName || '',
    mobile: basic.mobileMasked || '',
    email: basic.email || '',
    hireDate: employ.hireDate || '',
    directManagerName: employ.directManagerName || '',
  };
});

// 提交按钮：未完善时禁用并引导跳转到第一项缺失档案
function handleSubmitClick() {
  if (!allDone.value) {
    message.warning('请先完善全部档案信息后再提交入职审批');
    const first = checklist.value.find((i) => !i.done);
    if (first) emit('switch-tab', first.tab);
    return;
  }
  submitVisible.value = true;
}

function handleSubmitted() {
  submitVisible.value = false;
  loadData();
  emit('audit-change');
}

async function loadData() {
  loading.value = true;
  loadError.value = false;
  try {
    const [auditRes, profileRes] = await Promise.all([
      getMyAuditApi(),
      getMyProfileApi().catch(() => null),
    ]);
    data.value = auditRes?.data?.data ?? auditRes?.data ?? auditRes ?? null;
    profile.value = profileRes ?? null;
    openRounds.value = {};
  } catch (error) {
    // 加载失败必须可见化：失败时若静默置空，页面会渲染成「尚无审批实例」假象，
    // 与提交侧的在途拦截提示自相矛盾，用户无法分辨「没提交过」和「加载失败」
    console.error('[audit-status] 审核状态加载失败', error);
    loadError.value = true;
    data.value = null;
    message.error('审核状态加载失败，请重新加载');
  } finally {
    loading.value = false;
  }
}

onMounted(loadData);

defineExpose({ reload: loadData });
</script>

<template>
  <div class="onboard-space">
    <!-- 已通过兜底：父级正常会隐藏 Tab，此分支仅在状态刷新前短暂出现 -->
    <section v-if="approved" class="hero hero-success">
      <div class="hero-grid"></div>
      <div class="hero-icon tone-success">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path :d="ICON_PATHS.check" />
        </svg>
      </div>
      <div class="hero-main">
        <h2 class="hero-title">入职审批已通过</h2>
        <p class="hero-desc">您的入职审批已全部通过，账号已正常启用，该栏目无需再展示。</p>
      </div>
    </section>

    <Spin :spinning="loading" v-else>
      <div v-if="data" class="space-y-4">
        <!-- ===== 状态 Hero ===== -->
        <section class="hero" :class="`hero-${hero.tone}`">
          <div class="hero-grid"></div>
          <div class="hero-icon" :class="`tone-${hero.tone}`">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path :d="ICON_PATHS[hero.icon]" />
            </svg>
          </div>
          <div class="hero-main">
            <div class="hero-title-row">
              <h2 class="hero-title">{{ hero.title }}</h2>
              <Tag v-if="latest?.status" :color="statusMap[latest.status]?.color || 'default'">
                {{ statusMap[latest.status]?.label || latest.statusName }}
              </Tag>
            </div>
            <p class="hero-desc">{{ hero.desc }}</p>
            <div v-if="latest" class="hero-meta">
              提交人 {{ latest.submitterName || '-' }} · {{ formatDateTime(latest.submittedAt) || '-' }} 提交 ·
              共 {{ instances.length }} 次提交
            </div>
            <p v-if="auditState.type === 'pending' && latest?.submittedAt" class="hero-warn">
              当前流程于 {{ formatDateTime(latest.submittedAt) }} 发起；如需修改档案并重新发起，请先点击「撤销审批」。
            </p>
          </div>
          <div class="hero-action">
            <template v-if="hero.canAction">
              <Tooltip v-if="!allDone" title="请先完善全部档案信息">
                <span class="inline-block">
                  <Button type="primary" size="large" :disabled="!allDone" @click="handleSubmitClick">
                    {{ btnLabel }}
                  </Button>
                </span>
              </Tooltip>
              <Button v-else type="primary" size="large" @click="handleSubmitClick">
                {{ btnLabel }}
              </Button>
            </template>
            <div v-else class="hero-actions">
              <Button class="hero-refresh" @click="loadData">
                <svg class="refresh-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path :d="ICON_PATHS.refresh" />
                </svg>
                刷新进度
              </Button>
              <Button v-if="canCancel" danger @click="openCancel">撤销审批</Button>
            </div>
          </div>
        </section>

        <!-- ===== 提交前准备：四要素档案清单 ===== -->
        <section v-if="canSubmit" class="panel prep-panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">提交前准备</div>
              <div class="panel-sub">以下档案信息完善后才能提交入职审批，点击条目可直接跳转填写</div>
            </div>
            <div class="prep-progress">
              <div class="prep-progress-text">
                {{ doneCount }} / {{ checklist.length }}
                <span v-if="allDone" class="prep-ready">已就绪</span>
              </div>
              <div class="prep-bar">
                <div
                  class="prep-bar-fill"
                  :class="{ full: allDone }"
                  :style="{ width: `${(doneCount / checklist.length) * 100}%` }"
                ></div>
              </div>
            </div>
          </div>
          <div class="prep-grid">
            <div
              v-for="(item, i) in checklist"
              :key="item.key"
              class="prep-item"
              :class="{ done: item.done }"
              @click="emit('switch-tab', item.tab)"
            >
              <div class="prep-num" :class="{ done: item.done }">
                <svg v-if="item.done" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                  <path :d="ICON_PATHS.check" />
                </svg>
                <span v-else class="prep-num-text">{{ String(i + 1).padStart(2, '0') }}</span>
              </div>
              <div class="prep-info">
                <div class="prep-name">{{ item.label }}</div>
                <div class="prep-desc">{{ item.desc }}</div>
              </div>
              <Tag :color="item.done ? 'success' : 'warning'" class="prep-tag">
                {{ item.done ? '已完善' : '待完善' }}
              </Tag>
              <span class="prep-arrow">→</span>
            </div>
          </div>
        </section>

        <!-- ===== 审批进度（流水线，始终呈现最新一轮流程） ===== -->
        <template v-if="latest">
          <section class="panel pipeline-panel">
            <div class="panel-head">
              <div>
                <div class="panel-title">审批进度</div>
                <div class="panel-sub">
                  {{
                    latest.businessTitle
                      ? `${latest.businessTitle} · 当前流程的节点流转`
                      : '当前流程的节点流转'
                  }}
                </div>
              </div>
              <Tag :color="statusMap[latest.status]?.color || 'default'">
                {{ statusMap[latest.status]?.label || latest.statusName || '-' }}
              </Tag>
            </div>

            <div class="pipeline">
              <div v-for="(step, idx) in flowSteps" :key="idx" class="p-step">
                <div class="p-rail">
                  <div class="p-dot" :class="step.dotClass">
                    <svg v-if="step.kind === 'user'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <path :d="ICON_PATHS.user" />
                    </svg>
                    <svg v-else-if="step.kind === 'check'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                      <path :d="ICON_PATHS.check" />
                    </svg>
                    <span v-else class="p-dot-num">{{ step.text }}</span>
                  </div>
                  <div v-if="idx < flowSteps.length - 1" class="p-line" :class="step.lineClass"></div>
                </div>
                <div class="p-body">
                  <div class="p-title">
                    {{ step.title }}
                    <Tag v-if="step.mode" :color="step.mode === '会签' ? 'purple' : step.mode === '依次审批' ? 'orange' : 'blue'">
                      {{ step.mode }}
                    </Tag>
                  </div>
                  <div class="p-sub">{{ step.sub }}</div>
                </div>
              </div>
            </div>
          </section>

          <!-- ===== 审批记录（跨实例聚合时间轴：每轮提交一分组，按发起时间倒序，默认展开最新一轮） ===== -->
          <section class="panel record-panel">
            <div class="panel-head">
              <div>
                <div class="panel-title">审批记录</div>
                <div class="panel-sub">共 {{ instances.length }} 次提交，按发起时间倒序展示各轮每一步处理明细</div>
              </div>
            </div>
            <div v-if="rounds.length" class="round-list">
              <div v-for="r in rounds" :key="r.key" class="round-group">
                <button type="button" class="round-head" @click="toggleRound(r.key)">
                  <span class="chip-dot" :style="{ background: dotHex(r.inst.status) }"></span>
                  <span class="round-name">第 {{ r.roundNo }} 次提交</span>
                  <Tag size="small" :color="statusMap[r.inst.status]?.color || 'default'">
                    {{ statusMap[r.inst.status]?.label || r.inst.statusName || '-' }}
                  </Tag>
                  <span class="round-time">{{ formatDateTime(r.inst.submittedAt) || '-' }}</span>
                  <svg
                    class="round-caret"
                    :class="{ open: isOpenKey(r.key) }"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="M6 9l6 6 6-6" />
                  </svg>
                </button>
                <div v-show="isOpenKey(r.key)" class="round-body">
                  <div v-if="r.logs.length" class="log-timeline">
                    <Timeline>
                      <TimelineItem
                        v-for="(log, li) in r.logs"
                        :key="log.id ?? `${r.key}-${li}`"
                        :color="logActionColor[log.action] || 'blue'"
                      >
                        <div class="log-title">
                          {{ log.nodeName || logActionText[log.action] || '审批' }}
                          <Tag v-if="log.action === 0" color="geekblue" size="small" class="log-tag">发起</Tag>
                        </div>
                        <div class="log-sub">
                          {{ log.approverName || log.operatorName || '-' }} ·
                          {{ logActionText[log.action] || '--' }} · {{ formatDateTime(log.createTime || log.create_at) || '-' }}
                        </div>
                        <div v-if="log.comment || log.reason" class="log-comment">
                          {{ log.comment || log.reason }}
                        </div>
                      </TimelineItem>
                    </Timeline>
                  </div>
                  <div v-else class="log-empty">该轮暂无审批操作记录</div>
                </div>
              </div>
            </div>
            <div v-else class="log-empty">暂无审批记录</div>
          </section>
        </template>

        <!-- ===== 无实例（未提交过） ===== -->
        <section v-else class="panel empty-panel">
          <Empty description="尚无审批实例，完善档案后点击「提交入职审批」发起审批" />
        </section>
      </div>
    </Spin>

    <SubmitAuditDrawer
      v-model:visible="submitVisible"
      :row="selfRow"
      :completeness="checklist"
      @success="handleSubmitted"
    />

    <!-- ===== 撤销审批弹窗（审批进行中，发起人本人撤回，理由必填） ===== -->
    <Modal
      v-model:open="cancelVisible"
      title="撤销审批"
      ok-text="确认撤回"
      cancel-text="暂不"
      :confirm-loading="canceling"
      @ok="handleCancel"
    >
      <p class="cancel-tip">
        撤回后本次审批将立即终止，您可修改档案信息后重新提交。撤回理由会记录在审批日志中，供后续追溯。
      </p>
      <Input.TextArea
        v-model:value="cancelReason"
        :rows="3"
        :maxlength="200"
        show-count
        placeholder="请填写撤回理由（必填）"
      />
    </Modal>
  </div>
</template>

<style scoped>
.onboard-space {
  animation: rise-in 0.5s ease-out both;
}

@keyframes rise-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

@keyframes pulse-ring {
  0% {
    box-shadow: 0 0 0 0 hsl(var(--primary) / 35%);
  }
  70% {
    box-shadow: 0 0 0 9px hsl(var(--primary) / 0%);
  }
  100% {
    box-shadow: 0 0 0 0 hsl(var(--primary) / 0%);
  }
}

/* ===== 面板通用 ===== */
.panel {
  border: 1px solid hsl(var(--border));
  border-radius: 12px;
  padding: 18px 22px;
  background: hsl(var(--card));
  animation: rise-in 0.5s ease-out both;
}

.prep-panel {
  animation-delay: 0.06s;
}
.pipeline-panel {
  animation-delay: 0.12s;
}
.record-panel {
  animation-delay: 0.18s;
}

.panel-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}

.panel-title {
  font-size: 15px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.panel-sub {
  margin-top: 2px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== 状态 Hero ===== */
.hero {
  position: relative;
  display: flex;
  gap: 20px;
  align-items: center;
  overflow: hidden;
  padding: 26px 28px;
  border: 1px solid hsl(var(--border));
  border-radius: 14px;
  background: hsl(var(--card));
  animation: rise-in 0.5s ease-out both;
}

.hero-grid {
  position: absolute;
  inset: 0;
  opacity: 0.55;
  background-image: radial-gradient(hsl(var(--primary) / 16%) 1px, transparent 1px);
  background-size: 22px 22px;
  mask-image: linear-gradient(90deg, transparent, #000 28%, #000 72%, transparent);
  pointer-events: none;
}

.hero-primary {
  border-color: hsl(var(--primary) / 30%);
  background: linear-gradient(120deg, hsl(var(--card)), hsl(var(--primary) / 8%));
}
.hero-success {
  border-color: hsl(var(--success) / 30%);
  background: linear-gradient(120deg, hsl(var(--card)), hsl(var(--success) / 9%));
}
.hero-danger {
  border-color: hsl(var(--destructive) / 30%);
  background: linear-gradient(120deg, hsl(var(--card)), hsl(var(--destructive) / 8%));
}
.hero-default {
  background: linear-gradient(120deg, hsl(var(--card)), hsl(var(--muted) / 40%));
}

.hero-icon {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border: 1px solid hsl(var(--border));
  border-radius: 14px;
}

.hero-icon svg {
  width: 26px;
  height: 26px;
}

.tone-primary {
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 10%);
  border-color: hsl(var(--primary) / 30%);
}
.tone-success {
  color: hsl(var(--success));
  background: hsl(var(--success) / 12%);
  border-color: hsl(var(--success) / 35%);
}
.tone-danger {
  color: hsl(var(--destructive));
  background: hsl(var(--destructive) / 10%);
  border-color: hsl(var(--destructive) / 35%);
}
.tone-default {
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted) / 55%);
}

.hero-main {
  position: relative;
  flex: 1;
  min-width: 0;
}

.hero-title-row {
  display: flex;
  gap: 10px;
  align-items: center;
}

.hero-title {
  margin: 0;
  font-size: 19px;
  font-weight: 700;
  color: hsl(var(--foreground));
}

.hero-desc {
  margin: 6px 0 0;
  font-size: 13px;
  line-height: 1.6;
  color: hsl(var(--muted-foreground));
}

.hero-meta {
  margin-top: 8px;
  font-size: 12px;
  color: hsl(var(--muted-foreground) / 85%);
}

/* 在途流程引导：明确「先撤回再重新发起」的路径 */
.hero-warn {
  margin: 4px 0 0;
  font-size: 12px;
  color: hsl(var(--primary));
}

.hero-action {
  position: relative;
  flex: none;
}

.hero-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: flex-end;
}

/* 刷新进度：线框 + 浅色背景，hover 图标旋转 */
.hero-actions .ant-btn {
  border-radius: 8px;
}

.hero-refresh {
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  box-shadow: 0 1px 2px hsl(var(--foreground) / 6%);
  transition: all 0.25s ease;
}

.hero-refresh:hover {
  border-color: hsl(var(--primary) / 45%);
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 5%);
}

.hero-refresh .refresh-icon {
  transition: transform 0.6s ease;
}

.hero-refresh:hover .refresh-icon {
  transform: rotate(180deg);
}

.refresh-icon {
  width: 14px;
  height: 14px;
  margin-right: 4px;
  vertical-align: -2px;
}

/* ===== 提交前准备清单 ===== */
.prep-progress {
  flex: none;
  width: 180px;
  text-align: right;
}

.prep-progress-text {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
  font-variant-numeric: tabular-nums;
}

.prep-ready {
  margin-left: 6px;
  padding: 1px 8px;
  font-size: 11px;
  font-weight: 500;
  color: hsl(var(--success));
  background: hsl(var(--success) / 12%);
  border-radius: 999px;
}

.prep-bar {
  height: 5px;
  margin-top: 7px;
  overflow: hidden;
  background: hsl(var(--muted) / 70%);
  border-radius: 999px;
}

.prep-bar-fill {
  height: 100%;
  background: hsl(var(--primary));
  border-radius: 999px;
  transition: width 0.5s ease;
}

.prep-bar-fill.full {
  background: hsl(var(--success));
}

.prep-grid {
  display: grid;
  /* minmax(0,1fr)：轨道可收缩到内容最小宽度以下，防止 4 张卡片合计撑破面板 */
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  min-width: 0;
}

.prep-item {
  display: flex;
  min-width: 0;
  /* 硬裁切兜底：内部任何内容都不允许画出卡片圆角边界 */
  overflow: hidden;
  gap: 10px;
  align-items: center;
  padding: 12px 14px;
  cursor: pointer;
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  background: hsl(var(--card));
  transition:
    border-color 0.25s ease,
    background 0.25s ease,
    transform 0.25s ease,
    box-shadow 0.25s ease;
}

.prep-item:hover {
  border-color: hsl(var(--primary) / 45%);
  background: hsl(var(--primary) / 4%);
  transform: translateY(-2px);
  box-shadow: 0 6px 18px hsl(var(--primary) / 8%);
}

.prep-item.done:hover {
  border-color: hsl(var(--success) / 45%);
  background: hsl(var(--success) / 4%);
}

.prep-num {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border: 1px solid hsl(var(--border));
  border-radius: 9px;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted) / 40%);
  transition: all 0.25s ease;
}

.prep-num.done {
  color: hsl(var(--success));
  border-color: hsl(var(--success) / 40%);
  background: hsl(var(--success) / 12%);
}

.prep-num svg {
  width: 16px;
  height: 16px;
}

.prep-num-text {
  font-size: 12px;
  font-weight: 600;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.prep-info {
  flex: 1;
  min-width: 0;
}

.prep-name {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.prep-desc {
  margin-top: 2px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.prep-tag {
  /* 允许在极窄容器下适度收缩，避免与 prep-arrow 共同撑破卡片 */
  flex: 0 1 auto;
  min-width: 0;
  max-width: 35%;
  font-size: 11px;
  line-height: 18px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.prep-arrow {
  /* flex-shrink:2：空间不足时箭头先于 Tag 被"挤没"，保证状态标签尽量可见 */
  flex: 0 2 auto;
  min-width: 0;
  overflow: hidden;
  font-size: 14px;
  color: hsl(var(--muted-foreground) / 60%);
  transition: transform 0.25s ease, color 0.25s ease;
}

.prep-item:hover .prep-arrow {
  transform: translateX(3px);
  color: hsl(var(--primary));
}

/* 状态点（沿用实例状态色，用于记录分组头） */
.chip-dot {
  flex: none;
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

/* ===== 审批记录：跨实例聚合折叠分段（新→旧） ===== */
.round-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.round-group {
  overflow: hidden;
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  background: hsl(var(--card));
  transition: border-color 0.25s ease;
}

.round-group:hover {
  border-color: hsl(var(--primary) / 30%);
}

.round-head {
  display: flex;
  gap: 10px;
  align-items: center;
  width: 100%;
  padding: 10px 14px;
  cursor: pointer;
  text-align: left;
  border: none;
  background: transparent;
  transition: background 0.25s ease;
}

.round-head:hover {
  background: hsl(var(--muted) / 45%);
}

.round-name {
  flex: none;
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
  white-space: nowrap;
}

.round-time {
  margin-left: auto;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.round-caret {
  flex: none;
  width: 15px;
  height: 15px;
  color: hsl(var(--muted-foreground));
  transition: transform 0.25s ease;
}

.round-caret.open {
  transform: rotate(180deg);
}

.round-body {
  padding: 2px 14px 12px;
  border-top: 1px dashed hsl(var(--border));
}

/* ===== 审批流水线 ===== */
.pipeline {
  padding: 6px 2px 2px;
}

.p-step {
  display: flex;
  gap: 14px;
}

.p-rail {
  display: flex;
  flex: none;
  flex-direction: column;
  align-items: center;
  width: 34px;
}

.p-dot {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 2px solid hsl(var(--border));
  border-radius: 50%;
  background: hsl(var(--card));
  transition: all 0.3s ease;
}

.p-dot svg {
  width: 15px;
  height: 15px;
}

.p-dot-num {
  font-size: 12px;
  font-weight: 700;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.p-dot-done {
  color: hsl(var(--success));
  border-color: hsl(var(--success) / 55%);
  background: hsl(var(--success) / 12%);
}

.p-dot-active {
  color: hsl(var(--primary));
  border-color: hsl(var(--primary));
  background: hsl(var(--primary) / 10%);
  animation: pulse-ring 1.8s ease-out infinite;
}

.p-dot-err {
  color: hsl(var(--destructive));
  border-color: hsl(var(--destructive) / 55%);
  background: hsl(var(--destructive) / 10%);
}

.p-dot-muted {
  color: hsl(var(--muted-foreground));
  border-color: hsl(var(--border));
  background: hsl(var(--muted) / 40%);
}

.p-line {
  width: 2px;
  min-height: 26px;
  flex: 1;
  margin: 3px 0;
  border-radius: 2px;
  background: hsl(var(--border));
  transition: background 0.4s ease;
}

.p-line-done {
  background: hsl(var(--success) / 55%);
}

.p-line-active {
  background: linear-gradient(180deg, hsl(var(--primary)), hsl(var(--primary) / 30%));
}

.p-body {
  flex: 1;
  min-width: 0;
  padding-bottom: 26px;
}

.p-title {
  display: flex;
  gap: 8px;
  align-items: center;
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.p-sub {
  margin-top: 3px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== 审批记录时间线 ===== */
.log-timeline {
  padding: 4px 6px 0 2px;
}

.log-title {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.log-tag {
  margin-left: 4px;
  margin-inline-end: 0;
}

.log-sub {
  margin-top: 2px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.log-comment {
  margin-top: 6px;
  padding: 8px 12px;
  font-size: 12px;
  line-height: 1.6;
  color: hsl(var(--foreground) / 85%);
  background: hsl(var(--muted) / 50%);
  border-left: 3px solid hsl(var(--primary) / 40%);
  border-radius: 6px;
}

.log-empty {
  padding: 26px 0;
  font-size: 13px;
  text-align: center;
  color: hsl(var(--muted-foreground));
}

/* ===== 撤销审批弹窗 ===== */
.cancel-tip {
  margin-bottom: 12px;
  font-size: 13px;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
}

.empty-panel :deep(.ant-empty-description) {
  font-size: 12px;
}

/* ===== 加载失败态：与「未提交」严格区分，避免误导用户重复排查 ===== */
.error-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 36px 24px;
  text-align: center;
}

.error-icon {
  display: grid;
  place-items: center;
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: hsl(var(--destructive) / 0.1);
  color: hsl(var(--destructive));
}

.error-icon svg {
  width: 22px;
  height: 22px;
}

.error-text {
  max-width: 420px;
  font-size: 13px;
  line-height: 1.8;
  color: hsl(var(--muted-foreground));
}

/* ===== 响应式：窄屏时清单降为 2 列、Hero 纵向堆叠 ===== */
@media (max-width: 1280px) {
  .prep-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 768px) {
  .hero {
    flex-direction: column;
    align-items: flex-start;
    padding: 20px;
  }

  .hero-action {
    width: 100%;
  }

  .hero-action .ant-btn {
    width: 100%;
  }

  .hero-actions {
    width: 100%;
    flex-direction: column;
    align-items: stretch;
  }

  .prep-grid {
    grid-template-columns: 1fr;
  }

  .panel-head {
    flex-direction: column;
  }

  .round-time {
    display: none;
  }

  .prep-progress {
    width: 100%;
    text-align: left;
  }
}
</style>
