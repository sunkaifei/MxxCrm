<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import {
  Button,
  Input,
  InputNumber,
  Modal,
  Radio,
  RadioGroup,
  Table,
  message,
} from 'ant-design-vue';

import { allocateCommissionApi } from '#/api/core/finance';
import { $t } from '#/locales';

const props = defineProps<{
  visible: boolean;
  record?: any;
}>();

const emit = defineEmits<{
  (e: 'close', success?: boolean): void;
}>();

const loading = ref(false);
const allocateMethod = ref<number>(3); // 默认手动
const members = ref<any[]>([]);

// 待分配总额
const totalAmount = computed(() =>
  Number(props.record?.commissionAmount || props.record?.commission_amount || 0),
);

// 已分配总额
const allocatedTotal = computed(() =>
  members.value.reduce((sum, m) => sum + Number(m.amount || 0), 0),
);

// 剩余
const remaining = computed(() => totalAmount.value - allocatedTotal.value);

// 是否超限
const isOverLimit = computed(() => allocatedTotal.value > totalAmount.value);

// 分配方式选项
const methodOptions = computed(() => [
  { value: 1, label: $t('page.finance.teamCommission.allocate.average') },
  {
    value: 2,
    label: $t('page.finance.teamCommission.allocate.byPerformance'),
  },
  { value: 3, label: $t('page.finance.teamCommission.allocate.manual') },
]);

// 表格列
const columns = computed(() => {
  const cols: any[] = [
    {
      title: $t('page.finance.teamCommission.allocate.member'),
      dataIndex: 'employeeName',
      key: 'employeeName',
      width: 140,
    },
  ];

  // 按业绩比例时显示业绩列
  if (allocateMethod.value === 2) {
    cols.push({
      title: $t('page.finance.teamCommission.allocate.payment'),
      dataIndex: 'employeePayment',
      key: 'employeePayment',
      width: 140,
    });
  }

  // 非平均分配时显示金额输入列
  if (allocateMethod.value !== 1) {
    cols.push({
      title: $t('page.finance.teamCommission.allocate.amount'),
      dataIndex: 'amount',
      key: 'amount',
      width: 160,
    });
  } else {
    cols.push({
      title: $t('page.finance.teamCommission.allocate.amount'),
      dataIndex: 'amountDisplay',
      key: 'amountDisplay',
      width: 120,
    });
  }

  cols.push({
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 80,
  });

  return cols;
});

// 重新计算分配金额
function recalcAllocation() {
  if (allocateMethod.value === 1) {
    // 平均分配
    const count = members.value.length;
    if (count > 0) {
      const perPerson = totalAmount.value / count;
      members.value.forEach((m) => {
        m.amount = Number(perPerson.toFixed(2));
      });
    }
  } else if (allocateMethod.value === 2) {
    // 按业绩比例
    const totalPayment = members.value.reduce(
      (sum, m) => sum + Number(m.employeePayment || 0),
      0,
    );
    if (totalPayment > 0) {
      members.value.forEach((m) => {
        const payment = Number(m.employeePayment || 0);
        m.amount = Number(((totalAmount.value * payment) / totalPayment).toFixed(2));
      });
    }
  }
  // 手动填写不自动计算
}

// 切换分配方式
function handleMethodChange() {
  recalcAllocation();
}

// 添加成员
function addMember() {
  members.value.push({
    employeeId: undefined,
    employeeName: '',
    employeePayment: 0,
    amount: 0,
  });
}

// 删除成员
function removeMember(index: number) {
  members.value.splice(index, 1);
  if (allocateMethod.value !== 3) {
    recalcAllocation();
  }
}

// 重置
function resetState() {
  allocateMethod.value = 3;
  members.value = [];
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      resetState();
      // 默认添加 2 个空行
      addMember();
      addMember();
    }
  },
);

// 提交分配
async function handleSubmit() {
  if (members.value.length === 0) {
    message.warning($t('page.finance.teamCommission.allocate.noMember'));
    return;
  }

  // 校验成员数据
  for (const m of members.value) {
    if (!m.employeeId) {
      message.warning($t('page.finance.teamCommission.allocate.selectMember'));
      return;
    }
  }

  if (isOverLimit.value) {
    message.warning($t('page.finance.teamCommission.allocate.overLimit'));
    return;
  }

  // 手动分配时校验合计
  if (allocateMethod.value === 3) {
    for (const m of members.value) {
      if (!m.amount || m.amount <= 0) {
        message.warning($t('page.finance.teamCommission.allocate.amount') + '必填');
        return;
      }
    }
  }

  loading.value = true;
  try {
    await allocateCommissionApi({
      commissionResultId: props.record?.id,
      allocateMethod: allocateMethod.value,
      members: members.value.map((m) => ({
        employeeId: Number(m.employeeId),
        employeeName: m.employeeName,
        amount: Number(m.amount || 0),
        employeePayment:
          allocateMethod.value === 2 ? Number(m.employeePayment || 0) : undefined,
      })),
    });
    message.success($t('page.finance.teamCommission.allocate.success'));
    emit('close', true);
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.failed'));
  } finally {
    loading.value = false;
  }
}

