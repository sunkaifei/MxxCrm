<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Card,
  Col,
  Form,
  InputNumber,
  message,
  Modal,
  Row,
  Select,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteDeliveryApi,
  getDeliveryInfoApi,
  getDeliveryListApi,
  resendDeliveryApi,
  viewFullDeliveryApi,
} from '#/api/core/sale/delivery';
import { useDataScopeTabs } from '#/composables/use-data-scope-tabs';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
import { $t } from '#/locales';

import RecycleBin from '../../crm/components/RecycleBin.vue';

const accessStore = useAccessStore();

// 回收站 Tab 仅超管可见（与其他模块一致）
const { isSuperAdmin } = useSuperAdminGuard();

// 全部/下属虚拟订单 Tab 显示条件
const { canViewAll, canViewSubordinate } = useDataScopeTabs();

// 交付状态映射：1=待发送、2=已发送、3=已签收、4=已撤销、5=已失效
const statusMap: Record<number, { color: string; label: string }> = {
  1: { label: '待发送', color: 'default' },
  2: { label: '已发送', color: 'processing' },
  3: { label: '已签收', color: 'green' },
  4: { label: '已撤销', color: 'red' },
  5: { label: '已失效', color: 'orange' },
};

// 交付方式映射：1=卡密、2=下载链接、3=账号密码、4=激活码、5=服务开通
const deliveryMethodMap: Record<number, { color: string; label: string }> = {
  1: { label: '卡密', color: 'blue' },
  2: { label: '下载链接', color: 'cyan' },
  3: { label: '账号密码', color: 'orange' },
  4: { label: '激活码', color: 'green' },
  5: { label: '服务开通', color: 'purple' },
};

// 交付类型：1=自动 2=手动
const deliverTypeMap: Record<number, { color: string; label: string }> = {
  1: { label: '自动', color: 'blue' },
  2: { label: '手动', color: 'gold' },
};

const detailVisible = ref(false);
const detailData = ref<any>({});
const detailLoading = ref(false);
const fullContentVisible = ref(false);
const fullContent = ref('');
const fullLoading = ref(false);

// ========== 选项卡：全部虚拟订单 / 我的虚拟订单 / 下属虚拟订单 / 回收站 ==========
const allTabList = [
  { key: 'all', label: '全部虚拟订单' },
  { key: 'my', label: '我的虚拟订单' },
  { key: 'subordinate', label: '下属虚拟订单' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter((t) => keys.includes(t.key));
});

const activeTab = ref('my');

function handleTabChange(key: number | string) {
  activeTab.value = String(key);
  // 回收站视图由 RecycleBin 组件自行查询，不触发业务列表
  if (key === 'recycle') return;
  gridApi.query();
}

// ========== 搜索表单（与客户/联系人列表同构：手动表单置于筛选卡片，搜索按钮触发查询） ==========
const searchForm = ref<{
  customerId?: number;
  deliveryMethod?: number;
  orderId?: number;
  status?: number;
}>({});

const statusOptions = [
  { label: '待发送', value: 1 },
  { label: '已发送', value: 2 },
  { label: '已签收', value: 3 },
  { label: '已撤销', value: 4 },
  { label: '已失效', value: 5 },
];

const deliveryMethodOptions = [
  { label: '卡密', value: 1 },
  { label: '下载链接', value: 2 },
  { label: '账号密码', value: 3 },
  { label: '激活码', value: 4 },
  { label: '服务开通', value: 5 },
];

function handleSearch() {
  gridApi.query();
}

