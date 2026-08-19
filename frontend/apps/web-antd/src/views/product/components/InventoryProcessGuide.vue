<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';

import {
  LucideArrowRight,
  LucideArrowRightLeft,
  LucideCalendar,
  LucideCheckCircle,
  LucideChevronDown,
  LucideChevronUp,
  LucideInfo,
  LucidePackage,
  LucideTarget,
  LucideTrendingUp,
  LucideXCircle,
} from '@vben/icons';

import { Button } from 'ant-design-vue';

type StepKey =
  | 'alert'
  | 'check'
  | 'inbound'
  | 'outbound'
  | 'report'
  | 'stock'
  | 'transfer';

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
    key: 'inbound',
    title: '入库',
    desc: '采购入库、退货入库、初始入库等，审核通过后自动增加库存',
    nextTip: '入库完成后，可在库存管理中查看库存',
    route: '/inbound',
    icon: LucidePackage,
  },
  {
    key: 'stock',
    title: '库存',
    desc: '查看各仓库库存数量、可用量、冻结量，支持库存冻结/解冻',
    nextTip: '库存充足时，可创建出库单发货',
    route: '/warehouse/inventory',
    icon: LucidePackage,
  },
  {
    key: 'outbound',
    title: '出库',
    desc: '销售出库、领料出库、报废出库等，审核时检查库存充足性',
    nextTip: '出库后可定期进行库存盘点',
    route: '/outbound',
    icon: LucidePackage,
  },
  {
    key: 'check',
    title: '盘点',
    desc: '定期全盘、抽盘、动态盘点，差异自动生成盘盈入库/盘亏出库单',
    nextTip: '盘点差异处理完成后，可进行跨仓库调拨',
    route: '/inventory-check',
    icon: LucideCheckCircle,
  },
  {
    key: 'transfer',
    title: '调拨',
    desc: '跨仓库调拨，源仓库出库 → 在途 → 目标仓库入库',
    nextTip: '调拨完成后，关注库存预警与报表',
    route: '/transfer',
    icon: LucideArrowRightLeft,
  },
  {
    key: 'alert',
    title: '预警',
    desc: '低库存、高库存、呆滞库存预警，支持自动通知和采购建议',
    nextTip: '根据预警情况，查看库存报表分析',
    route: '/inventory-alert',
    icon: LucideXCircle,
  },
  {
    key: 'report',
    title: '报表',
    desc: '收发存报表、库存周转率、呆滞清单、成本报表',
    route: '/inventory-report',
    icon: LucideTrendingUp,
  },
];

// 盘点类型标签
const checkTags = [
  { key: 'full', label: '定期全盘', icon: LucideCalendar },
  { key: 'spot', label: '抽盘', icon: LucideTarget },
  { key: 'dynamic', label: '动态盘点', icon: LucideTarget },
];

const currentStepIndex = computed(() => {
  return steps.findIndex((s) => s.key === props.currentStep);
});

