<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import {
  LucideFilePenLine,
  LucideUndo2,
  LucideTrash2,
} from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Popconfirm, Tabs, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteWarehouseApi,
  getWarehouseListApi,
  purgeWarehouseApi,
  restoreWarehouseApi,
} from '#/api';
import { $t } from '#/locales';

import WarehouseDetailDrawer from '../components/WarehouseDetailDrawer.vue';
import WarehouseDrawer from './drawer.vue';

const accessStore = useAccessStore();

// ===== 范围视图（仓储管理改造 W2/W3）=====
const activeTab = ref('all');
const crossViewEnabled = ref(false);
const isManagement = ref(false);

const tabList = [
  { key: 'all', label: '全部仓库' },
  { key: 'mine', label: '我的仓库' },
  { key: 'recycle', label: '回收站' },
];

async function handleTabChange() {
  // 切换范围时更新列（回收站追加 删除人/删除时间）
  (gridApi as any)?.setGridOptions?.({ columns: buildColumns() });
  gridApi.query();
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'warehouseName',
      label: '仓库名称',
      componentProps: {
        placeholder: '请输入仓库名称',
        allowClear: true,
      },
    },
  ],
};

function getWarehouseTypeTag(type: number) {
  const map: Record<number, { color: string; label: string }> = {
    1: { label: '原材料仓', color: 'blue' },
    2: { label: '成品仓', color: 'green' },
    3: { label: '半成品仓', color: 'orange' },
    4: { label: '退货仓', color: 'red' },
    5: { label: '中转仓', color: 'purple' },
  };
  return map[type] || { label: '未知', color: 'default' };
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
        const res: any = await getWarehouseListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          warehouseName: formValues.warehouseName,
          scope: activeTab.value,
        });
        // 后端回传身份/互看标记，驱动 Tab 显隐
        if (res && typeof res === 'object') {
          crossViewEnabled.value = Boolean(res.crossViewEnabled);
          isManagement.value = Boolean(res.isManagement);
        }
        return res;
      },
    },
  },

  columns: buildColumns() as any,
};

function buildColumns() {
  const isRecycle = activeTab.value === 'recycle';
  return [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 60,
    },
    {
      title: '仓库编码',
      field: 'code',
      width: 110,
      slots: { default: 'code' },
    },
    {
      title: '仓库名称',
      field: 'warehouseName',
      minWidth: 140,
      slots: { default: 'warehouseName' },
    },
    {
      title: '仓库类型',
      field: 'warehouseType',
      width: 110,
      slots: { default: 'warehouseType' },
    },
    {
      title: '所在区域',
      field: 'region',
      width: 110,
    },
    {
      title: '面积(㎡)',
      field: 'areaSqm',
      width: 90,
    },
    {
      title: '负责人',
      field: 'contactPerson',
      width: 100,
    },
    {
      title: '联系电话',
      field: 'contactPhone',
      width: 130,
    },
    {
      title: '地址',
      field: 'address',
      minWidth: 160,
      showOverflow: 'tooltip',
    },
    {
      title: $t('ui.table.status'),
      field: 'isActive',
      width: 80,
      slots: { default: 'status' },
    },
    {
      title: '创建时间',
      field: 'createTime',
      width: 160,
    },
    ...(isRecycle
      ? [
          { title: '删除人', field: 'deleteByName', width: 100 },
          { title: '删除时间', field: 'deleteTime', width: 160 },
        ]
      : []),
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: isRecycle ? 140 : 120,
    },
  ];
}


const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: WarehouseDrawer,
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

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteWarehouseApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function handleCreate() {
  openDrawer(true);
}

// ===== 回收站操作（W2）=====
async function handleRestore(row: any) {
  row.pending = true;
  try {
    await restoreWarehouseApi(row.id);
    window.$message.success('已恢复');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handlePurge(row: any) {
  row.pending = true;
  try {
    await purgeWarehouseApi(row.id);
    window.$message.success('已彻底删除');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// ============ 仓库详情抽屉 ============
const detailVisible = ref(false);
const detailId = ref<null | number>(null);

function openDetail(row: any) {
  detailId.value = Number(row.id);
  detailVisible.value = true;
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.product.warehouse.title')">
      <template #form-header>
        <Tabs
          v-model:active-key="activeTab"
          class="mb-3"
          @change="handleTabChange"
        >
          <Tabs.TabPane
            v-for="tab in tabList.filter((t) => t.key !== 'all' || isManagement || crossViewEnabled)"
            :key="tab.key"
            :tab="tab.label"
          />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:warehouse:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.warehouse.button.create') }}
        </Button>
      </template>

      <template #code="{ row }">
        <a
          class="text-primary hover:underline cursor-pointer"
          @click="openDetail(row)"
        >
          {{ row.code || '-' }}
        </a>
      </template>

      <template #warehouseName="{ row }">
        <a
          class="text-primary hover:underline cursor-pointer"
          @click="openDetail(row)"
        >
          {{ row.warehouseName || '-' }}
        </a>
      </template>

      <template #warehouseType="{ row }">
        <Tag :color="getWarehouseTypeTag(row.warehouseType).color">
          {{ getWarehouseTypeTag(row.warehouseType).label }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="row.isActive !== false ? 'green' : 'red'">
          {{ row.isActive !== false ? '启用' : '停用' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <template v-if="activeTab === 'recycle'">
          <Popconfirm
            title="确认恢复该仓库？"
            ok-text="恢复"
            cancel-text="取消"
            @confirm="() => handleRestore(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('product:warehouse:delete')"
              type="link"
              :icon="h(LucideUndo2)"
            />
          </Popconfirm>
          <Popconfirm
            v-if="isManagement"
            title="彻底删除后不可恢复，确认？"
            ok-text="彻底删除"
            cancel-text="取消"
            @confirm="() => handlePurge(row)"
          >
            <Button type="link" danger :icon="h(LucideTrash2)" />
          </Popconfirm>
        </template>
        <template v-else>
          <Button
            v-if="accessStore.hasAccessCode('product:warehouse:update')"
            type="link"
            :icon="h(LucideFilePenLine)"
            @click="() => handleEdit(row)"
          />
          <Popconfirm
            :title="
              $t('ui.text.do_you_want_delete', {
                moduleName: $t('page.product.warehouse.title'),
              })
            "
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleDelete(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('product:warehouse:delete')"
              type="link"
              danger
              :icon="h(LucideTrash2)"
            />
          </Popconfirm>
        </template>
      </template>
    </Grid>
    <Drawer />

    <!-- 仓库详情抽屉 -->
    <WarehouseDetailDrawer
      :visible="detailVisible"
      :warehouse-id="detailId"
      @update:visible="(val) => (detailVisible = val)"
    />
  </Page>
</template>
