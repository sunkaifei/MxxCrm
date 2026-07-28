<template>
  <div class="sales-process-wrapper">
    <!-- 头部：标题 + 提示 + 操作按钮（独立一行） -->
    <div class="sales-process-header">
      <div class="flex items-center gap-2 flex-shrink-0">
        <LucideInfo class="process-icon-info" style="color: hsl(var(--primary))" />
        <span class="font-medium text-sm" style="color: hsl(var(--foreground))">销售流程</span>
      </div>
      <div class="flex items-center gap-2 flex-1 min-w-0 process-tip-wrapper">
        <span v-if="tipText" class="text-xs process-tip-text" style="color: hsl(var(--muted-foreground))">{{ tipText }}</span>
      </div>
      <Button type="link" size="small" @click="expanded = !expanded" class="!p-0 !h-auto flex-shrink-0">
        {{ expanded ? '收起' : '详细说明' }}
        <component :is="expanded ? LucideChevronUp : LucideChevronDown" class="inline-icon" />
      </Button>
    </div>

    <!-- 折叠态：横向步骤卡片条 -->
    <div v-show="!expanded" class="step-bar">
      <template v-for="(step, index) in steps" :key="step.key">
        <div
          class="step-bar-item"
          :class="getCollapsedStepClass(step.key)"
          @click="goToStep(step.key)"
        >
          <div class="step-bar-index" :class="getStepBadgeClass(step.key)">
            <component :is="step.icon" class="step-bar-icon" />
          </div>
          <span class="step-bar-title">{{ step.title }}</span>
        </div>
        <!-- 分隔箭头（最后一个不显示） -->
        <div v-if="index < steps.length - 1" class="step-bar-separator">
          <span class="step-bar-separator-text">&gt;</span>
        </div>
      </template>
    </div>

    <!-- 展开态：详细卡片 -->
    <div v-show="expanded" class="sales-process-detail">
      <div class="step-detail-grid">
        <div
          v-for="(step, idx) in steps"
          :key="step.key"
          class="step-detail-cell"
        >
          <div class="step-detail-card p-3 rounded-lg" :class="getStepCardClass(step.key)">
            <div class="flex items-center gap-2 mb-2">
              <div class="step-detail-index" :class="getStepBadgeClass(step.key)">
                <span class="step-detail-index-num">{{ idx + 1 }}</span>
              </div>
              <div class="flex items-center gap-1.5 flex-1 min-w-0">
                <component :is="step.icon" class="step-detail-icon" :style="getStepIconColor(step.key)" />
                <span class="font-medium text-sm truncate" :style="getStepTitleColor(step.key)">{{ step.title }}</span>
              </div>
            </div>
            <div class="text-xs leading-relaxed step-detail-desc" style="color: hsl(var(--muted-foreground))">{{ step.desc }}</div>
            <div v-if="step.nextTip && isCurrentStep(step.key)" class="mt-2 text-xs flex items-center gap-1 px-2 py-1 rounded inline-flex step-detail-tip" style="color: hsl(var(--primary)); background: hsl(var(--primary) / 0.08);">
              <LucideArrowRight class="tip-arrow-icon" />
              <span>{{ step.nextTip }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { Button } from 'ant-design-vue';
import {
  LucideInfo,
  LucideChevronDown,
  LucideChevronUp,
  LucideArrowRight,
  LucideTarget,
  LucideFileText,
  LucideFileSignature,
  LucideShoppingCart,
  LucideTruck,
  LucideWallet,
  LucideReceipt,
} from '@vben/icons';

type StepKey = 'opportunity' | 'quotation' | 'contract' | 'order' | 'shipment' | 'payment' | 'invoice';

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
    key: 'order',
    title: '订单',
    desc: '创建销售订单，安排发货交付',
    nextTip: '订单确认后，签订正式合同',
    route: '/sale/order',
    icon: LucideShoppingCart,
  },
  {
    key: 'contract',
    title: '合同',
    desc: '签订正式合同，约定付款条款',
    nextTip: '合同签订后，安排发货',
    route: '/sale/contract',
    icon: LucideFileSignature,
  },
  {
    key: 'shipment',
    title: '发货',
    desc: '创建发货单，跟踪物流配送',
    nextTip: '发货完成后，跟进客户回款',
    route: '/sale/shipment',
    icon: LucideTruck,
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
  const currentStepTitle = steps[idx]?.title ?? '';
  const nextStepTitle = nextStep?.title ?? '';
  return `当前在「${currentStepTitle}」环节，下一步：${nextStepTitle}`;
});