const tipText = computed(() => {
  const idx = currentStepIndex.value;
  if (idx < 0) return '';
  if (idx >= steps.length - 1) return '仓储流程已完成';
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

<template>
  <div class="inventory-process-wrapper">
    <!-- 头部：标题 + 提示 + 操作按钮 -->
    <div class="inventory-process-header">
      <div class="flex items-center gap-2 flex-shrink-0">
        <LucideInfo
          class="process-icon-info"
          style="color: hsl(var(--primary))"
        />
        <span class="font-medium text-sm" style="color: hsl(var(--foreground))"
          >仓储流程</span
        >
      </div>
      <div class="flex items-center gap-2 flex-1 min-w-0 process-tip-wrapper">
        <span
          v-if="tipText"
          class="text-xs process-tip-text"
          style="color: hsl(var(--muted-foreground))"
          >{{ tipText }}</span
        >
      </div>
      <Button
        type="link"
        size="small"
        @click="expanded = !expanded"
        class="!p-0 !h-auto flex-shrink-0"
      >
        {{ expanded ? '收起' : '详细说明' }}
        <component
          :is="expanded ? LucideChevronUp : LucideChevronDown"
          class="inline-icon"
        />
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
    <div v-show="expanded" class="inventory-process-detail">
      <div class="step-detail-grid">
        <div
          v-for="(step, idx) in steps"
          :key="step.key"
          class="step-detail-cell"
        >
          <div
            class="step-detail-card p-3 rounded-lg"
            :class="getStepCardClass(step.key)"
          >
            <div class="flex items-center gap-2 mb-2">
              <div
                class="step-detail-index"
                :class="getStepBadgeClass(step.key)"
              >
                <span class="step-detail-index-num">{{ idx + 1 }}</span>
              </div>
              <div class="flex items-center gap-1.5 flex-1 min-w-0">
                <component
                  :is="step.icon"
                  class="step-detail-icon"
                  :style="getStepIconColor(step.key)"
                />
                <span
                  class="font-medium text-sm truncate"
                  :style="getStepTitleColor(step.key)"
                  >{{ step.title }}</span
                >
              </div>
            </div>
            <div
              class="text-xs leading-relaxed step-detail-desc"
              style="color: hsl(var(--muted-foreground))"
            >
              {{ step.desc }}
            </div>
            <!-- 盘点类型特殊说明 -->
            <div
              v-if="step.key === 'check' && isCurrentStep(step.key)"
              class="mt-2 text-xs step-detail-tags"
            >
              <span class="step-tag" v-for="tag in checkTags" :key="tag.key">
                <component :is="tag.icon" class="step-tag-icon" />
                {{ tag.label }}
              </span>
            </div>
            <div
              v-if="step.nextTip && isCurrentStep(step.key)"
              class="mt-2 text-xs flex items-center gap-1 px-2 py-1 rounded inline-flex step-detail-tip"
              style="
                color: hsl(var(--primary));
                background: hsl(var(--primary) / 8%);
              "
            >
              <LucideArrowRight class="tip-arrow-icon" />
              <span>{{ step.nextTip }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ========== 外层统一卡片 ========== */
.inventory-process-wrapper {
  padding: 10px 14px;
  margin-bottom: 12px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  box-shadow: 0 1px 2px hsl(0deg 0% 0% / 3%);
}

.inventory-process-header {
  display: flex;
  gap: 12px;
  align-items: center;
  min-height: 32px;
  padding-bottom: 8px;
  margin-bottom: 10px;
  border-bottom: 1px dashed hsl(var(--border));
}

.process-icon-info {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
}

.inline-icon {
  display: inline-block;
  width: 14px;
  height: 14px;
  vertical-align: middle;
}

.tip-arrow-icon {
  flex-shrink: 0;
  width: 12px;
  height: 12px;
}

.process-tip-wrapper {
  display: flex;
  align-items: center;
  overflow: hidden;
}

.process-tip-text {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ========== 折叠态：横向步骤条 ========== */
.step-bar {
  display: flex;
  align-items: stretch;
  width: 100%;
  overflow: hidden;
  border-radius: 8px;
}

.step-bar-item {
  position: relative;
  display: flex;
  flex: 1;
  gap: 6px;
  align-items: center;
  justify-content: center;
  min-width: 0;
  padding: 12px 8px;
  cursor: pointer;
  background: transparent;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.step-bar-index {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  transition: all 0.2s ease;
}

.step-bar-icon {
  width: 14px;
  height: 14px;
  color: white;
}

.step-bar-title {
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
}

/* 折叠态：当前步骤 */
.step-bar-current {
  background: hsl(var(--primary) / 8%);
}

.step-bar-current .step-bar-title {
  font-weight: 600;
  color: hsl(var(--primary));
}

.step-bar-current .step-bar-index {
  background: hsl(var(--primary));
  box-shadow: 0 0 0 3px hsl(var(--primary) / 12%);
}

.step-bar-current:hover {
  background: hsl(var(--primary) / 15%);
}

/* 折叠态：已完成步骤 */
.step-bar-past .step-bar-title {
  color: hsl(142deg 71% 45%);
}

.step-bar-past .step-bar-index {
  background: hsl(142deg 71% 45%);
}

.step-bar-past:hover {
  background: hsl(142deg 71% 45% / 8%);
}

/* 折叠态：未开始步骤 */
.step-bar-default .step-bar-title {
  color: hsl(var(--muted-foreground));
}

.step-bar-default .step-bar-index {
  background: hsl(var(--muted-foreground) / 25%);
}

.step-bar-default:hover {
  background: hsl(var(--muted) / 50%);
}

.step-bar-default:hover .step-bar-index {
  background: hsl(var(--muted-foreground) / 40%);
}

/* 步骤之间的分隔箭头 */
.step-bar-separator {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  padding: 0 4px;
  color: hsl(var(--muted-foreground) / 40%);
}

.step-bar-separator-text {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  font-size: 16px;
  font-weight: 600;
  line-height: 1;
}

/* ========== 展开态：详细卡片 ========== */
.inventory-process-detail {
  padding-top: 4px;
  animation: fade-slide-down 0.25s ease-out;
}

@keyframes fade-slide-down {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Grid 布局：PC 默认 4 列，平板 2 列，手机 1 列 */
.step-detail-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
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
  min-height: 110px;
  margin: 0;
  cursor: pointer;
  background: hsl(var(--muted) / 50%);
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  transition: all 0.2s ease;
}

.step-detail-card:hover {
  border-color: hsl(var(--muted-foreground) / 30%);
  box-shadow: 0 2px 8px hsl(0deg 0% 0% / 6%);
}

.step-detail-index {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  transition: all 0.2s ease;
}

.step-detail-index-num {
  font-size: 12px;
  font-weight: 600;
  line-height: 1;
  color: white;
}

.step-detail-icon {
  flex-shrink: 0;
  width: 16px;
  height: 16px;
}

.step-detail-desc {
  padding-left: 30px;
  line-height: 1.6;
}

.step-detail-tip {
  margin-left: 30px;
}

/* 盘点类型标签 */
.step-detail-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding-left: 30px;
}

.step-tag {
  display: inline-flex;
  gap: 3px;
  align-items: center;
  padding: 2px 8px;
  font-size: 11px;
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 6%);
  border: 1px solid hsl(var(--primary) / 15%);
  border-radius: 4px;
}

.step-tag-icon {
  flex-shrink: 0;
  width: 11px;
  height: 11px;
}

/* 徽章：当前 */
.badge-current {
  background: hsl(var(--primary));
  box-shadow: 0 0 0 3px hsl(var(--primary) / 12%);
}

/* 徽章：已完成 */
.badge-past {
  background: hsl(142deg 71% 45%);
}

/* 徽章：未开始 */
.badge-default {
  background: hsl(var(--muted-foreground) / 25%);
}

/* 展开卡片：当前 */
.step-card-current {
  background: hsl(var(--primary) / 6%);
  border-color: hsl(var(--primary) / 30%);
}

.step-card-current:hover {
  border-color: hsl(var(--primary) / 50%);
  box-shadow: 0 4px 12px hsl(var(--primary) / 12%);
}

/* 展开卡片：已完成 */
.step-card-past {
  background: hsl(142deg 71% 45% / 6%);
  border-color: hsl(142deg 71% 45% / 30%);
}

.step-card-past:hover {
  border-color: hsl(142deg 71% 45% / 50%);
  box-shadow: 0 4px 12px hsl(142deg 71% 45% / 12%);
}

/* ========== 响应式 ========== */
@media (max-width: 1200px) {
  .step-bar-item {
    padding: 12px 6px;
  }

  .step-bar-title {
    font-size: 12px;
  }

  .step-detail-grid {
    grid-template-columns: repeat(3, 1fr);
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

  .step-bar-item:nth-last-child(-n + 3) {
    border-bottom: none;
  }

  /* 响应式下隐藏箭头，避免换行时混乱 */
  .step-bar-separator {
    display: none;
  }

  .step-detail-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }
}

@media (max-width: 768px) {
  .step-bar-item {
    flex: 0 0 calc(33.333% - 0px);
    border-bottom: 1px solid hsl(var(--border));
  }

  .step-bar-item:nth-last-child(-n + 3) {
    border-bottom: none;
  }

  .inventory-process-header {
    flex-wrap: wrap;
  }

  .process-tip-wrapper {
    flex: 0 0 100%;
    order: 3;
    width: 100%;
  }

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
