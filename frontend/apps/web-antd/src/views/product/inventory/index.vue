<script lang="ts" setup>
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { getInventoryListApi } from '#/api';
import { $t } from '#/locales';

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'productName',
      label: '产品名称',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'sku',
      label: 'SKU',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
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
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getInventoryListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          productName: formValues.productName,
          sku: formValues.sku,
        });
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: '产品名称',
      field: 'productName',
    },
    {
      title: '产品编码',
      field: 'productCode',
    },
    {
      title: '仓库',
      field: 'warehouseName',
    },
    {
      title: '库存数量',
      field: 'quantity',
    },
    {
      title: '预留数量',
      field: 'reservedQuantity',
    },
    {
      title: '可用数量',
      field: 'availableQuantity',
    },
  ],
};

const [Grid] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.product.inventory.title')" />
  </Page>
</template>
