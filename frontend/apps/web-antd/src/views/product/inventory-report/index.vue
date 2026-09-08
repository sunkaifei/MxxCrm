<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideImageOff } from '@vben/icons';

import { Button, Card, DatePicker, Form, Segmented, Tabs } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  getCostReportApi,
  getObsoleteReportApi,
  getStockReportApi,
  getTurnoverReportApi,
} from '#/api/core/product/report';
import { formatQty } from '#/components/UnitSelect';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import WarehouseDetailDrawer from '../components/WarehouseDetailDrawer.vue';

// 范围视图：后端报表接口返回权限标记动态判定"全部报表"可见性
// （超管/管理层 isManagement=true，开启仓库数据互看 crossViewEnabled=true），
// 无权用户即使落在"全部"也会自动回退"我的仓库报表"
const crossViewEnabled = ref(false);
const isManagement = ref(false);
const canViewAll = computed(
  () => isManagement.value || crossViewEnabled.value,
);
const activeScope = ref('all');
const scopeOptions = computed(() =>
  canViewAll.value
    ? [
        { label: $t('page.product.inventory.report.scope.all'), value: 'all' },
        { label: $t('page.product.inventory.report.scope.mine'), value: 'mine' },
      ]
    : [{ label: $t('page.product.inventory.report.scope.mine'), value: 'mine' }],
);

// 仓库详情抽屉
const warehouseDetailVisible = ref(false);
const warehouseDetailId = ref<null | number>(null);

function openWarehouseDetail(row: any) {
  if (!row.warehouseId) return;
  warehouseDetailId.value = Number(row.warehouseId);
  warehouseDetailVisible.value = true;
}

// 报表类型
const reportTypeOptions = [
  {
    label: $t('page.product.inventory.report.type.stockReport'),
    value: 'stock',
  },
  {
    label: $t('page.product.inventory.report.type.turnover'),
    value: 'turnover',
  },
  { label: $t('page.product.inventory.report.type.staleList'), value: 'stale' },
  { label: $t('page.product.inventory.report.type.costReport'), value: 'cost' },
];

const activeTab = ref('stock');

// 共享搜索表单
const searchForm = ref({
  dateRange: undefined as [string, string] | undefined,
});

function handleSearch() {
  switch (activeTab.value) {
    case 'cost': {
      costGridApi.query();
      break;
    }
    case 'stale': {
      staleGridApi.query();
      break;
    }
    case 'stock': {
      stockGridApi.query();
      break;
    }
    case 'turnover': {
      turnoverGridApi.query();
      break;
    }
  }
}

function handleReset() {
  searchForm.value.dateRange = undefined;
  handleSearch();
}

// 范围切换（全部报表 / 我的仓库报表）：重新加载当前报表类型
function handleScopeChange() {
  handleSearch();
}

