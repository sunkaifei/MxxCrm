<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deleteBrandApi, getBrandListApi } from '#/api';
import { $t } from '#/locales';

import BrandDrawer from './drawer.vue';

const accessStore = useAccessStore();

const statusLabelMap: Record<number, string> = {
  0: '正常',
  1: '停用',
};

const statusColorMap: Record<number, string> = {
  0: 'green',
  1: 'red',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '品牌名称',
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
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getBrandListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
        });
      },
    },
  },

  columns: [
    { type: 'seq', title: $t('ui.table.seq'), width: 60 },
    { title: '品牌名称', field: 'name', width: 140 },
    { title: '品牌英文名', field: 'englishName', width: 140 },
    { title: 'Logo', field: 'logo', width: 90, slots: { default: 'logo' } },
    { title: '品牌原产国', field: 'originCountry', width: 120 },
    { title: $t('ui.table.status'), field: 'status', width: 80, slots: { default: 'status' } },
    { title: '排序', field: 'sortOrder', width: 70 },
    { title: $t('ui.table.createTime'), field: 'createTime', width: 160, slots: { default: 'createTime' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 120 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: BrandDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteBrandApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.product.brand.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:brand:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.brand.button.create') }}
        </Button>
      </template>

      <template #logo="{ row }">
        <img
          v-if="row.logo"
          :src="row.logo"
          alt=""
          class="w-10 h-10 rounded object-cover border border-gray-100"
        />
        <span v-else class="text-xs text-gray-300">-</span>
      </template>

      <template #status="{ row }">
        <Tag :color="statusColorMap[row.status]">
          {{ statusLabelMap[row.status] || row.status }}
        </Tag>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:brand:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: '品牌' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('product:brand:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />
  </Page>
</template>