function isCurrentStep(key: StepKey) {
  return props.currentStep === key;
}

function isPastStep(key: StepKey) {
  const idx = steps.findIndex((s) => s.key === key);
  return idx < currentStepIndex.value;
}

// 折叠态步骤卡片样式
function getCollapsedStepClass(key: StepKey) {
  if (isCurrentStep(key)) return 'step-bar-current';
  if (isPastStep(key)) return 'step-bar-past';
  return 'step-bar-default';
}

// 步骤徽章背景色（图标圆圈）
function getStepBadgeClass(key: StepKey) {
  if (isCurrentStep(key)) return 'badge-current';
  if (isPastStep(key)) return 'badge-past';
  return 'badge-default';
}

// 展开态步骤卡片样式
function getStepCardClass(key: StepKey) {
  if (isCurrentStep(key)) return 'step-card-current';
  if (isPastStep(key)) return 'step-card-past';
  return 'step-card-default';
}

// 展开态步骤图标颜色
function getStepIconColor(key: StepKey) {
  if (isCurrentStep(key)) return { color: 'hsl(var(--primary))' };
  if (isPastStep(key)) return { color: 'hsl(142 71% 45%)' };
  return { color: 'hsl(var(--muted-foreground) / 0.6)' };
}

function getStepTitleColor(key: StepKey) {
  if (isCurrentStep(key)) return { color: 'hsl(var(--primary))' };
  if (isPastStep(key)) return { color: 'hsl(142 71% 35%)' };
  return { color: 'hsl(var(--muted-foreground))' };
}

function goToStep(key: StepKey) {
  const step = steps.find((s) => s.key === key);
  if (step?.route) {
    router.push(step.route);
  }
}
</script>

<style scoped>
/* ========== 外层统一卡片 ========== */
.sales-process-wrapper {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  padding: 10px 14px;
  box-shadow: 0 1px 2px hsl(0 0% 0% / 0.03);
  margin-bottom: 12px;
}

.sales-process-header {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 32px;
  padding-bottom: 8px;
  border-bottom: 1px dashed hsl(var(--border));
  margin-bottom: 10px;
}

