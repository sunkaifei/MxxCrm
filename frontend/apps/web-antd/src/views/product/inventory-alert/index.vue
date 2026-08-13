<script lang="ts" setup>
import { h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import { Button, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { getAlertListApi } from '#/api/core/product/alert';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import ProductDetailDrawer from '../components/ProductDetailDrawer.vue';
import WarehouseDetailDrawer from '../components/WarehouseDetailDrawer.vue';
import WarehouseSelectModal from '../inventory-check/WarehouseSelectModal.vue';
import RuleDrawer from './rule-drawer.vue';

const accessStore = useAccessStore();

// 预警类型选项
const alertTypeOptions = [
  { label: $t('page.product.inventory.alert.type.lowStock'), value: 'low_stock' },
  { label: $t('page.product.inventory.alert.type.highStock'), value: 'high_stock' },
  { label: $t('page.product.inventory.alert.type.stale'), value: 'stale' },
];

// ============ 仓库弹窗选择 ============
const warehouseSelectVisible = ref(false);

function openWarehouseSelect() {
  warehouseSelectVisible.value = true;
}

function onWarehouseSelected(warehouse: any) {
  gridApi.formApi?.setValues({
    warehouseId: String(warehouse.id),
    warehouseDisplay: warehouse.warehouseName ?? warehouse.name ?? '',
  });
}

function clearWarehouse() {
  gridApi.formApi?.setValues({ warehouseId: '', warehouseDisplay: '' });
}

// ============ 规则管理抽屉 ============
const ruleDrawerVisible = ref(false);

function openRuleDrawer() {
  ruleDrawerVisible.value = true;
}

// ============ 产品/仓库详情抽屉 ============
const productDetailVisible = ref(false);
const productDetailId = ref<number | null>(null);
const warehouseDetailVisible = ref(false);
const warehouseDetailId = ref<number | null>(null);

function openProductDetail(row: any) {
  if (!row.productId) return;
  productDetailId.value = Number(row.productId);
  productDetailVisible.value = true;
}

function openWarehouseDetail(row: any) {
  if (!row.warehouseId) return;
  warehouseDetailId.value = Number(row.warehouseId);
  warehouseDetailVisible.value = true;
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'productName',
      label: $t('page.product.inventory.alert.field.productName'),
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
        placeholder: '全部仓库（点击选择）',
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
    {
      component: 'Select',
      fieldName: 'alertType',
      label: $t('page.product.inventory.alert.field.alertType'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        options: alertTypeOptions,
        allowClear: true,
      },
    },
  ],
};

// 预警类型标签映射
function getAlertTypeTag(type: string) {
  const map: Record<string, { label: string; color: string }> = {
    low_stock: { label: $t('page.product.inventory.alert.type.lowStock'), color: 'error' },
    high_stock: { label: $t('page.product.inventory.alert.type.highStock'), color: 'warning' },
    stale: { label: $t('page.product.inventory.alert.type.stale'), color: 'geekblue' },
  };
  return map[type] || { label: $t('ui.unknown'), color: 'default' };
}

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

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getAlertListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          productName: formValues.productName,
          warehouseId: formValues.warehouseId || undefined,
          alertType: formValues.alertType,
        });
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inventory.alert.field.productName'), field: 'productName', minWidth: 140, slots: { default: 'productName' } },
    { title: $t('page.product.inventory.alert.field.warehouseName'), field: 'warehouseName', width: 120, slots: { default: 'warehouseName' } },
    { title: $t('page.product.inventory.alert.field.currentQuantity'), field: 'quantity', width: 110 },
    { title: $t('page.product.inventory.alert.field.minQuantity'), field: 'alertMinQuantity', width: 110 },
    { title: $t('page.product.inventory.alert.field.maxQuantity'), field: 'alertMaxQuantity', width: 110 },
    { title: $t('page.product.inventory.alert.field.staleDays'), field: 'obsoleteDays', width: 100 },
    { title: $t('page.product.inventory.alert.field.alertType'), field: 'alertType', width: 110, slots: { default: 'alertType' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 100 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="alert" />
    <Grid :table-title="$t('page.product.inventory.alert.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:alert:list')"
          type="primary"
          class="mr-2"
          @click="openRuleDrawer"
        >
          {{ $t('page.product.inventory.alert.action.viewRule') }}
        </Button>
      </template>

      <template #productName="{ row }">
        <a class="text-primary hover:underline cursor-pointer" @click="openProductDetail(row)">
          {{ row.productName || '-' }}
        </a>
      </template>

      <template #warehouseName="{ row }">
        <a class="text-primary hover:underline cursor-pointer" @click="openWarehouseDetail(row)">
          {{ row.warehouseName || '-' }}
        </a>
      </template>

      <template #alertType="{ row }">
        <Tag :color="getAlertTypeTag(row.alertType).color">
          {{ getAlertTypeTag(row.alertType).label }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:alert:list')"
          type="link"
          @click="openRuleDrawer"
        >
          {{ $t('page.product.inventory.alert.action.viewRule') }}
        </Button>
      </template>
    </Grid>

    <!-- 仓库选择弹窗 -->
    <WarehouseSelectModal
      :visible="warehouseSelectVisible"
      @update:visible="(val) => (warehouseSelectVisible = val)"
      @select="onWarehouseSelected"
    />

    <!-- 规则管理抽屉 -->
    <RuleDrawer
      :visible="ruleDrawerVisible"
      @update:visible="(val) => (ruleDrawerVisible = val)"
    />

    <!-- 产品详情抽屉 -->
    <ProductDetailDrawer
      :visible="productDetailVisible"
      :product-id="productDetailId"
      @update:visible="(val) => (productDetailVisible = val)"
    />

    <!-- 仓库详情抽屉 -->
    <WarehouseDetailDrawer
      :visible="warehouseDetailVisible"
      :warehouse-id="warehouseDetailId"
      @update:visible="(val) => (warehouseDetailVisible = val)"
    />
  </Page>
</template>
