<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { $t } from '#/locales';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2, LogOut } from '@vben/icons';
import { Button, Popconfirm, Switch, Tag } from 'ant-design-vue';
import UserDrawer from './drawer.vue';
import UserDetailDrawer from '../../crm/components/UserDetailDrawer.vue';
import {
  auditUserApi,
  deleteUserApi,
  getUserListApi,
  kickOfflineApi,
  updateUserApi,
} from '#/api';
import { statusList } from '#/store';
import { formatDateTime } from '@vben/utils';
import { useAccessStore } from '@vben/stores';

const accessStore = useAccessStore();

const detailVisible = ref(false);
const detailUserId = ref<number | string | null>(null);

function openDetail(row: any) {
  detailUserId.value = row.id;
  detailVisible.value = true;
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'userName',
      label: $t('page.system.user.username'),
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
  exportConfig: {},
  pagerConfig: {},
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getUserListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          userName: formValues.userName,
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
      width: 140,
      slots: { default: 'userName' },
    },
    {
      title: $t('page.system.user.nickName'),
      field: 'nickName',
      width: 140,
      slots: { default: 'nickName' },
    },
    {
      title: $t('page.system.user.dept'),
      field: 'deptName',
      width: 140,
    },
    {
      title: $t('page.system.user.post'),
      field: 'postName',
      width: 140,
    },
    {
      title: $t('page.system.user.mobile'),
      field: 'mobile',
      width: 120,
    },
    {
      title: $t('page.system.user.email'),
      field: 'email',
      width: 180,
    },
    {
      title: $t('page.system.user.role'),
      field: 'roleName',
      width: 120,
      slots: { default: 'roleName' },
    },
    {
      title: $t('page.system.user.directManager'),
      field: 'directManagerName',
      width: 120,
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      width: 80,
      slots: { default: 'status' },
    },
    {
      title: $t('page.system.setting.online'),
      field: 'online',
      width: 80,
      slots: { default: 'online' },
    },
    {
      title: $t('page.system.user.auditStatus.label'),
      field: 'auditStatus',
      width: 90,
      slots: { default: 'auditStatus' },
    },
    {
      title: $t('page.system.user.lastLoginTime'),
      field: 'lastLoginTime',
      width: 140,
      slots: { default: 'lastLoginTime' },
    },
    {
      title: $t('page.system.user.lastLoginIp'),
      field: 'lastLoginIp',
      width: 120,
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 140,
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
  row.status = checked ? 1 : 0;
  try {
    await updateUserApi({ id: row.id, status: row.status });
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

async function handleKickOffline(row: any) {
  row.pending = true;
  try {
    await kickOfflineApi(row.id);
    window.$message.success($t('page.system.setting.kickSuccess'));
  } finally {
    row.pending = false;
  }
}

async function handleAudit(row: any, auditStatus: number) {
  row.pending = true;
  try {
    await auditUserApi(row.id, auditStatus);
    window.$message.success(auditStatus === 1 ? '审核已通过' : '已拒绝');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}
</script>

<template>
  <Page>
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
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #userName="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.userName }}</a>
      </template>

      <template #nickName="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.nickName }}</a>
      </template>

      <template #roleName="{ row }">
        <Tag color="success">
          {{ row.roleName }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Switch
          :disabled="!accessStore.hasAccessCode('system:admin:update') || row.userType === 1"
          :checked="row.status === 1"
          :loading="row.pending"
          :checked-children="$t('ui.switch.active')"
          :un-checked-children="$t('ui.switch.inactive')"
          @change="(checked: any) => handleStatusChanged(row, checked)"
        />
      </template>

      <template #online="{ row }">
        <Tag :color="row.online ? 'success' : 'default'">
          {{ row.online ? $t('page.system.setting.online') : $t('ui.switch.inactive') }}
        </Tag>
      </template>

      <template #auditStatus="{ row }">
        <Tag v-if="row.auditStatus === 0" color="warning">
          {{ $t('page.system.user.auditStatus.pending') }}
        </Tag>
        <Tag v-else color="success">
          {{ $t('page.system.user.auditStatus.approved') }}
        </Tag>
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
          <Button
            danger
            v-access:code="['system:admin:delete']"
            link
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>

        <Popconfirm
          :title="$t('page.system.setting.kickConfirm')"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleKickOffline(row)"
        >
          <Button
            danger
            type="primary"
            link
            v-access:code="['system:admin:kick']"
            :icon="h(LogOut)"
            :disabled="row.userType === 1"
          />
        </Popconfirm>

        <!-- 审核：仅待审核用户（auditStatus=0）显示 -->
        <Popconfirm
          v-if="row.auditStatus === 0"
          :title="`确认审核通过该用户？通过后将自动启用，可登录系统。`"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleAudit(row, 1)"
        >
          <Button
            type="primary"
            link
            v-access:code="['system:admin:audit']"
            :disabled="row.userType === 1"
          >
            {{ $t('page.system.user.button.auditPass') }}
          </Button>
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
    <UserDetailDrawer v-model:visible="detailVisible" :id="detailUserId ?? undefined" />
  </Page>
</template>

<style scoped>
:deep(.vxe-table--empty-block) {
  min-height: 150px;
}

:deep(.vxe-grid) {
  overflow: hidden;
}
</style>
