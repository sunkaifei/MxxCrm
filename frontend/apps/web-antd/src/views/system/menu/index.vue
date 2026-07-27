<script lang="ts" setup>
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

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
  exportConfig: {},
  pagerConfig: {
    enabled: false,
  },
  cellConfig: {
    isHover: true,
  },
  rowConfig: { height: 48 },
  treeConfig: {
    parentField: 'parentId',
    childrenField: 'children',
    rowField: 'id',
    transform: false,
    accordion: true,
  },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async (_, formValues) => {
        const result = await getMenuTreeApi({
          keywords: formValues.name,
          status: formValues.status,
        });
        // 无数据 280px，有数据按内容自适应
        const items = Array.isArray(result) ? result : [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '280px' : '';
        }
        // 等DOM渲染完成后同步固定列行高并居中内容
        const syncFixedColumn = (retry = 0) => {
          const $el = gridApi.grid?.$el as HTMLElement | undefined;
          if (!$el) return;
          const mainBody = $el.querySelector('.vxe-table--body-wrapper tbody');
          const fixedRightBody = $el.querySelector('.vxe-table--fixed-right-wrapper tbody');
          if (!mainBody || !fixedRightBody) {
            if (retry < 3) setTimeout(() => syncFixedColumn(retry + 1), 200);
            return;
          }
          const rows1 = mainBody.querySelectorAll('tr.vxe-body--row');
          const rows2 = fixedRightBody.querySelectorAll('tr.vxe-body--row');
          const len = Math.min(rows1.length, rows2.length);
          if (len === 0) return;
          for (let i = 0; i < len; i++) {
            const h = (rows1[i] as HTMLElement).offsetHeight;
            if (h === 0) continue;
            (rows2[i] as HTMLElement).style.height = h + 'px';
            const tds = (rows2[i] as HTMLElement).querySelectorAll('td');
            tds.forEach((td: Element) => {
              const cell = td.querySelector('.vxe-cell');
              if (cell) {
                (cell as HTMLElement).style.display = 'flex';
                (cell as HTMLElement).style.alignItems = 'center';
                (cell as HTMLElement).style.justifyContent = 'center';
                (cell as HTMLElement).style.height = h + 'px';
              }
            });
          }
        };
        requestAnimationFrame(() => {
          syncFixedColumn();
          setTimeout(() => syncFixedColumn(), 200);
          setTimeout(() => syncFixedColumn(), 500);
        });
        return result;
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
      showOverflow: true,
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
      showOverflow: true,
    },
    {
      title: $t('page.system.menu.component'),
      field: 'component',
      width: 180,
      showOverflow: true,
    },
    {
      title: $t('page.system.menu.perm'),
      field: 'perm',
      width: 180,
      showOverflow: true,
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
  <Page>
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
          :disabled="!accessStore.hasAccessCode('system:menu:update')"
          :checked="row.status === 1"
          :loading="row.pending"
          :checked-children="$t('ui.switch.active')"
          :un-checked-children="$t('ui.switch.inactive')"
          @change="(checked: boolean) => handleStatusChanged(row, checked)"
        />
      </template>

      <template #action="{ row }">
        <div class="flex items-center justify-center" style="gap: 12px">
        <a
          v-if="row.type !== MenuType.BUTTON && accessStore.hasAccessCode('system:menu:add')"
          class="text-blue-600 cursor-pointer"
          @click="() => handleCreateChild(row)"
        >
          {{ $t('page.system.menu.button.createChild') }}
        </a>

        <a
          v-if="accessStore.hasAccessCode('system:menu:update')"
          class="text-blue-600 cursor-pointer"
          @click="() => handleEdit(row)"
        >
          编辑
        </a>

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
          <a class="text-red-500 cursor-pointer">删除</a>
        </Popconfirm>
        </div>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
