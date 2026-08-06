<script lang="ts" setup>
import { h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Drawer, Form, Input, InputNumber, Modal, Popconfirm, Switch, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  createAlertRuleApi,
  deleteAlertRuleApi,
  getAlertRuleInfoApi,
  getAlertRuleListApi,
  updateAlertRuleApi,
} from '#/api/core/product/alert';
import { $t } from '#/locales';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'productName',
      label: $t('page.product.inventory.alert.field.productName'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'warehouseName',
      label: $t('page.product.inventory.alert.field.warehouseName'),
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
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const result = await getAlertRuleListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          productName: formValues.productName,
          warehouseName: formValues.warehouseName,
        });
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '200px' : '';
        }
        return result;
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inventory.alert.field.productName'), field: 'productName', minWidth: 140 },
    { title: $t('page.product.inventory.alert.field.warehouseName'), field: 'warehouseName', width: 120 },
    {
      title: $t('page.product.inventory.alert.field.minQuantity'),
      field: 'minQuantity',
      width: 120,
    },
    {
      title: $t('page.product.inventory.alert.field.maxQuantity'),
      field: 'maxQuantity',
      width: 120,
    },
    {
      title: $t('page.product.inventory.alert.field.staleDays'),
      field: 'staleDays',
      width: 100,
    },
    {
      title: $t('page.product.inventory.alert.field.enableLowAlert'),
      field: 'enableLowAlert',
      width: 100,
      slots: { default: 'enableLowAlert' },
    },
    {
      title: $t('page.product.inventory.alert.field.enableHighAlert'),
      field: 'enableHighAlert',
      width: 100,
      slots: { default: 'enableHighAlert' },
    },
    {
      title: $t('page.product.inventory.alert.field.enableStaleAlert'),
      field: 'enableStaleAlert',
      width: 100,
      slots: { default: 'enableStaleAlert' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      width: 120,
      fixed: 'right',
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 编辑/新增抽屉
const drawerVisible = ref(false);
const drawerTitle = ref('');
const isEdit = ref(false);
const submitLoading = ref(false);
const editForm = ref({
  id: undefined as number | undefined,
  productId: undefined as number | undefined,
  productName: '',
  warehouseId: undefined as number | undefined,
  warehouseName: '',
  minQuantity: 0,
  maxQuantity: 0,
  staleDays: 0,
  enableLowAlert: true,
  enableHighAlert: true,
  enableStaleAlert: false,
});

function handleCreate() {
  isEdit.value = false;
  drawerTitle.value = $t('page.product.inventory.alert.action.create');
  editForm.value = {
    id: undefined,
    productId: undefined,
    productName: '',
    warehouseId: undefined,
    warehouseName: '',
    minQuantity: 0,
    maxQuantity: 0,
    staleDays: 0,
    enableLowAlert: true,
    enableHighAlert: true,
    enableStaleAlert: false,
  };
  drawerVisible.value = true;
}

async function handleEdit(row: any) {
  isEdit.value = true;
  drawerTitle.value = $t('page.product.inventory.alert.action.edit');
  try {
    const info = await getAlertRuleInfoApi(row.id);
    const data = (info as any)?.data ?? row;
    editForm.value = {
      id: data.id,
      productId: data.productId,
      productName: data.productName || '',
      warehouseId: data.warehouseId,
      warehouseName: data.warehouseName || '',
      minQuantity: data.minQuantity ?? 0,
      maxQuantity: data.maxQuantity ?? 0,
      staleDays: data.staleDays ?? 0,
      enableLowAlert: !!data.enableLowAlert,
      enableHighAlert: !!data.enableHighAlert,
      enableStaleAlert: !!data.enableStaleAlert,
    };
  } catch {
    editForm.value = { ...row };
  }
  drawerVisible.value = true;
}

async function handleSubmit() {
  submitLoading.value = true;
  try {
    if (isEdit.value) {
      await updateAlertRuleApi(editForm.value);
      message.success($t('ui.notification.update_success'));
    } else {
      await createAlertRuleApi(editForm.value);
      message.success($t('ui.notification.create_success'));
    }
    drawerVisible.value = false;
    gridApi.query();
  } finally {
    submitLoading.value = false;
  }
}

async function handleDelete(row: any) {
  try {
    await deleteAlertRuleApi([row.id]);
    message.success($t('ui.notification.delete_success'));
    gridApi.query();
  } catch {
    // 错误提示由拦截器处理
  }
}

async function handleBatchDelete() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) {
    message.warning($t('ui.prompt.please_select_data'));
    return;
  }
  Modal.confirm({
    title: $t('ui.text.batch_delete_title'),
    content: `${$t('ui.text.confirm_batch_delete')} ${records.length} ?`,
    onOk: async () => {
      try {
        await deleteAlertRuleApi(records.map((r: any) => r.id));
        message.success($t('ui.notification.delete_success'));
        gridApi.query();
      } catch {
        // ignore
      }
    },
  });
}
</script>

