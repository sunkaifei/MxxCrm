<script lang="ts" setup>
import { computed } from 'vue';

import { $t } from '#/locales';

/**
 * 提成方案预览组件
 * 根据当前表单配置实时渲染组织架构图 + 资金流向预览
 * 支持 6 种提成模式（commissionCategory 1-6）
 */
const props = defineProps<{
  /** 当前表单数据 */
  form: any;
  /** 阶梯配置（用于取比例） */
  tiers?: any[];
}>();

// 模拟演示数据，仅用于预览效果
const DEMO_TEAM_PAYMENT = 1_000_000; // 团队回款 100万
const DEMO_PERSONAL_PAYMENT = 100_000; // 个人回款 10万
const DEMO_COST = 60_000; // 成本 6万
const DEMO_POOL_BALANCE = 5_000; // 资金池余额

// 取第一档阶梯比例作为演示比例
const demoRate = computed(() => {
  const tier = props.tiers?.[0];
  if (tier && Number(tier.commissionRate) > 0) {
    return Number(tier.commissionRate);
  }
  return 0.05; // 默认 5%
});

const category = computed(() => Number(props.form?.commissionCategory ?? 1));
const beneficiaryRole = computed(() =>
  Number(props.form?.beneficiaryRole ?? 1),
);

// 模式标题与说明
const modeInfo = computed(() => {
  const map: Record<number, { title: string; desc: string }> = {
    1: {
      title: $t('page.finance.commissionRule.category.personal'),
      desc: $t('page.finance.commissionRule.preview.personalDesc'),
    },
    2: {
      title: $t('page.finance.commissionRule.category.management'),
      desc: $t('page.finance.commissionRule.preview.managementDesc'),
    },
    3: {
      title: $t('page.finance.commissionRule.category.teamBonus'),
      desc: $t('page.finance.commissionRule.preview.teamBonusDesc'),
    },
    4: {
      title: $t('page.finance.commissionRule.category.poolFund'),
      desc: $t('page.finance.commissionRule.preview.poolFundDesc'),
    },
    5: {
      title: $t('page.finance.commissionRule.category.reallocation'),
      desc: $t('page.finance.commissionRule.preview.reallocationDesc'),
    },
    6: {
      title: $t('page.finance.commissionRule.category.profit'),
      desc: $t('page.finance.commissionRule.preview.profitDesc'),
    },
  };
  return map[category.value] || map[1]!;
});

// 受益岗位名称
function roleName(role: number): string {
  const map: Record<number, string> = {
    1: $t('page.finance.commissionRule.beneficiary.sales'),
    2: $t('page.finance.commissionRule.beneficiary.supervisor'),
    3: $t('page.finance.commissionRule.beneficiary.manager'),
    4: $t('page.finance.commissionRule.beneficiary.director'),
    5: $t('page.finance.commissionRule.beneficiary.gm'),
  };
  return map[role] || map[1]!;
}

// 层级系数（模式 B）
function levelRate(level: number): number {
  // 优先用表单配置的第一档比例作为主管级
  const base = demoRate.value;
  if (level === 1) return base;
  if (level === 2) return base * (5 / 3); // 经理比例更高
  if (level === 3) return base * (8 / 3); // 总监比例最高
  return base;
}

function money(val: number): string {
  return `¥${Math.round(val).toLocaleString('en-US')}`;
}

function rateText(val: number): string {
  return `${(val * 100).toFixed(1)}%`;
}

// ===== 各模式预览计算 =====

// 模式 A: 个人提成
const personalAmount = computed(
  () => DEMO_PERSONAL_PAYMENT * demoRate.value,
);

// 模式 B: 多级管理分润
const managementLevels = computed(() => {
  // 根据 beneficiaryRole 决定展示层级数
  const role = beneficiaryRole.value;
  let levels = 3; // 默认展示 3 级
  if (role === 2) levels = 1; // 仅主管
  if (role === 3) levels = 2; // 主管+经理
  if (role === 5) levels = 3; // 总经理（取最顶级）

  const result: { name: string; rate: number; amount: number }[] = [];
  const names = [
    $t('page.finance.commissionRule.beneficiary.director'),
    $t('page.finance.commissionRule.beneficiary.manager'),
    $t('page.finance.commissionRule.beneficiary.supervisor'),
  ];
  for (let i = 0; i < levels; i++) {
    const lvl = levels - i; // 从高到低
    const r = levelRate(lvl);
    result.push({
      name: names[3 - lvl] || names[2]!,
      rate: r,
      amount: DEMO_TEAM_PAYMENT * r,
    });
  }
  return result;
});

