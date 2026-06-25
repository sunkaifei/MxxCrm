﻿﻿﻿﻿﻿﻿﻿﻿﻿﻿﻿﻿﻿﻿﻿<script lang="ts" setup>
import { h } from 'vue';
import { useRouter } from 'vue-router';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2, LucideImageOff } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, message, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deleteProductApi, getProductListApi } from '#/api';
import { $t } from '#/locales';

import ProductDrawer from './drawer.vue';

const accessStore = useAccessStore();
const router = useRouter();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
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
  rowConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getProductListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.name,
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
      title: '商品图',
      field: 'coverImage',
      width: 70,
      slots: { default: 'productImage' },
    },
    {
      title: '产品名称',
      field: 'name',
      minWidth: 160,
      align: 'left',
    },
    {
      title: '产品编号',
      field: 'productNo',
      width: 140,
    },
    {
      title: 'SKU',
      field: 'sku',
      width: 120,
    },
    {
      title: '单位',
      field: 'unit',
      width: 80,
    },
    {
      title: '销售价',
      field: 'salePrice',
      width: 100,
    },
    {
      title: $t('ui.table.status'),
      field: 'isActive',
      slots: { default: 'status' },
      width: 80,
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 120,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: ProductDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({
    create,
    row,
  });
  drawerApi.open();
}

function handleSkuManage(row: any) {
  router.push(`/product/sku?productId=${row.id}`);
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteProductApi([row.id]);
    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function handleCreate() {
  openDrawer(true);
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.product.list.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:product:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.list.button.create') }}
        </Button>
      </template>

      <template #productImage="{ row }">
        <div v-if="row.coverImage || row.imageUrl" class="w-10 h-10 rounded-lg border border-gray-200 overflow-hidden flex-shrink-0">
          <img :src="row.coverImage || row.imageUrl" alt="产品主图" class="w-full h-full object-cover" />
        </div>
        <div v-else class="w-10 h-10 rounded-lg border border-gray-200 flex-shrink-0 flex items-center justify-center bg-gray-50">
          <LucideImageOff class="w-5 h-5 text-gray-400" />
        </div>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt) }}
      </template>

      <template #status="{ row }">
        <Tag :color="row.isActive ? 'green' : 'red'">{{ row.isActive ? '启用' : '停用' }}</Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:product:view')"
          type="link"
          @click="() => handleSkuManage(row)"
        >{{ $t('page.product.list.button.view') }}</Button>
        <Button
          v-if="accessStore.hasAccessCode('product:product:save')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.product.list.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('product:product:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
