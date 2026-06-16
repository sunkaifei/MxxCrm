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
import { deleteContractApi, getContractListApi } from '#/api';
import { $t } from '#/locales';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'contractNo',
      label: '合同编号',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'customerName',
      label: '客户',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
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
        return await getContractListApi({
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
      title: '合同编号',
      field: 'contractNo',
    },
    {
      title: '客户',
      field: 'customerName',
    },
    {
      title: '合同金额',
      field: 'amount',
    },
    {
      title: '状态',
      field: 'status',
    },
    {
      title: '开始日期',
      field: 'startDate',
    },
    {
      title: '结束日期',
      field: 'endDate',
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
    await deleteContractApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.crm.contract.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('crm:contract:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.crm.contract.button.create') }}
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('crm:contract:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />

        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.crm.contract.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('crm:contract:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
  </Page>
</template>
