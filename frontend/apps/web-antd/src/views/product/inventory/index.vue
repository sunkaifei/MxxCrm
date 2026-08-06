<script lang="ts" setup>
import { h } from 'vue';

import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideList } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Tag, Tooltip } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { getInventoryListApi } from '#/api';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';

const accessStore = useAccessStore();

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
      fieldName: 'warehouseId',
      label: $t('page.inventory.form.warehouseId'),
      componentProps: {
        placeholder: $t('page.inventory.form.warehouseIdPlaceholder'),
        allowClear: true,
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
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getInventoryListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          productName: formValues.productName,
          warehouseId: formValues.warehouseId,
        });
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 60,
    },
    {
      title: $t('page.inventory.report.field.productName'),
      field: 'productName',
      minWidth: 140,
    },
    {
      title: $t('page.product.list.productCode'),
      field: 'productCode',
      width: 120,
    },
    {
      title: $t('page.inventory.report.field.warehouseName'),
      field: 'warehouseName',
      width: 120,
    },
    {
      title: $t('page.inventory.report.field.currentQty'),
      field: 'quantity',
      width: 100,
    },
    {
      title: $t('page.inventory.report.field.availableQty'),
      field: 'availableQuantity',
      width: 100,
    },
    {
      title: $t('page.inventory.report.field.reservedQty'),
      field: 'reservedQuantity',
      width: 100,
    },
    {
      title: $t('page.inventory.report.field.inTransitQty'),
      field: 'inTransitQuantity',
      width: 100,
    },
    {
      title: $t('page.inventory.report.field.frozenQty'),
      field: 'frozenQuantity',
      width: 100,
      slots: { default: 'frozenQuantity' },
    },
    {
      title: $t('page.inventory.report.field.avgCost'),
      field: 'avgCost',
      width: 110,
    },
    {
      title: $t('page.inventory.report.field.totalCost'),
      field: 'totalCost',
      width: 120,
    },
    {
      title: $t('page.inventory.report.field.lastMovement'),
      field: 'lastInboundTime',
      width: 150,
    },
    {
      title: $t('page.inventory.report.field.lastMovement'),
      field: 'lastOutboundTime',
      width: 150,
    },
    {
      title: $t('ui.table.updateTime'),
      field: 'updateTime',
      width: 150,
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 100,
    },
  ],
};

const [Grid] = useVbenVxeGrid({ gridOptions, formOptions });

function handleViewLog(row: any) {
  // 跳转到库存流水页面（按产品筛选）
  const _route = {
    name: 'StockLog',
    query: { productId: row.productId, productName: row.productName },
  };
  // 使用 router.push 跳转到库存流水页面（按产品筛选）
  // 实际项目中可使用 router.push({ name: 'StockLog', query: { productId: row.productId } })
  window.$message.info($t('page.inventory.message.viewStockLog', { name: row.productName }));
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="stock" />
    <Grid :table-title="$t('page.product.inventory.title')">
      <template #frozenQuantity="{ row }">
        <Tag v-if="row.frozenQuantity && Number(row.frozenQuantity) > 0" color="red">
          {{ row.frozenQuantity }}
        </Tag>
        <span v-else>{{ row.frozenQuantity ?? '0' }}</span>
      </template>

      <template #action="{ row }">
        <Tooltip :title="$t('page.inventory.tooltip.viewStockLog')">
          <Button
            v-if="accessStore.hasAccessCode('product:inventory:view')"
            type="link"
            :icon="h(LucideList)"
            @click="() => handleViewLog(row)"
          />
        </Tooltip>
      </template>
    </Grid>
  </Page>
</template>