<template>
  <Page>
    <Grid :table-title="$t('page.product.inventory.alert.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:alert:edit')"
          type="primary"
          class="mr-2"
          :icon="h(LucidePlus)"
          @click="handleCreate"
        >
          {{ $t('page.product.inventory.alert.action.create') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('product:alert:edit')"
          class="mr-2"
          danger
          ghost
          :icon="h(LucideTrash2)"
          @click="handleBatchDelete"
        >
          {{ $t('ui.button.batchDelete') }}
        </Button>
      </template>

      <template #enableLowAlert="{ row }">
        <Tag :color="row.enableLowAlert ? 'green' : 'default'">
          {{ row.enableLowAlert ? $t('ui.enabled') : $t('ui.disabled') }}
        </Tag>
      </template>
      <template #enableHighAlert="{ row }">
        <Tag :color="row.enableHighAlert ? 'green' : 'default'">
          {{ row.enableHighAlert ? $t('ui.enabled') : $t('ui.disabled') }}
        </Tag>
      </template>
      <template #enableStaleAlert="{ row }">
        <Tag :color="row.enableStaleAlert ? 'green' : 'default'">
          {{ row.enableStaleAlert ? $t('ui.enabled') : $t('ui.disabled') }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:alert:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        >
          {{ $t('page.product.inventory.alert.action.edit') }}
        </Button>
        <Popconfirm
          v-if="accessStore.hasAccessCode('product:alert:edit')"
          :title="$t('ui.text.do_you_want_delete')"
          @confirm="() => handleDelete(row)"
        >
          <Button type="link" danger :icon="h(LucideTrash2)">
            {{ $t('page.product.inventory.alert.action.delete') }}
          </Button>
        </Popconfirm>
      </template>
    </Grid>

    <Drawer
      v-model:open="drawerVisible"
      :title="drawerTitle"
      :width="500"
      placement="right"
      :mask-closable="true"
      :closable="true"
    >
      <Form layout="vertical">
        <Form.Item :label="$t('page.product.inventory.alert.field.productName')">
          <Input v-model:value="editForm.productName" :placeholder="$t('ui.placeholder.input')" />
        </Form.Item>
        <Form.Item :label="$t('page.product.inventory.alert.field.warehouseName')">
          <Input v-model:value="editForm.warehouseName" :placeholder="$t('ui.placeholder.input')" />
        </Form.Item>
        <Form.Item :label="$t('page.product.inventory.alert.field.minQuantity')">
          <InputNumber v-model:value="editForm.minQuantity" style="width: 100%" :min="0" />
        </Form.Item>
        <Form.Item :label="$t('page.product.inventory.alert.field.maxQuantity')">
          <InputNumber v-model:value="editForm.maxQuantity" style="width: 100%" :min="0" />
        </Form.Item>
        <Form.Item :label="$t('page.product.inventory.alert.field.staleDays')">
          <InputNumber v-model:value="editForm.staleDays" style="width: 100%" :min="0" />
        </Form.Item>
        <Form.Item :label="$t('page.product.inventory.alert.field.enableLowAlert')">
          <Switch v-model:checked="editForm.enableLowAlert" />
        </Form.Item>
        <Form.Item :label="$t('page.product.inventory.alert.field.enableHighAlert')">
          <Switch v-model:checked="editForm.enableHighAlert" />
        </Form.Item>
        <Form.Item :label="$t('page.product.inventory.alert.field.enableStaleAlert')">
          <Switch v-model:checked="editForm.enableStaleAlert" />
        </Form.Item>
      </Form>
      <template #footer>
        <div style="text-align: right">
          <Button class="mr-2" @click="drawerVisible = false">{{ $t('ui.button.cancel') }}</Button>
          <Button type="primary" :loading="submitLoading" @click="handleSubmit">{{ $t('ui.button.ok') }}</Button>
        </div>
      </template>
    </Drawer>
  </Page>
</template>
