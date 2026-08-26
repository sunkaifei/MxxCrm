<script lang="ts" setup>
// 员工入职审批详情（审批工作台 / 提交入职审批 共用组件）
// - 头部识别区：头像 + 姓名 + 提交类型徽章
// - 上次审批意见（驳回/撤回/退回后重新提交场景）
// - 档案完善度：四要素清单进度条（个人中心传入；无数据时隐藏）
// - 员工信息：分组归档（任职信息/联系方式），缺失字段红标「未填写」方便审核
// - 审批流程：实例快照优先（标注节点状态：当前/已通过/已驳回），无实例时展示当前生效流程
// - 审批记录：完整操作时间线（合成提交节点），发起人可撤销
import { computed, ref, watch } from 'vue';

import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Input,
  message,
  Modal,
  Spin,
  Tag,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';

import {
  cancelApprovalApi,
  getApprovalDetailApi,
  getApprovalFlowPreviewApi,
  getHrArchiveDetailApi,
} from '#/api';
import { sortApprovalNodes } from '#/api/core/system/approval';

const props = defineProps<{
  row: any;
  /** 最新审批实例ID（有则加载审批记录/流程快照，可撤销） */
  instanceId?: number;
  /** 流程编码（默认 hire_approval） */
  flowCode?: string;
  /** 档案完善度四要素（个人中心传入；HR 代提交无此数据时隐藏该区） */
  completeness?: { label: string; done: boolean }[];
  /** 当前审批人所在环节（1部门经理 2人事经理 3CEO 4财务；无则发起人/只读视角） */
  stage?: number | null;
}>();

const emit = defineEmits<{
  'cancel-success': [];
}>();

// 默认流程编码：系统内置「员工入职审批」流程
const DEFAULT_FLOW_CODE = 'hire_approval';

const flowLoading = ref(false);
const flowPreview = ref<any>(null);
const detailLoading = ref(false);
const instanceDetail = ref<any>(null);

const flowCode = computed(
  () =>
    props.flowCode ||
    instanceDetail.value?.flowCode ||
    DEFAULT_FLOW_CODE,
);

// 流程节点：实例快照优先（含节点状态），无实例时回退到当前生效流程模板
const flowNodes = computed(() => {
  const snapshot = instanceDetail.value?.flowNodes;
  const edges = instanceDetail.value?.flowEdges;
  if (snapshot?.length) {
    return sortApprovalNodes(snapshot, edges).filter(
      (n: any) => n.nodeType === 2,
    );
  }
  const preview = flowPreview.value || {};
  return sortApprovalNodes(preview.nodes, preview.edges).filter(
    (n: any) => n.nodeType === 2,
  );
});

// 是否为重新提交（最近实例 ∈ {4 驳回,5 撤回,6 退回修改}）
const isResubmit = computed(() =>
  [4, 5, 6].includes(instanceDetail.value?.status),
);

const statusTextMap: Record<number, string> = {
  4: '驳回',
  5: '撤回',
  6: '退回修改',
};

// 审批记录：最新实例完整操作时间线
const logs = computed<any[]>(() => instanceDetail.value?.logs || []);
const lastInstanceStatus = computed<number | undefined>(
  () => instanceDetail.value?.status,
);

const userStore = useUserStore();
const currentUserId = computed(() =>
  Number(userStore.userInfo?.userId ?? userStore.userInfo?.id ?? 0),
);

// ===== 员工完整档案（按审批环节差异化展示：人事经理/发起人可见） =====
const accessStore = useAccessStore();
const canViewArchive = computed(() =>
  accessStore.hasAccessCode('system:hr-archive:view'),
);
const adminId = computed(() => Number(props.row?.id || 0));
const archiveLoading = ref(false);
const hrArchive = ref<any>(null);

async function loadArchive() {
  hrArchive.value = null;
  if (!adminId.value) return;
  if (!canViewArchive.value) return;
  archiveLoading.value = true;
  try {
    const res: any = await getHrArchiveDetailApi(adminId.value);
    hrArchive.value = res?.data?.data ?? res?.data ?? res ?? null;
  } catch {
    hrArchive.value = null;
  } finally {
    archiveLoading.value = false;
  }
}

watch(
  adminId,
  () => {
    if (adminId.value) loadArchive();
  },
  { immediate: true },
);

// 完整档案展示条件：人事经理审批中（stage=2），或个人中心发起人自查（有完善度数据），
// 或当前用户曾处理过人事经理环节（已审批回看场景）
const isSubmitterView = computed(() => (props.completeness?.length || 0) > 0);
const iamProcessedStage2 = computed(() => {
  const d = instanceDetail.value;
  if (!d || d.flowCode !== 'hire_approval') return false;
  return (d.logs || []).some(
    (log: any) =>
      log.nodeKey === 'hr_manager' &&
      Number(log.approverId) === currentUserId.value,
  );
});
const showArchive = computed(
  () =>
    !!hrArchive.value &&
    (props.stage === 2 || isSubmitterView.value || iamProcessedStage2.value),
);
// 紧急联系人/证件信息属隐私：仅人事经理（当前审批或已处理回看）展示
const showEmergency = computed(
  () =>
    !!hrArchive.value &&
    (props.stage === 2 || iamProcessedStage2.value),
);

// 部门/岗位名称：优先完整档案（数组），回退列表行（数组/字符串）
function joinNames(v: any): string {
  if (Array.isArray(v)) return v.filter(Boolean).join('、');
  return v || '';
}
const deptText = computed(
  () =>
    joinNames(hrArchive.value?.deptNames) ||
    joinNames(props.row?.deptNames) ||
    props.row?.deptName ||
    '',
);
const postText = computed(
  () =>
    joinNames(hrArchive.value?.postNames) ||
    joinNames(props.row?.postNames) ||
    props.row?.postName ||
    '',
);

