<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Switch } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  batchDeleteUserManageApi,
  getUserManageListApi,
  updateUserManageStatusApi,
} from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

import UserDrawer from './drawer.vue';

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'username',
      label: $t('page.user.username'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'mobile',
      label: $t('page.user.mobile'),
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
        return await getUserManageListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          username: formValues.username,
          mobile: formValues.mobile,
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
      title: $t('page.user.username'),
      field: 'username',
    },
    {
      title: $t('page.user.nickname'),
      field: 'nickname',
    },
    {
      title: $t('page.user.mobile'),
      field: 'mobile',
      width: 130,
    },
    {
      title: $t('page.user.email'),
      field: 'email',
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      slots: { default: 'status' },
    },
    {
      title: $t('page.user.lastLoginTime'),
      field: 'lastLoginTime',
      slots: { default: 'lastLoginTime' },
      width: 180,
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      slots: { default: 'createTime' },
      width: 180,
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
  try {
    await updateUserManageStatusApi({
      id: row.id,
      status: checked ? '1' : '0',
    });
    row.status = checked ? '1' : '0';
    window.$message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
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
    await batchDeleteUserManageApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.user.title')">
      <template #toolbar-tools>
        <Button class="mr-2" type="primary" @click="handleCreate">
          {{ $t('page.user.button.create') }}
        </Button>
      </template>

      <template #status="{ row }">
        <Switch
          :checked="row.status === '1'"
          :loading="row.pending"
          :checked-children="$t('ui.switch.active')"
          :un-checked-children="$t('ui.switch.inactive')"
          @change="(checked: boolean) => handleStatusChanged(row, checked)"
        />
      </template>

      <template #lastLoginTime="{ row }">
        {{ formatDateTime(row.lastLoginTime) }}
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />

        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.user.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <template #reference>
            <Button type="danger" link :icon="h(LucideTrash2)" />
          </template>
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
