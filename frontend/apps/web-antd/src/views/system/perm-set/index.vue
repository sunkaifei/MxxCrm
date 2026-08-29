<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { Page, useVbenDrawer } from '@vben/common-ui';

import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Switch } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { copyPermSetApi, deletePermSetApi, getPermSetListApi, updatePermSetApi } from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

import PermSetDrawer from './drawer.vue';
// P3-2 权限预览面板：复用角色模块的预览组件（权限集模式仅展示菜单授权区块）
import PreviewDrawer from '../role/preview-drawer.vue';
import SetAuthDrawer from './set-auth.vue';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'permSetName',
      label: $t('page.system.permSet.name'),
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
        return await getPermSetListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          permSetName: formValues.permSetName,
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
      title: $t('page.system.permSet.name'),
      field: 'permSetName',
    },
    {
      title: $t('page.system.permSet.code'),
      field: 'permSetKey',
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
      width: 300,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleStatusChanged(row: any, checked: boolean) {
  row.pending = true;
  row.status = checked ? 1 : 2;
  try {
    await updatePermSetApi(row.id, row);
    window.$message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: PermSetDrawer,
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

// P3-2 权限预览面板：纯展示弹窗，关闭后无需刷新列表
const [PreviewAuthDrawer, previewDrawerApi] = useVbenDrawer({
  connectedComponent: PreviewDrawer,
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
    await deletePermSetApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function handleSetAuth(row: any) {
  openAuthDrawer(row);
}

// P3-2 权限预览：弹窗展示权限集的菜单授权区块
function handlePreview(row: any) {
  previewDrawerApi.setData({
    mode: 'permSet',
    row,
  });
  previewDrawerApi.open();
}

// 复制权限集（P3-1 一键复制）：成功后刷新列表展示新副本
async function handleCopy(row: any) {
  row.pending = true;
  try {
    await copyPermSetApi(row.id);
    window.$message.success('复制成功，新权限集名称为原名称加"副本"后缀');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page>
    <Grid :table-title="$t('page.system.permSet.title')">
      <template #toolbar-tools>
        <Button
          class="mr-2"
          type="primary"
          v-access:code="['system:perm_set:save']"
          @click="handleCreate"
        >
          {{ $t('page.system.permSet.button.create') }}
        </Button>
      </template>

      <template #status="{ row }">
        <Switch
          :disabled="!accessStore.hasAccessCode('system:perm_set:update')"
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
        <span class="action-link" v-access:code="['system:perm_set:view']"
          @click="() => handlePreview(row)"
        >预览</span>
        <span
          class="action-link"
          v-access:code="['system:perm_set:update']"
          @click="() => handleEdit(row)"
        >{{ $t('page.system.common.button.edit') }}</span>
        <Popconfirm
          title="确认复制该权限集？将同时复制其菜单授权配置"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleCopy(row)"
        >
          <span class="action-link" v-access:code="['system:perm_set:save']">复制</span>
        </Popconfirm>
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.system.permSet.module') })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <span class="action-link action-link-danger" v-access:code="['system:perm_set:delete']">{{ $t('page.system.common.button.delete') }}</span>
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
    <AuthDrawer />
    <PreviewAuthDrawer />
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
