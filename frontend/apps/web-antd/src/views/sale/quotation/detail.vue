<script lang="ts" setup>
import type { TableColumnsType } from 'ant-design-vue';

import { computed, h, onMounted, ref } from 'vue';

import { formatDateTime } from '@vben/utils';

import {
  Button,
  Card,
  Descriptions,
  Empty,
  Table,
  Tag,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';

import { getQuotationInfoApi } from '#/api';

const props = defineProps<{ id: number }>();
const emit = defineEmits<{ edit: [id: string] }>();

const loading = ref(false);
const detail = ref<any>({});
const items = ref<any[]>([]);
const approvals = ref<any[]>([]);

const currencyMap: Record<number, { code: string; symbol: string }> = {
  1: { code: 'CNY', symbol: '¥' },
  2: { code: 'USD', symbol: '$' },
  3: { code: 'EUR', symbol: '€' },
  4: { code: 'GBP', symbol: '£' },
  5: { code: 'JPY', symbol: '¥' },
  6: { code: 'HKD', symbol: 'HK$' },
};

const statusMap: Record<number, { color: string; label: string }> = {
  1: { label: '草稿', color: 'default' },
  2: { label: '待审批', color: 'blue' },
  3: { label: '已审批', color: 'green' },
  4: { label: '已发送', color: 'cyan' },
  5: { label: '已接受', color: 'green' },
  6: { label: '已拒绝', color: 'red' },
  7: { label: '已过期', color: 'orange' },
  8: { label: '已转订单', color: 'purple' },
};

const approvalStatusMap: Record<number, { color: string; label: string }> = {
  1: { label: '未提交', color: 'default' },
  2: { label: '审批中', color: 'blue' },
  3: { label: '已通过', color: 'green' },
  4: { label: '已驳回', color: 'red' },
  5: { label: '需修改', color: 'orange' },
};

const approvalTypeMap: Record<number, { color: string; label: string }> = {
  1: { label: '提交审批', color: 'blue' },
  2: { label: '审批通过', color: 'green' },
  3: { label: '审批驳回', color: 'red' },
  4: { label: '修改重报', color: 'orange' },
};

const currencySymbol = computed(
  () => currencyMap[Number(detail.value.currency)]?.symbol || '¥',
);
const currencyCode = computed(
  () => currencyMap[Number(detail.value.currency)]?.code || 'CNY',
);

const summary = computed(() => {
  const total = items.value.reduce(
    (sum, item) =>
      sum + Number(item.quantity || 0) * Number(item.unitPrice || 0),
    0,
  );
  const itemDiscount = items.value.reduce(
    (sum, item) => sum + Number(item.discountAmount || 0),
    0,
  );
  const tax = items.value.reduce(
    (sum, item) => sum + Number(item.taxAmount || 0),
    0,
  );
  const totalWeight = items.value.reduce(
    (sum, item) => sum + Number(item.weight || 0) * Number(item.quantity || 0),
    0,
  );
  return { total, itemDiscount, tax, totalWeight };
});

function formatMoney(val: any): string {
  return Number(val || 0).toLocaleString('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

const itemColumns: TableColumnsType = [
  {
    title: '#',
    width: 45,
    key: 'seq',
    align: 'center',
    customRender: ({ index }: any) => index + 1,
  },
  { title: '产品信息', key: 'product', width: 220 },
  { title: '规格', dataIndex: 'spec', width: 110 },
  { title: '单位', dataIndex: 'unit', width: 60, align: 'center' },
  {
    title: '单价',
    dataIndex: 'unitPrice',
    width: 100,
    align: 'right',
    customRender: ({ text }: any) =>
      `${currencySymbol.value}${formatMoney(text)}`,
  },
  {
    title: '数量',
    dataIndex: 'quantity',
    width: 70,
    align: 'right',
    customRender: ({ text }: any) => Number(text || 0),
  },
  {
    title: '折扣额',
    dataIndex: 'discountAmount',
    width: 85,
    align: 'right',
    customRender: ({ text }: any) =>
      Number(text || 0) > 0
        ? h(
            'span',
            { class: 'text-red-500' },
            `-${currencySymbol.value}${formatMoney(text)}`,
          )
        : '-',
  },
  {
    title: '税额',
    dataIndex: 'taxAmount',
    width: 85,
    align: 'right',
    customRender: ({ text }: any) =>
      Number(text || 0) > 0
        ? `${currencySymbol.value}${formatMoney(text)}`
        : '-',
  },
  {
    title: '小计',
    dataIndex: 'subtotal',
    width: 110,
    align: 'right',
    customRender: ({ text }: any) =>
      h(
        'span',
        { class: 'font-medium text-blue-600' },
        `${currencySymbol.value}${formatMoney(text)}`,
      ),
  },
];

async function fetchDetail() {
  loading.value = true;
  try {
    const res = await getQuotationInfoApi(props.id);
    const data = res?.data ?? res;
    detail.value = data;
    items.value = Array.isArray(data.items) ? data.items : [];
    approvals.value = Array.isArray(data.approvals) ? data.approvals : [];
  } finally {
    loading.value = false;
  }
}

onMounted(() => fetchDetail());
</script>

<template>
  <div v-loading="loading" class="quotation-detail">
    <!-- 头部信息 -->
    <div class="quotation-detail__header">
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <div class="text-lg font-semibold text-gray-800">
            {{ detail.title || detail.quotationNo || '-' }}
          </div>
          <div class="mt-1 flex items-center gap-2">
            <span class="text-sm text-gray-500">{{ detail.quotationNo }}</span>
            <Tag v-if="detail.status" :color="statusMap[detail.status]?.color">
              {{ statusMap[detail.status]?.label || detail.status }}
            </Tag>
            <Tag
              v-if="detail.approvalStatus"
              :color="approvalStatusMap[detail.approvalStatus]?.color"
            >
              {{ approvalStatusMap[detail.approvalStatus]?.label }}
            </Tag>
          </div>
        </div>
        <div class="flex gap-2">
          <Button size="small" @click="emit('edit', String(detail.id))">
            编辑
          </Button>
        </div>
      </div>
    </div>

    <!-- 金额统计卡片 -->
    <div class="quotation-detail__stats">
      <div class="quotation-detail__stat-item">
        <div class="text-xs text-gray-500">商品总额</div>
        <div class="mt-1 text-lg font-semibold text-gray-800">
          {{ currencySymbol }}{{ formatMoney(detail.totalAmount) }}
        </div>
      </div>
      <div class="quotation-detail__stat-item">
        <div class="text-xs text-gray-500">折扣金额</div>
        <div class="mt-1 text-lg font-semibold text-red-500">
          -{{ currencySymbol }}{{ formatMoney(detail.discountAmount) }}
        </div>
      </div>
      <div class="quotation-detail__stat-item">
        <div class="text-xs text-gray-500">税额合计</div>
        <div class="mt-1 text-lg font-semibold text-orange-500">
          {{ currencySymbol }}{{ formatMoney(detail.taxAmount) }}
        </div>
      </div>
      <div
        class="quotation-detail__stat-item quotation-detail__stat-item--grand"
      >
        <div class="text-xs text-gray-500">报价总计</div>
        <div class="mt-1 text-xl font-bold text-blue-600">
          {{ currencySymbol }}{{ formatMoney(detail.grandTotal) }}
        </div>
        <div class="text-xs text-gray-400">{{ currencyCode }}</div>
      </div>
    </div>

    <!-- 产品明细列表 -->
    <Card size="small" title="产品明细" class="mb-3">
      <Empty v-if="items.length === 0" description="暂无产品明细" />
      <Table
        v-else
        :columns="itemColumns"
        :data-source="items"
        :pagination="false"
        size="small"
        :scroll="{ x: 900 }"
        row-key="id"
        bordered
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'product'">
            <div class="flex flex-col">
              <span class="font-medium">{{ record.productName || '-' }}</span>
              <span v-if="record.productCode" class="text-xs text-gray-400">{{
                record.productCode
              }}</span>
            </div>
          </template>
        </template>
      </Table>

      <!-- 金额汇总 -->
      <div v-if="items.length > 0" class="mt-3 flex justify-end">
        <div class="w-72 rounded-lg bg-gray-50 p-3">
          <div class="flex justify-between py-1 text-sm">
            <span class="text-gray-500">商品总额</span>
            <span>{{ currencySymbol }}{{ formatMoney(summary.total) }}</span>
          </div>
          <div class="flex justify-between py-1 text-sm">
            <span class="text-gray-500">折扣合计</span>
            <span class="text-red-500"
              >-{{ currencySymbol
              }}{{ formatMoney(summary.itemDiscount) }}</span
            >
          </div>
          <div class="flex justify-between py-1 text-sm">
            <span class="text-gray-500">税额合计</span>
            <span class="text-orange-500"
              >{{ currencySymbol }}{{ formatMoney(summary.tax) }}</span
            >
          </div>
          <div
            class="mt-1 border-t border-gray-200 pt-2 flex justify-between items-baseline"
          >
            <span class="font-medium">报价总计</span>
            <span class="text-lg font-bold text-blue-600"
              >{{ currencySymbol }}{{ formatMoney(detail.grandTotal) }}</span
            >
          </div>
          <div class="flex justify-between py-1 text-xs text-gray-400">
            <span>总重量</span>
            <span>{{ summary.totalWeight.toFixed(3) }} kg</span>
          </div>
        </div>
      </div>
    </Card>

    <!-- 基本信息 -->
    <Card size="small" title="基本信息" class="mb-3">
      <Descriptions :column="{ xs: 1, sm: 2 }" size="small" :colon="false">
        <Descriptions.Item label="客户">
          {{ detail.customerName || '-' }}
        </Descriptions.Item>
        <Descriptions.Item label="联系人">
          {{ detail.contactName || '-' }}
        </Descriptions.Item>
        <Descriptions.Item label="商机">
          {{ detail.opportunityTitle || '-' }}
        </Descriptions.Item>
        <Descriptions.Item label="币种">{{ currencyCode }}</Descriptions.Item>
        <Descriptions.Item label="报价日期">
          {{ detail.quotationDate || '-' }}
        </Descriptions.Item>
        <Descriptions.Item label="有效期至">
          {{ detail.validUntil || '-' }}
        </Descriptions.Item>
        <Descriptions.Item label="负责人">
          {{ detail.ownerUserName || '-' }}
        </Descriptions.Item>
        <Descriptions.Item label="创建时间">
          {{ formatDateTime(detail.createTime) }}
        </Descriptions.Item>
      </Descriptions>
    </Card>

    <!-- 交易条款 -->
    <Card size="small" title="交易条款" class="mb-3">
      <Descriptions :column="{ xs: 1, sm: 2 }" size="small" :colon="false">
        <Descriptions.Item label="付款条件" :span="2">
          {{ detail.paymentTerms || '-' }}
        </Descriptions.Item>
        <Descriptions.Item label="交货条款">
          {{ detail.deliveryTerms || '-' }}
        </Descriptions.Item>
        <Descriptions.Item label="交货日期">
          {{ detail.deliveryDate || '-' }}
        </Descriptions.Item>
        <Descriptions.Item label="装运港">
          {{ detail.portOfLoading || '-' }}
        </Descriptions.Item>
        <Descriptions.Item label="目的港">
          {{ detail.portOfDestination || '-' }}
        </Descriptions.Item>
      </Descriptions>
    </Card>

    <!-- 银行信息 -->
    <Card v-if="detail.bankInfo" size="small" title="银行信息" class="mb-3">
      <pre class="text-sm text-gray-600 whitespace-pre-wrap">{{
        detail.bankInfo
      }}</pre>
    </Card>

    <!-- 审批记录 -->
    <Card size="small" title="审批记录" class="mb-3">
      <Empty v-if="approvals.length === 0" description="暂无审批记录" />
      <Timeline v-else>
        <TimelineItem
          v-for="item in approvals"
          :key="item.id"
          :color="approvalTypeMap[item.approvalType]?.color || 'gray'"
        >
          <div class="flex items-center gap-2">
            <Tag
              :color="approvalTypeMap[item.approvalType]?.color || 'default'"
            >
              {{ approvalTypeMap[item.approvalType]?.label || '未知' }}
            </Tag>
            <span class="text-xs text-gray-400">{{
              formatDateTime(item.createTime)
            }}</span>
          </div>
          <div class="mt-1 text-sm text-gray-600">
            审批人：{{ item.approverName || '-' }}
          </div>
          <div v-if="item.originalAmount" class="text-sm text-gray-600">
            原报价：{{ currencySymbol }}{{ formatMoney(item.originalAmount) }}
            <span v-if="item.adjustedAmount">
              → 调整后：{{ currencySymbol
              }}{{ formatMoney(item.adjustedAmount) }}
            </span>
          </div>
          <div v-if="item.approvalRemark" class="mt-1 text-sm text-gray-700">
            意见：{{ item.approvalRemark }}
          </div>
        </TimelineItem>
      </Timeline>
    </Card>

    <!-- 备注 -->
    <Card v-if="detail.remark" size="small" title="备注">
      <p class="text-sm text-gray-600">{{ detail.remark }}</p>
    </Card>
  </div>
</template>

<style scoped>
.quotation-detail {
  padding: 16px;
}

.quotation-detail__header {
  padding: 12px 0 16px;
  border-bottom: 1px solid #f0f0f0;
}

.quotation-detail__stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  padding: 16px 0;
  border-bottom: 1px solid #f0f0f0;
}

@media (max-width: 640px) {
  .quotation-detail__stats {
    grid-template-columns: repeat(2, 1fr);
  }
}

.quotation-detail__stat-item {
  padding: 12px;
  text-align: center;
  background: #fafafa;
  border-radius: 8px;
}

.quotation-detail__stat-item--grand {
  background: #eff6ff;
}
</style>
