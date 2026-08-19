<script lang="ts" setup>
import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import { useUserStore } from '@vben/stores';

import {
  Button,
  DatePicker,
  Input,
  InputNumber,
  message,
  Modal,
  Select,
  Tag,
} from 'ant-design-vue';

import { createFollowupApi, createPaymentApi, processApprovalApi } from '#/api';
import { $t } from '#/locales';

defineOptions({
  name: 'QuickProcessModal',
});

const props = defineProps<{
  /** 当前点击的待办项 */
  todoItem?: any;
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'processed'): void;
  /** 查看审批流完整详情：在工作台内嵌打开审批抽屉 */
  (
    e: 'viewApproval',
    payload: {
      businessId: number;
      businessType: string;
      instanceId: number;
    },
  ): void;
}>();

const router = useRouter();
const userStore = useUserStore();

const submitting = ref(false);
const comment = ref('');
// 跟进表单
const followUpForm = ref({
  method: 1,
  nextFollowAt: '' as string,
  content: '',
});
// 回款表单
const paymentForm = ref({
  amount: 0,
  method: 1,
});

// 跟进方式选项
const followUpMethods = [
  { value: 1, label: '电话' },
  { value: 2, label: '拜访' },
  { value: 3, label: '邮件' },
  { value: 4, label: '会议' },
  { value: 5, label: 'WhatsApp' },
  { value: 6, label: '微信' },
  { value: 7, label: '其他' },
];

// 回款方式选项
const paymentMethods = [
  { value: 1, label: '银行转账' },
  { value: 2, label: '现金' },
  { value: 3, label: '支票' },
  { value: 4, label: '支付宝' },
  { value: 5, label: '微信' },
  { value: 6, label: '其他' },
];

const todoType = computed(() => props.todoItem?.type || '');

const visibleProxy = computed({
  get: () => props.visible,
  set: (val: boolean) => emit('update:visible', val),
});

const modalTitle = computed(() => {
  switch (todoType.value) {
    case 'approval': {
      return $t('page.dashboard.quickProcessApproval');
    }
    case 'contract': {
      return $t('page.dashboard.quickProcessContract');
    }
    case 'followUp': {
      return $t('page.dashboard.quickProcessFollowUp');
    }
    case 'opportunity': {
      return $t('page.dashboard.quickProcessOpportunity');
    }
    case 'payment': {
      return $t('page.dashboard.quickProcessPayment');
    }
    default: {
      return $t('page.dashboard.quickProcess');
    }
  }
});

// 重置表单数据
function resetForm() {
  comment.value = '';
  followUpForm.value = {
    method: 1,
    nextFollowAt: '',
    content: '',
  };
  paymentForm.value = { amount: 0, method: 1 };
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      resetForm();
    }
  },
);

function close() {
  emit('update:visible', false);
}

// 审批通过/驳回
async function handleApproval(action: 1 | 2) {
  if (!props.todoItem?.id) {
    message.error('待办项 ID 缺失');
    return;
  }
  if (action === 2 && !comment.value.trim()) {
    message.warning('请填写驳回原因');
    return;
  }
  submitting.value = true;
  try {
    const userInfo: any = userStore.userInfo;
    await processApprovalApi({
      instanceId: props.todoItem.id,
      action,
      approverId: userInfo?.userId,
      approverName: userInfo?.realName || userInfo?.username,
      comment: comment.value,
    });
    message.success(action === 1 ? '已审批通过' : '已驳回');
    emit('processed');
    close();
  } catch (error: any) {
    message.error(error?.message || '处理失败');
  } finally {
    submitting.value = false;
  }
}

// 保存跟进
async function handleSaveFollowUp() {
  if (!followUpForm.value.content.trim()) {
    message.warning('请填写跟进记录');
    return;
  }
  submitting.value = true;
  try {
    await createFollowupApi({
      itemType: props.todoItem?.itemType || 'customer',
      itemId: props.todoItem?.itemId || props.todoItem?.businessId,
      method: followUpForm.value.method,
      content: followUpForm.value.content,
      nextFollowAt: followUpForm.value.nextFollowAt || undefined,
    });
    message.success('跟进已保存');
    emit('processed');
    close();
  } catch (error: any) {
    message.error(error?.message || '保存失败');
  } finally {
    submitting.value = false;
  }
}

