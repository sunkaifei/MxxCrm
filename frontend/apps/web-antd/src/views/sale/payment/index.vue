<script lang="ts" setup>
import { h } from 'vue';

import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deletePaymentApi, getPaymentListApi } from '#/api';
import { $t } from '#/locales';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'paymentNo',
      label: '支付编号',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'orderNo',
      label: '订单编号',
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
        return await getPaymentListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          paymentNo: formValues.paymentNo,
          orderNo: formValues.orderNo,
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
      title: '支付编号',
      field: 'paymentNo',
    },
    {
      title: '订单编号',
      field: 'orderNo',
    },
    {
      title: '支付金额',
      field: 'amount',
    },
    {
      title: '支付方式',
      field: 'paymentMethod',
    },
    {
      title: '支付时间',
      field: 'paymentTime',
      slots: { default: 'paymentAt' },
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
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

async function handleEdit(row: any) {
  window.$message.info(`编辑支付: ${row.id}`);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deletePaymentApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleCreate() {
  window.$message.info('新增支付');
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.sale.payment.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('sale:payment:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.sale.payment.button.create') }}
        </Button>
      </template>

      <template #paymentAt="{ row }">
        {{ formatDateTime(row.paymentTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('sale:payment:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.sale.payment.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('sale:payment:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
  </Page>
</template>
