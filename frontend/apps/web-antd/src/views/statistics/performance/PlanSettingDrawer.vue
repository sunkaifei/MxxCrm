<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { IconifyIcon } from '@vben/icons';
import { useUserStore } from '@vben/stores';

import {
  Alert,
  Button,
  Card,
  Drawer,
  InputNumber,
  message,
  Spin,
  Tag,
  Timeline,
  TimelineItem,
  Tooltip,
  Input,
} from 'ant-design-vue';

import {
  createPlanApi,
  getPlanDetailApi,
  getPlanListApi,
  modifyPlanApi,
  submitPlanApi,
  updatePlanTargetsApi,
} from '#/api/core/statistics';

const props = defineProps<{
  visible: boolean;
  year: number;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'success'): void;
}>();

const userStore = useUserStore();
const loading = ref(false);
const submitting = ref(false);

// ===== 计划状态 =====
type PlanStatus = 'none' | 'draft' | 'pending' | 'approved' | 'rejected';
const planStatus = ref<PlanStatus>('none');
const planId = ref<number | null>(null);
const planDetail = ref<any>(null);
const approvalLogs = ref<any[]>([]);

// ===== 入职月份判断 =====
const hireMonth = computed(() => {
  const hireDate = userStore.userInfo?.hireDate || userStore.userInfo?.createTime;
  if (!hireDate) return 1;
  const d = new Date(hireDate);
  if (d.getFullYear() === props.year) return d.getMonth() + 1;
  return 1;
});

const isCurrentYearHire = computed(() => hireMonth.value > 1);

// ===== 12 个月目标数据 =====
interface MonthlyTarget {
  month: number;
  editable: boolean;
  contractTargetAmount: number;
  paymentTargetAmount: number;
  contractTargetCount: number;
}

const monthlyTargets = ref<MonthlyTarget[]>([]);

// 可编辑的月份（用于批量操作的范围）
const editableMonths = computed(() => monthlyTargets.value.filter((m) => m.editable));

function initMonthlyTargets() {
  monthlyTargets.value = Array.from({ length: 12 }, (_, i) => {
    const month = i + 1;
    return {
      month,
      editable: month >= hireMonth.value,
      contractTargetAmount: 0,
      paymentTargetAmount: 0,
      contractTargetCount: 0,
    };
  });
}

// ===== 万元换算显示（纯展示，不干扰输入） =====
function toWan(val: number): string {
  if (!val) return '';
  const wan = val / 10000;
  return wan >= 1 ? `≈${wan.toFixed(2)}万` : '';
}

// ===== 全年累加 =====
const totalContractAmount = computed(() =>
  monthlyTargets.value.reduce((s, m) => s + (m.contractTargetAmount || 0), 0),
);
const totalPaymentAmount = computed(() =>
  monthlyTargets.value.reduce((s, m) => s + (m.paymentTargetAmount || 0), 0),
);
const totalContractCount = computed(() =>
  monthlyTargets.value.reduce((s, m) => s + (m.contractTargetCount || 0), 0),
);

function formatCurrency(val: number) {
  if (!val) return '¥0';
  if (val >= 10000) return `¥${(val / 10000).toFixed(2)}万`;
  return `¥${val.toLocaleString()}`;
}

// ===== 批量操作 =====
type BatchField = 'contractTargetAmount' | 'paymentTargetAmount' | 'contractTargetCount';
type BatchMode = 'applyAll' | 'splitTotal';

const batchField = ref<BatchField>('contractTargetAmount');
const batchMode = ref<BatchMode>('applyAll');
const batchValue = ref<number | undefined>(undefined);

const batchFieldLabels: Record<BatchField, string> = {
  contractTargetAmount: '合同金额',
  paymentTargetAmount: '回款金额',
  contractTargetCount: '合同数量',
};

const batchFieldUnits: Record<BatchField, string> = {
  contractTargetAmount: '元',
  paymentTargetAmount: '元',
  contractTargetCount: '个',
};

/**
 * 执行批量操作：
 * - applyAll：将输入值应用到所有可编辑月份
 * - splitTotal：将输入总数平均分解到所有可编辑月份（余数加到第一个月）
 */
