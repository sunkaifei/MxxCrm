<script lang="ts" setup>
import { ref, watch } from 'vue';

import { Empty, Spin, Tag, Timeline } from 'ant-design-vue';

import { getEditLogListApi } from '#/api';
import { formatDateTime } from '@vben/utils';

interface EditLogItem {
  field: string;
  fieldLabel: string;
  old?: string;
  new?: string;
}

interface EditLogRecord {
  id: number;
  businessType?: number;
  businessId?: number;
  businessNo?: string;
  businessTitle?: string;
  editorId?: number;
  editorName?: string;
  content?: EditLogItem[];
  editTime?: string;
}

const props = defineProps<{
  shipmentId: number | null | undefined;
  // 触发刷新的 key（外部修改后递增此值）
  refreshKey?: number;
}>();

const loading = ref(false);
const logs = ref<EditLogRecord[]>([]);

// 操作类型颜色映射
const actionColorMap: Record<string, string> = {
  新建发货单: '#10B981',
  删除发货单: '#EF4444',
  签收发货单: '#3B82F6',
};

// 字段标签颜色（按字段类别分组）
const fieldTagColorMap: Record<string, string> = {
  action: 'gold',
  shipmentNo: 'blue',
  totalQuantity: 'geekblue',
  items: 'cyan',
  shipmentDate: 'default',
  logisticsCompany: 'default',
  trackingNo: 'default',
  shippingMethod: 'default',
  receiverName: 'default',
  receiverPhone: 'default',
  shippingAddress: 'default',
  remark: 'default',
};

// 配送方式 / 状态等枚举值翻译
const valueLabelMap: Record<string, Record<string, string>> = {
  shippingMethod: {
    '1': '快递',
    '2': '物流',
    '3': '自提',
    '4': '送货上门',
    '5': '其他',
  },
  status: {
    '1': '待发货',
    '2': '已发货',
    '3': '已签收',
    '4': '已取消',
  },
};

function getFieldTagColor(field: string): string {
  return fieldTagColorMap[field] ?? 'default';
}

function getDisplayValue(field: string, val?: string | null): string {
  if (val === null || val === undefined || val === '') return '-';
  const mapper = valueLabelMap[field];
  if (mapper && mapper[val]) return mapper[val];
  return val;
}

// 推断操作的圆点颜色
function getDotColor(log: EditLogRecord): string {
  const actionItem = log.content?.find((c) => c.field === 'action');
  if (actionItem?.new && actionColorMap[actionItem.new]) {
    return actionColorMap[actionItem.new]!;
  }
  // 普通修改用琥珀色
  return '#F59E0B';
}

// 推断操作类型标签
function getActionLabel(log: EditLogRecord): string {
  const actionItem = log.content?.find((c) => c.field === 'action');
  return actionItem?.new || '修改';
}

// 推断操作类型颜色
function getActionLabelColor(log: EditLogRecord): string {
  const action = getActionLabel(log);
  return actionColorMap[action] ?? '#F59E0B';
}

// 过滤 action 字段，不参与字段变更列表
function getFieldChanges(log: EditLogRecord): EditLogItem[] {
  return (log.content || []).filter((c) => c.field !== 'action');
}

async function fetchLogs() {
  if (!props.shipmentId) {
    logs.value = [];
    return;
  }
  loading.value = true;
  try {
    const res: any = await getEditLogListApi({
      businessType: 4, // BUSINESS_TYPE_SHIPMENT
      businessId: props.shipmentId,
      page: 1,
      pageSize: 50,
    });
    const data = res?.data ?? res ?? {};
    logs.value = data.items || data.list || data.rows || [];
  } catch (e) {
    console.error('[发货日志] 加载失败:', e);
    logs.value = [];
  } finally {
    loading.value = false;
  }
}

watch(
  () => [props.shipmentId, props.refreshKey],
  () => {
    fetchLogs();
  },
  { immediate: true },
);
</script>

