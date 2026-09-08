<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideArrowRightLeft, LucideTrash2, LucideUndo2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';

import { Button, Popconfirm, Tag, Tooltip, Tabs } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getWarehouseListApi } from '#/api';
import {
  deleteTransferApi,
  getTransferListApi,
  restoreTransferApi,
  purgeTransferApi,
  transferInboundApi,
  transferOutboundApi,
  submitTransferApprovalApi,
} from '#/api/core/product/transfer';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import TransferDetail from './detail.vue';
import TransferDrawer from './drawer.vue';

const accessStore = useAccessStore();
const { isSuperAdmin } = useSuperAdminGuard();

// ===== 范围视图 =====
const activeTab = ref('all');
const tabList = [
  { key: 'all', label: '全部调拨' },
  { key: 'mine', label: '我的调拨' },
  { key: 'recycle', label: '回收站' },
];
const isRecycle = computed(() => activeTab.value === 'recycle');

async function handleTabChange() {
  refreshColumns();
  gridApi.query();
}

function refreshColumns() {
  const base = gridOptions.columns;
  if (!Array.isArray(base)) return;
  const hasDeleteTime = base.some((c: any) => c.field === 'deleteTime');
  if (isRecycle.value && !hasDeleteTime) {
    gridApi.setGridOptions({
      columns: [
        ...base,
        { title: '删除人', field: 'deleteByName', width: 100 },
        { title: '删除时间', field: 'deleteTime', width: 160 },
      ],
    } as any);
  } else if (!isRecycle.value && hasDeleteTime) {
    gridApi.setGridOptions({
      columns: base.filter((c: any) => c.field !== 'deleteTime' && c.field !== 'deleteByName'),
    } as any);
  }
}

