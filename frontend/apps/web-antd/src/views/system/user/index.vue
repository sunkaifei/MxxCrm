<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { $t } from '#/locales';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { Button, Popconfirm, Switch, Tag } from 'ant-design-vue';
import UserDrawer from './drawer.vue';
import { deleteUserApi, getUserListApi, updateUserApi } from '#/api';
import { statusList } from '#/store';
import { formatDateTime } from '@vben/utils';

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'userName',
      label: $t('page.system.user.usernName'),
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
  rowConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getUserListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          username: formValues.username,
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
      title: $t('page.system.user.username'),
      field: 'userName',
    },
    {
      title: $t('page.system.user.nickName'),
      field: 'nickName',
    },
    {
      title: $t('page.system.user.email'),
      field: 'email',
    },
    {
      title: $t('page.system.user.role'),
      field: 'roleName',
      slots: { default: 'roleName' },
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      slots: { default: 'status' },
    },
    {
      title: $t('page.system.user.lastLoginTime'),
      field: 'lastLoginTime',
      slots: { default: 'lastLoginTime' },
    },
    {
      title: $t('page.system.user.lastLoginIp'),
      field: 'lastLoginIp',
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

async function handleStatusChanged(row: any, checked: boolean) {
  row.pending = true;
  row.status = checked ? 1 : 2;
  try {
    await updateUserApi(row.id, row);
    window.$message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: UserDrawer,
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

function handleCreate() {
  openDrawer(true);
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteUserApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.system.user.title')">
      <template #toolbar-tools>
        <Button
          class="mr-2"
          v-access:code="['system:admin:save']"
          type="primary"
          @click="handleCreate"
        >
          {{ $t('page.system.user.button.create') }}
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt) }}
      </template>

      <template #roleName="{ row }">
        <Tag color="success">
          {{ row.roleName }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Switch
          :checked="row.status === 1"
          :loading="row.pending"
          :checked-children="$t('ui.switch.active')"
          :un-checked-children="$t('ui.switch.inactive')"
          @change="(checked: boolean) => handleStatusChanged(row, checked)"
        />
      </template>

      <template #lastLoginTime="{ row }">
        {{ formatDateTime(row.lastLoginTime) }}
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          v-access:code="['system:admin:update']"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />

        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.system.user.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <template #reference>
            <Button
              type="danger"
              v-access:code="['system:admin:delete']"
              link
              :icon="h(LucideTrash2)"
            />
          </template>
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
