<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Descriptions,
  DescriptionsItem,
  Drawer,
  Popconfirm,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteBrandApi, getBrandInfoApi, getBrandListApi } from '#/api';
import { $t } from '#/locales';

import BrandDrawer from './drawer.vue';

const accessStore = useAccessStore();

const statusLabelMap: Record<number, string> = {
  0: $t('page.product.brand.status.normal'),
  1: $t('page.product.brand.status.disabled'),
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
      fieldName: 'keyword',
      label: $t('page.product.brand.placeholder.keyword'),
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
          keyword: formValues.keyword,
        });
      },
    },
  },

  columns: [
    { type: 'seq', title: $t('ui.table.seq'), width: 60 },
    {
      title: $t('page.product.brand.field.name'),
      field: 'name',
      minWidth: 120,
      slots: { default: 'name' },
    },
    {
      title: $t('page.product.brand.field.nameEn'),
      field: 'nameEn',
      minWidth: 120,
    },
    {
      title: $t('page.product.brand.field.logo'),
      field: 'logo',
      width: 90,
      slots: { default: 'logo' },
    },
    {
      title: $t('page.product.brand.field.country'),
      field: 'country',
      minWidth: 100,
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      width: 80,
      slots: { default: 'status' },
    },
    {
      title: $t('page.product.brand.field.sort'),
      field: 'sortOrder',
      width: 70,
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      minWidth: 150,
      slots: { default: 'createTime' },
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

function handleCreate() {
  openDrawer(true);
}
function handleEdit(row: any) {
  openDrawer(false, row);
}

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

// ===== 详情抽屉 =====
const detailVisible = ref(false);
const detailLoading = ref(false);
const detailData = ref<any>({});

async function openDetail(row: any) {
  detailVisible.value = true;
  detailLoading.value = true;
  detailData.value = {};
  try {
    const resp = await getBrandInfoApi(row.id);
    detailData.value = resp?.data ?? resp ?? {};
  } catch {
    detailData.value = { ...row };
  } finally {
    detailLoading.value = false;
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

      <template #name="{ row }">
        <Button type="link" class="!px-0" @click="openDetail(row)">
          {{ row.name }}
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
          v-if="accessStore.hasAccessCode('product:brand:update')"
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

    <!-- 品牌详情抽屉 -->
    <Drawer
      :open="detailVisible"
      :width="520"
      :title="`${$t('page.product.brand.detail')} - ${detailData.name || ''}`"
      @close="detailVisible = false"
    >
      <div v-if="detailLoading" class="flex justify-center py-16">
        <span class="text-gray-400">{{
          $t('page.product.brand.loading')
        }}</span>
      </div>
      <Descriptions v-else :column="1" bordered size="small">
        <DescriptionsItem :label="$t('page.product.brand.field.name')">
          {{ detailData.name }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.product.brand.field.nameEn')">
          {{ detailData.nameEn || '-' }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.product.brand.field.logo')">
          <img
            v-if="detailData.logo"
            :src="detailData.logo"
            alt=""
            class="w-16 h-16 rounded object-cover border border-gray-100"
          />
          <span v-else class="text-gray-300">-</span>
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.product.brand.field.country')">
          {{ detailData.country || '-' }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.product.brand.field.website')">
          <a
            v-if="detailData.website"
            :href="detailData.website"
            target="_blank"
            class="text-blue-500 hover:underline"
          >
            {{ detailData.website }}
          </a>
          <span v-else class="text-gray-300">-</span>
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.product.brand.field.status')">
          <Tag :color="statusColorMap[detailData.status]">
            {{ statusLabelMap[detailData.status] || detailData.status }}
          </Tag>
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.product.brand.field.description')">
          {{ detailData.description || '-' }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.product.brand.field.createTime')">
          {{ formatDateTime(detailData.createTime) || '-' }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.product.brand.field.updateTime')">
          {{ formatDateTime(detailData.updateTime) || '-' }}
        </DescriptionsItem>
      </Descriptions>
    </Drawer>
  </Page>
</template>
