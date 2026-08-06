<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import type { VbenFormSchema } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { $t } from '#/locales';
import { createInboundApi, getInboundInfoApi, updateInboundApi } from '#/api/core/product/inbound';
import { getWarehouseListApi } from '#/api/core/product/warehouse';
import { Alert, message, Tooltip } from 'ant-design-vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

const drawerClass = computed(() => [
  'inbound-drawer',
  { 'inbound-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

// 入库类型选项
const inboundTypeOptions = [
  { label: $t('page.product.inbound.type.purchase'), value: 'purchase' },
  { label: $t('page.product.inbound.type.return'), value: 'return' },
  { label: $t('page.product.inbound.type.surplus'), value: 'surplus' },
  { label: $t('page.product.inbound.type.initial'), value: 'initial' },
  { label: $t('page.product.inbound.type.other'), value: 'other' },
];

// 仓库选项（异步加载）
const warehouseOptions = ref<{ label: string; value: number }[]>([]);

async function loadWarehouseOptions() {
  try {
    const resp = await getWarehouseListApi({ page: 1, pageSize: 999 });
    const list = resp?.data ?? resp ?? [];
    warehouseOptions.value = (Array.isArray(list) ? list : []).map((w: any) => ({
      label: w.warehouseName ?? w.name ?? w.label,
      value: Number(w.id ?? w.value),
    }));
  } catch (e) {
    console.error('[入库] 加载仓库列表失败:', e);
  }
}

const formSchema: VbenFormSchema[] = [
  {
    component: 'Divider',
    fieldName: '_div1',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => $t('page.product.inbound.drawer.basicInfo') }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'inboundType',
    label: $t('page.product.inbound.drawer.inboundType'),
    rules: 'required',
    defaultValue: 'purchase',
    componentProps: {
      placeholder: $t('page.product.inbound.drawer.inboundTypePlaceholder'),
      options: inboundTypeOptions,
      allowClear: true,
    },
  },
  {
    component: 'Select',
    fieldName: 'warehouseId',
    label: $t('page.product.inbound.drawer.warehouse'),
    rules: 'required',
    componentProps: {
      placeholder: $t('page.product.inbound.drawer.warehousePlaceholder'),
      options: warehouseOptions,
      allowClear: true,
      showSearch: true,
      filterOption: (input: string, option: any) =>
        (option?.label ?? '').toLowerCase().includes(input.toLowerCase()),
    },
  },
  {
    component: 'Input',
    fieldName: 'sourceOrderNo',
    label: $t('page.product.inbound.drawer.sourceOrderNo'),
    componentProps: { placeholder: $t('page.product.inbound.drawer.sourceOrderNoPlaceholder'), allowClear: true },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: $t('page.product.inbound.drawer.remark'),
    componentProps: { placeholder: $t('page.product.inbound.drawer.remarkPlaceholder'), allowClear: true, rows: 2 },
    formItemClass: 'col-span-2',
  },
  {
    component: 'Divider',
    fieldName: '_div2',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => $t('page.product.inbound.drawer.detail') }),
    formItemClass: 'col-span-2',
  },
];

const [MainForm, mainFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: { class: 'w-full' },
  },
  schema: formSchema,
});

const [Drawer, drawerApi] = useVbenDrawer({
  async onConfirm() {
    try {
      const valid = await mainFormApi.validate();
      if (!valid.valid) return;

      confirmLoading.value = true;
      const values = await mainFormApi.getValues();

      const data = { ...values };

      if (drawerData.value.create) {
        await createInboundApi(data);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateInboundApi({ ...data, id: drawerData.value.row.id });
        message.success($t('ui.notification.update_success'));
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      confirmLoading.value = false;
    }
  },
  onCancel() {
    drawerApi.close();
  },
  onOpenChange(isOpen: boolean) {
    if (isOpen) {
      isFullscreen.value = false;
      drawerData.value = drawerApi.getData<{ create: boolean; row?: any }>() || { create: true };
      mainFormApi.resetForm();
      confirmLoading.value = false;
      loadWarehouseOptions();
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getInboundInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const num = (v: any) => (v === null || v === undefined ? undefined : Number(v));

    mainFormApi.setValues({
      inboundType: data.inboundType ?? 'purchase',
      warehouseId: data.warehouseId ? num(data.warehouseId) : undefined,
      sourceOrderNo: data.sourceOrderNo,
      remark: data.remark,
    });
  } catch (e) {
    console.error('[入库] 加载详情失败:', e);
  }
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="drawerData.create ? $t('page.product.inbound.drawer.title.create') : $t('page.product.inbound.drawer.title.edit')"
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? $t('page.product.inbound.drawer.restore') : $t('page.product.inbound.drawer.fullscreen')">
        <button type="button" class="inbound-drawer__fs-btn" @click="toggleFullscreen">
          <svg v-if="!isFullscreen" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </button>
      </Tooltip>
    </template>

    <div class="inbound-drawer__body">
      <MainForm />
      <Alert
        type="info"
        show-icon
        :message="$t('page.product.inbound.drawer.alertTitle')"
        :description="$t('page.product.inbound.drawer.alertDesc')"
        class="inbound-item-alert"
      />
    </div>
  </Drawer>
</template>

<style>
.inbound-drawer {
  width: 75vw !important;
}

.inbound-drawer--fullscreen {
  width: 100vw !important;
}

.inbound-drawer__fs-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin-right: 8px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(0, 0, 0, 0.45);
  cursor: pointer;
  transition: all 0.2s;
}

.inbound-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

.inbound-drawer__body {
  padding: 0 8px;
  overflow-y: auto;
  height: calc(100vh - 150px);
}

.inbound-drawer__body .ant-divider {
  margin: 12px 0 8px;
}

.inbound-drawer__body .ant-divider-inner-text {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}

.inbound-item-alert {
  margin-top: 8px;
}
</style>
