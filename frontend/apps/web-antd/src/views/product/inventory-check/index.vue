<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import {
  LucideCheckCircle,
  LucideFilePenLine,
  LucidePlay,
  LucideSearch,
  LucideSquare,
  LucideTrash2,
} from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  cancelCheckApi,
  completeCheckApi,
  deleteCheckApi,
  getCheckListApi,
  submitCheckApi,
} from '#/api/core/product/check';
import { getWarehouseListApi } from '#/api/core/product/warehouse';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import DetailDrawer from './detail-drawer.vue';
import CheckDrawer from './drawer.vue';
import InputDrawer from './input-drawer.vue';

const accessStore = useAccessStore();

const warehouseOptions = ref<{ label: string; value: number }[]>([]);

async function loadWarehouseOptions() {
  try {
    const resp: any = await getWarehouseListApi({ page: 1, pageSize: 999 });
    const list = resp?.data ?? resp ?? [];
    warehouseOptions.value = (Array.isArray(list) ? list : []).map(
      (w: any) => ({
        label: w.warehouseName ?? w.name ?? w.label,
        value: Number(w.id ?? w.value),
      }),
    );
  } catch (error) {
    console.error('[InventoryCheck] 加载仓库选项失败:', error);
  }
}

onMounted(() => {
  loadWarehouseOptions();
});

function getCheckTypeOptions() {
  return [
    { label: $t('page.product.inventory.check.type.1'), value: 1 },
    { label: $t('page.product.inventory.check.type.2'), value: 2 },
    { label: $t('page.product.inventory.check.type.3'), value: 3 },
  ];
}

function getStatusOptions() {
  return [
    { label: $t('page.product.inventory.check.status.0'), value: 0 },
    { label: $t('page.product.inventory.check.status.1'), value: 1 },
    { label: $t('page.product.inventory.check.status.2'), value: 2 },
    { label: $t('page.product.inventory.check.status.3'), value: 3 },
  ];
}

function getCheckTypeTag(type: number) {
  const map: Record<number, { color: string; label: string }> = {
    1: { label: $t('page.product.inventory.check.type.1'), color: 'blue' },
    2: { label: $t('page.product.inventory.check.type.2'), color: 'cyan' },
    3: { label: $t('page.product.inventory.check.type.3'), color: 'purple' },
  };
  return map[type] || { label: $t('ui.unknown'), color: 'default' };
}

