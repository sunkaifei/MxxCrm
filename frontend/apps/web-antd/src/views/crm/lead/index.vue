<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteLeadApi, getLeadListApi } from '#/api';
import { $t } from '#/locales';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'companyName',
      label: '线索名称',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'contactName',
      label: '联系人',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'phone',
      label: '电话',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'source',
      label: '来源',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
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
        return await getLeadListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
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
      title: '线索名称',
      field: 'companyName',
    },
    {
      title: '联系人',
      field: 'contactName',
    },
    {
      title: '电话',
      field: 'phone',
    },
    {
      title: '来源',
      field: 'source',
    },
    {
      title: '状态',
      field: 'status',
    },
    {
      title: '负责人',
      field: 'assignee',
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

function handleCreate() {
  // TODO: open create drawer
}

function handleEdit(row: any) {
  // TODO: open edit drawer
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteLeadApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.crm.lead.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('crm:lead:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.crm.lead.button.create') }}
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('crm:lead:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />

        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.crm.lead.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('crm:lead:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
  </Page>
</template>
