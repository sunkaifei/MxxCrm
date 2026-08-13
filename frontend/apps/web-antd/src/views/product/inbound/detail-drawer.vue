<script lang="ts" setup>
/**
 * 入库单详情抽屉（只读，专业 ERP 风格）
 *
 * 用法：
 * <InboundDetailDrawer v-model:visible="visible" :inbound-id="inboundId" />
 */
import { computed, ref, watch } from 'vue';

import { Drawer, Spin, Empty, Tag, Divider, Table, Tooltip, Button } from 'ant-design-vue';

import { getInboundInfoApi } from '#/api/core/product/inbound';
import { $t } from '#/locales';

const props = defineProps<{
  visible: boolean;
  inboundId?: number | null;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
}>();

const loading = ref(false);
const detail = ref<any>(null);
const isFullscreen = ref(false);

const drawerWidth = computed(() => (isFullscreen.value ? '100%' : '75%'));

// ===== 类型 & 状态映射 =====
const typeMap: Record<string, { label: string; color: string; icon: string }> = {
  purchase: { label: $t('page.product.inbound.type.purchase'), color: 'blue', icon: '🚚' },
  return: { label: $t('page.product.inbound.type.return'), color: 'orange', icon: '↩️' },
  surplus: { label: $t('page.product.inbound.type.surplus'), color: 'green', icon: '📈' },
  initial: { label: $t('page.product.inbound.type.initial'), color: 'cyan', icon: '🏁' },
  other: { label: $t('page.product.inbound.type.other'), color: 'default', icon: '📦' },
};

const statusMap: Record<number, { label: string; color: string; step: number }> = {
  0: { label: $t('page.product.inbound.status.0'), color: 'default', step: 0 },
  1: { label: $t('page.product.inbound.status.1'), color: 'processing', step: 1 },
  2: { label: $t('page.product.inbound.status.2'), color: 'warning', step: 2 },
  3: { label: $t('page.product.inbound.status.3'), color: 'success', step: 3 },
  4: { label: $t('page.product.inbound.status.4'), color: 'error', step: 1 },
};

function getType(val?: string) {
  return typeMap[val || ''] || { label: val || '-', color: 'default', icon: '📦' };
}

function getStatus(val?: number) {
  return statusMap[val ?? -1] || { label: '-', color: 'default', step: 0 };
}

// 审核进度条步骤
const auditSteps = computed(() => {
  const s = getStatus(detail.value?.status).step;
  return [
    { key: 'draft', label: $t('page.product.inbound.status.0'), done: s >= 0, active: s === 0 },
    { key: 'pending', label: $t('page.product.inbound.status.1'), done: s >= 1, active: s === 1 },
    { key: 'approved', label: $t('page.product.inbound.status.2'), done: s >= 2, active: s === 2 },
    { key: 'done', label: $t('page.product.inbound.status.3'), done: s >= 3, active: s === 3 },
  ];
});

// ===== 明细表格列 =====
const itemColumns = computed(() => [
  { title: '#', key: 'seq', width: 45, customRender: ({ index }: any) => index + 1 },
  { title: $t('page.product.inbound.field.itemProductCode'), dataIndex: 'productCode', width: 120, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.inbound.field.itemProductName'), dataIndex: 'productName', ellipsis: true, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.inbound.field.spec') || '规格', dataIndex: 'spec', width: 100, customRender: ({ value }: any) => value || '-' },
  { title: $t('page.product.inbound.field.unit') || '单位', dataIndex: 'unit', width: 70, customRender: ({ value }: any) => value || '-' },
  {
    title: $t('page.product.inbound.field.itemQuantity'),
    dataIndex: 'quantity',
    width: 90,
    customRender: ({ value }: any) => value ?? '-',
  },
  {
    title: $t('page.product.inbound.field.itemUnitPrice'),
    dataIndex: 'unitPrice',
    width: 100,
    customRender: ({ value }: any) => (value != null ? `¥${Number(value).toFixed(2)}` : '-'),
  },
  {
    title: $t('page.product.inbound.field.totalPrice') || '金额',
    dataIndex: 'totalPrice',
    width: 110,
    customRender: ({ value }: any) => (value != null ? `¥${Number(value).toFixed(2)}` : '-'),
  },
  { title: $t('page.product.inbound.field.remark'), dataIndex: 'remark', width: 120, ellipsis: true, customRender: ({ value }: any) => value || '-' },
]);