async function handleRecycleRestore(row: any) {
  row.pending = true;
  try {
    await restoreTransferApi(Number(row.id));
    window.$message.success('已恢复');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleRecyclePurge(row: any) {
  row.pending = true;
  try {
    await purgeTransferApi(Number(row.id));
    window.$message.success('已彻底删除');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// ===== W11 单仓防线：启用仓库 <2 时禁止新建调拨 =====
const warehouseCount = ref<number>(-1);
const singleWarehouse = computed(() => warehouseCount.value >= 0 && warehouseCount.value < 2);

onMounted(async () => {
  try {
    const res: any = await getWarehouseListApi({ page: 1, pageSize: 1 });
    warehouseCount.value = Number(res?.total ?? 0);
  } catch {
    warehouseCount.value = -1;
  }
});

// 调拨状态选项
const statusOptions = [
  { label: $t('page.product.inventory.transfer.status.0'), value: 0 },
  { label: $t('page.product.inventory.transfer.status.1'), value: 1 },
  { label: $t('page.product.inventory.transfer.status.2'), value: 2 },
  { label: $t('page.product.inventory.transfer.status.3'), value: 3 },
  { label: $t('page.product.inventory.transfer.status.4'), value: 4 },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'transferNo',
      label: $t('page.product.inventory.transfer.field.transferNo'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.product.inventory.transfer.field.status'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        options: statusOptions,
        allowClear: true,
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'createTimeRange',
      label: $t('page.product.inventory.transfer.field.createTime'),
      componentProps: {
        placeholder: [
          $t('ui.placeholder.startDate'),
          $t('ui.placeholder.endDate'),
        ],
        allowClear: true,
        valueFormat: 'YYYY-MM-DD',
        range: true,
      },
    },
  ],
};

// 调拨状态标签映射
function getTransferStatusTag(status: number) {
  const map: Record<number, { color: string; label: string }> = {
    0: {
      label: $t('page.product.inventory.transfer.status.0'),
      color: 'default',
    },
    1: {
      label: $t('page.product.inventory.transfer.status.1'),
      color: 'processing',
    },
    2: {
      label: $t('page.product.inventory.transfer.status.2'),
      color: 'warning',
    },
    3: {
      label: $t('page.product.inventory.transfer.status.3'),
      color: 'success',
    },
    4: {
      label: $t('page.product.inventory.transfer.status.4'),
      color: 'error',
    },
  };
  return map[status] || { label: $t('ui.unknown'), color: 'default' };
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
        return await getTransferListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          transferNo: formValues.transferNo,
          status: formValues.status,
          scope: activeTab.value,
          createTimeStart: formValues.createTimeRange?.[0],
          createTimeEnd: formValues.createTimeRange?.[1],
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
      title: $t('page.product.inventory.transfer.field.transferNo'),
      field: 'transferNo',
      width: 140,
      slots: { default: 'transferNo' },
    },
    {
      title: $t('page.product.inventory.transfer.field.sourceWarehouse'),
      field: 'sourceWarehouse',
      minWidth: 120,
    },
    {
      title: $t('page.product.inventory.transfer.field.targetWarehouse'),
      field: 'targetWarehouse',
      minWidth: 120,
    },
    {
      title: $t('page.product.inventory.transfer.field.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('page.product.inventory.transfer.field.totalQuantity'),
      field: 'totalQuantity',
      width: 100,
    },
    {
      title: $t('page.product.inventory.transfer.field.createdBy'),
      field: 'createdByName',
      width: 100,
    },
    {
      title: $t('page.product.inventory.transfer.field.createTime'),
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

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: TransferDrawer,
  onOpenChange(isOpen: boolean) {
    if (!isOpen) {
      const data = drawerApi.getData();
      if (data?.needRefresh) {
        gridApi.query();
      }
    }
  },
});

// 点选方案之外的调拨详情抽屉：单号点击打开（信息+明细+审批流+审批操作）
const [DetailDrawer, detailDrawerApi] = useVbenDrawer({
  connectedComponent: TransferDetail,
  onOpenChange(isOpen: boolean) {
    if (!isOpen) {
      const data = detailDrawerApi.getData();
      if (data?.needRefresh) {
        gridApi.query();
      }
    }
  },
});

function openDetail(row: any) {
  detailDrawerApi.setData({ id: row.id });
  detailDrawerApi.open();
}

// 提交审批入口（草稿态，列表操作列）
async function handleSubmitApproval(row: any) {
  row.pending = true;
  try {
    await submitTransferApprovalApi(row.id);
    window.$message.success('已提交审批（仓管 → CEO 两级审批）');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({
    create,
    row,
  });
  drawerApi.open();
}

async function handleOutbound(row: any) {
  row.pending = true;
  try {
    await transferOutboundApi(row.id);
    window.$message.success(
      $t('page.product.inventory.transfer.action.outboundSuccess'),
    );
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleInbound(row: any) {
  row.pending = true;
  try {
    await transferInboundApi(row.id);
    window.$message.success(
      $t('page.product.inventory.transfer.action.inboundSuccess'),
    );
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteTransferApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="transfer" />
    <Grid :table-title="$t('page.product.inventory.transfer.title')">
      <template #form-header>
        <Tabs
          v-model:active-key="activeTab"
          class="mb-3"
          @change="handleTabChange"
        >
          <Tabs.TabPane
            v-for="tab in tabList"
            :key="tab.key"
            :tab="tab.label"
          />
        </Tabs>
      </template>
      <template #toolbar-actions>
        <Tooltip :title="singleWarehouse ? '当前仅有一个仓库，无仓可调' : ''">
          <Button
            v-if="accessStore.hasAccessCode('product:transfer:create')"
            type="primary"
            :disabled="singleWarehouse"
            @click="openDrawer(true)"
          >
            {{ $t('page.product.inventory.transfer.create') }}
          </Button>
        </Tooltip>
      </template>

      <template #transferNo="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:transfer:view')"
          type="link"
          class="px-0"
          @click="() => openDetail(row)"
        >
          {{ row.transferNo }}
        </Button>
        <span v-else>{{ row.transferNo }}</span>
      </template>

      <template #status="{ row }">
        <Tag :color="getTransferStatusTag(row.status).color">
          {{ getTransferStatusTag(row.status).label }}
        </Tag>
      </template>

      <template #action="{ row }">
        <template v-if="isRecycle">
          <Popconfirm
            title="确认恢复该单据？"
            ok-text="恢复"
            cancel-text="取消"
            @confirm="() => handleRecycleRestore(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('product:transfer:delete')"
              type="link"
              size="small"
              :icon="h(LucideUndo2)"
            />
          </Popconfirm>
          <Popconfirm
            v-if="isSuperAdmin"
            title="彻底删除后不可恢复，确认？"
            ok-text="彻底删除"
            cancel-text="取消"
            @confirm="() => handleRecyclePurge(row)"
          >
            <Button type="link" danger size="small" :icon="h(LucideTrash2)" />
          </Popconfirm>
        </template>
        <template v-else>
        <Button
          v-if="
            accessStore.hasAccessCode('product:transfer:create') &&
            (row.status === 0 || row.status === 6)
          "
          type="link"
          @click="() => handleSubmitApproval(row)"
        >
          提交审批
        </Button>
        <Button
          v-if="
            accessStore.hasAccessCode('product:transfer:outbound') &&
            row.status === 2
          "
          type="link"
          :icon="h(LucideArrowRightLeft)"
          @click="() => handleOutbound(row)"
        >
          {{ $t('page.product.inventory.transfer.action.outbound') }}
        </Button>
        <Button
          v-if="
            accessStore.hasAccessCode('product:transfer:inbound') &&
            row.status === 3
          "
          type="link"
          :icon="h(LucideArrowRightLeft)"
          @click="() => handleInbound(row)"
        >
          {{ $t('page.product.inventory.transfer.action.inbound') }}
        </Button>
        <Popconfirm
          v-if="row.status === 0 || row.status === 5 || row.status === 6"
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.product.inventory.transfer.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('product:transfer:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
              </template>
      </template>
    </Grid>
    <Drawer />
    <DetailDrawer />
  </Page>
</template>
