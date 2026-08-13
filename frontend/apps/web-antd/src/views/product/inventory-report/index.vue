<script lang="ts" setup>
import { ref } from 'vue';

import { Page } from '@vben/common-ui';

import { Button, Card, DatePicker, Form, Tabs } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  getCostReportApi,
  getObsoleteReportApi,
  getStockReportApi,
  getTurnoverReportApi,
} from '#/api/core/product/report';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import WarehouseDetailDrawer from '../components/WarehouseDetailDrawer.vue';

// 仓库详情抽屉
const warehouseDetailVisible = ref(false);
const warehouseDetailId = ref<number | null>(null);

function openWarehouseDetail(row: any) {
  if (!row.warehouseId) return;
  warehouseDetailId.value = Number(row.warehouseId);
  warehouseDetailVisible.value = true;
}

// 报表类型
const reportTypeOptions = [
  { label: $t('page.product.inventory.report.type.stockReport'), value: 'stock' },
  { label: $t('page.product.inventory.report.type.turnover'), value: 'turnover' },
  { label: $t('page.product.inventory.report.type.staleList'), value: 'stale' },
  { label: $t('page.product.inventory.report.type.costReport'), value: 'cost' },
];

const activeTab = ref('stock');

// 共享搜索表单
const searchForm = ref({
  dateRange: [] as string[],
});

function handleSearch() {
  switch (activeTab.value) {
    case 'stock': { stockGridApi.query(); break; }
    case 'turnover': { turnoverGridApi.query(); break; }
    case 'stale': { staleGridApi.query(); break; }
    case 'cost': { costGridApi.query(); break; }
  }
}

function handleReset() {
  searchForm.value.dateRange = [];
  handleSearch();
}

function handleGridHeight(api: any, data: any[]) {
  const gridEl = api.grid?.$el as HTMLElement | undefined;
  if (gridEl) {
    gridEl.style.height = data.length === 0 ? '200px' : '';
  }
}

// 收发存报表
const stockGridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async () => {
        const res: any = await getStockReportApi({
          startDate: searchForm.value.dateRange?.[0],
          endDate: searchForm.value.dateRange?.[1],
        });
        const items = Array.isArray(res) ? res : (res?.items ?? []);
        handleGridHeight(stockGridApi, items);
        return { items, total: items.length };
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inventory.report.field.productName'), field: 'productName', minWidth: 140 },
    { title: $t('page.product.inventory.report.field.warehouseName'), field: 'warehouseName', width: 120, slots: { default: 'warehouseName' } },
    { title: $t('page.product.inventory.report.field.openingQty'), field: 'beginQuantity', width: 110 },
    { title: $t('page.product.inventory.report.field.inboundQty'), field: 'inboundQuantity', width: 110 },
    { title: $t('page.product.inventory.report.field.outboundQty'), field: 'outboundQuantity', width: 110 },
    { title: $t('page.product.inventory.report.field.closingQty'), field: 'endQuantity', width: 110 },
  ],
};

// 库存周转率报表
const turnoverGridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async () => {
        const res: any = await getTurnoverReportApi({
          startDate: searchForm.value.dateRange?.[0],
          endDate: searchForm.value.dateRange?.[1],
        });
        const items = Array.isArray(res) ? res : (res?.items ?? []);
        handleGridHeight(turnoverGridApi, items);
        return { items, total: items.length };
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inventory.report.field.productName'), field: 'productName', minWidth: 140 },
    { title: $t('page.product.inventory.report.field.warehouseName'), field: 'warehouseName', width: 120, slots: { default: 'warehouseName' } },
    { title: $t('page.product.inventory.report.field.turnoverRate'), field: 'turnoverRate', width: 120 },
    { title: $t('page.product.inventory.report.field.avgInventory'), field: 'avgQuantity', width: 120 },
    { title: $t('page.product.inventory.report.field.outboundQty'), field: 'outboundQuantity', width: 110 },
  ],
};

// 呆滞库存清单报表
const staleGridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async () => {
        const res: any = await getObsoleteReportApi({
          days: 90,
        });
        const items = Array.isArray(res) ? res : (res?.items ?? []);
        handleGridHeight(staleGridApi, items);
        return { items, total: items.length };
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inventory.report.field.productName'), field: 'productName', minWidth: 140 },
    { title: $t('page.product.inventory.report.field.warehouseName'), field: 'warehouseName', width: 120, slots: { default: 'warehouseName' } },
    { title: $t('page.product.inventory.report.field.currentQty'), field: 'quantity', width: 110 },
    { title: $t('page.product.inventory.report.field.staleDays'), field: 'obsoleteDays', width: 110 },
    { title: $t('page.product.inventory.report.field.lastMovement'), field: 'lastOutboundTime', width: 160 },
  ],
};

