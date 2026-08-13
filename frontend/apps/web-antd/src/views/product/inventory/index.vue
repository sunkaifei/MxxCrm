<script lang="ts" setup>
import { computed, h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideChevronDown, LucideChevronUp, LucideList } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Tag, Tooltip } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { getInventoryListApi } from '#/api';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import StockLogDrawer from '../components/StockLogDrawer.vue';
import WarehouseSelectModal from '../inventory-check/WarehouseSelectModal.vue';

const accessStore = useAccessStore();

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

// ============ 树形数据转换 ============
// 将扁平行转换为按 产品 → 规格(SKU) → 仓库 的三级树形结构
function buildTreeData(flatList: any[]): any[] {
  const productMap = new Map<number, any>();

  for (const row of flatList) {
    const pid = Number(row.productId);
    if (!productMap.has(pid)) {
      productMap.set(pid, {
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
      });
    }

    const product = productMap.get(pid)!;
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

  return Array.from(productMap.values());
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
        const res: any = await getInventoryListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          productName: formValues.productName,
          warehouseId: formValues.warehouseId,
        });
        const flatList = res?.items ?? res?.list ?? [];
        const total = res?.total ?? flatList.length;
        const treeData = buildTreeData(flatList);
        return { items: treeData, total };
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 50,
      align: 'center',
    },
    {
      title: '产品 / 仓库',
      field: 'productName',
      minWidth: 180,
      slots: { default: 'productName' },
    },
    {
      title: '产品编码',
      field: 'productCode',
      width: 120,
      slots: { default: 'productCode' },
    },
    {
      title: '库存数量',
      field: 'quantity',
      width: 110,
      align: 'right',
      slots: { default: 'quantity' },
    },
    {
      title: '可用数量',
      field: 'availableQuantity',
      width: 110,
      align: 'right',
      slots: { default: 'availableQuantity' },
    },
    {
      title: '预留数量',
      field: 'reservedQuantity',
      width: 100,
      align: 'right',
      slots: { default: 'reservedQuantity' },
    },
    {
      title: '冻结数量',
      field: 'frozenQuantity',
      width: 100,
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
      title: '最后入库',
      field: 'lastInboundTime',
      width: 150,
      slots: { default: 'lastInboundTime' },
    },
    {
      title: '最后出库',
      field: 'lastOutboundTime',
      width: 150,
      slots: { default: 'lastOutboundTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 80,
      align: 'center',
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function handleViewLog(row: any) {
  stockLogVisible.value = true;
  stockLogProductId.value = Number(row.productId);
  stockLogProductName.value = row.productName || '';
}

// ============ 库存流水抽屉 ============
const stockLogVisible = ref(false);
const stockLogProductId = ref<number | null>(null);
const stockLogProductName = ref('');

function formatNumber(val: any): string {
  const n = Number(val ?? 0);
  return n.toLocaleString('zh-CN');
}

function formatMoney(val: any): string {
  const n = Number(val ?? 0);
  return n.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="stock" />
    <Grid :table-title="$t('page.product.inventory.title')">
      <template #toolbar-tools>
        <Button class="mr-2" size="small" @click="toggleExpandAll">
          <template #icon>
            <component :is="allExpanded ? h(LucideChevronUp) : h(LucideChevronDown)" />
          </template>
          {{ allExpanded ? '全部折叠' : '全部展开' }}
        </Button>
      </template>

      <!-- 第一列：产品 / 规格 / 仓库 -->
      <template #productName="{ row }">
        <template v-if="row._isProduct">
          <span class="font-semibold text-foreground">{{ row.productName }}</span>
          <Tag class="ml-2" color="blue" :bordered="false">{{ row._specCount }}种规格</Tag>
          <Tag color="geekblue" :bordered="false">{{ row._warehouseCount }}个仓库</Tag>
        </template>
        <template v-else-if="row._isSpec">
          <span class="pl-2 inline-flex items-center gap-1">
            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" class="text-muted-foreground">
              <rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" />
              <rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" />
            </svg>
            <span class="text-foreground font-medium">{{ row.specText }}</span>
            <span v-if="row.skuCode" class="text-xs text-muted-foreground font-mono">{{ row.skuCode }}</span>
          </span>
        </template>
        <template v-else>
          <span class="text-muted-foreground pl-4">
            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" class="inline-block -mt-0.5">
              <path d="M3 21V8l9-5 9 5v13" /><path d="M3 21h18" />
            </svg>
            {{ row.warehouseName }}
          </span>
        </template>
      </template>

      <!-- 产品编码列：仅产品行显示 -->
      <template #productCode="{ row }">
        <span v-if="row._isProduct" class="font-mono text-xs text-muted-foreground">{{ row.productCode || '-' }}</span>
        <span v-else class="text-muted-foreground">—</span>
      </template>

      <!-- 库存数量列 -->
      <template #quantity="{ row }">
        <template v-if="row._isProduct || row._isSpec">
          <span class="font-semibold" :class="row._isProduct ? 'text-base' : ''">{{ formatNumber(row.quantity) }}</span>
        </template>
        <template v-else>
          <span :class="{ 'text-orange-500 font-medium': Number(row.quantity) <= 0 }">
            {{ formatNumber(row.quantity) }}
          </span>
        </template>
      </template>

      <!-- 可用数量列 -->
      <template #availableQuantity="{ row }">
        <span :class="(row._isProduct || row._isSpec) ? 'font-semibold' : ''">
          {{ formatNumber(row.availableQuantity) }}
        </span>
      </template>

      <!-- 预留数量列 -->
      <template #reservedQuantity="{ row }">
        <span v-if="Number(row.reservedQuantity) > 0" class="text-orange-500">
          {{ formatNumber(row.reservedQuantity) }}
        </span>
        <span v-else class="text-muted-foreground">0</span>
      </template>

      <!-- 冻结数量列 -->
      <template #frozenQuantity="{ row }">
        <Tag v-if="row.frozenQuantity && Number(row.frozenQuantity) > 0" color="red" :bordered="false">
          {{ formatNumber(row.frozenQuantity) }}
        </Tag>
        <span v-else-if="row._isWarehouse" class="text-muted-foreground">0</span>
        <span v-else class="text-muted-foreground">0</span>
      </template>

      <!-- 库存总成本列 -->
      <template #totalCost="{ row }">
        <span :class="row._isProduct ? 'font-semibold text-primary' : (row._isSpec ? 'font-medium' : 'text-muted-foreground')">
          ¥{{ formatMoney(row.totalCost) }}
        </span>
      </template>

      <!-- 时间列：仅仓库行显示具体时间 -->
      <template #lastInboundTime="{ row }">
        <span v-if="row._isWarehouse" class="text-xs text-muted-foreground">{{ row.lastInboundTime || '-' }}</span>
        <span v-else class="text-muted-foreground">—</span>
      </template>

      <template #lastOutboundTime="{ row }">
        <span v-if="row._isWarehouse" class="text-xs text-muted-foreground">{{ row.lastOutboundTime || '-' }}</span>
        <span v-else class="text-muted-foreground">—</span>
      </template>

      <!-- 操作列 -->
      <template #action="{ row }">
        <Tooltip v-if="row._isProduct" :title="$t('page.inventory.tooltip.viewStockLog')">
          <Button
            v-if="accessStore.hasAccessCode('product:inventory:view')"
            type="link"
            :icon="h(LucideList)"
            @click="() => handleViewLog(row)"
          />
        </Tooltip>
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