function handleClose() {
  emit('close');
}

function formatMoney(val: any) {
  return Number(val || 0).toFixed(2);
}
</script>

<template>
  <Modal
    :open="visible"
    :title="$t('page.finance.teamCommission.allocate.title')"
    :width="720"
    :mask-closable="false"
    :destroy-on-close="true"
    @cancel="handleClose"
  >
    <div class="allocate-summary">
      <div class="summary-item">
        <span class="summary-label">{{ $t('page.finance.teamCommission.allocate.total') }}</span>
        <span class="summary-value primary">¥{{ formatMoney(totalAmount) }}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">{{ $t('page.finance.teamCommission.allocate.allocatedTotal') }}</span>
        <span class="summary-value" :class="{ over: isOverLimit }">¥{{ formatMoney(allocatedTotal) }}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">{{ $t('page.finance.teamCommission.allocate.remaining') }}</span>
        <span class="summary-value" :class="{ 'text-red': remaining < 0 }">¥{{ formatMoney(remaining) }}</span>
      </div>
    </div>

    <div class="allocate-method">
      <span class="method-label">{{ $t('page.finance.teamCommission.allocate.method') }}:</span>
      <RadioGroup v-model:value="allocateMethod" @change="handleMethodChange">
        <Radio v-for="opt in methodOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </Radio>
      </RadioGroup>
    </div>

    <div class="mb-2">
      <Button type="dashed" size="small" @click="addMember">
        + {{ $t('page.finance.teamCommission.allocate.member') }}
      </Button>
    </div>

    <Table
      :data-source="members"
      :columns="columns"
      :pagination="false"
      row-key="employeeId"
      size="small"
      bordered
    >
      <template #bodyCell="{ column, index }">
        <template v-if="column.key === 'employeeName'">
          <div style="display: flex; gap: 4px;">
            <InputNumber
              v-model:value="members[index].employeeId"
              :placeholder="'ID'"
              :min="0"
              style="width: 80px"
            />
            <Input
              v-model:value="members[index].employeeName"
              :placeholder="$t('page.finance.teamCommission.allocate.member')"
              style="flex: 1"
            />
          </div>
        </template>
        <template v-else-if="column.key === 'employeePayment'">
          <InputNumber
            v-model:value="members[index].employeePayment"
            :min="0"
            :precision="2"
            style="width: 100%"
            @change="recalcAllocation"
          />
        </template>
        <template v-else-if="column.key === 'amount'">
          <InputNumber
            v-model:value="members[index].amount"
            :min="0"
            :precision="2"
            style="width: 100%"
            :disabled="allocateMethod !== 3"
          />
        </template>
        <template v-else-if="column.key === 'amountDisplay'">
          <span class="amount-display">¥{{ formatMoney(members[index].amount) }}</span>
        </template>
        <template v-else-if="column.key === 'action'">
          <Button type="link" danger size="small" @click="removeMember(index)">
            {{ $t('page.finance.common.delete') }}
          </Button>
        </template>
      </template>
    </Table>

    <div v-if="isOverLimit" class="over-limit-warn">
      {{ $t('page.finance.teamCommission.allocate.overLimit') }}
    </div>

    <template #footer>
      <div style="display: flex; justify-content: flex-end; gap: 8px;">
        <Button @click="handleClose">{{ $t('page.finance.common.cancel') }}</Button>
        <Button
          type="primary"
          :loading="loading"
          :disabled="isOverLimit || members.length === 0"
          @click="handleSubmit"
        >
          {{ $t('page.finance.teamCommission.allocate.confirm') }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<style scoped>
.allocate-summary {
  display: flex;
  gap: 32px;
  padding: 12px 16px;
  background: #f8fafc;
  border-radius: 6px;
  margin-bottom: 16px;
}

.summary-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.summary-label {
  font-size: 12px;
  color: #64748b;
}

.summary-value {
  font-size: 18px;
  font-weight: 700;
  color: #1e293b;
}

.summary-value.primary {
  color: #1677ff;
}

.summary-value.over {
  color: #dc2626;
}

.text-red {
  color: #dc2626 !important;
}

.allocate-method {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.method-label {
  font-size: 14px;
  font-weight: 600;
  color: #1e293b;
}

.amount-display {
  font-weight: 600;
  color: #1677ff;
}

.over-limit-warn {
  margin-top: 8px;
  padding: 8px 12px;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 4px;
  color: #dc2626;
  font-size: 13px;
}
</style>
