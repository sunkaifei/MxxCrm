<template>
  <a-card class="mb-3" size="small" :bordered="false">
    <div class="flex items-center justify-between flex-wrap gap-2">
      <div class="flex items-center gap-4 flex-1">
        <div class="flex items-center gap-2 flex-shrink-0">
          <LucideInfo class="text-blue-500 process-icon-info" />
          <span class="font-medium text-sm">销售流程：</span>
        </div>
        <div class="flex items-center gap-1 flex-wrap process-flow">
          <template v-for="(step, index) in steps" :key="step.key">
            <div
              class="process-flow-item flex items-center gap-1.5 px-3 py-1 rounded-full text-sm cursor-pointer transition-all"
              :class="getFlowItemClass(step.key)"
              @click="goToStep(step.key)"
            >
              <component :is="step.icon" class="flow-icon" />
              <span>{{ step.title }}</span>
            </div>
            <div v-if="index < steps.length - 1" class="flow-arrow text-gray-300">
              <LucideArrowRight class="arrow-icon" />
            </div>
          </template>
        </div>
      </div>
      <div class="flex items-center gap-2 flex-shrink-0">
        <span class="text-gray-500 text-xs">{{ tipText }}</span>
        <a-button type="link" size="small" @click="expanded = !expanded">
          {{ expanded ? '收起' : '详细说明' }}
          <component :is="expanded ? LucideChevronUp : LucideChevronDown" class="inline-icon" />
        </a-button>
      </div>
    </div>
    <div v-if="expanded" class="mt-3 pt-3 border-t border-gray-100">
      <div class="grid grid-cols-2 md:grid-cols-3 gap-x-4 gap-y-2 text-sm">
        <div v-for="step in steps" :key="step.key" class="flex items-start gap-2 p-2 rounded-lg" :class="getStepDetailClass(step.key)">
          <div class="flex-shrink-0 w-6 h-6 rounded-full flex items-center justify-center" :class="getStepBgClass(step.key)">
            <component :is="step.icon" class="step-icon-sm" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="font-medium" :class="getStepTitleColor(step.key)">{{ step.title }}</div>
            <div class="text-gray-500 text-xs mt-0.5">{{ step.desc }}</div>
            <div v-if="step.nextTip && isCurrentStep(step.key)" class="text-blue-600 text-xs mt-1 flex items-center gap-1">
              <LucideArrowRight class="tip-arrow-icon" />
              <span>{{ step.nextTip }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </a-card>
</template>

<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { Button, Card } from 'ant-design-vue';
import {
  LucideInfo,
  LucideChevronDown,
  LucideChevronUp,
  LucideArrowRight,
  LucideTarget,
  LucideFileText,
  LucideFileSignature,
  LucideShoppingCart,
  LucideWallet,
  LucideReceipt,
} from '@vben/icons';

type StepKey = 'opportunity' | 'quotation' | 'contract' | 'order' | 'payment' | 'invoice';

interface ProcessStep {
  key: StepKey;
  title: string;
  desc: string;
  nextTip?: string;
  route: string;
  icon: any;
}

const props = defineProps<{
  currentStep: StepKey;
}>();

const router = useRouter();
const expanded = ref(false);

const steps: ProcessStep[] = [
  {
    key: 'opportunity',
    title: '商机',
    desc: '识别销售机会，跟进客户需求',
    nextTip: '确定合作意向后，创建报价单',
    route: '/sale/opportunity',
    icon: LucideTarget,
  },
  {
    key: 'quotation',
    title: '报价单',
    desc: '向客户提供产品报价和方案',
    nextTip: '客户确认报价后，签订合同',
    route: '/sale/quotation',
    icon: LucideFileText,
  },
  {
    key: 'contract',
    title: '合同',
    desc: '签订正式合同，约定付款条款',
    nextTip: '合同签订后，根据合同创建销售订单',
    route: '/sale/contract',
    icon: LucideFileSignature,
  },
  {
    key: 'order',
    title: '订单',
    desc: '创建销售订单，安排发货交付',
    nextTip: '发货后，跟进客户回款',
    route: '/sale/order',
    icon: LucideShoppingCart,
  },
  {
    key: 'payment',
    title: '回款',
    desc: '登记客户回款，核销应收款项',
    nextTip: '收到回款后，为客户开具发票',
    route: '/sale/payment',
    icon: LucideWallet,
  },
  {
    key: 'invoice',
    title: '发票',
    desc: '开具发票，完成销售闭环',
    route: '/sale/invoice',
    icon: LucideReceipt,
  },
];

const currentStepIndex = computed(() => {
  return steps.findIndex((s) => s.key === props.currentStep);
});

const tipText = computed(() => {
  const idx = currentStepIndex.value;
  if (idx < 0) return '';
  if (idx >= steps.length - 1) return '销售流程已完成';
  const nextStep = steps[idx + 1];
  return `当前在「${steps[idx].title}」环节，下一步：${nextStep.title}`;
});

function isCurrentStep(key: StepKey) {
  return props.currentStep === key;
}

function isPastStep(key: StepKey) {
  const idx = steps.findIndex((s) => s.key === key);
  return idx < currentStepIndex.value;
}

function getFlowItemClass(key: StepKey) {
  if (isCurrentStep(key)) return 'bg-blue-500 text-white shadow-sm';
  if (isPastStep(key)) return 'bg-green-50 text-green-700 hover:bg-green-100';
  return 'bg-gray-100 text-gray-500 hover:bg-gray-200';
}

function getStepDetailClass(key: StepKey) {
  if (isCurrentStep(key)) return 'bg-blue-50';
  return '';
}

function getStepBgClass(key: StepKey) {
  if (isPastStep(key)) return 'bg-green-500';
  if (isCurrentStep(key)) return 'bg-blue-500';
  return 'bg-gray-300';
}

function getStepTitleColor(key: StepKey) {
  if (isCurrentStep(key)) return 'text-blue-700';
  return 'text-gray-700';
}

function goToStep(key: StepKey) {
  const step = steps.find((s) => s.key === key);
  if (step?.route) {
    router.push(step.route);
  }
}
</script>

<style scoped>
.process-icon-info {
  width: 18px;
  height: 18px;
}

.process-flow-item {
  font-size: 13px;
  white-space: nowrap;
}

.flow-icon {
  width: 14px;
  height: 14px;
}

.flow-arrow {
  display: flex;
  align-items: center;
}

.arrow-icon {
  width: 16px;
  height: 16px;
}

.inline-icon {
  width: 14px;
  height: 14px;
  display: inline-block;
  vertical-align: middle;
}

.step-icon-sm {
  width: 14px;
  height: 14px;
  color: white;
}

.tip-arrow-icon {
  width: 12px;
  height: 12px;
  flex-shrink: 0;
}
</style>