// 履历：教育经历（kind=1）/ 工作经历（kind=2）
const resumeItems = computed<any[]>(() => hrArchive.value?.resume || []);
const eduItems = computed<any[]>(() =>
  resumeItems.value.filter((i: any) => i.kind === 1),
);
const workItems = computed<any[]>(() =>
  resumeItems.value.filter((i: any) => i.kind === 2),
);
const resumeSections = computed(() =>
  [
    { title: '教育经历', items: eduItems.value },
    { title: '工作经历', items: workItems.value },
  ].filter((s) => s.items.length),
);

function dateRange(item: any): string {
  const s = item?.startDate || '';
  const e = item?.endDate || '';
  if (s && e) return `${s} ~ ${e}`;
  if (s) return `${s} 至今`;
  return e ? `至 ${e}` : '—';
}

// 隐私脱敏：手机号 / 身份证 / 银行卡
function maskMobile(v?: string): string {
  if (!v) return '';
  return v.replace(/^(\d{3})\d{4}(\d{4})$/, '$1****$2') || '***';
}
function maskIdCard(v?: string): string {
  if (!v) return '';
  if (v.length < 8) return '***';
  return `${v.slice(0, 6)}********${v.slice(-4)}`;
}
function maskBank(v?: string): string {
  if (!v) return '';
  if (v.length < 8) return '***';
  return `**** **** **** ${v.slice(-4)}`;
}

// 提交节点：审批日志表无「提交」动作，由实例 submittedAt 组合为首条发起记录
const submitLogs = computed<any[]>(() => {
  const detail = instanceDetail.value;
  const arr = logs.value || [];
  if (!detail?.submittedAt) return arr;
  return [
    {
      id: `submit-${detail.id}`,
      action: 0, // 提交
      nodeName: '提交审批',
      approverName:
        detail.submitterName ||
        props.row?.nickName ||
        props.row?.userName ||
        '',
      createTime: detail.submittedAt,
    },
    ...arr,
  ];
});
const restLogs = computed<any[]>(() => {
  const arr = submitLogs.value;
  return arr[0]?.action === 0 ? arr.slice(1) : arr;
});

// ===== 撤销审批（仅发起人本人，且实例进行中 status=1/2） =====
const cancelVisible = ref(false);
const cancelReason = ref('');
const canceling = ref(false);
const canCancel = computed(
  () =>
    [1, 2].includes(instanceDetail.value?.status) &&
    instanceDetail.value?.submitterId === currentUserId.value,
);

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
  canceling.value = true;
  try {
    await cancelApprovalApi({
      instanceId: Number(props.instanceId),
      cancelReason: reason,
    });
    message.success('已撤回，可修改档案后重新提交');
    cancelVisible.value = false;
    await loadDetail();
    emit('cancel-success');
  } catch (error: any) {
    message.error(error?.message || '撤回失败');
  } finally {
    canceling.value = false;
  }
}

// 上次驳回/撤回/退回意见（从最新实例日志取 action=2/7 的 comment/reason）
const lastOpinion = computed(() => {
  const rejectLog = [...logs.value]
    .reverse()
    .find(
      (log: any) =>
        [2, 7].includes(log.action) && (log.comment || log.reason),
    );
  return rejectLog?.comment || rejectLog?.reason || '';
});

// 日志动作映射（与审批引擎一致）
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

// 节点状态标注（实例快照：0=未开始,1=审批中,2=已通过,3=已驳回）
const nodeStatusMap: Record<number, { label: string; color: string }> = {
  0: { label: '未开始', color: 'default' },
  1: { label: '审批中', color: 'processing' },
  2: { label: '已通过', color: 'success' },
  3: { label: '已驳回', color: 'error' },
};

// 我在某节点的处理结果（从日志反查，用于「每级审批都能看到自己处理的结果」）
function myActionOnNode(nodeKey?: string): string {
  if (!nodeKey) return '';
  const mine = [...logs.value]
    .reverse()
    .find(
      (log: any) =>
        log.nodeKey === nodeKey &&
        Number(log.approverId) === currentUserId.value,
    );
  return mine ? logActionText[mine.action] || '' : '';
}

// approverType 展示文本（与审批引擎 approver_type 定义一致）
function approverTypeText(node: any) {
  const map: Record<number, string> = {
    1: '指定成员',
    2: '指定角色',
    3: '部门主管',
    4: '发起人自己',
    5: '指定岗位',
    6: '直属上级',
    7: '部门主管链',
  };
  return map[node.approverType] || '审批人';
}

// 审批模式映射
const approveModeMap: Record<number, string> = {
  1: '或签',
  2: '会签',
  3: '依次审批',
};

async function loadFlowPreview() {
  flowLoading.value = true;
  try {
    const res: any = await getApprovalFlowPreviewApi(flowCode.value);
    flowPreview.value = res?.data?.data ?? res?.data ?? res ?? null;
  } catch {
    flowPreview.value = null;
  } finally {
    flowLoading.value = false;
  }
}

async function loadDetail() {
  instanceDetail.value = null;
  if (!props.instanceId) return;
  detailLoading.value = true;
  try {
    const res: any = await getApprovalDetailApi(props.instanceId);
    instanceDetail.value = res?.data?.data ?? res?.data ?? res ?? null;
  } catch {
    instanceDetail.value = null;
  } finally {
    detailLoading.value = false;
  }
}

