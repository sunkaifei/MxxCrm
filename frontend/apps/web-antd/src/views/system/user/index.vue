<script lang="ts" setup>
import { ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { $t } from '#/locales';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { Button, Dropdown, Menu, Popconfirm, Tag } from 'ant-design-vue';
import UserDrawer from './drawer.vue';
import UserDetailDrawer from '../../crm/components/UserDetailDrawer.vue';
import {
  deleteUserApi,
  getUserListApi,
  kickOfflineApi,
  updateUserApi,
} from '#/api';
import { submitApprovalApi } from '#/api/core/system/approval';
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
      title: $t('page.system.user.hireDate'),
      field: 'hireDate',
      width: 110,
    },
    {
      title: $t('page.system.user.salaryEnabled'),
      field: 'salaryEnabled',
      width: 100,
      slots: { default: 'salaryEnabled' },
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
      width: 170,
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

// 提交审核：走系统审批流引擎（business_type=user），审批通过后自动启用用户
async function handleSubmitAudit(row: any) {
  row.pending = true;
  try {
    await submitApprovalApi({
      flowCode: 'user_approval',
      businessType: 'user',
      businessId: row.id,
      businessTitle: row.nickName || row.userName || `用户#${row.id}`,
    });
    window.$message.success('已提交审核，等待审批人处理');
    gridApi.query();
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
        <Tag :color="row.status === 1 ? 'success' : 'default'">
          {{ row.status === 1 ? $t('enum.status.ON') : $t('enum.status.OFF') }}
        </Tag>
      </template>

      <template #online="{ row }">
        <Tag :color="row.online ? 'success' : 'default'">
          {{ row.online ? $t('page.system.setting.online') : $t('ui.switch.inactive') }}
        </Tag>
      </template>

      <template #salaryEnabled="{ row }">
        <Tag :color="row.salaryEnabled === 1 ? 'success' : 'default'">
          {{ row.salaryEnabled === 1 ? $t('page.system.user.salaryYes') : $t('page.system.user.salaryNo') }}
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
        <!-- 待审核用户：提交审核走系统审批流引擎 -->
        <Button
          v-if="row.auditStatus === 0 && row.userType !== 1 && accessStore.hasAccessCode('system:admin:audit')"
          type="link"
          class="!px-0"
          :loading="row.pending"
          @click="() => handleSubmitAudit(row)"
        >
          {{ $t('page.system.user.button.submitAudit') }}
        </Button>

        <!-- 更多操作下拉菜单：停用/启用、下线、编辑、删除 -->
        <Dropdown placement="bottomRight">
          <Button type="link">{{ $t('page.system.user.button.more') }}</Button>
          <template #overlay>
            <Menu>
              <Menu.Item
                v-if="row.userType !== 1 && accessStore.hasAccessCode('system:admin:update')"
                key="toggleStatus"
                @click="() => handleStatusChanged(row, row.status !== 1)"
              >
                {{ row.status === 1 ? $t('page.system.user.button.disable') : $t('page.system.user.button.enable') }}
              </Menu.Item>
              <Popconfirm
                :title="$t('page.system.setting.kickConfirm')"
                :ok-text="$t('ui.button.ok')"
                :cancel-text="$t('ui.button.cancel')"
                @confirm="() => handleKickOffline(row)"
              >
                <Menu.Item
                  v-if="accessStore.hasAccessCode('system:admin:kick')"
                  key="kick"
                  :disabled="row.userType === 1"
                >
                  {{ $t('page.system.user.button.kickOffline') }}
                </Menu.Item>
              </Popconfirm>
              <Menu.Item
                v-if="accessStore.hasAccessCode('system:admin:update')"
                key="edit"
                @click="() => handleEdit(row)"
              >
                {{ $t('page.system.user.button.editAction') }}
              </Menu.Item>
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
                <Menu.Item
                  v-if="accessStore.hasAccessCode('system:admin:delete')"
                  key="delete"
                  danger
                >
                  {{ $t('page.system.user.button.deleteAction') }}
                </Menu.Item>
              </Popconfirm>
            </Menu>
          </template>
        </Dropdown>
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