<template>
  <div class="shipment-timeline">
    <div class="shipment-timeline__header">
      <div class="shipment-timeline__title">
        <span class="shipment-timeline__icon"></span>
        <span>操作追溯时间轴</span>
      </div>
      <span class="shipment-timeline__count">{{ logs.length }} 条记录</span>
    </div>

    <Spin :spinning="loading">
      <Timeline v-if="logs.length > 0" class="shipment-timeline__list">
        <Timeline.Item
          v-for="log in logs"
          :key="log.id"
          :color="getDotColor(log)"
        >
          <!-- 时间轴头部：操作类型 + 操作人 + 时间 -->
          <div class="shipment-timeline__item-header">
            <div class="shipment-timeline__user">
              <span
                class="shipment-timeline__avatar"
                :style="{ backgroundColor: getActionLabelColor(log) }"
              >
                {{ log.editorName?.charAt(0) || '?' }}
              </span>
              <span class="shipment-timeline__name">
                {{ log.editorName || '系统' }}
              </span>
              <span
                class="shipment-timeline__action-tag"
                :style="{
                  color: getActionLabelColor(log),
                  borderColor: getActionLabelColor(log),
                }"
              >
                {{ getActionLabel(log) }}
              </span>
            </div>
            <span class="shipment-timeline__time">
              {{ log.editTime ? formatDateTime(log.editTime) : '-' }}
            </span>
          </div>

          <!-- 字段变更明细 -->
          <div
            v-if="getFieldChanges(log).length > 0"
            class="shipment-timeline__changes"
          >
            <div
              v-for="(item, idx) in getFieldChanges(log)"
              :key="idx"
              class="shipment-timeline__change"
            >
              <Tag :color="getFieldTagColor(item.field)" class="shipment-timeline__field-tag">
                {{ item.fieldLabel }}
              </Tag>
              <div class="shipment-timeline__values">
                <template v-if="item.old !== null && item.old !== undefined && item.new !== null && item.new !== undefined">
                  <span class="shipment-timeline__old">{{ getDisplayValue(item.field, item.old) }}</span>
                  <span class="shipment-timeline__arrow">→</span>
                  <span class="shipment-timeline__new">{{ getDisplayValue(item.field, item.new) }}</span>
                </template>
                <template v-else-if="item.new === null || item.new === undefined">
                  <span class="shipment-timeline__deleted">删除: {{ getDisplayValue(item.field, item.old) }}</span>
                </template>
                <template v-else>
                  <span class="shipment-timeline__new">{{ getDisplayValue(item.field, item.new) }}</span>
                </template>
              </div>
            </div>
          </div>
        </Timeline.Item>
      </Timeline>
      <Empty
        v-else
        description="暂无操作记录"
        :image-style="{ height: '60px' }"
        class="shipment-timeline__empty"
      />
    </Spin>
  </div>
</template>

<style scoped>
.shipment-timeline {
  --blp-bg: #FAFBFC;
  --blp-border: #E5E7EB;
  --blp-accent: #F59E0B;
  --blp-deep: #0F2942;
  --blp-success: #10B981;
  --blp-danger: #EF4444;
  --blp-info: #3B82F6;

  padding: 16px;
  background: var(--blp-bg);
  border: 1px solid var(--blp-border);
  border-radius: 6px;
  font-family: 'JetBrains Mono', 'Cascadia Code', Menlo, Consolas, monospace;
}

.shipment-timeline__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  margin-bottom: 16px;
  border-bottom: 1px dashed var(--blp-border);
}

.shipment-timeline__title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--blp-deep);
  font-weight: 600;
  font-size: 13px;
  letter-spacing: 0.5px;
}

.shipment-timeline__icon {
  display: inline-block;
  width: 4px;
  height: 14px;
  background: var(--blp-accent);
}

.shipment-timeline__count {
  color: #6B7280;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}

.shipment-timeline__list {
  padding-left: 4px;
}

.shipment-timeline__item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
  flex-wrap: wrap;
  gap: 4px;
}

.shipment-timeline__user {
  display: flex;
  align-items: center;
  gap: 8px;
}

.shipment-timeline__avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  color: #fff;
  font-size: 11px;
  font-weight: 600;
}

.shipment-timeline__name {
  color: #374151;
  font-size: 12px;
  font-weight: 500;
}

.shipment-timeline__action-tag {
  padding: 1px 6px;
  border: 1px solid currentColor;
  border-radius: 2px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.5px;
  background: rgba(255, 255, 255, 0.6);
}

.shipment-timeline__time {
  color: #9CA3AF;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}

.shipment-timeline__changes {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px;
  background: #fff;
  border-left: 2px solid var(--blp-accent);
  border-radius: 0 4px 4px 0;
}

.shipment-timeline__change {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 4px 0;
  border-bottom: 1px dotted #F3F4F6;
}

.shipment-timeline__change:last-child {
  border-bottom: none;
}

.shipment-timeline__field-tag {
  flex-shrink: 0;
  min-width: 80px;
  text-align: center;
  font-family: inherit;
  font-size: 11px;
}

.shipment-timeline__values {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  font-size: 12px;
  line-height: 1.6;
}

.shipment-timeline__old {
  color: #9CA3AF;
  text-decoration: line-through;
  text-decoration-color: var(--blp-danger);
}

.shipment-timeline__arrow {
  color: var(--blp-accent);
  font-weight: bold;
}

.shipment-timeline__new {
  color: var(--blp-success);
  font-weight: 500;
}

.shipment-timeline__deleted {
  color: var(--blp-danger);
  font-weight: 500;
}

.shipment-timeline__empty {
  padding: 24px 0;
}
</style>
