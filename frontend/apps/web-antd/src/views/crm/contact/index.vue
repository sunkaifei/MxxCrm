<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getContactListApi } from '#/api';
import { $t } from '#/locales';

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: '姓名',
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
      fieldName: 'email',
      label: '邮箱',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
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
        return await getContactListApi({
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
      title: '姓名',
      field: 'name',
    },
    {
      title: '职位',
      field: 'title',
    },
    {
      title: '邮箱',
      field: 'email',
    },
    {
      title: '电话',
      field: 'phone',
    },
    {
      title: '决策人',
      field: 'isDecisionMaker',
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
    <Grid :table-title="$t('page.crm.contact.title')">
      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt) }}
      </template>
    </Grid>
  </Page>
</template>
