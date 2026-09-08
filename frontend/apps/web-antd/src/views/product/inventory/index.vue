<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import {
  LucideChevronDown,
  LucideChevronUp,
  LucideList,
  LucideTrash2,
  LucideUndo2,
} from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import {
  Button,
  Popconfirm,
  Segmented,
  Tabs,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  getInventoryListApi,
  purgeInventoryApi,
  recycleDeleteInventoryApi,
  restoreInventoryApi,
} from '#/api';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import StockLogDrawer from '../components/StockLogDrawer.vue';
import WarehouseSelectModal from '../inventory-check/WarehouseSelectModal.vue';

const accessStore = useAccessStore();

// ============ 视图模式（明细 / 汇总） ============
const viewMode = ref<'detail' | 'summary'>('detail');
// ===== 范围视图（所有库存/我的库存/回收站）=====
const activeTab = ref('all');
const crossViewEnabled = ref(false);
const isManagement = ref(false);
const isRecycle = computed(() => activeTab.value === 'recycle');
const tabList = [
  { key: 'all', label: '所有库存' },
  { key: 'mine', label: '我的库存' },
  { key: 'recycle', label: '回收站' },
];

async function handleTabChange() {
  gridApi.setGridOptions({ columns: buildColumns() });
  gridApi.query();
}

// ============ 统计条（总量来自接口，合计为当前页口径） ============
const stats = ref({ available: 0, cost: 0, quantity: 0, total: 0 });

function updateStats(total: number, flatList: any[]) {
  let quantity = 0;
  let available = 0;
  let cost = 0;
  for (const row of flatList) {
    quantity += Number(row.quantity ?? 0);
    available += Number(row.availableQuantity ?? 0);
    cost += Number(row.totalCost ?? 0);
  }
  stats.value = { available, cost, quantity, total };
}

// ============ 仓库弹窗选择 ============
const warehouseSelectVisible = ref(false);
const warehouseDisplay = ref('');

function openWarehouseSelect() {
  warehouseSelectVisible.value = true;
}

function onWarehouseSelected(warehouse: any) {
  warehouseDisplay.value = warehouse.warehouseName ?? warehouse.name ?? '';
  gridApi.formApi?.setValues({
    warehouseId: String(warehouse.id),
    warehouseDisplay: warehouse.warehouseName ?? warehouse.name ?? '',
  });
}

function clearWarehouse() {
  warehouseDisplay.value = '';
  gridApi.formApi?.setValues({ warehouseId: '', warehouseDisplay: '' });
}

// ============ 树形数据转换（汇总视图） ============
// 将扁平行转换为按 产品 → 规格(SKU) → 仓库 的三级树形结构
function buildTreeData(flatList: any[]): any[] {
  const productMap = new Map<number, any>();

  for (const row of flatList) {
    const pid = Number(row.productId);
    let product = productMap.get(pid);
    if (!product) {
      product = {
        _id: `product_${pid}`,
        productId: pid,
        productName: row.productName,
        productCode: row.productCode,
        warehouseId: undefined,
        warehouseName: '全部',
        quantity: 0,
        reservedQuantity: 0,
        availableQuantity: 0,
        inTransitQuantity: 0,
        frozenQuantity: 0,
        totalCost: 0,
        _isProduct: true,
        _specCount: 0,
        _warehouseCount: 0,
        children: [],
      };
      productMap.set(pid, product);
    }
    product.quantity += Number(row.quantity ?? 0);
    product.reservedQuantity += Number(row.reservedQuantity ?? 0);
    product.availableQuantity += Number(row.availableQuantity ?? 0);
    product.inTransitQuantity += Number(row.inTransitQuantity ?? 0);
    product.frozenQuantity += Number(row.frozenQuantity ?? 0);
    product.totalCost += Number(row.totalCost ?? 0);
    product._warehouseCount++;

    // 规格层级
    const skuId = row.skuId ? Number(row.skuId) : 0;
    const specKey = `spec_${pid}_${skuId}`;
    let specNode = product.children.find((c: any) => c._id === specKey);
    if (!specNode) {
      specNode = {
        _id: specKey,
        _parentId: `product_${pid}`,
        _isSpec: true,
        skuId: skuId || undefined,
        skuCode: row.skuCode,
        specText: row.specText || '默认规格',
        productName: row.specText || '默认规格',
        quantity: 0,
        reservedQuantity: 0,
        availableQuantity: 0,
        frozenQuantity: 0,
        totalCost: 0,
        children: [],
      };
      product.children.push(specNode);
      product._specCount++;
    }
    specNode.quantity += Number(row.quantity ?? 0);
    specNode.reservedQuantity += Number(row.reservedQuantity ?? 0);
    specNode.availableQuantity += Number(row.availableQuantity ?? 0);
    specNode.frozenQuantity += Number(row.frozenQuantity ?? 0);
    specNode.totalCost += Number(row.totalCost ?? 0);

    // 仓库层级（叶子节点）
    specNode.children.push({
      ...row,
      _id: `wh_${pid}_${skuId}_${row.warehouseId}`,
      _parentId: specKey,
      _isWarehouse: true,
    });
  }

  return [...productMap.values()];
}