// ===== 数据加载 =====
async function loadDetail(id: number) {
  loading.value = true;
  try {
    const res: any = await getInboundInfoApi(id);
    const raw = res?.data ?? res;
    // 后端返回 { detail: {...}, items: [...] }，扁平化到顶层
    if (raw?.detail) {
      detail.value = { ...raw.detail, items: raw.items ?? [] };
    } else {
      detail.value = raw;
    }
  } catch {
    detail.value = null;
  } finally {
    loading.value = false;
  }
}

watch(
  () => [props.visible, props.inboundId] as const,
  ([v, id]) => {
    if (v && id) {
      loadDetail(id);
    }
    if (!v) {
      detail.value = null;
      isFullscreen.value = false;
    }
  },
);
</script>

<template>
  <Drawer
    :open="visible"
    :width="drawerWidth"
    placement="right"
    :title="$t('page.product.inbound.detail')"
    @close="emit('update:visible', false)"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? $t('page.product.inbound.drawer.restore') : $t('page.product.inbound.drawer.fullscreen')">
        <Button type="text" size="small" @click="isFullscreen = !isFullscreen">
          <svg v-if="!isFullscreen" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </Button>
      </Tooltip>
    </template>

    <Spin :spinning="loading">
      <div v-if="detail" class="inbound-detail">
        <!-- ===== 头部卡片：单号 + 状态 + 类型 ===== -->
        <div class="detail-header">
          <div class="header-left">
            <div class="header-icon" :style="{ background: `hsl(var(--primary) / 0.1)` }">
              <span class="text-xl">{{ getType(detail.inboundType).icon }}</span>
            </div>
            <div class="header-info">
              <div class="header-title-row">
                <h2 class="header-title">{{ detail.inboundNo || '-' }}</h2>
                <Tag :color="getStatus(detail.status).color" class="header-status-tag">
                  {{ getStatus(detail.status).label }}
                </Tag>
              </div>
              <div class="header-meta">
                <Tag :color="getType(detail.inboundType).color" class="header-type-tag">
                  {{ getType(detail.inboundType).label }}
                </Tag>
                <span class="header-meta-item">{{ $t('page.product.inbound.field.warehouse') }}：{{ detail.warehouseName || '-' }}</span>
                <span class="header-meta-item">{{ $t('page.product.inbound.field.createTime') }}：{{ detail.createTime || '-' }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- ===== 审核进度条 ===== -->
        <div class="audit-progress">
          <div
            v-for="(step, i) in auditSteps"
            :key="step.key"
            class="progress-step"
            :class="{
              'step-done': step.done && !step.active,
              'step-active': step.active,
              'step-pending': !step.done,
            }"
          >
            <div class="progress-dot">
              <svg v-if="step.done && !step.active" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12" />
              </svg>
              <span v-else-if="step.active" class="progress-pulse"></span>
              <span v-else class="progress-num">{{ i + 1 }}</span>
            </div>
            <span class="progress-label">{{ step.label }}</span>
            <div v-if="i < auditSteps.length - 1" class="progress-bar" :class="{ 'bar-filled': step.done }"></div>
          </div>
        </div>

        <!-- ===== 汇总数据卡片 ===== -->
        <div class="summary-cards">
          <div class="summary-card">
            <div class="summary-label">{{ $t('page.product.inbound.field.totalQuantity') }}</div>
            <div class="summary-value">{{ detail.totalQuantity ?? '-' }}</div>
          </div>
          <div class="summary-card summary-card--primary">
            <div class="summary-label">{{ $t('page.product.inbound.field.totalAmount') }}</div>
            <div class="summary-value">¥{{ Number(detail.totalAmount ?? 0).toFixed(2) }}</div>
          </div>
          <div class="summary-card">
            <div class="summary-label">{{ $t('page.product.inbound.field.items') }}</div>
            <div class="summary-value">{{ detail.items?.length ?? 0 }}</div>
          </div>
        </div>

        <Divider style="margin: 16px 0 12px" />

        <!-- ===== 基本信息 ===== -->
        <div class="section-title">{{ $t('page.product.inbound.drawer.basicInfo') }}</div>
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">{{ $t('page.product.inbound.field.inboundNo') }}</span>
            <span class="info-value">{{ detail.inboundNo || '-' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">{{ $t('page.product.inbound.field.inboundType') }}</span>
            <span class="info-value">{{ getType(detail.inboundType).label }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">{{ $t('page.product.inbound.field.warehouse') }}</span>
            <span class="info-value">{{ detail.warehouseName || '-' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">{{ $t('page.product.inbound.field.status') }}</span>
            <span class="info-value">
              <Tag :color="getStatus(detail.status).color">{{ getStatus(detail.status).label }}</Tag>
            </span>
          </div>
          <div v-if="detail.sourceOrderNo" class="info-item">
            <span class="info-label">{{ $t('page.product.inbound.field.sourceOrderNo') }}</span>
            <span class="info-value">{{ detail.sourceOrderNo }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">{{ $t('page.product.inbound.field.createdBy') }}</span>
            <span class="info-value">{{ detail.createdByName || '-' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">{{ $t('page.product.inbound.field.createTime') }}</span>
            <span class="info-value">{{ detail.createTime || '-' }}</span>
          </div>
          <div v-if="detail.auditByName" class="info-item">
            <span class="info-label">{{ $t('page.product.inbound.field.auditedBy') || '审核人' }}</span>
            <span class="info-value">{{ detail.auditByName }}</span>
          </div>
          <div v-if="detail.auditTime" class="info-item">
            <span class="info-label">{{ $t('page.product.inbound.field.auditTime') || '审核时间' }}</span>
            <span class="info-value">{{ detail.auditTime }}</span>
          </div>
        </div>

        <!-- ===== 备注 ===== -->
        <div v-if="detail.remark" class="remark-box">
          <span class="info-label">{{ $t('page.product.inbound.field.remark') }}</span>
          <p class="remark-text">{{ detail.remark }}</p>
        </div>

        <!-- ===== 入库明细表 ===== -->
        <div class="section-title" style="margin-top: 20px">{{ $t('page.product.inbound.field.items') }}</div>
        <Table
          :columns="itemColumns"
          :data-source="detail.items || []"
          :pagination="false"
          size="small"
          :scroll="{ x: 800 }"
          row-key="id"
          class="items-table"
          bordered
        >
          <template #emptyText>
            <Empty :description="$t('page.product.inbound.message.noItems') || '暂无明细'" />
          </template>
        </Table>
      </div>

      <Empty v-else-if="!loading" :description="$t('page.product.inbound.message.noData') || '暂无数据'" />
    </Spin>
  </Drawer>
</template>

<style scoped>
/* ===== 整体容器 ===== */
.inbound-detail {
  padding-bottom: 24px;
}

/* ===== 头部卡片 ===== */
.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 20px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
}

.header-left {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  flex: 1;
  min-width: 0;
}

.header-icon {
  flex-shrink: 0;
  width: 52px;
  height: 52px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.header-info {
  flex: 1;
  min-width: 0;
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}

.header-title {
  font-size: 18px;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin: 0;
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-status-tag {
  flex-shrink: 0;
  font-size: 12px;
  padding: 2px 10px;
  border-radius: 4px;
}

.header-meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px 14px;
}

.header-type-tag {
  flex-shrink: 0;
}

.header-meta-item {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
}

/* ===== 审核进度条 ===== */
.audit-progress {
  display: flex;
  align-items: flex-start;
  margin-top: 20px;
  padding: 0 8px;
}

.progress-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex: 1;
  position: relative;
}

.progress-dot {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  z-index: 1;
  transition: all 0.3s ease;
}

.progress-num {
  color: hsl(var(--muted-foreground));
}

.step-pending .progress-dot {
  background: hsl(var(--muted) / 0.6);
  border: 2px solid hsl(var(--border));
  color: hsl(var(--muted-foreground) / 0.6);
}

.step-done .progress-dot {
  background: hsl(142 71% 45%);
  border: 2px solid hsl(142 71% 45%);
}

.step-active .progress-dot {
  background: hsl(var(--primary));
  border: 2px solid hsl(var(--primary));
  box-shadow: 0 0 0 4px hsl(var(--primary) / 0.12);
}

.progress-pulse {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: white;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.7); }
}

.progress-label {
  margin-top: 6px;
  font-size: 11px;
  white-space: nowrap;
}

.step-pending .progress-label {
  color: hsl(var(--muted-foreground) / 0.6);
}

.step-done .progress-label {
  color: hsl(142 71% 35%);
}

.step-active .progress-label {
  color: hsl(var(--primary));
  font-weight: 600;
}

.progress-bar {
  position: absolute;
  top: 14px;
  left: 50%;
  width: 100%;
  height: 2px;
  background: hsl(var(--border));
  z-index: 0;
}

.bar-filled {
  background: hsl(142 71% 45%);
}

/* ===== 汇总卡片 ===== */
.summary-cards {
  display: flex;
  gap: 12px;
  margin-top: 20px;
}

.summary-card {
  flex: 1;
  padding: 14px 16px;
  border-radius: 10px;
  background: hsl(var(--muted) / 0.4);
  border: 1px solid hsl(var(--border));
  text-align: center;
}

.summary-card--primary {
  background: hsl(var(--primary) / 0.06);
  border-color: hsl(var(--primary) / 0.2);
}

.summary-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  margin-bottom: 4px;
}

.summary-value {
  font-size: 20px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.summary-card--primary .summary-value {
  color: hsl(var(--primary));
}

/* ===== 信息网格 ===== */
.section-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin-bottom: 10px;
  padding-left: 8px;
  border-left: 3px solid hsl(var(--primary));
  line-height: 1.2;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0;
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  overflow: hidden;
}

.info-item {
  padding: 10px 14px;
  border-bottom: 1px solid hsl(var(--border));
  border-right: 1px solid hsl(var(--border));
  background: hsl(var(--card));
}

.info-item:nth-child(3n) {
  border-right: none;
}

.info-item:nth-last-child(-n+3):nth-child(3n+1),
.info-item:nth-last-child(-n+3):nth-child(3n+1) ~ .info-item {
  border-bottom: none;
}

.info-label {
  display: block;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin-bottom: 3px;
}

.info-value {
  display: block;
  font-size: 13px;
  color: hsl(var(--foreground));
  font-weight: 500;
  word-break: break-all;
}

/* ===== 备注框 ===== */
.remark-box {
  margin-top: 12px;
  padding: 12px 14px;
  background: hsl(var(--muted) / 0.3);
  border-radius: 8px;
  border-left: 3px solid hsl(var(--primary) / 0.3);
}

.remark-box .info-label {
  margin-bottom: 4px;
}

.remark-text {
  font-size: 13px;
  color: hsl(var(--foreground));
  line-height: 1.6;
  margin: 0;
}

/* ===== 明细表格 ===== */
.items-table :deep(.ant-table) {
  border-radius: 8px;
}

.items-table :deep(.ant-table-thead > tr > th) {
  background: hsl(var(--muted) / 0.5);
  font-weight: 600;
  font-size: 12px;
}

.items-table :deep(.ant-table-tbody > tr > td) {
  font-size: 13px;
  padding: 8px 12px;
}

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .info-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .info-item:nth-child(3n) {
    border-right: 1px solid hsl(var(--border));
  }

  .info-item:nth-child(2n) {
    border-right: none;
  }

  .summary-cards {
    flex-direction: column;
    gap: 8px;
  }

  .header-meta {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
}
</style>
