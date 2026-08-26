<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, message, Popconfirm, Switch, Tabs } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deletePostApi, getPostListApi, updatePostApi } from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

import PostDrawer from './drawer.vue';
import SalaryBand from './salary-band.vue';

const accessStore = useAccessStore();

// 页面 Tab：岗位管理 / 薪资带宽（带宽入口内嵌岗位管理页）
const activeTab = ref('post');

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
  cellConfig: {},
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

    message.success($t('ui.notification.update_success'));
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
    await deletePostApi([row.id]);

    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Tabs
      v-model:active-key="activeTab"
      class="post-tabs"
      :destroy-inactive-tab-pane="false"
    >
      <Tabs.TabPane key="post" tab="岗位管理">
        <Grid>
          <template #toolbar-tools>
            <Button
              v-if="accessStore.hasAccessCode('system:post:save')"
              type="primary"
              class="mr-2"
              @click="handleCreate"
            >
              {{ $t('page.system.post.button.create') }}
            </Button>
          </template>

          <template #createdAt="{ row }">
            {{ formatDateTime(row.createTime) }}
          </template>

          <template #status="{ row }">
            <Switch
              v-model:checked="row.status"
              :checked-value="1"
              :disabled="!accessStore.hasAccessCode('system:post:update')"
              :loading="row.pending"
              :un-checked-value="2"
              @change="(checked: any) => handleStatusChanged(row, checked)"
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
      </Tabs.TabPane>

      <Tabs.TabPane key="salaryBand" tab="薪资带宽">
        <SalaryBand />
      </Tabs.TabPane>
    </Tabs>
  </Page>
</template>

<style scoped>
/* 让 Tabs 内容区撑满 Page 高度，保证内嵌 Grid 自适应滚动 */
.post-tabs,
.post-tabs :deep(.ant-tabs-content-holder),
.post-tabs :deep(.ant-tabs-content),
.post-tabs :deep(.ant-tabs-tabpane-active) {
  height: 100%;
}
</style>