function applyBatch() {
  if (batchValue.value === undefined || batchValue.value === null) {
    message.warning('请先输入数值');
    return;
  }
  const val = Number(batchValue.value);
  if (isNaN(val) || val < 0) {
    message.warning('请输入有效的非负数');
    return;
  }

  const targets = editableMonths.value;
  if (targets.length === 0) {
    message.warning('没有可应用的月份');
    return;
  }

  if (batchMode.value === 'applyAll') {
    // 应用到所有可编辑月份
    targets.forEach((m) => {
      m[batchField.value] = val;
    });
    message.success(`已将 ${val}${batchFieldUnits[batchField.value]} 应用到 ${targets.length} 个月份`);
  } else {
    // 按总数平均分解
    const base = Math.floor(val / targets.length);
    let remainder = val - base * targets.length;
    targets.forEach((m) => {
      if (remainder > 0) {
        m[batchField.value] = base + 1;
        remainder--;
      } else {
        m[batchField.value] = base;
      }
    });
    message.success(`已将 ${val}${batchFieldUnits[batchField.value]} 平均分解到 ${targets.length} 个月份（每月 ${base}）`);
  }

  batchValue.value = undefined;
}

/**
 * 快速清零所有可编辑月份
 */
function clearAll() {
  editableMonths.value.forEach((m) => {
    m.contractTargetAmount = 0;
    m.paymentTargetAmount = 0;
    m.contractTargetCount = 0;
  });
  message.success('已清零所有可编辑月份');
}

// ===== 加载计划数据 =====
async function loadPlan() {
  loading.value = true;
  try {
    const employeeId = userStore.userInfo?.userId || userStore.userInfo?.id;
    const res = await getPlanListApi({ year: props.year, employeeId });
    // 注意：requestClient 已配置 responseReturn: 'data'，拦截器会自动解包 data 字段
    // 所以 res 本身就是 plans 数组，无需再 res?.data
    const plans = Array.isArray(res) ? res : (res?.data || []);
    if (plans.length === 0) {
      planStatus.value = 'none';
      planId.value = null;
      planDetail.value = null;
      initMonthlyTargets();
      return;
    }

    const plan = plans[0];
    planId.value = plan.id;
    const statusNum = Number(plan.status);
    planStatus.value = (['draft', 'pending', 'approved', 'rejected'] as const)[statusNum] || 'none';

    const detailRes = await getPlanDetailApi(plan.id);
    // detailRes 已被拦截器解包，本身就是详情对象
    const detail = detailRes && typeof detailRes === 'object' && !Array.isArray(detailRes)
      ? (detailRes.data || detailRes)
      : detailRes;
    planDetail.value = detail;
    approvalLogs.value = detail?.approvalLogs || detail?.approval_logs || [];

    const monthlyList = detail?.monthlyTargets || detail?.monthly_targets || [];
    monthlyTargets.value = Array.from({ length: 12 }, (_, i) => {
      const month = i + 1;
      const matched = monthlyList.find((m: any) => m.month === month) || {};
      return {
        month,
        editable: month >= hireMonth.value && planStatus.value !== 'pending' && planStatus.value !== 'approved',
        contractTargetAmount: Number(matched.contractTargetAmount || 0),
        paymentTargetAmount: Number(matched.paymentTargetAmount || 0),
        contractTargetCount: Number(matched.contractTargetCount || 0),
      };
    });
  } catch (e: any) {
    const msg = e?.message || '加载计划失败';
    if (msg.includes('403') || msg.includes('权限')) {
      message.warning('您没有查看销售计划的权限');
    } else {
      console.error('加载计划失败', e);
    }
    initMonthlyTargets();
  } finally {
    loading.value = false;
  }
}

// ===== 保存草稿 =====
async function handleSaveDraft() {
  if (totalContractAmount.value === 0) {
    message.warning('请至少填写一个月的合同目标金额');
    return;
  }
  submitting.value = true;
  try {
    const targets = monthlyTargets.value
      .filter((m) => m.editable)
      .map((m) => ({
        month: m.month,
        contractTargetAmount: m.contractTargetAmount,
        paymentTargetAmount: m.paymentTargetAmount,
        contractTargetCount: m.contractTargetCount,
      }));

    if (planId.value && (planStatus.value === 'draft' || planStatus.value === 'rejected')) {
      await updatePlanTargetsApi({
        planId: planId.value,
        monthlyTargets: targets,
      });
    } else if (!planId.value) {
      const res = await createPlanApi({ year: props.year, monthlyTargets: targets });
      // res 已被拦截器解包，本身就是详情对象
      const newId = res?.id ?? res?.data?.id;
      if (typeof newId === 'number' && newId > 0) {
        planId.value = newId;
      } else {
        throw new Error('创建计划失败：未返回有效ID');
      }
    }
    planStatus.value = 'draft';
    message.success('草稿已保存');
    await loadPlan();
  } catch (e: any) {
    message.error(e?.message || '保存失败');
  } finally {
    submitting.value = false;
  }
}