watch(
  () => props.instanceId,
  () => {
    if (props.instanceId) {
      loadDetail();
      loadFlowPreview();
    } else {
      instanceDetail.value = null;
      // 无实例时仍展示当前生效流程模板
      loadFlowPreview();
    }
  },
  { immediate: true },
);

// ===== 员工信息归档（缺失字段红标「未填写」，方便审核信息完整性） =====
const infoGroups = computed(() => {
  const r = props.row || {};
  const f = (v: any) =>
    v === undefined || v === null || v === '' ? '' : String(v);
  // 部门/岗位名称取值链：完整档案(deptNames/postNames) → 详情接口数组 → 列表行单数(deptName/postName)
  const pickNames = (archive: any, rowNames: any, single: any) => {
    const src = archive?.length
      ? archive
      : rowNames?.length
        ? rowNames
        : single
          ? [single]
          : [];
    return src
      .filter((v: any) => v !== null && v !== undefined && v !== '')
      .join('、');
  };
  const contactItems = [
    { label: '手机号', value: f(r.mobile) },
    { label: '邮箱', value: f(r.email) },
    { label: '用户名', value: f(r.userName) },
  ];
  if (f(r.roleName)) contactItems.push({ label: '角色', value: f(r.roleName) });
  // 合同信息（决定试用期法定上限，供审批人参考）
  const contractType = r.contractType ?? hrArchive.value?.contractType;
  const contractMonths = r.contractMonths ?? hrArchive.value?.contractMonths;
  return [
    {
      title: '任职信息',
      items: [
        {
          label: '部门',
          value: pickNames(hrArchive.value?.deptNames, r.deptNames, r.deptName),
        },
        {
          label: '岗位',
          value: pickNames(hrArchive.value?.postNames, r.postNames, r.postName),
        },
        { label: '直属上级', value: f(r.directManagerName) },
        { label: '入职时间', value: f(r.hireDate) },
        {
          label: '合同类型',
          value:
            contractType === 1
              ? '固定期限'
              : contractType === 2
                ? '无固定期限'
                : '',
        },
        {
          label: '合同期限（月）',
          value: f(contractMonths),
        },
      ],
    },
    { title: '联系方式', items: contactItems },
  ];
});

// ===== 入职定薪：薪资带宽参照 + 各环节定薪数据 =====
// 带宽参照（岗位薪资带宽，来自 mxx_hr_salary_band）
const salaryBand = computed(() => instanceDetail.value?.salaryBand || null);

// 各环节定薪数据（来自 mxx_hr_hire_salary_data，按 stage 正序）
const salaryStages = computed<any[]>(
  () => instanceDetail.value?.hireSalaryStages || [],
);

// 环节元信息：stage → 节点名 / 图标序号
const stageMeta: Record<number, { label: string; icon: string }> = {
  1: { label: '部门经理审批', icon: '①' },
  2: { label: '人事经理审批', icon: '②' },
  3: { label: 'CEO终审', icon: '③' },
  4: { label: '财务定薪录入', icon: '④' },
};

// 带宽评估结果文案
function bandStatusText(s: any): { label: string; color: string } {
  if (s?.bandStatus === 1) return { label: '带宽内', color: 'success' };
  if (s?.bandStatus === 2) return { label: '超带宽（转CEO特批）', color: 'warning' };
  return { label: '未评估', color: 'default' };
}

// 金额格式化：如 4500 / 4500.00 → 去掉多余小数
function moneyText(v: any): string {
  if (v === undefined || v === null || v === '') return '';
  const n = Number(v);
  if (Number.isNaN(n)) return String(v);
  return Number.isInteger(n) ? String(n) : n.toFixed(2);
}

// 环节是否已填写（用于高亮已处理环节）
function stageFilled(s: any): boolean {
  if (!s) return false;
  return [
    s.suggestedSalary,
    s.probationMonths,
    s.bandStatus,
    s.bandReason,
    s.probationRatio,
    s.ceoOpinion,
    s.finalSalary,
    s.comment,
  ].some((v) => v !== undefined && v !== null && v !== '');
}

// 定薪档案结果（财务录入后生成）
const finalResult = computed(() => {
  const finance = salaryStages.value.find((s: any) => s.stage === 4);
  const ceo = salaryStages.value.find((s: any) => s.stage === 3);
  return {
    finalSalary: moneyText(finance?.finalSalary),
    effectiveDate: finance?.effectiveDate || '',
    bandStatus: finance?.bandStatus,
    ceoOpinion: ceo?.ceoOpinion || '',
  };
});

// 档案完善度（个人中心传入；HR 代提交无数据时隐藏）
const completenessItems = computed(() => props.completeness || []);
const completenessDone = computed(
  () => completenessItems.value.filter((i) => i.done).length,
);
const completenessPct = computed(() => {
  const total = completenessItems.value.length || 1;
  return `${(completenessDone.value / total) * 100}%`;
});

// 头部识别区
const headName = computed(
  () => props.row?.nickName || props.row?.userName || '员工',
);
const headSub = computed(() => {
  return (
    [props.row?.userName, deptText.value, postText.value]
      .filter(Boolean)
      .join(' · ') || '—'
  );
});
</script>