function handleGridHeight(api: any, data: any[]) {
  const gridEl = api.grid?.$el as HTMLElement | undefined;
  if (gridEl) {
    if (data.length === 0) {
      gridEl.style.setProperty('height', '200px', 'important');
    } else {
      gridEl.style.removeProperty('height');
    }
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
        const load = async (scope: string) => {
          const res: any = await getStockReportApi({
            startDate: searchForm.value.dateRange?.[0],
            endDate: searchForm.value.dateRange?.[1],
            scope,
          });
          crossViewEnabled.value = Boolean(res?.crossViewEnabled);
          isManagement.value = Boolean(res?.isManagement);
          return res;
        };
        let res = await load(activeScope.value);
        // 普通用户（无管理/互看权限）落在"全部报表"时自动切到"我的仓库报表"
        if (
          activeScope.value === 'all' &&
          !isManagement.value &&
          !crossViewEnabled.value
        ) {
          activeScope.value = 'mine';
          res = await load('mine');
        }
        const items = Array.isArray(res) ? res : (res?.items ?? []);
        handleGridHeight(stockGridApi, items);
        return { items, total: items.length };
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: $t('page.product.inventory.report.field.productCode'),
      field: 'productCode',
      width: 130,
      align: 'left',
    },
    {
      title: $t('page.product.inventory.report.field.productImage'),
      field: 'imageUrl',
      width: 70,
      slots: { default: 'productImage' },
    },
    {
      title: $t('page.product.inventory.report.field.productName'),
      field: 'productName',
      minWidth: 140,
      align: 'left',
    },
    {
      title: '规格',
      field: 'specDesc',
      minWidth: 130,
      align: 'left',
      formatter: ({ cellValue }: any) => cellValue || '—',
    },
    {
      title: $t('page.product.inventory.report.field.warehouseName'),
      field: 'warehouseName',
      width: 120,
      slots: { default: 'warehouseName' },
    },
    {
      title: $t('page.product.inventory.report.field.openingQty'),
      field: 'beginQuantity',
      width: 110,
      formatter: ({ cellValue, row }: any) => formatQty(cellValue, row?.unit),
    },
    {
      title: $t('page.product.inventory.report.field.inboundQty'),
      field: 'inboundQuantity',
      width: 110,
      formatter: ({ cellValue, row }: any) => formatQty(cellValue, row?.unit),
    },
    {
      title: $t('page.product.inventory.report.field.outboundQty'),
      field: 'outboundQuantity',
      width: 110,
      formatter: ({ cellValue, row }: any) => formatQty(cellValue, row?.unit),
    },
    {
      title: $t('page.product.inventory.report.field.closingQty'),
      field: 'endQuantity',
      width: 110,
      formatter: ({ cellValue, row }: any) => formatQty(cellValue, row?.unit),
    },
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
        const load = async (scope: string) => {
          const res: any = await getTurnoverReportApi({
            startDate: searchForm.value.dateRange?.[0],
            endDate: searchForm.value.dateRange?.[1],
            scope,
          });
          crossViewEnabled.value = Boolean(res?.crossViewEnabled);
          isManagement.value = Boolean(res?.isManagement);
          return res;
        };
        let res = await load(activeScope.value);
        // 普通用户（无管理/互看权限）落在"全部报表"时自动切到"我的仓库报表"
        if (
          activeScope.value === 'all' &&
          !isManagement.value &&
          !crossViewEnabled.value
        ) {
          activeScope.value = 'mine';
          res = await load('mine');
        }
        const items = Array.isArray(res) ? res : (res?.items ?? []);
        handleGridHeight(turnoverGridApi, items);
        return { items, total: items.length };
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: $t('page.product.inventory.report.field.productCode'),
      field: 'productCode',
      width: 130,
      align: 'left',
    },
    {
      title: $t('page.product.inventory.report.field.productImage'),
      field: 'imageUrl',
      width: 70,
      slots: { default: 'productImage' },
    },
    {
      title: $t('page.product.inventory.report.field.productName'),
      field: 'productName',
      minWidth: 140,
      align: 'left',
    },
    {
      title: $t('page.product.inventory.report.field.warehouseName'),
      field: 'warehouseName',
      width: 120,
      slots: { default: 'warehouseName' },
    },
    {
      title: $t('page.product.inventory.report.field.turnoverRate'),
      field: 'turnoverRate',
      width: 120,
    },
    {
      title: $t('page.product.inventory.report.field.avgInventory'),
      field: 'avgQuantity',
      width: 120,
      formatter: ({ cellValue, row }: any) => formatQty(cellValue, row?.unit),
    },
    {
      title: $t('page.product.inventory.report.field.outboundQty'),
      field: 'outboundQuantity',
      width: 110,
      formatter: ({ cellValue, row }: any) => formatQty(cellValue, row?.unit),
    },
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
        const load = async (scope: string) => {
          const res: any = await getObsoleteReportApi({
            days: 90,
            scope,
          });
          crossViewEnabled.value = Boolean(res?.crossViewEnabled);
          isManagement.value = Boolean(res?.isManagement);
          return res;
        };
        let res = await load(activeScope.value);
        // 普通用户（无管理/互看权限）落在"全部报表"时自动切到"我的仓库报表"
        if (
          activeScope.value === 'all' &&
          !isManagement.value &&
          !crossViewEnabled.value
        ) {
          activeScope.value = 'mine';
          res = await load('mine');
        }
        const items = Array.isArray(res) ? res : (res?.items ?? []);
        handleGridHeight(staleGridApi, items);
        return { items, total: items.length };
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: $t('page.product.inventory.report.field.productCode'),
      field: 'productCode',
      width: 130,
      align: 'left',
    },
    {
      title: $t('page.product.inventory.report.field.productImage'),
      field: 'imageUrl',
      width: 70,
      slots: { default: 'productImage' },
    },
    {
      title: $t('page.product.inventory.report.field.productName'),
      field: 'productName',
      minWidth: 140,
      align: 'left',
    },
    {
      title: $t('page.product.inventory.report.field.warehouseName'),
      field: 'warehouseName',
      width: 120,
      slots: { default: 'warehouseName' },
    },
    {
      title: $t('page.product.inventory.report.field.currentQty'),
      field: 'quantity',
      width: 110,
      formatter: ({ cellValue, row }: any) => formatQty(cellValue, row?.unit),
    },
    {
      title: $t('page.product.inventory.report.field.staleDays'),
      field: 'obsoleteDays',
      width: 110,
    },
    {
      title: $t('page.product.inventory.report.field.lastMovement'),
      field: 'lastOutboundTime',
      width: 160,
    },
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
        const load = async (scope: string) => {
          const res: any = await getCostReportApi({
            scope,
          });
          crossViewEnabled.value = Boolean(res?.crossViewEnabled);
          isManagement.value = Boolean(res?.isManagement);
          return res;
        };
        let res = await load(activeScope.value);
        // 普通用户（无管理/互看权限）落在"全部报表"时自动切到"我的仓库报表"
        if (
          activeScope.value === 'all' &&
          !isManagement.value &&
          !crossViewEnabled.value
        ) {
          activeScope.value = 'mine';
          res = await load('mine');
        }
        const items = Array.isArray(res) ? res : (res?.items ?? []);
        handleGridHeight(costGridApi, items);
        return { items, total: items.length };
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: $t('page.product.inventory.report.field.productCode'),
      field: 'productCode',
      width: 130,
      align: 'left',
    },
    {
      title: $t('page.product.inventory.report.field.productImage'),
      field: 'imageUrl',
      width: 70,
      slots: { default: 'productImage' },
    },
    {
      title: $t('page.product.inventory.report.field.productName'),
      field: 'productName',
      minWidth: 140,
      align: 'left',
    },
    {
      title: $t('page.product.inventory.report.field.warehouseName'),
      field: 'warehouseName',
      width: 120,
      slots: { default: 'warehouseName' },
    },
    {
      title: $t('page.product.inventory.report.field.unitCost'),
      field: 'lastInCost',
      width: 110,
    },
    {
      title: $t('page.product.inventory.report.field.totalCost'),
      field: 'totalCost',
      width: 110,
    },
    {
      title: $t('page.product.inventory.report.field.avgCost'),
      field: 'avgCost',
      width: 110,
    },
  ],
};

