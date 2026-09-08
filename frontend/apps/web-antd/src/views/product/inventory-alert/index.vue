<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideTrash2, LucideUndo2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { useRouter } from 'vue-router';

import { Button, Popconfirm, Tabs, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getAlertListApi } from '#/api/core/product/alert';
import {
  purgeInventoryApi,
  restoreInventoryApi,
} from '#/api/core/product/inventory';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import ProductDetailDrawer from '../components/ProductDetailDrawer.vue';
import WarehouseDetailDrawer from '../components/WarehouseDetailDrawer.vue';
import WarehouseSelectModal from '../inventory-check/WarehouseSelectModal.vue';

const router = useRouter();
const accessStore = useAccessStore();

// ============ 范围视图（全部库存预警/我的库存预警/回收站） ============
const activeTab = ref('all');
const crossViewEnabled = ref(false);
const isManagement = ref(false);
const isRecycle = computed(() => activeTab.value === 'recycle');
const tabList = [
  { key: 'all', label: $t('page.product.inventory.alert.tab.all') },
  { key: 'mine', label: $t('page.product.inventory.alert.tab.mine') },
  { key: 'recycle', label: $t('page.product.inventory.alert.tab.recycle') },
];

async function handleTabChange() {
  gridApi.setGridOptions({ columns: buildColumns() });
  gridApi.query();
}