<template>
  <div class="flex-1 overflow-y-auto space-y-4">
    <!-- ===== 头部识别区 ===== -->
    <div class="su-head">
      <div class="su-avatar">{{ headName.charAt(0) }}</div>
      <div class="su-head-main">
        <div class="su-head-name-row">
          <span class="su-head-name">{{ headName }}</span>
          <Tag
            :color="isResubmit ? 'orange' : 'geekblue'"
            class="su-head-tag"
          >
            {{ isResubmit ? '重新提交' : '首次提交' }}
          </Tag>
        </div>
        <div class="su-head-sub">{{ headSub }}</div>
      </div>
    </div>

    <!-- ===== 上次驳回/撤回/退回意见（重新提交场景） ===== -->
    <div v-if="isResubmit" class="su-card su-card-reject">
      <div class="su-card-head">
        <span class="su-card-title su-title-danger">上次审批意见</span>
        <span class="su-card-extra">
          {{
            lastInstanceStatus !== undefined
              ? `审批已被${statusTextMap[lastInstanceStatus] || '退回'}`
              : '审批未通过'
          }}
        </span>
      </div>
      <div class="su-reject-body">{{ lastOpinion || '（无意见记录）' }}</div>
    </div>

    <!-- ===== 档案完善度（个人中心场景） ===== -->
    <div v-if="completenessItems.length" class="su-card">
      <div class="su-card-head">
        <span class="su-card-title">档案完善度</span>
        <span class="su-card-extra">
          {{ completenessDone }} / {{ completenessItems.length }} 项已完善
        </span>
      </div>
      <div class="su-bar">
        <div
          class="su-bar-fill"
          :class="{ ok: completenessDone === completenessItems.length }"
          :style="{ width: completenessPct }"
        ></div>
      </div>
      <div class="su-check-grid">
        <div
          v-for="item in completenessItems"
          :key="item.label"
          class="su-check"
          :class="{ ok: item.done }"
        >
          <span class="su-check-icon">
            <svg
              v-if="item.done"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M20 6L9 17l-5-5" />
            </svg>
            <svg
              v-else
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </span>
          <span class="su-check-label">{{ item.label }}</span>
          <Tag
            :color="item.done ? 'success' : 'warning'"
            size="small"
            class="su-check-tag"
          >
            {{ item.done ? '已完善' : '待完善' }}
          </Tag>
        </div>
      </div>
    </div>

    <!-- ===== 员工信息归档 ===== -->
    <div class="su-card">
      <div class="su-card-head">
        <span class="su-card-title">员工信息</span>
        <span class="su-card-extra">审核请以档案系统为准</span>
      </div>
      <div v-for="group in infoGroups" :key="group.title" class="su-group">
        <div class="su-group-title">{{ group.title }}</div>
        <div class="su-group-grid">
          <div v-for="it in group.items" :key="it.label" class="su-field">
            <div class="su-field-label">{{ it.label }}</div>
            <div class="su-field-value" :class="{ empty: !it.value }">
              {{ it.value || '未填写' }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ===== 员工完整档案（按审批环节差异化：人事经理审批/发起人自查可见） ===== -->
    <div v-if="showArchive" class="su-card">
      <div class="su-card-head">
        <span class="su-card-title">员工完整档案</span>
        <span class="su-card-extra">
          <Tag
            v-if="showEmergency"
            color="geekblue"
            size="small"
            class="su-log-tag"
            >人事经理审核</Tag
          >
          <span v-else>发起人自查</span>
        </span>
      </div>
      <Spin :spinning="archiveLoading">
        <!-- 履历：教育经历 / 工作经历 -->
        <template v-for="sec in resumeSections" :key="sec.title">
          <div class="su-arch-sec">
            <div class="su-group-title">{{ sec.title }}</div>
            <div class="su-arch-list">
              <div v-for="item in sec.items" :key="item.id" class="su-arch-item">
                <div
                  class="su-arch-dot"
                  :class="sec.title === '教育经历' ? 'edu' : 'work'"
                ></div>
                <div class="su-arch-main">
                  <div class="su-arch-title">{{ item.title || '未填写' }}</div>
                  <div v-if="item.org" class="su-arch-org">{{ item.org }}</div>
                  <div class="su-arch-date">{{ dateRange(item) }}</div>
                  <div v-if="item.remark" class="su-arch-remark">
                    {{ item.remark }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </template>

        <!-- 紧急联系人（仅人事经理） -->
        <div v-if="showEmergency" class="su-arch-sec">
          <div class="su-group-title">紧急联系人</div>
          <div
            v-if="hrArchive?.emergencyContacts?.length"
            class="su-arch-grid"
          >
            <div
              v-for="c in hrArchive.emergencyContacts"
              :key="c.id"
              class="su-arch-contact"
            >
              <div class="su-arch-contact-name">{{ c.name }}</div>
              <div class="su-arch-contact-rel">{{ c.relation || '—' }}</div>
              <div class="su-arch-contact-mobile">
                {{ maskMobile(c.mobile) || '未填写' }}
              </div>
            </div>
          </div>
          <div v-else class="su-empty-tip">暂无紧急联系人</div>
        </div>

        <!-- 证件信息（仅人事经理，脱敏展示） -->
        <div v-if="showEmergency" class="su-arch-sec">
          <div class="su-group-title">证件信息</div>
          <div class="su-group-grid">
            <div class="su-field">
              <div class="su-field-label">身份证号</div>
              <div
                class="su-field-value"
                :class="{ empty: !hrArchive?.idCardNo }"
              >
                {{ maskIdCard(hrArchive?.idCardNo) || '未填写' }}
              </div>
            </div>
            <div class="su-field">
              <div class="su-field-label">银行卡号</div>
              <div
                class="su-field-value"
                :class="{ empty: !hrArchive?.bankCardNo }"
              >
                {{ maskBank(hrArchive?.bankCardNo) || '未填写' }}
              </div>
            </div>
          </div>
        </div>
      </Spin>
    </div>

    <!-- ===== 入职定薪方案（带宽参照 + 各环节填写/意见/结果） ===== -->
    <div v-if="salaryStages.length" class="su-card">
      <div class="su-card-head">
        <span class="su-card-title">定薪方案</span>
        <span class="su-card-extra">各审批环节填写内容 · 全程留痕</span>
      </div>

      <!-- 带宽参照条 -->
      <div class="su-band-ref">
        <div class="su-band-post">
          <span class="su-band-post-icon">薪</span>
          <div class="su-band-post-main">
            <div class="su-band-post-name">
              {{ salaryBand?.postName || row?.postName || '岗位' }}薪资带宽
            </div>
            <div class="su-band-range">
              <span class="su-band-num">{{
                moneyText(salaryBand?.minSalary) || '未配置'
              }}</span>
              <span class="su-band-tilde">~</span>
              <span class="su-band-num">{{
                moneyText(salaryBand?.maxSalary) || '—'
              }}</span>
              <span class="su-band-unit">元/月</span>
            </div>
          </div>
        </div>
        <!-- 最终定薪结果 -->
        <div v-if="finalResult.finalSalary" class="su-band-result">
          <span class="su-band-result-label">最终定薪</span>
          <span class="su-band-result-num">{{ finalResult.finalSalary }}</span>
          <span class="su-band-unit">元/月</span>
          <span
            v-if="finalResult.effectiveDate"
            class="su-band-result-date"
          >
            {{ finalResult.effectiveDate }} 生效
          </span>
        </div>
      </div>

      <!-- 各环节定薪数据 -->
      <div class="su-stages">
        <div
          v-for="s in salaryStages"
          :key="s.nodeKey || s.stage"
          class="su-stage"
          :class="{ pending: !stageFilled(s) }"
        >
          <div class="su-stage-head">
            <span class="su-stage-icon">{{
              stageMeta[s.stage]?.icon || '•'
            }}</span>
            <span class="su-stage-name">{{
              stageMeta[s.stage]?.label || s.nodeKey
            }}</span>
            <Tag
              :color="stageFilled(s) ? 'green' : 'default'"
              size="small"
              class="su-stage-tag"
            >
              {{ stageFilled(s) ? '已填写' : '未填写' }}
            </Tag>
          </div>

          <div v-if="stageFilled(s)" class="su-stage-body">
            <!-- ① 部门经理：能力评估 + 建议工资 + 试用期 -->
            <div v-if="s.stage === 1" class="su-stage-grid">
              <div v-if="s.abilityAssessment" class="su-stage-item wide">
                <span class="su-stage-k">工作能力评估</span>
                <span class="su-stage-v">{{ s.abilityAssessment }}</span>
              </div>
              <div
                v-if="
                  s.suggestedSalary !== null && s.suggestedSalary !== undefined
                "
                class="su-stage-item"
              >
                <span class="su-stage-k">建议工资</span>
                <span class="su-stage-v strong">
                  {{ moneyText(s.suggestedSalary) }} 元/月
                </span>
              </div>
              <div
                v-if="
                  s.probationMonths !== null && s.probationMonths !== undefined
                "
                class="su-stage-item"
              >
                <span class="su-stage-k">试用期</span>
                <span class="su-stage-v strong">
                  {{ s.probationMonths }} 个月
                </span>
              </div>
            </div>

            <!-- ② 人事经理：谈定工资 + 带宽评估 + 试用期比例 + 超带宽原因 -->
            <div v-if="s.stage === 2" class="su-stage-grid">
              <div
                v-if="
                  s.negotiatedSalary !== null &&
                  s.negotiatedSalary !== undefined
                "
                class="su-stage-item"
              >
                <span class="su-stage-k">谈定工资</span>
                <span class="su-stage-v strong">
                  {{ moneyText(s.negotiatedSalary) }} 元/月
                </span>
              </div>
              <div
                v-if="
                  s.bandStatus !== null && s.bandStatus !== undefined
                "
                class="su-stage-item"
              >
                <span class="su-stage-k">带宽评估</span>
                <span class="su-stage-v strong">
                  <Tag :color="bandStatusText(s).color" size="small">{{
                    bandStatusText(s).label
                  }}</Tag>
                </span>
              </div>
              <div
                v-if="
                  s.probationRatio !== null && s.probationRatio !== undefined
                "
                class="su-stage-item"
              >
                <span class="su-stage-k">试用期工资比例</span>
                <span class="su-stage-v strong">
                  {{ Math.round(Number(s.probationRatio) * 100) }}%
                </span>
              </div>
              <div v-if="s.bandReason" class="su-stage-item wide">
                <span class="su-stage-k">超带宽原因</span>
                <span class="su-stage-v">{{ s.bandReason }}</span>
              </div>
            </div>

            <!-- ③ CEO终审：特批意见 -->
            <div v-if="s.stage === 3" class="su-stage-grid">
              <div v-if="s.ceoOpinion" class="su-stage-item wide">
                <span class="su-stage-k">终审意见</span>
                <span class="su-stage-v">{{ s.ceoOpinion }}</span>
              </div>
            </div>

            <!-- ④ 财务定薪录入：最终定薪 + 生效日期 -->
            <div v-if="s.stage === 4" class="su-stage-grid">
              <div
                v-if="
                  s.finalSalary !== null && s.finalSalary !== undefined
                "
                class="su-stage-item"
              >
                <span class="su-stage-k">最终定薪</span>
                <span class="su-stage-v strong">
                  {{ moneyText(s.finalSalary) }} 元/月
                </span>
              </div>
              <div v-if="s.effectiveDate" class="su-stage-item">
                <span class="su-stage-k">生效日期</span>
                <span class="su-stage-v strong">{{ s.effectiveDate }}</span>
              </div>
            </div>

            <!-- 通用：审批意见 -->
            <div v-if="s.comment" class="su-stage-item wide">
              <span class="su-stage-k">审批意见</span>
              <span class="su-stage-v">{{ s.comment }}</span>
            </div>
            <div v-if="s.approverName || s.createTime" class="su-stage-meta">
              {{ s.approverName || '-' }} ·
              {{ formatDateTime(s.createTime) }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ===== 审批流程 ===== -->
    <div class="su-card">
      <div class="su-card-head">
        <span class="su-card-title">审批流程</span>
        <span class="su-card-extra">
          {{ flowPreview?.flowName || instanceDetail?.flowName || flowCode }}
        </span>
      </div>
      <Spin :spinning="flowLoading">
        <div v-if="flowNodes.length" class="su-flow">
          <!-- 发起人节点 -->
          <div class="su-flow-node su-flow-start">
            <div class="su-flow-node-title">发起人</div>
            <div class="su-flow-node-sub">
              {{ row?.nickName || row?.userName || '-' }}
            </div>
          </div>
          <span class="su-flow-link">→</span>

          <!-- 审批节点 -->
          <template v-for="(node, idx) in flowNodes" :key="node.nodeKey">
            <div
              class="su-flow-node"
              :class="[
                node.nodeStatus === 1 ? 'su-flow-active' : '',
                node.nodeStatus === 2 ? 'su-flow-done' : '',
                node.nodeStatus === 3 ? 'su-flow-rejected' : '',
              ]"
            >
              <div class="su-flow-node-title">{{ node.nodeName }}</div>
              <div class="su-flow-node-sub">
                {{ node.approverTypeDesc || approverTypeText(node) }}
              </div>
              <div class="su-flow-node-tags">
                <Tag
                  v-if="node.nodeStatus !== undefined"
                  size="small"
                  :color="
                    (nodeStatusMap[node.nodeStatus] || {}).color || 'default'
                  "
                  class="su-flow-node-tag"
                >
                  {{ (nodeStatusMap[node.nodeStatus] || {}).label || '未开始' }}
                </Tag>
                <span
                  v-if="myActionOnNode(node.nodeKey)"
                  class="su-flow-mine"
                >
                  我：{{ myActionOnNode(node.nodeKey) }}
                </span>
                <Tag
                  v-if="
                    node.nodeStatus === undefined &&
                    node.approveMode &&
                    approveModeMap[node.approveMode]
                  "
                  size="small"
                  :color="
                    node.approveMode === 2
                      ? 'purple'
                      : node.approveMode === 3
                        ? 'orange'
                        : 'blue'
                  "
                  class="su-flow-node-tag"
                >
                  {{ approveModeMap[node.approveMode] }}
                </Tag>
              </div>
            </div>
            <span
              v-if="idx < flowNodes.length - 1"
              class="su-flow-link"
              >→</span
            >
          </template>

          <!-- 结束节点 -->
          <template v-if="flowNodes.length">
            <span class="su-flow-link">→</span>
            <div class="su-flow-node su-flow-end">
              <div class="su-flow-node-title">结束</div>
              <div class="su-flow-node-sub">流程完成</div>
            </div>
          </template>
        </div>
        <div v-else class="su-empty-tip">
          流程模板未配置或已停用，请联系管理员
        </div>
      </Spin>
    </div>

    <!-- ===== 审批记录（最新实例时间线） ===== -->
    <div class="su-card">
      <div class="su-card-head">
        <span class="su-card-title">审批记录</span>
        <div class="su-card-head-actions">
          <Button
            v-if="canCancel"
            danger
            type="text"
            size="small"
            @click="openCancel"
          >
            撤销审批
          </Button>
          <span class="su-card-extra">{{ submitLogs.length }} 条</span>
        </div>
      </div>
      <Spin :spinning="detailLoading">
        <div v-if="submitLogs.length" class="su-log">
          <Timeline>
            <TimelineItem
              v-if="submitLogs[0]?.action === 0"
              key="submit-entry"
              color="blue"
            >
              <div class="su-log-title">
                {{ submitLogs[0].nodeName }}
                <Tag color="geekblue" size="small" class="su-log-tag"
                  >发起</Tag
                >
              </div>
              <div class="su-log-sub">
                {{ submitLogs[0].approverName }} · 提交 ·
                {{ formatDateTime(submitLogs[0].createTime) }}
              </div>
            </TimelineItem>
            <TimelineItem
              v-for="log in restLogs"
              :key="log.id"
              :color="logActionColor[log.action] || 'blue'"
            >
              <div class="su-log-title">
                {{ log.nodeName || logActionText[log.action] || '审批' }}
              </div>
              <div class="su-log-sub">
                {{ log.approverName || log.operatorName || '-' }} ·
                {{ logActionText[log.action] || '--' }} ·
                {{ formatDateTime(log.createTime || log.create_at) }}
              </div>
              <div v-if="log.comment || log.reason" class="su-log-comment">
                {{ log.comment || log.reason }}
              </div>
            </TimelineItem>
          </Timeline>
        </div>
        <div v-else class="su-empty-tip">
          {{
            isResubmit
              ? '暂无审批记录'
              : '尚无审批记录，提交后将在此展示流转过程'
          }}
        </div>
      </Spin>
    </div>

    <!-- ===== 撤销审批弹窗（发起人撤回，理由必填） ===== -->
    <Modal
      v-model:open="cancelVisible"
      title="撤销审批"
      ok-text="确认撤回"
      cancel-text="暂不"
      :confirm-loading="canceling"
      @ok="handleCancel"
    >
      <p class="su-cancel-tip">
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
/* ===== 头部识别区 ===== */
.su-head {
  display: flex;
  gap: 14px;
  align-items: center;
  padding-bottom: 16px;
  border-bottom: 1px dashed hsl(var(--border));
}

.su-avatar {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  font-size: 18px;
  font-weight: 700;
  color: hsl(var(--primary-foreground));
  background: linear-gradient(
    135deg,
    hsl(var(--primary)),
    hsl(var(--primary) / 70%)
  );
  border-radius: 12px;
}

.su-head-main {
  flex: 1;
  min-width: 0;
}

.su-head-name-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.su-head-name {
  font-size: 17px;
  font-weight: 700;
  color: hsl(var(--foreground));
}

.su-head-tag {
  margin-inline-end: 0;
}

.su-head-sub {
  margin-top: 3px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ===== 卡片通用 ===== */
.su-card {
  padding: 16px 18px;
  border: 1px solid hsl(var(--border));
  border-radius: 12px;
  background: hsl(var(--card));
  animation: su-rise 0.4s ease-out both;
}

.su-card:nth-child(2) {
  animation-delay: 0.05s;
}
.su-card:nth-child(3) {
  animation-delay: 0.1s;
}
.su-card:nth-child(4) {
  animation-delay: 0.15s;
}
.su-card:nth-child(5) {
  animation-delay: 0.2s;
}

@keyframes su-rise {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

.su-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.su-card-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.su-card-extra {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.su-card-head-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.su-title-danger {
  color: hsl(var(--destructive));
}

.su-card-reject {
  border-color: hsl(var(--destructive) / 35%);
  background: hsl(var(--destructive) / 5%);
}

.su-reject-body {
  padding: 10px 12px;
  font-size: 13px;
  line-height: 1.7;
  color: hsl(var(--foreground) / 85%);
  background: hsl(var(--card));
  border-left: 3px solid hsl(var(--destructive) / 50%);
  border-radius: 6px;
}

/* ===== 档案完善度 ===== */
.su-bar {
  height: 6px;
  margin-bottom: 12px;
  overflow: hidden;
  background: hsl(var(--muted) / 70%);
  border-radius: 999px;
}

.su-bar-fill {
  height: 100%;
  background: hsl(var(--primary));
  border-radius: 999px;
  transition: width 0.5s ease;
}

.su-bar-fill.ok {
  background: hsl(var(--success));
}

.su-check-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}

.su-check {
  display: flex;
  min-width: 0;
  gap: 6px;
  align-items: center;
  padding: 8px 10px;
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  background: hsl(var(--muted) / 30%);
}

.su-check.ok {
  border-color: hsl(var(--success) / 35%);
  background: hsl(var(--success) / 7%);
}

.su-check-icon {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  color: hsl(var(--destructive));
  border-radius: 50%;
  background: hsl(var(--destructive) / 12%);
}

.su-check.ok .su-check-icon {
  color: hsl(var(--success));
  background: hsl(var(--success) / 14%);
}

.su-check-icon svg {
  width: 10px;
  height: 10px;
}

.su-check-label {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  font-weight: 500;
  color: hsl(var(--foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.su-check-tag {
  flex: none;
  margin-inline-end: 0;
}

/* ===== 员工信息归档 ===== */
.su-group + .su-group {
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px dashed hsl(var(--border));
}

.su-group-title {
  margin-bottom: 10px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: hsl(var(--primary));
}

.su-group-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px 16px;
}

.su-field {
  min-width: 0;
}

.su-field-label {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.su-field-value {
  margin-top: 2px;
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.su-field-value.empty {
  color: hsl(var(--destructive));
  font-weight: 400;
}

/* ===== 审批流程横向流水线 ===== */
.su-flow {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  padding: 6px 4px 4px;
  overflow-x: auto;
}

.su-flow-node {
  flex: none;
  min-width: 118px;
  padding: 10px 12px;
  text-align: center;
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  background: hsl(var(--card));
}

.su-flow-start {
  border-color: hsl(var(--primary) / 45%);
  background: hsl(var(--primary) / 8%);
}

.su-flow-end {
  border-color: hsl(var(--border));
  background: hsl(var(--muted) / 40%);
}

/* 节点状态：审批中 / 已通过 / 已驳回 */
.su-flow-active {
  border-color: hsl(var(--primary));
  background: hsl(var(--primary) / 10%);
  box-shadow: 0 0 0 1px hsl(var(--primary) / 30%);
}

.su-flow-done {
  border-color: hsl(var(--success) / 50%);
  background: hsl(var(--success) / 8%);
}

.su-flow-rejected {
  border-color: hsl(var(--destructive) / 55%);
  background: hsl(var(--destructive) / 8%);
}

.su-flow-node-title {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
  white-space: nowrap;
}

.su-flow-start .su-flow-node-title {
  color: hsl(var(--primary));
}

.su-flow-node-sub {
  margin-top: 3px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.su-flow-node-tags {
  display: flex;
  gap: 4px;
  align-items: center;
  justify-content: center;
  margin-top: 4px;
}

.su-flow-node-tag {
  margin-inline-end: 0;
}

.su-flow-mine {
  font-size: 11px;
  font-weight: 600;
  color: hsl(var(--primary));
  white-space: nowrap;
}

.su-flow-link {
  flex: none;
  padding: 0 6px;
  font-size: 15px;
  color: hsl(var(--muted-foreground) / 60%);
}

/* ===== 审批记录时间线 ===== */
.su-log {
  padding: 2px 4px 0;
}

.su-log-title {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.su-log-tag {
  margin-left: 4px;
  margin-inline-end: 0;
}

.su-log-sub {
  margin-top: 2px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.su-log-comment {
  margin-top: 6px;
  padding: 8px 12px;
  font-size: 12px;
  line-height: 1.6;
  color: hsl(var(--foreground) / 85%);
  background: hsl(var(--muted) / 50%);
  border-left: 3px solid hsl(var(--primary) / 40%);
  border-radius: 6px;
}

.su-empty-tip {
  padding: 20px 0;
  font-size: 12px;
  text-align: center;
  color: hsl(var(--muted-foreground));
}

/* ===== 撤销审批弹窗 ===== */
.su-cancel-tip {
  margin-bottom: 12px;
  font-size: 12px;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
}

/* ===== 入职定薪：带宽参照条 ===== */
.su-band-ref {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  margin-bottom: 14px;
  background: linear-gradient(
    135deg,
    hsl(var(--primary) / 10%),
    hsl(var(--primary) / 3%)
  );
  border: 1px solid hsl(var(--primary) / 25%);
  border-radius: 10px;
}

.su-band-post {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.su-band-post-icon {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  font-size: 14px;
  font-weight: 700;
  color: hsl(var(--primary-foreground));
  background: hsl(var(--primary));
  border-radius: 9px;
}

.su-band-post-main {
  min-width: 0;
}

.su-band-post-name {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.su-band-range {
  display: flex;
  align-items: baseline;
  gap: 4px;
  margin-top: 2px;
}

.su-band-num {
  font-size: 15px;
  font-weight: 700;
  color: hsl(var(--primary));
}

.su-band-tilde {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.su-band-unit {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.su-band-result {
  flex: none;
  padding: 8px 14px;
  text-align: right;
  background: hsl(var(--success) / 10%);
  border: 1px solid hsl(var(--success) / 30%);
  border-radius: 10px;
}

.su-band-result-label {
  display: block;
  font-size: 11px;
  color: hsl(var(--success));
}

.su-band-result-num {
  margin-left: 2px;
  font-size: 18px;
  font-weight: 700;
  color: hsl(var(--success));
}

.su-band-result-date {
  display: block;
  margin-top: 2px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

/* ===== 入职定薪：各环节纵向时间线 ===== */
.su-stages {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.su-stage {
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  background: hsl(var(--card));
}

.su-stage.pending {
  border-style: dashed;
  background: hsl(var(--muted) / 25%);
}

.su-stage-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
}

.su-stage:not(.pending) .su-stage-head {
  border-bottom: 1px dashed hsl(var(--border));
}

.su-stage-icon {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  font-size: 12px;
  font-weight: 700;
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 12%);
  border-radius: 6px;
}

.su-stage.pending .su-stage-icon {
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted) / 60%);
}

.su-stage-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.su-stage-tag {
  flex: none;
  margin-inline-end: 0;
}

.su-stage-body {
  padding: 10px 12px 12px;
}

.su-stage-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px 16px;
}

.su-stage-item {
  min-width: 0;
}

.su-stage-item.wide {
  grid-column: 1 / -1;
  margin-top: 6px;
  padding: 8px 10px;
  background: hsl(var(--muted) / 40%);
  border-radius: 8px;
}

.su-stage-k {
  display: block;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.su-stage-v {
  display: block;
  margin-top: 2px;
  font-size: 13px;
  line-height: 1.6;
  color: hsl(var(--foreground) / 85%);
  word-break: break-all;
}

.su-stage-v.strong {
  font-weight: 600;
  color: hsl(var(--foreground));
}

.su-stage-meta {
  margin-top: 8px;
  padding-top: 8px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  border-top: 1px dashed hsl(var(--border));
}

/* ===== 员工完整档案：履历 / 紧急联系人 / 证件 ===== */
.su-arch-sec + .su-arch-sec {
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px dashed hsl(var(--border));
}

.su-arch-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.su-arch-item {
  display: flex;
  gap: 10px;
  min-width: 0;
  padding: 10px 12px;
  background: hsl(var(--muted) / 25%);
  border: 1px solid hsl(var(--border) / 80%);
  border-radius: 10px;
}

.su-arch-dot {
  flex: none;
  width: 8px;
  height: 8px;
  margin-top: 6px;
  background: hsl(var(--success));
  border-radius: 50%;
  box-shadow: 0 0 0 3px hsl(var(--success) / 15%);
}

.su-arch-dot.edu {
  background: hsl(var(--primary));
  box-shadow: 0 0 0 3px hsl(var(--primary) / 15%);
}

.su-arch-main {
  flex: 1;
  min-width: 0;
}

.su-arch-title {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.su-arch-org {
  margin-top: 2px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.su-arch-date {
  margin-top: 2px;
  font-size: 11px;
  font-weight: 500;
  color: hsl(var(--primary) / 80%);
}

.su-arch-remark {
  margin-top: 6px;
  padding: 6px 10px;
  font-size: 12px;
  line-height: 1.6;
  color: hsl(var(--foreground) / 80%);
  background: hsl(var(--card));
  border-left: 3px solid hsl(var(--primary) / 35%);
  border-radius: 6px;
}

.su-arch-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.su-arch-contact {
  min-width: 0;
  padding: 10px 12px;
  text-align: center;
  background: hsl(var(--muted) / 25%);
  border: 1px solid hsl(var(--border) / 80%);
  border-radius: 10px;
}

.su-arch-contact-name {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.su-arch-contact-rel {
  margin-top: 2px;
  font-size: 11px;
  color: hsl(var(--primary) / 80%);
}

.su-arch-contact-mobile {
  margin-top: 4px;
  font-size: 12px;
  letter-spacing: 0.5px;
  color: hsl(var(--muted-foreground));
}

@media (max-width: 768px) {
  .su-arch-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
