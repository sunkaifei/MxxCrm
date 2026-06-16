<script lang="ts" setup>
import { h } from 'vue';

import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deletePurchaseOrderApi, getPurchaseOrderListApi } from '#/api';
import { $t } from '#/locales';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'poNo',
      label: '采购单号',
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
        return await getPurchaseOrderListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          poNo: formValues.poNo,
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
      title: '采购单号',
      field: 'poNo',
    },
    {
      title: '供应商',
      field: 'supplierName',
    },
    {
      title: '采购金额',
      field: 'totalAmount',
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
      width: 200,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleEdit(row: any) {
  window.$message.info(`编辑采购单: ${row.id}`);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deletePurchaseOrderApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleCreate() {
  window.$message.info('新增采购单');
}

async function handleConfirm(row: any) {
  window.$message.info(`确认采购: ${row.id}`);
}

async function handleReceive(row: any) {
  window.$message.info(`入库: ${row.id}`);
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.purchase.po.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('purchase:po:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.purchase.po.button.create') }}
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt) }}
      </template>

      <template #status="{ row }">
        <Tag>{{ row.status }}</Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('purchase:po:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.purchase.po.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('purchase:po:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
  </Page>
</template>