// ===== 提交审批 =====
async function handleSubmit() {
  if (!planId.value) {
    message.warning('请先保存草稿');
    return;
  }
  submitting.value = true;
  try {
    await submitPlanApi(planId.value);
    planStatus.value = 'pending';
    message.success('已提交审批，请等待上级审批');
    await loadPlan();
    emit('success');
  } catch (e: any) {
    message.error(e?.message || '提交失败');
  } finally {
    submitting.value = false;
  }
}

// ===== 申请修改 =====
const modifyReason = ref('');
const showModifyInput = ref(false);

async function handleApplyModify() {
  if (!modifyReason.value.trim()) {
    message.warning('请填写修改原因');
    return;
  }
  submitting.value = true;
  try {
    const targets = monthlyTargets.value
      .filter((m) => m.editable)
      .map((m) => ({
        month: m.month,
        contractTargetAmount: m.contractTargetAmount,
        paymentTargetAmount: m.paymentTargetAmount,
        contractTargetCount: m.contractTargetCount,
      }));
    await modifyPlanApi({
      planId: planId.value!,
      reason: modifyReason.value,
      monthlyTargets: targets,
    });
    planStatus.value = 'pending';
    modifyReason.value = '';
    showModifyInput.value = false;
    message.success('修改申请已提交，请等待审批');
    await loadPlan();
  } catch (e: any) {
    message.error(e?.message || '申请修改失败');
  } finally {
    submitting.value = false;
  }
}

// ===== 监听 visible 变化 =====
watch(
  () => props.visible,
  (val) => {
    if (val) loadPlan();
  },
);

// ===== 状态配置 =====
const statusConfig = computed<Record<PlanStatus, { color: string; text: string; desc: string }>>(() => {
  const pendingDesc = planDetail.value?.currentApproverName
    ? `计划已提交，当前审批人：${planDetail.value.currentApproverName}（第${planDetail.value.approvalLevel || 1}级/共${planDetail.value.totalLevels || 1}级），期间不可修改`
    : '计划已提交，等待上级审批，期间不可修改';
  return {
    none: { color: 'default', text: '未创建', desc: '请填写每月销售目标并保存' },
    draft: { color: 'warning', text: '草稿', desc: '草稿可继续编辑，确认后点击提交审批' },
    pending: { color: 'processing', text: '审批中', desc: pendingDesc },
    approved: { color: 'success', text: '已通过', desc: '计划已审批通过，如需修改请申请变更' },
    rejected: { color: 'error', text: '已驳回', desc: '计划被驳回，请修改后重新提交' },
  };
});

const canEdit = computed(() =>
  planStatus.value === 'none' || planStatus.value === 'draft' || planStatus.value === 'rejected',
);
const canSubmit = computed(() => planStatus.value === 'draft' || planStatus.value === 'rejected');
const canModify = computed(() => planStatus.value === 'approved');

const drawerFooter = computed(() => {
  const buttons = [];
  if (canEdit.value) {
    buttons.push({ text: '保存草稿', type: 'default', action: handleSaveDraft });
  }
  if (canSubmit.value) {
    buttons.push({ text: '提交审批', type: 'primary', action: handleSubmit });
  }
  if (canModify.value && !showModifyInput.value) {
    buttons.push({ text: '申请修改', type: 'default', action: () => (showModifyInput.value = true) });
  }
  return buttons;
});

// 月份名称
const monthNames = ['一月', '二月', '三月', '四月', '五月', '六月', '七月', '八月', '九月', '十月', '十一月', '十二月'];
</script>