function getStatusTag(status: number) {
  const map: Record<number, { color: string; label: string }> = {
    0: { label: $t('page.product.inventory.check.status.0'), color: 'default' },
    1: {
      label: $t('page.product.inventory.check.status.1'),
      color: 'processing',
    },
    2: { label: $t('page.product.inventory.check.status.2'), color: 'success' },
    3: { label: $t('page.product.inventory.check.status.3'), color: 'default' },
  };
  return map[status] || { label: $t('ui.unknown'), color: 'default' };
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'checkNo',
      label: $t('page.product.inventory.check.field.checkNo'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'checkType',
      label: $t('page.product.inventory.check.field.checkType'),
      componentProps: {
        placeholder: $t('ui.placeholder.all'),
        allowClear: true,
        options: getCheckTypeOptions(),
      },
    },
    {
      component: 'Select',
      fieldName: 'warehouseId',
      label: $t('page.product.inventory.check.field.warehouse'),
      componentProps: {
        placeholder: $t('ui.placeholder.all'),
        allowClear: true,
        options: warehouseOptions,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          (option?.label ?? '').toLowerCase().includes(input.toLowerCase()),
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.product.inventory.check.field.status'),
      componentProps: {
        placeholder: $t('ui.placeholder.all'),
        allowClear: true,
        options: getStatusOptions(),
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
        return await getCheckListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          checkNo: formValues.checkNo,
          checkType: formValues.checkType,
          warehouseId: formValues.warehouseId,
          status: formValues.status,
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
      title: $t('page.product.inventory.check.field.checkNo'),
      field: 'stocktakeNo',
      width: 160,
    },
    {
      title: $t('page.product.inventory.check.field.checkType'),
      field: 'stocktakeType',
      width: 110,
      slots: { default: 'checkType' },
    },
    {
      title: $t('page.product.inventory.check.field.warehouse'),
      field: 'warehouseName',
      minWidth: 140,
    },
    {
      title: $t('page.product.inventory.check.field.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('page.product.inventory.check.field.totalItems'),
      field: 'totalItems',
      width: 90,
      align: 'right',
    },
    {
      title: $t('page.product.inventory.check.field.surplusCount'),
      field: 'surplusCount',
      width: 90,
      align: 'right',
      slots: { default: 'surplusCount' },
    },
    {
      title: $t('page.product.inventory.check.field.shortageCount'),
      field: 'shortageCount',
      width: 90,
      align: 'right',
      slots: { default: 'shortageCount' },
    },
    {
      title: $t('page.product.inventory.check.field.checkBy'),
      field: 'createdByName',
      width: 100,
    },
    {
      title: $t('page.product.inventory.check.field.checkTime'),
      field: 'createTime',
      width: 160,
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 200,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 新增/编辑弹窗
const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: CheckDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

// 盘点录入弹窗
const [InputDrawerComp, inputDrawerApi] = useVbenDrawer({
  connectedComponent: InputDrawer,
  onClosed() {
    const data = inputDrawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

// 盘点详情弹窗
const [DetailDrawerComp, detailDrawerApi] = useVbenDrawer({
  connectedComponent: DetailDrawer,
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

function handleCreate() {
  openDrawer(true);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteCheckApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// 提交（草稿→盘点中）
async function handleSubmit(row: any) {
  row.pending = true;
  try {
    await submitCheckApi(row.id);
    window.$message.success(
      $t('page.product.inventory.check.action.submitSuccess'),
    );
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// 打开录入弹窗
function handleInput(row: any) {
  inputDrawerApi.setData({ row });
  inputDrawerApi.open();
}

// 完成盘点
async function handleComplete(row: any) {
  row.pending = true;
  try {
    await completeCheckApi(row.id);
    window.$message.success(
      $t('page.product.inventory.check.action.completeSuccess'),
    );
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// 取消盘点
async function handleCancel(row: any) {
  row.pending = true;
  try {
    await cancelCheckApi(row.id);
    window.$message.success(
      $t('page.product.inventory.check.action.cancelSuccess'),
    );
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// 查看详情
function handleDetail(row: any) {
  detailDrawerApi.setData({ row });
  detailDrawerApi.open();
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="check" />
    <Grid :table-title="$t('page.product.inventory.check.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:check:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.inventory.check.create') }}
        </Button>
      </template>

      <template #checkType="{ row }">
        <Tag
          :color="
            getCheckTypeTag(
              row.stocktakeType === 'full'
                ? 1
                : row.stocktakeType === 'dynamic'
                  ? 3
                  : 2,
            ).color
          "
        >
          {{
            getCheckTypeTag(
              row.stocktakeType === 'full'
                ? 1
                : row.stocktakeType === 'dynamic'
                  ? 3
                  : 2,
            ).label
          }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="getStatusTag(row.status).color">
          {{ getStatusTag(row.status).label }}
        </Tag>
      </template>

      <template #surplusCount="{ row }">
        <span
          :class="{ 'text-green-600 font-medium': (row.surplusCount ?? 0) > 0 }"
        >
          {{ row.surplusCount ?? 0 }}
        </span>
      </template>

      <template #shortageCount="{ row }">
        <span
          :class="{ 'text-red-600 font-medium': (row.shortageCount ?? 0) > 0 }"
        >
          {{ row.shortageCount ?? 0 }}
        </span>
      </template>

      <template #action="{ row }">
        <!-- 草稿(0)：编辑 / 提交 / 删除 -->
        <template v-if="row.status === 0">
          <Button
            v-if="accessStore.hasAccessCode('product:check:update')"
            type="link"
            size="small"
            :icon="h(LucideFilePenLine)"
            @click="() => handleEdit(row)"
          />
          <Popconfirm
            v-if="accessStore.hasAccessCode('product:check:audit')"
            :title="$t('page.product.inventory.check.action.submitConfirm')"
            @confirm="() => handleSubmit(row)"
          >
            <Button type="link" size="small" :icon="h(LucidePlay)" />
          </Popconfirm>
          <Popconfirm
            v-if="accessStore.hasAccessCode('product:check:delete')"
            :title="
              $t('ui.text.do_you_want_delete', {
                moduleName: $t('page.product.inventory.check.title'),
              })
            "
            @confirm="() => handleDelete(row)"
          >
            <Button type="link" size="small" danger :icon="h(LucideTrash2)" />
          </Popconfirm>
        </template>

        <!-- 盘点中(1)：录入实盘 / 完成 / 取消 -->
        <template v-else-if="row.status === 1">
          <Button
            v-if="accessStore.hasAccessCode('product:check:update')"
            type="link"
            size="small"
            :icon="h(LucideFilePenLine)"
            @click="() => handleInput(row)"
          >
            {{ $t('page.product.inventory.check.input') }}
          </Button>
          <Popconfirm
            v-if="accessStore.hasAccessCode('product:check:audit')"
            :title="$t('page.product.inventory.check.action.completeConfirm')"
            @confirm="() => handleComplete(row)"
          >
            <Button type="link" size="small" :icon="h(LucideCheckCircle)">
              {{ $t('page.product.inventory.check.complete') }}
            </Button>
          </Popconfirm>
          <Popconfirm
            v-if="accessStore.hasAccessCode('product:check:update')"
            :title="$t('page.product.inventory.check.action.cancelConfirm')"
            @confirm="() => handleCancel(row)"
          >
            <Button type="link" size="small" :icon="h(LucideSquare)" />
          </Popconfirm>
        </template>

        <!-- 已完成(2) / 已取消(3)：查看详情 -->
        <template v-else>
          <Button
            type="link"
            size="small"
            :icon="h(LucideSearch)"
            @click="() => handleDetail(row)"
          >
            {{ $t('page.product.inventory.check.viewDetail') }}
          </Button>
        </template>
      </template>
    </Grid>
    <Drawer />
    <InputDrawerComp />
    <DetailDrawerComp />
  </Page>
</template>
