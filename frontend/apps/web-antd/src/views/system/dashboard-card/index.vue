<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import { Button, Modal, Popconfirm, Select, Switch, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  assignDashboardCardRolesApi,
  deleteDashboardCardApi,
  getDashboardCardListApi,
  updateDashboardCardApi,
} from '#/api/core/system/dashboard-card';
import { getRoleOptionsApi } from '#/api/core/system/role';
import { $t } from '#/locales';

import CardDrawer from './drawer.vue';

// ===== 角色选项（value->label 映射） =====
const roleOptions = ref<any[]>([]);
const roleLabelMap = ref<Record<number, string>>({});

async function loadRoles() {
  try {
    const res: any = await getRoleOptionsApi();
    const list = Array.isArray(res) ? res : res?.data || res?.list || [];
    roleOptions.value = list.map((item: any) => ({
      label: item.label ?? item.roleName,
      value: item.value ?? item.id,
    }));
    roleLabelMap.value = {};
    list.forEach((item: any) => {
      const id = Number(item.value ?? item.id);
      roleLabelMap.value[id] = item.label ?? item.roleName;
    });
  } catch {
    roleOptions.value = [];
  }
}
loadRoles();

// ===== 角色分配弹窗 =====
const assignVisible = ref(false);
const assignSaving = ref(false);
const assignCard = ref<any>(null);
const assignRoleIds = ref<number[]>([]);

function openAssign(row: any) {
  assignCard.value = row;
  assignRoleIds.value = [...(row.roleIds || [])];
  assignVisible.value = true;
}

async function handleAssignConfirm() {
  if (!assignCard.value) return;
  assignSaving.value = true;
  try {
    await assignDashboardCardRolesApi({
      cardId: Number(assignCard.value.id),
      roleIds: assignRoleIds.value,
    });
    window.$message.success($t('ui.notification.update_success'));
    assignVisible.value = false;
    gridApi.query();
  } catch {
    // 错误由全局拦截器处理
  } finally {
    assignSaving.value = false;
  }
}

// ===== 列表 =====
const formOptions = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: $t('page.system.dashboardCard.keyword'),
      componentProps: {
        placeholder: $t('page.system.dashboardCard.keywordPlaceholder'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.system.dashboardCard.status'),
      componentProps: {
        allowClear: true,
        options: [
          { label: $t('ui.switch.active'), value: 1 },
          { label: $t('ui.switch.inactive'), value: 0 },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  pagerConfig: {},
  cellConfig: {},
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getDashboardCardListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          name: formValues.name,
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
      title: $t('page.system.dashboardCard.cardCode'),
      field: 'cardCode',
      width: 140,
      slots: { default: 'cardCode' },
    },
    {
      title: $t('page.system.dashboardCard.cardName'),
      field: 'cardName',
      minWidth: 120,
    },
    {
      title: $t('page.system.dashboardCard.pageKey'),
      field: 'pageKey',
      width: 140,
      slots: { default: 'pageKey' },
    },
    {
      title: $t('page.system.dashboardCard.visibleRoles'),
      field: 'roleIds',
      minWidth: 200,
      slots: { default: 'visibleRoles' },
    },
    {
      title: $t('page.system.dashboardCard.sortOrder'),
      field: 'sortOrder',
      width: 80,
      align: 'center',
    },
    {
      title: $t('page.system.dashboardCard.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 170,
      slots: { default: 'createdAt' },
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
  connectedComponent: CardDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
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
    await deleteDashboardCardApi([Number(row.id)]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function handleBatchDelete() {
  const selectedRows = gridApi.grid?.getCheckboxRecords();
  if (!selectedRows || selectedRows.length === 0) {
    window.$message.warning($t('ui.notification.select_row'));
    return;
  }
  Modal.confirm({
    title: $t('ui.text.do_you_want_delete', {
      moduleName: $t('page.system.dashboardCard.module'),
    }),
    okText: $t('ui.button.ok'),
    cancelText: $t('ui.button.cancel'),
    onOk: async () => {
      const ids = selectedRows.map((row: any) => Number(row.id));
      try {
        await deleteDashboardCardApi(ids);
        window.$message.success($t('ui.notification.delete_success'));
        gridApi.query();
      } catch {
        // 错误由全局拦截器处理
      }
    },
  });
}

async function handleStatusChange(row: any, checked: boolean) {
  const newStatus = checked ? 1 : 0;
  try {
    await updateDashboardCardApi({ id: Number(row.id), status: newStatus });
    row.status = newStatus;
    window.$message.success($t('ui.notification.update_success'));
  } catch {
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.system.dashboardCard.title')">
      <template #toolbar-tools>
        <Button
          class="mr-2"
          v-access:code="['system:dashboard:save']"
          type="primary"
          @click="handleCreate"
        >
          {{ $t('page.system.dashboardCard.button.create') }}
        </Button>
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.system.dashboardCard.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleBatchDelete"
        >
          <Button
            danger
            v-access:code="['system:dashboard:delete']"
            :icon="h(LucideTrash2)"
          >
            {{ $t('ui.button.batch_delete') }}
          </Button>
        </Popconfirm>
      </template>

      <template #cardCode="{ row }">
        <Tag color="geekblue">{{ row.cardCode }}</Tag>
      </template>

      <template #pageKey="{ row }">
        <span class="text-xs text-gray-500">{{ row.pageKey }}</span>
      </template>

      <template #visibleRoles="{ row }">
        <div class="flex flex-wrap gap-1">
          <Tag
            v-for="rid in row.roleIds || []"
            :key="rid"
            color="blue"
          >
            {{ roleLabelMap[Number(rid)] || `#${rid}` }}
          </Tag>
          <Tag v-if="!row.roleIds || row.roleIds.length === 0" color="default">
            {{ $t('page.system.dashboardCard.noRoles') }}
          </Tag>
        </div>
      </template>

      <template #status="{ row }">
        <Switch
          :checked="row.status === 1"
          :checked-children="$t('ui.switch.active')"
          :un-checked-children="$t('ui.switch.inactive')"
          @change="(checked: any) => handleStatusChange(row, checked)"
        />
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          v-access:code="['system:dashboard:update']"
          :icon="h(LucideFilePenLine)"
          @click="() => openAssign(row)"
        >
          {{ $t('page.system.dashboardCard.button.assignRoles') }}
        </Button>
        <Button
          type="primary"
          link
          v-access:code="['system:dashboard:update']"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.system.dashboardCard.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            danger
            v-access:code="['system:dashboard:delete']"
            link
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />

    <!-- 角色分配弹窗 -->
    <Modal
      v-model:open="assignVisible"
      :title="
        $t('page.system.dashboardCard.assignTitle', {
          name: assignCard?.cardName || '',
        })
      "
      :confirm-loading="assignSaving"
      :ok-text="$t('ui.button.ok')"
      :cancel-text="$t('ui.button.cancel')"
      @ok="handleAssignConfirm"
    >
      <p class="mb-3 text-sm text-gray-500">
        {{ $t('page.system.dashboardCard.assignTip') }}
      </p>
      <Select
        v-model:value="assignRoleIds"
        :options="roleOptions"
        mode="multiple"
        class="w-full"
        :placeholder="$t('ui.placeholder.select')"
      />
    </Modal>
  </Page>
</template>