function handleReset() {
  searchForm.value = {};
  gridApi.query();
}

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
        };
        if (searchForm.value.orderId) params.orderId = searchForm.value.orderId;
        if (searchForm.value.customerId)
          params.customerId = searchForm.value.customerId;
        if (searchForm.value.status) params.status = searchForm.value.status;
        if (searchForm.value.deliveryMethod)
          params.deliveryMethod = searchForm.value.deliveryMethod;
        const result = await getDeliveryListApi(params);
        // 无数据固定 600px（空态居中）；有数据默认 600px，内容超过则响应式撑高
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          if (items.length === 0) {
            gridEl.style.setProperty('height', '600px', 'important');
            gridEl.style.removeProperty('min-height');
          } else {
            gridEl.style.removeProperty('height');
            gridEl.style.setProperty('min-height', '600px', 'important');
          }
        }
        return result;
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

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

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
  <Page>
    <!-- 筛选卡片：选项卡 + 搜索表单/回收站视图（与客户/联系人列表同构） -->
    <Card :bordered="false" class="delivery-list-filter-card">
      <Tabs
        v-model:active-key="activeTab"
        class="mb-4"
        @change="handleTabChange"
      >
        <Tabs.TabPane
          v-for="tab in tabList"
          :key="tab.key"
          :tab="tab.label"
        />
        <Tabs.TabPane v-if="isSuperAdmin" key="recycle" tab="回收站" />
      </Tabs>

      <!-- 回收站视图：与其他模块共用 RecycleBin，module=delivery -->
      <RecycleBin v-show="activeTab === 'recycle'" :module="'delivery'" />

      <Form
        v-show="activeTab !== 'recycle'"
        :model="searchForm"
        layout="inline"
        :label-col="{ style: { width: '90px' } }"
        class="delivery-search-form"
        @keyup.enter="handleSearch"
      >
        <Row :gutter="[16, 12]" style="width: 100%">
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="订单ID" name="orderId">
              <InputNumber
                v-model:value="searchForm.orderId"
                placeholder="请输入订单ID"
                allow-clear
                style="width: 100%"
                :controls="false"
              />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="客户ID" name="customerId">
              <InputNumber
                v-model:value="searchForm.customerId"
                placeholder="请输入客户ID"
                allow-clear
                style="width: 100%"
                :controls="false"
              />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="交付状态" name="status">
              <Select
                v-model:value="searchForm.status"
                placeholder="请选择"
                allow-clear
                style="width: 100%"
                :options="statusOptions"
              />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="交付方式" name="deliveryMethod">
              <Select
                v-model:value="searchForm.deliveryMethod"
                placeholder="请选择"
                allow-clear
                style="width: 100%"
                :options="deliveryMethodOptions"
              />
            </Form.Item>
          </Col>
        </Row>
        <div class="mt-3 flex flex-wrap items-center gap-2">
          <Button type="default" @click="handleSearch">搜索</Button>
          <Button type="default" @click="handleReset">刷新</Button>
        </div>
      </Form>
    </Card>

    <Grid
      v-show="activeTab !== 'recycle'"
      :table-title="$t('page.sale.delivery.title')"
      class="delivery-grid-card"
    >
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
          {{
            row.deliveryMethodName ||
            deliveryMethodMap[row.deliveryMethod]?.label
          }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #cardKeyMasked="{ row }">
        <span class="font-mono">
          {{ row.cardKeyMasked || '-' }}
        </span>
      </template>

      <template #status="{ row }">
        <Tag
          v-if="row.status && statusMap[row.status]"
          :color="statusMap[row.status]?.color"
        >
          {{ row.statusName || statusMap[row.status]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #deliverType="{ row }">
        <Tag
          v-if="row.deliverType && deliverTypeMap[row.deliverType]"
          :color="deliverTypeMap[row.deliverType]?.color"
        >
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
      <a-descriptions
        :column="1"
        bordered
        size="small"
        :loading="detailLoading"
      >
        <a-descriptions-item label="交付单号">
          {{ detailData.deliveryNo }}
        </a-descriptions-item>
        <a-descriptions-item label="订单ID">
          {{ detailData.orderId }}
        </a-descriptions-item>
        <a-descriptions-item label="商品名">
          {{ detailData.productName }}
        </a-descriptions-item>
        <a-descriptions-item label="交付方式">
          {{ detailData.deliveryMethodName }}
        </a-descriptions-item>
        <a-descriptions-item label="卡密(脱敏)">
          <span class="font-mono">{{ detailData.cardKeyMasked }}</span>
        </a-descriptions-item>
        <a-descriptions-item label="状态">
          {{ detailData.statusName }}
        </a-descriptions-item>
        <a-descriptions-item label="发送时间">
          {{ formatDateTime(detailData.sentTime) }}
        </a-descriptions-item>
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
/* 筛选卡片与表格卡片间距（scoped 固化，不依赖 Tailwind 工具类） */
.delivery-list-filter-card {
  margin-bottom: 16px;
}

.delivery-grid-card {
  margin-top: 16px;
}

.delivery-search-form :deep(.ant-form-item) {
  margin-bottom: 0;
}

.delivery-search-form :deep(.ant-form-item-control) {
  flex: 1;
}

.delivery-list__no-link {
  font-family: 'JetBrains Mono', 'Cascadia Code', Menlo, Consolas, monospace;
  font-size: 13px;
  font-weight: 600;
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
  padding: 12px;
  margin: 0;
  overflow: auto;
  font-family: 'JetBrains Mono', 'Cascadia Code', Menlo, Consolas, monospace;
  font-size: 13px;
  word-break: break-all;
  white-space: pre-wrap;
  background-color: #f5f5f5;
  border-radius: 4px;
}
</style>
