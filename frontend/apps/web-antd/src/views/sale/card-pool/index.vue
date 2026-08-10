<script lang="ts" setup>
import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';
import { useAccessStore } from '@vben/stores';

import { Button, Popconfirm, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import type { VbenFormProps } from '@vben/common-ui';
import { deleteCardPoolApi, getCardPoolListApi } from '#/api';
import { $t } from '#/locales';

const accessStore = useAccessStore();

// 卡密状态映射：1=未售 2=已锁定 3=已售 4=已作废
const statusMap: Record<number, { label: string; color: string }> = {
  1: { label: '未售', color: 'default' },
  2: { label: '已锁定', color: 'orange' },
  3: { label: '已售', color: 'green' },
  4: { label: '已作废', color: 'red' },
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'InputNumber',
      fieldName: 'productId',
      label: '商品ID',
      componentProps: {
        placeholder: '请输入商品ID',
        allowClear: true,
        controls: false,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: '请选择',
        allowClear: true,
        options: [
          { label: '未售', value: 1 },
          { label: '已锁定', value: 2 },
          { label: '已售', value: 3 },
          { label: '已作废', value: 4 },
        ],
      },
    },
    {
      component: 'Input',
      fieldName: 'batchNo',
      label: '批次号',
      componentProps: {
        placeholder: '请输入批次号',
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
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          productId: formValues.productId,
          status: formValues.status,
          batchNo: formValues.batchNo,
        };
        return await getCardPoolListApi(params);
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
      title: 'ID',
      field: 'id',
      width: 80,
    },
    {
      title: '商品ID',
      field: 'productId',
      width: 100,
    },
    {
      title: '批次号',
      field: 'batchNo',
      width: 160,
    },
    {
      title: '卡密',
      field: 'cardKeyMasked',
      width: 200,
      slots: { default: 'cardKeyMasked' },
    },
    {
      title: '状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: '已售订单ID',
      field: 'soldOrderId',
      width: 120,
      slots: { default: 'soldOrderId' },
    },
    {
      title: '售出时间',
      field: 'soldTime',
      width: 160,
      slots: { default: 'soldTime' },
    },
    {
      title: '导入批次',
      field: 'importBatch',
      width: 140,
    },
    {
      title: '过期时间',
      field: 'expireTime',
      width: 160,
      slots: { default: 'expireTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 100,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleDelete(row: any) {
  try {
    await deleteCardPoolApi(row.id);
    message.success($t('ui.notification.delete_success'));
    gridApi.query();
  } catch {
    // 错误由全局拦截器处理
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.sale.cardPool.title')">
      <template #cardKeyMasked="{ row }">
        <span class="font-mono">{{ row.cardKeyMasked || '-' }}</span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status && statusMap[row.status]" :color="statusMap[row.status]?.color">
          {{ row.statusName || statusMap[row.status]?.label }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #soldOrderId="{ row }">
        <span v-if="row.soldOrderId">{{ row.soldOrderId }}</span>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #soldTime="{ row }">
        {{ formatDateTime(row.soldTime) }}
      </template>

      <template #expireTime="{ row }">
        {{ formatDateTime(row.expireTime) }}
      </template>

      <template #action="{ row }">
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.sale.cardPool.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('sale:card-pool:delete')"
            type="link"
            danger
            size="small"
          >
            {{ $t('page.sale.cardPool.button.delete') }}
          </Button>
        </Popconfirm>
      </template>
    </Grid>
  </Page>
</template>