// ============ 展开/折叠全部 ============
const allExpanded = ref(true);

function toggleExpandAll() {
  const grid = gridApi.grid;
  if (!grid) return;
  const treeData = grid.getTableData().fullData;
  if (allExpanded.value) {
    treeData.forEach((row: any) => {
      if (row._isProduct) grid.setTreeExpand(row, false);
    });
    allExpanded.value = false;
  } else {
    treeData.forEach((row: any) => {
      if (row._isProduct) grid.setTreeExpand(row, true);
    });
    allExpanded.value = true;
  }
}

// ===== 回收站操作 =====
async function handleRecycleDelete(row: any) {
  row.pending = true;
  try {
    await recycleDeleteInventoryApi([Number(row.id)]);
    window.$message.success('已移入回收站');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleRestore(row: any) {
  row.pending = true;
  try {
    await restoreInventoryApi(Number(row.id));
    window.$message.success('已恢复');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handlePurge(row: any) {
  row.pending = true;
  try {
    await purgeInventoryApi(Number(row.id));
    window.$message.success('已彻底删除');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function handleViewModeChange() {
  allExpanded.value = true;
  gridApi.setGridOptions({
    columns: viewMode.value === 'detail' ? detailColumns : summaryColumns,
  });
  gridApi.query();
}

// ============ 列定义 ============
function buildColumns(): VxeGridProps['columns'] {
  const isRecycle = activeTab.value === 'recycle';
  const cols: VxeGridProps['columns'] = [
  {
    title: $t('ui.table.seq'),
    type: 'seq',
    width: 48,
    align: 'center',
    fixed: 'left',
  },
  {
    title: '商品',
    field: 'productName',
    minWidth: 220,
    fixed: 'left',
    slots: { default: 'productCell' },
  },
  {
    title: '仓库',
    field: 'warehouseName',
    width: 130,
    slots: { default: 'warehouseCell' },
  },
  {
    title: '库存数量',
    field: 'quantity',
    width: 96,
    align: 'right',
    sortable: true,
    slots: { default: 'quantity' },
  },
  {
    title: '可用数量',
    field: 'availableQuantity',
    width: 96,
    align: 'right',
    sortable: true,
    slots: { default: 'availableQuantity' },
  },
  {
    title: '预留',
    field: 'reservedQuantity',
    width: 80,
    align: 'right',
    sortable: true,
    slots: { default: 'reservedQuantity' },
  },
  {
    title: '冻结',
    field: 'frozenQuantity',
    width: 80,
    align: 'right',
    sortable: true,
    slots: { default: 'frozenQuantity' },
  },
  {
    title: '在途',
    field: 'inTransitQuantity',
    width: 80,
    align: 'right',
    slots: { default: 'inTransitQuantity' },
  },
  {
    title: '成本单价',
    field: 'avgCost',
    width: 100,
    align: 'right',
    slots: { default: 'avgCost' },
  },
  {
    title: '库存总成本',
    field: 'totalCost',
    width: 110,
    align: 'right',
    slots: { default: 'totalCost' },
  },
  {
    title: '最近出入库',
    field: 'lastInboundTime',
    width: 150,
    slots: { default: 'recentActivity' },
  },
    ...(isRecycle
      ? [
          { title: '删除人', field: 'deleteByName', width: 100 },
          { title: '删除时间', field: 'deleteTime', width: 160 },
        ]
      : []),
  {
    title: $t('ui.table.action'),
    field: 'action',
    fixed: 'right',
    slots: { default: 'action' },
    width: isRecycle ? 110 : 72,
    align: 'center',
  },
  ];
  return cols;
}

const detailColumns = buildColumns();

const summaryColumns: VxeGridProps['columns'] = [
  {
    title: $t('ui.table.seq'),
    type: 'seq',
    width: 48,
    align: 'center',
    fixed: 'left',
  },
  {
    title: '产品 / 规格 / 仓库',
    field: 'productName',
    minWidth: 240,
    slots: { default: 'productCell' },
  },
  {
    title: '库存数量',
    field: 'quantity',
    width: 100,
    align: 'right',
    slots: { default: 'quantity' },
  },
  {
    title: '可用数量',
    field: 'availableQuantity',
    width: 100,
    align: 'right',
    slots: { default: 'availableQuantity' },
  },
  {
    title: '预留',
    field: 'reservedQuantity',
    width: 80,
    align: 'right',
    slots: { default: 'reservedQuantity' },
  },
  {
    title: '冻结',
    field: 'frozenQuantity',
    width: 80,
    align: 'right',
    slots: { default: 'frozenQuantity' },
  },
  {
    title: '库存总成本',
    field: 'totalCost',
    width: 120,
    align: 'right',
    slots: { default: 'totalCost' },
  },
  {
    title: '最近出入库',
    field: 'lastInboundTime',
    width: 150,
    slots: { default: 'recentActivity' },
  },
  {
    title: $t('ui.table.action'),
    field: 'action',
    fixed: 'right',
    slots: { default: 'action' },
    width: 72,
    align: 'center',
  },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'productName',
      label: $t('page.inventory.form.productName'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'warehouseDisplay',
      label: '仓库',
      componentProps: {
        placeholder: '点击选择仓库',
        readOnly: true,
        allowClear: true,
        style: { cursor: 'pointer' },
        onClick: () => openWarehouseSelect(),
        onChange: (e: any) => {
          if (!e?.target?.value) {
            clearWarehouse();
          }
        },
      },
    },
    {
      component: 'Input',
      fieldName: 'warehouseId',
      dependencies: { triggerFields: ['warehouseDisplay'] },
      formItemClass: 'hidden',
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
  cellConfig: { isHover: true } as any,
  stripe: true,
  // 树形配置常驻：明细视图行无 children 即为普通行
  treeConfig: {
    transform: false,
    rowField: '_id',
    parentField: '_parentId',
    expandAll: true,
  },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const load = async (scope: string) => {
          const r: any = await getInventoryListApi({
            page: page.currentPage,
            pageSize: page.pageSize,
            productName: formValues.productName,
            warehouseId: formValues.warehouseId,
            scope,
          });
          crossViewEnabled.value = Boolean(r?.crossViewEnabled);
          isManagement.value = Boolean(r?.isManagement);
          return r;
        };
        let res = await load(activeTab.value);
        // 普通用户（无管理/互看权限）默认落在"所有库存"时，自动切到"我的库存"并加载其数据
        if (
          activeTab.value === 'all' &&
          !isManagement.value &&
          !crossViewEnabled.value
        ) {
          activeTab.value = 'mine';
          res = await load('mine');
        }
        const flatList = (res?.items ?? res?.list ?? []).map(
          (row: any, idx: number) => ({
            ...row,
            _id: `row_${page.currentPage}_${idx}`,
          }),
        );
        const total = res?.total ?? flatList.length;
        updateStats(total, flatList);
        if (viewMode.value === 'summary') {
          return { items: buildTreeData(flatList), total };
        }
        return { items: flatList, total };
      },
    },
  },

  columns: detailColumns,
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// ============ 库存流水抽屉 ============
const stockLogVisible = ref(false);
const stockLogProductId = ref<null | number>(null);
const stockLogProductName = ref('');

function handleViewLog(row: any) {
  stockLogVisible.value = true;
  stockLogProductId.value = Number(row.productId);
  stockLogProductName.value = row.productName || '';
}

function formatNumber(val: any): string {
  const n = Number(val ?? 0);
  return n.toLocaleString('zh-CN');
}

function formatMoney(val: any): string {
  const n = Number(val ?? 0);
  return n.toLocaleString('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

function shortTime(val: any): string {
  const s = String(val ?? '');
  return s ? s.slice(0, 16) : '—';
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="stock" />

    <!-- 统计条：全量记录数 + 当前页合计 -->
    <div
      class="mb-1 flex flex-wrap items-center gap-x-5 gap-y-1 px-0.5 py-1 text-foreground"
    >
      <div class="flex items-baseline gap-1.5">
        <span class="text-xs text-muted-foreground">库存记录</span>
        <span class="text-sm font-semibold tabular-nums">{{
          formatNumber(stats.total)
        }}</span>
        <span class="text-xs text-muted-foreground">条</span>
      </div>
      <div class="h-4 w-px bg-border"></div>
      <div class="flex items-baseline gap-1.5">
        <span class="text-xs text-muted-foreground">本页合计 · 库存</span>
        <span class="text-sm font-semibold tabular-nums">{{
          formatNumber(stats.quantity)
        }}</span>
      </div>
      <div class="h-4 w-px bg-border"></div>
      <div class="flex items-baseline gap-1.5">
        <span class="text-xs text-muted-foreground">本页合计 · 可用</span>
        <span class="text-sm font-semibold tabular-nums">{{
          formatNumber(stats.available)
        }}</span>
      </div>
      <div class="h-4 w-px bg-border"></div>
      <div class="flex items-baseline gap-1.5">
        <span class="text-xs text-muted-foreground">本页合计 · 库存成本</span>
        <span class="text-sm font-semibold tabular-nums"
          >¥{{ formatMoney(stats.cost) }}</span
        >
      </div>
    </div>

    <Grid :table-title="$t('page.product.inventory.title')">
      <template #form-header>
        <Tabs
          v-model:active-key="activeTab"
          class="mb-3"
          @change="handleTabChange"
        >
          <Tabs.TabPane
            v-for="tab in tabList.filter((t) => t.key !== 'all' || isManagement || crossViewEnabled)"
            :key="tab.key"
            :tab="tab.label"
          />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Segmented
          v-model:value="viewMode"
          class="mr-2"
          size="small"
          :options="[
            { label: '明细', value: 'detail' },
            { label: '汇总', value: 'summary' },
          ]"
          @change="handleViewModeChange"
        />
        <Button
          v-if="viewMode === 'summary'"
          class="mr-2"
          size="small"
          @click="toggleExpandAll"
        >
          <template #icon>
            <component
              :is="allExpanded ? h(LucideChevronUp) : h(LucideChevronDown)"
            />
          </template>
          {{ allExpanded ? '全部折叠' : '全部展开' }}
        </Button>
      </template>

      <!-- 商品列：明细为两行式单元格；汇总视图为 产品/规格/仓库 三级树 -->
      <template #productCell="{ row }">
        <template v-if="row._isProduct">
          <div class="flex flex-col leading-5">
            <div class="flex flex-wrap items-center gap-2">
              <span class="font-semibold text-foreground">{{
                row.productName
              }}</span>
              <Tag color="blue" :bordered="false">
                {{ row._specCount }}种规格
              </Tag>
              <Tag color="geekblue" :bordered="false">
                {{ row._warehouseCount }}个仓库
              </Tag>
            </div>
            <span
              v-if="row.productCode"
              class="truncate font-mono text-xs text-muted-foreground"
              >{{ row.productCode }}</span
            >
          </div>
        </template>
        <template v-else-if="row._isSpec">
          <span class="inline-flex items-center gap-1">
            <svg
              viewBox="0 0 24 24"
              width="13"
              height="13"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              class="text-muted-foreground"
            >
              <rect x="3" y="3" width="7" height="7" />
              <rect x="14" y="3" width="7" height="7" />
              <rect x="3" y="14" width="7" height="7" />
              <rect x="14" y="14" width="7" height="7" />
            </svg>
            <span class="font-medium text-foreground">{{ row.specText }}</span>
            <span
              v-if="row.skuCode"
              class="font-mono text-xs text-muted-foreground"
              >{{ row.skuCode }}</span
            >
          </span>
        </template>
        <template v-else-if="row._isWarehouse">
          <span class="text-muted-foreground">
            <svg
              viewBox="0 0 24 24"
              width="13"
              height="13"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              class="mr-1 inline-block -mt-0.5"
            >
              <path d="M3 21V8l9-5 9 5v13" />
              <path d="M3 21h18" />
            </svg>
            {{ row.warehouseName }}
          </span>
        </template>
        <template v-else>
          <!-- 明细视图：产品名 + 编码/规格 两行式 -->
          <div class="flex flex-col leading-5">
            <span class="font-medium text-foreground">{{
              row.productName || '—'
            }}</span>
            <span
              v-if="row.productCode || row.specText"
              class="truncate font-mono text-xs text-muted-foreground"
            >
              {{ row.productCode || '' }}{{ row.productCode && row.specText ? ' · ' : '' }}{{ row.specText || '' }}
            </span>
          </div>
        </template>
      </template>

      <!-- 仓库列（仅明细视图使用） -->
      <template #warehouseCell="{ row }">
        <span class="inline-flex items-center gap-1.5">
          <svg
            viewBox="0 0 24 24"
            width="14"
            height="14"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            class="text-muted-foreground"
          >
            <path d="M3 21V8l9-5 9 5v13" />
            <path d="M3 21h18" />
          </svg>
          <span>{{ row.warehouseName || '—' }}</span>
        </span>
      </template>

      <!-- 库存数量列：0 值警示 -->
      <template #quantity="{ row }">
        <span
          v-if="Number(row.quantity) === 0"
          class="font-medium text-amber-500"
          >0</span
        >
        <span
          v-else
          :class="
            row._isProduct
              ? 'font-semibold text-base'
              : row._isSpec
                ? 'font-semibold'
                : ''
          "
          >{{ formatNumber(row.quantity) }}</span
        >
      </template>

      <!-- 可用数量列：主视角数字，0 值警示 -->
      <template #availableQuantity="{ row }">
        <span
          v-if="Number(row.availableQuantity) === 0"
          class="font-semibold text-amber-500"
          >0</span
        >
        <span v-else class="font-semibold">{{
          formatNumber(row.availableQuantity)
        }}</span>
      </template>

      <!-- 预留数量列 -->
      <template #reservedQuantity="{ row }">
        <span v-if="Number(row.reservedQuantity) > 0">{{
          formatNumber(row.reservedQuantity)
        }}</span>
        <span v-else class="text-muted-foreground">—</span>
      </template>

      <!-- 冻结数量列 -->
      <template #frozenQuantity="{ row }">
        <span
          v-if="Number(row.frozenQuantity) > 0"
          class="font-medium text-red-500"
          >{{ formatNumber(row.frozenQuantity) }}</span
        >
        <span v-else class="text-muted-foreground">—</span>
      </template>

      <!-- 在途数量列 -->
      <template #inTransitQuantity="{ row }">
        <span v-if="Number(row.inTransitQuantity) > 0">{{
          formatNumber(row.inTransitQuantity)
        }}</span>
        <span v-else class="text-muted-foreground">—</span>
      </template>

      <!-- 成本单价列 -->
      <template #avgCost="{ row }">
        <span v-if="Number(row.avgCost) > 0">¥{{ formatMoney(row.avgCost) }}</span>
        <span v-else class="text-muted-foreground">—</span>
      </template>

      <!-- 库存总成本列 -->
      <template #totalCost="{ row }">
        <span
          v-if="Number(row.totalCost) !== 0"
          :class="
            row._isProduct
              ? 'font-semibold text-primary'
              : row._isSpec
                ? 'font-medium'
                : ''
          "
          >¥{{ formatMoney(row.totalCost) }}</span
        >
        <span v-else class="text-muted-foreground">—</span>
      </template>

      <!-- 最近出入库列：两行式，秒位截断 -->
      <template #recentActivity="{ row }">
        <div
          v-if="row.lastInboundTime || row.lastOutboundTime"
          class="flex flex-col text-xs leading-5 text-muted-foreground"
        >
          <div>
            <span class="inline-block w-4">入</span
            ><span class="tabular-nums">{{
              shortTime(row.lastInboundTime)
            }}</span>
          </div>
          <div>
            <span class="inline-block w-4">出</span
            ><span class="tabular-nums">{{
              shortTime(row.lastOutboundTime)
            }}</span>
          </div>
        </div>
        <span v-else class="text-muted-foreground">—</span>
      </template>

      <!-- 操作列：明细视图每行可看流水；汇总视图仅产品行；回收站为恢复/彻底删除 -->
      <template #action="{ row }">
        <template v-if="isRecycle">
          <Popconfirm
            title="确认恢复该库存记录？"
            ok-text="恢复"
            cancel-text="取消"
            @confirm="() => handleRestore(row)"
          >
            <Button type="link" size="small" :icon="h(LucideUndo2)" />
          </Popconfirm>
          <Popconfirm
            v-if="isManagement"
            title="彻底删除后不可恢复，确认？"
            ok-text="彻底删除"
            cancel-text="取消"
            @confirm="() => handlePurge(row)"
          >
            <Button type="link" danger size="small" :icon="h(LucideTrash2)" />
          </Popconfirm>
        </template>
        <template v-else>
          <Tooltip
            v-if="viewMode === 'summary' ? row._isProduct : true"
            :title="$t('page.inventory.tooltip.viewStockLog')"
          >
            <Button
              v-if="accessStore.hasAccessCode('product:inventory:view')"
              type="link"
              size="small"
              :icon="h(LucideList)"
              @click="() => handleViewLog(row)"
            />
          </Tooltip>
          <Popconfirm
            v-if="
              !isRecycle &&
              accessStore.hasAccessCode('product:inventory:update') &&
              !viewMode.startsWith &&
              Number(row.quantity) === 0
            "
            title="仅零库存记录可删除，确认移入回收站？"
            ok-text="删除"
            cancel-text="取消"
            @confirm="() => handleRecycleDelete(row)"
          >
            <Button type="link" danger size="small" :icon="h(LucideTrash2)" />
          </Popconfirm>
        </template>
      </template>
    </Grid>

    <!-- 仓库选择弹窗 -->
    <WarehouseSelectModal
      :visible="warehouseSelectVisible"
      @update:visible="(val) => (warehouseSelectVisible = val)"
      @select="onWarehouseSelected"
    />

    <!-- 库存流水抽屉 -->
    <StockLogDrawer
      :visible="stockLogVisible"
      :product-id="stockLogProductId"
      :product-name="stockLogProductName"
      @update:visible="(val) => (stockLogVisible = val)"
    />
  </Page>
</template>