<template>
  <Drawer
    :open="visible"
    title="个人销售计划设置"
    width="820px"
    :body-style="{ padding: '0' }"
    @close="emit('update:visible', false)"
  >
    <Spin :spinning="loading">
      <!-- 顶部信息区 -->
      <div class="border-b bg-gradient-to-r from-blue-50 to-indigo-50 p-5">
        <div class="mb-3 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <IconifyIcon icon="lucide:calendar-days" class="text-lg text-primary" />
            <span class="text-base font-semibold">{{ year }} 年销售计划</span>
            <Tag :color="statusConfig[planStatus].color">{{ statusConfig[planStatus].text }}</Tag>
          </div>
          <Button size="small" @click="emit('update:visible', false)">
            <template #icon><IconifyIcon icon="lucide:x" /></template>
            关闭
          </Button>
        </div>
        <Alert :message="statusConfig[planStatus].desc" type="info" show-icon />
        <Alert
          v-if="isCurrentYearHire"
          :message="`您于 ${year} 年 ${hireMonth} 月入职，目标从 ${hireMonth} 月开始设置，之前月份不可编辑`"
          type="warning"
          show-icon
          class="mt-2"
        />
      </div>

      <!-- 批量操作工具栏 -->
      <div v-if="canEdit" class="border-b bg-white p-4">
        <div class="mb-2 flex items-center gap-2 text-sm font-medium text-gray-700">
          <IconifyIcon icon="lucide:wand-2" class="text-primary" />
          <span>快捷批量操作</span>
          <span class="text-xs font-normal text-gray-400">（仅作用于可编辑月份）</span>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <!-- 字段选择 -->
          <select
            v-model="batchField"
            class="batch-select"
          >
            <option value="contractTargetAmount">合同金额</option>
            <option value="paymentTargetAmount">回款金额</option>
            <option value="contractTargetCount">合同数量</option>
          </select>

          <!-- 模式切换 -->
          <div class="flex rounded-md border border-gray-300 overflow-hidden">
            <button
              :class="['batch-mode-btn', batchMode === 'applyAll' ? 'batch-mode-active' : '']"
              @click="batchMode = 'applyAll'"
            >
              应用到所有月份
            </button>
            <button
              :class="['batch-mode-btn', batchMode === 'splitTotal' ? 'batch-mode-active' : '']"
              @click="batchMode = 'splitTotal'"
            >
              按总数分解
            </button>
          </div>

          <!-- 数值输入 -->
          <InputNumber
            v-model:value="batchValue"
            :min="0"
            :step="batchField === 'contractTargetCount' ? 1 : 10000"
            :placeholder="batchMode === 'applyAll' ? `每月${batchFieldLabels[batchField]}` : `总${batchFieldLabels[batchField]}`"
            style="width: 180px"
          />
          <span class="text-xs text-gray-400">{{ batchFieldUnits[batchField] }}</span>

          <!-- 执行按钮 -->
          <Button type="primary" size="small" @click="applyBatch">
            <template #icon><IconifyIcon icon="lucide:check" /></template>
            应用
          </Button>

          <!-- 清零 -->
          <Tooltip title="清零所有可编辑月份的数据">
            <Button size="small" danger ghost @click="clearAll">
              <template #icon><IconifyIcon icon="lucide:eraser" /></template>
              清零
            </Button>
          </Tooltip>

          <!-- 提示 -->
          <span class="ml-auto text-xs text-gray-400">
            <IconifyIcon icon="lucide:info" class="mr-1" />
            <template v-if="batchMode === 'applyAll'">将输入值填入每个可编辑月份</template>
            <template v-else>将总数平均分解到每个可编辑月份，余数分配到前几个月</template>
          </span>
        </div>
      </div>

      <!-- 月度目标设置表 -->
      <div class="p-4">
        <!-- approved 状态只读锁定提示 -->
        <Alert
          v-if="planStatus === 'approved'"
          message="计划已审批通过，数据已锁定"
          description="审批通过的计划不可直接编辑。如需调整目标，请点击底部「申请修改」按钮发起变更审批，经上级审批通过后方可修改。"
          type="warning"
          show-icon
          class="mb-4"
        />

        <!-- 审批节点链（展示完整审批流程） -->
        <Card
          v-if="planDetail?.approvalNodes?.length"
          size="small"
          class="mb-4"
        >
          <template #title>
            <div class="flex items-center gap-2">
              <IconifyIcon icon="lucide:git-branch" />
              <span>审批流程</span>
            </div>
          </template>
          <Timeline>
            <TimelineItem
              v-for="node in planDetail.approvalNodes"
              :key="node.id"
              :color="node.status === 1 ? 'green' : node.status === 2 ? 'red' : 'blue'"
            >
              <div class="flex items-center gap-2">
                <span class="font-medium">第{{ node.level }}级</span>
                <span>{{ node.approverName }}</span>
                <Tag
                  v-if="node.status === 0"
                  color="processing"
                  size="small"
                >待审批</Tag>
                <Tag v-else-if="node.status === 1" color="success" size="small">已通过</Tag>
                <Tag v-else-if="node.status === 2" color="error" size="small">已驳回</Tag>
                <Tag v-else-if="node.status === 3" color="default" size="small">已跳过</Tag>
              </div>
              <div v-if="node.comment" class="mt-1 text-xs text-gray-500">
                意见：{{ node.comment }}
              </div>
            </TimelineItem>
          </Timeline>
        </Card>

        <Card size="small" class="mb-4">
          <template #title>
            <div class="flex items-center gap-2">
              <IconifyIcon icon="lucide:edit-3" />
              <span>月度目标设置</span>
            </div>
          </template>
          <template #extra>
            <span class="text-xs text-gray-400">单位：元 / 个</span>
          </template>

          <!-- 表头 -->
          <div class="target-row target-header">
            <div class="target-cell target-month">月份</div>
            <div class="target-cell target-amount">合同金额目标</div>
            <div class="target-cell target-amount">回款金额目标</div>
            <div class="target-cell target-count">合同数量</div>
          </div>

          <!-- 月份行 -->
          <div
            v-for="m in monthlyTargets"
            :key="m.month"
            class="target-row"
            :class="{ 'target-row-disabled': !m.editable || !canEdit }"
          >
            <div class="target-cell target-month">
              <span class="text-sm font-medium">{{ monthNames[m.month - 1] }}</span>
              <Tag v-if="!m.editable" size="small" color="default" class="ml-1">入职前</Tag>
            </div>
            <div class="target-cell target-amount">
              <InputNumber
                v-model:value="m.contractTargetAmount"
                :disabled="!m.editable || !canEdit"
                :min="0"
                :step="10000"
                placeholder="0"
                style="width: 100%"
              />
              <span v-if="toWan(m.contractTargetAmount)" class="wan-hint">{{ toWan(m.contractTargetAmount) }}</span>
            </div>
            <div class="target-cell target-amount">
              <InputNumber
                v-model:value="m.paymentTargetAmount"
                :disabled="!m.editable || !canEdit"
                :min="0"
                :step="10000"
                placeholder="0"
                style="width: 100%"
              />
              <span v-if="toWan(m.paymentTargetAmount)" class="wan-hint">{{ toWan(m.paymentTargetAmount) }}</span>
            </div>
            <div class="target-cell target-count">
              <InputNumber
                v-model:value="m.contractTargetCount"
                :disabled="!m.editable || !canEdit"
                :min="0"
                :step="1"
                placeholder="0"
                style="width: 100%"
              />
            </div>
          </div>
        </Card>

        <!-- 全年累加汇总 -->
        <Card size="small" class="mb-4 summary-card" :body-style="{ background: 'linear-gradient(135deg, #f0f7ff 0%, #f5f0ff 100%)' }">
          <div class="grid grid-cols-3 gap-4 text-center">
            <div class="summary-item">
              <div class="text-xs text-gray-500 mb-1">全年合同目标累加</div>
              <div class="text-xl font-bold text-blue-600">{{ formatCurrency(totalContractAmount) }}</div>
            </div>
            <div class="summary-item">
              <div class="text-xs text-gray-500 mb-1">全年回款目标累加</div>
              <div class="text-xl font-bold text-purple-600">{{ formatCurrency(totalPaymentAmount) }}</div>
            </div>
            <div class="summary-item">
              <div class="text-xs text-gray-500 mb-1">全年合同数量</div>
              <div class="text-xl font-bold text-orange-500">{{ totalContractCount }} 个</div>
            </div>
          </div>
        </Card>

        <!-- 申请修改输入框 -->
        <Card v-if="showModifyInput" size="small" class="mb-4">
          <template #title>
            <span class="text-red-500">申请修改已审批通过的计划</span>
          </template>
          <Input.TextArea
            v-model:value="modifyReason"
            :rows="3"
            placeholder="请填写修改原因，提交后将进入审批流程"
          />
          <div class="mt-2 flex justify-end gap-2">
            <Button size="small" @click="showModifyInput = false">取消</Button>
            <Button size="small" type="primary" :loading="submitting" @click="handleApplyModify">
              提交修改申请
            </Button>
          </div>
        </Card>

        <!-- 审批记录 -->
        <Card v-if="approvalLogs.length > 0" size="small">
          <template #title>
            <div class="flex items-center gap-2">
              <IconifyIcon icon="lucide:history" />
              <span>审批记录</span>
            </div>
          </template>
          <Timeline>
            <TimelineItem
              v-for="log in approvalLogs"
              :key="log.id"
              :color="log.action === 2 ? 'green' : log.action === 3 ? 'red' : 'blue'"
            >
              <div class="font-medium">
                {{ { 1: '提交审批', 2: '审批通过', 3: '驳回', 4: '申请修改' }[log.action] || '操作' }}
              </div>
              <div class="text-xs text-gray-500">
                {{ log.operatorName }} · {{ log.createTime }}
              </div>
              <div v-if="log.reason" class="mt-1 text-sm text-gray-600">
                原因：{{ log.reason }}
              </div>
            </TimelineItem>
          </Timeline>
        </Card>
      </div>
    </Spin>

    <!-- 底部按钮 -->
    <template #footer>
      <div class="flex justify-end gap-2">
        <Button @click="emit('update:visible', false)">取消</Button>
        <template v-for="btn in drawerFooter" :key="btn.text">
          <Button :type="btn.type as any" :loading="submitting" @click="btn.action">
            {{ btn.text }}
          </Button>
        </template>
      </div>
    </template>
  </Drawer>
