<script lang="ts" setup>
import { Page } from '@vben/common-ui';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { getOrderItemListApi } from '#/api';
import { $t } from '#/locales';

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
  rowConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getOrderItemListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
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
      title: 'SKU',
      field: 'sku',
    },
    {
      title: '数量',
      field: 'quantity',
    },
    {
      title: '单价',
      field: 'unitPrice',
    },
    {
      title: '小计',
      field: 'subtotal',
    },
  ],
};

const [Grid] = useVbenVxeGrid({ gridOptions });
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.sale.orderItem.title')" />
  </Page>
</template>
