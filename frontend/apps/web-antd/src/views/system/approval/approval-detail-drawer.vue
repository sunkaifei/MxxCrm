<script lang="ts" setup>
// 审批详情抽屉（审批工作台 todo 页共用）
// - 通用业务摘要：类型/标题/提交人/时间/状态/候选审批人
// - 业务详情：按 business_type 分发（user/hire → 员工入职审批详情组件；其余 → 通用摘要）
// - 审批流程 + 审批记录：hire 组件内置；其他类型通用时间线展示
// - 操作区：通过/驳回/退回/转办/委派/加签/抄送/取消（按权限显隐）
import { computed, ref, watch } from 'vue';

import { useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  DatePicker,
  Drawer,
  Input,
  InputNumber,
  message,
  Modal,
  Select,
  Spin,
  Tag,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';

import {
  addCcApprovalApi,
  addSignApprovalApi,
  cancelApprovalApi,
  delegateApprovalApi,
  getApprovalDetailApi,
  getUserDetailApi,
  processApprovalApi,
  rejectToApprovalApi,
  transferApprovalApi,
} from '#/api';
import { searchUsersApi } from '#/api/core/message/chat';
import { calcMaxProbation, formatContractText } from '#/utils/probation';

import HireApprovalDetail from './hire-approval-detail.vue';

const props = defineProps<{
  /** 列表行数据（ApprovalInstanceVO） */
  row: any;
}>();

const emit = defineEmits<{ success: [] }>();

const userStore = useUserStore();
const currentUserId = computed(() => userStore.userInfo?.userId);

const businessTypeMap: Record<string, { color: string; label: string }> = {
  contract: { label: '合同', color: 'geekblue' },
  expense: { label: '报销', color: 'volcano' },
  invoice: { label: '发票', color: 'purple' },
  leave: { label: '请假', color: 'orange' },
  order: { label: '订单', color: 'cyan' },
  payment: { label: '回款', color: 'gold' },
  purchase: { label: '采购', color: 'magenta' },
  quotation: { label: '报价单', color: 'blue' },
  refund: { label: '退款', color: 'red' },
  visit: { label: '外勤', color: 'lime' },
  hire: { label: '员工入职', color: 'geekblue' },
  user: { label: '员工入职', color: 'geekblue' },
  inbound: { label: '入库', color: 'lime' },
  outbound: { label: '出库', color: 'volcano' },
};

// 实例状态：1=待审批,2=审批中,3=已通过,4=已驳回,5=已撤回,6=待修改
const approvalStatusList: Record<number, { color: string; label: string }> = {
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
  5: { label: '已撤回', color: 'default' },
  6: { label: '待修改', color: 'orange' },
};

// 日志动作：0=提交,1=通过,2=驳回,3=转办,4=委派,5=加签,6=退回,7=取消,8=抄送
const logActionText: Record<number, string> = {
  0: '提交',
  1: '审批通过',
  2: '驳回',
  3: '转办',
  4: '委派',
  5: '加签',
  6: '退回',
  7: '取消',
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

// 节点状态标注（0=未开始,1=审批中,2=已通过,3=已驳回）
const nodeStatusMap: Record<number, { label: string; color: string }> = {
  0: { label: '未开始', color: 'default' },
  1: { label: '审批中', color: 'processing' },
  2: { label: '已通过', color: 'success' },
  3: { label: '已驳回', color: 'error' },
};

const visible = defineModel<boolean>('open', { default: false });

const detailLoading = ref(false);
const detailData = ref<any>(null);

// ============ 增强功能弹窗状态 ============
const modalState = ref<{
  type:
    | 'addCc'
    | 'addSign'
    | 'cancel'
    | 'delegate'
    | 'process'
    | 'rejectTo'
    | 'transfer'
    | null;
  /** process 时的动作：approve=通过 reject=驳回 */
  action?: 'approve' | 'reject';
}>({ type: null });

// 表单字段
const targetUserId = ref<number | undefined>(undefined);
const targetUserName = ref('');
const targetUserIds = ref<number[]>([]);
const addSignType = ref<1 | 2 | 3>(2);
const rejectToNodeKey = ref<string | undefined>('');
const commentText = ref('');
const cancelReason = ref('');
const ccReason = ref('');

// ===== 入职定薪（hire_approval 各环节填写项） =====
const suggestedSalary = ref<number | undefined>(undefined); // ① 部门经理：建议工资
const probationMonths = ref<number | undefined>(undefined); // ① 部门经理：试用期（月）
const abilityAssessment = ref(''); // ① 部门经理：工作能力评估（仅审批人可见）
const hireNegotiatedSalary = ref<number | undefined>(undefined); // ② 人事：谈定工资（与候选人协商确定）
const hireBandStatus = ref<number | undefined>(undefined); // ② 人事：1带宽内 2超带宽
const hireBandReason = ref(''); // ② 人事：超带宽原因
const hireProbationRatio = ref<number | undefined>(undefined); // ② 人事：试用期工资比例（%）
const ceoOpinion = ref(''); // ③ CEO：终审意见
const finalSalary = ref<number | undefined>(undefined); // ④ 财务：最终定薪
const effectiveDate = ref(''); // ④ 财务：生效日期

// 用户远程搜索
const userOptions = ref<{ label: string; value: number }[]>([]);
const userSearching = ref(false);
let userSearchTimer: any = null;

// ============ 权限判断（与待办列表一致） ============
function isCandidateApprover(row: any) {
  if (!row) return false;
  const list: number[] = row.candidateApprovers || [];
  return (
    currentUserId.value !== null &&
    currentUserId.value !== undefined &&
    list.includes(Number(currentUserId.value))
  );
}

function isActionable(row: any) {
  if (!row) return false;
  return row.status === 1 || row.status === 2 || row.status === 6;
}

function isSubmitter(row: any) {
  if (!row) return false;
  return (
    currentUserId.value !== null &&
    currentUserId.value !== undefined &&
    Number(row.submitterId) === Number(currentUserId.value)
  );
}

function canProcess(row: any) {
  return isCandidateApprover(row) && (row.status === 1 || row.status === 2);
}

function canApproverAction(row: any) {
  return canProcess(row);
}

function canCc(row: any) {
  if (!isActionable(row)) return false;
  if (isSubmitter(row)) return true;
  return isCandidateApprover(row);
}

function canCancel(row: any) {
  return isSubmitter(row) && (row.status === 1 || row.status === 2);
}

// user/hire 类型：撤回由入职详情组件内部处理（避免按钮重复）
const isHireType = computed(
  () => detailData.value?.businessType === 'user' || detailData.value?.businessType === 'hire',
);

// ===== 入职定薪：当前审批人所在环节（hire_approval 各节点 key → 环节序号） =====
const hireStageMap: Record<string, number> = {
  n_1787687341591_1: 1, // 部门经理审批
  hr_manager: 2, // 人事经理审批
  ceo_approval: 3, // CEO终审
  finance_manager: 4, // 财务定薪录入
};

const currentHireStage = computed<number | null>(() => {
  const d = detailData.value;
  if (!d || d.flowCode !== 'hire_approval') return null;
  // 当前处于审批中的节点
  const active = (d.flowNodes || []).find(
    (n: any) => n.nodeType === 2 && n.nodeStatus === 1,
  );
  if (!active) return null;
  const stage = hireStageMap[active.nodeKey];
  if (!stage) return null;
  // 当前用户必须是该节点候选审批人（节点级优先，回退实例级）
  const candidates: number[] =
    active.candidateApprovers || d.candidateApprovers || [];
  if (
    currentUserId.value !== null &&
    currentUserId.value !== undefined &&
    !candidates.map(Number).includes(Number(currentUserId.value))
  ) {
    return null;
  }
  return stage;
});

// 环节名称（底部操作栏提示）
const hireStageLabel = computed(() => {
  const map: Record<number, string> = {
    1: '部门经理审批',
    2: '人事经理审批',
    3: 'CEO终审',
    4: '财务定薪录入',
  };
  return currentHireStage.value ? map[currentHireStage.value] || '' : '';
});

// ============ 数据加载 ============
async function openDetail(row: any) {
  detailLoading.value = true;
  detailData.value = null;
  employeeRow.value = null;
  try {
    const res: any = await getApprovalDetailApi(row.id);
    detailData.value = res?.data?.data ?? res?.data ?? res ?? null;
    await loadEmployee();
  } catch (error: any) {
    message.error(error?.message || '加载详情失败');
  } finally {
    detailLoading.value = false;
  }
}

// 入职审批：按 business_id 拉取被审核员工数据
const employeeRow = ref<any>(null);
async function loadEmployee() {
  const bt = detailData.value?.businessType;
  if (bt !== 'user' && bt !== 'hire') return;
  const businessId = detailData.value?.businessId;
  if (!businessId) return;
  try {
    const res: any = await getUserDetailApi(businessId);
    employeeRow.value = res?.data?.data ?? res?.data ?? res ?? null;
  } catch {
    employeeRow.value = null;
  }
}

// ===== 入职定薪：合同期限 → 试用期法定上限（《劳动合同法》第十九条，与后端硬校验同规则） =====
const hireMaxProbation = computed(() =>
  calcMaxProbation(
    employeeRow.value?.contractType,
    employeeRow.value?.contractMonths,
  ),
);
const hireContractHint = computed(() =>
  formatContractText(
    employeeRow.value?.contractType,
    employeeRow.value?.contractMonths,
  ),
);

// ===== 入职定薪：带宽金额参照（带宽仅作提醒参照和是否特批的判定依据，非强制区间） =====
const hireBandRef = computed(() => detailData.value?.salaryBand || null);
const hireManagerStage = computed<any>(
  () =>
    (detailData.value?.hireSalaryStages || []).find(
      (s: any) => s.stage === 1,
    ) || null,
);
const hireHrStage = computed<any>(
  () =>
    (detailData.value?.hireSalaryStages || []).find(
      (s: any) => s.stage === 2,
    ) || null,
);
function moneyText(v: any): string {
  if (v === undefined || v === null || v === '') return '';
  const n = Number(v);
  if (Number.isNaN(n)) return String(v);
  return Number.isInteger(n) ? String(n) : n.toFixed(2);
}
const hireSuggestedSalaryText = computed(() =>
  moneyText(hireManagerStage.value?.suggestedSalary),
);
// 已保存的谈定工资展示文本（人事环节尚未发生则回退空串，不误显示）
const hireNegotiatedSalaryText = computed(() =>
  hireHrStage.value?.negotiatedSalary === undefined ||
  hireHrStage.value?.negotiatedSalary === null
    ? ''
    : moneyText(hireHrStage.value.negotiatedSalary),
);
// 参与带宽比对的实际工资金额：弹窗内正在填写的谈定工资 > 已保存谈定工资 > 部门经理建议工资
const hireBandAmount = computed<number | null>(() => {
  const raw =
    hireNegotiatedSalary.value ??
    hireHrStage.value?.negotiatedSalary ??
    hireManagerStage.value?.suggestedSalary;
  if (raw === undefined || raw === null || raw === '') return null;
  const n = Number(raw);
  return Number.isNaN(n) ? null : n;
});
// 实际工资金额是否越出带宽区间（带宽未配置或暂无金额时不判定）
const hireBandOutOfRange = computed(() => {
  const band = hireBandRef.value;
  const amount = hireBandAmount.value;
  if (!band || amount === null) return false;
  return (
    amount < Number(band.minSalary ?? 0) || amount > Number(band.maxSalary ?? 0)
  );
});

// 部门经理环节打开「审批通过」弹窗时，试用期建议默认 2 个月（钳制在法定上限内）
watch([modalState, employeeRow], () => {
  if (
    modalState.value.type === 'process' &&
    modalState.value.action === 'approve' &&
    currentHireStage.value === 1 &&
    (probationMonths.value === undefined || probationMonths.value === null)
  ) {
    probationMonths.value = Math.min(2, hireMaxProbation.value);
  }
});

watch(visible, (val) => {
  if (val && props.row?.id) {
    openDetail(props.row);
  }
});

// ============ 审批处理 ============
function openProcess(action: 'approve' | 'reject') {
  resetModalForm();
  commentText.value = '';
  modalState.value = { type: 'process', action };
}

// ============ 用户远程搜索 ============
function handleUserSearch(keyword: string) {
  if (userSearchTimer) clearTimeout(userSearchTimer);
  if (!keyword.trim()) {
    userOptions.value = [];
    return;
  }
  userSearchTimer = setTimeout(async () => {
    userSearching.value = true;
    try {
      const res: any = await searchUsersApi({
        keyword,
        page: 1,
        pageSize: 20,
      });
      const list: any[] = res.list || res || [];
      userOptions.value = list.map((u: any) => ({
        label:
          u.realName ||
          u.nickName ||
          u.userName ||
          u.username ||
          `用户${u.userId || u.id}`,
        value: u.userId || u.id,
      }));
    } catch {
      userOptions.value = [];
    } finally {
      userSearching.value = false;
    }
  }, 300);
}

function resetModalForm() {
  targetUserId.value = undefined;
  targetUserName.value = '';
  targetUserIds.value = [];
  addSignType.value = 2;
  rejectToNodeKey.value = '';
  commentText.value = '';
  cancelReason.value = '';
  ccReason.value = '';
  suggestedSalary.value = undefined;
  probationMonths.value = undefined;
  abilityAssessment.value = '';
  hireNegotiatedSalary.value = undefined;
  hireBandStatus.value = undefined;
  hireBandReason.value = '';
  hireProbationRatio.value = undefined;
  ceoOpinion.value = '';
  finalSalary.value = undefined;
  effectiveDate.value = '';
  userOptions.value = [];
}

function openModal(
  type: 'addCc' | 'addSign' | 'cancel' | 'delegate' | 'process' | 'rejectTo' | 'transfer',
  action?: 'approve' | 'reject',
) {
  resetModalForm();
  modalState.value = { type, action };
}

function closeModal() {
  modalState.value = { type: null };
}

const modalVisible = computed({
  get: () => modalState.value.type !== null,
  set: (v: boolean) => {
    if (!v) closeModal();
  },
});

const modalTitle = computed(() => {
  const map: Record<string, string> = {
    addCc: '添加抄送',
    addSign: '加签',
    cancel: '撤回审批',
    delegate: '委派审批人',
    process:
      modalState.value.action === 'approve' ? '审批通过' : '驳回审批',
    rejectTo: '退回审批',
    transfer: '转办审批',
  };
  return modalState.value.type ? map[modalState.value.type] : '';
});

// 退回节点选项：基于详情中的 flowNodes（审批过的节点）
const rejectNodeOptions = computed(() => {
  if (!detailData.value) return [];
  const nodes: any[] = detailData.value.flowNodes || [];
  return [
    { label: '退回到发起人（修改后重新提交）', value: '' },
    ...nodes
      .filter((n) => n.nodeType === 2)
      .map((n) => ({
        label: `退回到节点：${n.nodeName}`,
        value: n.nodeKey,
      })),
  ];
});

// ============ 提交处理 ============
async function handleSubmit() {
  const { type } = modalState.value;
  if (!type) return;
  const row = detailData.value;
  if (!row?.id) return;
  try {
    switch (type) {
      case 'process': {
        const isApprove = modalState.value.action === 'approve';
        // 驳回必须有原因（后端强制）
        if (!isApprove && !commentText.value.trim()) {
          message.warning('请填写驳回原因');
          return;
        }
        const payload: any = {
          action: isApprove ? 1 : 2,
          approverId: Number(userStore.userInfo?.userId ?? 0),
          approverName:
            userStore.userInfo?.realName || userStore.userInfo?.username,
          comment: commentText.value || undefined,
          instanceId: row.id,
        };
        // 入职定薪：通过时按当前环节校验并携带定薪字段
        const stage = currentHireStage.value;
        if (isApprove && stage === 1) {
          // ① 部门经理：工作能力评估 + 建议工资 + 试用期
          if (!abilityAssessment.value.trim()) {
            message.warning('请填写工作能力评估');
            return;
          }
          if (suggestedSalary.value === undefined || suggestedSalary.value === null) {
            message.warning('请填写建议工资');
            return;
          }
          if (!probationMonths.value) {
            message.warning('请填写试用期月数');
            return;
          }
          // 法定上限前置拦截（后端 save_stage 亦有同规则硬校验）
          if (
            hireMaxProbation.value === 0 ||
            Number(probationMonths.value) > hireMaxProbation.value
          ) {
            message.warning(hireContractHint.value);
            return;
          }
          payload.abilityAssessment = abilityAssessment.value.trim();
          payload.suggestedSalary = Number(suggestedSalary.value);
          payload.probationMonths = Number(probationMonths.value);
        } else if (isApprove && stage === 2) {
          // ② 人事经理：谈定工资 + 带宽评估 + 试用期比例 + 超带宽原因
          if (
            hireNegotiatedSalary.value === undefined ||
            hireNegotiatedSalary.value === null
          ) {
            message.warning('请填写与候选人协商确定的谈定工资');
            return;
          }
          if (!hireBandStatus.value) {
            message.warning('请选择带宽评估结果');
            return;
          }
          // 数据一致性拦截：实际工资金额（谈定优先）越出带宽区间，不能评估为“带宽内”（后端同规则兜底）
          if (hireBandStatus.value === 1 && hireBandOutOfRange.value) {
            message.warning(
              '工资金额超出岗位薪资带宽，请将带宽评估改为「超带宽」并填写原因转 CEO 特批',
            );
            return;
          }
          if (hireProbationRatio.value === undefined || hireProbationRatio.value === null) {
            message.warning('请填写试用期工资比例');
            return;
          }
          if (hireBandStatus.value === 2 && !hireBandReason.value.trim()) {
            message.warning('超带宽必须填写原因');
            return;
          }
          payload.negotiatedSalary = Number(hireNegotiatedSalary.value);
          payload.bandStatus = hireBandStatus.value;
          if (hireBandStatus.value === 2) {
            payload.bandReason = hireBandReason.value.trim();
          }
          payload.probationRatio = Number(hireProbationRatio.value) / 100;
        } else if (isApprove && stage === 3) {
          // ③ CEO终审：特批意见
          if (!ceoOpinion.value.trim()) {
            message.warning('请填写终审意见');
            return;
          }
          payload.ceoOpinion = ceoOpinion.value.trim();
        } else if (isApprove && stage === 4) {
          // ④ 财务：最终定薪 + 生效日期
          if (finalSalary.value === undefined || finalSalary.value === null) {
            message.warning('请填写最终定薪');
            return;
          }
          if (!effectiveDate.value) {
            message.warning('请选择生效日期');
            return;
          }
          payload.finalSalary = Number(finalSalary.value);
          payload.effectiveDate = effectiveDate.value;
        }
        await processApprovalApi(payload);
        message.success(isApprove ? '已审批通过' : '已驳回');
        break;
      }
      case 'addCc': {
        if (targetUserIds.value.length === 0) {
          message.warning('请选择抄送用户');
          return;
        }
        await addCcApprovalApi({
          ccReason: ccReason.value || undefined,
          instanceId: row.id,
          userIds: targetUserIds.value,
        });
        message.success('已添加抄送');
        break;
      }
      case 'addSign': {
        if (targetUserIds.value.length === 0) {
          message.warning('请选择加签用户');
          return;
        }
        await addSignApprovalApi({
          addSignType: addSignType.value,
          comment: commentText.value || undefined,
          instanceId: row.id,
          targetUserIds: targetUserIds.value,
        });
        message.success('已加签');
        break;
      }
      case 'cancel': {
        await cancelApprovalApi({
          cancelReason: cancelReason.value || undefined,
          instanceId: row.id,
        });
        message.success('已撤回审批');
        break;
      }
      case 'delegate': {
        if (!targetUserId.value) {
          message.warning('请选择被委派人');
          return;
        }
        await delegateApprovalApi({
          comment: commentText.value || undefined,
          instanceId: row.id,
          targetUserId: targetUserId.value,
          targetUserName: targetUserName.value || undefined,
        });
        message.success('已委派');
        break;
      }
      case 'rejectTo': {
        await rejectToApprovalApi({
          comment: commentText.value || undefined,
          instanceId: row.id,
          rejectToNodeKey:
            rejectToNodeKey.value === ''
              ? undefined
              : rejectToNodeKey.value || undefined,
        });
        message.success('已退回');
        break;
      }
      case 'transfer': {
        if (!targetUserId.value) {
          message.warning('请选择转办目标用户');
          return;
        }
        await transferApprovalApi({
          comment: commentText.value || undefined,
          instanceId: row.id,
          targetUserId: targetUserId.value,
          targetUserName: targetUserName.value || undefined,
        });
        message.success('已转办');
        break;
      }
    }
    closeModal();
    emit('success');
    await openDetail(row);
  } catch (error: any) {
    message.error(error?.message || '操作失败');
  }
}

// 流程节点（非 hire 类型展示）：按节点顺序展示实例快照
const flowNodes = computed<any[]>(() => {
  const nodes: any[] = detailData.value?.flowNodes || [];
  return nodes.filter((n) => n.nodeType === 2);
});
</script>

<template>
  <Drawer
    v-model:open="visible"
    :title="detailData?.businessTitle || '审批详情'"
    placement="right"
    width="75%"
    destroy-on-close
  >
    <div v-if="detailLoading" class="flex justify-center py-16">
      <Spin />
    </div>

    <div v-else-if="detailData" class="space-y-4 px-2">
      <!-- ===== 业务详情：入职审批（员工头像/姓名/职位等优先展示） ===== -->
      <template v-if="isHireType">
        <HireApprovalDetail
          :row="employeeRow || { nickName: detailData.submitterName }"
          :instance-id="Number(detailData.id)"
          :stage="currentHireStage"
          @cancel-success="emit('success')"
        />
      </template>

      <!-- ===== 业务详情：其他类型（通用展示） ===== -->
      <template v-else>
        <!-- 审批流程 -->
        <div class="rounded-xl border border-gray-100 p-4">
          <div class="mb-3 flex items-center justify-between">
            <span class="text-sm font-semibold">审批流程</span>
            <span class="text-xs text-gray-400">
              {{ detailData.flowCode }}
            </span>
          </div>
          <div v-if="flowNodes.length" class="flex items-center overflow-x-auto py-1">
            <template v-for="(node, idx) in flowNodes" :key="node.nodeKey">
              <div
                class="min-w-[118px] flex-none rounded-lg border p-2.5 text-center"
                :class="{
                  'border-primary bg-primary/10':
                    node.nodeStatus === 1,
                  'border-green-400/50 bg-green-50':
                    node.nodeStatus === 2,
                  'border-red-400/60 bg-red-50':
                    node.nodeStatus === 3,
                }"
              >
                <div class="whitespace-nowrap text-[13px] font-semibold">
                  {{ node.nodeName }}
                </div>
                <div
                  v-if="node.nodeStatus !== undefined"
                  class="mt-1"
                >
                  <Tag
                    size="small"
                    :color="
                      (nodeStatusMap[node.nodeStatus] || {}).color || 'default'
                    "
                  >
                    {{ (nodeStatusMap[node.nodeStatus] || {}).label || '未开始' }}
                  </Tag>
                </div>
              </div>
              <span
                v-if="idx < flowNodes.length - 1"
                class="flex-none px-2 text-gray-300"
                >→</span
              >
            </template>
          </div>
          <div v-else class="py-6 text-center text-xs text-gray-400">
            暂无流程数据
          </div>
        </div>

        <!-- 审批记录 -->
        <div class="rounded-xl border border-gray-100 p-4">
          <div class="mb-3 flex items-center justify-between">
            <span class="text-sm font-semibold">审批记录</span>
            <span class="text-xs text-gray-400">
              {{ detailData.logs?.length || 0 }} 条
            </span>
          </div>
          <Timeline v-if="detailData.logs?.length" class="pt-1">
            <TimelineItem
              v-for="log in detailData.logs"
              :key="log.id"
              :color="logActionColor[log.action] || 'blue'"
            >
              <div class="text-[13px] font-medium">
                {{ log.nodeName || logActionText[log.action] || '审批' }}
              </div>
              <div class="text-sm text-gray-500">
                {{ log.approverName || log.operatorName }} ·
                {{ logActionText[log.action] || '--' }} ·
                {{ formatDateTime(log.createTime || log.create_at) }}
              </div>
              <div
                v-if="log.comment || log.reason"
                class="mt-1 rounded border-l-[3px] border-primary/40 bg-gray-50 px-3 py-1.5 text-sm text-gray-600"
              >
                {{ log.comment || log.reason }}
              </div>
            </TimelineItem>
          </Timeline>
          <div v-else class="py-6 text-center text-xs text-gray-400">
            暂无审批记录
          </div>
        </div>
      </template>

      <!-- ===== 通用业务摘要（业务类型/状态/提交人/时间/候选审批人） ===== -->
      <div class="rounded-xl border border-gray-100 p-4">
        <div class="grid grid-cols-2 gap-x-6 gap-y-3">
          <div class="flex items-center justify-between gap-2">
            <span class="text-xs text-gray-500">业务类型：</span>
            <Tag
              :color="
                businessTypeMap[detailData.businessType]?.color || 'default'
              "
            >
              {{
                businessTypeMap[detailData.businessType]?.label ||
                detailData.businessType
              }}
            </Tag>
          </div>
          <div class="flex items-center justify-between gap-2">
            <span class="text-xs text-gray-500">当前状态：</span>
            <Tag
              :color="approvalStatusList[detailData.status]?.color || 'default'"
            >
              {{ approvalStatusList[detailData.status]?.label || '未知' }}
            </Tag>
          </div>
          <div class="flex items-center justify-between gap-2">
            <span class="text-xs text-gray-500">提交人：</span>
            <span class="text-sm">{{ detailData.submitterName }}</span>
          </div>
          <div class="flex items-center justify-between gap-2">
            <span class="text-xs text-gray-500">提交时间：</span>
            <span class="text-sm">
              {{ formatDateTime(detailData.submittedAt) || '-' }}
            </span>
          </div>
        </div>

        <!-- 候选审批人列表 -->
        <div
          v-if="
            detailData.candidateApproverNames?.length > 0 &&
            (detailData.status === 1 || detailData.status === 2)
          "
          class="mt-3 border-t border-gray-100 pt-3"
        >
          <div class="flex items-start justify-between gap-2">
            <span class="text-xs text-gray-500">候选审批人：</span>
            <div class="flex max-w-[70%] flex-wrap justify-end gap-1.5">
              <span
                v-for="(name, idx) in detailData.candidateApproverNames"
                :key="idx"
                class="inline-flex items-center gap-1 rounded px-2 py-0.5 text-xs"
                :class="
                  detailData.processedApprovers?.includes(
                    detailData.candidateApprovers?.[idx],
                  )
                    ? 'bg-green-100 text-green-700'
                    : 'bg-blue-50 text-blue-600'
                "
              >
                <svg
                  v-if="
                    detailData.processedApprovers?.includes(
                      detailData.candidateApprovers?.[idx],
                    )
                  "
                  class="h-3 w-3"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="3"
                  viewBox="0 0 24 24"
                >
                  <path
                    d="M5 13l4 4L19 7"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
                {{ name }}
              </span>
            </div>
          </div>
          <div
            v-if="detailData.candidateApprovers?.length > 1"
            class="text-right text-xs text-gray-400"
          >
            已处理 {{ detailData.processedApprovers?.length || 0 }} /
            {{ detailData.candidateApprovers?.length || 0 }} 人
          </div>
        </div>
      </div>
    </div>

    <!-- 增强功能统一弹窗 -->
    <Modal
      v-model:visible="modalVisible"
      :title="modalTitle"
      destroy-on-close
      width="520px"
      @cancel="closeModal"
    >
      <div class="space-y-4 py-2">
        <!-- 退回目标节点选择 -->
        <div v-if="modalState.type === 'rejectTo'">
          <div class="mb-2 text-sm text-gray-600">退回到：</div>
          <Select
            v-model:value="rejectToNodeKey"
            :options="rejectNodeOptions"
            placeholder="请选择退回目标"
            style="width: 100%"
          />
          <div class="mt-1 text-xs text-gray-400">
            默认退回到发起人，发起人修改后可重新提交
          </div>
        </div>

        <!-- 转办 / 委派：单选用户 -->
        <div
          v-if="
            modalState.type === 'transfer' || modalState.type === 'delegate'
          "
        >
          <div class="mb-2 text-sm text-gray-600">
            {{ modalState.type === 'transfer' ? '转办给：' : '委派给：' }}
          </div>
          <Select
            v-model:value="targetUserId"
            :filter-option="false"
            :loading="userSearching"
            :options="userOptions"
            allow-clear
            show-search
            placeholder="输入姓名/用户名搜索"
            style="width: 100%"
            @search="handleUserSearch"
            @change="
              (v: any) => {
                const opt = userOptions.find((o) => o.value === v);
                targetUserName = opt?.label || '';
              }
            "
          />
          <div class="mt-1 text-xs text-gray-400">
            <template v-if="modalState.type === 'transfer'">
              转办后责任转移，原审批人不再参与此节点审批
            </template>
            <template v-else>
              委派后责任仍归原审批人，被委派人处理后转回
            </template>
          </div>
        </div>

        <!-- 加签：多选用户 + 类型选择 -->
        <div v-if="modalState.type === 'addSign'">
          <div class="mb-2 text-sm text-gray-600">加签类型：</div>
          <Select
            v-model:value="addSignType"
            :options="[
              { label: '前加签（先由加签人审批，再由我审批）', value: 1 },
              { label: '后加签（我通过后，再由加签人审批）', value: 2 },
              { label: '并加签（与同时审批，并行处理）', value: 3 },
            ]"
            style="width: 100%"
          />
          <div class="mt-3 mb-2 text-sm text-gray-600">加签用户：</div>
          <Select
            v-model:value="targetUserIds"
            :filter-option="false"
            :loading="userSearching"
            :options="userOptions"
            allow-clear
            mode="multiple"
            placeholder="输入姓名/用户名搜索（可多选）"
            style="width: 100%"
            @search="handleUserSearch"
          />
        </div>

        <!-- 抄送：多选用户 -->
        <div v-if="modalState.type === 'addCc'">
          <div class="mb-2 text-sm text-gray-600">抄送给：</div>
          <Select
            v-model:value="targetUserIds"
            :filter-option="false"
            :loading="userSearching"
            :options="userOptions"
            allow-clear
            mode="multiple"
            placeholder="输入姓名/用户名搜索（可多选）"
            style="width: 100%"
            @search="handleUserSearch"
          />
        </div>

        <!-- 取消：撤回原因 -->
        <div v-if="modalState.type === 'cancel'">
          <div class="mb-2 text-sm text-gray-600">
            撤回原因（仅发起人可撤回进行中的审批）：
          </div>
          <Input.TextArea
            v-model:value="cancelReason"
            :rows="4"
            placeholder="请填写撤回原因"
          />
        </div>

        <!-- 审批处理（通过/驳回）：审批意见 + 入职定薪填写项（按环节） -->
        <div v-if="modalState.type === 'process'" class="space-y-3">
          <div>
            <div class="mb-2 text-sm text-gray-600">
              {{
                modalState.action === 'approve'
                  ? '审批意见：'
                  : '驳回原因（必填）：'
              }}
            </div>
            <Input.TextArea
              v-model:value="commentText"
              :rows="3"
              :placeholder="
                modalState.action === 'approve'
                  ? '请填写审批意见（可选）'
                  : '请填写驳回原因'
              "
            />
          </div>

          <!-- 入职定薪信息（hire_approval 通过时按当前审批环节显示对应填写项） -->
          <div
            v-if="modalState.action === 'approve' && currentHireStage"
            class="space-y-3 rounded-lg border border-gray-100 bg-gray-50/60 p-3 dark:border-gray-700/60 dark:bg-gray-800/40"
          >
            <div class="text-xs font-semibold text-gray-600 dark:text-gray-300">
              入职定薪信息
              <span class="ml-1 font-normal text-gray-400">
                （
                {{
                  {
                    1: '部门经理审批',
                    2: '人事经理审批',
                    3: 'CEO终审',
                    4: '财务定薪录入',
                  }[currentHireStage]
                }}
                环节）
              </span>
            </div>

            <!-- 隐私提示：本环节填写内容仅审批人可见，提交人不可见 -->
            <div
              class="flex items-start gap-1.5 rounded-md border border-amber-200/70 bg-amber-50/80 px-2.5 py-1.5 text-xs text-amber-700 dark:border-amber-500/30 dark:bg-amber-500/10 dark:text-amber-400"
            >
              <span class="mt-px shrink-0">🔒</span>
              <span>
                本环节填写的评估与定薪信息仅审批流程内可见，提交审批的人无法查看，可放心填写。
              </span>
            </div>

            <!-- 带宽金额参照：岗位带宽区间仅作提醒参照，工资金额由人事与候选人协商确定 -->
            <div
              class="space-y-1 rounded-md border border-blue-200/70 bg-blue-50/70 px-2.5 py-2 text-xs text-gray-700 dark:border-blue-500/30 dark:bg-blue-500/10 dark:text-gray-200"
            >
              <div
                v-if="currentHireStage !== 1"
                class="flex items-center justify-between"
              >
                <span>部门经理建议工资</span>
                <span class="font-semibold text-gray-900 dark:text-white">
                  {{
                    hireSuggestedSalaryText
                      ? `${hireSuggestedSalaryText} 元/月`
                      : '—'
                  }}
                </span>
              </div>
              <div
                v-if="currentHireStage !== 1 && hireNegotiatedSalaryText"
                class="flex items-center justify-between"
              >
                <span>谈定工资（人事协商结果）</span>
                <span class="font-semibold text-gray-900 dark:text-white">
                  {{ hireNegotiatedSalaryText }} 元/月
                </span>
              </div>
              <div class="flex items-center justify-between">
                <span>
                  {{ hireBandRef?.postName || employeeRow?.postName || '岗位' }}薪资带宽（参照）
                </span>
                <span
                  v-if="hireBandRef"
                  class="font-semibold text-gray-900 dark:text-white"
                >
                  {{ moneyText(hireBandRef.minSalary) }} ~
                  {{ moneyText(hireBandRef.maxSalary) }} 元/月
                </span>
                <span v-else class="font-semibold text-orange-600 dark:text-orange-400">
                  未配置，请联系管理员维护岗位薪资带宽
                </span>
              </div>
              <div
                v-if="hireBandOutOfRange"
                class="font-medium text-red-600 dark:text-red-400"
              >
                ⚠ 工资金额不在岗位带宽区间内，应将带宽评估选为「超带宽」并说明原因转 CEO 特批
              </div>
            </div>

            <!-- ① 部门经理：工作能力评估 + 建议工资 + 试用期 -->
            <template v-if="currentHireStage === 1">
              <div>
                <div class="mb-1 text-sm text-gray-600 dark:text-gray-300">
                  工作能力评估（必填）
                </div>
                <Input.TextArea
                  v-model:value="abilityAssessment"
                  :rows="3"
                  :maxlength="500"
                  show-count
                  placeholder="对候选人的工作能力、岗位匹配度等进行评估（仅审批人可见）"
                />
              </div>
              <div>
                <div class="mb-1 text-sm text-gray-600 dark:text-gray-300">
                  建议工资（元/月）
                </div>
                <InputNumber
                  v-model:value="suggestedSalary"
                  :min="0"
                  :precision="2"
                  placeholder="如 6000"
                  style="width: 100%"
                />
              </div>
              <div>
                <div class="mb-1 text-sm text-gray-600 dark:text-gray-300">
                  试用期（月）
                  <span class="ml-1 text-xs font-normal text-gray-400">
                    {{ hireContractHint }}
                  </span>
                </div>
                <InputNumber
                  v-model:value="probationMonths"
                  :min="0"
                  :max="hireMaxProbation > 0 ? hireMaxProbation : undefined"
                  :precision="0"
                  :disabled="hireMaxProbation === 0"
                  placeholder="默认建议 2 个月"
                  style="width: 100%"
                />
              </div>
            </template>

            <!-- ② 人事经理：谈定工资 + 带宽评估 + 试用期比例 + 超带宽原因 -->
            <template v-else-if="currentHireStage === 2">
              <div>
                <div class="mb-1 text-sm text-gray-600 dark:text-gray-300">
                  谈定工资（元/月，必填）
                  <span class="ml-1 text-xs font-normal text-gray-400">
                    与候选人协商确定的金额，财务据此录入最终定薪
                  </span>
                </div>
                <InputNumber
                  v-model:value="hireNegotiatedSalary"
                  :min="0"
                  :precision="2"
                  placeholder="与候选人协商确定的月工资，如 6500"
                  style="width: 100%"
                />
              </div>
              <div>
                <div class="mb-1 text-sm text-gray-600 dark:text-gray-300">
                  带宽评估
                </div>
                <Select
                  v-model:value="hireBandStatus"
                  :options="[
                    { label: '带宽内', value: 1 },
                    { label: '超带宽（转CEO特批）', value: 2 },
                  ]"
                  placeholder="评估谈定工资是否在岗位薪资带宽内"
                  style="width: 100%"
                />
              </div>
              <div>
                <div class="mb-1 text-sm text-gray-600 dark:text-gray-300">
                  试用期工资比例（%）
                </div>
                <InputNumber
                  v-model:value="hireProbationRatio"
                  :min="0"
                  :max="100"
                  :precision="0"
                  placeholder="如 80"
                  style="width: 100%"
                />
              </div>
              <div v-if="hireBandStatus === 2">
                <div class="mb-1 text-sm text-gray-600 dark:text-gray-300">
                  超带宽原因（必填）
                </div>
                <Input.TextArea
                  v-model:value="hireBandReason"
                  :rows="2"
                  :maxlength="200"
                  show-count
                  placeholder="说明为何超出带宽范围，将转 CEO 特批"
                />
              </div>
            </template>

            <!-- ③ CEO终审：特批意见 -->
            <template v-else-if="currentHireStage === 3">
              <div>
                <div class="mb-1 text-sm text-gray-600 dark:text-gray-300">
                  终审意见（必填）
                </div>
                <Input.TextArea
                  v-model:value="ceoOpinion"
                  :rows="3"
                  :maxlength="200"
                  show-count
                  placeholder="请给出最终审批意见（含特批说明）"
                />
              </div>
            </template>

            <!-- ④ 财务：最终定薪 + 生效日期 -->
            <template v-else-if="currentHireStage === 4">
              <div>
                <div class="mb-1 text-sm text-gray-600 dark:text-gray-300">
                  最终定薪（元/月）
                </div>
                <InputNumber
                  v-model:value="finalSalary"
                  :min="0"
                  :precision="2"
                  placeholder="如 6000"
                  style="width: 100%"
                />
              </div>
              <div>
                <div class="mb-1 text-sm text-gray-600 dark:text-gray-300">
                  生效日期
                </div>
                <DatePicker
                  v-model:value="effectiveDate"
                  value-format="YYYY-MM-DD"
                  placeholder="选择定薪生效日期"
                  style="width: 100%"
                />
              </div>
            </template>
          </div>
        </div>

        <!-- 退回 / 转办 / 委派 / 加签：审批意见 -->
        <div
          v-if="
            modalState.type &&
            ['addSign', 'delegate', 'rejectTo', 'transfer'].includes(
              modalState.type,
            )
          "
        >
          <div class="mb-2 text-sm text-gray-600">审批意见：</div>
          <Input.TextArea
            v-model:value="commentText"
            :rows="3"
            placeholder="请填写说明（可选）"
          />
        </div>

        <!-- 抄送理由 -->
        <div v-if="modalState.type === 'addCc'">
          <div class="mb-2 text-sm text-gray-600">抄送说明：</div>
          <Input.TextArea
            v-model:value="ccReason"
            :rows="3"
            placeholder="请填写抄送说明（可选）"
          />
        </div>
      </div>

      <template #footer>
        <Button @click="closeModal">取消</Button>
        <Button
          type="primary"
          :danger="modalState.type === 'process' && modalState.action === 'reject'"
          @click="handleSubmit"
        >
          {{
            modalState.type === 'process'
              ? modalState.action === 'approve'
                ? '确认通过'
                : '确认驳回'
              : '确认'
          }}
        </Button>
      </template>
    </Modal>

    <!-- ===== 底部操作栏（固定，操作按钮下沉） ===== -->
    <template #footer>
      <div
        v-if="
          canProcess(detailData) ||
          canCancel(detailData) ||
          canCc(detailData)
        "
        class="flex flex-wrap items-center justify-between gap-2"
      >
        <div class="min-w-0">
          <span v-if="currentHireStage" class="text-xs text-gray-400">
            当前环节：{{ hireStageLabel }}
          </span>
        </div>
        <div class="flex flex-wrap items-center justify-end gap-2">
          <Button
            v-if="canCancel(detailData) && !isHireType"
            danger
            @click="openModal('cancel')"
          >
            取消
          </Button>
          <Button v-if="canCc(detailData)" @click="openModal('addCc')">
            抄送
          </Button>
          <Button
            v-if="canApproverAction(detailData)"
            @click="openModal('addSign')"
          >
            加签
          </Button>
          <Button
            v-if="canApproverAction(detailData)"
            @click="openModal('delegate')"
          >
            委派
          </Button>
          <Button
            v-if="canApproverAction(detailData)"
            @click="openModal('transfer')"
          >
            转办
          </Button>
          <Button
            v-if="canApproverAction(detailData)"
            @click="openModal('rejectTo')"
          >
            退回
          </Button>
          <Button
            v-if="canProcess(detailData)"
            danger
            @click="openProcess('reject')"
          >
            驳回
          </Button>
          <Button
            v-if="canProcess(detailData)"
            type="primary"
            @click="openProcess('approve')"
          >
            通过
          </Button>
        </div>
      </div>
    </template>
  </Drawer>
</template>