</template>

<style scoped>
/* 批量操作下拉选择 */
.batch-select {
  height: 28px;
  padding: 0 8px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  font-size: 13px;
  background: #fff;
  cursor: pointer;
  outline: none;
  transition: border-color 0.2s;
}
.batch-select:hover {
  border-color: #4096ff;
}
.batch-select:focus {
  border-color: #4096ff;
  box-shadow: 0 0 0 2px rgba(5, 145, 255, 0.1);
}

/* 模式切换按钮 */
.batch-mode-btn {
  padding: 4px 12px;
  font-size: 13px;
  background: #fff;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
  color: #595959;
}
.batch-mode-btn:not(:last-child) {
  border-right: 1px solid #d9d9d9;
}
.batch-mode-btn:hover {
  background: #f0f7ff;
  color: #4096ff;
}
.batch-mode-active {
  background: #1677ff !important;
  color: #fff !important;
}
.batch-mode-active:hover {
  background: #4096ff !important;
  color: #fff !important;
}

/* 月度目标表格行 */
.target-row {
  display: grid;
  grid-template-columns: 140px 1fr 1fr 1fr;
  gap: 8px;
  align-items: center;
  padding: 8px;
  border-radius: 6px;
  transition: background 0.2s;
}
.target-row:hover {
  background: #f5f7fa;
}
.target-row-disabled {
  opacity: 0.45;
}
.target-row-disabled:hover {
  background: transparent;
}

/* 表头行 */
.target-header {
  font-size: 12px;
  color: #8c8c8c;
  font-weight: 500;
  border-bottom: 1px solid #f0f0f0;
  padding-bottom: 8px;
  margin-bottom: 4px;
}
.target-header:hover {
  background: transparent;
}

.target-cell {
  display: flex;
  align-items: center;
}
.target-month {
  justify-content: flex-start;
}
.target-amount {
  position: relative;
  flex-direction: column;
  align-items: stretch;
}

/* 万元换算提示 */
.wan-hint {
  position: absolute;
  right: 4px;
  bottom: -14px;
  font-size: 10px;
  color: #bfbfbf;
  pointer-events: none;
  line-height: 1;
}

/* 汇总卡片 */
.summary-card {
  border: 1px solid #e6f4ff;
}
.summary-item {
  padding: 4px 0;
}

/* 滚动条美化 */
:deep(.ant-drawer-body) {
  scrollbar-width: thin;
  scrollbar-color: #d9d9d9 transparent;
}
:deep(.ant-drawer-body::-webkit-scrollbar) {
  width: 6px;
}
:deep(.ant-drawer-body::-webkit-scrollbar-thumb) {
  background: #d9d9d9;
  border-radius: 3px;
}
</style>