async function handleRestore(row: any) {
  row.pending = true;
  try {
    await restoreInventoryApi(Number(row.id));
    window.$message.success($t('page.product.inventory.alert.action.restore'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handlePurge(row: any) {
  row.pending = true;
  try {
    await purgeInventoryApi(Number(row.id));
    window.$message.success($t('page.product.inventory.alert.action.purge'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// 预警类型选项
const alertTypeOptions = [
  {
    label: $t('page.product.inventory.alert.type.lowStock'),
    value: 'low_stock',
  },
  {
    label: $t('page.product.inventory.alert.type.highStock'),
    value: 'high_stock',
  },
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

// ============ 产品/仓库详情抽屉 ============
const productDetailVisible = ref(false);
const productDetailId = ref<null | number>(null);
const warehouseDetailVisible = ref(false);
const warehouseDetailId = ref<null | number>(null);

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
  const map: Record<string, { color: string; label: string }> = {
    low_stock: {
      label: $t('page.product.inventory.alert.type.lowStock'),
      color: 'error',
    },
    high_stock: {
      label: $t('page.product.inventory.alert.type.highStock'),
      color: 'warning',
    },
    stale: {
      label: $t('page.product.inventory.alert.type.stale'),
      color: 'geekblue',
    },
  };
  return map[type] || { label: $t('ui.unknown'), color: 'default' };
}

function buildColumns(): VxeGridProps['columns'] {
  const recycleMode = activeTab.value === 'recycle';
  return [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: $t('page.product.inventory.alert.field.productName'),
      field: 'productName',
      minWidth: 140,
      slots: { default: 'productName' },
    },
    {
      title: $t('page.product.inventory.alert.field.specText'),
      field: 'specText',
      minWidth: 120,
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: $t('page.product.inventory.alert.field.warehouseName'),
      field: 'warehouseName',
      width: 120,
      slots: { default: 'warehouseName' },
    },
    {
      title: $t('page.product.inventory.alert.field.currentQuantity'),
      field: 'quantity',
      width: 110,
    },
    {
      title: $t('page.product.inventory.alert.field.minQuantity'),
      field: 'alertMinQuantity',
      width: 110,
    },
    {
      title: $t('page.product.inventory.alert.field.maxQuantity'),
      field: 'alertMaxQuantity',
      width: 110,
    },
    {
      title: $t('page.product.inventory.alert.field.staleDays'),
      field: 'obsoleteDays',
      width: 100,
    },
    {
      title: $t('page.product.inventory.alert.field.alertType'),
      field: 'alertType',
      width: 110,
      slots: { default: 'alertType' },
    },
    {
      title: $t('page.product.inventory.alert.field.diff'),
      field: 'alertDiff',
      minWidth: 130,
      slots: { default: 'alertDiff' },
    },
    ...(recycleMode
      ? [
          {
            title: $t('page.product.inventory.alert.field.deleteByName'),
            field: 'deleteByName',
            width: 100,
          },
          {
            title: $t('page.product.inventory.alert.field.deleteTime'),
            field: 'deleteTime',
            width: 160,
          },
        ]
      : []),
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: recycleMode ? 120 : 100,
    },
  ];
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
        const load = async (scope: string) => {
          const r: any = await getAlertListApi({
            page: page.currentPage,
            pageSize: page.pageSize,
            productName: formValues.productName,
            warehouseId: formValues.warehouseId || undefined,
            alertType: formValues.alertType,
            scope,
          });
          crossViewEnabled.value = Boolean(r?.crossViewEnabled);
          isManagement.value = Boolean(r?.isManagement);
          return r;
        };
        let res = await load(activeTab.value);
        // 普通用户（无管理/互看权限）默认落在"全部库存预警"时，自动切到"我的库存预警"并加载其数据
        if (
          activeTab.value === 'all' &&
          !isManagement.value &&
          !crossViewEnabled.value
        ) {
          activeTab.value = 'mine';
          res = await load('mine');
        }
        return res;
      },
    },
  },

  columns: buildColumns(),
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="alert" />
    <Grid :table-title="$t('page.product.inventory.alert.title')">
      <template #form-header>
        <Tabs
          v-model:active-key="activeTab"
          class="mb-3"
          @change="handleTabChange"
        >
          <Tabs.TabPane
            v-for="tab in tabList.filter(
              (t) => t.key !== 'all' || isManagement || crossViewEnabled,
            )"
            :key="tab.key"
            :tab="tab.label"
          />
        </Tabs>
      </template>

      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:alert:update')"
          type="primary"
          class="mr-2"
          @click="router.push('/alert-rule?create=1')"
        >
          {{ $t('page.product.inventory.alert.action.create') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('product:alert:list')"
          class="mr-2"
          @click="router.push('/alert-rule')"
        >
          {{ $t('page.product.inventory.alert.action.viewRule') }}
        </Button>
      </template>

      <template #productName="{ row }">
        <a
          class="text-primary hover:underline cursor-pointer"
          @click="openProductDetail(row)"
        >
          {{ row.productName || '-' }}
        </a>
      </template>

      <template #warehouseName="{ row }">
        <a
          class="text-primary hover:underline cursor-pointer"
          @click="openWarehouseDetail(row)"
        >
          {{ row.warehouseName || '-' }}
        </a>
      </template>

      <template #alertType="{ row }">
        <Tag :color="getAlertTypeTag(row.alertType).color">
          {{ getAlertTypeTag(row.alertType).label }}
        </Tag>
      </template>

      <template #alertDiff="{ row }">
        <span v-if="row.alertType === 'low_stock'">
          缺 {{ Math.max(0, (row.alertMinQuantity ?? 0) - (row.quantity ?? 0)) }}
          件
        </span>
        <span v-else-if="row.alertType === 'high_stock'">
          超 {{ Math.max(0, (row.quantity ?? 0) - (row.alertMaxQuantity ?? 0)) }}
          件
        </span>
        <span v-else-if="row.alertType === 'stale'">
          {{ row.obsoleteDays ?? 0 }} 天未出入库
        </span>
        <span v-else>-</span>
      </template>

      <template #action="{ row }">
        <template v-if="isRecycle">
          <Popconfirm
            :title="$t('page.product.inventory.alert.action.restoreConfirm')"
            :ok-text="$t('page.product.inventory.alert.action.restore')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleRestore(row)"
          >
            <Button type="link" size="small" :icon="h(LucideUndo2)" />
          </Popconfirm>
          <Popconfirm
            v-if="isManagement"
            :title="$t('page.product.inventory.alert.action.purgeConfirm')"
            :ok-text="$t('page.product.inventory.alert.action.purge')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handlePurge(row)"
          >
            <Button type="link" danger size="small" :icon="h(LucideTrash2)" />
          </Popconfirm>
        </template>
        <template v-else>
          <Button
            v-if="accessStore.hasAccessCode('product:alert:list')"
            type="link"
            @click="
              router.push({
                path: '/alert-rule',
                query: {
                  productId: row.productId,
                  productName: row.productName,
                  ...(row.warehouseId ? { warehouseId: row.warehouseId } : {}),
                  ...(row.warehouseName
                    ? { warehouseName: row.warehouseName }
                    : {}),
                },
              })
            "
          >
            {{ $t('page.product.inventory.alert.action.viewRule') }}
          </Button>
        </template>
      </template>
    </Grid>

    <!-- 仓库选择弹窗 -->
    <WarehouseSelectModal
      :visible="warehouseSelectVisible"
      @update:visible="(val) => (warehouseSelectVisible = val)"
      @select="onWarehouseSelected"
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