// 库存成本报表
const costGridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async () => {
        const res: any = await getCostReportApi({});
        const items = Array.isArray(res) ? res : (res?.items ?? []);
        handleGridHeight(costGridApi, items);
        return { items, total: items.length };
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inventory.report.field.productName'), field: 'productName', minWidth: 140 },
    { title: $t('page.product.inventory.report.field.warehouseName'), field: 'warehouseName', width: 120, slots: { default: 'warehouseName' } },
    { title: $t('page.product.inventory.report.field.unitCost'), field: 'lastInCost', width: 110 },
    { title: $t('page.product.inventory.report.field.totalCost'), field: 'totalCost', width: 110 },
    { title: $t('page.product.inventory.report.field.avgCost'), field: 'avgCost', width: 110 },
  ],
};

const [StockGrid, stockGridApi] = useVbenVxeGrid({ gridOptions: stockGridOptions });
const [TurnoverGrid, turnoverGridApi] = useVbenVxeGrid({ gridOptions: turnoverGridOptions });
const [StaleGrid, staleGridApi] = useVbenVxeGrid({ gridOptions: staleGridOptions });
const [CostGrid, costGridApi] = useVbenVxeGrid({ gridOptions: costGridOptions });

function handleTabChange(key: string | number) {
  activeTab.value = key as string;
  switch (key) {
    case 'stock': { stockGridApi.query(); break; }
    case 'turnover': { turnoverGridApi.query(); break; }
    case 'stale': { staleGridApi.query(); break; }
    case 'cost': { costGridApi.query(); break; }
  }
}
</script>

<template>
  <Page>
    <InventoryProcessGuide current-step="report" />
    <Card :bordered="false" class="mb-4">
      <Tabs v-model:active-key="activeTab" @change="handleTabChange" class="mb-4">
        <Tabs.TabPane
          v-for="item in reportTypeOptions"
          :key="item.value"
          :tab="item.label"
        />
      </Tabs>

      <Form layout="inline" :model="searchForm">
        <Form.Item :label="$t('page.product.inventory.report.field.dateRange')">
          <DatePicker.RangePicker
            v-model:value="searchForm.dateRange"
            :placeholder="[$t('ui.placeholder.startDate'), $t('ui.placeholder.endDate')]"
            value-format="YYYY-MM-DD"
            allow-clear
          />
        </Form.Item>
        <Form.Item>
          <Button type="primary" @click="handleSearch">{{ $t('ui.button.search') }}</Button>
        </Form.Item>
        <Form.Item>
          <Button @click="handleReset">{{ $t('ui.button.reset') }}</Button>
        </Form.Item>
      </Form>
    </Card>

    <div v-show="activeTab === 'stock'" class="mt-4">
      <StockGrid :table-title="$t('page.product.inventory.report.type.stockReport')">
        <template #warehouseName="{ row }">
          <a class="text-primary hover:underline cursor-pointer" @click="openWarehouseDetail(row)">{{ row.warehouseName || '-' }}</a>
        </template>
      </StockGrid>
    </div>
    <div v-show="activeTab === 'turnover'" class="mt-4">
      <TurnoverGrid :table-title="$t('page.product.inventory.report.type.turnover')">
        <template #warehouseName="{ row }">
          <a class="text-primary hover:underline cursor-pointer" @click="openWarehouseDetail(row)">{{ row.warehouseName || '-' }}</a>
        </template>
      </TurnoverGrid>
    </div>
    <div v-show="activeTab === 'stale'" class="mt-4">
      <StaleGrid :table-title="$t('page.product.inventory.report.type.staleList')">
        <template #warehouseName="{ row }">
          <a class="text-primary hover:underline cursor-pointer" @click="openWarehouseDetail(row)">{{ row.warehouseName || '-' }}</a>
        </template>
      </StaleGrid>
    </div>
    <div v-show="activeTab === 'cost'" class="mt-4">
      <CostGrid :table-title="$t('page.product.inventory.report.type.costReport')">
        <template #warehouseName="{ row }">
          <a class="text-primary hover:underline cursor-pointer" @click="openWarehouseDetail(row)">{{ row.warehouseName || '-' }}</a>
        </template>
      </CostGrid>
    </div>

    <!-- 仓库详情抽屉 -->
    <WarehouseDetailDrawer
      :visible="warehouseDetailVisible"
      :warehouse-id="warehouseDetailId"
      @update:visible="(val) => (warehouseDetailVisible = val)"
    />
  </Page>
</template>
