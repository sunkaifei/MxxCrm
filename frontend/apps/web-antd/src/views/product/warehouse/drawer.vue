﻿<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import type { VbenFormSchema } from '@vben/common-ui';
import { $t } from '#/locales';
import { createWarehouseApi, getWarehouseInfoApi, updateWarehouseApi, getAdminOptionsApi } from '#/api';
import { message, Tooltip } from 'ant-design-vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

const drawerClass = computed(() => [
  'warehouse-drawer',
  { 'warehouse-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const warehouseTypeOptions = [
  { label: '原材料仓', value: 1 },
  { label: '成品仓', value: 2 },
  { label: '半成品仓', value: 3 },
  { label: '退货仓', value: 4 },
  { label: '中转仓', value: 5 },
];

const logisticsOptions = [
  { label: '顺丰速运', value: 'sf' },
  { label: '京东物流', value: 'jd' },
  { label: '中通快递', value: 'zto' },
  { label: '圆通速递', value: 'yto' },
  { label: '韵达快递', value: 'yd' },
  { label: '申通快递', value: 'sto' },
  { label: '百世快递', value: 'htky' },
  { label: '德邦物流', value: 'db' },
  { label: 'EMS', value: 'ems' },
  { label: '极兔速递', value: 'jt' },
  { label: '自提', value: 'pickup' },
];

const userOptions = ref<{ label: string; value: number }[]>([]);

async function loadUserOptions() {
  try {
    const resp = await getAdminOptionsApi();
    const list = resp?.data ?? resp ?? [];
    userOptions.value = (Array.isArray(list) ? list : []).map((u: any) => ({
      label: u.label,
      value: Number(u.value),
    }));
  } catch (e) {
    console.error('[仓库] 加载用户列表失败:', e);
  }
}

const formSchema: VbenFormSchema[] = [
  {
    component: 'Divider',
    fieldName: '_div1',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => '基本信息' }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'warehouseName',
    label: '仓库名称',
    rules: 'required',
    componentProps: { placeholder: '请输入仓库名称', allowClear: true },
    formItemClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'code',
    label: '仓库编码',
    componentProps: { placeholder: '如：WH001', allowClear: true },
  },
  {
    component: 'Select',
    fieldName: 'warehouseType',
    label: '仓库类型',
    defaultValue: 1,
    componentProps: { placeholder: '请选择仓库类型', options: warehouseTypeOptions, allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'region',
    label: '所在区域',
    componentProps: { placeholder: '如：华东区、广东省', allowClear: true },
  },
  {
    component: 'InputNumber',
    fieldName: 'areaSqm',
    label: '仓库面积(㎡)',
    componentProps: { placeholder: '0', min: 0, precision: 2, style: { width: '100%' } },
  },
  {
    component: 'Divider',
    fieldName: '_div2',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => '联系信息' }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'managerId',
    label: '负责人',
    componentProps: {
      placeholder: '搜索选择负责人',
      options: userOptions,
      allowClear: true,
      showSearch: true,
      filterOption: (input: string, option: any) => (option?.label ?? '').toLowerCase().includes(input.toLowerCase()),
    },
  },
  {
    component: 'Input',
    fieldName: 'contactPhone',
    label: '联系电话',
    componentProps: { placeholder: '请输入联系电话', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'backupPhone',
    label: '备用电话',
    componentProps: { placeholder: '请输入备用联系电话', allowClear: true },
  },
  {
    component: 'Select',
    fieldName: 'logisticsTypes',
    label: '支持物流',
    componentProps: {
      placeholder: '请选择支持的物流',
      options: logisticsOptions,
      mode: 'multiple',
      allowClear: true,
    },
  },
  {
    component: 'Textarea',
    fieldName: 'address',
    label: '仓库地址',
    componentProps: { placeholder: '请输入详细地址', allowClear: true, rows: 2 },
    formItemClass: 'col-span-2',
  },
  {
    component: 'Divider',
    fieldName: '_div3',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => '其他设置' }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'RadioGroup',
    fieldName: 'isActive',
    label: '状态',
    defaultValue: true,
    componentProps: {
      optionType: 'button',
      options: [
        { label: '启用', value: true },
        { label: '停用', value: false },
      ],
    },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: { placeholder: '请输入备注信息', allowClear: true, rows: 2 },
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

      let logisticsStr: string | undefined;
      if (Array.isArray(values.logisticsTypes)) {
        logisticsStr = values.logisticsTypes.join(',');
      }

      const data = {
        ...values,
        logisticsTypes: logisticsStr,
        isActive: values.isActive !== false,
      };

      if (drawerData.value.create) {
        await createWarehouseApi(data);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateWarehouseApi({ ...data, id: drawerData.value.row.id });
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
      loadUserOptions();
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getWarehouseInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const num = (v: any) => (v === null || v === undefined ? undefined : Number(v));

    let logisticsArr: string[] | undefined;
    if (data.logisticsTypes) {
      logisticsArr = String(data.logisticsTypes).split(',').filter(Boolean);
    }

    mainFormApi.setValues({
      warehouseName: data.warehouseName,
      code: data.code,
      warehouseType: num(data.warehouseType) ?? 1,
      region: data.region,
      areaSqm: data.areaSqm ? Number(data.areaSqm) : undefined,
      managerId: data.managerId ? num(data.managerId) : undefined,
      contactPhone: data.contactPhone,
      backupPhone: data.backupPhone,
      logisticsTypes: logisticsArr,
      address: data.address,
      isActive: data.isActive !== false,
      remark: data.remark,
    });
  } catch (e) {
    console.error('[仓库] 加载详情失败:', e);
  }
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="drawerData.create ? '新建仓库' : '编辑仓库'"
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '还原' : '最大化'">
        <button type="button" class="warehouse-drawer__fs-btn" @click="toggleFullscreen">
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

    <div class="warehouse-drawer__body">
      <MainForm />
    </div>
  </Drawer>
</template>

<style>
.warehouse-drawer {
  width: 75vw !important;
}

.warehouse-drawer--fullscreen {
  width: 100vw !important;
}

.warehouse-drawer__fs-btn {
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

.warehouse-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

.warehouse-drawer__body {
  padding: 0 8px;
  overflow-y: auto;
  height: calc(100vh - 150px);
}

.warehouse-drawer__body .ant-divider {
  margin: 12px 0 8px;
}

.warehouse-drawer__body .ant-divider-inner-text {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}
</style>