// 登记回款
async function handleSavePayment() {
  if (!paymentForm.value.amount || paymentForm.value.amount <= 0) {
    message.warning('请输入回款金额');
    return;
  }
  submitting.value = true;
  try {
    await createPaymentApi({
      contractId: props.todoItem?.contractId || props.todoItem?.businessId,
      contractTitle: props.todoItem?.title,
      amount: paymentForm.value.amount,
      method: paymentForm.value.method,
    });
    message.success('回款已登记');
    emit('processed');
    close();
  } catch (error: any) {
    message.error(error?.message || '登记失败');
  } finally {
    submitting.value = false;
  }
}

// 查看详情跳转
function goDetail() {
  const item = props.todoItem;
  if (!item) return;
  // 审批类型：通过 emit 在工作台内嵌打开审批抽屉，无需跳转
  if (todoType.value === 'approval') {
    const businessId = Number(item.businessId);
    const instanceId = Number(item.id);
    const businessType = item.businessType || '';
    if (!businessId || !instanceId) {
      message.warning('审批信息缺失，无法查看详情');
      return;
    }
    close();
    emit('viewApproval', { businessType, businessId, instanceId });
    return;
  }
  let path: string;
  switch (todoType.value) {
    case 'contract': {
      path = '/sale/contract';
      break;
    }
    case 'followUp': {
      path = item.itemType === 'lead' ? '/crm/lead' : '/crm/customer';
      break;
    }
    case 'opportunity': {
      path = '/sale/opportunity';
      break;
    }
    case 'payment': {
      path = '/sale/payment';
      break;
    }
    default: {
      return;
    }
  }
  close();
  router.push(path).catch(() => {
    // 跳转失败忽略
  });
}
</script>