// 模式 C: 团队激励奖金
const teamBonusReached = computed(() => {
  const target = Number(props.form?.bonusTarget ?? 500_000);
  return DEMO_TEAM_PAYMENT >= target;
});
const teamBonusAmount = computed(() => {
  if (!teamBonusReached.value) return 0;
  return Number(props.form?.bonusFixedAmount ?? 2_000);
});

// 模式 D: 团建资金池
const poolDeposit = computed(() => DEMO_TEAM_PAYMENT * demoRate.value);

// 模式 E: 总提成再分配
const reallocationTotal = computed(
  () => DEMO_TEAM_PAYMENT * demoRate.value,
);
const reallocationMembers = computed(() => {
  // 模拟 3 个成员按业绩比例分配
  const payments = [200_000, 150_000, 150_000];
  const total = payments.reduce((a, b) => a + b, 0);
  return payments.map((p, i) => ({
    name: `${$t('page.finance.commissionRule.preview.member')} ${String.fromCharCode(65 + i)}`,
    payment: p,
    amount: reallocationTotal.value * (p / total),
  }));
});

// 模式 F: 利润提成
const profitAmount = computed(() => {
  const profit = DEMO_PERSONAL_PAYMENT - DEMO_COST;
  return profit * demoRate.value;
});
</script>

