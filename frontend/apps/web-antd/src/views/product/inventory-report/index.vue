<script lang="ts" setup>
import { ref } from 'vue';

import { Page } from '@vben/common-ui';

import { Button, Card, DatePicker, Form, Tabs } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { getStockReportApi, getTurnoverReportApi } from '#/api/core/product/report';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';

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
    case 'stock': stockGridApi.query(); break;
    case 'turnover': turnoverGridApi.query(); break;
    case 'stale': staleGridApi.query(); break;
    case 'cost': costGridApi.query(); break;
  }
}

function handleReset() {
  searchForm.value.dateRange = [];
  handleSearch();
}

// 收发存报表
const stockGridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const result = await getStockReportApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          dateStart: searchForm.value.dateRange?.[0],
          dateEnd: searchForm.value.dateRange?.[1],
        });
        const items = (result as any)?.items ?? [];
        const gridEl = stockGridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '200px' : '';
        }
        return result;
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inventory.report.field.productName'), field: 'productName', minWidth: 140 },
    { title: $t('page.product.inventory.report.field.warehouseName'), field: 'warehouseName', width: 120 },
    { title: $t('page.product.inventory.report.field.openingQty'), field: 'openingQuantity', width: 110 },
    { title: $t('page.product.inventory.report.field.inboundQty'), field: 'inboundQuantity', width: 110 },
    { title: $t('page.product.inventory.report.field.outboundQty'), field: 'outboundQuantity', width: 110 },
    { title: $t('page.product.inventory.report.field.closingQty'), field: 'closingQuantity', width: 110 },
  ],
};

// 库存周转率报表
const turnoverGridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const result = await getTurnoverReportApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          dateStart: searchForm.value.dateRange?.[0],
          dateEnd: searchForm.value.dateRange?.[1],
        });
        const items = (result as any)?.items ?? [];
        const gridEl = turnoverGridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '200px' : '';
        }
        return result;
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inventory.report.field.productName'), field: 'productName', minWidth: 140 },
    { title: $t('page.product.inventory.report.field.warehouseName'), field: 'warehouseName', width: 120 },
    { title: $t('page.product.inventory.report.field.turnoverRate'), field: 'turnoverRate', width: 120 },
    { title: $t('page.product.inventory.report.field.avgInventory'), field: 'avgInventory', width: 120 },
    { title: $t('page.product.inventory.report.field.outboundQty'), field: 'outboundQuantity', width: 110 },
  ],
};

// 呆滞库存清单报表
const staleGridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const result = await getStockReportApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          staleOnly: true,
          dateStart: searchForm.value.dateRange?.[0],
          dateEnd: searchForm.value.dateRange?.[1],
        });
        const items = (result as any)?.items ?? [];
        const gridEl = staleGridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '200px' : '';
        }
        return result;
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inventory.report.field.productName'), field: 'productName', minWidth: 140 },
    { title: $t('page.product.inventory.report.field.warehouseName'), field: 'warehouseName', width: 120 },
    { title: $t('page.product.inventory.report.field.currentQty'), field: 'currentQuantity', width: 110 },
    { title: $t('page.product.inventory.report.field.staleDays'), field: 'staleDays', width: 110 },
    { title: $t('page.product.inventory.report.field.lastMovement'), field: 'lastMovementTime', width: 160 },
  ],
};

// 库存成本报表
const costGridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const result = await getStockReportApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          costOnly: true,
          dateStart: searchForm.value.dateRange?.[0],
          dateEnd: searchForm.value.dateRange?.[1],
        });
        const items = (result as any)?.items ?? [];
        const gridEl = costGridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '200px' : '';
        }
        return result;
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inventory.report.field.productName'), field: 'productName', minWidth: 140 },
    { title: $t('page.product.inventory.report.field.warehouseName'), field: 'warehouseName', width: 120 },
    { title: $t('page.product.inventory.report.field.unitCost'), field: 'unitCost', width: 110 },
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
  // 切换 tab 后刷新对应表格
  switch (key) {
    case 'stock':
      stockGridApi.query();
      break;
    case 'turnover':
      turnoverGridApi.query();
      break;
    case 'stale':
      staleGridApi.query();
      break;
    case 'cost':
      costGridApi.query();
      break;
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
      <StockGrid :table-title="$t('page.product.inventory.report.type.stockReport')" />
    </div>
    <div v-show="activeTab === 'turnover'" class="mt-4">
      <TurnoverGrid :table-title="$t('page.product.inventory.report.type.turnover')" />
    </div>
    <div v-show="activeTab === 'stale'" class="mt-4">
      <StaleGrid :table-title="$t('page.product.inventory.report.type.staleList')" />
    </div>
    <div v-show="activeTab === 'cost'" class="mt-4">
      <CostGrid :table-title="$t('page.product.inventory.report.type.costReport')" />
    </div>
  </Page>
</template>
