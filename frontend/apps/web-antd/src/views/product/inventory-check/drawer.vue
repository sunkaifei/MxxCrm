<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import type { VbenFormSchema } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import {
  createCheckApi,
  getCheckInfoApi,
  updateCheckApi,
} from '#/api/core/product/check';
import { getWarehouseListApi } from '#/api/core/product/warehouse';
import { $t } from '#/locales';
import { message, Tooltip } from 'ant-design-vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

const drawerClass = computed(() => [
  'check-drawer',
  { 'check-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const checkTypeOptions = [
  { label: $t('page.product.inventory.check.type.1'), value: 1 },
  { label: $t('page.product.inventory.check.type.2'), value: 2 },
  { label: $t('page.product.inventory.check.type.3'), value: 3 },
];

// 盘点类型说明文案
const checkTypeDescMap: Record<number, string> = {
  1: $t('page.product.inventory.check.typeDesc.1'),
  2: $t('page.product.inventory.check.typeDesc.2'),
  3: $t('page.product.inventory.check.typeDesc.3'),
};

const warehouseOptions = ref<{ label: string; value: number }[]>([]);

async function loadWarehouseOptions() {
  try {
    const resp: any = await getWarehouseListApi({ page: 1, pageSize: 999 });
    const list = resp?.data ?? resp ?? [];
    warehouseOptions.value = (Array.isArray(list) ? list : []).map(
      (w: any) => ({
        label: w.label ?? w.warehouseName ?? w.name,
        value: Number(w.value ?? w.id),
      }),
    );
  } catch (e) {
    console.error('[库存盘点] 加载仓库选项失败:', e);
  }
}

// 根据盘点类型更新说明文本
function updateCheckTypeDesc(type?: number) {
  const desc = type ? checkTypeDescMap[type] ?? '' : '';
  mainFormApi.setFieldValue('checkTypeDesc', desc);
}

const formSchema: VbenFormSchema[] = [
  {
    component: 'Divider',
    fieldName: '_div1',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => $t('page.product.inventory.check.drawer.basicInfo') }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'checkType',
    label: $t('page.product.inventory.check.field.checkType'),
    defaultValue: 1,
    rules: 'required',
    componentProps: {
      placeholder: $t('page.product.inventory.check.drawer.checkTypePlaceholder'),
      options: checkTypeOptions,
      allowClear: true,
      onChange: (val: number) => {
        updateCheckTypeDesc(val);
      },
    },
  },
  {
    component: 'Select',
    fieldName: 'warehouseId',
    label: $t('page.product.inventory.check.field.warehouse'),
    rules: 'required',
    componentProps: {
      placeholder: $t('page.product.inventory.check.drawer.warehousePlaceholder'),
      options: warehouseOptions,
      allowClear: true,
      showSearch: true,
      filterOption: (input: string, option: any) =>
        (option?.label ?? '').toLowerCase().includes(input.toLowerCase()),
    },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: $t('page.product.inventory.check.field.remark'),
    componentProps: {
      placeholder: $t('page.product.inventory.check.drawer.remarkPlaceholder'),
      allowClear: true,
      rows: 2,
    },
    formItemClass: 'col-span-2',
  },
  {
    component: 'Divider',
    fieldName: '_div2',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => $t('page.product.inventory.check.drawer.checkTypeDesc') }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'checkTypeDesc',
    label: $t('page.product.inventory.check.drawer.desc'),
    componentProps: {
      readOnly: true,
      placeholder: $t('page.product.inventory.check.drawer.descPlaceholder'),
    },
    formItemClass: 'col-span-2',
  },
  {
    component: 'Divider',
    fieldName: '_div3',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => $t('page.product.inventory.check.drawer.detail') }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: '_itemsHint',
    label: $t('page.product.inventory.check.drawer.detailHint'),
    defaultValue: $t('page.product.inventory.check.drawer.detailHintText'),
    componentProps: {
      readOnly: true,
    },
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

      // 剔除辅助字段
      const { _div1, _div2, _div3, _itemsHint, checkTypeDesc, ...rest } =
        values as any;

      const data = {
        ...rest,
        checkType: Number(rest.checkType),
        warehouseId: rest.warehouseId ? Number(rest.warehouseId) : undefined,
      };

      if (drawerData.value.create) {
        await createCheckApi(data);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateCheckApi({ ...data, id: drawerData.value.row.id });
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
  async onOpenChange(isOpen: boolean) {
    if (isOpen) {
      isFullscreen.value = false;
      drawerData.value =
        drawerApi.getData<{ create: boolean; row?: any }>() || {
          create: true,
        };
      await mainFormApi.resetForm();
      confirmLoading.value = false;
      loadWarehouseOptions();
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      } else {
        // 新建时根据默认盘点类型初始化说明
        updateCheckTypeDesc(1);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getCheckInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const num = (v: any) =>
      v === null || v === undefined ? undefined : Number(v);

    const checkType = num(data.checkType) ?? 1;
    mainFormApi.setValues({
      checkType,
      warehouseId: data.warehouseId ? num(data.warehouseId) : undefined,
      remark: data.remark,
    });
    // 根据详情中的盘点类型回填说明
    updateCheckTypeDesc(checkType);
  } catch (e) {
    console.error('[库存盘点] 加载详情失败:', e);
  }
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="drawerData.create ? $t('page.product.inventory.check.drawer.createTitle') : $t('page.product.inventory.check.drawer.editTitle')"
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? $t('page.product.inventory.check.drawer.restore') : $t('page.product.inventory.check.drawer.fullscreen')">
        <button
          type="button"
          class="check-drawer__fs-btn"
          @click="toggleFullscreen"
        >
          <svg
            v-if="!isFullscreen"
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </button>
      </Tooltip>
    </template>

    <div class="check-drawer__body">
      <MainForm />
    </div>
  </Drawer>
</template>

<style>
.check-drawer {
  width: 75vw !important;
}

.check-drawer--fullscreen {
  width: 100vw !important;
}

.check-drawer__fs-btn {
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

.check-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

.check-drawer__body {
  padding: 0 8px;
  overflow-y: auto;
  height: calc(100vh - 150px);
}

.check-drawer__body .ant-divider {
  margin: 12px 0 8px;
}

.check-drawer__body .ant-divider-inner-text {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}
</style>