.process-icon-info {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

.inline-icon {
  width: 14px;
  height: 14px;
  display: inline-block;
  vertical-align: middle;
}

.tip-arrow-icon {
  width: 12px;
  height: 12px;
  flex-shrink: 0;
}

.process-tip-wrapper {
  display: flex;
  align-items: center;
  overflow: hidden;
}

.process-tip-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

/* ========== 折叠态：横向步骤条 ========== */
.step-bar {
  display: flex;
  width: 100%;
  border-radius: 8px;
  overflow: hidden;
  align-items: stretch;
}

.step-bar-item {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 12px 8px;
  cursor: pointer;
  position: relative;
  transition: all 0.2s ease;
  min-width: 0;
  background: transparent;
  border-radius: 6px;
}

.step-bar-index {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.step-bar-icon {
  width: 14px;
  height: 14px;
  color: white;
}

.step-bar-title {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 折叠态：当前步骤 */
.step-bar-current {
  background: hsl(var(--primary) / 0.08);
}

.step-bar-current .step-bar-title {
  color: hsl(var(--primary));
  font-weight: 600;
}

.step-bar-current .step-bar-index {
  background: hsl(var(--primary));
  box-shadow: 0 0 0 3px hsl(var(--primary) / 0.12);
}

.step-bar-current:hover {
  background: hsl(var(--primary) / 0.15);
}

/* 折叠态：已完成步骤 */
.step-bar-past .step-bar-title {
  color: hsl(142 71% 45%);
}

.step-bar-past .step-bar-index {
  background: hsl(142 71% 45%);
}

.step-bar-past:hover {
  background: hsl(142 71% 45% / 0.08);
}

/* 折叠态：未开始步骤 */
.step-bar-default .step-bar-title {
  color: hsl(var(--muted-foreground));
}

.step-bar-default .step-bar-index {
  background: hsl(var(--muted-foreground) / 0.25);
}

.step-bar-default:hover {
  background: hsl(var(--muted) / 0.5);
}

.step-bar-default:hover .step-bar-index {
  background: hsl(var(--muted-foreground) / 0.4);
}

/* 步骤之间的分隔箭头 */
.step-bar-separator {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  padding: 0 4px;
  color: hsl(var(--muted-foreground) / 0.4);
}

.step-bar-separator-text {
  font-size: 16px;
  font-weight: 600;
  line-height: 1;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

/* ========== 展开态：详细卡片 ========== */
.sales-process-detail {
  animation: fadeSlideDown 0.25s ease-out;
  padding-top: 4px;
}

@keyframes fadeSlideDown {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Grid 布局：PC 默认 3 列，2 列，平板 2 列，手机 1 列 */
.step-detail-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  width: 100%;
}

.step-detail-cell {
  display: flex;
  min-width: 0;
}

.step-detail-cell > .step-detail-card {
  width: 100%;
}

.step-detail-card {
  background: hsl(var(--muted) / 0.5);
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  transition: all 0.2s ease;
  cursor: pointer;
  min-height: 110px;
  margin: 0;
}

.step-detail-card:hover {
  border-color: hsl(var(--muted-foreground) / 0.3);
  box-shadow: 0 2px 8px hsl(0 0% 0% / 0.06);
}

.step-detail-index {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.step-detail-index-num {
  font-size: 12px;
  font-weight: 600;
  color: white;
  line-height: 1;
}

.step-detail-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.step-detail-desc {
  padding-left: 30px;
  line-height: 1.6;
}

.step-detail-tip {
  margin-left: 30px;
}

/* 徽章：当前 */
.badge-current {
  background: hsl(var(--primary));
  box-shadow: 0 0 0 3px hsl(var(--primary) / 0.12);
}

/* 徽章：已完成 */
.badge-past {
  background: hsl(142 71% 45%);
}

/* 徽章：未开始 */
.badge-default {
  background: hsl(var(--muted-foreground) / 0.25);
}

/* 展开卡片：当前 */
.step-card-current {
  background: hsl(var(--primary) / 0.06);
  border-color: hsl(var(--primary) / 0.3);
}

.step-card-current:hover {
  border-color: hsl(var(--primary) / 0.5);
  box-shadow: 0 4px 12px hsl(var(--primary) / 0.12);
}

/* 展开卡片：已完成 */
.step-card-past {
  background: hsl(142 71% 45% / 0.06);
  border-color: hsl(142 71% 45% / 0.3);
}

.step-card-past:hover {
  border-color: hsl(142 71% 45% / 0.5);
  box-shadow: 0 4px 12px hsl(142 71% 45% / 0.12);
}

/* ========== 响应式 ========== */
@media (max-width: 1200px) {
  .step-bar-item {
    padding: 12px 6px;
  }
  .step-bar-title {
    font-size: 12px;
  }
}

@media (max-width: 992px) {
  .step-bar {
    flex-wrap: wrap;
  }
  .step-bar-item {
    flex: 0 0 calc(25% - 0px);
    border-bottom: 1px solid hsl(var(--border));
  }
  .step-bar-item:nth-last-child(-n+3) {
    border-bottom: none;
  }

  /* 响应式下隐藏箭头，避免换行时混乱 */
  .step-bar-separator {
    display: none;
  }
}

@media (max-width: 768px) {
  .step-bar-item {
    flex: 0 0 calc(33.333% - 0px);
    border-bottom: 1px solid hsl(var(--border));
  }
  .step-bar-item:nth-last-child(-n+3) {
    border-bottom: none;
  }

  .sales-process-header {
    flex-wrap: wrap;
  }
  .process-tip-wrapper {
    order: 3;
    width: 100%;
    flex: 0 0 100%;
  }

  /* 展开态：2 列 */
  .step-detail-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .step-detail-card {
    min-height: 96px;
  }
}

@media (max-width: 576px) {
  .step-bar-item {
    flex: 0 0 calc(50% - 0px);
    border-bottom: 1px solid hsl(var(--border));
  }
  .step-bar-item:nth-last-child(-n+2) {
    border-bottom: none;
  }

  /* 展开态：1 列 */
  .step-detail-grid {
    grid-template-columns: 1fr;
    gap: 10px;
  }

  .step-detail-card {
    min-height: auto;
    padding: 10px !important;
  }
}
</style>
