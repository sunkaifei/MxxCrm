<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideEye, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Switch } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deleteDeptApi, getDeptListApi, updateDeptApi } from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

import DeptDrawer from './drawer.vue';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'deptName',
      label: $t('page.system.dept.deptName'),
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
  pagerConfig: {
    enabled: false,
  },
  treeConfig: {
    parentField: 'parentId',
    childrenField: 'children',
    rowField: 'id',
    transform: false,
  },
  cellConfig: {
    isHover: true,
    height: 48,
  },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async (_, formValues) => {
        return await getDeptListApi({
          keywords: formValues.deptName,
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
      title: $t('page.system.dept.deptName'),
      field: 'deptName',
      minWidth: 180,
      treeNode: true,
    },
    {
      title: $t('page.system.dept.code'),
      field: 'code',
      width: 120,
    },
    {
      title: $t('page.system.dept.leader'),
      field: 'leader',
      width: 120,
    },
    {
      title: $t('page.system.dept.phone'),
      field: 'phone',
      width: 130,
    },
    {
      title: $t('page.system.dept.email'),
      field: 'email',
      width: 180,
    },
    {
      title: $t('ui.table.sortId'),
      field: 'sort',
      width: 80,
    },
    {
      title: $t('ui.table.status'),
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
      width: 180,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleStatusChanged(row: any, checked: boolean) {
  row.pending = true;
  row.status = checked ? 1 : 2;
  try {
    await updateDeptApi(row.id, row);

    window.$message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: DeptDrawer,
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
    await deleteDeptApi(row.id);

    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('system:dept:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.system.dept.button.create') }}
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt) }}
      </template>

      <template #status="{ row }">
        <Switch
          v-model:checked="row.status"
          :checked-value="1"
          :loading="row.pending"
          :un-checked-value="2"
          @change="(checked: boolean) => handleStatusChanged(row, checked)"
        />
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('system:dept:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.system.dept.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('system:dept:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
