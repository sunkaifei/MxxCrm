<script lang="ts" setup>
import { h, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Drawer, Form, Input, InputNumber, Modal, Popconfirm, Select, Switch, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  createAlertRuleApi,
  deleteAlertRuleApi,
  getAlertRuleInfoApi,
  getAlertRuleListApi,
  updateAlertRuleApi,
} from '#/api/core/product/alert';
import { getProductListApi } from '#/api';
import { $t } from '#/locales';

import WarehouseSelectModal from '../inventory-check/WarehouseSelectModal.vue';

const accessStore = useAccessStore();
const router = useRouter();

// ============ 产品列表选项（异步加载） ============
const productOptions = ref<{ label: string; value: number }[]>([]);

async function loadProductOptions() {
  try {
    const res: any = await getProductListApi({ page: 1, pageSize: 999 });
    const list = res?.list || res?.items || res || [];
    productOptions.value = list.map((p: any) => ({
      label: p.name || p.productName || '',
      value: Number(p.id),
    }));
  } catch {
    productOptions.value = [];
  }
}

// ============ 仓库弹窗选择 ============
const warehouseSelectVisible = ref(false);

function openWarehouseSelect() {
  warehouseSelectVisible.value = true;
}

function onWarehouseSelected(warehouse: any) {
  editForm.value.warehouseId = Number(warehouse.id);
  editForm.value.warehouseName = warehouse.warehouseName ?? warehouse.name ?? '';
}

function clearWarehouse() {
  editForm.value.warehouseId = undefined;
  editForm.value.warehouseName = '';
}

// ============ 列表筛选 ============
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
    {
      title: $t('page.product.inventory.alert.field.productName'),
      field: 'productName',
      minWidth: 140,
    },
    {
      title: $t('page.product.inventory.alert.field.warehouseName'),
      field: 'warehouseName',
      width: 120,
    },
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

// ============ 编辑/新增抽屉 ============
const drawerVisible = ref(false);
const drawerTitle = ref('');
const isEdit = ref(false);
const submitLoading = ref(false);

// 注意：fieldName 使用后端 camelCase 字段名
const editForm = ref({
  id: undefined as number | undefined,
  productId: undefined as number | undefined,
  warehouseId: undefined as number | undefined,
  warehouseName: '',
  minQuantity: 0,
  maxQuantity: 0,
  staleDays: 90,
  enableLowAlert: true,
  enableHighAlert: false,
  enableStaleAlert: false,
  notifyUsers: '' as string,
});

function handleCreate() {
  isEdit.value = false;
  drawerTitle.value = $t('page.product.inventory.alert.action.create');
  editForm.value = {
    id: undefined,
    productId: undefined,
    warehouseId: undefined,
    warehouseName: '',
    minQuantity: 0,
    maxQuantity: 0,
    staleDays: 90,
    enableLowAlert: true,
    enableHighAlert: false,
    enableStaleAlert: false,
    notifyUsers: '',
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
      productId: data.productId ? Number(data.productId) : undefined,
      warehouseId: data.warehouseId ? Number(data.warehouseId) : undefined,
      warehouseName: data.warehouseName || '',
      minQuantity: data.minQuantity ?? 0,
      maxQuantity: data.maxQuantity ?? 0,
      staleDays: data.staleDays ?? 90,
      enableLowAlert: data.enableLowAlert ?? true,
      enableHighAlert: data.enableHighAlert ?? false,
      enableStaleAlert: data.enableStaleAlert ?? false,
      notifyUsers: data.notifyUsers || '',
    };
  } catch {
    editForm.value = { ...row };
  }
  drawerVisible.value = true;
}