<template>
  <div class="commission-scheme-preview">
    <div class="preview-header">
      <span class="preview-badge">{{ $t('page.finance.commissionRule.preview.title') }}</span>
      <span class="preview-mode-tag">{{ modeInfo.title }}</span>
    </div>

    <div class="preview-desc">{{ modeInfo.desc }}</div>

    <div class="preview-canvas">
      <!-- 模式 A: 个人提成 -->
      <div v-if="category === 1" class="scheme-tree single-node">
        <div class="tree-node node-sales">
          <div class="node-title">{{ roleName(1) }}</div>
          <div class="node-body">
            <div class="node-line">
              <span class="label">{{ $t('page.finance.commissionRule.preview.payment') }}</span>
              <span class="value">{{ money(DEMO_PERSONAL_PAYMENT) }}</span>
            </div>
            <div class="node-line">
              <span class="label">{{ $t('page.finance.commissionRule.preview.rate') }}</span>
              <span class="value accent">{{ rateText(demoRate) }}</span>
            </div>
            <div class="node-line highlight">
              <span class="label">{{ $t('page.finance.commissionRule.preview.commission') }}</span>
              <span class="value strong">{{ money(personalAmount) }}</span>
            </div>
          </div>
          <div class="node-tag tag-green">{{ $t('page.finance.commissionRule.preview.toSalary') }}</div>
        </div>
      </div>

      <!-- 模式 B: 多级管理分润 -->
      <div v-else-if="category === 2" class="scheme-tree tree-vertical">
        <div
          v-for="(lvl, idx) in managementLevels"
          :key="idx"
          class="tree-level"
          :class="{ 'has-children': idx < managementLevels.length - 1 }"
        >
          <div class="tree-node node-manager">
            <div class="node-title">{{ lvl.name }}</div>
            <div class="node-body">
              <div class="node-line">
                <span class="label">{{ $t('page.finance.commissionRule.preview.teamPayment') }}</span>
                <span class="value">{{ money(DEMO_TEAM_PAYMENT) }}</span>
              </div>
              <div class="node-line">
                <span class="label">{{ $t('page.finance.commissionRule.preview.rate') }}</span>
                <span class="value accent">{{ rateText(lvl.rate) }}</span>
              </div>
              <div class="node-line highlight">
                <span class="label">{{ $t('page.finance.commissionRule.preview.share') }}</span>
                <span class="value strong">{{ money(lvl.amount) }}</span>
              </div>
            </div>
            <div class="node-tag tag-blue">{{ $t('page.finance.commissionRule.preview.toSalary') }}</div>
          </div>
        </div>
        <!-- 底层销售员 -->
        <div class="tree-level leaf-level">
          <div class="leaf-row">
            <div class="tree-node node-sales small">
              <div class="node-title">{{ $t('page.finance.commissionRule.preview.sales') }}</div>
              <div class="node-tag tag-green">{{ $t('page.finance.commissionRule.preview.personalCommission') }}</div>
            </div>
            <div class="tree-node node-sales small">
              <div class="node-title">{{ $t('page.finance.commissionRule.preview.sales') }}</div>
              <div class="node-tag tag-green">{{ $t('page.finance.commissionRule.preview.personalCommission') }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 模式 C: 团队激励奖金 -->
      <div v-else-if="category === 3" class="scheme-tree tree-vertical">
        <div class="tree-level has-children">
          <div class="tree-node node-manager">
            <div class="node-title">{{ roleName(beneficiaryRole) }}</div>
            <div class="node-body">
              <div class="node-line">
                <span class="label">{{ $t('page.finance.commissionRule.preview.teamPayment') }}</span>
                <span class="value">{{ money(DEMO_TEAM_PAYMENT) }}</span>
              </div>
              <div class="node-line">
                <span class="label">{{ $t('page.finance.commissionRule.preview.target') }}</span>
                <span class="value">{{ money(Number(form?.bonusTarget ?? 500000)) }}</span>
              </div>
              <div class="node-line highlight" :class="{ 'not-reached': !teamBonusReached }">
                <span class="label">{{ $t('page.finance.commissionRule.preview.bonus') }}</span>
                <span class="value strong">
                  {{ teamBonusReached ? money(teamBonusAmount) : $t('page.finance.commissionRule.preview.notReached') }}
                </span>
              </div>
            </div>
            <div class="node-tag tag-blue">{{ $t('page.finance.commissionRule.preview.toSalary') }}</div>
          </div>
        </div>
        <div class="tree-level leaf-level">
          <div class="leaf-row">
            <div class="tree-node node-sales small">
              <div class="node-title">{{ $t('page.finance.commissionRule.preview.member') }}</div>
              <div class="node-tag tag-green">{{ $t('page.finance.commissionRule.preview.notAffected') }}</div>
            </div>
            <div class="tree-node node-sales small">
              <div class="node-title">{{ $t('page.finance.commissionRule.preview.member') }}</div>
              <div class="node-tag tag-green">{{ $t('page.finance.commissionRule.preview.notAffected') }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 模式 D: 团建资金池 -->
      <div v-else-if="category === 4" class="scheme-tree tree-horizontal">
        <div class="tree-node node-payment">
          <div class="node-title">{{ $t('page.finance.commissionRule.preview.teamPayment') }}</div>
          <div class="node-body">
            <div class="node-line">
              <span class="value">{{ money(DEMO_TEAM_PAYMENT) }}</span>
            </div>
            <div class="node-line">
              <span class="label">{{ $t('page.finance.commissionRule.preview.rate') }}</span>
              <span class="value accent">{{ rateText(demoRate) }}</span>
            </div>
          </div>
        </div>
        <div class="flow-arrow">
          <span class="flow-amount">{{ money(poolDeposit) }}</span>
        </div>
        <div class="tree-node node-pool">
          <div class="node-title">{{ $t('page.finance.commissionRule.preview.pool') }}</div>
          <div class="node-body">
            <div class="node-line">
              <span class="label">{{ $t('page.finance.commissionRule.preview.deposit') }}</span>
              <span class="value accent">+{{ money(poolDeposit) }}</span>
            </div>
            <div class="node-line">
              <span class="label">{{ $t('page.finance.commissionRule.preview.poolBalance') }}</span>
              <span class="value strong">{{ money(DEMO_POOL_BALANCE + poolDeposit) }}</span>
            </div>
          </div>
          <div class="node-tag tag-orange">{{ $t('page.finance.commissionRule.preview.notToSalary') }}</div>
        </div>
        <div class="flow-arrow down">
          <span class="flow-amount">{{ $t('page.finance.commissionRule.preview.expense') }}</span>
        </div>
        <div class="tree-node node-expense">
          <div class="node-title">{{ $t('page.finance.commissionRule.preview.teamBuilding') }}</div>
          <div class="node-tag tag-orange">{{ $t('page.finance.commissionRule.preview.dinnerActivity') }}</div>
        </div>
      </div>

      <!-- 模式 E: 总提成再分配 -->
      <div v-else-if="category === 5" class="scheme-tree tree-vertical">
        <div class="tree-level has-children wide">
          <div class="tree-node node-pending">
            <div class="node-title">{{ roleName(beneficiaryRole) }}</div>
            <div class="node-body">
              <div class="node-line">
                <span class="label">{{ $t('page.finance.commissionRule.preview.teamPayment') }}</span>
                <span class="value">{{ money(DEMO_TEAM_PAYMENT) }}</span>
              </div>
              <div class="node-line">
                <span class="label">{{ $t('page.finance.commissionRule.preview.rate') }}</span>
                <span class="value accent">{{ rateText(demoRate) }}</span>
              </div>
              <div class="node-line highlight">
                <span class="label">{{ $t('page.finance.commissionRule.preview.totalCollected') }}</span>
                <span class="value strong">{{ money(reallocationTotal) }}</span>
              </div>
            </div>
            <div class="node-tag tag-purple">{{ $t('page.finance.commissionRule.preview.pending') }}</div>
          </div>
        </div>
        <div class="tree-level leaf-level">
          <div class="leaf-row">
            <div
              v-for="(m, i) in reallocationMembers"
              :key="i"
              class="tree-node node-sales small"
            >
              <div class="node-title">{{ m.name }}</div>
              <div class="node-body">
                <div class="node-line">
                  <span class="value strong">{{ money(m.amount) }}</span>
                </div>
              </div>
              <div class="node-tag tag-green">{{ $t('page.finance.commissionRule.preview.byPerformance') }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 模式 F: 利润提成 -->
      <div v-else-if="category === 6" class="scheme-tree single-node">
        <div class="tree-node node-sales">
          <div class="node-title">{{ roleName(1) }}</div>
          <div class="node-body">
            <div class="node-line">
              <span class="label">{{ $t('page.finance.commissionRule.preview.payment') }}</span>
              <span class="value">{{ money(DEMO_PERSONAL_PAYMENT) }}</span>
            </div>
            <div class="node-line">
              <span class="label">{{ $t('page.finance.commissionRule.preview.cost') }}</span>
              <span class="value">-{{ money(DEMO_COST) }}</span>
            </div>
            <div class="node-line">
              <span class="label">{{ $t('page.finance.commissionRule.preview.profit') }}</span>
              <span class="value accent">{{ money(DEMO_PERSONAL_PAYMENT - DEMO_COST) }}</span>
            </div>
            <div class="node-line">
              <span class="label">{{ $t('page.finance.commissionRule.preview.rate') }}</span>
              <span class="value accent">{{ rateText(demoRate) }}</span>
            </div>
            <div class="node-line highlight">
              <span class="label">{{ $t('page.finance.commissionRule.preview.commission') }}</span>
              <span class="value strong">{{ money(profitAmount) }}</span>
            </div>
          </div>
          <div class="node-tag tag-green">{{ $t('page.finance.commissionRule.preview.toSalary') }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.commission-scheme-preview {
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 16px;
}

.preview-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.preview-badge {
  font-size: 13px;
  font-weight: 600;
  color: #475569;
}

.preview-mode-tag {
  display: inline-block;
  padding: 2px 10px;
  background: #dbeafe;
  color: #1d4ed8;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
}

.preview-desc {
  font-size: 12px;
  color: #64748b;
  margin-bottom: 16px;
  line-height: 1.5;
}

.preview-canvas {
  display: flex;
  justify-content: center;
  align-items: flex-start;
  min-height: 200px;
  padding: 12px 0;
}

/* ===== 树状结构通用 ===== */
.scheme-tree {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}

.tree-vertical {
  gap: 0;
}

.tree-level {
  position: relative;
  display: flex;
  justify-content: center;
  padding: 8px 0;
}

.tree-level.has-children::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 50%;
  width: 2px;
  height: 20px;
  background: #cbd5e1;
  transform: translateX(-50%);
}

.leaf-level {
  padding-top: 28px;
  position: relative;
}

.leaf-level::before {
  content: '';
  position: absolute;
  top: 0;
  left: 50%;
  width: 2px;
  height: 20px;
  background: #cbd5e1;
  transform: translateX(-50%);
}

.leaf-row {
  display: flex;
  gap: 24px;
  justify-content: center;
  position: relative;
}

.leaf-row::before {
  content: '';
  position: absolute;
  top: -20px;
  left: 20%;
  right: 20%;
  height: 2px;
  background: #cbd5e1;
}

/* 水平流向（模式 D） */
.tree-horizontal {
  flex-direction: row;
  align-items: center;
  gap: 0;
  flex-wrap: wrap;
  justify-content: center;
}

.flow-arrow {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  height: 40px;
  position: relative;
  color: #64748b;
}

.flow-arrow::before {
  content: '';
  position: absolute;
  left: 0;
  right: 12px;
  top: 50%;
  height: 2px;
  background: #cbd5e1;
}

.flow-arrow::after {
  content: '';
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  border: 6px solid transparent;
  border-left-color: #cbd5e1;
}

.flow-arrow.down {
  width: 40px;
  height: 60px;
  flex-direction: column;
}

.flow-arrow.down::before {
  left: 50%;
  right: auto;
  top: 0;
  bottom: 12px;
  width: 2px;
  height: auto;
  transform: translateX(-50%);
}

.flow-arrow.down::after {
  top: auto;
  bottom: 0;
  left: 50%;
  right: auto;
  transform: translateX(-50%);
  border: 6px solid transparent;
  border-top-color: #cbd5e1;
  border-left-color: transparent;
}

.flow-amount {
  font-size: 12px;
  font-weight: 600;
  color: #1d4ed8;
  background: #fff;
  padding: 2px 6px;
  border-radius: 4px;
  z-index: 1;
}

/* ===== 节点卡片 ===== */
.tree-node {
  background: #fff;
  border: 2px solid #e2e8f0;
  border-radius: 8px;
  padding: 12px 16px;
  min-width: 180px;
  box-shadow: 0 1px 3px rgb(0 0 0 / 8%);
  transition: all 0.2s;
}

.tree-node:hover {
  box-shadow: 0 4px 12px rgb(0 0 0 / 12%);
}

.tree-node.small {
  min-width: 130px;
  padding: 8px 12px;
}

.single-node .tree-node {
  min-width: 240px;
}

.node-title {
  font-size: 14px;
  font-weight: 600;
  color: #1e293b;
  margin-bottom: 8px;
  text-align: center;
}

.node-body {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.node-line {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  gap: 12px;
}

.node-line .label {
  color: #64748b;
}

.node-line .value {
  color: #334155;
  font-weight: 500;
}

.node-line .value.accent {
  color: #1d4ed8;
}

.node-line .value.strong {
  color: #dc2626;
  font-weight: 700;
  font-size: 13px;
}

.node-line.highlight {
  background: #fef3c7;
  padding: 4px 8px;
  border-radius: 4px;
  margin-top: 4px;
}

.node-line.highlight .value.strong {
  color: #b45309;
}

.node-line.not-reached .value.strong {
  color: #94a3b8;
}

.node-tag {
  display: inline-block;
  margin-top: 8px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  width: 100%;
  text-align: center;
}

.tag-green {
  background: #dcfce7;
  color: #166534;
}

.tag-blue {
  background: #dbeafe;
  color: #1e40af;
}

.tag-orange {
  background: #fed7aa;
  color: #9a3412;
}

.tag-purple {
  background: #f3e8ff;
  color: #6b21a8;
}

/* 节点颜色变体 */
.node-sales {
  border-color: #86efac;
}

.node-sales .node-title {
  color: #166534;
}

.node-manager {
  border-color: #93c5fd;
}

.node-manager .node-title {
  color: #1e40af;
}

.node-pool {
  border-color: #fdba74;
}

.node-pool .node-title {
  color: #9a3412;
}

.node-pending {
  border-color: #d8b4fe;
}

.node-pending .node-title {
  color: #6b21a8;
}

.node-payment {
  border-color: #cbd5e1;
}

.node-expense {
  border-color: #fca5a5;
}

.node-expense .node-title {
  color: #991b1b;
}
</style>
