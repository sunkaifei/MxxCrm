<script lang="ts" setup>
import { ref } from 'vue';

import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';
import { useAccessStore } from '@vben/stores';

import { Button, Modal, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import type { VbenFormProps } from '@vben/common-ui';
import {
  deleteDeliveryApi,
  getDeliveryInfoApi,
  getDeliveryListApi,
  resendDeliveryApi,
  viewFullDeliveryApi,
} from '#/api/core/sale/delivery';
import { $t } from '#/locales';

const accessStore = useAccessStore();

// 交付状态映射：1=待发送、2=已发送、3=已签收、4=已撤销、5=已失效
const statusMap: Record<number, { label: string; color: string }> = {
  1: { label: '待发送', color: 'default' },
  2: { label: '已发送', color: 'processing' },
  3: { label: '已签收', color: 'green' },
  4: { label: '已撤销', color: 'red' },
  5: { label: '已失效', color: 'orange' },
};

// 交付方式映射：1=卡密、2=下载链接、3=账号密码、4=激活码、5=服务开通
const deliveryMethodMap: Record<number, { label: string; color: string }> = {
  1: { label: '卡密', color: 'blue' },
  2: { label: '下载链接', color: 'cyan' },
  3: { label: '账号密码', color: 'orange' },
  4: { label: '激活码', color: 'green' },
  5: { label: '服务开通', color: 'purple' },
};

// 交付类型：1=自动 2=手动
const deliverTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: '自动', color: 'blue' },
  2: { label: '手动', color: 'gold' },
};