const [StockGrid, stockGridApi] = useVbenVxeGrid({
  gridOptions: stockGridOptions,
});
const [TurnoverGrid, turnoverGridApi] = useVbenVxeGrid({
  gridOptions: turnoverGridOptions,
});
const [StaleGrid, staleGridApi] = useVbenVxeGrid({
  gridOptions: staleGridOptions,
});
const [CostGrid, costGridApi] = useVbenVxeGrid({
  gridOptions: costGridOptions,
});

function handleTabChange(key: number | string) {
  activeTab.value = key as string;
  switch (key) {
    case 'cost': {
      costGridApi.query();
      break;
    }
    case 'stale': {
      staleGridApi.query();
      break;
    }
    case 'stock': {
      stockGridApi.query();
      break;
    }
    case 'turnover': {
      turnoverGridApi.query();
      break;
    }
  }
}
</script>

<template>
  <Page>
    <InventoryProcessGuide current-step="report" />
    <Card :bordered="false" class="mb-4">
      <Segmented
        v-model:value="activeScope"
        :options="scopeOptions"
        class="mb-4"
        @change="handleScopeChange"
      />

      <Tabs
        v-model:active-key="activeTab"
        @change="handleTabChange"
        class="mb-4"
      >
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
            :placeholder="[
              $t('ui.placeholder.startDate'),
              $t('ui.placeholder.endDate'),
            ]"
            value-format="YYYY-MM-DD"
            allow-clear
          />
        </Form.Item>
        <Form.Item>
          <Button type="primary" @click="handleSearch">
            {{ $t('ui.button.search') }}
          </Button>
        </Form.Item>
        <Form.Item>
          <Button @click="handleReset">{{ $t('ui.button.reset') }}</Button>
        </Form.Item>
      </Form>
    </Card>

    <div v-show="activeTab === 'stock'" class="mt-4">
      <StockGrid
        :table-title="$t('page.product.inventory.report.type.stockReport')"
      >
        <template #productImage="{ row }">
          <div
            v-if="row.imageUrl"
            class="mx-auto h-10 w-10 flex-shrink-0 overflow-hidden rounded-lg border border-[hsl(var(--border))]"
          >
            <img
              :src="row.imageUrl"
              alt="产品图"
              class="h-full w-full object-cover"
            />
          </div>
          <div
            v-else
            class="mx-auto h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))] flex"
          >
            <LucideImageOff class="h-5 w-5 text-[hsl(var(--muted-foreground))]" />
          </div>
        </template>
        <template #warehouseName="{ row }">
          <a
            class="text-primary hover:underline cursor-pointer"
            @click="openWarehouseDetail(row)"
            >{{ row.warehouseName || '-' }}</a
          >
        </template>
      </StockGrid>
    </div>
    <div v-show="activeTab === 'turnover'" class="mt-4">
      <TurnoverGrid
        :table-title="$t('page.product.inventory.report.type.turnover')"
      >
        <template #productImage="{ row }">
          <div
            v-if="row.imageUrl"
            class="mx-auto h-10 w-10 flex-shrink-0 overflow-hidden rounded-lg border border-[hsl(var(--border))]"
          >
            <img
              :src="row.imageUrl"
              alt="产品图"
              class="h-full w-full object-cover"
            />
          </div>
          <div
            v-else
            class="mx-auto h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))] flex"
          >
            <LucideImageOff class="h-5 w-5 text-[hsl(var(--muted-foreground))]" />
          </div>
        </template>
        <template #warehouseName="{ row }">
          <a
            class="text-primary hover:underline cursor-pointer"
            @click="openWarehouseDetail(row)"
            >{{ row.warehouseName || '-' }}</a
          >
        </template>
      </TurnoverGrid>
    </div>
    <div v-show="activeTab === 'stale'" class="mt-4">
      <StaleGrid
        :table-title="$t('page.product.inventory.report.type.staleList')"
      >
        <template #productImage="{ row }">
          <div
            v-if="row.imageUrl"
            class="mx-auto h-10 w-10 flex-shrink-0 overflow-hidden rounded-lg border border-[hsl(var(--border))]"
          >
            <img
              :src="row.imageUrl"
              alt="产品图"
              class="h-full w-full object-cover"
            />
          </div>
          <div
            v-else
            class="mx-auto h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))] flex"
          >
            <LucideImageOff class="h-5 w-5 text-[hsl(var(--muted-foreground))]" />
          </div>
        </template>
        <template #warehouseName="{ row }">
          <a
            class="text-primary hover:underline cursor-pointer"
            @click="openWarehouseDetail(row)"
            >{{ row.warehouseName || '-' }}</a
          >
        </template>
      </StaleGrid>
    </div>
    <div v-show="activeTab === 'cost'" class="mt-4">
      <CostGrid
        :table-title="$t('page.product.inventory.report.type.costReport')"
      >
        <template #productImage="{ row }">
          <div
            v-if="row.imageUrl"
            class="mx-auto h-10 w-10 flex-shrink-0 overflow-hidden rounded-lg border border-[hsl(var(--border))]"
          >
            <img
              :src="row.imageUrl"
              alt="产品图"
              class="h-full w-full object-cover"
            />
          </div>
          <div
            v-else
            class="mx-auto h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))] flex"
          >
            <LucideImageOff class="h-5 w-5 text-[hsl(var(--muted-foreground))]" />
          </div>
        </template>
        <template #warehouseName="{ row }">
          <a
            class="text-primary hover:underline cursor-pointer"
            @click="openWarehouseDetail(row)"
            >{{ row.warehouseName || '-' }}</a
          >
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
