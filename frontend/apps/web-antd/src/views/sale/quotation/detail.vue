<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';

import { LucideDollarSign, LucideFileText, LucideUser } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import { Card, Descriptions, Divider, Tag } from 'ant-design-vue';

import { getQuotationInfoApi } from '#/api';

const props = defineProps<{ id: number }>();
const emit = defineEmits<{ edit: [id: string] }>();

const loading = ref(false);
const detail = ref<any>({});

const statusColorMap: Record<string, string> = {
  draft: 'default',
  sent: 'blue',
  accepted: 'green',
  rejected: 'red',
  expired: 'orange',
  revised: 'purple',
};

const statusLabelMap: Record<string, string> = {
  draft: '草稿',
  sent: '已发送',
  accepted: '已接受',
  rejected: '已拒绝',
  expired: '已过期',
  revised: '已修订',
};

const canEdit = computed(() => ['draft'].includes(detail.value.status));

async function fetchDetail() {
  loading.value = true;
  try {
    const res = await getQuotationInfoApi(props.id);
    detail.value = res.data ?? res;
  } finally {
    loading.value = false;
  }
}

onMounted(() => fetchDetail());
</script>

<template>
  <div v-loading="loading" class="p-4">
    <!-- 头部卡片 -->
    <div class="flex items-start justify-between mb-4">
      <div class="flex items-center gap-3">
        <div class="flex items-center justify-center w-10 h-10 rounded-lg bg-blue-50">
          <LucideFileText class="w-5 h-5 text-blue-500" />
        </div>
        <div>
          <div class="text-lg font-semibold text-gray-800">{{ detail.title || '-' }}</div>
          <div class="flex items-center gap-2 mt-1">
            <span class="text-sm text-gray-500">{{ detail.quotationNo }}</span>
            <Tag :color="statusColorMap[detail.status]">{{ statusLabelMap[detail.status] || detail.status }}</Tag>
          </div>
        </div>
      </div>
      <div class="flex gap-2">
        <a-button v-if="canEdit" size="small" @click="emit('edit', String(detail.id))">编辑</a-button>
        <a-button v-if="detail.status === 'sent'" type="primary" size="small">转为订单</a-button>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-4 gap-3 mb-4">
      <div class="p-3 text-center bg-gray-50 rounded-lg">
        <div class="text-xs text-gray-500">总金额</div>
        <div class="text-lg font-semibold text-blue-600">{{ detail.currency }} {{ detail.grandTotal || '-' }}</div>
      </div>
      <div class="p-3 text-center bg-gray-50 rounded-lg">
        <div class="text-xs text-gray-500">币种</div>
        <div class="text-lg font-semibold">{{ detail.currency || '-' }}</div>
      </div>
      <div class="p-3 text-center bg-gray-50 rounded-lg">
        <div class="text-xs text-gray-500">有效期</div>
        <div class="text-lg font-semibold">{{ detail.validUntil || '-' }}</div>
      </div>
      <div class="p-3 text-center bg-gray-50 rounded-lg">
        <div class="text-xs text-gray-500">负责人</div>
        <div class="text-lg font-semibold">{{ detail.assignedTo || '-' }}</div>
      </div>
    </div>

    <!-- 基本信息 -->
    <Card size="small" title="基本信息" class="mb-4">
      <Descriptions :column="2" size="small" :colon="false">
        <Descriptions.Item label="客户ID">{{ detail.customerId || '-' }}</Descriptions.Item>
        <Descriptions.Item label="联系人ID">{{ detail.contactId || '-' }}</Descriptions.Item>
        <Descriptions.Item label="商机ID">{{ detail.opportunityId || '-' }}</Descriptions.Item>
        <Descriptions.Item label="状态">{{ statusLabelMap[detail.status] || detail.status }}</Descriptions.Item>
        <Descriptions.Item label="税额">{{ detail.currency }} {{ detail.taxAmount || '0' }}</Descriptions.Item>
        <Descriptions.Item label="折扣">{{ detail.currency }} {{ detail.discountAmount || '0' }}</Descriptions.Item>
        <Descriptions.Item label="付款条件" :span="2">{{ detail.paymentTerms || '-' }}</Descriptions.Item>
        <Descriptions.Item label="交货条款" :span="2">{{ detail.deliveryTerms || '-' }}</Descriptions.Item>
        <Descriptions.Item label="装运港">{{ detail.portOfLoading || '-' }}</Descriptions.Item>
        <Descriptions.Item label="目的港">{{ detail.portOfDestination || '-' }}</Descriptions.Item>
        <Descriptions.Item label="交货日期">{{ detail.deliveryDate || '-' }}</Descriptions.Item>
        <Descriptions.Item label="创建时间">{{ formatDateTime(detail.createdAt) }}</Descriptions.Item>
      </Descriptions>
    </Card>

    <!-- 银行信息 -->
    <Card v-if="detail.bankInfo" size="small" title="银行信息" class="mb-4">
      <pre class="text-sm text-gray-600 whitespace-pre-wrap">{{ detail.bankInfo }}</pre>
    </Card>

    <!-- 备注 -->
    <Card v-if="detail.notes" size="small" title="备注">
      <p class="text-sm text-gray-600">{{ detail.notes }}</p>
    </Card>
  </div>
</template>