<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { Page, useVbenDrawer } from '@vben/common-ui';

import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Switch } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteRoleApi, getRoleListApi, updateRoleApi } from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

import RoleDrawer from './drawer.vue';
import SetAuthDrawer from './set-auth.vue';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: $t('page.system.role.name'),
      defaultValue: '',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('ui.table.status'),
      componentProps: {
        options: statusList,
        placeholder: $t('ui.placeholder.select'),
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
  minHeight: 600,
  exportConfig: {},
  pagerConfig: {},
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getRoleListApi({
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
      width: 70,
    },
    {
      title: $t('page.system.role.name'),
      field: 'roleName',
    },
    {
      title: $t('page.system.role.code'),
      field: 'roleKey',
    },
    {
      title: $t('ui.table.sortId'),
      field: 'sort',
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      slots: { default: 'status' },
      width: 95,
    },
    {
      title: $t('ui.table.remark'),
      field: 'remark',
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      slots: { default: 'createTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 180,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleStatusChanged(row: any, checked: boolean) {
  row.pending = true;
  row.status = checked ? 1 : 2;
  try {
    await updateRoleApi(row.id, row);
    window.$message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: RoleDrawer,
  onClosed() {
    gridApi.query();
  },
});

const [AuthDrawer, authDrawerApi] = useVbenDrawer({
  connectedComponent: SetAuthDrawer,
  onClosed() {
    gridApi.query();
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({
    create,
    row,
  });
  drawerApi.open();
}

function openAuthDrawer(row?: any) {
  authDrawerApi.setData({
    row,
  });
  authDrawerApi.open();
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
    await deleteRoleApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function handleSetAuth(row: any) {
  openAuthDrawer(row);
}
</script>

<template>
  <Page>
    <Grid :table-title="$t('page.system.role.title')">
      <template #toolbar-tools>
        <Button
          class="mr-2"
          type="primary"
          v-access:code="['system:role:save']"
          @click="handleCreate"
        >
          {{ $t('page.system.role.button.create') }}
        </Button>
      </template>

      <template #status="{ row }">
        <Switch
          :disabled="!accessStore.hasAccessCode('system:role:update')"
          :checked="row.status === 1"
          :loading="row.pending"
          :checked-children="$t('ui.switch.active')"
          :un-checked-children="$t('ui.switch.inactive')"
          @change="(checked: any) => handleStatusChanged(row, checked)"
        />
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <span class="action-link" @click="() => handleSetAuth(row)">{{ $t('page.system.user.authority') }}</span>
        <span
          class="action-link"
          v-access:code="['system:role:update']"
          @click="() => handleEdit(row)"
        >{{ $t('page.system.common.button.edit') }}</span>
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.system.role.module') })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <span class="action-link action-link-danger" v-access:code="['system:role:delete']">{{ $t('page.system.common.button.delete') }}</span>
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
    <AuthDrawer />
  </Page>
</template>

<style scoped>
.action-link {
  color: #1677ff;
  cursor: pointer;
  margin-right: 12px;
}
.action-link:hover {
  text-decoration: underline;
}
.action-link-danger {
  color: #ff4d4f;
}
</style>
