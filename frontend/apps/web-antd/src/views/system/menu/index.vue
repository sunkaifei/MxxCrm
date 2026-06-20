<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { IconifyIcon, LucideFilePenLine, LucideTrash2 } from '@vben/icons';

import { Button, Popconfirm, Switch, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deleteMenuApi, getMenuTreeApi, updateMenuApi } from '#/api';
import { $t } from '#/locales';
import { MenuType, statusList } from '#/store';

import MenuDrawer from './drawer.vue';
import { useAccessStore } from '@vben/stores';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: $t('ui.button.search'),
      componentProps: {
        placeholder: $t('ui.placeholder.search'),
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
  rowConfig: {
    isHover: true,
    height: 56,
  },
  stripe: true,
  treeConfig: {
    parentField: 'parentId',
    childrenField: 'children',
    rowField: 'id',
    transform: false,
  },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async (_, formValues) => {
        return await getMenuTreeApi({
          keywords: formValues.name,
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
      title: $t('page.system.menu.name'),
      field: 'meta.name',
      slots: { default: 'title' },
      treeNode: true,
      width: 200,
    },
    {
      title: $t('page.system.menu.type'),
      field: 'type',
      slots: { default: 'type' },
      width: 120,
    },
    {
      title: $t('page.system.menu.icon'),
      field: 'meta.icon',
      slots: { default: 'icon' },
      width: 80,
    },
    {
      title: $t('page.system.menu.path'),
      field: 'path',
      width: 180,
    },
    {
      title: $t('page.system.menu.component'),
      field: 'component',
      width: 180,
    },
    {
      title: $t('page.system.menu.perm'),
      field: 'perm',
      width: 180,
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      slots: { default: 'status' },
      width: 120,
    },
    {
      title: $t('ui.table.updateTime'),
      field: 'updateTime',
      formatter: 'formatDateTime',
      width: 160,
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

const [Grid, gridApi] = useVbenVxeGrid({
  gridOptions,
  formOptions,
});

const expandAll = () => {
  gridApi.grid?.setAllTreeExpand(true);
};

const collapseAll = () => {
  gridApi.grid?.setAllTreeExpand(false);
};

async function handleStatusChanged(row: any, checked: boolean) {
  row.pending = true;
  row.status = checked ? 1 : 2;
  try {
    await updateMenuApi(row.id, row);
    window.$message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: MenuDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any, parentId?: any) {
  drawerApi.setData({
    create,
    row,
    parentId,
    onRefresh: () => gridApi.query(),
  });
  drawerApi.open();
}

function handleCreate() {
  openDrawer(true);
}

function handleCreateChild(row: any) {
  openDrawer(true, null, row.id);
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteMenuApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.system.menu.title')">
      <template #toolbar-tools>
        <Button
          class="mr-2"
          type="primary"
          v-access:code="['system:menu:add']"
          @click="handleCreate"
        >
          {{ $t('page.system.menu.button.create') }}
        </Button>
        <Button class="mr-2" @click="expandAll">
          {{ $t('ui.tree.expand_all') }}
        </Button>
        <Button class="mr-2" @click="collapseAll">
          {{ $t('ui.tree.collapse_all') }}
        </Button>
      </template>

      <template #title="{ row }">
        <span class="mr-4">{{ $t(row.meta.name) }}</span>
      </template>

      <template #type="{ row }">
        <Tag v-if="row.type === MenuType.FOLDER" color="warning">
          {{ $t('enum.menuType.folder') }}
        </Tag>
        <Tag v-if="row.type === MenuType.MENU" color="success">
          {{ $t('enum.menuType.menu') }}
        </Tag>
        <Tag v-if="row.type === MenuType.BUTTON" color="error">
          {{ $t('enum.menuType.button') }}
        </Tag>
      </template>

      <template #icon="{ row }">
        <div class="flex h-full items-center justify-center">
          <IconifyIcon
            v-if="row.meta?.icon"
            :icon="row.meta.icon"
            class="size-5 text-gray-500"
          />
          <span v-else class="text-gray-400 text-sm">无图标</span>
        </div>
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

      <template #action="{ row }">
        <div class="flex items-center" style="gap: 8px">
        <Button
          type="primary"
          link
          v-access:code="['system:menu:add']"
          @click="() => handleCreateChild(row)"
        >
          {{ $t('page.system.menu.button.createChild') }}
        </Button>

        <Button
          type="primary"
          v-access:code="['system:menu:update']"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />

        <Popconfirm
          v-if="accessStore.hasAccessCode('system:menu:delete')"
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.system.menu.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            type="danger"
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
        </div>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
