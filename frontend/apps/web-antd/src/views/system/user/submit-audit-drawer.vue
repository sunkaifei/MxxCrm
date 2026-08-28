<script lang="ts" setup>
// 提交入职审批抽屉（F2·审核工作台）：员工本人 / HR 代提交入职审批前的确认页
// 业务详情区（头部识别/档案完善度/员工信息/审批流程/审批记录/撤销）复用 HireApprovalDetail 组件，
// 与审批工作台（todo）详情保持完全一致的展示，此处仅负责提交动作。
import { computed, ref, watch } from 'vue';

import { Button, Drawer, message, Select } from 'ant-design-vue';

import { submitApprovalApi } from '#/api';

import HireApprovalDetail from '../approval/hire-approval-detail.vue';

const props = defineProps<{
  row: any;
  visible: boolean;
  /** 档案完善度四要素（个人中心传入；HR 代提交无此数据时隐藏该区） */
  completeness?: { label: string; done: boolean }[];
}>();

const emit = defineEmits<{
  success: [];
  'update:visible': [val: boolean];
}>();

// 默认流程编码：系统内置「员工入职审批」流程
const DEFAULT_FLOW_CODE = 'hire_approval';

// 最大化状态：默认 75% 宽度，最大化到 100%
const isMaximized = ref(false);
const drawerWidth = computed(() => (isMaximized.value ? '100%' : '75%'));

function toggleMaximize() {
  isMaximized.value = !isMaximized.value;
}
function handleClose() {
  emit('update:visible', false);
}

// 本地控制 Drawer 开关：父组件打开时同步打开
const drawerOpen = ref(false);
watch(
  () => props.visible,
  (val) => {
    drawerOpen.value = val;
  },
  { immediate: true },
);

watch(drawerOpen, (val) => {
  if (!val) {
    isMaximized.value = false;
    emit('update:visible', false);
  }
});

// 流程编码：优先行数据 → 系统默认 hire_approval
const flowCode = computed(() => props.row?.flowCode || DEFAULT_FLOW_CODE);

// ===== 合同信息（提交入职审批时填写，随提交写入员工档案并随审批实例留痕）=====
const CONTRACT_TYPE_OPTIONS = [
  { label: '固定期限', value: 1 },
  { label: '无固定期限', value: 2 },
];
const CONTRACT_MONTHS_OPTIONS = [
  { label: '6 个月（试用期上限 1 月）', value: 6 },
  { label: '1 年（试用期上限 2 月）', value: 12 },
  { label: '2 年（试用期上限 2 月）', value: 24 },
  { label: '3 年（试用期上限 6 月）', value: 36 },
  { label: '5 年（试用期上限 6 月）', value: 60 },
];
const contractType = ref<number | undefined>(undefined);
const contractMonths = ref<number | undefined>(undefined);
// 无固定期限合同依法不约定合同期限，隐藏期限选择
const showContractMonths = computed(() => contractType.value !== 2);
// 切换合同类型时联动清空/回填期限
watch(contractType, (val, old) => {
  if (val === 2) {
    contractMonths.value = undefined;
  } else if (old === 2 && contractMonths.value === undefined) {
    contractMonths.value = 36;
  }
});
// 抽屉打开时预填员工档案已有合同信息（HR 已维护则回显，避免重复填写）
watch(
  () => props.visible,
  (val) => {
    if (val) {
      contractType.value = props.row?.contractType ?? 1;
      contractMonths.value =
        props.row?.contractType === 2 ? undefined : (props.row?.contractMonths ?? 36);
    }
  },
);

const submitting = ref(false);

async function handleSubmit() {
  const row = props.row;
  if (!row?.id) return;
  // 合同信息提交前必填：固定期限必选期限，无固定期限无期限
  if (!contractType.value) {
    message.warning('请选择合同类型');
    return;
  }
  if (contractType.value !== 2 && !contractMonths.value) {
    message.warning('请选择合同期限');
    return;
  }
  submitting.value = true;
  try {
    await submitApprovalApi({
      flowCode: flowCode.value,
      businessType: 'user',
      businessId: row.id,
      businessTitle: row.nickName || row.userName || `用户#${row.id}`,
      // 合同信息：经 extra_data 提交，后端写入员工档案并随审批实例留痕
      extraData: {
        contractType: contractType.value,
        contractMonths: contractType.value === 2 ? null : contractMonths.value,
      },
    });
    message.success('已提交审核，等待审批人处理');
    handleClose();
    emit('success');
  } catch (error: any) {
    message.error(error?.message || '提交失败');
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <Drawer
    v-model:open="drawerOpen"
    :title="'提交入职审批'"
    placement="right"
    :width="drawerWidth"
    :closable="false"
    :header-style="{ borderBottom: '1px solid hsl(var(--border))', padding: '16px 24px' }"
  >
    <template #extra>
      <div class="flex items-center gap-1">
        <Button type="text" size="small" @click="toggleMaximize">
          {{ isMaximized ? '⤓ 还原' : '⤢' }}
        </Button>
        <Button type="text" size="small" @click="handleClose">
          <svg
            class="w-4 h-4"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </Button>
      </div>
    </template>

    <div class="flex h-full flex-col">
      <!-- 员工入职审批详情（与审批工作台共用） -->
      <HireApprovalDetail
        :row="row"
        :instance-id="row?.approvalInstanceId ? Number(row.approvalInstanceId) : undefined"
        :flow-code="flowCode"
        :completeness="completeness"
        class="flex-1 overflow-y-auto px-6 py-5"
        @cancel-success="emit('success')"
      />

      <!-- ===== 合同信息（提交前必填，随提交写入员工档案并随审批实例留痕） ===== -->
      <div class="su-contract">
        <div class="su-contract-title">合同信息</div>
        <div class="su-contract-form">
          <div class="su-contract-item">
            <span class="su-contract-label">合同类型</span>
            <Select
              v-model:value="contractType"
              :options="CONTRACT_TYPE_OPTIONS"
              style="width: 180px"
              placeholder="请选择合同类型"
            />
          </div>
          <div v-if="showContractMonths" class="su-contract-item">
            <span class="su-contract-label">合同期限</span>
            <Select
              v-model:value="contractMonths"
              :options="CONTRACT_MONTHS_OPTIONS"
              style="width: 240px"
              placeholder="请选择合同期限"
            />
          </div>
        </div>
        <div class="su-contract-tip">
          合同信息将随审批提交写入员工档案，并供审批人在详情中复核
        </div>
      </div>

      <!-- ===== 底部操作区 ===== -->
      <div class="su-footer">
        <Button @click="handleClose">取消</Button>
        <Button type="primary" :loading="submitting" @click="handleSubmit">
          确认提交
        </Button>
      </div>
    </div>
  </Drawer>
</template>

<style scoped>
/* ===== 合同信息表单区 ===== */
.su-contract {
  padding: 12px 24px;
  border-top: 1px solid hsl(var(--border));
  background: hsl(var(--muted) / 0.3);
}

.su-contract-title {
  font-weight: 600;
  margin-bottom: 8px;
}

.su-contract-form {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}

.su-contract-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.su-contract-label {
  color: hsl(var(--foreground) / 0.65);
}

.su-contract-tip {
  margin-top: 8px;
  font-size: 12px;
  color: hsl(var(--foreground) / 0.45);
}

/* ===== 底部操作区 ===== */
.su-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  padding: 14px 24px;
  border-top: 1px solid hsl(var(--border));
}
</style>
