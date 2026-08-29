<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { Page, useVbenDrawer } from '@vben/common-ui';

import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Switch } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { copyRoleApi, deleteRoleApi, getRoleListApi, updateRoleApi } from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

import RoleDrawer from './drawer.vue';
// P3-2 权限预览面板：展示角色的菜单授权/数据范围/成员三区块
import PreviewDrawer from './preview-drawer.vue';
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
      width: 300,
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

// P3-2 权限预览：弹窗展示角色的菜单/数据范围/成员三区块
function handlePreview(row: any) {
  previewDrawerApi.setData({
    mode: 'role',
    row,
  });
  previewDrawerApi.open();
}

// 复制角色（P3-1 一键复制）：成功后刷新列表展示新副本
async function handleCopy(row: any) {
  row.pending = true;
  try {
    await copyRoleApi(row.id);
    window.$message.success('复制成功，新角色名称为原名称加"副本"后缀');
  } finally {
    row.pending = false;
    gridApi.query();
  }
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
        <span class="action-link" v-access:code="['system:role:view']"
          @click="() => handlePreview(row)"
        >预览</span>
        <span
          class="action-link"
          v-access:code="['system:role:update']"
          @click="() => handleEdit(row)"
        >{{ $t('page.system.common.button.edit') }}</span>
        <Popconfirm
          title="确认复制该角色？将同时复制其菜单、部门与数据范围配置"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleCopy(row)"
        >
          <span class="action-link" v-access:code="['system:role:save']">复制</span>
        </Popconfirm>
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
