<script lang="ts" setup>
import { h } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { $t } from '#/locales';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2, LucidePencil } from '@vben/icons';
import { Button, Popconfirm, Switch } from 'ant-design-vue';
import RoleDrawer from './drawer.vue';
import SetAuthDrawer from './set-auth.vue';
import { deleteRoleApi, getRoleListApi, updateRoleApi } from '#/api';
import { formatDateTime } from '@vben/utils';
import { useAccessStore } from '@vben/stores';
import { statusList } from '#/store';

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
      width: 120,
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
  <Page auto-content-height>
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
          @change="(checked: boolean) => handleStatusChanged(row, checked)"
        />
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucidePencil)"
          @click="() => handleSetAuth(row)"
        />

        <Button
          type="primary"
          link
          v-access:code="['system:role:update']"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />

        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.system.role.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            type="danger"
            v-access:code="['system:role:delete']"
            link
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
    <AuthDrawer />
  </Page>
</template>
