<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getLeadPoolListApi } from '#/api';
import { $t } from '#/locales';

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'companyName',
      label: '公司名称',
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
        return await getLeadPoolListApi({
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
      title: '公司名称',
      field: 'companyName',
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
      title: $t('ui.table.createTime'),
      field: 'createTime',
      slots: { default: 'createdAt' },
    },
  ],
};

const [Grid] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.crm.leadPool.title')">
      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt) }}
      </template>
    </Grid>
  </Page>
</template>