async function handleSubmit() {
  // 至少启用一种预警
  if (!editForm.value.enableLowAlert && !editForm.value.enableHighAlert && !editForm.value.enableStaleAlert) {
    message.warning('请至少启用一种预警类型');
    return;
  }
  // 低库存预警需要设置最低阈值
  if (editForm.value.enableLowAlert && (!editForm.value.minQuantity || editForm.value.minQuantity <= 0)) {
    message.warning('启用低库存预警时需设置最低数量');
    return;
  }
  // 高库存预警需要设置最高阈值
  if (editForm.value.enableHighAlert && (!editForm.value.maxQuantity || editForm.value.maxQuantity <= 0)) {
    message.warning('启用高库存预警时需设置最高数量');
    return;
  }

  submitLoading.value = true;
  try {
    // 构造提交数据，确保字段名与后端 camelCase 匹配
    const payload = {
      productId: editForm.value.productId || undefined,
      warehouseId: editForm.value.warehouseId || undefined,
      minQuantity: editForm.value.minQuantity || undefined,
      maxQuantity: editForm.value.maxQuantity || undefined,
      staleDays: editForm.value.staleDays ?? 90,
      enableLowAlert: editForm.value.enableLowAlert,
      enableHighAlert: editForm.value.enableHighAlert,
      enableStaleAlert: editForm.value.enableStaleAlert,
      notifyUsers: editForm.value.notifyUsers || undefined,
    };

    if (isEdit.value) {
      await updateAlertRuleApi({ ...payload, id: editForm.value.id });
      message.success($t('ui.notification.update_success'));
    } else {
      await createAlertRuleApi(payload);
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

onMounted(() => {
  loadProductOptions();
});
</script>

<template>
  <Page>
    <Grid :table-title="$t('page.product.inventory.alert.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:alert:update')"
          type="primary"
          class="mr-2"
          :icon="h(LucidePlus)"
          @click="handleCreate"
        >
          {{ $t('page.product.inventory.alert.action.create') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('product:alert:update')"
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
          v-if="accessStore.hasAccessCode('product:alert:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        >
          {{ $t('page.product.inventory.alert.action.edit') }}
        </Button>
        <Popconfirm
          v-if="accessStore.hasAccessCode('product:alert:update')"
          :title="$t('ui.text.do_you_want_delete')"
          @confirm="() => handleDelete(row)"
        >
          <Button type="link" danger :icon="h(LucideTrash2)">
            {{ $t('page.product.inventory.alert.action.delete') }}
          </Button>
        </Popconfirm>
      </template>
    </Grid>

    <!-- 新增/编辑抽屉 -->
    <Drawer
      v-model:open="drawerVisible"
      :title="drawerTitle"
      :width="500"
      placement="right"
      :mask-closable="true"
      :closable="true"
    >
      <Form layout="vertical">
        <!-- 产品选择：不选=全部产品 -->
        <Form.Item label="产品（不选=全部产品）">
          <Select
            v-model:value="editForm.productId"
            placeholder="全部产品（不选则对所有产品生效）"
            allow-clear
            show-search
            :options="productOptions"
            :filter-option="(input: string, option: any) => (option?.label ?? '').toLowerCase().includes(input.toLowerCase())"
            style="width: 100%"
          />
        </Form.Item>

        <!-- 仓库选择：不选=全部仓库 -->
        <Form.Item label="仓库（不选=全部仓库）">
          <Input
            :value="editForm.warehouseName || ''"
            placeholder="全部仓库（不选则对所有仓库生效）"
            readonly
            allow-clear
            style="cursor: pointer"
            @click="openWarehouseSelect"
            @change="(e: any) => { if (!e?.target?.value) clearWarehouse(); }"
          />
        </Form.Item>

        <Form.Item :label="$t('page.product.inventory.alert.field.minQuantity')">
          <InputNumber v-model:value="editForm.minQuantity" style="width: 100%" :min="0" placeholder="最低库存阈值" />
        </Form.Item>
        <Form.Item :label="$t('page.product.inventory.alert.field.maxQuantity')">
          <InputNumber v-model:value="editForm.maxQuantity" style="width: 100%" :min="0" placeholder="最高库存阈值" />
        </Form.Item>
        <Form.Item :label="$t('page.product.inventory.alert.field.staleDays')">
          <InputNumber v-model:value="editForm.staleDays" style="width: 100%" :min="0" placeholder="呆滞天数（默认90）" />
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
        <Form.Item label="通知用户（逗号分隔的用户ID）">
          <Input
            v-model:value="editForm.notifyUsers"
            placeholder="如：1,2,3（留空则不通知）"
            allow-clear
          />
        </Form.Item>
      </Form>
      <template #footer>
        <div style="text-align: right">
          <Button class="mr-2" @click="drawerVisible = false">{{ $t('ui.button.cancel') }}</Button>
          <Button type="primary" :loading="submitLoading" @click="handleSubmit">{{ $t('ui.button.ok') }}</Button>
        </div>
      </template>
    </Drawer>

    <!-- 仓库选择弹窗 -->
    <WarehouseSelectModal
      :visible="warehouseSelectVisible"
      @update:visible="(val) => (warehouseSelectVisible = val)"
      @select="onWarehouseSelected"
    />
  </Page>
</template>
