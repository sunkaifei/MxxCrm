<script lang="ts" setup>
import { h } from 'vue';

import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import { Button, Tag, Tooltip } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { getAlertListApi } from '#/api/core/product/alert';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';

const accessStore = useAccessStore();

// 预警类型选项
const alertTypeOptions = [
  { label: $t('page.product.inventory.alert.type.lowStock'), value: 'low_stock' },
  { label: $t('page.product.inventory.alert.type.highStock'), value: 'high_stock' },
  { label: $t('page.product.inventory.alert.type.stale'), value: 'stale' },
];

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
          alertType: formValues.alertType,
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
      title: $t('page.product.inventory.alert.field.productName'),
      field: 'productName',
      minWidth: 140,
    },
    {
      title: $t('page.product.inventory.alert.field.warehouseName'),
      field: 'warehouseName',
      width: 120,
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
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 100,
    },
  ],
};

const [Grid] = useVbenVxeGrid({ gridOptions, formOptions });

function handleViewRule(row: any) {
  window.$message.info($t('page.product.inventory.alert.action.viewRule') + ' #' + row.productName);
}

function handleToRule() {
  window.$message.info($t('page.product.inventory.alert.action.viewRule'));
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="alert" />
    <Grid :table-title="$t('page.product.inventory.alert.title')">
      <template #alertType="{ row }">
        <Tag :color="getAlertTypeTag(row.alertType).color">
          {{ getAlertTypeTag(row.alertType).label }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Tooltip :title="$t('page.product.inventory.alert.action.viewRule')">
          <Button
            v-if="accessStore.hasAccessCode('product:alert:list')"
            type="link"
            @click="() => handleViewRule(row)"
          >
            {{ $t('page.product.inventory.alert.action.viewRule') }}
          </Button>
        </Tooltip>
      </template>
    </Grid>
  </Page>
</template>