const detailVisible = ref(false);
const detailData = ref<any>({});
const detailLoading = ref(false);
const fullContentVisible = ref(false);
const fullContent = ref('');
const fullLoading = ref(false);

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'InputNumber',
      fieldName: 'orderId',
      label: '订单ID',
      componentProps: {
        placeholder: '请输入订单ID',
        allowClear: true,
        controls: false,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'customerId',
      label: '客户ID',
      componentProps: {
        placeholder: '请输入客户ID',
        allowClear: true,
        controls: false,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '交付状态',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '待发送', value: 1 },
          { label: '已发送', value: 2 },
          { label: '已签收', value: 3 },
          { label: '已撤销', value: 4 },
          { label: '已失效', value: 5 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'deliveryMethod',
      label: '交付方式',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '卡密', value: 1 },
          { label: '下载链接', value: 2 },
          { label: '账号密码', value: 3 },
          { label: '激活码', value: 4 },
          { label: '服务开通', value: 5 },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          orderId: formValues.orderId,
          customerId: formValues.customerId,
          status: formValues.status,
          deliveryMethod: formValues.deliveryMethod,
        };
        return await getDeliveryListApi(params);
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: '交付单号',
      field: 'deliveryNo',
      width: 180,
      slots: { default: 'deliveryNo' },
    },
    {
      title: '订单ID',
      field: 'orderId',
      width: 100,
    },
    {
      title: '商品名',
      field: 'productName',
      minWidth: 160,
    },
    {
      title: '交付方式',
      field: 'deliveryMethod',
      width: 110,
      slots: { default: 'deliveryMethod' },
    },
    {
      title: '卡密',
      field: 'cardKeyMasked',
      width: 180,
      slots: { default: 'cardKeyMasked' },
    },
    {
      title: '交付状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: '交付类型',
      field: 'deliverType',
      width: 100,
      slots: { default: 'deliverType' },
    },
    {
      title: '发送时间',
      field: 'sentTime',
      width: 160,
      slots: { default: 'sentTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 220,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleViewDetail(row: any) {
  detailLoading.value = true;
  detailVisible.value = true;
  detailData.value = {};
  try {
    detailData.value = await getDeliveryInfoApi(row.id);
  } catch {
    message.error('获取详情失败');
  } finally {
    detailLoading.value = false;
  }
}

async function handleViewFull(row: any) {
  fullLoading.value = true;
  fullContentVisible.value = true;
  fullContent.value = '';
  try {
    const res = await viewFullDeliveryApi(row.id);
    fullContent.value =
      (res && (res.cardKey || res.fullContent || res.content)) ||
      JSON.stringify(res ?? {}, null, 2);
  } catch {
    message.error('获取完整内容失败');
  } finally {
    fullLoading.value = false;
  }
}

function handleResend(row: any) {
  Modal.confirm({
    title: '重发通知',
    content: `确定要重发交付单「${row.deliveryNo || ''}」的通知吗？`,
    okText: $t('ui.button.ok'),
    cancelText: $t('ui.button.cancel'),
    onOk: async () => {
      try {
        await resendDeliveryApi(row.id);
        message.success('重发成功');
        gridApi.query();
      } catch {
        message.error('重发失败');
      }
    },
  });
}

async function handleDelete(row: any) {
  Modal.confirm({
    title: '删除确认',
    content: `确定要删除交付单「${row.deliveryNo || ''}」吗？`,
    okText: $t('ui.button.ok'),
    cancelText: $t('ui.button.cancel'),
    onOk: async () => {
      try {
        await deleteDeliveryApi([row.id]);
        message.success('删除成功');
        gridApi.query();
      } catch {
        message.error('删除失败');
      }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.sale.delivery.title')">
      <template #deliveryNo="{ row }">
        <a
          v-if="row.deliveryNo"
          class="delivery-list__no-link"
          @click="handleViewDetail(row)"
        >
          {{ row.deliveryNo }}
        </a>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #deliveryMethod="{ row }">
        <Tag
          v-if="row.deliveryMethod && deliveryMethodMap[row.deliveryMethod]"
          :color="deliveryMethodMap[row.deliveryMethod]?.color"
        >
          {{ row.deliveryMethodName || deliveryMethodMap[row.deliveryMethod]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #cardKeyMasked="{ row }">
        <span class="font-mono">
          {{ row.cardKeyMasked || '-' }}
        </span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status && statusMap[row.status]" :color="statusMap[row.status]?.color">
          {{ row.statusName || statusMap[row.status]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #deliverType="{ row }">
        <Tag v-if="row.deliverType && deliverTypeMap[row.deliverType]" :color="deliverTypeMap[row.deliverType]?.color">
          {{ deliverTypeMap[row.deliverType]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #sentTime="{ row }">
        {{ formatDateTime(row.sentTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('sale:delivery:view')"
          type="link"
          size="small"
          @click="() => handleViewDetail(row)"
        >
          查看详情
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:delivery:view')"
          type="link"
          size="small"
          @click="() => handleViewFull(row)"
        >
          {{ $t('page.sale.delivery.button.view') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:delivery:update')"
          type="link"
          size="small"
          @click="() => handleResend(row)"
        >
          {{ $t('page.sale.delivery.button.resend') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('sale:delivery:delete')"
          type="link"
          size="small"
          danger
          @click="() => handleDelete(row)"
        >
          删除
        </Button>
      </template>
    </Grid>

    <Modal
      v-model:open="detailVisible"
      :title="$t('page.sale.delivery.title')"
      :footer="null"
      width="640px"
    >
      <a-descriptions :column="1" bordered size="small" :loading="detailLoading">
        <a-descriptions-item label="交付单号">{{ detailData.deliveryNo }}</a-descriptions-item>
        <a-descriptions-item label="订单ID">{{ detailData.orderId }}</a-descriptions-item>
        <a-descriptions-item label="商品名">{{ detailData.productName }}</a-descriptions-item>
        <a-descriptions-item label="交付方式">{{ detailData.deliveryMethodName }}</a-descriptions-item>
        <a-descriptions-item label="卡密(脱敏)">
          <span class="font-mono">{{ detailData.cardKeyMasked }}</span>
        </a-descriptions-item>
        <a-descriptions-item label="状态">{{ detailData.statusName }}</a-descriptions-item>
        <a-descriptions-item label="发送时间">{{ formatDateTime(detailData.sentTime) }}</a-descriptions-item>
      </a-descriptions>
    </Modal>

    <Modal
      v-model:open="fullContentVisible"
      :title="$t('page.sale.delivery.button.view')"
      :footer="null"
      width="640px"
    >
      <a-spin :spinning="fullLoading">
        <pre class="delivery-list__full-content">{{ fullContent }}</pre>
      </a-spin>
    </Modal>
  </Page>
</template>

<style scoped>
.delivery-list__no-link {
  font-family: 'JetBrains Mono', 'Cascadia Code', Menlo, Consolas, monospace;
  font-weight: 600;
  font-size: 13px;
  color: #0f2942;
  text-decoration: none;
  cursor: pointer;
  transition: color 0.2s;
}

.delivery-list__no-link:hover {
  color: #f59e0b;
}

.delivery-list__full-content {
  max-height: 400px;
  overflow: auto;
  margin: 0;
  padding: 12px;
  background-color: #f5f5f5;
  border-radius: 4px;
  font-family: 'JetBrains Mono', 'Cascadia Code', Menlo, Consolas, monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
