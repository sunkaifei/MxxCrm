<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Switch } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deletePostApi, getPostListApi, updatePostApi } from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

import PostDrawer from './drawer.vue';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'postName',
      label: $t('page.system.post.postName'),
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
        return await getPostListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          postName: formValues.postName,
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
      title: $t('page.system.post.postName'),
      field: 'postName',
    },
    {
      title: $t('page.system.post.postCode'),
      field: 'postCode',
    },
    {
      title: $t('ui.table.sortId'),
      field: 'sort',
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      slots: { default: 'status' },
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
    await updatePostApi(row.id, row);

    window.$message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: PostDrawer,
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
    await deletePostApi(row.id);

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
          v-if="accessStore.hasAccessCode('system:post:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.system.post.button.create') }}
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
          v-if="accessStore.hasAccessCode('system:post:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.system.post.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('system:post:delete')"
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
