<script lang="ts" setup>
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import { Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { getStockLogListApi } from '#/api/core/product/stock-log';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';

const accessStore = useAccessStore();

// 变动类型选项
const changeTypeOptions = [
  { label: $t('page.inventory.inbound.title'), value: 'inbound' },
  { label: $t('page.inventory.outbound.title'), value: 'outbound' },
  { label: $t('page.inventory.changeType.transferOut'), value: 'transfer_out' },
  { label: $t('page.inventory.changeType.transferIn'), value: 'transfer_in' },
  { label: $t('page.inventory.changeType.check'), value: 'check' },
  { label: $t('page.inventory.changeType.freeze'), value: 'freeze' },
  { label: $t('page.inventory.changeType.unfreeze'), value: 'unfreeze' },
  { label: $t('page.inventory.changeType.setup'), value: 'setup' },
  { label: $t('page.inventory.changeType.adjust'), value: 'adjust' },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'productId',
      label: $t('page.inventory.form.productId'),
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
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'changeType',
      label: $t('page.inventory.stockLog.title'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        options: changeTypeOptions,
        allowClear: true,
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'dateRange',
      label: $t('page.inventory.report.field.dateRange'),
      componentProps: {
        placeholder: [$t('ui.placeholder.startDate'), $t('ui.placeholder.endDate')],
        allowClear: true,
        valueFormat: 'YYYY-MM-DD',
        range: true,
      },
    },
  ],
};

// 变动类型标签
function getChangeTypeTag(type: string) {
  const map: Record<string, { label: string; color: string }> = {
    inbound: { label: $t('page.inventory.inbound.title'), color: 'green' },
    outbound: { label: $t('page.inventory.outbound.title'), color: 'red' },
    transfer_in: { label: $t('page.inventory.changeType.transferIn'), color: 'blue' },
    transfer_out: { label: $t('page.inventory.changeType.transferOut'), color: 'orange' },
    check: { label: $t('page.inventory.changeType.check'), color: 'purple' },
    freeze: { label: $t('page.inventory.changeType.freeze'), color: 'geekblue' },
    unfreeze: { label: $t('page.inventory.changeType.unfreeze'), color: 'cyan' },
    setup: { label: $t('page.inventory.changeType.setup'), color: 'default' },
    adjust: { label: $t('page.inventory.changeType.adjust'), color: 'warning' },
  };
  return map[type] || { label: type, color: 'default' };
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
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const result = await getStockLogListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          productId: formValues.productId,
          warehouseId: formValues.warehouseId,
          changeType: formValues.changeType,
          startTime: formValues.dateRange?.[0],
          endTime: formValues.dateRange?.[1],
        });
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '200px' : '';
        }
        return result;
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.inventory.column.productId'), field: 'productId', width: 100 },
    { title: $t('page.inventory.column.warehouseId'), field: 'warehouseId', width: 100 },
    {
      title: $t('page.inventory.column.changeType'),
      field: 'changeType',
      width: 120,
      slots: { default: 'changeType' },
    },
    { title: $t('page.inventory.column.quantityBefore'), field: 'quantityBefore', width: 120 },
    { title: $t('page.inventory.column.changeQuantity'), field: 'changeQuantity', width: 120 },
    { title: $t('page.inventory.column.quantityAfter'), field: 'quantityAfter', width: 120 },
    { title: $t('page.inventory.column.bizNo'), field: 'bizNo', width: 160 },
    { title: $t('page.inventory.column.operator'), field: 'operatorId', width: 100 },
    { title: $t('page.inventory.report.field.dateRange'), field: 'createTime', width: 170 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page>
    <InventoryProcessGuide current-step="stock" />
    <Grid
      :table-title="$t('page.inventory.stockLog.title')"
    >
      <template #changeType="{ row }">
        <Tag :color="getChangeTypeTag(row.changeType).color">
          {{ getChangeTypeTag(row.changeType).label }}
        </Tag>
      </template>
    </Grid>
  </Page>
</template>