<template>
  <Modal
    v-model:visible="visibleProxy"
    :title="modalTitle"
    width="560px"
    :footer="null"
    destroy-on-close
    @cancel="close"
  >
    <div v-if="todoItem" class="space-y-4">
      <!-- 审批类型 -->
      <template v-if="todoType === 'approval'">
        <div class="rounded bg-gray-50 p-3 text-sm">
          <div class="mb-1">
            <span class="text-gray-500">业务标题：</span>
            <span>{{ todoItem.businessTitle || '--' }}</span>
          </div>
          <div class="mb-1">
            <span class="text-gray-500">提交人：</span>
            <span>{{ todoItem.submitterName || '--' }}</span>
          </div>
          <div>
            <span class="text-gray-500">状态：</span>
            <Tag color="processing">
              {{ todoItem.statusName || '待审批' }}
            </Tag>
          </div>
        </div>
        <div>
          <div class="mb-1 text-sm text-gray-600">审批意见</div>
          <Input.TextArea
            v-model:value="comment"
            :rows="4"
            placeholder="请填写审批意见（驳回时必填）"
          />
        </div>
        <div class="flex justify-end gap-2">
          <Button :loading="submitting" danger @click="handleApproval(2)">
            驳回
          </Button>
          <Button
            type="primary"
            :loading="submitting"
            @click="handleApproval(1)"
          >
            通过
          </Button>
        </div>
      </template>

      <!-- 跟进类型 -->
      <template v-else-if="todoType === 'followUp'">
        <div class="rounded bg-gray-50 p-3 text-sm">
          <div class="mb-1">
            <span class="text-gray-500">名称：</span>
            <span>{{ todoItem.name || todoItem.businessTitle || '--' }}</span>
          </div>
          <div>
            <span class="text-gray-500">逾期天数：</span>
            <Tag color="red">{{ todoItem.overdueDays || 0 }} 天</Tag>
          </div>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <div class="mb-1 text-sm text-gray-600">跟进方式</div>
            <Select
              v-model:value="followUpForm.method"
              class="w-full"
              :options="followUpMethods"
            />
          </div>
          <div>
            <div class="mb-1 text-sm text-gray-600">下次跟进日期</div>
            <DatePicker
              v-model:value="followUpForm.nextFollowAt"
              class="w-full"
              value-format="YYYY-MM-DD HH:mm:ss"
              :show-time="{ format: 'HH:mm' }"
              format="YYYY-MM-DD HH:mm"
            />
          </div>
        </div>
        <div>
          <div class="mb-1 text-sm text-gray-600">跟进记录</div>
          <Input.TextArea
            v-model:value="followUpForm.content"
            :rows="4"
            placeholder="请填写跟进记录"
          />
        </div>
        <div class="flex justify-end">
          <Button
            type="primary"
            :loading="submitting"
            @click="handleSaveFollowUp"
          >
            保存跟进
          </Button>
        </div>
      </template>

      <!-- 待回款类型 -->
      <template v-else-if="todoType === 'payment'">
        <div class="rounded bg-gray-50 p-3 text-sm">
          <div class="mb-1">
            <span class="text-gray-500">合同标题：</span>
            <span>{{ todoItem.title || todoItem.businessTitle || '--' }}</span>
          </div>
          <div class="mb-1">
            <span class="text-gray-500">计划金额：</span>
            <span>¥{{ todoItem.planAmount || 0 }}</span>
          </div>
          <div>
            <span class="text-gray-500">已收金额：</span>
            <span>¥{{ todoItem.receivedAmount || 0 }}</span>
          </div>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <div class="mb-1 text-sm text-gray-600">回款金额</div>
            <InputNumber
              v-model:value="paymentForm.amount"
              class="w-full"
              :min="0"
              :precision="2"
              placeholder="请输入金额"
            />
          </div>
          <div>
            <div class="mb-1 text-sm text-gray-600">回款方式</div>
            <Select
              v-model:value="paymentForm.method"
              class="w-full"
              :options="paymentMethods"
            />
          </div>
        </div>
        <div class="flex justify-end">
          <Button
            type="primary"
            :loading="submitting"
            @click="handleSavePayment"
          >
            登记回款
          </Button>
        </div>
      </template>

      <!-- 合同到期类型 -->
      <template v-else-if="todoType === 'contract'">
        <div class="rounded bg-gray-50 p-3 text-sm">
          <div class="mb-1">
            <span class="text-gray-500">合同编号：</span>
            <span>{{ todoItem.contractNo || '--' }}</span>
          </div>
          <div class="mb-1">
            <span class="text-gray-500">标题：</span>
            <span>{{ todoItem.title || '--' }}</span>
          </div>
          <div class="mb-1">
            <span class="text-gray-500">到期日期：</span>
            <span>{{ todoItem.endDate || '--' }}</span>
          </div>
          <div>
            <span class="text-gray-500">剩余天数：</span>
            <Tag color="orange">{{ todoItem.remainingDays ?? 0 }} 天</Tag>
          </div>
        </div>
        <div class="flex justify-end">
          <Button type="primary" @click="goDetail">查看详情</Button>
        </div>
      </template>

      <!-- 停滞商机类型 -->
      <template v-else-if="todoType === 'opportunity'">
        <div class="rounded bg-gray-50 p-3 text-sm">
          <div class="mb-1">
            <span class="text-gray-500">商机名称：</span>
            <span>{{ todoItem.title || '--' }}</span>
          </div>
          <div class="mb-1">
            <span class="text-gray-500">阶段：</span>
            <span>{{ todoItem.stageName || '--' }}</span>
          </div>
          <div class="mb-1">
            <span class="text-gray-500">最后更新：</span>
            <span>{{ todoItem.updateTime || '--' }}</span>
          </div>
          <div>
            <span class="text-gray-500">停滞天数：</span>
            <Tag color="red">{{ todoItem.stagnantDays || 0 }} 天</Tag>
          </div>
        </div>
        <div class="flex justify-end">
          <Button type="primary" @click="goDetail">查看详情</Button>
        </div>
      </template>
    </div>

    <!-- 底部固定链接 -->
    <div class="mt-4 border-t border-dashed border-gray-200 pt-3 text-center">
      <a
        class="cursor-pointer text-sm text-blue-500 hover:underline"
        @click="goDetail"
      >
        {{ $t('page.dashboard.viewDetail') }} →
      </a>
    </div>
  </Modal>
</template>
