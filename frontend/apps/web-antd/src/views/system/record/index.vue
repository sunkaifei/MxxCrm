<script lang="ts" setup>
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { $t } from '#/locales';
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideTrash2 } from '@vben/icons';
import { Button, Popconfirm } from 'ant-design-vue';
import { deleteRecordApi, getRecordListApi } from '#/api';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'username',
      label: $t('page.system.record.username'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'createTime',
      label: $t('ui.input-search.optTime'),
      componentProps: {
        type: 'datetimerange',
        rangeSeparator: '至',
        startPlaceholder: '开始时间',
        endPlaceholder: '结束时间',
        valueFormat: 'YYYY-MM-DD HH:mm:ss',
        showTime: true,
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
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getRecordListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          username: formValues.username,
          createTime: formValues.createTime,
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
      title: $t('page.system.record.username'),
      field: 'username',
    },
    {
      title: $t('page.system.record.description'),
      field: 'description',
    },
    {
      title: $t('page.system.record.method'),
      field: 'method',
    },
    {
      title: $t('page.system.record.path'),
      field: 'path',
    },
    {
      title: $t('page.system.record.statusCode'),
      field: 'statusCode',
    },
    {
      title: $t('page.system.record.msg'),
      field: 'msg',
    },
    {
      title: $t('page.system.record.elapsed'),
      field: 'elapsed',
    },
    {
      title: $t('page.system.record.platform'),
      field: 'platform',
    },
    {
      title: $t('page.system.record.ip'),
      field: 'ip',
    },
    {
      title: $t('page.system.record.createTime'),
      field: 'createTime',
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteRecordApi(row.id);

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
      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.system.record.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('system:record:delete')"
            type="danger"
            link
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
  </Page>
</template>
